# Current product direction — 2026-10-30

The user's latest instruction supersedes the extension-origin reader, options page, original logo and landing-page assumptions in the attached planning specification:

- Automatically detect a supported regional restriction, retrieve public content and render it directly inside x.com’s DOM. No activation button, iframe or separate extension reader tab.
- Match X’s native profile/post layout, typography, timeline borders and current page colors. The injected area is not a separate branded panel.
- The only extension-owned HTML interface is the popup. Settings and The Record controls belong there.
- Use a clean SVG tracing of the user-provided six slanted black bars, replacing the original loop/X mark.
- Keep Rust-owned validation, data requests, session eligibility, local Record and Leptos rendering. The background Rust runtime produces a typed DOM tree and actions; a generic content adapter mounts those elements inside X with isolated styles matching X. Leptos renders the popup. This avoids compiling WASM under the host page CSP (blocked in the tested Chromium strict-CSP context) without loosening that policy.
- Remove the standalone reader/options resources and all entry points into them. No landing page is required under the revised instruction.

Manual input may navigate to the requested X URL, where the content is injected. A confirmed regional notice starts an automatic public-data request; unrelated errors never do. Short-lived reader tokens remain in isolated extension state and do not appear in page URLs or DOM attributes.

## Release review update (2026-10-30)

The user now explicitly requests TypeScript for handwritten browser adapters. Rust still owns domain/provider/runtime/storage/view decisions. Strict TypeScript is build-time tooling only; packaged output is local JavaScript with no npm runtime dependencies. This supersedes the original no-TypeScript requirement. Successful automatic displays of restricted accounts/posts now count uniquely in The Record; manual unrelated lookups do not. The popup follows design-reference.md and retains By Onur as explicitly requested. X media URLs use a packaged player, without remote X application code.
