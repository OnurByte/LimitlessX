# Compatibility

| Browser | Evidence | Status |
|---|---|---|
| Chromium (Playwright full browser) | tests/browser/smoke.py: real MV3 worker/WASM response and Leptos popup | Automatic inline integration passed; see current-build.json |
| Firefox (Playwright Firefox binary through geckodriver) | tests/browser/firefox_smoke.py: installed MV3 package, event-page WASM response and Leptos popup | Automatic inline integration passed using live public status/20 |
| Chrome, Brave, Opera, Edge, Vivaldi | Same Chromium package intended | Not individually verified |
| Zen, Floorp, LibreWolf, Waterfox | Same Firefox package intended | Not individually verified |

Public content is rendered directly in X's DOM, within a ShadowRoot. The popup is the extension's only HTML interface. Only the packaged stylesheet is exposed to X/Twitter domains; executable resources remain private. Both browser packages retain locally packaged WASM and synchronous background listener registration. Current tests exercise automatic DOM injection, native timeline styling, and a strict host-page CSP. Earlier extension-tab tests were removed.

Local Chromium install: open the browser's Extensions page, enable Developer mode, choose Load unpacked, select `dist/chromium`.

Local Firefox test install: open about:debugging, select This Firefox, choose Load Temporary Add-on, select `dist/firefox/manifest.json`. Temporary add-ons are removed at browser shutdown. Permanent distribution requires Mozilla signing.

These runtime proofs are not full compatibility certification or a store listing. See the release audit for the remaining gates.
