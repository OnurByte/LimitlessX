pub mod view;
use limitless_core::Target;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sender {
    pub id: String,
    pub url: String,
    #[serde(default)]
    pub page_url: Option<String>,
    pub tab_id: Option<i32>,
    pub frame_id: u32,
    pub extension_origin: String,
}
impl Sender {
    pub fn current_url(&self) -> &str {
        self.page_url.as_deref().unwrap_or(&self.url)
    }
    fn same_document_origin(&self) -> bool {
        let (Ok(document), Ok(current)) = (
            url::Url::parse(&self.url),
            url::Url::parse(self.current_url()),
        ) else {
            return false;
        };
        document.origin() == current.origin()
    }
}
pub fn authorized(sender: &Sender, op: &str) -> bool {
    if sender.id.is_empty() || sender.frame_id != 0 {
        return false;
    }
    if let Some(path) = sender.url.strip_prefix(&sender.extension_origin) {
        return matches!(path.split('?').next(), Some("popup.html"))
            && matches!(
                op,
                "ping"
                    | "record"
                    | "settings"
                    | "clear"
                    | "open_manual"
                    | "current"
                    | "open_current"
            );
    }
    let Ok(u) = url::Url::parse(&sender.url) else {
        return false;
    };
    sender.same_document_origin()
        && u.scheme() == "https"
        && matches!(
            u.host_str(),
            Some("x.com" | "www.x.com" | "twitter.com" | "www.twitter.com")
        )
        && sender.tab_id.is_some()
        && matches!(
            op,
            "inspect"
                | "open_detected"
                | "auto_detected"
                | "pending"
                | "read"
                | "timeline"
                | "open_post"
        )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub target: Target,
    pub source: Target,
    pub restricted: bool,
    #[serde(default)]
    pub deliberate: bool,
    #[serde(default)]
    pub pending: bool,
    pub expires: f64,
    pub tab_id: i32,
    #[serde(default)]
    pub allowed_posts: Vec<String>,
    #[serde(default)]
    pub content: Option<view::Content>,
}
impl Session {
    pub fn bound_to(&self, sender: &Sender) -> bool {
        sender.tab_id == Some(self.tab_id)
            && Target::parse(sender.current_url()).is_ok_and(|target| target == self.source)
    }
    pub fn valid(&self, target: &Target, now: f64) -> bool {
        self.target == *target && target.is_valid() && now.is_finite() && now < self.expires
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm;
