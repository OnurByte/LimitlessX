<div align="center">
<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/brand/mark-dark.svg">
  <img src="docs/brand/mark-light.svg" alt="LimitlessX" width="88" height="72">
</picture>

# LimitlessX

### No VPN. No proxy setup. No DPI hacks.

Read region-restricted public X content directly on X.

[![Checks](https://github.com/OnurByte/LimitlessX/actions/workflows/check.yml/badge.svg)](https://github.com/OnurByte/LimitlessX/actions/workflows/check.yml)
[![MIT license](https://img.shields.io/badge/license-MIT-black)](LICENSE)
[![Open source](https://img.shields.io/badge/open-source-black?logo=github)](https://github.com/OnurByte/LimitlessX)

[![Chromium](https://img.shields.io/badge/Chromium-development_build-4285F4?logo=googlechrome&logoColor=white)](#install)
[![Firefox](https://img.shields.io/badge/Firefox-development_build-FF7139?logo=firefoxbrowser&logoColor=white)](#install)
[![Helium](https://img.shields.io/badge/Helium-locally_verified-black)](docs/compatibility.md)

</div>

LimitlessX detects supported country-withholding notices and displays available public profiles, posts and replies inside X. It uses FxTwitter as an alternative public source, with no X login or separate LimitlessX account.

**Open source, MIT licensed.** Rust owns fetching, validation, sessions and storage. TypeScript connects those components to the browser. All executable code ships in the extension. There is no LimitlessX backend, analytics or remote executable code.

## What it does

- Automatically replaces a supported regional-withholding notice with available public content.
- Shows five posts first, then loads more as you scroll. Existing videos keep playing during feed expansion.
- Displays profile details, media, quoted posts and available conversation replies in X's visual style.
- Resolves exact public account identities when a username lookup fails, without confusing similarly named accounts.
- Keeps unique local counts of successfully viewed restricted accounts and posts. Disable or clear them in the popup.
- Uses one popup for manual URLs, current-page actions and settings. No extension reader page or iframe.

## Install

**Development builds. Not published in browser stores yet.** These badges describe build targets and test evidence, not store availability.

Build the project below, then load the generated package:

| Browser | Local installation | Verification |
|---|---|---|
| Chromium | Open the Extensions page, enable Developer mode, choose **Load unpacked**, select `dist/chromium` | Packaged extension integration tests |
| Helium | Use `chrome://extensions`, enable Developer mode and load `dist/chromium` | Actual X session tested locally |
| Firefox | Open `about:debugging`, choose **This Firefox**, **Load Temporary Add-on**, select `dist/firefox/manifest.json` | Packaged extension integration tests |

Firefox temporary add-ons disappear when the browser closes. Permanent Firefox distribution requires signing. Other Chromium/Firefox variants are intended targets; see [individual browser evidence](docs/compatibility.md) before assuming compatibility.

On a supported blocked X profile or post, reading starts automatically. The popup also accepts an `@handle` or an X profile/post URL.

## Privacy and limits

The public provider receives requested identifiers and network information, including your IP address. Media loads from X media hosts. Requests omit credentials; the extension does not read X passwords, authentication cookies or private messages. This is not an anonymity tool.

The Record stores unique salted identifier hashes in local IndexedDB. It does not store post bodies or plaintext browsing history. Failed requests and ordinary manual lookups do not increase restricted-content totals.

Availability depends on the provider and public source. Private content is unavailable. Timelines, replies and metadata can be incomplete; this is a focused reader, not a complete authenticated X client. Identity recovery fixes a known username-lookup failure, not every possible provider outage. Read the [privacy details](docs/privacy.md) and [release audit](docs/verification/release-audit.md).

## Build

Requirements: Node.js 22+, npm, rustup, Python for browser tests, `wasm-bindgen` 0.2.129 and Binaryen `version_133`. Rust 1.99.1 and the WASM target are pinned in `rust-toolchain.toml`.

```sh
npm ci --ignore-scripts
cargo install wasm-bindgen-cli --version 0.2.129 --locked
# Install Binaryen version_133 and place wasm-opt on PATH.
cargo xtask build chromium
cargo xtask build firefox
cargo xtask test
cargo xtask package
```

If needed, set `WASM_BINDGEN` and `WASM_OPT` to their executable paths. `cargo xtask` typechecks and compiles the handwritten TypeScript adapters; application logic remains in Rust. Packages and a reviewable source archive are written to `dist/releases`.

## Verify and contribute

```sh
npm run typecheck
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo clippy --target wasm32-unknown-unknown -p limitless-runtime -p limitless-ui --locked -- -D warnings
python3 -m pip install -r tests/browser/requirements.txt
python3 -m playwright install chromium firefox
```

The [CI workflow](.github/workflows/check.yml) runs native checks, builds both packages and exercises the installed extensions. Browser fixtures cover malformed responses, identity recovery, navigation, pagination, counters and retry behavior. Firefox's inline test also uses a live public provider response; that dependency is disclosed in its output.

Issues and pull requests are welcome. Keep permissions narrow, executable code local, provider text escaped and public/private boundaries explicit. Include a regression test for behavioral fixes. See [architecture](docs/architecture.md), [design](docs/brand.md) and [browser tests](tests/browser).

Built with Rust, WebAssembly, Leptos and TypeScript. Public data via [FxTwitter / FxEmbed](https://github.com/FxEmbed/FxEmbed).
