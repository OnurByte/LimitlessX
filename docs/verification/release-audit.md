# Release audit — development in progress

A green native test run is not a release decision. Each gate below needs its own evidence.

| Specification requirement | Evidence so far | Remaining |
|---|---|---|
| P0 FxTwitter withheld capability | Live historically withheld timeline returns 20 posts and pagination; provenance in feasibility.json | Current X notice against real representative accounts/posts, reliable protected/suspended/deleted comparisons |
| P1 Rust/WASM runtime | Real packaged Leptos popup and background ping passed Chromium and Firefox | Full messaging lifecycle/restart tests |
| P2 original brand | SVG master and 16/32/48/128 PNG assets, brand.md | Visual QA of all sizes/themes |
| P3 restriction detection | Native notice/target tests and bounded DOM adapter | Real X notice and navigation/false-positive browser tests |
| P4 profile reader | Typed profile/timeline adapter and Leptos reader | Browser error, pagination and profile identity tests |
| P5 posts and media | Typed post adapter; text escaping and media host allowlist | Actual browser image/video/error tests |
| P6 popup and Record | Leptos pages and Rust IndexedDB transactions | Real concurrent, restart, disable/reset tests |
| P7 security/performance | Sender/expiry tests; credential omission, streaming limits, packaged code | Network evidence, malicious-message tests, measured startup/idle baseline |
| P8 cross-browser compatibility | MV3 runtime smoke on full Chromium and Firefox | Full flow on both; individual variants labeled untested |
| P9 landing/store | Repo README, privacy and compatibility docs | Landing page, source archives, store listings and submission |
| P10 public v1 | No public claim | All release gates and actual store publication |

No store URLs or install counts are invented. Publishing signed store packages requires store account access and provider feasibility evidence; these remain external release gates.
