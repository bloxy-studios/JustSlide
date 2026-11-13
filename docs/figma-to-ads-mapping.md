# Figma → implementation mapping

| Figma Node | Intended Function | Figma Geometry | ADS Component/Primitive | Token/Type Style | Interaction | Asset Path | Implementation File | Verification |
|---|---|---|---|---|---|---|---|---|
| 1:6 / 35:13 | Window | 1440×1024 / 1100×720, pad 12, gap 12 | Compiled layout | `space.150`, `--slide-backdrop` | backdrop customizable | — | `components/SlideWindow.tsx` | geometry 0 fail |
| 1:9 | Sidebar | 240, gap 8 | nav landmark | `space.100` | hover labels host | — | `SlideWindow.tsx` | geometry |
| 23:3 | Toggle sidebar | 32×32, icon 16 | button | `color.icon`, hover `neutral.subtle.hovered` | `settings.update{sidebarCollapsed}` | `ui/sidebar-toggle.svg` | `NavigationControls.tsx` | unit + geometry |
| 23:4–23:6 | Back/Forward/Reload | 3×32, gap 4 | button | `radius.large` | `page.back/forward/reload/stop`; disabled from WebKit state | `ui/back,forward,reload.svg` | `NavigationControls.tsx` | geometry |
| 7:46 | Address/search | 240×32 r8 | input | `elevation.surface`, `--slide-font-field` | host at rest, full URL editing, Enter→`page.navigate`/`tabs.open` | — | `AddressSearchBar.tsx` | unit |
| 7:47–7:86 | Website favorites | 43×34 r8, icon 18 | button list | `elevation.surface`, `radius.large` | `tabs.open{url}`; label "Open X website" | `favorites/*.svg` (preview only) | `FavoriteGrid.tsx` | unit + geometry (13/13) |
| 20:19 | Add favorite | 43×34 dashed 2/4 | button | `color.icon.subtle` | `favorites.add` from active tab | `ui/add-favorite.svg` | `FavoriteGrid.tsx` | geometry |
| 7:88 | Pinned trigger | 240×28 | button `aria-expanded` | `--slide-font-section-title`, count `--slide-font-count` | `ui.setPinnedCollapsed` (persisted) | `ui/pin.svg`, `ui/chevron-*.svg` | `TabSections.tsx` | unit |
| 10:2… | Pinned rows | 240×32 | list rows | `--slide-font-row` | same as tabs | favicons (runtime) | `BrowserTabRow.tsx` | geometry |
| 12:139 | Separator | 224×1 | hr | `color.border` | — | — | `TabSections.tsx` | visual |
| 12:140 | New Tab | 240×32 | button | `--slide-font-row-strong` | `tabs.open{about:blank}` | `ui/new-tab.svg` | `TabSections.tsx` | geometry |
| 12:289 | Tab list (scrolls) | flex | scroll container | — | independent scroll; footer anchored | — | `TabSections.tsx` | geometry both sizes |
| 30:100 | Tab row states | 240×32 | button + close + ADS DropdownMenu | `elevation.surface`, `border.bold`, `border.focused` | activate, middle-click close, ⌥↑/↓ reorder, ContextMenu/⇧F10 menu | `ui/close.svg` | `BrowserTabRow.tsx` | unit + geometry |
| 35:313 | Agent activity | 240×56 r10 | region `aria-live` | `border.bold`, `--slide-font-caption` | `agents.pause/resume` | `workspaces/claude-code-activity.svg` | `AgentActivitySummary.tsx` | unit + geometry |
| 12:290 | Workspace footer | 240×64 | footer, list | `--slide-font-overline` | `workspaces.switch`, `workspaces.create` | `workspaces/*.svg`, `ui/downloads.svg`, `ui/new-workspace.svg` | `WorkspaceDock.tsx` | unit + geometry (6/6) |
| 35:322 | Activity dot | 7×7 | span | `background.success.bold` | only for working agent | — | `WorkspaceDock.tsx` | unit |
| 30:122 | Hover label | 224×32 r8 | role=tooltip | `background.neutral.bold`, `text.inverse` | 300 ms delay, Esc | — | `HoverLabel.tsx` | manual (preview) |
| 1:7 | Page window | flex, r16 | main landmark + native anchor | `radius.xxlarge`, `elevation.surface` | reports bounds; native WKWebView placed above | — | `WebContentHostAnchor.tsx`, `BrowserWindowController.swift` | geometry; native pending |
