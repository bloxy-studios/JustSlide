import AppKit
import WebKit

/// macOS rendering adapter: owns every external-page `WKWebView` and turns
/// core `EngineCommand`s into WebKit calls on the main thread, reporting real
/// WebKit callbacks back as `EngineEvent`s. External pages never receive a
/// script message handler or the chrome's URL schemes.
@MainActor
final class WebKitBrowserAdapter: NSObject {
    private struct Tab {
        let view: WKWebView
        var navigationId: String
        /// True between a core-requested load and WebKit starting it.
        var coreInitiated: Bool
        var observers: [NSKeyValueObservation]
    }

    private var tabs: [String: Tab] = [:]
    private var stores: [String: WKWebsiteDataStore] = [:]
    private let emit: ([String: Any]) -> Void
    /// Where the human-visible page is attached (rounded container).
    let pageContainer: NSView
    private(set) var shownTabId: String?
    private static let automationWorld = WKContentWorld.world(name: "justslide-automation")

    init(pageContainer: NSView, emit: @escaping ([String: Any]) -> Void) {
        self.pageContainer = pageContainer
        self.emit = emit
        super.init()
    }

    /// Typed capability registry reported to the core (never guessed there).
    static var capabilities: [String: Any] {
        func f(_ s: String, _ n: String) -> [String: String] { ["support": s, "note": n] }
        let os = ProcessInfo.processInfo.operatingSystemVersionString
        return [
            "engine": "webkit", "engineVersion": "system WebKit, macOS \(os)",
            "screenshots": f("supported", "WKWebView.takeSnapshot"),
            "backgroundCapture": f("partial", "verified 2026-11-13: takeSnapshot returns an image for views in no window; may be stale (possiblyStale=true)"),
            "pageSnapshot": f("supported", "bounded DOM/ARIA walk in an isolated WKContentWorld"),
            "profileIsolation": f("supported", "WKWebsiteDataStore(forIdentifier:) per profile, macOS 14+"),
            "privateProfiles": f("supported", "WKWebsiteDataStore.nonPersistent()"),
            "downloads": f("partial", "WKDownload to ~/Downloads; UI pending (M5)"),
            "webInspector": f("supported", "isInspectable in DEBUG builds"),
            "extensions": f("unknown", "WKWebExtensionController not integrated yet (M6)"),
            "tabSuspension": f("unsupported", "not implemented"),
            "pageAutomation": f("unsupported", "click/type not exposed in protocol v1"),
        ]
    }

    // MARK: Command dispatch

    func handle(_ cmd: [String: Any]) {
        let type = cmd["type"] as? String ?? ""
        let tabId = cmd["tabId"] as? String ?? ""
        switch type {
        case "createView":
            createView(tabId: tabId, dataStoreId: cmd["dataStoreId"] as? String ?? "", isPrivate: cmd["private"] as? Bool ?? false,
                       url: cmd["url"] as? String ?? "about:blank", navigationId: cmd["navigationId"] as? String ?? "")
        case "destroyView": destroyView(tabId)
        case "navigate": navigate(tabId, url: cmd["url"] as? String ?? "", navigationId: cmd["navigationId"] as? String ?? "")
        case "goBack": tabs[tabId]?.view.goBack()
        case "goForward": tabs[tabId]?.view.goForward()
        case "reload": tabs[tabId]?.view.reload()
        case "stop": tabs[tabId]?.view.stopLoading()
        case "show": show(cmd["tabId"] as? String, focus: cmd["focus"] as? Bool ?? false)
        case "screenshot": screenshot(requestId: cmd["requestId"] as? String ?? "", tabId: tabId, maxEdge: cmd["maxEdgePx"] as? Int ?? 1280, format: cmd["format"] as? String ?? "png")
        case "snapshot": axSnapshot(requestId: cmd["requestId"] as? String ?? "", tabId: tabId, maxNodes: cmd["maxNodes"] as? Int ?? 500)
        default: NSLog("JustSlide: unknown engine command %@", type)
        }
    }

    // MARK: Views

    private func store(for id: String, isPrivate: Bool) -> WKWebsiteDataStore {
        if isPrivate { return .nonPersistent() }
        if let s = stores[id] { return s }
        let s: WKWebsiteDataStore
        if let uuid = UUID(uuidString: id) { s = WKWebsiteDataStore(forIdentifier: uuid) } else { s = .nonPersistent() }
        stores[id] = s
        return s
    }

