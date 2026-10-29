# Compatibility

| Browser | Evidence | Status |
|---|---|---|
| Chromium (Playwright full browser) | tests/browser/smoke.py: real MV3 worker/WASM response and Leptos popup | Runtime proof passed; complete flow verification pending |
| Firefox (Playwright Firefox binary through geckodriver) | tests/browser/firefox_smoke.py: installed MV3 package, event-page WASM response and Leptos popup | Runtime proof passed; complete flow verification pending |
| Chrome, Brave, Opera, Edge, Vivaldi | Same Chromium package intended | Not individually verified |
| Zen, Floorp, LibreWolf, Waterfox | Same Firefox package intended | Not individually verified |

The reader opens in an extension tab. No iframe or web-accessible application resources are required. Firefox MV3 uses a classic bootstrap importing the same background module as Chromium; Chromium uses a module service worker. Both register the message listener before awaiting WASM initialization.

Local Chromium install: open the browser's Extensions page, enable Developer mode, choose Load unpacked, select `dist/chromium`.

Local Firefox test install: open about:debugging, select This Firefox, choose Load Temporary Add-on, select `dist/firefox/manifest.json`. Temporary add-ons are removed at browser shutdown. Permanent distribution requires Mozilla signing.

These runtime proofs are not full compatibility certification or a store listing. See the release audit for the remaining gates.
