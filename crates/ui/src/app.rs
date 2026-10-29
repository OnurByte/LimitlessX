use leptos::prelude::*;
use serde::Serialize;
use serde_json::Value;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace=limitlessUI,catch)]
    async fn send(message: JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(js_namespace=limitlessUI)]
    pub fn open_options();
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
    view! {<header class="masthead"><a class="wordmark" href="popup.html"><img src="brand/mark.svg" alt=""/><h1>"LimitlessX"</h1></a><span class="edition">"PUBLIC READER"</span></header>}
}
#[component]
pub fn Footer() -> impl IntoView {
    view! {<footer><button class="text-button" on:click=move |_|open_options()>"Settings"</button><a href="https://github.com/OnurByte/LimitlessX" target="_blank" rel="noopener noreferrer">"GitHub ↗"</a></footer>}
}
#[component]
pub fn Attribution() -> impl IntoView {
    view! {<p class="attribution">"Public data via "<a href="https://github.com/FxEmbed/FxEmbed" target="_blank" rel="noopener noreferrer">"FxTwitter"</a>". Availability can change."</p>}
}
#[wasm_bindgen(start)]
pub fn start() {
    let path = web_sys::window()
        .and_then(|w| w.location().pathname().ok())
        .unwrap_or_default();
    let body = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .body()
        .unwrap();
    if path.ends_with("reader.html") {
        body.set_class_name("reader-page");
        leptos::mount::mount_to_body(|| view! {<crate::reader::Reader/>});
    } else if path.ends_with("options.html") {
        body.set_class_name("options-page");
        leptos::mount::mount_to_body(|| view! {<crate::options::Options/>});
    } else {
        body.set_class_name("popup-page");
        leptos::mount::mount_to_body(|| view! {<crate::popup::Popup/>});
    }
}
