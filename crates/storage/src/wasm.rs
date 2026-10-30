use limitless_core::{Error, Record, record_key};
use rexie::{ObjectStore, Rexie, TransactionMode};
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace=limitlessBridge)]
    fn record_changed();
    #[wasm_bindgen(js_namespace=limitlessBridge)]
    fn random_bytes(size: u32) -> js_sys::Uint8Array;
}
thread_local! {static DB:RefCell<Option<Rc<Rexie>>>=const{RefCell::new(None)};}
fn err<T>(r: Result<T, rexie::Error>) -> Result<T, Error> {
    r.map_err(|_| Error::Storage)
}
fn val<T: Serialize>(v: &T) -> Result<JsValue, Error> {
    serde_wasm_bindgen::to_value(v).map_err(|_| Error::Storage)
}
#[derive(Serialize, Deserialize)]
struct Meta {
    salt: Vec<u8>,
    enabled: bool,
    since: f64,
}
async fn database() -> Result<Rc<Rexie>, Error> {
    if let Some(db) = DB.with(|d| d.borrow().clone()) {
        return Ok(db);
    }
    let db = Rc::new(err(Rexie::builder("limitlessx")
        .version(1)
        .add_object_store(ObjectStore::new("meta"))
        .add_object_store(ObjectStore::new("profiles"))
        .add_object_store(ObjectStore::new("posts"))
        .build()
        .await)?);
    DB.with(|d| *d.borrow_mut() = Some(db.clone()));
    Ok(db)
}
/// A read/write transaction also serializes initial salt creation across contexts.
pub async fn record(
    kind: Option<(&str, &str)>,
    enabled: Option<bool>,
    clear: bool,
) -> Result<Record, Error> {
    let db = database().await?;
    let tx = err(db.transaction(&["meta", "profiles", "posts"], TransactionMode::ReadWrite))?;
    let meta_store = err(tx.store("meta"))?;
    let key = JsValue::from_str("settings");
    let mut meta = match err(meta_store.get(key.clone()).await)? {
        Some(v) => serde_wasm_bindgen::from_value::<Meta>(v).map_err(|_| Error::Storage)?,
        None => Meta {
            salt: random_bytes(32).to_vec(),
            enabled: true,
            since: js_sys::Date::now(),
        },
    };
    if let Some(enabled) = enabled {
        meta.enabled = enabled;
    }
    let profiles = err(tx.store("profiles"))?;
    let posts = err(tx.store("posts"))?;
    if clear {
        err(profiles.clear().await)?;
        err(posts.clear().await)?;
        meta.salt = random_bytes(32).to_vec();
        meta.since = js_sys::Date::now();
    }
    err(meta_store.put(&val(&meta)?, Some(&key)).await)?;
    if meta.enabled
        && let Some((kind, id)) = kind
    {
        let store = match kind {
            "profile" => &profiles,
            "post" => &posts,
            _ => return Err(Error::Storage),
        };
        let key = JsValue::from_str(&record_key(&meta.salt, kind, id));
        // put is idempotent under the store's unique primary key, in the same transaction as settings.
        err(store.put(&JsValue::TRUE, Some(&key)).await)?;
    }
    let result = Record {
        profiles: err(profiles.count(None).await)?,
        posts: err(posts.count(None).await)?,
        enabled: meta.enabled,
        since: meta.since,
    };
    err(tx.done().await)?;
    if kind.is_some() || enabled.is_some() || clear {
        record_changed();
    }
    Ok(result)
}
