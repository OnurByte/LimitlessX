use crate::app::{Attribution, Footer, Header, call};
use leptos::prelude::*;
use limitless_core::Target;
use serde::Deserialize;
use serde_json::json;
#[derive(Clone, Deserialize)]
struct Current {
    target: Target,
    restricted: bool,
}
#[component]
pub fn Popup() -> impl IntoView {
    let current = RwSignal::new(None::<Current>);
    let input = RwSignal::new(String::new());
    let error = RwSignal::new(None::<String>);
    let busy = RwSignal::new(false);
    leptos::task::spawn_local(async move {
        if let Ok(v) = call(json!({"op":"current"})).await {
            current.set(serde_json::from_value(v).ok());
        }
    });
    let open = move |current_page: bool| {
        error.set(None);
        busy.set(true);
        let message = if current_page {
            json!({"op":"open_current"})
        } else {
            json!({"op":"open_manual","input":input.get_untracked()})
        };
        leptos::task::spawn_local(async move {
            match call(message).await {
                Ok(_) => {}
                Err(e) => error.set(Some(e)),
            }
            busy.set(false);
        });
    };
    view! {<main><Header/><section class="intro"><p class="eyebrow">"PUBLIC CONTENT, WHEN AVAILABLE"</p><p class="tagline">"No VPN."<br/>"No proxy setup."<br/>"No DPI hacks."</p><p class="muted">"An alternative way to read public X content."</p></section>
        {move ||current.get().map(|c|view!{<section class="current panel"><h2>"Current page"</h2><p class="target">{c.target.label()}</p><p class="context-status">{if c.restricted{"Regional restriction detected"}else{"No regional restriction detected"}}</p><button class="primary" disabled=move ||busy.get() on:click=move |_|open(true)>"View public content"<span aria-hidden="true">"↗"</span></button></section>})}
        <form class="lookup" on:submit=move |ev|{ev.prevent_default();open(false);}>
            <label for="target-input">{move ||if current.get().is_some(){"Or open another profile or post"}else{"X profile or post"}}</label>
            <input id="target-input" type="text" placeholder="@handle or https://x.com/…" autocomplete="off" spellcheck="false" maxlength="2048" prop:value=move ||input.get() on:input=move |ev|input.set(event_target_value(&ev))/>
            <button class="primary" disabled=move ||busy.get() || input.get().trim().is_empty()>{move ||if busy.get(){"Opening…"}else{"Open public reader"}}<span aria-hidden="true">"↗"</span></button>
        </form>
        {move ||error.get().map(|e|view!{<p class="error" role="alert">{e}</p>})}
        <p class="fine disclosure">"When you open content, its identifier is sent to FxTwitter. Private content is unavailable."</p><crate::record::RecordPanel/><Attribution/><Footer/>
    </main>}
}
