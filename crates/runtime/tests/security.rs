use limitless_core::Target;
use limitless_runtime::{Sender, Session, authorized};
fn sender(url: &str) -> Sender {
    Sender {
        id: "abc".into(),
        url: url.into(),
        tab_id: Some(4),
        frame_id: 0,
        extension_origin: "chrome-extension://abc/".into(),
    }
}
#[test]
fn page_cannot_call_privileged_operations() {
    assert!(authorized(
        &sender("chrome-extension://abc/popup.html"),
        "record"
    ));
    assert!(!authorized(&sender("https://x.com/test"), "record"));
    assert!(authorized(&sender("https://x.com/test"), "inspect"));
    assert!(!authorized(&sender("https://evil.test"), "inspect"));
    assert!(!authorized(
        &Sender {
            frame_id: 2,
            ..sender("https://x.com/test")
        },
        "inspect"
    ));
    assert!(!authorized(
        &sender("chrome-extension://abc/reader.html.fake"),
        "record"
    ));
}
#[test]
fn sessions_expire_and_bind_to_exact_target() {
    let target = Target::parse("@test").unwrap();
    let s = Session {
        target: target.clone(),
        restricted: true,
        expires: 10000.,
        tab_id: 4,
        allowed_posts: vec![],
    };
    assert!(s.valid(&target, 9999.));
    assert!(!s.valid(&target, 10000.));
    assert!(!s.valid(&Target::parse("@other").unwrap(), 9999.));
}
