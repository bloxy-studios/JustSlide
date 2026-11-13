# Plan (ordered milestones)

| Milestone | Scope | State 2026-11-13 |
|---|---|---|
| M0 Discovery | repo/toolchain/Mac, Figma extraction, ADS MCP, licences, rules, ADRs | **done** (Figma via Composio; Mac via SimBridge health only) |
| M1 Native WebKit spike | build `.app`, run `JustSlideTests` (focus, isolation, bridge isolation, background capture), screenshots | **done** on Intel macOS 26.5.1 (build ok, 5/5 XCTest, native self-check) |
| M2 ADS visual shell | React/ADS chrome, Figma geometry gate at 1440×1024 / 1100×720 | **done** — 0 geometry failures in preview and in the native WKWebView |
| M3 Real basic browser | address/nav/tabs/favorites/pinned/workspaces/persistence | core + UI + adapter written; core verified with test engine; native pending |
| M4 MCP + live state | pairing, grants, activity, pause/revoke, background tabs without focus theft | protocol harness **passes**; pairing UI + Keychain pending |
| M5 Polish | history/bookmarks/downloads UI, find/zoom, private windows, native menus, approvals UI | partial (menus, downloads sink) |
| M6 WebExtensions | WKWebExtensionController experiment + matrix | not started |
| M7 Performance/release | native measurements, signing/notarisation, CI | core baseline only |

Next three actions: (1) pairing UI + Keychain token storage + native approval sheets so a real Claude Code/Codex client can connect to the running app; (2) drive the live app end to end (address bar typing, back/forward, agent tabs while typing) once the window is on the active Space; (3) downloads/history UI and restart-restore check in the app.
