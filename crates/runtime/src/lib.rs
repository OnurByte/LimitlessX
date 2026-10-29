use limitless_core::Target;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sender {
    pub id: String,
    pub url: String,
    pub tab_id: Option<i32>,
    pub frame_id: u32,
    pub extension_origin: String,
}
pub fn authorized(sender: &Sender, op: &str) -> bool {
    if sender.id.is_empty() || sender.frame_id != 0 {
        return false;
    }
    if let Some(path) = sender.url.strip_prefix(&sender.extension_origin) {
        return matches!(
            path.split('?').next(),
            Some("popup.html" | "options.html" | "reader.html")
        ) && matches!(
            op,
            "ping"
                | "record"
                | "settings"
                | "clear"
                | "open_manual"
                | "read"
                | "timeline"
                | "open_post"
                | "current"
                | "open_current"
        );
    }
    let Ok(u) = url::Url::parse(&sender.url) else {
        return false;
    };
    u.scheme() == "https"
        && matches!(
            u.host_str(),
            Some("x.com" | "www.x.com" | "twitter.com" | "www.twitter.com")
        )
        && sender.tab_id.is_some()
        && matches!(op, "inspect" | "open_detected")
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub target: Target,
    pub restricted: bool,
    pub expires: f64,
    pub tab_id: i32,
    #[serde(default)]
    pub allowed_posts: Vec<String>,
}
impl Session {
    pub fn valid(&self, target: &Target, now: f64) -> bool {
        self.target == *target && target.is_valid() && now.is_finite() && now < self.expires
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm;
