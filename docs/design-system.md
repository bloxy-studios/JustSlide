# Design system usage

- **Packages** (pinned, Apache-2.0): `@atlaskit/tokens 20.11.0`, `primitives 22.7.1`, `css 1.1.8`, `css-reset 8.4.10`, `dropdown-menu 21.3.1`, `icon 38.0.2` (globe fallback only), `spinner 20.4.4`; `@compiled/react 1.0.2` + `@compiled/parcel-config 0.5.7`, Parcel 2.16.4, React 19.3.1. Verified with the public ADS MCP (`mcp.atlassian.com/v1/ads/public/mcp`, 20 tools) on 2026-11-13.
- **Build**: Compiled extracts CSS at build time (`.compiledcssrc` `extract: true`, tokens Babel plugin without auto-fallback). Theme CSS is lazy-loaded by `setGlobalTheme`; Parcel's import map is hash-pinned in the CSP (`scripts/finalize-chrome-bundle.mjs`).
- **Theme**: light/dark/system via `setGlobalTheme`; Slide layer described in `DESIGN.md`. Appearance is stored in the core (`settings.update`) so it persists across restarts; the native host should pass the stored theme before first paint to avoid a flash (open item).
- **Icons**: Figma-exported SVGs rendered as `currentColor` masks (mono UI glyphs) or images (brand marks). No ADS icon pack substitution for designed slots.
- **Not used**: ADS Tooltip (geometry mismatch, see 30:122), ADS Textfield (address bar is a bespoke 32 px field per 7:46).
- **Known ADS runtime behaviour**: `@atlaskit/feature-gate-js-client` (Statsig) is bundled transitively and stays uninitialised; the chrome CSP `connect-src 'none'` blocks any network (`tests/visual/csp_check.py`: 0 requests).
