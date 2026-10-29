use crate::app::{Footer, Header};
use leptos::prelude::*;
#[component]
pub fn Options() -> impl IntoView {
    view! {<main class="document"><Header/><section class="page-title"><p class="eyebrow">"YOUR DEVICE. YOUR RECORD."</p><h2>"A little less friction."</h2><p class="muted">"A focused reader, with a record you control."</p></section><crate::record::RecordPanel controls=true/>
    <section class="prose"><h2>"What The Record counts"</h2><p>"Distinct restricted profiles and posts you deliberately open after LimitlessX detects a regional restriction on X. Manual lookups and automatically loaded timelines do not add to the counters."</p><p>"Reopening the same content never increases its unique count. Turning recording off keeps existing counts. Clear record removes the stored ledger and starts a new record."</p>
    <h2>"What stays here"</h2><p>"The Record lives in this browser’s IndexedDB. It stores per-install salted hashes of content identifiers, two counters, and your recording preference. It does not store the text of what you read. Uninstalling the extension removes its local data."</p>
    <h2>"What leaves your device"</h2><p>"Only after you choose to open content, LimitlessX requests public data from api.fxtwitter.com. FxTwitter receives the requested account or post identifier and your network address. Loading images or videos connects to X’s public media hosts. These services have their own policies."</p><p>"Requests omit credentials. LimitlessX does not use your X cookies, collect passwords, run analytics, or operate a backend. It makes no anonymity promise."</p>
    <h2>"Availability has limits"</h2><p>"FxTwitter may return incomplete, stale, or unavailable data. An unavailable response does not prove regional withholding. Protected, suspended and deleted content are separate states. LimitlessX cannot unlock accounts or guarantee access."</p>
    <h2>"Open source"</h2><p><a href="https://github.com/OnurByte/LimitlessX" target="_blank" rel="noopener noreferrer">"Source, issues and releases ↗"</a>" · MIT license · Version 0.1.0"</p></section><Footer/></main>}
}
