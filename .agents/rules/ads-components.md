---
trigger: glob
globs: "apps/browser-chrome/**"
description: ADS tokens/components usage, Slide theme layer, font and licence status.
---
- Style with `token()` from `@atlaskit/tokens` via Compiled (`@compiled/react` cssMap, static values only). App-specific sizes/type live as `--slide-*` variables in `src/styles.css`; backdrop-derived token values live in `src/theme/slide-theme.ts`.
- Do not add Tailwind, shadcn, MUI, Chakra or another design system. Do not hand-author new hex colours in components.
- Use ADS components only where semantics match (DropdownMenu, Spinner, Icon). Browser-specific rows/tiles are composed from tokens.
- Fonts: never bundle Atlassian Sans/Mono (not publicly licensed). The ADS family token falls back to SF Pro on macOS.
- Licence: ADS licence covers Atlassian add-ons only; `@atlaskit/*` npm packages are Apache-2.0. Status and owner decision: `docs/licensing-and-assets.md`.
