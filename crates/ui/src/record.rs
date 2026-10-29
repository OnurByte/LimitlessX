use crate::app::call;
use leptos::prelude::*;
use limitless_core::Record;
use serde_json::json;
#[component]
pub fn RecordPanel(#[prop(default = false)] controls: bool) -> impl IntoView {
    let record = RwSignal::new(None::<Record>);
    let error = RwSignal::new(None::<String>);
    let busy = RwSignal::new(false);
    let refresh = move || {
        leptos::task::spawn_local(async move {
            match call(json!({"op":"record"})).await {
                Ok(v) => match serde_json::from_value(v) {
                    Ok(v) => record.set(Some(v)),
                    Err(_) => error.set(Some("Could not read The Record.".into())),
                },
                Err(e) => error.set(Some(e)),
            }
        })
    };
    refresh();
    let _focus = window_event_listener(leptos::ev::focus, move |_| refresh());
    let update = move |message| {
        busy.set(true);
        error.set(None);
        leptos::task::spawn_local(async move {
            match call(message).await {
                Ok(v) => {
                    record.set(serde_json::from_value(v).ok());
                }
                Err(e) => error.set(Some(e)),
            }
            busy.set(false);
        });
    };
    view! {<section class="record panel" aria-label="The Record"><div class="section-heading"><h2>"The Record"</h2><span aria-hidden="true">"⁜"</span></div>
        {move ||record.get().map(|r|view!{<div class="metrics"><div><strong>{r.profiles}</strong><span>"restricted profiles"<br/>"viewed"</span></div><div><strong>{r.posts}</strong><span>"posts"<br/>"viewed"</span></div></div>
            <p class="fine">{if r.enabled{"Stored on this device · Since installation"}else{"Recording disabled · Stored on this device"}}</p>})}
        {move ||if record.get().is_none() && error.get().is_none(){Some(view!{<p class="muted" role="status">"Loading local record…"</p>})}else{None}}
        {controls.then(||view!{<div class="record-controls"><label class="switch"><input type="checkbox" prop:checked=move ||record.get().is_some_and(|r|r.enabled) disabled=move ||busy.get() on:change=move |ev|update(json!({"op":"settings","enabled":event_target_checked(&ev)}))/><span>"Keep a private record"</span></label><button class="secondary" disabled=move ||busy.get() on:click=move |_|update(json!({"op":"clear"}))>"Clear record"</button></div>})}
        {move ||error.get().map(|e|view!{<p class="error" role="alert">{e}</p>})}
    </section>}
}
