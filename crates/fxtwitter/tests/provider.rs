use limitless_core::Error;
use limitless_fxtwitter::{endpoint, parse_post, parse_profile, parse_timeline};
const USER: &str = r#"{"code":200,"user":{"type":"profile","id":"12","screen_name":"jack","name":"jack","description":"bio","avatar_url":"https://pbs.twimg.com/a.jpg","protected":false}}"#;
const POST: &str = r#"{"code":200,"status":{"type":"status","provider":"twitter","id":"20","text":"just setting up my twttr","author":{"type":"profile","id":"12","screen_name":"jack","name":"jack","protected":false},"created_at":"Tue Mar 21 20:50:14 +0000 2006","media":{"photos":[{"type":"photo","url":"https://pbs.twimg.com/a.jpg","altText":"a","width":1,"height":1}]}}}"#;
#[test]
fn normalizes_only_real_fields_and_keeps_text_as_text() {
    let p = parse_profile(200, USER, "jack").unwrap();
    assert_eq!(p.id, "12");
    assert_eq!(p.biography.as_deref(), Some("bio"));
    let p = parse_post(200, POST, "20").unwrap();
    assert_eq!(p.text, "just setting up my twttr");
    assert_eq!(p.author.avatar, None);
    assert_eq!(p.media[0].alt.as_deref(), Some("a"));
}
#[test]
fn cannot_substitute_wrong_content_or_provider() {
    assert!(parse_profile(200, USER, "other").is_err());
    assert!(parse_post(200, POST, "21").is_err());
    assert!(parse_post(200, &POST.replace("twitter", "mastodon"), "20").is_err());
    assert!(
        parse_post(
            200,
            &POST.replace("https://pbs.twimg.com/a.jpg", "javascript:alert(1)"),
            "20"
        )
        .unwrap()
        .media
        .is_empty()
    );
}
#[test]
fn separates_unavailability_states_and_never_infers_withholding() {
    for (body, error) in [
        (r#"{"code":404,"reason":"suspended"}"#, Error::Suspended),
        (
            r#"{"code":200,"status":{"type":"tombstone","provider":"twitter","reason":"deleted","message":"gone"}}"#,
            Error::Deleted,
        ),
        (
            r#"{"code":200,"status":{"type":"tombstone","provider":"twitter","reason":"private","message":"private"}}"#,
            Error::Protected,
        ),
        (
            r#"{"code":404,"message":"User not found"}"#,
            Error::NotFound,
        ),
        (r#"{"code":429}"#, Error::RateLimited),
    ] {
        assert_eq!(parse_post(200, body, "20").unwrap_err(), error);
    }
    assert_eq!(parse_post(503, POST, "20").unwrap_err(), Error::Unavailable);
    assert_eq!(
        parse_post(200, "not json", "20").unwrap_err(),
        Error::InvalidResponse
    );
    assert_eq!(
        parse_profile(200, &USER.replace("false", "true"), "jack").unwrap_err(),
        Error::Protected
    );
}
#[test]
fn preserves_pagination_and_does_not_invent_empty_arrays() {
    let status: serde_json::Value = serde_json::from_str(POST).unwrap();
    let timeline =
        serde_json::json!({"code":200,"results":[status["status"]],"cursor":{"bottom":"next+/="}})
            .to_string();
    let t = parse_timeline(200, &timeline).unwrap();
    assert_eq!(t.posts.len(), 1);
    assert_eq!(t.next_cursor.as_deref(), Some("next+/="));
    assert!(parse_timeline(200, r#"{"code":200}"#).is_err());
    assert!(
        parse_timeline(200, r#"{"code":200,"results":[],"cursor":{"bottom":null}}"#)
            .unwrap()
            .posts
            .is_empty()
    );
    assert!(
        endpoint(
            &limitless_core::Target::parse("@jack").unwrap(),
            true,
            Some("next+/=")
        )
        .unwrap()
        .ends_with("?cursor=next%2B%2F%3D")
    );
}
