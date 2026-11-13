# Testing and evidence

| Suite | Command | Where it runs | Result 2026-11-13 |
|---|---|---|---|
| Protocol registry + validator | `cargo test -p justslide-protocol` | any | 6 pass |
| Core scenarios (test-double engine) | `cargo test -p justslide-core --features test-engine` | any | 17 + 4 unit pass |
| C ABI | `cargo test -p justslide-ffi [--features design-fixture]` | any | pass / pass |
| MCP e2e (real binary ↔ real core IPC) | `cargo test -p justslide-mcp` | Unix | 2 e2e + 3 endpoint unit pass |
| UI components | `bun run test:ui` | any | 11 pass |
| Protocol drift | `bun run protocol:check` | any | pass |
| Chrome CSP / no-network | `python3 tests/visual/csp_check.py` | Playwright Chromium | pass (0 violations, 0 requests) |
| Figma geometry + pixels | `python3 tests/visual/capture_and_compare.py` | Playwright Chromium (preview) | 0 failures at 1 px, 4 compositions |
| Release bundle guard | `node scripts/check-release-bundle.mjs [--release]` | any | pass (4 licence warnings; `--release` fails until resolved) |
| Native XCTest (`JustSlideTests`) | `xcodebuild -project native/macos/JustSlide.xcodeproj -scheme JustSlide -destination 'platform=macOS' test` | macOS | **not run** |
| Native screenshots | launch with `-JustSlideDesignFixture desktop|compact` | macOS | **not run** |
| WebKit (Playwright WebKitGTK) preview | `--engines webkit` | Linux | **not run** — missing system libraries on this host |

Environment: Linux x86_64 (Amazon Linux 2023), 2 vCPU, 4 GB; Rust 1.100.0; Bun 1.4.4; Node 24.22.3; Python 3.9. Owner Mac (via SimBridge health): Intel x86_64, macOS 26.5.1, Xcode 26.6 (17F113).
