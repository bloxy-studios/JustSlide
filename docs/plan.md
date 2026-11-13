# Plan (ordered milestones)

| Milestone | Scope | State 2026-11-13 |
|---|---|---|
| M0 Discovery | repo/toolchain/Mac, Figma extraction, ADS MCP, licences, rules, ADRs | **done** (Figma via Composio; Mac via SimBridge health only) |
| M1 Native WebKit spike | build `.app`, run `JustSlideTests` (focus, isolation, bridge isolation, background capture), screenshots | **blocked**: JustSlide not registered in SimBridge; macOS destination support unverified |
| M2 ADS visual shell | React/ADS chrome, Figma geometry gate at 1440×1024 / 1100×720 | **done in preview** (0 geometry failures); native screenshots pending M1 |
| M3 Real basic browser | address/nav/tabs/favorites/pinned/workspaces/persistence | core + UI + adapter written; core verified with test engine; native pending |
| M4 MCP + live state | pairing, grants, activity, pause/revoke, background tabs without focus theft | protocol harness **passes**; pairing UI + Keychain pending |
| M5 Polish | history/bookmarks/downloads UI, find/zoom, private windows, native menus, approvals UI | partial (menus, downloads sink) |
| M6 WebExtensions | WKWebExtensionController experiment + matrix | not started |
| M7 Performance/release | native measurements, signing/notarisation, CI | core baseline only |

Next three actions: (1) owner registers JustSlide in SimBridge (or runs `xcodebuild` locally) and installs Rust + Bun on the Mac; (2) fix first-compile issues and run `JustSlideTests`; (3) capture native 1440×1024 / 1100×720 screenshots in fixture mode and compare.
