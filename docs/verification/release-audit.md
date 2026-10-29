# Release audit — development build

The current behavior is defined by [the user's revised direction](../implementation-direction.md). The attached original spec is historical where it conflicts.

| Requirement | Current evidence | Remaining |
|---|---|---|
| Direct X DOM injection; popup only | Chromium automatic inline test and Firefox live-provider inline test; package validator rejects reader/options pages | Real X layouts/notices across supported languages |
| Automatic regional detection | Rust notice rules; generic suspended error ignored; one lookup for an unchanged notice; route refresh integration contracts | Actual current account notice verified in Helium; additional languages and regional post notices remain |
| X-style presentation | Native post/profile typography, avatar column, timeline borders and page theme; actual rendered screenshots; theme change test | Actual Helium profile/post/reply geometry and SVG actions captured; unusual media layouts and broader comparisons remain |
| User-provided logo | Six-stripe SVG master, retained reference image, 16/32/48/128 PNG derivatives | None for local assets |
| Rust-first app | Domain/provider/runtime/storage in Rust; Leptos popup; Rust inline DOM view and generic browser adapter | Independent store source review |
| Provider capability | Live historically withheld account timeline returned 20 posts with pagination; normal status/20 verified | Current regional profile notice verified in Helium; profile/timeline endpoint availability remains inconsistent, reliable private/suspended/deleted live examples remain |
| Local Record | Atomic IndexedDB, salted unique keys, disable/reset, automatic-vs-deliberate eligibility, dedup and restart tests | Broader concurrent reset stress tests |
| Security | Exact source/tab sessions, expiry before fetch, escaped DOM text, narrow permissions, credential omission, host CSP preserved, private executable resources | Complete threat review and network/media audit |
| Cross-browser builds | MV3 Chromium worker and Firefox event-page smoke, automatic inline rendering in both | Individually untested browser variants remain labeled untested |
| Packaging | cargo xtask build chromium/firefox, test and package; deterministic zip test; source archive and CI workflow | CI result after pushing; signing/store submission |
| Public v1.0 | Not declared | All remaining gates, actual browser-store publication |

No store availability badges, store URLs, universal access claims or variant compatibility claims are invented. A standalone landing page and extension reader/options pages are excluded by the user's revised instruction.
