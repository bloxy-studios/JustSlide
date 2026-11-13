# JustSlide design contract

This is JustSlide's own design contract (not an Atlassian document). Source of truth: Figma file **Slide** `nw47hqMLTtmBMksJ4BnJKb`, version `2420685396735920550`, last modified 2026-11-11T19:26:40Z. Inventory: `docs/figma-inventory.md`; machine-readable: `docs/figma-manifest.json`.

## Layout (both window sizes)
- Window: user backdrop, 12 px inset, 12 px gap. Sidebar 240 px; page window fills the rest, 16 px radius, white surface.
- Sidebar, top→bottom, 8 px gaps: navigation row 40 (32 px targets) · address bar 32 · favorites grid 5×43×34 tiles, 4 px gaps/padding · pinned section (28 trigger + 32 rows, 4 gaps) · separator inset 8 · New Tab 32 · **independently scrolling** tab list · conditional agent card 56 · workspace footer 64 (anchored).
- Rows use flex centring (Figma's 7/5 px vertical paddings are centring, not tokens).

## Semantics (tested)
Top favorites are **websites** (`Open ChatGPT website`). The bottom dock **switches workspaces** (`Switch to ChatGPT workspace`). An agent-workspace icon never implies a connection; the activity card and dot appear only when the core reports a working/paused agent.

## States (Figma 30:100)
Default · Hover (white, close visible) · Selected (white, 1 px ink border, weight 590, 16 px favicon, close visible) · Keyboard focus (white, 2 px `color.border.focused`). Hover labels (30:122): sidebar-wide dark label, 8 px inset, below favorites rows / above the dock.

## Tokens
Components use ADS `token()`s only. Two app layers:
1. **Slide theme layer** (`apps/browser-chrome/src/theme/slide-theme.ts`): sets ADS token *values* for text/border/focus/neutral-bold/success-bold from the user backdrop. At `#ACFFF7` it reproduces Figma exactly (`#0E1B1B`, `#304B48`, `#45615F`, `#627F87`, `#10574F`, `#128066`); other backdrops rotate hue/scale saturation (contrast-tested). Dark mode uses ADS dark values.
2. **Slide type/size variables** (`--slide-font-*` in `src/styles.css`): SF Pro scale 510/590 at 13/18, 12/18, 11/18, 11/16, 10/16 on the ADS font-family token.

Spacing → `space.050/100/150`; radii → `radius.large` (8), `radius.medium` (6), `radius.xxlarge` (16); card radius 10 px is app-specific.

## Typography
The ADS family token is `"Atlassian Sans", ui-sans-serif, -apple-system, …`. Atlassian Sans is not publicly licensed and is **not shipped**, so on macOS the stack renders **SF Pro — the font in the Figma file**. Linux preview captures substitute Inter (documented, sandbox only).

## Exceptions
Recorded with rationale in `docs/design-fidelity.md` (E-1…E-7).
