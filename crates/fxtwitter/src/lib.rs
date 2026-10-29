//! FxTwitter v2 wire types. Unknown fields are ignored; required identity fields are not.
use limitless_core::{
    Error, Media, Post, Profile, Target, Timeline, safe_media_url, valid_handle, valid_id,
};
use serde::Deserialize;

pub fn endpoint(target: &Target, timeline: bool, cursor: Option<&str>) -> Result<String, Error> {
    if !target.is_valid() {
        return Err(Error::InvalidTarget);
    }
    let path = match target {
        Target::Profile { handle } => format!(
            "profile/{handle}{}",
            if timeline { "/statuses" } else { "" }
        ),
        Target::Post { id } => format!("status/{id}"),
    };
    let mut url = url::Url::parse(&format!("https://api.fxtwitter.com/2/{path}")).unwrap();
    if let Some(cursor) = cursor {
        if cursor.len() > 4096 {
            return Err(Error::InvalidResponse);
        }
        url.query_pairs_mut().append_pair("cursor", cursor);
    }
    Ok(url.into())
}
#[derive(Deserialize)]
struct Envelope {
    code: u16,
    reason: Option<String>,
}
fn error_reason(reason: &str) -> Error {
    match reason {
        "suspended" => Error::Suspended,
        "deleted" => Error::Deleted,
        "private" | "protected" => Error::Protected,
        "not_found" => Error::NotFound,
        _ => Error::Unavailable,
    }
}
fn check(http: u16, body: &str) -> Result<(), Error> {
    if http == 429 {
        return Err(Error::RateLimited);
    }
    if http >= 500 {
        return Err(Error::Unavailable);
    }
    if body.len() > 4 * 1024 * 1024 {
        return Err(Error::InvalidResponse);
    }
    let e: Envelope = serde_json::from_str(body).map_err(|_| Error::InvalidResponse)?;
    if let Some(reason) = e.reason {
        return Err(error_reason(&reason));
    }
    for code in [http, e.code] {
        match code {
            200 => {}
            404 => return Err(Error::NotFound),
            429 => return Err(Error::RateLimited),
            _ => return Err(Error::Unavailable),
        }
    }
    Ok(())
}
#[derive(Deserialize)]
struct RawProfile {
    #[serde(rename = "type")]
    kind: String,
    id: String,
    screen_name: String,
    name: String,
    protected: bool,
    description: Option<String>,
    avatar_url: Option<String>,
    banner_url: Option<String>,
    location: Option<String>,
    joined: Option<String>,
    followers: Option<u64>,
    following: Option<u64>,
    website: Option<RawWebsite>,
    verification: Option<RawVerification>,
}
#[derive(Deserialize)]
struct RawWebsite {
    url: String,
}
#[derive(Deserialize)]
struct RawVerification {
    verified: bool,
    #[serde(rename = "type")]
    kind: Option<String>,
}
impl RawProfile {
    fn normalize(self) -> Result<Profile, Error> {
        if self.protected {
            return Err(Error::Protected);
        }
        if self.kind != "profile"
            || !valid_id(&self.id)
            || self.name.len() > 4096
            || self.description.as_ref().is_some_and(|x| x.len() > 32768)
        {
            return Err(Error::InvalidResponse);
        }
        valid_handle(&self.screen_name).ok_or(Error::InvalidResponse)?;
        let handle = self.screen_name;
        Ok(Profile {
            id: self.id,
            handle,
            name: self.name,
            biography: self.description,
            avatar: self.avatar_url.and_then(|u| safe_media_url(&u)),
            banner: self.banner_url.and_then(|u| safe_media_url(&u)),
            location: self.location.filter(|v| !v.is_empty()),
            joined: self.joined,
            followers: self.followers,
            following: self.following,
            website: self
                .website
                .and_then(|w| url::Url::parse(&w.url).ok())
                .filter(|u| {
                    u.scheme() == "https" && u.username().is_empty() && u.password().is_none()
                })
                .map(Into::into),
            verified: self.verification.as_ref().map(|v| v.verified),
            verification_type: self.verification.and_then(|v| v.kind),
        })
    }
}
#[derive(Deserialize)]
struct ProfileResponse {
    user: RawProfile,
}
pub fn parse_profile(http: u16, body: &str, expected_handle: &str) -> Result<Profile, Error> {
    check(http, body)?;
    let r: ProfileResponse = serde_json::from_str(body).map_err(|_| Error::InvalidResponse)?;
    let p = r.user.normalize()?;
    if !p.handle.eq_ignore_ascii_case(expected_handle) {
        return Err(Error::InvalidResponse);
    }
    Ok(p)
}
#[derive(Deserialize)]
#[serde(tag = "type")]
enum RawPost {
    #[serde(rename = "status")]
    Status(Box<RawStatus>),
    #[serde(rename = "tombstone")]
    Tombstone { reason: String },
}
#[derive(Deserialize)]
struct RawStatus {
    provider: String,
    id: String,
    text: String,
    author: Box<RawProfile>,
    created_at: Option<String>,
    media: Option<RawMedia>,
    #[serde(flatten)]
    metrics: limitless_core::Metrics,
    quote: Option<Box<RawPost>>,
    replying_to: Option<RawReply>,
}
#[derive(Deserialize)]
struct RawReply {
    screen_name: Option<String>,
}
#[derive(Deserialize, Default)]
struct RawMedia {
    photos: Option<Vec<RawAttachment>>,
    videos: Option<Vec<RawAttachment>>,
}
#[derive(Deserialize)]
struct RawAttachment {
    #[serde(rename = "type")]
    kind: String,
    url: String,
    thumbnail_url: Option<String>,
    #[serde(rename = "altText")]
    alt: Option<String>,
}
impl RawPost {
    fn normalize(self) -> Result<Post, Error> {
        self.normalize_depth(0)
    }
    fn normalize_depth(self, depth: u8) -> Result<Post, Error> {
        if depth > 6 {
            return Err(Error::InvalidResponse);
        }
        match self {
            Self::Tombstone { reason } => Err(error_reason(&reason)),
            Self::Status(raw) => {
                let RawStatus {
                    provider,
                    id,
                    text,
                    author,
                    created_at,
                    media,
                    metrics,
                    quote,
                    replying_to,
                } = *raw;
                if provider != "twitter"
                    || !valid_id(&id)
                    || text.len() > 256 * 1024
                    || created_at.as_ref().is_some_and(|x| x.len() > 128)
                {
                    return Err(Error::InvalidResponse);
                }
                let raw = media.unwrap_or_default();
                let media = raw
                    .photos
                    .unwrap_or_default()
                    .into_iter()
                    .chain(raw.videos.unwrap_or_default())
                    .take(16)
                    .filter_map(|m| {
                        let kind = match m.kind.as_str() {
                            "photo" => "photo",
                            "video" | "gif" => "video",
                            _ => return None,
                        };
                        Some(Media {
                            kind: kind.into(),
                            url: safe_media_url(&m.url)?,
                            preview: m.thumbnail_url.and_then(|u| safe_media_url(&u)),
                            alt: m.alt,
                        })
                    })
                    .collect();
                Ok(Post {
                    id,
                    text,
                    author: author.normalize()?,
                    published: created_at,
                    media,
                    metrics,
                    quote: quote
                        .and_then(|q| q.normalize_depth(depth + 1).ok())
                        .map(Box::new),
                    replying_to: replying_to
                        .and_then(|r| r.screen_name)
                        .filter(|h| valid_handle(h).is_some()),
                })
            }
        }
    }
}
#[derive(Deserialize)]
struct PostResponse {
    status: RawPost,
}
pub fn parse_post(http: u16, body: &str, expected_id: &str) -> Result<Post, Error> {
    check(http, body)?;
    let r: PostResponse = serde_json::from_str(body).map_err(|_| Error::InvalidResponse)?;
    let p = r.status.normalize()?;
    if p.id != expected_id {
        return Err(Error::InvalidResponse);
    }
    Ok(p)
}
#[derive(Deserialize)]
struct Cursor {
    bottom: Option<String>,
}
#[derive(Deserialize)]
struct TimelineResponse {
    results: Vec<RawPost>,
    cursor: Option<Cursor>,
}
pub fn parse_timeline(http: u16, body: &str) -> Result<Timeline, Error> {
    check(http, body)?;
    let r: TimelineResponse = serde_json::from_str(body).map_err(|_| Error::InvalidResponse)?;
    if r.results.len() > 200 {
        return Err(Error::InvalidResponse);
    }
    let posts = r
        .results
        .into_iter()
        .map(RawPost::normalize)
        .collect::<Result<_, _>>()?;
    let next_cursor = r.cursor.and_then(|c| c.bottom).filter(|s| !s.is_empty());
    if next_cursor.as_ref().is_some_and(|s| s.len() > 4096) {
        return Err(Error::InvalidResponse);
    }
    Ok(Timeline { posts, next_cursor })
}

