# Architecture

The Rust workspace is authoritative. `core` owns identifiers, targets, notice eligibility, normalized types, errors and salted record keys. `fxtwitter` owns typed v2 envelopes, response validation, endpoint construction, errors and pagination. `runtime` owns trusted-message authorization, short-lived sessions, fetching, normalization and deliberate-view decisions. `storage` owns atomic IndexedDB transactions. `ui` renders popup, settings and reader using Leptos. `xtask` builds browser packages.

## Browser boundary

Packaged JavaScript registers listeners synchronously before WASM initialization. A single initialization promise is reused for each active worker/page. The bridge forwards browser API calls. The content adapter observes bounded X notice containers, debounces DOM changes, and offers a button only when Rust identifies a supported notice with the official withholding-policy link. It never makes provider requests automatically.

The reader uses an extension tab. No executable resources are web-accessible and there is no cross-origin postMessage surface. This is the packaged extension-tab fallback allowed by the specification; embedded rendering has not been claimed or shipped. The two runtime smoke tests verify packaged WASM in Chromium service workers and Firefox event pages.

## Deliberate views

Opening from a validated X notice creates a target-bound, ten-minute session. A manual lookup creates an ineligible session. Sessions live in browser.storage.session and survive worker restarts, but disappear when the browser exits. They are not durable browsing history. At most 100 active sessions are retained. Loading a profile can fetch its public timeline, but does not count those posts. A timeline post must be opened deliberately; only IDs actually returned in that reader's timeline can create a child reader session.

## Provider and storage

Rust uses Fetch with credentials omitted, redirects rejected, cache disabled, a 15-second abort deadline and a 4 MiB streaming response limit. HTTP or envelope rate limits produce an explicit error and a Retry-After cooldown. Provider media URLs are limited to HTTPS pbs.twimg.com and video.twimg.com. Leptos escapes all provider text.

IndexedDB has three stores: `meta`, `profiles` and `posts`. A single read/write transaction checks the recording setting, initializes a random per-install salt, upserts a domain-separated SHA-256 key, and reads counts. Duplicate insertions overwrite the same primary key. Disabling and clearing run through the same transaction scope. Clearing rotates the salt and deletes both ledgers. No public identifiers or post text are required in the durable record.
