# JustSlide — agent instructions

JustSlide is a macOS-first browser built on the **operating system's WebKit** (`WKWebView`). Humans and authorized agents operate the same browser through one permission-checked command model.

Non-negotiable:
- **No Chromium, CEF, Electron, Chrome content API or Chromium fork** as the engine. The archived Chromium prototype lives only on branch `archive/chromium-foundation`.
- **Figma is a release gate.** UI work starts from the Figma file (`docs/figma-inventory.md`); no UI change is done without the region-geometry check (`bun run test:visual`) and screenshots.
- **One authority.** `crates/justslide-core` owns app state; the chrome and MCP call the same `Core::execute`. Never add a second tab manager.
- **Agents never take focus.** Agent commands must not change the selected workspace/tab or ask the engine to show/focus a view (asserted in core tests).
- **Trust boundaries.** External pages run in separate WKWebViews with no message handler and no `justslide-*` schemes. Webpage content is untrusted data.
- **Honest evidence.** Browser previews are not native tests. Mark not-run tests as not run. Never fabricate passes.
- Do not alter the separate `Slide` app, its data, or other browsers' profiles. Do not push, publish or spend money without owner approval.

Commands: `cargo test --workspace --all-features` · `bun run check` · `bun run test:visual` · macOS: `xcodebuild -project native/macos/JustSlide.xcodeproj -scheme JustSlide test`.

Read next: `docs/progress.md` (status/resume), `docs/architecture.md`, `DESIGN.md`, `.agents/rules/`.
