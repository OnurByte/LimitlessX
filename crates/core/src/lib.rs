//! Provider-independent domain and security boundaries. No browser APIs here.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Target {
    Profile { handle: String },
    Post { id: String },
}
impl Target {
    pub fn parse(input: &str) -> Result<Self, Error> {
        let input = input.trim();
        if input.len() > 2048 || input.chars().any(char::is_control) {
            return Err(Error::InvalidTarget);
        }
        if !input.contains("://") && !input.contains('/') {
            return valid_handle(input.trim_start_matches('@'))
                .map(|handle| Self::Profile { handle })
                .ok_or(Error::InvalidTarget);
        }
        let url = Url::parse(input).map_err(|_| Error::InvalidTarget)?;
        if url.scheme() != "https"
            || !url.username().is_empty()
            || url.password().is_some()
            || url.port().is_some()
            || !matches!(
                url.host_str(),
                Some(
                    "x.com"
                        | "www.x.com"
                        | "twitter.com"
                        | "www.twitter.com"
                        | "mobile.twitter.com"
                )
            )
        {
            return Err(Error::InvalidTarget);
        }
        let parts: Vec<_> = url
            .path()
            .trim_end_matches('/')
            .split('/')
            .skip(1)
            .collect();
        match parts.as_slice() {
            [handle] | [handle, "with_replies" | "reposts" | "media"] => valid_handle(handle)
                .map(|handle| Self::Profile { handle })
                .ok_or(Error::InvalidTarget),
            [handle, "status", id]
                if (*handle == "i" || valid_handle(handle).is_some()) && valid_id(id) =>
            {
                Ok(Self::Post { id: (*id).into() })
            }
            _ => Err(Error::InvalidTarget),
        }
    }
    pub fn canonical_url(&self) -> String {
        match self {
            Self::Profile { handle } => format!("https://x.com/{handle}"),
            Self::Post { id } => format!("https://x.com/i/status/{id}"),
        }
    }
    pub fn label(&self) -> String {
        match self {
            Self::Profile { handle } => format!("@{handle}"),
            Self::Post { id } => format!("Post {id}"),
        }
    }
    pub fn is_valid(&self) -> bool {
        match self {
            Self::Profile { handle } => valid_handle(handle).as_ref() == Some(handle),
            Self::Post { id } => valid_id(id),
        }
    }
}
pub fn valid_handle(handle: &str) -> Option<String> {
    let h = handle.to_ascii_lowercase();
    if h.is_empty()
        || h.len() > 15
        || !h.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_')
        || matches!(
            h.as_str(),
            "home"
                | "explore"
                | "search"
                | "notifications"
                | "messages"
                | "settings"
                | "i"
                | "intent"
                | "compose"
                | "login"
                | "logout"
                | "signup"
                | "tos"
                | "privacy"
                | "share"
        )
    {
        None
    } else {
        Some(h)
    }
}
pub fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 20
        && id.bytes().all(|c| c.is_ascii_digit())
        && id.parse::<u64>().is_ok_and(|n| n > 0)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Notice {
    pub text: String,
    pub policy_url: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Restriction {
    Regional,
}
impl Restriction {
    pub fn detect(target: &Target, notice: &Notice) -> Option<Self> {
        if notice.text.len() > 1600 {
            return None;
        }
        let u = Url::parse(&notice.policy_url).ok()?;
        if u.scheme() != "https"
            || !matches!(u.host_str(), Some("help.x.com" | "help.twitter.com"))
            || !u.username().is_empty()
            || u.port().is_some()
            || !u
                .path()
                .ends_with("/rules-and-policies/post-withheld-by-country")
        {
            return None;
        }
        let t = notice
            .text
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase();
        let t = t
            .strip_prefix("account withheld")
            .unwrap_or(&t)
            .trim_start();
        let phrase = match target {
            Target::Profile { handle } => {
                let prefix = format!("@{handle}'s account has been withheld in ");
                let alternate = format!("@{handle} has been withheld in ");
                t.starts_with(&prefix)
                    || t.starts_with(&alternate)
                    || (t.starts_with(&format!("@{handle}"))
                        && t.contains("hesab")
                        && t.contains("erişime engellen")
                        && (t.contains("ülke") || t.contains("türkiye")))
            }
            Target::Post { .. } => {
                t.starts_with("this post has been withheld in ")
                    || t.starts_with("this tweet has been withheld in ")
                    || ((t.starts_with("bu gönderi") || t.starts_with("bu tweet"))
                        && t.contains("erişime engellen"))
            }
        };
        phrase.then_some(Self::Regional)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
#[serde(rename_all = "snake_case")]
pub enum Error {
    #[error("Enter an X profile or post URL, or an account handle.")]
    InvalidTarget,
    #[error("This account is protected. Private content is unavailable.")]
    Protected,
    #[error("The provider reports that this account is suspended.")]
    Suspended,
    #[error("The provider reports that this post was deleted.")]
    Deleted,
    #[error("The provider could not find this content.")]
    NotFound,
    #[error("This content is unavailable from the provider.")]
    Unavailable,
    #[error("FxTwitter is receiving too many requests. Try again later.")]
    RateLimited,
    #[error("FxTwitter returned an unexpected response.")]
    InvalidResponse,
    #[error("The request timed out. Try again.")]
    Timeout,
    #[error("Could not reach FxTwitter. Check your connection and try again.")]
    Network,
    #[error("This reader session expired. Open the content again.")]
    Expired,
    #[error("This action is not allowed from this page.")]
    Unauthorized,
    #[error("Local storage is unavailable. Try reopening the extension.")]
    Storage,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Profile {
    pub id: String,
    pub handle: String,
    pub name: String,
    pub biography: Option<String>,
    pub avatar: Option<String>,
    #[serde(default)]
    pub banner: Option<String>,
    #[serde(default)]
    pub location: Option<String>,
    #[serde(default)]
    pub website: Option<String>,
    #[serde(default)]
    pub joined: Option<String>,
    #[serde(default)]
    pub followers: Option<u64>,
    #[serde(default)]
    pub following: Option<u64>,
    #[serde(default)]
    pub statuses: Option<u64>,
    #[serde(default)]
    pub verified: Option<bool>,
    #[serde(default)]
    pub verification_type: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Media {
    pub kind: String,
    pub url: String,
    pub preview: Option<String>,
    pub alt: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Post {
    pub id: String,
    pub text: String,
    pub author: Profile,
    pub published: Option<String>,
    pub media: Vec<Media>,
    #[serde(default)]
    pub metrics: Metrics,
    #[serde(default)]
    pub quote: Option<Box<Post>>,
    #[serde(default)]
    pub replying_to: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Metrics {
    pub replies: Option<u64>,
    pub reposts: Option<u64>,
    pub likes: Option<u64>,
    pub views: Option<u64>,
    pub bookmarks: Option<u64>,
    pub quotes: Option<u64>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub post: Post,
    pub thread: Vec<Post>,
    pub replies: Vec<Post>,
    pub next_cursor: Option<String>,
    pub unavailable_replies: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timeline {
    pub posts: Vec<Post>,
    pub next_cursor: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Record {
    pub profiles: u32,
    pub posts: u32,
    pub enabled: bool,
    pub since: f64,
}

pub fn safe_media_url(raw: &str) -> Option<String> {
    let u = Url::parse(raw).ok()?;
    (u.scheme() == "https"
        && u.username().is_empty()
        && u.password().is_none()
        && u.port().is_none()
        && matches!(u.host_str(), Some("pbs.twimg.com" | "video.twimg.com")))
    .then(|| u.to_string())
}
/// Domain-separated salted identifiers. Salt never leaves the installation.
pub fn record_key(salt: &[u8], kind: &str, id: &str) -> String {
    let mut hash = Sha256::new();
    hash.update(salt);
    hash.update([0]);
    hash.update(kind);
    hash.update([0]);
    hash.update(id);
    format!("{:x}", hash.finalize())
}
