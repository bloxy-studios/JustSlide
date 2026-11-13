# Progress checkpoint — 2026-11-13 (session 1, WebKit edition)

Branch `webkit/foundation` (local; not pushed — spec forbids automatic pushes). Chromium prototype preserved on `archive/chromium-foundation` (32ddddc).

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
| 1 | 1440×1024 layout | **pass (preview)**, native not run |
| 2 | 1100×720 compact, pinned collapsed, agent activity | **pass (preview)**, native not run |
| 3 | Favorites vs workspace dock semantics | pass (UI tests) |
| 4 | Active tab ↔ address ↔ page | UI pass; native page not run |
| 5 | Dock stable while tabs scroll | layout pass (preview); scroll interaction not automated |
| 6 | Figma icons in slots | pass (preview); licences pending |
| 7 | Font/ADS status documented | done |
| 8 | Backdrop persists + contrast | core + contrast tests pass; first-paint flash not tested |
| 9 | Native HTTPS navigation | **not run** |
| 10 | Create/pin/move/close/reopen, select workspaces | core pass (test engine) |
| 11 | Persist favorites/tabs/order across restart | core pass |
| 12 | Real load/error state | core pass (test engine); native not run |
| 13 | Popups and downloads | popups core pass; downloads UI/permission not done |
| 14 | Renderer recovery, restoration, form/audio protection | crash→reload + lazy restore pass; form/audio protection not implemented |
| 15 | Agent background tabs, focus unchanged | core + MCP e2e pass (test engine); native typing test not run |
| 16 | Two agents scoped | pass |
| 17 | Screenshot of requested tab | pass with test double; native not run |
| 18 | Revoked token | pass |
| 19 | Hostile site cannot reach native | native test written, not run; CSP pass |
| 20 | Injected instructions grant nothing | pass |
| 21 | Build/test on Mac arch | **not run** |
| 22 | Preview vs native distinguished | done |
| 23 | Accessibility/keyboard | labels, focus states, keyboard reorder/menu; no automated a11y audit yet |
| 24 | Performance measurements | core baseline only |
| 25 | Release bundle hygiene | pass (`check-release-bundle.mjs`) |

## Blockers
1. **No Mac build path**: SimBridge has only GoldPilot registered; JustSlide needs cloning on the Mac, registration + Trust builds, and grant reconnection; macOS-destination support in `build_start` is unverified. Rust and Bun must be installed on the Mac.
2. **Owner decisions**: host change (ADR 0001); ADS/Atlassian licence use; Nucleo icon licence; workspace brand icons; traffic-light placement (E-5); collapsed sidebar design (E-7).
3. GitHub write from this environment previously returned 403; this session did not push.

## Resume
```bash
git clone <bundle or remote> JustSlide && cd JustSlide && git checkout webkit/foundation
bun install && cargo test --workspace --all-features && bun run check
bun run build:preview && python3 tests/visual/capture_and_compare.py
cd native/macos && xcodebuild -project JustSlide.xcodeproj -scheme JustSlide -destination 'platform=macOS' test
```