    private func createView(tabId: String, dataStoreId: String, isPrivate: Bool, url: String, navigationId: String) {
        if tabs[tabId] != nil { navigate(tabId, url: url, navigationId: navigationId); return }
        let config = WKWebViewConfiguration()
        config.websiteDataStore = store(for: dataStoreId, isPrivate: isPrivate)
        config.preferences.javaScriptCanOpenWindowsAutomatically = false
        config.defaultWebpagePreferences.allowsContentJavaScript = true
        let view = WKWebView(frame: pageContainer.bounds, configuration: config)
        view.navigationDelegate = self
        view.uiDelegate = self
        view.allowsBackForwardNavigationGestures = true
        view.autoresizingMask = [.width, .height]
        #if DEBUG
        view.isInspectable = true
        #endif
        view.identifier = NSUserInterfaceItemIdentifier(tabId)
        let obs: [NSKeyValueObservation] = [
            view.observe(\.title, options: [.new]) { [weak self] v, _ in
                Task { @MainActor in self?.emit(["type": "titleChanged", "tabId": tabId, "title": v.title ?? ""]) }
            },
            view.observe(\.url, options: [.new]) { [weak self] v, _ in
                Task { @MainActor in
                    guard let u = v.url?.absoluteString else { return }
                    self?.emit(["type": "urlChanged", "tabId": tabId, "url": u, "canGoBack": v.canGoBack, "canGoForward": v.canGoForward])
                }
            },
        ]
        tabs[tabId] = Tab(view: view, navigationId: navigationId, coreInitiated: true, observers: obs)
        load(view, url)
    }

    private func load(_ view: WKWebView, _ url: String) {
        guard let u = URL(string: url), ["http", "https", "about"].contains(u.scheme ?? "") else { return }
        if u.scheme == "about" { view.loadHTMLString("", baseURL: nil) } else { view.load(URLRequest(url: u)) }
    }

    private func navigate(_ tabId: String, url: String, navigationId: String) {
        guard var t = tabs[tabId] else { return }
        t.navigationId = navigationId
        t.coreInitiated = true
        tabs[tabId] = t
        load(t.view, url)
    }

    private func destroyView(_ tabId: String) {
        guard let t = tabs.removeValue(forKey: tabId) else { return }
        t.observers.forEach { $0.invalidate() }
        t.view.stopLoading()
        t.view.removeFromSuperview()
        if shownTabId == tabId { shownTabId = nil }
    }

    /// Attaches a page to the human window. Only emitted by the core for
    /// human actions. Background views are never added to the window and
    /// never become first responder.
    private func show(_ tabId: String?, focus: Bool) {
        for v in pageContainer.subviews where v.identifier?.rawValue != tabId { v.removeFromSuperview() }
        shownTabId = tabId
        guard let tabId, let view = tabs[tabId]?.view else { return }
        if view.superview !== pageContainer {
            view.frame = pageContainer.bounds
            pageContainer.addSubview(view)
        }
        if focus { pageContainer.window?.makeFirstResponder(view) }
    }

    func view(for tabId: String) -> WKWebView? { tabs[tabId]?.view }
    func tabId(of view: WKWebView) -> String? { view.identifier?.rawValue }

    private func navId(_ view: WKWebView) -> String { tabId(of: view).flatMap { tabs[$0]?.navigationId } ?? "" }

    /// Page-initiated navigations (link clicks, redirects to new documents)
    /// get a fresh id so stale automation targets are detected by the core.
    private func beginNavigation(_ view: WKWebView) -> String {
        guard let id = tabId(of: view), var t = tabs[id] else { return "" }
        if !t.coreInitiated { t.navigationId = "nav_wk" + UUID().uuidString.replacingOccurrences(of: "-", with: "").prefix(16).lowercased() }
        t.coreInitiated = false
        tabs[id] = t
        return t.navigationId
    }

    // MARK: Capture

    private func screenshot(requestId: String, tabId: String, maxEdge: Int, format: String) {
        guard let t = tabs[tabId] else {
            emit(["type": "screenshotResult", "requestId": requestId, "navigationId": "", "capture": NSNull(), "error": "no live view for tab"])
            return
        }
        let nav = t.navigationId
        let config = WKSnapshotConfiguration()
        let b = t.view.bounds.size
        let scale = min(1, CGFloat(maxEdge) / max(b.width, b.height, 1))
        config.snapshotWidth = NSNumber(value: Double(b.width * scale))
        let stale = t.view.window == nil
        t.view.takeSnapshot(with: config) { [weak self] image, error in
            guard let self else { return }
            guard let image, let tiff = image.tiffRepresentation, let rep = NSBitmapImageRep(data: tiff),
                  let data = rep.representation(using: format == "jpeg" ? .jpeg : .png, properties: format == "jpeg" ? [.compressionFactor: 0.8] : [:]) else {
                self.emit(["type": "screenshotResult", "requestId": requestId, "navigationId": nav, "capture": NSNull(), "error": error?.localizedDescription ?? "snapshot failed"])
                return
            }
            self.emit(["type": "screenshotResult", "requestId": requestId, "navigationId": nav, "error": NSNull(), "capture": [
                "mimeType": format == "jpeg" ? "image/jpeg" : "image/png", "dataBase64": data.base64EncodedString(),
                "width": rep.pixelsWide, "height": rep.pixelsHigh, "possiblyStale": stale]])
        }
    }

