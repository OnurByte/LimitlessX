use limitless_core::{Notice, Restriction, Target, record_key, safe_media_url};

#[test]
fn parses_supported_urls_without_leaking_queries() {
    assert_eq!(
        Target::parse("https://x.com/Some_User/status/1234567890123456789?s=20").unwrap(),
        Target::Post {
            id: "1234567890123456789".into()
        }
    );
    assert_eq!(
        Target::parse("@Some_User").unwrap(),
        Target::Profile {
            handle: "some_user".into()
        }
    );
    assert_eq!(
        Target::parse("https://twitter.com/Some_User/").unwrap(),
        Target::Profile {
            handle: "some_user".into()
        }
    );
}
#[test]
fn rejects_confused_hosts_routes_credentials_and_unicode() {
    for s in [
        "https://x.com.evil.test/foo",
        "https://evil.test/x.com/foo",
        "https://u:p@x.com/foo",
        "http://x.com/foo",
        "https://x.com:444/foo",
        "https://x.com/home",
        "https://x.com/i",
        "https://x.com/foo/followers",
        "https://x.com/foo/status/notanid",
        "a\nb",
        "@ı",
        "",
        "https://x.com/foo%2fbar",
    ] {
        assert!(Target::parse(s).is_err(), "accepted {s}");
    }
}
#[test]
fn requires_explicit_matching_notice_and_policy_link() {
    let target = Target::parse("@example").unwrap();
    let notice = Notice {
        text: "@example's account has been withheld in Türkiye in response to a legal demand."
            .into(),
        policy_url: "https://help.x.com/en/rules-and-policies/post-withheld-by-country".into(),
    };
    assert_eq!(
        Restriction::detect(&target, &notice),
        Some(Restriction::Regional)
    );
    assert_eq!(
        Restriction::detect(&Target::parse("@other").unwrap(), &notice),
        None
    );
    assert_eq!(
        Restriction::detect(
            &target,
            &Notice {
                text: "Something went wrong. Try again.".into(),
                ..notice.clone()
            }
        ),
        None
    );
    assert_eq!(
        Restriction::detect(
            &target,
            &Notice {
                policy_url: "https://evil.test/post-withheld-by-country".into(),
                ..notice
            }
        ),
        None
    );
}
#[test]
fn never_classifies_private_suspended_or_missing_as_regional() {
    let t = Target::parse("@test").unwrap();
    for text in [
        "@test Account suspended",
        "@test These posts are protected",
        "@test This account doesn’t exist",
        "@test unavailable in your country",
        "@test wrote: account has been withheld in Türkiye",
    ] {
        assert_eq!(
            Restriction::detect(
                &t,
                &Notice {
                    text: text.into(),
                    policy_url: "https://help.x.com/en/rules-and-policies/post-withheld-by-country"
                        .into()
                }
            ),
            None
        );
    }
}
#[test]
fn media_is_https_on_explicit_provider_hosts() {
    assert!(safe_media_url("https://pbs.twimg.com/media/abc.jpg").is_some());
    for s in [
        "javascript:alert(1)",
        "data:text/html,x",
        "https://evil.test/a",
        "https://pbs.twimg.com.evil.test/a",
        "http://pbs.twimg.com/a",
        "https://a@pbs.twimg.com/a",
    ] {
        assert!(safe_media_url(s).is_none());
    }
}
#[test]
fn record_is_per_install_per_kind_and_does_not_store_ids() {
    let a = record_key(&[1; 32], "profile", "422861505");
    assert_eq!(a.len(), 64);
    assert_ne!(a, record_key(&[2; 32], "profile", "422861505"));
    assert_ne!(a, record_key(&[1; 32], "post", "422861505"));
    assert!(!a.contains("422861505"));
}

#[test]
fn detects_the_actual_helium_x_account_withheld_dom_text() {
    let notice=Notice{text:"Account Withheld@ekrem_imamoglu has been withheld in TR in response to a legal demand. https://Learn more.".into(),policy_url:"https://help.x.com/en/rules-and-policies/post-withheld-by-country".into()};
    assert_eq!(
        Restriction::detect(&Target::parse("@ekrem_imamoglu").unwrap(), &notice),
        Some(Restriction::Regional)
    );
    assert_eq!(
        Restriction::detect(&Target::parse("@other").unwrap(), &notice),
        None
    );
}
