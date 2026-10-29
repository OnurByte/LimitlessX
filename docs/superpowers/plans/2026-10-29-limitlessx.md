# LimitlessX Implementation Plan

> **For agentic workers:** Use superpowers:executing-plans. Continue inline as requested by “bitir”.

**Goal:** Deliver the Rust-first extension, local record, brand, landing page, independently packaged browser builds, and evidence for all release gates.
**Architecture:** Rust core validates input, restriction evidence, provider payloads and eligibility. A Rust/WASM background dispatches requests and owns IndexedDB; Leptos renders extension pages. Small packaged JavaScript adapters expose browser APIs and observe bounded DOM notices.
**Tech Stack:** Rust, Leptos CSR, wasm-bindgen, web-sys Fetch, serde, thiserror, IndexedDB, Cargo xtask, Python browser harness.
**Spec:** docs/product-spec.md

## Global Constraints
- No TypeScript, React, Node application framework, remote executable code, telemetry, X credentials, or broad history permissions.
- FxTwitter API v2, replaceable provider boundary, explicit attribution, honest errors and optional fields.
- Chromium and Firefox MV3 packages; locally packaged WASM; minimal CSP and web-accessible resources.
- Deduplicated, atomic, per-install hashed local Record. Only deliberate successful eligible views count.
- English UI, system theme, monochrome original SVG, 4px spacing, 8px controls, 12px panels.
- No store or compatibility claims without actual evidence.

## Review Focus
- Malicious URLs/provider media and untrusted page messages must never become executable content or arbitrary network requests.
- Navigation races must never apply an old restriction to a different target.
- Concurrent record insertion, settings changes and clearing must remain atomic across tabs and restarts.
- Worker restarts must preserve valid short-lived reader context, without exposing it to arbitrary pages.
- Missing, private, deleted, suspended, rate-limited and malformed data must not be called regional withholding.

## Task 1: Feasibility and reproducible toolchain
Files: scripts/probe-provider.py, docs/verification/feasibility.json, rust-toolchain.toml, Cargo.toml, .cargo/config.toml.
- [ ] Probe live normal, historically withheld, protected, suspended, missing and invalid targets, retaining minimal evidence and provenance.
- [ ] Document limits: live provider availability does not establish a present X notice; no universal accessibility claim.
- [ ] Pin toolchain and build tools. Verify WASM target and browser availability.

## Task 2: Typed domain and provider
Files: crates/core/{src/lib.rs,tests/domain.rs}, crates/fxtwitter/{src/lib.rs,tests/provider.rs}, tests/fixtures.
Interfaces: Target::parse(&str), Restriction::detect(&Target, &Notice), provider::parse_profile/parse_post/parse_timeline -> Result<normalized types, Error>.
- [ ] Write tests for URL host/path attacks, reserved routes, matching notice evidence and deliberate eligibility; observe failures, implement and pass.
- [ ] Write fixture tests for envelopes, tombstones, missing fields, media schemes, code-vs-HTTP errors and pagination; observe failures, implement and pass.

## Task 3: Runtime, sessions and storage
Files: crates/runtime/src, crates/storage/src, extension/adapters/{background,api}.js.
Interfaces: async dispatch(message, trusted_sender) -> serialized response. IndexedDB owns settings, salted hashed keys and short-lived sessions.
- [ ] Verify real IndexedDB races and persistence in browser tests.
- [ ] Restrict operations by trusted sender, bind contexts to target and short expiry, omit credentials and bound requests with abort and payload limits.
- [ ] Verify background WASM startup and message response in both browsers before expanding UI.

## Task 4: Leptos pages, detection and brand
Files: crates/ui/src/{lib,popup,reader,options}.rs, extension/adapters/content.js, extension/static.
- [ ] Browser tests first for manual input, notices, false positives, navigation races, explicit fetch, provider errors, pagination, deliberate post activation and record controls.
- [ ] Build quiet responsive popup, reader and settings; escaped text, safe media URLs, attribution, loading, error and empty states.
- [ ] Add original monochrome SVG mark and raster extension icons, system light/dark styles.
- [ ] Prefer extension-tab fallback initially, then verify embedding and narrowly scoped reader resources if embedding can be made safe.

## Task 5: Packaging and release verification
Files: crates/xtask/src/main.rs, extension/manifests, .github/workflows, tests/browser.
- [ ] Implement cargo xtask build chromium / firefox, test and package. Validate artifacts, permissions, CSP, local scripts and repeatability.
- [ ] Run native tests, WASM builds, browser integration for Chromium and Firefox, restart/concurrency/security tests and measured performance baselines.
- [ ] Add source-review archives and CI; run quality checks.

## Task 6: Landing page and release documentation
Files: site/, README.md, LICENSE, docs/{architecture,privacy,brand,compatibility}.md, docs/verification/release-audit.md.
- [ ] Build accessible static landing page with accurate install availability, official badge assets where available, privacy, Record illustration and open-source links.
- [ ] Prepare store listings and submission instructions. Published store URLs require actual store publication; never invent them.
- [ ] Audit every spec section and gate against authoritative evidence; keep the goal active for incomplete external release gates.
