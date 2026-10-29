use limitless_core::{Profile, Target, Timeline};
use limitless_runtime::view::{Content, Node, reader};
fn nodes<'a>(node: &'a Node, out: &mut Vec<&'a Node>) {
    out.push(node);
    if let Node::Element { children, .. } = node {
        for child in children {
            nodes(child, out)
        }
    }
}
#[test]
fn provider_text_stays_a_text_node_and_sessions_never_become_dom_attributes() {
    let payload = "<script>steal()</script>";
    let content = Content {
        profile: Some(Profile {
            id: "12".into(),
            handle: "jack".into(),
            name: payload.into(),
            biography: Some(payload.into()),
            avatar: None,
            ..Default::default()
        }),
        timeline: Some(Timeline {
            posts: vec![],
            next_cursor: Some("next+/=".into()),
        }),
        ..Default::default()
    };
    let view = reader(
        &content,
        &Target::parse("@jack").unwrap(),
        "secret-session",
        true,
    );
    let mut all = vec![];
    nodes(&view, &mut all);
    assert!(all.iter().any(|n| matches!(n,Node::Text(s) if s==payload)));
    assert!(
        all.iter()
            .all(|n| !matches!(n,Node::Element{tag,..} if tag=="script"||tag=="iframe"))
    );
    assert!(all.iter().all(|n|!matches!(n,Node::Element{attrs,..} if attrs.iter().any(|(_,v)|v.contains("secret-session")))));
    assert!(all.iter().any(|n|matches!(n,Node::Element{action:Some(a),..} if a["op"]=="timeline"&&a["session"]=="secret-session"&&a["cursor"]=="next+/=")));
}

#[test]
fn timeline_starts_with_five_and_reveals_buffered_batches() {
    let mut content = Content {
        timeline: Some(Timeline {
            posts: (1..=12)
                .map(|id| limitless_core::Post {
                    id: id.to_string(),
                    text: format!("post {id}"),
                    ..Default::default()
                })
                .collect(),
            next_cursor: Some("next".into()),
        }),
        ..Default::default()
    };
    let target = Target::parse("@jack").unwrap();
    let count = |c: &Content| {
        let view = reader(c, &target, "token", false);
        let mut all = vec![];
        nodes(&view, &mut all);
        all.iter()
            .filter(|n| matches!(n,Node::Element{tag,..} if tag=="article"))
            .count()
    };
    assert_eq!(count(&content), 5);
    assert!(content.reveal_buffered());
    assert_eq!(count(&content), 10);
    assert!(content.reveal_buffered());
    assert_eq!(count(&content), 12);
    assert!(!content.reveal_buffered());
}
