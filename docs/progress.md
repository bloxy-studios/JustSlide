# Progress checkpoint — 2026-11-13 (session 2, WebKit edition)

Branch `webkit/foundation` is on GitHub (pushed with the owner's go-ahead; `main` untouched). The 20 PNG evidence files are not on GitHub (text-only push channel) — Figma renders are mirrored as base64 in `design/figma/renders-b64/`. Chromium prototype preserved on `archive/chromium-foundation` (32ddddc).

## What changed
- `crates/justslide-protocol` — canonical registry v1.1 (40 commands, 17 events, `exposure` ui/mcp).
- `crates/justslide-core` — the single authority: state, SQLite migrations, grants/scopes/pause/revoke, approvals, idempotency, rate limits, event log, Unix-socket IPC, engine abstraction, test-double engine, dev host, benchmark.
- `crates/justslide-ffi` — C ABI for the macOS host (+ header).
- `crates/justslide-mcp` — stdio MCP bridge ported to the shared protocol (session-nonce request ids fix an idempotency collision across sessions).
- `apps/browser-chrome` — React 19 + ADS tokens chrome built to Figma; Slide theme layer; dev fixture preview; production bundle with hash-pinned CSP and a release guard.
- `native/macos` — Swift host: `WebKitBrowserAdapter`, chrome host (schemes + reply handler), window, menus, `CoreBridge`, XCTests, Xcode project + XcodeGen spec, build scripts.
- Design evidence: `design/figma/` (node tree, renders), `assets/icons/` (44 SVGs), `docs/figma-*`, `docs/evidence/visual/2026-11-13/`.

## Spec §20 acceptance status
| # | Test | Status |
|---|---|---|
| 1 | 1440×1024 layout | **pass native** (0 geometry failures, 0.71 % pixels) + preview |
| 2 | 1100×720 compact, pinned collapsed, agent activity | **pass native** (0 geometry failures, 0.88 % pixels) + preview |
| 3 | Favorites vs workspace dock semantics | pass (UI tests) |
| 4 | Active tab ↔ address ↔ page | UI pass; native page not run |
| 5 | Dock stable while tabs scroll | layout pass (preview); scroll interaction not automated |
| 6 | Figma icons in slots | pass (preview); licences pending |
| 7 | Font/ADS status documented | done |
| 8 | Backdrop persists + contrast | core + contrast tests pass; first-paint flash not tested |
| 9 | Native HTTPS navigation | **pass native** (example.com, apple.com via core + WebKit); back/forward/reload not driven yet |
| 10 | Create/pin/move/close/reopen, select workspaces | core pass (test engine) |
| 11 | Persist favorites/tabs/order across restart | core pass |
| 12 | Real load/error state | core pass (test engine); native not run |
| 13 | Popups and downloads | popups core pass; downloads UI/permission not done |
| 14 | Renderer recovery, restoration, form/audio protection | crash→reload + lazy restore pass; form/audio protection not implemented |
| 15 | Agent background tabs, focus unchanged | MCP e2e pass on macOS (test engine) + native XCTest: background WKWebView never takes first responder; live typing test not run |
| 16 | Two agents scoped | pass (Linux + macOS) |
| 17 | Screenshot of requested tab | test-double pass; native: WebKit snapshots a background view (image, possiblyStale) |
| 18 | Revoked token | pass |
| 19 | Hostile site cannot reach native | **pass native** (no `justslide` handler in page views) + CSP pass |
| 20 | Injected instructions grant nothing | pass |
| 21 | Build/test on Mac arch | **pass** Intel x86_64 (Xcode build, 5/5 XCTest, 33/33 cargo); arm64 unverified |
| 22 | Preview vs native distinguished | done |
| 23 | Accessibility/keyboard | labels, focus states, keyboard reorder/menu; no automated a11y audit yet |
| 24 | Performance measurements | core baseline only |
| 25 | Release bundle hygiene | pass (`check-release-bundle.mjs`) |

## Blockers
1. **SimBridge window screenshots**: the app launches in the background, macOS refuses activation, and the window stays off the active Space, so SimBridge can't capture it. Native visuals are verified by the in-app self-check instead; snapshots are saved on the Mac (`~/Library/Application Support/JustSlide Dev/evidence/`). For a person-visible capture, unlock the Mac and switch to the JustSlide window.
2. **Owner decisions**: host change (ADR 0001); ADS/Atlassian licence; Nucleo icon licence; workspace brand icons; traffic-light placement (E-5); collapsed sidebar design (E-7).
3. Not yet native: pairing UI + Keychain, approval sheets, downloads UI, back/forward/reload driven by clicks, restart restore in the app, MV3 extensions, Retina (@2×) comparison, performance on the Mac.

## Resume
```bash
git clone <bundle or remote> JustSlide && cd JustSlide && git checkout webkit/foundation
bun install && cargo test --workspace --all-features && bun run check
bun run build:preview && python3 tests/visual/capture_and_compare.py
cd native/macos && xcodebuild -project JustSlide.xcodeproj -scheme JustSlide -destination 'platform=macOS' test
```
