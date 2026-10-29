> Historical attachment. The user’s later instructions in [implementation-direction.md](implementation-direction.md) supersede its reader-page, activation, logo and standalone-page assumptions.

"
# LimitlessX
**Brand Identity, Product Specification & Rust Engineering Plan**

Version: 0.3  
Status: Planning  
Repository: `OnurByte/LimitlessX` (proposed)  
License: MIT  
Primary language: Rust  
Application runtime: WebAssembly  
Targets: Firefox and Chromium browsers  
Product language: English

---

## 1. Product identity

**Brand:** LimitlessX

**Primary tagline:**

### No VPN. No proxy setup. No DPI hacks.

**Product description:**

View region-restricted public content on X through an alternative source, without configuring network-circumvention tools.

**Short description:**

A Rust-powered, cross-browser extension for viewing accessible region-restricted X accounts and posts.

### Character

- Independent and unconventional.
- Minimal, direct, and recognizable.
- Technically transparent.
- Privacy-conscious without unverifiable anonymity claims.
- Strong identity without exaggerated marketing.

The name LimitlessX communicates unrestricted exploration, but the application must clearly distinguish its intended purpose from guaranteed censorship circumvention.

There is no claim that the extension can access every restricted account.

---

## 2. Brand design

### Logo concept

An original geometric mark combining a continuous loop with a sharply interrupted X-shaped intersection.

It should suggest continuity despite an obstruction.

Requirements:

- Distinctive SVG master.
- Legible at 16, 32, 48, and 128 pixels.
- Monochrome.
- Slight material depth permitted.
- No traditional lock, shield, or VPN symbol.
- No animations or decorative glow.

### Design system

| Element | Standard |
|---|---|
| Background | Black / white |
| Typography | Helvetica Neue, Helvetica, Arial |
| Primary text | High contrast |
| Secondary text | Neutral grayscale |
| Controls | 8 px radius |
| Panels | 12 px radius |
| Spacing | 4 px increments |
| Themes | Follow system |
| Icons | Original SVG assets |

Browser compatibility icons retain their official colors.

### Writing style

Short, direct English.

Avoid artificial urgency, technical grandstanding, fake achievements, unnecessary explanations, and exaggerated privacy promises.

---

## 3. Rust-first architecture

### Technology decisions

| Layer | Technology |
|---|---|
| Main language | Rust |
| Compilation target | `wasm32-unknown-unknown` |
| UI | Leptos |
| WebAssembly bindings | wasm-bindgen |
| Browser interfaces | web-sys + narrowly scoped bindings |
| HTTP | web-sys Fetch API |
| JSON serialization | serde |
| Data validation | Rust typed deserialization + semantic checks |
| Error handling | thiserror |
| Local persistent data | IndexedDB |
| Build orchestration | Cargo xtask |
| WASM optimization | wasm-opt |
| Unit tests | cargo test |
| Browser integration tests | WebDriver / Playwright harness |
| CI | GitHub Actions |

### Language policy

- No TypeScript.
- No React.
- No Node-based application framework.
- No unnecessary JavaScript dependencies.
- Rust owns application logic, API interpretation, state transitions, view models, and rendering components.
- Small packaged JavaScript adapters are permitted only where required for browser interoperability.
- Generated JavaScript from wasm-bindgen is expected and permitted.

WXT is removed from the architecture.

### Runtime boundaries

**Rust core**

Owns profile and post models, API response normalization, eligibility rules, errors, identifiers, and local statistics logic.

**Rust/WASM UI**

Uses Leptos for the extension popup, options, and alternative reader.

**Content-script adapter**

A minimal JavaScript module observes X navigation and restriction notices, inserts an entry point, and communicates with the extension runtime.

**Background adapter**

Registers browser lifecycle and messaging listeners, initializes the required WASM runtime, and delegates validated operations to Rust.

No application business rules should be duplicated in JavaScript.

---

## 4. Rendering strategy

### Extension-origin reader

Use a packaged extension page to host the Rust/WASM reader.

The content script can insert an extension-origin iframe near a withheld account notice.

This separates the Leptos application from X's JavaScript and CSS environment.

Benefits:

- Rust UI does not need to run directly inside X's content-script environment.
- Leptos controls the entire reader.
- Styling remains isolated.
- The extension can apply its own WASM-compatible security policy.

### Technical conditions

- The reader must be a packaged local resource.
- Only required resources are web-accessible.
- Cross-origin messaging uses explicit origins, validated message structures, and short-lived session context.
- X page content is never treated as trusted code.
- The reader must not inherit X session credentials.
- If embedded rendering fails due to browser restrictions, provide an extension-tab reader fallback.

