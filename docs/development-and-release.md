# Development and release

Prerequisites (Mac): Xcode 26.x, Rust (`rustup`; `rust-toolchain.toml` pins 1.100.0), Bun 1.4.4. Intel and Apple Silicon: the run-script builds `x86_64-apple-darwin` and/or `aarch64-apple-darwin` per `ARCHS` and lipo-merges. Only Intel is the owner's verified hardware; arm64 is unverified.

```bash
bun install && bun run check                       # portable checks
cd native/macos && python3 scripts/gen_xcodeproj.py # regenerate project after adding Swift files (or: xcodegen generate)
xcodebuild -project JustSlide.xcodeproj -scheme JustSlide -configuration Debug -destination 'platform=macOS' build test
open build/Debug/JustSlide.app --args -JustSlideDesignFixture compact   # DEBUG fixture mode
```

Identity: bundle id `app.justslide.JustSlide.dev`, data `~/Library/Application Support/JustSlide Dev` (separate from Slide, Safari, Chrome, the Chromium prototype). Ad-hoc signing (`-`) for development; hardened runtime on; App Sandbox off (see security doc).

Release (not started, requires owner): Developer ID signing + notarisation, sandbox with app-group socket, Sparkle-style updates, `check-release-bundle.mjs --release` must pass (licence decisions), CI (GitHub Actions macOS runner) running `bun run check` + `xcodebuild test`.

SimBridge path: SimBridge builds what is on GitHub with `xcodebuild` (no shell). Needed: push a branch, clone JustSlide on the Mac, add it in SimBridge Projects + Trust builds, reconnect the grant with the project ticked. Unverified: whether `build_start` supports a macOS destination (it is iOS-simulator oriented).
