use crate::{Sender, Session, authorized};
use limitless_core::{Error, Notice, Post, Profile, Restriction, Target};
use limitless_fxtwitter as provider;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::cell::Cell;
use wasm_bindgen::{JsCast, prelude::*};
use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace=limitlessBridge)]
    fn random_bytes(size: u32) -> js_sys::Uint8Array;
    #[wasm_bindgen(js_namespace=limitlessBridge,catch)]
    async fn session_get(key: &str) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(js_namespace=limitlessBridge,catch)]
    async fn session_set(key: &str, value: JsValue) -> Result<(), JsValue>;
    #[wasm_bindgen(js_namespace=limitlessBridge,catch)]
    async fn session_remove(keys: JsValue) -> Result<(), JsValue>;
    #[wasm_bindgen(js_namespace=limitlessBridge,catch)]
    async fn session_all() -> Result<JsValue, JsValue>;
    #[wasm_bindgen(js_namespace=limitlessBridge,catch)]
    async fn new_tab() -> Result<i32, JsValue>;
    #[wasm_bindgen(js_namespace=limitlessBridge,catch)]
    async fn navigate_tab(tab_id: i32, url: &str) -> Result<(), JsValue>;
    #[wasm_bindgen(js_namespace=limitlessBridge,catch)]
    async fn show_in_tab(tab_id: i32, context: JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(js_namespace=limitlessBridge,catch)]
    async fn active_context() -> Result<JsValue, JsValue>;
    #[wasm_bindgen(js_namespace=limitlessBridge)]
    fn schedule_abort(controller: &web_sys::AbortController, ms: u32) -> i32;
    #[wasm_bindgen(js_namespace=limitlessBridge)]
    fn cancel_timer(id: i32);
    #[wasm_bindgen(js_namespace=globalThis,js_name=fetch)]
    fn global_fetch(request: &web_sys::Request) -> js_sys::Promise;
}
fn now() -> f64 {
    js_sys::Date::now()
}
fn js<T: Serialize>(value: &T) -> JsValue {
    value
        .serialize(&serde_wasm_bindgen::Serializer::json_compatible())
        .unwrap_or(JsValue::NULL)
}
fn decode<T: serde::de::DeserializeOwned>(value: JsValue) -> Result<T, Error> {
    serde_wasm_bindgen::from_value(value).map_err(|_| Error::InvalidResponse)
}
#[derive(Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
enum Request {
    Ping,
    Record,
    Clear,
    Settings {
        enabled: bool,
    },
    Inspect {
        notices: Vec<Notice>,
    },
    OpenDetected {
        notices: Vec<Notice>,
    },
    AutoDetected {
        notices: Vec<Notice>,
    },
    Pending,
    Current,
    OpenCurrent,
    OpenManual {
        input: String,
    },
    Read {
        session: String,
        target: Target,
    },
    Timeline {
        session: String,
        target: Target,
        cursor: Option<String>,
    },
    OpenPost {
        session: String,
        id: String,
    },
    Replies {
        session: String,
        target: Target,
        cursor: String,
    },
}
#[derive(Deserialize)]
struct ActiveContext {
    url: String,
    notices: Vec<Notice>,
    tab_id: i32,
}
fn restricted(target: &Target, notices: &[Notice]) -> bool {
    notices.len() <= 8
        && notices
            .iter()
            .any(|n| Restriction::detect(target, n).is_some())
}
fn session_key(token: &str) -> Result<String, Error> {
    if token.len() == 48 && token.bytes().all(|b| b.is_ascii_hexdigit()) {
        Ok(format!("reader:{token}"))
    } else {
        Err(Error::Expired)
    }
}
async fn load_session(
    token: &str,
    target: Option<&Target>,
    sender: &Sender,
) -> Result<Session, Error> {
    let value = session_get(&session_key(token)?)
        .await
        .map_err(|_| Error::Storage)?;
    let s: Session = decode(value).map_err(|_| Error::Expired)?;
    if !s.valid(target.unwrap_or(&s.target), now()) || !s.bound_to(sender) {
        return Err(Error::Expired);
    }
    Ok(s)
}
async fn create_reader(
    tab_id: i32,
    target: Target,
    source: Target,
    is_restricted: bool,
    expires: Option<f64>,
    pending: bool,
    deliberate: bool,
) -> Result<Value, Error> {
    // Only live reader contexts are retained in browser memory. No disk-backed session ledger.
    let all: std::collections::HashMap<String, Session> =
        decode(session_all().await.map_err(|_| Error::Storage)?).unwrap_or_default();
    let expired: Vec<_> = all
        .iter()
        .filter(|(_, s)| s.expires <= now())
        .map(|(k, _)| k.clone())
        .collect();
    if !expired.is_empty() {
        session_remove(js(&expired))
            .await
            .map_err(|_| Error::Storage)?;
    }
    if all.len().saturating_sub(expired.len()) >= 100 {
        return Err(Error::Unavailable);
    }
    let token: String = random_bytes(24)
        .to_vec()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    let s = Session {
        target: target.clone(),
        source,
        pending,
        deliberate,
        restricted: is_restricted,
        expires: expires.unwrap_or(now() + 10. * 60. * 1000.),
        tab_id,
        allowed_posts: vec![],
        content: None,
    };
    session_set(&session_key(&token)?, js(&s))
        .await
        .map_err(|_| Error::Storage)?;
    Ok(
        json!({"session":token,"target":target,"view":crate::view::loading(&target),"load":{"op":"read","session":token,"target":target}}),
    )
}
async fn deliver(tab_id: i32, context: Value) -> Result<Value, Error> {
    let result: Value = decode(
        show_in_tab(tab_id, js(&context))
            .await
            .map_err(|_| Error::Unavailable)?,
    )?;
    if result["ok"] != true {
        return Err(Error::Unavailable);
    }
    Ok(json!({"opened":true}))
}

