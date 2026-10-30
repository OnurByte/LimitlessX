# Architecture

The Rust workspace is authoritative. `core` owns identifiers, targets, notice eligibility, normalized types, errors and salted record keys. `fxtwitter` owns typed v2 envelopes, response validation, endpoint construction, errors and pagination. `runtime` owns trusted-message authorization, short-lived sessions, fetching, normalization and successful restricted-view decisions. `storage` owns atomic IndexedDB transactions. `ui` renders the popup (including settings) and content directly injected into X using Leptos for the popup and a Rust-owned typed DOM view for inline content. `xtask` builds browser packages.

## Browser boundary

Strict TypeScript compiles to packaged JavaScript and registers listeners synchronously before WASM initialization. A single initialization promise is reused for each active worker/page. The bridge forwards browser API calls. The content adapter observes bounded X notice containers, debounces DOM changes, and sends bounded notice data to Rust, which checks the official withholding-policy link. When Rust confirms a regional-withholding notice, it automatically starts one public lookup for that route/notice. Its own rendered DOM updates never trigger duplicate lookups.

The background Rust runtime constructs a typed DOM tree and action messages. A narrow, generic content adapter renders that tree into a DOM container inside X, using text nodes and an allowlist of tags/attributes. It does not parse provider text as HTML or duplicate provider/eligibility/view logic. A ShadowRoot and constructed stylesheet isolate the presentation from X's CSS. No iframe, extension reader page or options page exists; only `popup.html` is packaged as an extension-owned HTML interface, rendered with Leptos. Only the packaged stylesheet is web-accessible to supported X/Twitter origins; all executable resources remain private.

This is deliberate: the tested Chromium rejects even minimal WebAssembly compilation from a strict-CSP X content context. No page policy is changed, bypass flag is used, or alternative extension page introduced. Rust/WASM remains in the background and popup; X receives ordinary DOM nodes.

Reader contexts are returned over extension messaging to isolated content-script state. There is no page-level postMessage listener and no session token in DOM attributes or page URLs. A context is bound to its source X target and tab ID. Navigating to a new route invalidates pending view updates and removes its injected container.

## Local view counts

Opening from a validated X notice creates a target- and tab-bound, ten-minute session. Successful automatic and explicit displays of restricted content count once per public ID; ordinary manual lookups create ineligible sessions. Sessions live in browser.storage.session and survive worker restarts, but disappear when the browser exits. They are not durable browsing history. At most 100 active sessions are retained. Loading a profile can fetch its public timeline, but does not count those posts. A timeline post must be opened individually; only IDs actually returned in that reader's timeline can create a child context in the same injected container. Manual popup input opens an X tab with a pending ineligible context held in browser memory.

## Provider and storage

Rust uses Fetch with credentials omitted, redirects rejected, cache disabled, a 15-second abort deadline and a 4 MiB streaming response limit. HTTP or envelope rate limits produce an explicit error and a Retry-After cooldown. Provider media URLs are limited to HTTPS pbs.twimg.com and video.twimg.com. Provider text is represented as text nodes, never parsed as markup.

IndexedDB has three stores: `meta`, `profiles` and `posts`. A single read/write transaction checks the recording setting, initializes a random per-install salt, upserts a domain-separated SHA-256 key, and reads counts. Duplicate insertions overwrite the same primary key. Disabling and clearing run through the same transaction scope. Clearing rotates the salt and deletes both ledgers. No public identifiers or post text are required in the durable record.

## Browser tooling and media

Handwritten adapters live in extension/adapters/*.ts with strict TypeScript contracts. cargo xtask compiles them using the pinned npm lockfile, then packages emitted JS with Rust/WASM. No npm runtime dependency ships. The media adapter binds Rust-rendered controls to HTML video backed by HTTPS X media URLs. It does not import X application/player code or use an embed iframe. Media is paused on route removal, retained during matching post updates, and exposes retry/source access on failure.
