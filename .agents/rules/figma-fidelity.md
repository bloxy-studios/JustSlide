---
trigger: glob
globs: "apps/browser-chrome/src/**, assets/icons/**, design/**, DESIGN.md"
description: Figma is the UI source of truth; geometry checks and screenshots gate every UI change.
---
- Source: Figma file `nw47hqMLTtmBMksJ4BnJKb` ("Slide"), frames 1:6 (1440×1024), 35:13 (1100×720), 30:52 (states). Use Figma Dev Mode MCP (or the Composio Figma connector) to re-read nodes; never guess.
- Assets come from `assets/icons/` (exported from Figma, mapped in `design/figma/asset-export-map.json`). Never draw replacements or reference expiring Figma URLs.
- After any UI change: `bun run build:preview && bun run test:visual`. Zero geometry failures at 1 px is required; record any accepted difference in `docs/design-fidelity.md`.
- Keep the semantic split: top favorites = websites; bottom dock = workspaces. Required labels are tested in `apps/browser-chrome/src/__tests__`.
- The #ACFFF7 backdrop is a user theme example, not a brand colour.