thread_local! {static COOLDOWN:Cell<f64>=const{Cell::new(0.)};}
struct Timer(i32);
impl Drop for Timer {
    fn drop(&mut self) {
        cancel_timer(self.0)
    }
}
async fn fetch(
    target: &Target,
    timeline: bool,
    cursor: Option<&str>,
) -> Result<(u16, String), Error> {
    fetch_url(provider::endpoint(target, timeline, cursor)?).await
}
async fn fetch_url(endpoint: String) -> Result<(u16, String), Error> {
    if COOLDOWN.with(|v| v.get() > now()) {
        return Err(Error::RateLimited);
    }
    let controller = web_sys::AbortController::new().map_err(|_| Error::Network)?;
    let init = web_sys::RequestInit::new();
    init.set_method("GET");
    init.set_credentials(web_sys::RequestCredentials::Omit);
    init.set_redirect(web_sys::RequestRedirect::Error);
    init.set_cache(web_sys::RequestCache::NoStore);
    init.set_referrer("");
    init.set_signal(Some(&controller.signal()));
    let request =
        web_sys::Request::new_with_str_and_init(&endpoint, &init).map_err(|_| Error::Network)?;
    request
        .headers()
        .set("Accept", "application/json")
        .map_err(|_| Error::Network)?;
    let _timer = Timer(schedule_abort(&controller, 15000));
    let map_net = |_: JsValue| {
        if controller.signal().aborted() {
            Error::Timeout
        } else {
            Error::Network
        }
    };
    let response: web_sys::Response = JsFuture::from(global_fetch(&request))
        .await
        .map_err(map_net)?
        .dyn_into()
        .map_err(|_| Error::InvalidResponse)?;
    let status = response.status();
    if status == 429 {
        let retry = response.headers().get("Retry-After").ok().flatten();
        let seconds = retry
            .and_then(|s| {
                s.parse::<f64>().ok().or_else(|| {
                    let date = js_sys::Date::parse(&s);
                    date.is_finite().then_some((date - now()) / 1000.)
                })
            })
            .unwrap_or(60.)
            .clamp(1., 3600.);
        COOLDOWN.with(|v| v.set(now() + seconds * 1000.));
        return Err(Error::RateLimited);
    }
    if status >= 500 {
        return Err(Error::Unavailable);
    }
    if response
        .headers()
        .get("Content-Length")
        .ok()
        .flatten()
        .and_then(|s| s.parse::<usize>().ok())
        .is_some_and(|n| n > 4 * 1024 * 1024)
    {
        controller.abort();
        return Err(Error::InvalidResponse);
    }
    let reader = response
        .body()
        .ok_or(Error::InvalidResponse)?
        .get_reader()
        .unchecked_into::<web_sys::ReadableStreamDefaultReader>();
    let mut bytes = Vec::new();
    loop {
        let chunk = JsFuture::from(reader.read()).await.map_err(map_net)?;
        if js_sys::Reflect::get(&chunk, &"done".into())
            .ok()
            .and_then(|v| v.as_bool())
            == Some(true)
        {
            break;
        }
        let chunk =
            js_sys::Reflect::get(&chunk, &"value".into()).map_err(|_| Error::InvalidResponse)?;
        let chunk = js_sys::Uint8Array::new(&chunk);
        if bytes.len() + chunk.length() as usize > 4 * 1024 * 1024 {
            controller.abort();
            return Err(Error::InvalidResponse);
        }
        bytes.extend(chunk.to_vec());
    }
    Ok((
        status,
        String::from_utf8(bytes).map_err(|_| Error::InvalidResponse)?,
    ))
}
async fn get_post(target: &Target) -> Result<Post, Error> {
    let Target::Post { id } = target else {
        return Err(Error::InvalidTarget);
    };
    let (h, b) = fetch(target, false, None).await?;
    provider::parse_post(h, &b, id)
}
async fn get_conversation(
    target: &Target,
    cursor: Option<&str>,
) -> Result<limitless_core::Conversation, Error> {
    let Target::Post { id } = target else {
        return Err(Error::InvalidTarget);
    };
    let (h, b) = fetch_url(provider::conversation_endpoint(id, cursor)?).await?;
    provider::parse_conversation(h, &b, id)
}
async fn get_profile(target: &Target) -> Result<Profile, Error> {
    let Target::Profile { handle } = target else {
        return Err(Error::InvalidTarget);
    };
    let (h, b) = fetch(target, false, None).await?;
    provider::parse_profile(h, &b, handle)
}
async fn timeline(
    token: &str,
    session: &mut Session,
    cursor: Option<&str>,
) -> Result<limitless_core::Timeline, Error> {
    if !matches!(session.target, Target::Profile { .. }) {
        return Err(Error::InvalidTarget);
    }
    let (h, b) = fetch(&session.target, true, cursor).await?;
    let t = provider::parse_timeline(h, &b)?;
    for post in &t.posts {
        if !session.allowed_posts.contains(&post.id) && session.allowed_posts.len() < 2000 {
            session.allowed_posts.push(post.id.clone());
        }
    }
    session_set(&session_key(token)?, js(session))
        .await
        .map_err(|_| Error::Storage)?;
    Ok(t)
}
async fn handle(request: Request, sender: Sender) -> Result<Value, Error> {
    let automatic = matches!(&request, Request::AutoDetected { .. });
    match request {
        Request::Ping => Ok(json!({"runtime":"rust-wasm"})),
        Request::Record => Ok(json!(limitless_storage::record(None, None, false).await?)),
        Request::Settings { enabled } => Ok(json!(
            limitless_storage::record(None, Some(enabled), false).await?
        )),
        Request::Clear => Ok(json!(limitless_storage::record(None, None, true).await?)),
        Request::Inspect { notices } => {
            let target = Target::parse(sender.current_url())?;
            Ok(json!({"eligible":restricted(&target,&notices),"target":target}))
        }
        Request::OpenDetected { notices } | Request::AutoDetected { notices } => {
            let target = Target::parse(sender.current_url())?;
            if !restricted(&target, &notices) {
                return Err(Error::Unauthorized);
            }
            create_reader(
                sender.tab_id.ok_or(Error::Unauthorized)?,
                target.clone(),
                target,
                true,
                None,
                false,
                !automatic,
            )
            .await
        }
        Request::OpenManual { input } => {
            let target = Target::parse(&input)?;
            let tab_id = new_tab().await.map_err(|_| Error::Unavailable)?;
            create_reader(
                tab_id,
                target.clone(),
                target.clone(),
                false,
                None,
                true,
                true,
            )
            .await?;
            navigate_tab(tab_id, &target.canonical_url())
                .await
                .map_err(|_| Error::Unavailable)?;
            Ok(json!({"opened":true}))
        }
        Request::Pending => {
            let all: std::collections::HashMap<String, Session> =
                decode(session_all().await.map_err(|_| Error::Storage)?)?;
            for (key, mut session) in all {
                if session.valid(&session.target, now())
                    && session.bound_to(&sender)
                    && Target::parse(sender.current_url())
                        .is_ok_and(|target| target == session.target)
                {
                    session.pending = false;
                    if let Some(content) = session.content.as_mut() {
                        let filter = crate::view::ProfileFilter::from_url(sender.current_url());
                        if content.profile_filter != filter {
                            content.profile_filter = filter;
                            content.visible_posts = 5;
                        }
                    }
                    session_set(&key, js(&session))
                        .await
                        .map_err(|_| Error::Storage)?;
                    let token = key.strip_prefix("reader:").ok_or(Error::Expired)?;
                    if let Some(content) = &session.content {
                        return Ok(
                            json!({"view":crate::view::reader(content,&session.target,token,session.restricted)}),
                        );
                    }
                    return Ok(
                        json!({"session":token,"target":session.target,"view":crate::view::loading(&session.target),"load":{"op":"read","session":token,"target":session.target}}),
                    );
                }
            }
            Ok(Value::Null)
        }
        Request::Current | Request::OpenCurrent => {
            let open = matches!(request, Request::OpenCurrent);
            let c: ActiveContext = decode(active_context().await.map_err(|_| Error::Unavailable)?)?;
            match Target::parse(&c.url) {
                Ok(target) => {
                    let is_restricted = restricted(&target, &c.notices);
                    if open {
                        let context = create_reader(
                            c.tab_id,
                            target.clone(),
                            target,
                            is_restricted,
                            None,
                            false,
                            true,
                        )
                        .await?;
                        deliver(c.tab_id, context).await
                    } else {
                        Ok(json!({"target":target,"restricted":is_restricted}))
                    }
                }
                Err(_) => {
                    if open {
                        Err(Error::InvalidTarget)
                    } else {
                        Ok(Value::Null)
                    }
                }
            }
        }
        Request::Read { session, target } => {
            let mut s = load_session(&session, Some(&target), &sender).await?;
            match &target {
                Target::Post { .. } => {
                    let (post, conversation, replies_error) = match get_conversation(&target, None)
                        .await
                    {
                        Ok(conversation) => (conversation.post.clone(), Some(conversation), None),
                        Err(
                            error @ (Error::Protected
                            | Error::Deleted
                            | Error::Suspended
                            | Error::RateLimited
                            | Error::Network
                            | Error::Timeout),
                        ) => return Err(error),
                        Err(error) => (get_post(&target).await?, None, Some(error.to_string())),
                    };
                    if let Some(conversation) = &conversation {
                        for post in conversation
                            .thread
                            .iter()
                            .chain(conversation.replies.iter())
                        {
                            if !s.allowed_posts.contains(&post.id) {
                                s.allowed_posts.push(post.id.clone());
                            }
                        }
                    }
                    let record_error = if s.restricted && s.deliberate {
                        limitless_storage::record(Some(("post", &post.id)), None, false)
                            .await
                            .err()
                            .map(|e| e.to_string())
                    } else {
                        None
                    };
                    s.content = Some(crate::view::Content {
                        post: Some(post),
                        conversation,
                        replies_error,
                        record_error,
                        ..Default::default()
                    });
                    session_set(&session_key(&session)?, js(&s))
                        .await
                        .map_err(|_| Error::Storage)?;
                    Ok(
                        json!({"view":crate::view::reader(s.content.as_ref().unwrap(),&target,&session,s.restricted)}),
                    )
                }
                Target::Profile { handle } => {
                    let (profile, feed) =
                        futures::join!(get_profile(&target), timeline(&session, &mut s, None));
                    if let Err(error @ (Error::Protected | Error::Suspended | Error::Deleted)) =
                        &profile
                    {
                        return Err(error.clone());
                    }
                    if let Err(error) = &profile
                        && feed.is_err()
                    {
                        return Err(error.clone());
                    }
                    let mut profile_error = profile.as_ref().err().map(ToString::to_string);
                    let profile = profile.ok().or_else(|| {
                        feed.as_ref()
                            .ok()?
                            .posts
                            .iter()
                            .find(|p| p.author.handle.eq_ignore_ascii_case(handle))
                            .map(|p| p.author.clone())
                    });
                    if profile.is_some() {
                        profile_error = None;
                    }
                    let record_error = if s.restricted && s.deliberate {
                        if let Some(p) = profile.as_ref() {
                            limitless_storage::record(Some(("profile", &p.id)), None, false)
                                .await
                                .err()
                                .map(|e| e.to_string())
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    let feed_error = feed.as_ref().err().map(ToString::to_string);
                    s.content = Some(crate::view::Content {
                        profile_filter: crate::view::ProfileFilter::from_url(sender.current_url()),
                        profile,
                        timeline: feed.ok(),
                        profile_error,
                        timeline_error: feed_error,
                        record_error,
                        ..Default::default()
                    });
                    session_set(&session_key(&session)?, js(&s))
                        .await
                        .map_err(|_| Error::Storage)?;
                    Ok(
                        json!({"view":crate::view::reader(s.content.as_ref().unwrap(),&target,&session,s.restricted)}),
                    )
                }
            }
        }
        Request::Timeline {
            session,
            target,
            cursor,
        } => {
            let mut s = load_session(&session, Some(&target), &sender).await?;
            let content = s.content.as_mut().ok_or(Error::Expired)?;
            if !content.reveal_buffered() {
                let expected = content
                    .timeline
                    .as_ref()
                    .ok_or(Error::InvalidTarget)?
                    .next_cursor
                    .as_ref();
                if expected.is_none() || expected != cursor.as_ref() {
                    return Err(Error::InvalidTarget);
                }
                let visible = content.visible_limit();
                let next = match timeline(&session, &mut s, cursor.as_deref()).await {
                    Ok(next) => next,
                    Err(error) => {
                        let content = s.content.as_mut().ok_or(Error::Expired)?;
                        content.timeline_error = Some(error.to_string());
                        session_set(&session_key(&session)?, js(&s))
                            .await
                            .map_err(|_| Error::Storage)?;
                        return Ok(
                            json!({"view":crate::view::reader(s.content.as_ref().unwrap(),&target,&session,s.restricted)}),
                        );
                    }
                };
                let content = s.content.as_mut().ok_or(Error::Expired)?;
                let feed = content.timeline.as_mut().ok_or(Error::InvalidTarget)?;
                for post in next.posts {
                    if !feed.posts.iter().any(|p| p.id == post.id) {
                        feed.posts.push(post);
                    }
                }
                feed.next_cursor = next
                    .next_cursor
                    .filter(|next| Some(next) != cursor.as_ref());
                content.visible_posts = (visible + 5).min(feed.posts.len());
                content.timeline_error = None;
            }
            session_set(&session_key(&session)?, js(&s))
                .await
                .map_err(|_| Error::Storage)?;
            Ok(
                json!({"view":crate::view::reader(s.content.as_ref().unwrap(),&target,&session,s.restricted)}),
            )
        }
        Request::Replies {
            session,
            target,
            cursor,
        } => {
            let mut s = load_session(&session, Some(&target), &sender).await?;
            let next = get_conversation(&target, Some(&cursor)).await?;
            for post in &next.replies {
                if !s.allowed_posts.contains(&post.id) && s.allowed_posts.len() < 2000 {
                    s.allowed_posts.push(post.id.clone());
                }
            }
            let conversation = s
                .content
                .as_mut()
                .and_then(|c| c.conversation.as_mut())
                .ok_or(Error::Expired)?;
            for post in next.replies {
                if !conversation.replies.iter().any(|p| p.id == post.id) {
                    conversation.replies.push(post);
                }
            }
            conversation.unavailable_replies += next.unavailable_replies;
            conversation.next_cursor = next.next_cursor.filter(|c| c != &cursor);
            session_set(&session_key(&session)?, js(&s))
                .await
                .map_err(|_| Error::Storage)?;
            Ok(
                json!({"view":crate::view::reader(s.content.as_ref().unwrap(),&target,&session,s.restricted)}),
            )
        }
        Request::OpenPost { session, id } => {
            let s = load_session(&session, None, &sender).await?;
            if !limitless_core::valid_id(&id) || !s.allowed_posts.contains(&id) {
                return Err(Error::Unauthorized);
            }
            create_reader(
                s.tab_id,
                Target::Post { id },
                s.source,
                s.restricted,
                Some(s.expires),
                false,
                true,
            )
            .await
        }
    }
}
#[wasm_bindgen]
pub async fn dispatch(message: JsValue, sender: JsValue) -> JsValue {
    let raw_message: Value = decode(message.clone()).unwrap_or(Value::Null);
    let result = async {
        let sender: Sender = decode(sender).map_err(|_| Error::Unauthorized)?;
        let raw: Value = decode(message.clone())?;
        let op = raw
            .get("op")
            .and_then(Value::as_str)
            .ok_or(Error::Unauthorized)?;
        if !authorized(&sender, op) {
            return Err(Error::Unauthorized);
        }
        let request: Request = decode(message)?;
        handle(request, sender).await
    }
    .await;
    js(&match result {
        Ok(data) => json!({"ok":true,"data":data}),
        Err(error) => {
            json!({"ok":false,"message":error.to_string(),"view":crate::view::error(&error, matches!(raw_message["op"].as_str(),Some("read"|"timeline")).then_some(raw_message)),"error":error})
        }
    })
}
