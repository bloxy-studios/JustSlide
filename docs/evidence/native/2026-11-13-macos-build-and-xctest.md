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
