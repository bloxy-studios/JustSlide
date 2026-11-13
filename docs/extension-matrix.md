# WebExtension compatibility (WebKit)

Status: **not tested**. No macOS run occurred this session. `WKWebExtension`, `WKWebExtensionController`, `WKWebExtensionContext` (macOS 15.4+/Safari 18.4 SDK era) are the intended path; JustSlide's deployment target is macOS 14, so extension support will be gated at runtime.

| Feature | Status | Test |
|---|---|---|
| Content scripts + isolation | unknown | not run |
| runtime messaging / storage | unknown | not run |
| Service worker background | unknown | not run |
| Action popups / options pages | unknown | not run |
| tabs/windows events ↔ workspaces | unknown | not run |
| Permissions / origin grants | unknown | not run |
| Native messaging | unknown | not run |
| Install / update / enable / remove | unknown | not run |

JustSlide will not claim Chrome Web Store compatibility. Next step (M6): a lawful MV3 fixture in `tests/fixtures/extensions/` loaded through `WKWebExtensionController` on the owner's Mac (macOS 26.5.1, Xcode 26.6).
