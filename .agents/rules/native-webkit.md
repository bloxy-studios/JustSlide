---
trigger: glob
globs: "native/macos/**"
description: macOS WebKit adapter rules — threading, focus, profiles, security, verification.
---
- All WebKit/AppKit calls on the main actor. The adapter only executes core `EngineCommand`s and reports real callbacks as `EngineEvent`s; it never invents state.
- Background views are never added to the window and never become first responder; only `show{focus:true}` from a human action may focus a page.
- One `WKWebsiteDataStore(forIdentifier:)` per profile (macOS 14+); private = `.nonPersistent()`.
- The chrome WKWebView alone gets the `justslide` message handler and `justslide-chrome:`/`justslide-favicon:` schemes. Page views get fresh configurations.
- Native claims require `xcodebuild test` on a Mac; record arch/macOS/Xcode and results in `docs/evidence/native/`.
