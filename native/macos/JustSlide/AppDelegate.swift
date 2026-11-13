import AppKit

@main
@MainActor
final class AppDelegate: NSObject, NSApplicationDelegate {
    private var core: CoreBridge!
    private var windowController: BrowserWindowController!
    private var nextId = 1

    static func main() {
        let app = NSApplication.shared
        let delegate = AppDelegate()
        app.delegate = delegate
        app.setActivationPolicy(.regular)
        app.run()
    }

    /// `~/Library/Application Support/JustSlide Dev` — distinct from Slide,
    /// Safari, Chrome and the archived Chromium prototype.
    static var dataDirectory: URL {
        FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0].appendingPathComponent(Bundle.main.object(forInfoDictionaryKey: "JSDataDirectoryName") as? String ?? "JustSlide Dev")
    }

    func applicationDidFinishLaunching(_ notification: Notification) {
        let fixture = Self.designFixtureVariant
        guard let core = CoreBridge(dataDirectory: fixture == nil ? Self.dataDirectory : nil, capabilities: WebKitBrowserAdapter.capabilities, inMemory: fixture != nil, enableAgentIPC: fixture == nil) else {
            let a = NSAlert(); a.messageText = "JustSlide could not open its profile"; a.informativeText = Self.dataDirectory.path; a.runModal()
            NSApp.terminate(nil); return
        }
        self.core = core
        if let fixture { core.loadDesignFixture(variant: fixture) }
        windowController = BrowserWindowController(core: core, resources: Bundle.main.resourceURL!, size: NSSize(width: 1440, height: 1024), startInFixture: fixture)
        windowController.showWindow(nil)
        if fixture == nil { core.restoreSession() }
        NSApp.mainMenu = buildMenu()
        NSApp.activate()
    }

    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool { true }

    static var designFixtureVariant: String? {
        #if DEBUG
        return UserDefaults.standard.string(forKey: "JustSlideDesignFixture")
        #else
        return nil
        #endif
    }

    // MARK: Menus (every action is a core command)

    private func run(_ command: String, _ args: [String: Any] = [:]) {
        nextId += 1
        core.execute(id: "menu-\(nextId)", command: command, args: args) { _ in }
    }

    private func withActiveTab(_ f: @escaping (String) -> Void) {
        nextId += 1
        core.execute(id: "menu-\(nextId)", command: "state.snapshot", args: [:]) { r in
            guard let s = r["result"] as? [String: Any], let tabs = s["tabs"] as? [[String: Any]],
                  let t = tabs.first(where: { $0["active"] as? Bool == true }), let id = t["id"] as? String else { return }
            f(id)
        }
    }

    @objc func newTab(_ s: Any?) { run("tabs.open", ["input": "about:blank"]); focusAddressBar(nil) }
    @objc func closeTab(_ s: Any?) { withActiveTab { self.run("tabs.close", ["tabId": $0]) } }
    @objc func reopenTab(_ s: Any?) { run("tabs.reopen") }
    @objc func reload(_ s: Any?) { withActiveTab { self.run("page.reload", ["tabId": $0]) } }
    @objc func goBack(_ s: Any?) { withActiveTab { self.run("page.back", ["tabId": $0]) } }
    @objc func goForward(_ s: Any?) { withActiveTab { self.run("page.forward", ["tabId": $0]) } }
    @objc func focusAddressBar(_ s: Any?) {
        windowController.window?.makeFirstResponder(windowController.chrome)
        windowController.chrome.evaluateJavaScript("document.querySelector('[aria-label=\"Address and search\"]')?.focus()")
    }

    #if DEBUG
    @objc func fixtureDesktop(_ s: Any?) { windowController.showDesignFixture("desktop") }
    @objc func fixtureCompact(_ s: Any?) { windowController.showDesignFixture("compact") }
    @objc func liveBrowser(_ s: Any?) { windowController.showLiveChrome() }
    #endif

    private func buildMenu() -> NSMenu {
        let main = NSMenu()
        func sub(_ title: String, _ items: [NSMenuItem]) { let m = NSMenu(title: title); items.forEach { m.addItem($0) }; let i = NSMenuItem(); i.submenu = m; main.addItem(i) }
        func item(_ t: String, _ a: Selector?, _ k: String, _ mods: NSEvent.ModifierFlags = .command) -> NSMenuItem { let i = NSMenuItem(title: t, action: a, keyEquivalent: k); i.keyEquivalentModifierMask = mods; return i }
        sub("JustSlide", [item("About JustSlide", #selector(NSApplication.orderFrontStandardAboutPanel(_:)), ""), .separator(), item("Hide JustSlide", #selector(NSApplication.hide(_:)), "h"), .separator(), item("Quit JustSlide", #selector(NSApplication.terminate(_:)), "q")])
        sub("File", [item("New Tab", #selector(newTab(_:)), "t"), item("Reopen Closed Tab", #selector(reopenTab(_:)), "t", [.command, .shift]), item("Open Location…", #selector(focusAddressBar(_:)), "l"), .separator(), item("Close Tab", #selector(closeTab(_:)), "w")])
        sub("Edit", [item("Undo", Selector(("undo:")), "z"), item("Redo", Selector(("redo:")), "z", [.command, .shift]), .separator(), item("Cut", #selector(NSText.cut(_:)), "x"), item("Copy", #selector(NSText.copy(_:)), "c"), item("Paste", #selector(NSText.paste(_:)), "v"), item("Select All", #selector(NSText.selectAll(_:)), "a")])
        sub("View", [item("Reload Page", #selector(reload(_:)), "r"), item("Enter Full Screen", #selector(NSWindow.toggleFullScreen(_:)), "f", [.command, .control])])
        sub("History", [item("Back", #selector(goBack(_:)), "["), item("Forward", #selector(goForward(_:)), "]")])
        sub("Window", [item("Minimize", #selector(NSWindow.performMiniaturize(_:)), "m"), item("Zoom", #selector(NSWindow.performZoom(_:)), "")])
        #if DEBUG
        sub("Develop", [item("Design Fixture Desktop", #selector(fixtureDesktop(_:)), ""), item("Design Fixture Compact", #selector(fixtureCompact(_:)), ""), item("Live Browser", #selector(liveBrowser(_:)), "")])
        #endif
        return main
    }
}