#[derive(Deserialize)]
struct ConversationResponse {
    status: RawPost,
    thread: Option<Vec<RawPost>>,
    replies: Vec<RawPost>,
    cursor: Option<Cursor>,
}
pub fn parse_conversation(
    http: u16,
    body: &str,
    expected: &str,
) -> Result<limitless_core::Conversation, Error> {
    check(http, body)?;
    let r: ConversationResponse = serde_json::from_str(body).map_err(|_| Error::InvalidResponse)?;
    let post = r.status.normalize()?;
    if post.id != expected {
        return Err(Error::InvalidResponse);
    }
    if r.replies.len() > 200 || r.thread.as_ref().is_some_and(|t| t.len() > 200) {
        return Err(Error::InvalidResponse);
    }
    let mut unavailable = 0;
    let mut normalize = |values: Vec<RawPost>| -> Result<Vec<Post>, Error> {
        values
            .into_iter()
            .filter_map(|raw| match raw {
                RawPost::Tombstone { .. } => {
                    unavailable += 1;
                    None
                }
                post => Some(post.normalize()),
            })
            .collect()
    };
    let thread = normalize(r.thread.unwrap_or_default())?
        .into_iter()
        .filter(|p| p.id != expected)
        .collect();
    let replies = normalize(r.replies)?;
    let next_cursor = r.cursor.and_then(|c| c.bottom).filter(|c| !c.is_empty());
    if next_cursor.as_ref().is_some_and(|c| c.len() > 4096) {
        return Err(Error::InvalidResponse);
    }
    Ok(limitless_core::Conversation {
        post,
        thread,
        replies,
        next_cursor,
        unavailable_replies: unavailable,
    })
}
pub fn conversation_endpoint(id: &str, cursor: Option<&str>) -> Result<String, Error> {
    if !valid_id(id) {
        return Err(Error::InvalidTarget);
    }
    let mut url =
        url::Url::parse(&format!("https://api.fxtwitter.com/2/conversation/{id}")).unwrap();
    if let Some(cursor) = cursor {
        if cursor.len() > 4096 {
            return Err(Error::InvalidResponse);
        }
        url.query_pairs_mut().append_pair("cursor", cursor);
    }
    Ok(url.into())
}
