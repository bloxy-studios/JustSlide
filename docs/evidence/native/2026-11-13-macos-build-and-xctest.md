# Native macOS evidence — 2026-11-13

Host: owner's MacBook Pro, **Intel x86_64**, macOS **26.5.1 (25F80)**, Xcode 26.6 (17F113). Driven remotely through SimBridge (no shell); builds sync `origin/webkit/foundation` first.

## Build — job `12Khw3fNF2gYV9Uuh3eTsw`
- SHA `2b7746d` · scheme `JustSlide` · Debug · **ok: true** · 578 s (cold: rustup toolchain + crates + Bun install + Swift).
- Errors: 0. Warnings: 4 × `asset needs owner/legal review` (expected, from `check-release-bundle.mjs`).
- Product: `JustSlide.app` (Debug, x86_64).

## XCTest — job `3mi5hpzzeuVsiXae8mnx9Q`
- SHA `2b7746d` · **5 passed, 0 failed, 0 skipped** (5.4 s) · result "Passed".

| Test | Result | What it proves on real WebKit |
|---|---|---|
| testBackgroundViewNeverStealsKeyboardFocus | pass | loading a tab that is not attached keeps the person's text field first responder; only `show(focus:true)` focuses a page |
| testProfilesUseSeparateWebsiteDataStores | pass | a cookie set in `WKWebsiteDataStore(forIdentifier: A)` is invisible in store B |
| testExternalPagesHaveNoPrivilegedBridge | pass | `window.webkit.messageHandlers.justslide` is `undefined` in page views |
| testBackgroundSnapshotBehaviourIsReported | pass | `JUSTSLIDE-EVIDENCE background-snapshot: image 520x560 possiblyStale=true` — WebKit returns an image for a view that is in no window |
| testCoreRoundTripOverFFI | pass | Swift → C ABI → Rust core → engine command (`createView https://example.com/`) round trip |

Log noise worth knowing: WebContent processes log sandbox lookups (pasteboard, launchservicesd, RunningBoard assertions) because the unsandboxed dev app lacks WebKit's private entitlements; none affected results.

Consequence: `backgroundCapture` capability changed from `unknown` to `partial` (image returned; pixels may be stale for views not in a window — callers receive `possiblyStale`).

## Native Figma comparison — app session `xH8flqF4sOW0qlUS`, SHA `b9d365f`
DEBUG build, `Develop ▸ Design Fixture …` (design fixture rendered by the **real macOS WKWebView**; SF Pro via the ADS font stack). Display: 1920×1049 visible, backing scale 1.0 → compared against the Figma @1x renders. Self-check: `native/macos/JustSlide/NativeFixtureCheck.swift` (same selectors and 1 px tolerance as the Linux harness; pixels = max channel |Δ| > 32/255).

| Frame | Content size | Geometry failures | Regions | Favorites | Dock | Visible rows | Whole-frame pixels >32 | Mean |Δ| |
|---|---|---|---|---|---|---|---|---|
| 35:13 compact | 1100×720 | **0** (all Δ = 0 px) | 10/10 | 13/13 | 6/6 | 8/8 | 0.88 % | 0.99 |
| 1:6 desktop | 1440×1024 | **0** (all Δ = 0 px) | 9/9 | 13/13 | 6/6 | 15/15 | 0.71 % | 0.84 |

Font evidence: the selected-tab title ("Slide - Figma", 590 13/18) measures **166 px**, identical to the Figma text box (`I30:103;30:85`, 166×18), i.e. WebKit resolved the ADS stack to SF Pro.
Snapshots saved on the Mac: `~/Library/Application Support/JustSlide Dev/evidence/native-fixture-compact@1x.png` (sha256 06c77336…), `native-fixture-desktop@1x.png` (sha256 01b2d7f7…).

Highest per-region pixel deltas: address bar 8.6 % (desktop) / 3.4 % (compact), tab list 4.7–5.8 %, favorites 4.9 % — glyph and icon rasterisation (Figma's renderer vs CoreText/WebKit), not position (geometry is exact).

Limitation: SimBridge could not take its own window screenshot — the app was launched in the background, macOS 14+ refused activation, and the window stayed off the active Space (`onActiveSpace=0, occlusion=hidden`). The window now sets `.moveToActiveSpace`; a person-visible screenshot needs the Mac unlocked/on the app's Space.

## Rust suite on macOS — `project_cargo test`, job `-f4h1sl0aM73jLQjXvm2yw`, SHA `b9d365f`
`cargo test` exit 0 (compile 6 m 25 s, Intel x86_64): protocol 6, core unit 4, **core scenarios 17**, C ABI 1, MCP endpoint 3, **MCP e2e 2** (real `justslide-mcp` binary ↔ real core over a macOS Unix socket: three background tabs + pin + capture with the person's window/focus unchanged; revoked token refused) — **33/33 pass**.

## Live HTTPS navigation — app session `ju1yJnBMNp9incSb`, SHA `72380f3`
`Develop ▸ Live Navigation Check` → core `tabs.open` ×2 → WebKit adapter → real network:
```
navigationStarted/Committed/Finished  example.com     title "Example Domain"   (0.9 s)
navigationStarted/Committed/Finished  www.apple.com   title "Apple"            (5.0 s)
faviconChanged ×2
JUSTSLIDE-LIVE-CHECK tabs: [{url https://example.com/, title "Example Domain", loadState loaded, secure true, active false},
                            {url https://www.apple.com/, title "Apple", loadState loaded, secure true, active true}]  revision 14
```
Titles, URLs, security and load state in the core come from WebKit callbacks, not from the UI.
Not run natively yet: back/forward/reload clicks, app-restart restoration (covered on macOS by the core persistence test), downloads, popups, crash recovery.
