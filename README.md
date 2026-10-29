# LimitlessX

**No VPN. No proxy setup. No DPI hacks.**

A Rust-powered browser extension for viewing public X content through FxTwitter, when available. Firefox and Chromium builds share one Rust workspace and a Leptos interface.

**Development version. Not yet released in browser stores.** Availability is not guaranteed. The live provider returns some historically withheld public content, but has inconsistent profile responses; current regional-notice verification and the full release audit are still in progress.

Restricted content appears automatically inside X and follows its native timeline styling.

The current interface direction is [direct X injection with a popup only](docs/implementation-direction.md).

## Build

Install rustup; the project pins Rust 1.99.1 and the WASM target in `rust-toolchain.toml`. Install matching tools:

```sh
cargo install wasm-bindgen-cli --version 0.2.129 --locked
# Install Binaryen version_133 and put wasm-opt on PATH.
cargo xtask build chromium
cargo xtask build firefox
cargo xtask test
```

If tools are not on PATH, set `WASM_BINDGEN` and `WASM_OPT` to their executable paths. See [tool versions](scripts/tools.env.example). Browser integration requires Python with Playwright and Selenium, plus Chromium, Firefox and geckodriver; see `tests/browser/`.

## Use

Load `dist/chromium` as an unpacked extension, or temporarily load `dist/firefox/manifest.json` through Firefox's about:debugging. Open a public X profile/post from the popup, or use the LimitlessX action on a supported regional-withholding notice. A confirmed regional-withholding notice automatically starts the public-data lookup. Generic failures do not. Public content is injected directly into the X page. There is no extension reader page or iframe. Manual input opens the corresponding X URL and renders there.

Automatic displays and manual lookups do not add to The Record. Eligible restricted profiles count once after a successful deliberate open. Fetched timeline posts count only when individually opened. Settings inside the popup can disable recording or clear the local ledger.

## Project

- [Product specification](docs/product-spec.md)
- [Architecture](docs/architecture.md)
- [Privacy](docs/privacy.md)
- [Browser evidence](docs/compatibility.md)
- [Implementation progress](docs/progress.md)

MIT license. Contributions should preserve Rust-owned application logic, narrow browser adapters, explicit provider attribution and honest capability claims. Run the native tests and both browser suites before proposing a release. Use fixtures for error states; do not classify unavailable content as a regional block.
