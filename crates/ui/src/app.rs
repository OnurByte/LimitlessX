use leptos::prelude::*;
use serde::Serialize;
use serde_json::Value;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace=limitlessUI,catch)]
    async fn send(message: JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(js_namespace=limitlessUI)]
    pub fn asset_url(path: &str) -> String;
}
pub async fn call(message: Value) -> Result<Value, String> {
    let value = message
        .serialize(&serde_wasm_bindgen::Serializer::json_compatible())
        .map_err(|_| "Could not prepare request.".to_owned())?;
    let result = send(value)
        .await
        .map_err(|_| "Could not reach LimitlessX. Reopen the extension.".to_owned())?;
    let value: Value = serde_wasm_bindgen::from_value(result)
        .map_err(|_| "Unexpected extension response.".to_owned())?;
    if value["ok"] == true {
        Ok(value["data"].clone())
    } else {
        Err(value["message"]
            .as_str()
            .unwrap_or("Could not complete this action.")
            .into())
    }
}
#[component]
pub fn Header() -> impl IntoView {
    view! {<header class="masthead"><div class="wordmark"><span class="brand-mark" aria-hidden="true" inner_html=include_str!("../../../extension/static/brand/mark.svg")></span><h1>"LimitlessX"</h1></div></header>}
}
#[component]
pub fn Footer() -> impl IntoView {
    view! {<footer><a class="fine" href="https://onur.af" target="_blank" rel="noopener noreferrer">"By Onur"</a><a href="https://github.com/OnurByte/LimitlessX" target="_blank" rel="noopener noreferrer">"GitHub ↗"</a></footer>}
}
#[component]
pub fn Attribution() -> impl IntoView {
    view! {<p class="attribution">"Public data via "<a href="https://github.com/FxEmbed/FxEmbed" target="_blank" rel="noopener noreferrer">"FxTwitter"</a>"."</p>}
}
#[wasm_bindgen(start)]
pub fn start() {
    let window = web_sys::window().unwrap();
    if window
        .location()
        .protocol()
        .ok()
        .is_some_and(|p| p == "chrome-extension:" || p == "moz-extension:")
    {
        window
            .document()
            .unwrap()
            .body()
            .unwrap()
            .set_class_name("popup-page");
        leptos::mount::mount_to_body(|| view! {<crate::popup::Popup/>});
    }
}
