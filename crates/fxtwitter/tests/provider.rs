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

#[test]
fn normalizes_real_profile_metadata_and_post_metrics_without_inventing_nulls() {
    let mut user: serde_json::Value = serde_json::from_str(USER).unwrap();
    user["user"]["banner_url"] = serde_json::json!("https://pbs.twimg.com/profile_banners/12/1");
    user["user"]["followers"] = serde_json::json!(9876543);
    user["user"]["following"] = serde_json::json!(820);
    user["user"]["location"] = serde_json::json!("İstanbul");
    user["user"]["joined"] = serde_json::json!("Sun Nov 27 19:45:16 +0000 2011");
    user["user"]["verification"] = serde_json::json!({"verified":true,"type":"individual"});
    let p = parse_profile(200, &user.to_string(), "jack").unwrap();
    assert_eq!(p.followers, Some(9876543));
    assert!(p.banner.is_some());
    assert_eq!(p.verified, Some(true));
    let mut status: serde_json::Value = serde_json::from_str(POST).unwrap();
    status["status"]["likes"] = serde_json::json!(0);
    status["status"]["views"] = serde_json::Value::Null;
    status["status"]["replies"] = serde_json::json!(34);
    let p = parse_post(200, &status.to_string(), "20").unwrap();
    assert_eq!(p.metrics.likes, Some(0));
    assert_eq!(p.metrics.views, None);
    assert_eq!(p.metrics.replies, Some(34));
}
#[test]
fn conversation_checks_focal_identity_and_retains_reply_cursor() {
    let status: serde_json::Value = serde_json::from_str(POST).unwrap();
    let mut reply = status["status"].clone();
    reply["id"] = serde_json::json!("21");
    reply["text"] = serde_json::json!("a real reply");
    let body=serde_json::json!({"code":200,"status":status["status"],"thread":null,"replies":[reply,{"type":"tombstone","reason":"deleted","provider":"twitter","message":"gone"}],"cursor":{"bottom":"reply+/="}}).to_string();
    let c = limitless_fxtwitter::parse_conversation(200, &body, "20").unwrap();
    assert_eq!(c.post.id, "20");
    assert_eq!(c.replies[0].id, "21");
    assert_eq!(c.next_cursor.as_deref(), Some("reply+/="));
    assert_eq!(c.unavailable_replies, 1);
    assert!(limitless_fxtwitter::parse_conversation(200, &body, "22").is_err());
    assert_eq!(
        limitless_fxtwitter::conversation_endpoint("20", Some("reply+/=")).unwrap(),
        "https://api.fxtwitter.com/2/conversation/20?cursor=reply%2B%2F%3D"
    );
}

#[test]
fn accepts_the_actual_live_v2_conversation_shape() {
    let c = limitless_fxtwitter::parse_conversation(
        200,
        include_str!("../../../tests/fixtures/public-conversation.json"),
        "2115761952937128381",
    )
    .unwrap();
    assert_eq!(c.replies.len(), 2);
    assert!(c.next_cursor.is_some());
    assert!(c.post.metrics.likes.is_some());
}

#[test]
fn identity_resolution_is_exact_and_never_uses_suggestion_counters() {
    let body = r#"{"code":200,"users":[{"id":"99","screen_name":"jackfan","protected":false},{"id":"12","screen_name":"Jack","protected":false,"followers":0}]}"#;
    assert_eq!(
        limitless_fxtwitter::parse_identity(200, body, "jack").unwrap(),
        "12"
    );
    assert!(limitless_fxtwitter::parse_identity(200, body, "other").is_err());
    assert!(limitless_fxtwitter::id_endpoint("12/evil", true, None).is_err());
    assert_eq!(
        limitless_fxtwitter::id_endpoint("12", true, Some("+/=")).unwrap(),
        "https://api.fxtwitter.com/2/profile/id:12/statuses?count=5&cursor=%2B%2F%3D"
    );
}
