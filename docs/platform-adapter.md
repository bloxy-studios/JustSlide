# macOS WebKit adapter

File: `native/macos/JustSlide/WebKitBrowserAdapter.swift` (status: **written, not yet compiled** — first build expected to need fixes).

- **Views**: one `WKWebView` per tab, fresh `WKWebViewConfiguration` (no message handlers, no custom schemes), `javaScriptCanOpenWindowsAutomatically = false`, `isInspectable` in DEBUG.
- **Profiles**: `WKWebsiteDataStore(forIdentifier: UUID)` (macOS 14+) keyed by the core profile's `dataStoreId`; private profiles use `.nonPersistent()`. Deployment target macOS 14.0.
- **Navigation**: provisional/commit/finish/fail callbacks → engine events; KVO on `title`/`url` for same-document changes; core-initiated loads keep the core's navigation id, page-initiated navigations get a fresh one (stale-target detection).
- **Schemes**: pages may navigate to http(s)/about/data/blob only; `justslide-*` schemes are cancelled.
- **Popups**: `createWebViewWith` returns nil and emits `popupRequested`; only `.linkActivated` counts as a user gesture (conservative; script `window.open` with user activation is treated as blocked — tracked).
- **Dialogs/file input**: shown as sheets only for the attached (human-visible) view; background tabs' `confirm()` resolves false, `alert()` is dismissed, file pickers are refused.
- **Downloads**: `WKDownload` to ~/Downloads with unique names (UI/permissions in M5).
- **Capture**: `takeSnapshot` (PNG/JPEG, max edge); `possiblyStale` when the view is not in a window. Background capture is reported `unknown` until `testBackgroundSnapshotBehaviourIsReported` runs on a Mac.
- **Page snapshot**: bounded DOM/ARIA walk in `WKContentWorld.world(name: "justslide-automation")`; results flagged untrusted by the core.
- **Focus**: background views are never added to the window; `show(focus:true)` (human only) calls `makeFirstResponder`.
- **Crash**: `webViewWebContentProcessDidTerminate` → `processTerminated`; core recreates the view on reload.
- **Extensions**: `WKWebExtensionController` not integrated (see `docs/extension-matrix.md`).
