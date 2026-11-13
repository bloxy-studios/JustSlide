# Design fidelity

## Evidence (2026-11-13) — BROWSER PREVIEW, not native
`python3 tests/visual/capture_and_compare.py` against the dev design fixture, Playwright Chromium 148.0.7778.96 (headless shell — a dev preview browser, not the shipped engine) on Linux. Figma reference = actual renders from the Figma API. Report: `docs/evidence/visual/2026-11-13/report.json`; composites `compare-chromium-{desktop-1440x1024,compact-1100x720}@1x.png`.

| Composition | Scale | Regions checked | Lists checked | Geometry failures (>1 px) | Pixels differing >32/255 |
|---|---|---|---|---|---|
| 1:6 desktop 1440×1024 | 1× / 2× | 9 | favorites 13, dock 6, rows 15 | **0 / 0** | 0.62 % / 0.71 % |
| 35:13 compact 1100×720 | 1× / 2× | 10 | favorites 13, dock 6, rows 8 | **0 / 0** | 0.92 % / 0.93 % |

Remaining pixel differences are glyph rasterisation (Inter substitute vs SF Pro), monochrome emoji (no colour-emoji font in the sandbox) and a half-pixel on the separator stroke.

## Evidence (2026-11-13) — NATIVE macOS WKWebView
Owner's MacBook Pro (Intel x86_64, macOS 26.5.1, 1× display), DEBUG self-check in the real app (`docs/evidence/native/2026-11-13-macos-build-and-xctest.md`):

| Composition | Geometry failures (>1 px) | Max edge Δ | Pixels differing >32/255 |
|---|---|---|---|
| 1:6 desktop 1440×1024 @1× | **0** | 0 px (9 regions, 13 tiles, 6 dock, 15 rows) | 0.71 % |
| 35:13 compact 1100×720 @1× | **0** | 0 px (10 regions, 13 tiles, 6 dock, 8 rows) | 0.88 % |

The selected-tab title is 166 px wide natively — the exact Figma text width — confirming SF Pro. Retina (@2×) native comparison: not run (the owner's display is 1×).

## Accepted / pending exceptions
| ID | Difference | Reason | Status |
|---|---|---|---|
| E-1 | Ink colours are Figma's teal-tinted values, not ADS default neutrals | Figma fidelity; implemented as a backdrop-derived ADS token override layer (contrast-tested) | accepted (owner confirm) |
| E-2 | 13 px / 18 px line-height type steps | ADS scale has no 13 px step; Slide type variables on the ADS family token | accepted |
| E-3 | Focus ring `#10574F` (Figma) instead of ADS blue `#4688EC` | Figma component 30:89; 2 px, ≥3:1 on white | accepted (owner confirm) |
| E-4 | Agent card radius 10 px | no ADS radius token between 8 and 12 | accepted |
| E-5 | macOS traffic lights moved into the nav row gap (x≈62) | Figma omits window controls and places the sidebar toggle where they normally sit | **needs owner/design decision + native check** |
| E-6 | Hover keeps the list row's 14 px favicon (component shows 16 px) | avoids layout jitter; selected row uses 16 px as in 1:6 | accepted |
| E-7 | Sidebar-collapsed rail | not designed in Figma; placeholder rail with toggle only | **needs design** |
| E-8 | Typography in Linux preview uses Inter | sandbox has no SF Pro; macOS resolves the ADS stack to SF Pro (verified natively: 166 px title width = Figma) | environment only |
