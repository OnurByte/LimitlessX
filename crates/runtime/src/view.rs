//! A typed DOM description rendered by the narrow content-script DOM adapter.
//! All product copy, view decisions and actions are constructed in Rust.
use limitless_core::{Error, Post, Profile, Target, Timeline};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Node {
    Text(String),
    Element {
        tag: String,
        attrs: Vec<(String, String)>,
        children: Vec<Node>,
        #[serde(skip_serializing_if = "Option::is_none")]
        action: Option<Value>,
    },
}
fn text(s: impl Into<String>) -> Node {
    Node::Text(s.into())
}
fn el(tag: &str, class: &str, children: Vec<Node>) -> Node {
    Node::Element {
        tag: tag.into(),
        attrs: if class.is_empty() {
            vec![]
        } else {
            vec![("class".into(), class.into())]
        },
        children,
        action: None,
    }
}
fn attr(mut node: Node, key: &str, value: impl Into<String>) -> Node {
    if let Node::Element { attrs, .. } = &mut node {
        attrs.push((key.into(), value.into()));
    }
    node
}
fn button(label: &str, action: Value) -> Node {
    let mut node = el("button", "secondary", vec![text(label)]);
    if let Node::Element { action: a, .. } = &mut node {
        *a = Some(action);
    }
    attr(node, "type", "button")
}
fn link(label: &str, url: String) -> Node {
    attr(
        attr(
            attr(el("a", "source-link", vec![text(label)]), "href", url),
            "target",
            "_blank",
        ),
        "rel",
        "noopener noreferrer",
    )
}
fn frame(children: Vec<Node>) -> Node {
    el("main", "reader x-reader", children)
}
pub fn loading(_target: &Target) -> Node {
    frame(vec![attr(
        el("p", "x-loading", vec![text("Loading public content…")]),
        "role",
        "status",
    )])
}
pub fn error(error: &Error, retry: Option<Value>) -> Node {
    let mut children = vec![el("p", "", vec![text(error.to_string())])];
    if let Some(retry) = retry {
        children.push(button("Try again", retry));
    }
    frame(vec![attr(
        el("section", "x-error", children),
        "role",
        "alert",
    )])
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Content {
    pub profile: Option<Profile>,
    pub post: Option<Post>,
    pub timeline: Option<Timeline>,
    pub profile_error: Option<String>,
    pub timeline_error: Option<String>,
    pub record_error: Option<String>,
    #[serde(default)]
    pub conversation: Option<limitless_core::Conversation>,
    #[serde(default)]
    pub replies_error: Option<String>,
}
fn svg(path: &str, class: &str) -> Node {
    attr(
        attr(
            attr(
                el("svg", class, vec![attr(el("path", "", vec![]), "d", path)]),
                "viewBox",
                "0 0 24 24",
            ),
            "fill",
            "currentColor",
        ),
        "aria-hidden",
        "true",
    )
}
fn compact(n: u64) -> String {
    let (value, suffix) = if n >= 1_000_000_000 {
        (n as f64 / 1e9, "B")
    } else if n >= 1_000_000 {
        (n as f64 / 1e6, "M")
    } else if n >= 1000 {
        (n as f64 / 1e3, "K")
    } else {
        return n.to_string();
    };
    format!(
        "{}{suffix}",
        format!("{value:.1}")
            .trim_end_matches('0')
            .trim_end_matches('.')
    )
}
fn linked_text(value: &str) -> Vec<Node> {
    value
        .split_inclusive(char::is_whitespace)
        .flat_map(|chunk| {
            let token = chunk.trim_end_matches(char::is_whitespace);
            let rest = &chunk[token.len()..];
            let url = if token.starts_with("https://") {
                url::Url::parse(token)
                    .ok()
                    .filter(|u| u.username().is_empty() && u.password().is_none())
                    .map(Into::into)
            } else if token.starts_with('@') && limitless_core::valid_handle(&token[1..]).is_some()
            {
                Some(format!("https://x.com/{}", &token[1..]))
            } else if token.starts_with('#') && token.len() > 1 {
                let mut u = url::Url::parse("https://x.com/hashtag/").unwrap();
                u.path_segments_mut().unwrap().push(&token[1..]);
                Some(u.into())
            } else {
                None
            };
            let first = if let Some(url) = url {
                attr(link(token, url), "class", "x-entity")
            } else {
                text(token)
            };
            vec![first, text(rest)]
        })
        .collect()
}
fn badge(p: &Profile) -> Option<Node> {
    p.verified.filter(|v| *v).map(|_| {
        let path = crate::icons::VERIFIED;
        attr(
            attr(
                svg(
                    path,
                    if p.verification_type.as_deref() == Some("organization") {
                        "x-verified organization"
                    } else {
                        "x-verified"
                    },
                ),
                "viewBox",
                "0 0 22 22",
            ),
            "aria-label",
            "Verified",
        )
    })
}
fn author(p: &Profile, detailed: bool) -> Node {
    let mut name = vec![attr(
        link(&p.name, format!("https://x.com/{}", p.handle)),
        "class",
        "x-author-name",
    )];
    if let Some(badge) = badge(p) {
        name.push(badge);
    }
    el(
        "div",
        if detailed {
            "x-author detailed-author"
        } else {
            "x-author"
        },
        vec![
            el("span", "x-name-line", name),
            attr(
                link(
                    &format!("@{}", p.handle),
                    format!("https://x.com/{}", p.handle),
                ),
                "class",
                "x-handle",
            ),
        ],
    )
}
fn avatar(p: &Profile, class: &str) -> Node {
    if let Some(src) = &p.avatar {
        attr(
            attr(
                attr(attr(el("img", class, vec![]), "src", src), "alt", ""),
                "crossorigin",
                "anonymous",
            ),
            "referrerpolicy",
            "no-referrer",
        )
    } else {
        attr(
            el("div", &format!("{class} avatar-placeholder"), vec![]),
            "aria-label",
            "Profile image unavailable",
        )
    }
}
fn profile(p: &Profile) -> Node {
    let mut hero = vec![];
    if let Some(banner) = &p.banner {
        hero.push(attr(
            attr(
                attr(el("img", "x-banner", vec![]), "src", banner),
                "alt",
                "",
            ),
            "referrerpolicy",
            "no-referrer",
        ));
    }
    hero.push(avatar(p, "x-profile-avatar"));
    let mut name = vec![text(&p.name)];
    if let Some(b) = badge(p) {
        name.push(b);
    }
    let mut info = vec![
        el(
            "div",
            "x-profile-actions",
            vec![attr(
                link(
                    "Follow",
                    format!("https://x.com/intent/follow?screen_name={}", p.handle),
                ),
                "class",
                "x-follow",
            )],
        ),
        el(
            "div",
            "x-profile-names",
            vec![
                el("h2", "", name),
                el("p", "x-handle", vec![text(format!("@{}", p.handle))]),
            ],
        ),
    ];
    if let Some(bio) = &p.biography {
        info.push(el("div", "biography", linked_text(bio)));
    }
    let mut details = vec![];
    if let Some(location) = &p.location {
        details.push(el("span", "x-profile-location", vec![text(location)]));
    }
    if let Some(website) = &p.website {
        let label = url::Url::parse(website)
            .ok()
            .and_then(|u| u.host_str().map(str::to_owned))
            .unwrap_or_else(|| website.clone());
        details.push(attr(
            link(&label, website.clone()),
            "class",
            "x-profile-website",
        ));
    }
    if let Some(joined) = &p.joined {
        let fields: Vec<_> = joined.split_whitespace().collect();
        let label = if fields.len() >= 6 {
            format!("Joined {} {}", fields[1], fields[5])
        } else {
            format!("Joined {joined}")
        };
        details.push(el("span", "x-profile-joined", vec![text(label)]));
    }
    if !details.is_empty() {
        info.push(el("div", "x-profile-details", details));
    }
    let mut counts = vec![];
    for (number, label, path) in [
        (p.following, "Following", "following"),
        (p.followers, "Followers", "verified_followers"),
    ] {
        if let Some(n) = number {
            counts.push(
                attr(
                    link("", format!("https://x.com/{}/{path}", p.handle)),
                    "class",
                    "x-profile-count",
                )
                .with_children(vec![
                    el("strong", "", vec![text(compact(n))]),
                    text(format!(" {label}")),
                ]),
            );
        }
    }
    if !counts.is_empty() {
        info.push(el("div", "x-profile-counts", counts));
    }
    el(
        "section",
        "x-profile",
        vec![
            el("div", "x-profile-hero", hero),
            el("div", "x-profile-info", info),
        ],
    )
}
impl Node {
    fn with_children(mut self, next: Vec<Node>) -> Self {
        if let Node::Element { children, .. } = &mut self {
            *children = next;
        }
        self
    }
}
fn short_date(value: &str) -> String {
    let fields: Vec<_> = value.split_whitespace().collect();
    if fields.len() == 6 {
        format!("{} {}, {}", fields[1], fields[2], fields[5])
    } else {
        value.into()
    }
}
fn date_label(value: &str, detailed: bool) -> String {
    #[cfg(target_arch = "wasm32")]
    {
        let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_str(value));
        if date.get_time().is_finite() {
            let months = [
                "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
            ];
            let month = months[date.get_month() as usize];
            let day = date.get_date();
            let year = date.get_full_year();
            if detailed {
                let h = date.get_hours();
                return format!(
                    "{}:{:02} {} · {month} {day}, {year}",
                    if h.is_multiple_of(12) { 12 } else { h % 12 },
                    date.get_minutes(),
                    if h >= 12 { "PM" } else { "AM" }
                );
            }
            let elapsed = (js_sys::Date::now() - date.get_time()) / 1000.;
            if (0.0..60.).contains(&elapsed) {
                return format!("{}s", elapsed.max(1.) as u64);
            }
            if (60.0..3600.).contains(&elapsed) {
                return format!("{}m", (elapsed / 60.) as u64);
            }
            if (3600.0..86400.).contains(&elapsed) {
                return format!("{}h", (elapsed / 3600.) as u64);
            }
            let current_year = js_sys::Date::new_0().get_full_year();
            return if year == current_year {
                format!("{month} {day}")
            } else {
                format!("{month} {day}, {year}")
            };
        }
    }
    let _ = detailed;
    short_date(value)
}
fn media(p: &Post) -> Node {
    el(
        "div",
        "media-grid",
        p.media
            .iter()
            .map(|m| {
                if m.kind == "photo" {
                    attr(
                        attr(
                            attr(
                                attr(
                                    attr(el("img", "post-media", vec![]), "src", &m.url),
                                    "alt",
                                    m.alt.as_deref().unwrap_or("Image from this post"),
                                ),
                                "loading",
                                "lazy",
                            ),
                            "crossorigin",
                            "anonymous",
                        ),
                        "referrerpolicy",
                        "no-referrer",
                    )
                } else {
                    let mut n = attr(
                        attr(
                            attr(
                                attr(el("video", "post-media", vec![]), "src", &m.url),
                                "controls",
                                "",
                            ),
                            "preload",
                            "none",
                        ),
                        "crossorigin",
                        "anonymous",
                    );
                    if let Some(preview) = &m.preview {
                        n = attr(n, "poster", preview);
                    }
                    n
                }
            })
            .collect(),
    )
}
fn metrics(p: &Post, detailed: bool) -> Node {
    let mut items = vec![];
    let data = if detailed {
        vec![
            ("Reply", crate::icons::REPLY, p.metrics.replies),
            ("Repost", crate::icons::REPOST, p.metrics.reposts),
            ("Like", crate::icons::LIKE, p.metrics.likes),
            ("Bookmark", crate::icons::BOOKMARK, p.metrics.bookmarks),
            ("Share", crate::icons::SHARE, None),
        ]
    } else {
        vec![
            ("Reply", crate::icons::REPLY, p.metrics.replies),
            ("Repost", crate::icons::REPOST, p.metrics.reposts),
            ("Like", crate::icons::LIKE, p.metrics.likes),
            ("Views", crate::icons::VIEWS, p.metrics.views),
            ("Bookmark", crate::icons::BOOKMARK, None),
            ("Share", crate::icons::SHARE, None),
        ]
    };
    for (label, path, count) in data {
        let mut children = vec![svg(path, "x-metric-icon")];
        if let Some(number) = count.filter(|n| *n > 0) {
            children.push(el("span", "x-metric-value", vec![text(compact(number))]));
        }
        let href = match label {
            "Reply" => format!("https://x.com/intent/post?in_reply_to={}", p.id),
            "Repost" => format!("https://x.com/intent/retweet?tweet_id={}", p.id),
            "Like" => format!("https://x.com/intent/like?tweet_id={}", p.id),
            _ => format!("https://x.com/{}/status/{}", p.author.handle, p.id),
        };
        items.push(
            attr(
                attr(
                    link("", href),
                    "class",
                    format!("x-metric metric-{}", label.to_ascii_lowercase()),
                ),
                "aria-label",
                format!(
                    "{label}{} · Open on X",
                    count.map(|n| format!(" {n}")).unwrap_or_default()
                ),
            )
            .with_children(children),
        );
    }
    attr(
        el(
            "div",
            if detailed {
                "x-metrics detailed-metrics"
            } else {
                "x-metrics"
            },
            items,
        ),
        "role",
        "group",
    )
}
fn quote(p: &Post, depth: u8) -> Node {
    let mut children = vec![
        el(
            "div",
            "x-quote-author",
            vec![
                avatar(&p.author, "x-quote-avatar"),
                author(&p.author, false),
            ],
        ),
        el("div", "x-quote-text", linked_text(&p.text)),
        media(p),
    ];
    if depth < 3
        && let Some(q) = &p.quote
    {
        children.push(quote(q, depth + 1));
    }
    attr(
        el("section", "x-quote", children),
        "aria-label",
        "Quoted post",
    )
}
fn post(p: &Post, token: &str, detailed: bool) -> Node {
    let mut headline = vec![author(&p.author, detailed)];
    if !detailed && let Some(date) = &p.published {
        headline.push(el(
            "time",
            "x-timeline-time",
            vec![text(format!("· {}", date_label(date, false)))],
        ));
    }
    headline.push(attr(
        link(
            "…",
            format!("https://x.com/{}/status/{}", p.author.handle, p.id),
        ),
        "class",
        "x-post-more",
    ));
    let header = el("div", "post-heading", headline);
    let mut body = vec![];
    let mut body_text = el("p", "post-text", linked_text(&p.text));
    if !detailed {
        if let Node::Element { action, .. } = &mut body_text {
            *action = Some(json!({"op":"open_post","session":token,"id":p.id}));
        }
        body_text = attr(attr(body_text, "role", "link"), "tabindex", "0");
    }
    body.push(body_text);
    body.push(media(p));
    if let Some(q) = &p.quote {
        body.push(quote(q, 0));
    }
    if detailed {
        let mut meta = vec![];
        if let Some(date) = &p.published {
            meta.push(el("time", "", vec![text(date_label(date, true))]));
        }
        if let Some(views) = p.metrics.views {
            meta.push(text(" · "));
            meta.push(el("strong", "", vec![text(compact(views))]));
            meta.push(text(" Views"));
        }
        if !meta.is_empty() {
            body.push(el("div", "x-post-meta", meta));
        }
        body.push(metrics(p, true));
        let mut children = vec![el(
            "div",
            "x-detail-author",
            vec![avatar(&p.author, "post-avatar"), header],
        )];
        children.extend(body);
        el("article", "post-card detailed-post", children)
    } else {
        body.insert(0, header);
        body.push(metrics(p, false));
        el(
            "article",
            "post-card",
            vec![
                avatar(&p.author, "post-avatar"),
                el("div", "post-content", body),
            ],
        )
    }
}
pub fn reader(content: &Content, target: &Target, token: &str, _restricted: bool) -> Node {
    let mut children = vec![];
    if let Some(p) = &content.profile {
        children.push(profile(p));
        children.push(attr(
            el(
                "div",
                "x-profile-tabs",
                vec![
                    el("span", "x-tab active", vec![text("Posts")]),
                    attr(
                        link(
                            "Replies",
                            format!("{}/with_replies", target.canonical_url()),
                        ),
                        "class",
                        "x-tab",
                    ),
                    attr(
                        link("Media", format!("{}/media", target.canonical_url())),
                        "class",
                        "x-tab",
                    ),
                ],
            ),
            "role",
            "navigation",
        ));
    }
    for (label, error) in [
        ("Profile lookup", &content.profile_error),
        ("Posts", &content.timeline_error),
        ("The Record could not update", &content.record_error),
    ] {
        if let Some(error) = error {
            children.push(el("p", "notice", vec![text(format!("{label}: {error}"))]));
        }
    }
    if let Some(p) = &content.post {
        children.push(post(p, token, true));
    }
    if let Some(t) = &content.timeline {
        if t.posts.is_empty() {
            children.push(el(
                "p",
                "empty-state",
                vec![text("No public posts were returned by the provider.")],
            ));
        }
        for p in &t.posts {
            children.push(post(p, token, false));
        }
        if let Some(cursor) = &t.next_cursor {
            children.push(button(
                "Load more posts",
                json!({"op":"timeline","session":token,"target":target,"cursor":cursor}),
            ));
        }
    }
    if let Some(c) = &content.conversation {
        for p in &c.thread {
            children.push(post(p, token, false));
        }
        children.push(el("div", "x-replies-heading", vec![text("Most relevant")]));
        for p in &c.replies {
            children.push(post(p, token, false));
        }
        if c.unavailable_replies > 0 {
            children.push(el(
                "p",
                "notice",
                vec![text("Some replies are unavailable from the source.")],
            ));
        }
        if let Some(cursor) = &c.next_cursor {
            children.push(attr(
                button(
                    "Show more replies",
                    json!({"op":"replies","session":token,"target":target,"cursor":cursor}),
                ),
                "class",
                "x-more-replies",
            ));
        }
    }
    if let Some(error) = &content.replies_error {
        children.push(el(
            "p",
            "notice",
            vec![text(format!("Replies unavailable: {error}"))],
        ));
    }
    children.push(el(
        "p",
        "attribution",
        vec![
            text("LimitlessX · via "),
            link("FxTwitter", "https://github.com/FxEmbed/FxEmbed".into()),
        ],
    ));
    frame(children)
}
