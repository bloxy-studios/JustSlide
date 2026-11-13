# Licensing and assets

Inventory command: `node scripts/license-audit.mjs` → `docs/evidence/test-results/licenses.json` (2026-11-13).

| Item | Licence / terms | Status |
|---|---|---|
| npm production closure (114 pkgs) | MIT 74, Apache-2.0 33, ISC 4, BSD-3 2, 0BSD 1 | approved (permissive) |
| `@atlaskit/*` code packages (32) | Apache-2.0 each. ADS Licence (13 Mar 2025) grants use for Atlassian-integrated Add-Ons; §8 says OSS components may be used under their own licences | **needs owner/legal review** before release |
| ADS visual assets, guidelines, Figma kits | ADS Licence only (Add-Ons) | **blocked** for JustSlide shipping; not copied |
| Atlassian Sans / Mono | not publicly distributed | **blocked**; not bundled (system SF Pro used) |
| Rust crates (161) | MIT/Apache-2.0 dual, Unicode-3.0, Zlib, Unlicense | approved |
| UI glyphs from Figma (`assets/icons/ui`, `workspaces/personal.svg`) | Figma layer names indicate Nucleo icons (e.g. `chevron-right-outline-24`, `plus-outline-duo-18`) — commercial licence | **needs owner confirmation of Nucleo licence** |
| Workspace brand icons (Codex, Claude Code, ChatGPT) | third-party trademarks | **needs owner/legal review**; flagged by `check-release-bundle.mjs --release` |
| Favorite/favicon brand marks (YouTube, Gmail, AWS, …) | third-party trademarks | dev fixture only; excluded from the app bundle (enforced) — real tabs show favicons WebKit loads from the site |
| Inter (Linux preview only) | OFL-1.1 | sandbox tool, not in repo/bundle |
| Mascot, app icon | not in the repo or Figma frames | pending owner asset |
