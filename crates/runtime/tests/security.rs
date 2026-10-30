use limitless_core::Target;
use limitless_runtime::{Sender, Session, authorized};
fn sender(url: &str) -> Sender {
    Sender {
        id: "abc".into(),
        url: url.into(),
        page_url: None,
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
        source: target.clone(),
        pending: false,
        restricted: true,
        deliberate: true,
        created: 0.,
        expires: 10000.,
        tab_id: 4,
        allowed_posts: vec![],
        content: None,
    };
    assert!(s.valid(&target, 9999.));
    assert!(!s.valid(&target, 10000.));
    assert!(!s.valid(&Target::parse("@other").unwrap(), 9999.));
}

#[test]
fn inline_session_cannot_be_moved_to_another_x_tab_or_route() {
    let target = Target::parse("@test").unwrap();
    let s = Session {
        target: target.clone(),
        source: target,
        restricted: true,
        deliberate: true,
        pending: false,
        created: 0.,
        expires: 10000.,
        tab_id: 4,
        allowed_posts: vec![],
        content: None,
    };
    assert!(s.bound_to(&sender("https://x.com/test")));
    assert!(!s.bound_to(&Sender {
        tab_id: Some(5),
        ..sender("https://x.com/test")
    }));
    assert!(!s.bound_to(&sender("https://x.com/other")));
    assert!(!authorized(
        &sender("chrome-extension://abc/reader.html"),
        "read"
    ));
    assert!(!authorized(
        &sender("chrome-extension://abc/options.html"),
        "record"
    ));
}

#[test]
fn spa_route_is_current_but_cannot_change_its_document_origin() {
    let mut origin = sender("https://x.com/jack/status/20");
    origin.page_url = Some("https://x.com/jack/status/21".into());
    assert!(authorized(&origin, "auto_detected"));
    assert_eq!(origin.current_url(), "https://x.com/jack/status/21");
    let session = Session {
        target: Target::parse("https://x.com/jack/status/20").unwrap(),
        source: Target::parse("https://x.com/jack/status/20").unwrap(),
        restricted: true,
        deliberate: false,
        pending: false,
        created: 0.,
        expires: 10000.,
        tab_id: 4,
        allowed_posts: vec![],
        content: None,
    };
    assert!(!session.bound_to(&origin));
    origin.page_url = Some("https://evil.test/jack/status/21".into());
    assert!(!authorized(&origin, "auto_detected"));
    origin.page_url = Some("chrome-extension://abc/popup.html".into());
    assert!(!authorized(&origin, "record"));
}
