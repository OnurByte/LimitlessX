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
}
fn profile(p: &Profile) -> Node {
    let mut heading = vec![];
    if let Some(src) = &p.avatar {
        heading.push(attr(
            attr(
                attr(attr(el("img", "avatar", vec![]), "src", src), "alt", ""),
                "crossorigin",
                "anonymous",
            ),
            "referrerpolicy",
            "no-referrer",
        ));
    }
    heading.push(el(
        "div",
        "",
        vec![
            el("h2", "", vec![text(&p.name)]),
            el("p", "muted", vec![text(format!("@{}", p.handle))]),
        ],
    ));
    let mut children = vec![el("div", "profile-heading", heading)];
    if let Some(bio) = &p.biography {
        children.push(el("p", "biography", vec![text(bio)]));
    }
    el("section", "profile-card x-profile", children)
}
fn post(p: &Post, token: &str, detailed: bool) -> Node {
    let mut heading = vec![el(
        "div",
        "",
        vec![
            el("strong", "", vec![text(&p.author.name)]),
            el(
                "span",
                "muted",
                vec![text(format!(" @{}", p.author.handle))],
            ),
        ],
    )];
    if let Some(date) = &p.published {
        let parts: Vec<_> = date.split_whitespace().collect();
        let display = if parts.len() == 6
            && parts[2].parse::<u8>().is_ok()
            && parts[5].parse::<u16>().is_ok()
        {
            format!("{} {}, {}", parts[1], parts[2], parts[5])
        } else {
            date.clone()
        };
        heading.push(el("time", "", vec![text(format!("· {display}"))]));
    }
    let media = p
        .media
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
                let mut node = attr(
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
                    node = attr(node, "poster", preview);
                }
                node
            }
        })
        .collect();
    let mut actions = vec![link(
        "Original on X ↗",
        format!("https://x.com/{}/status/{}", p.author.handle, p.id),
    )];
    if !detailed {
        actions.push(button(
            "Open post",
            json!({"op":"open_post","session":token,"id":p.id}),
        ));
    }
    let body = el(
        "div",
        "post-content",
        vec![
            el("div", "post-heading", heading),
            el("p", "post-text", vec![text(&p.text)]),
            el("div", "media-grid", media),
            el("div", "post-actions", actions),
        ],
    );
    let avatar = if let Some(src) = &p.author.avatar {
        attr(
            attr(
                attr(
                    attr(el("img", "post-avatar", vec![]), "src", src),
                    "alt",
                    "",
                ),
                "crossorigin",
                "anonymous",
            ),
            "referrerpolicy",
            "no-referrer",
        )
    } else {
        attr(
            el("div", "post-avatar avatar-placeholder", vec![]),
            "aria-label",
            "Profile image unavailable",
        )
    };
    el(
        "article",
        if detailed {
            "post-card detailed-post"
        } else {
            "post-card"
        },
        vec![avatar, body],
    )
}
pub fn reader(content: &Content, target: &Target, token: &str, _restricted: bool) -> Node {
    let mut children = vec![];
    if let Some(p) = &content.profile {
        children.push(profile(p));
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
        children.push(el(
            "div",
            "feed-title",
            vec![
                el("h2", "", vec![text("Posts")]),
                el("span", "fine", vec![text("")]),
            ],
        ));
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
    children.push(el(
        "p",
        "attribution",
        vec![
            text("LimitlessX · via "),
            link("FxTwitter", "https://github.com/FxEmbed/FxEmbed".into()),
            text(". Availability can change."),
        ],
    ));
    frame(children)
}
