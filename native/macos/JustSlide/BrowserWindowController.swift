import AppKit
import WebKit

/// One browser window: backdrop + trusted chrome WKWebView (full window) +
/// rounded page container positioned over the chrome's page-host anchor.
@MainActor
final class BrowserWindowController: NSWindowController, NSWindowDelegate {
    let chrome: WKWebView
    let pageContainer: NSView
    let adapter: WebKitBrowserAdapter
    private let core: CoreBridge
    private let handler: ChromeMessageHandler
    private let resources: URL

    init(core: CoreBridge, resources: URL, size: NSSize, startInFixture: String? = nil) {
        self.core = core
        self.resources = resources
        let container = NSView()
        pageContainer = container
        handler = ChromeMessageHandler(core: core)
        let config = WKWebViewConfiguration()
        var roots = ["app": resources.appendingPathComponent("Chrome")]
        #if DEBUG
        roots["preview"] = resources.appendingPathComponent("ChromePreview")
        #endif
        config.setURLSchemeHandler(ChromeSchemeHandler(roots: roots), forURLScheme: "justslide-chrome")
        config.setURLSchemeHandler(FaviconSchemeHandler(), forURLScheme: "justslide-favicon")
        config.userContentController.addScriptMessageHandler(handler, contentWorld: .page, name: "justslide")
        config.websiteDataStore = .nonPersistent()
        chrome = WKWebView(frame: NSRect(origin: .zero, size: size), configuration: config)
        chrome.setAccessibilityLabel("JustSlide browser chrome")
        #if DEBUG
        chrome.isInspectable = true
        #endif
        adapter = WebKitBrowserAdapter(pageContainer: container) { [weak core] ev in core?.send(event: ev) }
        let window = NSWindow(contentRect: NSRect(origin: .zero, size: size), styleMask: [.titled, .closable, .miniaturizable, .resizable, .fullSizeContentView], backing: .buffered, defer: false)
        window.titlebarAppearsTransparent = true
        window.titleVisibility = .hidden
        window.title = "JustSlide"
        window.minSize = NSSize(width: 800, height: 560)
        window.isMovableByWindowBackground = false
        // Follow the active Space when ordered front (apps launched in the
        // background otherwise open on a Space the person isn't viewing).
        window.collectionBehavior.insert(.moveToActiveSpace)
        super.init(window: window)
        window.delegate = self
        let content = NSView(frame: window.contentLayoutRect)
        content.wantsLayer = true
        window.contentView = content
        chrome.frame = content.bounds
        chrome.autoresizingMask = [.width, .height]
        content.addSubview(chrome)
        pageContainer.wantsLayer = true
        pageContainer.layer?.cornerRadius = 16
        pageContainer.layer?.cornerCurve = .continuous
        pageContainer.layer?.masksToBounds = true
        pageContainer.setAccessibilityRole(.group)
        pageContainer.setAccessibilityLabel("Web page")
        content.addSubview(pageContainer)
        handler.onContentBounds = { [weak self] rect, radius in self?.placePage(rect, radius) }
        core.onEngineCommand = { [weak self] cmd in self?.adapter.handle(cmd) }
        core.onCoreEvent = { [weak self] seq, _ in self?.chrome.evaluateJavaScript("window.__justslideEvent && window.__justslideEvent(\(seq))") }
        window.center()
        layoutTrafficLights()
        if let v = startInFixture { showDesignFixture(v) } else { showLiveChrome() }
        if let screen = window.screen ?? NSScreen.main {
            NSLog("JustSlide: screen visibleFrame %@ backingScale %.1f", NSStringFromRect(screen.visibleFrame), screen.backingScaleFactor)
        }
    }

    /// Normal mode: production chrome driven by the Rust core.
    func showLiveChrome() {
        pageContainer.isHidden = false
        window?.title = "JustSlide"
        chrome.load(URLRequest(url: URL(string: "justslide-chrome://app/index.html")!))
    }

    #if DEBUG
    /// DEBUG ONLY: renders the Figma reference content (dev fixture, JS bridge)
    /// in this real WKWebView at the Figma frame size for native comparison.
    /// No page view is shown; nothing here touches the user's profile.
    func showDesignFixture(_ variant: String) {
        pageContainer.isHidden = true
        let size = variant == "compact" ? NSSize(width: 1100, height: 720) : NSSize(width: 1440, height: 1024)
        window?.setContentSize(size)
        window?.title = "JustSlide — DESIGN FIXTURE (\(variant))"
        chrome.load(URLRequest(url: URL(string: "justslide-chrome://preview/preview.html?variant=\(variant)")!))
        layoutTrafficLights()
        NSLog("JustSlide: design fixture %@ requested %@ actual content %@", variant, NSStringFromSize(size), NSStringFromSize(window?.contentView?.bounds.size ?? .zero))
        NativeFixtureCheck.run(chrome: chrome, variant: variant, previewRoot: resources.appendingPathComponent("ChromePreview"))
    }
    #else
    func showDesignFixture(_ variant: String) {}
    #endif

    required init?(coder: NSCoder) { fatalError("not supported") }

    /// Chrome reports CSS-pixel rects (top-left origin) of the page anchor.
    private func placePage(_ rect: CGRect, _ radius: CGFloat) {
        guard let content = window?.contentView else { return }
        pageContainer.frame = NSRect(x: rect.minX, y: content.bounds.height - rect.maxY, width: rect.width, height: rect.height)
        pageContainer.layer?.cornerRadius = radius
    }

    /// Figma omits macOS window controls and puts the sidebar toggle where
    /// they normally sit; we move them into the gap between the toggle and
    /// history controls (docs/design-fidelity.md, exception E-5; needs owner
    /// sign-off and native verification).
    private func layoutTrafficLights() {
        guard let window, let close = window.standardWindowButton(.closeButton), let container = close.superview?.superview else { return }
        let rowCenterFromTop: CGFloat = 32, startX: CGFloat = 62
        var f = container.frame
        f.size.height = 52
        f.origin.y = window.frame.height - f.height
        container.frame = f
        for (i, kind) in [NSWindow.ButtonType.closeButton, .miniaturizeButton, .zoomButton].enumerated() {
            guard let b = window.standardWindowButton(kind) else { continue }
            b.setFrameOrigin(NSPoint(x: startX + CGFloat(i) * 20, y: f.height - rowCenterFromTop - b.frame.height / 2))
        }
    }

    func windowDidResize(_ notification: Notification) { layoutTrafficLights() }
    func windowDidExitFullScreen(_ notification: Notification) { layoutTrafficLights() }
}