Embedding and background WASM initialization must be verified in both browser families before building the complete interface.

### Content Security Policy

Use packaged scripts and the minimum necessary `wasm-unsafe-eval` allowance.

Remote executable JavaScript is prohibited.

---

## 5. Core feature: restricted account reader

### Entry points

1. An X profile displaying a confirmed country-specific withheld notice.
2. A direct X post with a confirmed regional restriction.
3. A manually submitted X URL in the popup.

### Flow

1. Detect the page and restriction state.
2. Offer the LimitlessX action.
3. Wait for user activation.
4. Request public data through FxTwitter.
5. Validate and normalize the response in Rust.
6. Render available data in the Leptos reader.
7. Record successful deliberate views locally, if enabled.

### Account information

Display only fields actually returned by the provider:

- Display name and handle.
- Avatar and biography.
- Public posts.
- Media.
- Publication dates.
- Pagination when available.

Never fabricate missing information.

### Restrictions

A geographically withheld account, suspended account, deleted account, and protected account are different states.

Do not treat a generic API failure as evidence of a regional block.

---

## 6. FxTwitter integration

**Primary provider:** FxTwitter API v2.

The provider adapter will be written in Rust.

Responsibilities:

- Request construction.
- Fetching and timeouts.
- Rate-limit handling.
- Strict response parsing.
- Normalized Rust data models.
- Error classification.
- Pagination.
- Explicit provider attribution.

The provider interface must remain replaceable without rewriting the UI.

### Feasibility gate

Before implementing the full product, confirm that FxTwitter can retrieve representative withheld public content.

Test normal accounts, regional withholding, protected content, suspension, deletion, and API errors.

If actual withheld content cannot be retrieved, reconsider the source strategy before making accessibility claims.

The extension will not use users' X authentication cookies or credentials.

---

## 7. Private statistics: The Record

The Record measures distinct restricted public content successfully opened through LimitlessX.

### Main metrics

**Restricted profiles viewed**

The number of unique accounts with verified regional restriction states deliberately opened through the reader.

**Posts viewed**

The number of unique posts deliberately opened in an eligible restricted-content flow.

Automatically fetched timelines do not count as viewed posts.

### Proposed appearance

```text
THE RECORD                             ⁜

20                     10
restricted profiles    posts
viewed                 viewed

Since installation
Stored on this device

Clear record
```

### Storage design

- IndexedDB as the source of truth.
- Atomic unique-key insertion for deduplication.
- Per-install local identifiers derived from normalized account IDs and post IDs.
- No plaintext browsing history required.
- No remote analytics.
- No account registration.
- No telemetry.
- Statistics disabled on user request.
- Clear record removes the stored ledger.

Reopening the same content never increases its unique count.

Separate tabs and browser restarts must not cause duplicate counting.

### Scope

The first release includes only two counters.

No leaderboards, achievements, streaks, gamification, country rankings, or public statistics.

---

## 8. Extension popup

### Suggested composition

```text
LIMITLESSX                              ∞

No VPN. No proxy setup.
No DPI hacks.

CURRENT PAGE

@account
Restricted in your region

[ View public content ]

────────────────────────────────

THE RECORD

20                      10
restricted profiles     posts
viewed                  viewed

Stored locally · Since installation

Settings                       GitHub
```

All data in the example is illustrative.

### Behavior

On X, the popup identifies the current supported context.

On other websites, it offers a field for an X URL or account handle.

The main action must use a meaningful label rather than ambiguous generic text.

The popup is built entirely with Leptos components.

---

## 9. Supported browsers

### Official installation badges

Display the official Chrome Web Store and Firefox Add-ons badges prominently.

### Chromium family

- Chrome
- Brave
- Opera
- Edge
- Vivaldi

### Firefox family

- Firefox
- Zen
- Floorp
- LibreWolf
- Waterfox

Use recognizable browser icons beneath the official installation badges.

Do not imply every listed browser has been tested until verification is complete.

### Distribution model

Generate independent Chromium and Firefox Manifest V3 packages from the same Rust workspace.

Where API behavior differs, handle the difference in a small compatibility adapter.

No separate application implementations.

---

## 10. Workspace structure

```text
LimitlessX/
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml
├── crates/
│   ├── core/
│   │   ├── models/
│   │   ├── restrictions/
│   │   └── errors/
│   ├── fxtwitter/
│   ├── storage/
│   ├── runtime/
│   ├── ui/
│   │   ├── popup/
│   │   ├── reader/
│   │   └── options/
│   └── xtask/
├── extension/
│   ├── adapters/
│   │   ├── background.js
│   │   └── content.js
│   ├── manifests/
│   │   ├── chromium.json
│   │   └── firefox.json
│   └── static/
│       ├── brand/
│       └── styles/
├── tests/
│   ├── fixtures/
│   ├── browser/
│   └── integration/
├── docs/
│   ├── architecture.md
│   ├── compatibility.md
│   ├── privacy.md
│   └── brand.md
├── .github/
│   └── workflows/
└── README.md
```