    /// Bounded, read-only accessibility-ish snapshot from an isolated world;
    /// page scripts cannot observe or tamper with it.
    private func axSnapshot(requestId: String, tabId: String, maxNodes: Int) {
        guard let t = tabs[tabId] else {
            emit(["type": "snapshotResult", "requestId": requestId, "navigationId": "", "url": "", "nodes": NSNull(), "truncated": false, "error": "no live view for tab"])
            return
        }
        let nav = t.navigationId
        let js = """
        const out = []; let truncated = false;
        const walk = (el, depth) => {
          if (out.length >= max) { truncated = true; return; }
          const role = el.getAttribute('role') || ({A:'link',BUTTON:'button',H1:'heading',H2:'heading',H3:'heading',INPUT:'textbox',IMG:'img',NAV:'navigation',MAIN:'main'})[el.tagName];
          if (role) out.push({role, name: (el.getAttribute('aria-label') || el.alt || el.innerText || '').trim().slice(0, 200), depth});
          for (const c of el.children) walk(c, depth + 1);
        };
        walk(document.body || document.documentElement, 0);
        return {nodes: out, truncated, url: location.href};
        """
        t.view.callAsyncJavaScript(js, arguments: ["max": maxNodes], in: nil, in: Self.automationWorld) { [weak self] result in
            switch result {
            case .success(let v):
                let d = v as? [String: Any] ?? [:]
                self?.emit(["type": "snapshotResult", "requestId": requestId, "navigationId": nav, "url": d["url"] as? String ?? "", "nodes": d["nodes"] ?? [], "truncated": d["truncated"] as? Bool ?? false, "error": NSNull()])
            case .failure(let e):
                self?.emit(["type": "snapshotResult", "requestId": requestId, "navigationId": nav, "url": "", "nodes": NSNull(), "truncated": false, "error": e.localizedDescription])
            }
        }
    }

    private func reportFavicon(_ view: WKWebView) {
        guard let id = tabId(of: view) else { return }
        let js = "const l=[...document.querySelectorAll('link[rel~=icon],link[rel=apple-touch-icon]')].map(x=>x.href).find(h=>h.startsWith('https://')||h.startsWith('http://')); return l || new URL('/favicon.ico', location.href).href;"
        view.callAsyncJavaScript(js, arguments: [:], in: nil, in: Self.automationWorld) { [weak self] r in
            if case .success(let v) = r, let href = v as? String { self?.emit(["type": "faviconChanged", "tabId": id, "faviconUrl": href]) }
        }
    }
}

extension WebKitBrowserAdapter: WKNavigationDelegate {
    func webView(_ webView: WKWebView, didStartProvisionalNavigation navigation: WKNavigation!) {
        guard let id = tabId(of: webView) else { return }
        emit(["type": "navigationStarted", "tabId": id, "navigationId": beginNavigation(webView), "url": webView.url?.absoluteString ?? ""])
    }

    func webView(_ webView: WKWebView, didCommit navigation: WKNavigation!) {
        guard let id = tabId(of: webView) else { return }
        emit(["type": "navigationCommitted", "tabId": id, "navigationId": navId(webView), "url": webView.url?.absoluteString ?? "",
              "secure": webView.hasOnlySecureContent && webView.url?.scheme == "https", "canGoBack": webView.canGoBack, "canGoForward": webView.canGoForward])
    }

    func webView(_ webView: WKWebView, didFinish navigation: WKNavigation!) {
        guard let id = tabId(of: webView) else { return }
        emit(["type": "navigationFinished", "tabId": id, "navigationId": navId(webView), "title": webView.title ?? ""])
        reportFavicon(webView)
    }

    func webView(_ webView: WKWebView, didFail navigation: WKNavigation!, withError error: Error) { failed(webView, error) }
    func webView(_ webView: WKWebView, didFailProvisionalNavigation navigation: WKNavigation!, withError error: Error) { failed(webView, error) }

    private func failed(_ webView: WKWebView, _ error: Error) {
        guard let id = tabId(of: webView) else { return }
        if (error as NSError).code == NSURLErrorCancelled { return }
        emit(["type": "navigationFailed", "tabId": id, "navigationId": navId(webView), "message": error.localizedDescription])
    }

