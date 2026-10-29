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
    async fn open_tab(url: &str) -> Result<(), JsValue>;
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
async fn load_session(token: &str, target: Option<&Target>) -> Result<Session, Error> {
    let value = session_get(&session_key(token)?)
        .await
        .map_err(|_| Error::Storage)?;
    let s: Session = decode(value).map_err(|_| Error::Expired)?;
    if !s.valid(target.unwrap_or(&s.target), now()) {
        return Err(Error::Expired);
    }
    Ok(s)
}
async fn create_reader(
    sender: &Sender,
    target: Target,
    is_restricted: bool,
    expires: Option<f64>,
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
        target,
        restricted: is_restricted,
        expires: expires.unwrap_or(now() + 10. * 60. * 1000.),
        tab_id: sender.tab_id.unwrap_or(-1),
        allowed_posts: vec![],
    };
    session_set(&session_key(&token)?, js(&s))
        .await
        .map_err(|_| Error::Storage)?;
    let target = serde_json::to_string(&s.target).map_err(|_| Error::InvalidTarget)?;
    let mut url = url::Url::parse(&format!("{}reader.html", sender.extension_origin))
        .map_err(|_| Error::Unauthorized)?;
    url.query_pairs_mut()
        .append_pair("session", &token)
        .append_pair("target", &target);
    open_tab(url.as_ref())
        .await
        .map_err(|_| Error::Unavailable)?;
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
    if COOLDOWN.with(|v| v.get() > now()) {
        return Err(Error::RateLimited);
    }
    let endpoint = provider::endpoint(target, timeline, cursor)?;
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
    match request {
        Request::Ping => Ok(json!({"runtime":"rust-wasm"})),
        Request::Record => Ok(json!(limitless_storage::record(None, None, false).await?)),
        Request::Settings { enabled } => Ok(json!(
            limitless_storage::record(None, Some(enabled), false).await?
        )),
        Request::Clear => Ok(json!(limitless_storage::record(None, None, true).await?)),
        Request::Inspect { notices } => {
            let target = Target::parse(&sender.url)?;
            Ok(json!({"eligible":restricted(&target,&notices),"target":target}))
        }
        Request::OpenDetected { notices } => {
            let target = Target::parse(&sender.url)?;
            if !restricted(&target, &notices) {
                return Err(Error::Unauthorized);
            }
            create_reader(&sender, target, true, None).await
        }
        Request::OpenManual { input } => {
            create_reader(&sender, Target::parse(&input)?, false, None).await
        }
        Request::Current | Request::OpenCurrent => {
            let open = matches!(request, Request::OpenCurrent);
            let c: ActiveContext = decode(active_context().await.map_err(|_| Error::Unavailable)?)?;
            match Target::parse(&c.url) {
                Ok(target) => {
                    let is_restricted = restricted(&target, &c.notices);
                    if open {
                        create_reader(
                            &Sender {
                                tab_id: Some(c.tab_id),
                                ..sender
                            },
                            target,
                            is_restricted,
                            None,
                        )
                        .await
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
            let mut s = load_session(&session, Some(&target)).await?;
            match &target {
                Target::Post { .. } => {
                    let post = get_post(&target).await?;
                    let record_error = if s.restricted {
                        limitless_storage::record(Some(("post", &post.id)), None, false)
                            .await
                            .err()
                            .map(|e| e.to_string())
                    } else {
                        None
                    };
                    Ok(json!({"post":post,"restricted":s.restricted,"record_error":record_error}))
                }
                Target::Profile { handle } => {
                    let profile = get_profile(&target).await;
                    let feed = timeline(&session, &mut s, None).await;
                    if profile.is_err() && feed.is_err() {
                        return Err(profile.unwrap_err());
                    }
                    let profile_error = profile.as_ref().err().map(ToString::to_string);
                    let profile = profile.ok().or_else(|| {
                        feed.as_ref()
                            .ok()?
                            .posts
                            .iter()
                            .find(|p| p.author.handle == *handle)
                            .map(|p| p.author.clone())
                    });
                    let record_error = if s.restricted {
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
                    Ok(
                        json!({"profile":profile,"timeline":feed.ok(),"profile_error":profile_error,"timeline_error":feed_error,"restricted":s.restricted,"record_error":record_error}),
                    )
                }
            }
        }
        Request::Timeline {
            session,
            target,
            cursor,
        } => {
            let mut s = load_session(&session, Some(&target)).await?;
            Ok(json!(timeline(&session, &mut s, cursor.as_deref()).await?))
        }
        Request::OpenPost { session, id } => {
            let s = load_session(&session, None).await?;
            if !limitless_core::valid_id(&id) || !s.allowed_posts.contains(&id) {
                return Err(Error::Unauthorized);
            }
            create_reader(&sender, Target::Post { id }, s.restricted, Some(s.expires)).await
        }
    }
}
#[wasm_bindgen]
pub async fn dispatch(message: JsValue, sender: JsValue) -> JsValue {
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
        Err(error) => json!({"ok":false,"message":error.to_string(),"error":error}),
    })
}
