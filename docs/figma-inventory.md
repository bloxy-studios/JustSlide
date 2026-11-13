# Figma inventory

Retrieved 2026-11-13 05:31 UTC through the **Composio Figma connector** (account connected by the owner; REST `GET /v1/files/:key/nodes`, `GET /v1/images/:key`). Antigravity's Figma Dev Mode MCP was not available in this environment; the connector returned the same REST data, actual renders and asset exports. File version `2420685396735920550`, lastModified `2026-11-11T19:26:40Z`, role `owner`.

| Node | Name | Size | Reference render | Notes |
|---|---|---|---|---|
| 1:6 | Desktop - 1 | 1440×1024 | `design/figma/renders/1-6@{1,2}x.png` | main frame; padding 12, gap 12; backdrop `#ACFFF7` (variable-bound) |
| 1:9 | Sidebar | 240×1000 | `1-9@2x.png` | column, gap 8 |
| 6:17 | Navigation Controls | 240×40 | `6-17@2x.png` | toggle 23:3; back 23:4, forward 23:5, reload 23:6 (32×32, gap 4) |
| 7:46 | Address Bar | 240×32 | `7-46@2x.png` | white, r8, text "figma.com" SF Pro 510 12/18 `#0E1B1B` |
| 7:87 | Website favorites / 12 sites | 240×118 | `7-87@2x.png` | 12 tiles 43×34 r8 + Add (dashed 2/4 `#393939`) |
| 25:4 | Pinned Section | 240×208 | `25-4@2x.png` | trigger 7:88 (28h; "Pinned" 590 12/18; count 510 11/18 op .6; chevron) + 5 items 32h |
| 12:139 | Section separator | 224×0 | — | `#627F87` 1 px, inset 8 |
| 20:82 | Browser Sidebar | 240×490 | `20-82@2x.png` | New Tab 12:140 + tab list 12:289 (vertical scroll) |
| 30:103 | Tab / Slide - Figma / Selected | 240×32 | — | instance of 30:78 |
| 12:290 | Workspace footer | 240×64 | `12-290@2x.png` | heading "WORKSPACES" 590 10/16 ls .6 op .68 + "Personal"; Downloads, Personal (selected), Codex, Claude Code, ChatGPT, New workspace |
| 1:7 | Web Page Window | 1164×1000 | `1-7@2x.png` | white r16 |
| 30:52 | Slide / Interaction states | 400×790 | `30-52@{1,2}x.png` | tab states 30:56/67/78/89 (set 30:100), tooltip 30:122, behaviour notes |
| 35:13 | Slide / Compact - agent working example | 1100×720 | `35-13@{1,2}x.png` | pinned collapsed; agent card 35:313 (240×56 r10); activity dot 35:322 (7×7 `#128066`, 1.5 px backdrop ring) |

Hidden tooltip instances 34:31…34:121 (Figma component 30:122) define every hover label: "Open YouTube website" … "Add website favorite", "Downloads", "Switch to Personal|Codex|Claude Code|ChatGPT workspace", "Create a new workspace".

Component descriptions (from the file):
- **Slide / Tab** — "Compact 32 px tab row. Selected persists; hover is lighter; keyboard focus uses a distinct outline. The close control appears on hovered or selected tabs."
- **Slide / Tooltip** — "Hover label that distinguishes website launchers from workspace switchers."

## Assets
44 icon slots exported as SVG (`assets/icons/{ui,favorites,favicons,workspaces}/`), map in `design/figma/asset-export-map.json`. Four image fills exist in the file but none are used by the required frames. Colours and sizes are set on layers (no published styles); the backdrop and page-window fills are bound to Figma variables.