The Rust workspace is authoritative. The JavaScript adapters remain small, explicitly reviewed, and independent of application state models.

---

## 11. Build system

Use Cargo xtask as the single build orchestration entry point.

Proposed CLI contract:

```sh
cargo xtask build chromium
cargo xtask build firefox
cargo xtask test
cargo xtask package
```

These are planned project commands, not commands supplied by an existing framework.

The build pipeline will:

1. Compile the relevant Rust crates to WASM.
2. Generate interoperability code using wasm-bindgen.
3. Optimize release WASM.
4. Copy packaged UI and static resources.
5. Generate browser-specific manifests.
6. Validate resources, permissions, and CSP.
7. Produce separate extension archives.

### Reproducibility

- Commit Cargo.lock.
- Pin Rust toolchain.
- Version required build tools.
- Generate release artifacts in CI.
- Preserve source information needed for store review.
- Never load remote executable code.

---

## 12. Performance

Rust/WASM does not automatically make an extension faster than JavaScript.

### Goals

- Small startup payload.
- No repeated WASM initialization for the same active UI context.
- Lazy initialization of the reader.
- Minimal JavaScript injected into X.
- No continuous full-page DOM scanning.
- No speculative API calls by default.
- Debounced navigation observation.
- No background CPU activity when idle.

### Measurements

Track:

- Packaged size.
- WASM initialization time.
- Popup interactive latency.
- Reader open latency.
- API response handling overhead.
- Idle resource consumption.
- Compatibility regressions.

Set numeric performance budgets only after obtaining baseline measurements.

---

## 13. Privacy and security

- No backend owned by LimitlessX.
- No remote telemetry.
- No transfer of X session tokens.
- No collection of passwords or private messages.
- No broad browsing-history permissions.
- No external code execution.
- No default automatic lookup of every X account encountered.
- Clear explanation of what FxTwitter receives.
- Local statistics remain local.
- Explicit reset and disable controls.
- No content spoofing when the source fails.

Avoid declaring the product anonymous or untraceable.

---

## 14. Landing page

### Hero

**LimitlessX**

No VPN. No proxy setup. No DPI hacks.

View region-restricted public X content through an alternative source, when available.

Official Chrome and Firefox install badges.

Below them, recognizable compatibility icons for the additional browser families.

### Sections

**How it works**  
A three-step explanation of detection, alternative lookup, and reading.

**The Record**  
A small illustration of the private local viewing counters.

**Privacy**  
Direct explanations of local data and third-party API requests.

**Open source**  
Repository, license, releases, and contributing instructions.

Avoid inflated marketing statistics, fake social proof, animation-heavy pages, and decorative security claims.

---

## 15. Release roadmap

| Phase | Deliverable |
|---|---|
| P0 | FxTwitter withheld-content feasibility |
| P1 | Rust/WASM cross-browser runtime proof |
| P2 | LimitlessX brand and design system |
| P3 | Restriction detection and content-script adapter |
| P4 | Rust alternative account reader |
| P5 | Rust post and media reader |
| P6 | Leptos popup and The Record |
| P7 | Security and performance verification |
| P8 | Firefox and Chromium compatibility |
| P9 | Landing page and store submission |
| P10 | Public v1.0 |

### Release gates

Before v1.0:

- Withheld content capability is verified against real examples.
- Chrome and Firefox can load the Rust/WASM reader.
- Background initialization and messaging survive worker restarts.
- Embedded rendering or its tab fallback works.
- Unique local counters remain correct under concurrent use.
- Privacy claims match actual network and storage behavior.
- Browser variants are labeled according to their verified compatibility.
- All executable code is packaged locally.

---

## 16. Non-goals

LimitlessX is not:

- A VPN.
- A network proxy service.
- A DPI bypass implementation.
- An anonymous browser.
- A full X client.
- An X account-unlocking service.
- A private-account viewer.
- A content archive.
- A social analytics platform.

It is a focused Rust-powered browser extension for accessing public content when an alternative source makes that content available.

---

## Final product direction

**LimitlessX should make a restricted-content lookup feel like an ordinary browser capability.**

The interface remains quiet, the data stays transparent, and the application does not need a separate account or a managed backend.

Rust owns the product. Browser-specific adapters only connect it to the web extension platform.
"