    func webViewWebContentProcessDidTerminate(_ webView: WKWebView) {
        guard let id = tabId(of: webView) else { return }
        emit(["type": "processTerminated", "tabId": id])
    }

    func webView(_ webView: WKWebView, decidePolicyFor action: WKNavigationAction, preferences: WKWebpagePreferences, decisionHandler: @escaping (WKNavigationActionPolicy, WKWebpagePreferences) -> Void) {
        // Pages may only navigate to web schemes; JustSlide-internal schemes
        // are never reachable from untrusted content.
        let scheme = action.request.url?.scheme?.lowercased() ?? ""
        if ["http", "https", "about", "data", "blob"].contains(scheme) {
            decisionHandler(action.shouldPerformDownload ? .download : .allow, preferences)
        } else {
            decisionHandler(.cancel, preferences)
        }
    }

    func webView(_ webView: WKWebView, decidePolicyFor response: WKNavigationResponse, decisionHandler: @escaping (WKNavigationResponsePolicy) -> Void) {
        decisionHandler(response.canShowMIMEType ? .allow : .download)
    }

    func webView(_ webView: WKWebView, navigationAction: WKNavigationAction, didBecome download: WKDownload) { download.delegate = DownloadSink.shared }
    func webView(_ webView: WKWebView, navigationResponse: WKNavigationResponse, didBecome download: WKDownload) { download.delegate = DownloadSink.shared }
}

extension WebKitBrowserAdapter: WKUIDelegate {
    /// New windows become JustSlide tabs owned by the core; WebKit never opens
    /// its own windows. Only link activations count as user gestures.
    func webView(_ webView: WKWebView, createWebViewWith configuration: WKWebViewConfiguration, for action: WKNavigationAction, windowFeatures: WKWindowFeatures) -> WKWebView? {
        if let id = tabId(of: webView), let url = action.request.url?.absoluteString {
            emit(["type": "popupRequested", "openerTabId": id, "url": url, "userGesture": action.navigationType == .linkActivated])
        }
        return nil
    }

    func webView(_ webView: WKWebView, runJavaScriptAlertPanelWithMessage message: String, initiatedByFrame frame: WKFrameInfo, completionHandler: @escaping () -> Void) {
        let a = NSAlert()
        a.messageText = frame.request.url?.host ?? "This page"
        a.informativeText = message
        if let w = webView.window, webView.superview != nil { a.beginSheetModal(for: w) { _ in completionHandler() } } else { completionHandler() }
    }

    func webView(_ webView: WKWebView, runJavaScriptConfirmPanelWithMessage message: String, initiatedByFrame frame: WKFrameInfo, completionHandler: @escaping (Bool) -> Void) {
        let a = NSAlert()
        a.messageText = frame.request.url?.host ?? "This page"
        a.informativeText = message
        a.addButton(withTitle: "OK"); a.addButton(withTitle: "Cancel")
        // Background tabs cannot interrupt the person: dialogs are declined.
        if let w = webView.window, webView.superview != nil { a.beginSheetModal(for: w) { completionHandler($0 == .alertFirstButtonReturn) } } else { completionHandler(false) }
    }

    func webView(_ webView: WKWebView, runOpenPanelWith parameters: WKOpenPanelParameters, initiatedByFrame frame: WKFrameInfo, completionHandler: @escaping ([URL]?) -> Void) {
        guard let w = webView.window, webView.superview != nil else { completionHandler(nil); return }
        let p = NSOpenPanel()
        p.allowsMultipleSelection = parameters.allowsMultipleSelection
        p.canChooseDirectories = parameters.allowsDirectories
        p.beginSheetModal(for: w) { completionHandler($0 == .OK ? p.urls : nil) }
    }
}

/// Minimal download sink: user-initiated downloads go to ~/Downloads with a
/// unique name. Progress UI and permission prompts land in M5.
@MainActor
final class DownloadSink: NSObject, WKDownloadDelegate {
    static let shared = DownloadSink()
    func download(_ download: WKDownload, decideDestinationUsing response: URLResponse, suggestedFilename: String, completionHandler: @escaping (URL?) -> Void) {
        let dir = FileManager.default.urls(for: .downloadsDirectory, in: .userDomainMask).first!
        var dest = dir.appendingPathComponent((suggestedFilename as NSString).lastPathComponent)
        var n = 1
        while FileManager.default.fileExists(atPath: dest.path) {
            let base = (suggestedFilename as NSString).deletingPathExtension, ext = (suggestedFilename as NSString).pathExtension
            dest = dir.appendingPathComponent("\(base) (\(n))" + (ext.isEmpty ? "" : ".\(ext)")); n += 1
        }
        completionHandler(dest)
    }
}
