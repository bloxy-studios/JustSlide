# JustSlide

A macOS-first web browser built on the **system WebKit** where the person and authorised AI agents operate the same browser through one permission-checked command model. Agents work in their own workspaces — creating tabs, pinning, navigating, capturing — without moving the person's tab, workspace or keyboard focus.

> Status (2026-11-13): the native macOS app builds and runs on the owner's Intel Mac (macOS 26.5.1): 5/5 XCTests, 33/33 Rust tests on macOS, real HTTPS browsing, and the chrome matches Figma 1:6 and 35:13 with 0 geometry failures in the real WKWebView. See `docs/progress.md`.

## What works today (verified)
- **Core** (`crates/justslide-core`): workspaces, tabs, favorites, pinned state, settings, persistence (SQLite), session restore, agent grants with scopes, pause/revoke, approvals, rate limits, idempotency, scoped events — 17 scenario + 4 unit tests.
- **Agents** (`crates/justslide-mcp`): a stdio MCP server; end-to-end test drives the real binary against the real core and creates three background tabs while the person's selection and focus stay unchanged; revocation cuts off a live session.
- **Chrome UI** (`apps/browser-chrome`): reproduces Figma frames 1:6 and 35:13 with **0 region-geometry differences > 1 px** at 1440×1024 and 1100×720 (browser preview); strict CSP with zero network requests.

## Native macOS app (verified on Intel, macOS 26.5.1)
`native/macos`: AppKit window, trusted chrome WKWebView, `WebKitBrowserAdapter` (one WKWebView per tab, per-profile data stores, snapshots, popups, dialogs, downloads), C-ABI bridge to the core. Evidence: `docs/evidence/native/2026-11-13-macos-build-and-xctest.md`.

## Quick start
```bash
bun install                      # Bun 1.4.4 (hoisted linker, see bunfig.toml)
cargo test --workspace --all-features
bun run check                    # protocol drift, typecheck, UI tests, production build + bundle guard, Rust tests
bun run build:preview && bun run test:visual   # Figma geometry comparison (needs Python Playwright)
bun run dev:ui                   # http://localhost:41730/preview.html?variant=compact  (DESIGN FIXTURE, not a browser)
```
macOS (Xcode 26, Rust, Bun): `xcodebuild -project native/macos/JustSlide.xcodeproj -scheme JustSlide -destination 'platform=macOS' test`. DEBUG fixture mode: launch with `-JustSlideDesignFixture desktop|compact`.

Agent hookup: see `docs/mcp-and-plugins.md`. Local dev host without the Mac app (test-double engine): `cargo run -p justslide-core --features test-engine --bin justslide-devhost -- --data-dir /tmp/js --grant-ops read.metadata,write.organize --token-file /tmp/js/token`.

## Limitations
Pairing UI/Keychain and approval sheets pending; WebExtensions untested; downloads/history/bookmarks UI pending; App Sandbox off in dev builds; ADS/brand-asset licences need owner review; Chromium is not used (prototype archived on `archive/chromium-foundation`).

## Layout
`crates/` Rust (protocol, core, ffi, mcp) · `apps/browser-chrome/` React chrome · `native/macos/` Swift host · `design/figma/` + `assets/icons/` design sources · `tests/` fixtures + visual harness · `docs/` architecture, ADRs, evidence. Start with `AGENTS.md`, `DESIGN.md`, `docs/architecture.md`.
