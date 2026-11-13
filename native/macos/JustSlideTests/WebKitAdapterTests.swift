import AppKit
import WebKit
import XCTest
@testable import JustSlide

/// Native WebKit invariants. These run on macOS only (xcodebuild test).
@MainActor
final class WebKitAdapterTests: XCTestCase {
    private func makeWindow() -> (NSWindow, NSTextField, NSView) {
        let w = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 800, height: 600), styleMask: [.titled], backing: .buffered, defer: false)
        let field = NSTextField(frame: NSRect(x: 10, y: 10, width: 200, height: 24))
        let container = NSView(frame: NSRect(x: 260, y: 10, width: 520, height: 560))
        w.contentView!.addSubview(field); w.contentView!.addSubview(container)
        w.makeKeyAndOrderFront(nil)
        XCTAssertTrue(w.makeFirstResponder(field))
        return (w, field, container)
    }

    private func waitFor(_ events: @escaping () -> Bool, timeout: TimeInterval = 15) {
        let exp = expectation(description: "condition")
        func poll() { if events() { exp.fulfill() } else { DispatchQueue.main.asyncAfter(deadline: .now() + 0.05, execute: poll) } }
        poll()
        wait(for: [exp], timeout: timeout)
    }

    func testBackgroundViewNeverStealsKeyboardFocus() {
        let (w, field, container) = makeWindow()
        var events: [[String: Any]] = []
        let a = WebKitBrowserAdapter(pageContainer: container) { events.append($0) }
        a.handle(["type": "createView", "tabId": "tab_bg000001", "dataStoreId": UUID().uuidString, "private": false, "url": "about:blank", "navigationId": "nav_1"])
        a.handle(["type": "show", "tabId": NSNull(), "focus": false])
        waitFor { events.contains { $0["type"] as? String == "navigationFinished" } }
        XCTAssertTrue(w.firstResponder === field.currentEditor() || w.firstResponder === field, "background load must not take focus")
        XCTAssertNil(a.view(for: "tab_bg000001")?.window, "background views are not attached to the window")
        a.handle(["type": "show", "tabId": "tab_bg000001", "focus": true])
        XCTAssertTrue(w.firstResponder === a.view(for: "tab_bg000001"))
    }

    func testProfilesUseSeparateWebsiteDataStores() {
        let idA = UUID(), idB = UUID()
        let a = WKWebsiteDataStore(forIdentifier: idA), b = WKWebsiteDataStore(forIdentifier: idB)
        let cookie = HTTPCookie(properties: [.domain: "isolation.justslide.test", .path: "/", .name: "probe", .value: "A", .secure: "TRUE", .expires: Date().addingTimeInterval(600)])!
        let set = expectation(description: "set")
        a.httpCookieStore.setCookie(cookie) { set.fulfill() }
        wait(for: [set], timeout: 5)
        let read = expectation(description: "read")
        var inB: [HTTPCookie] = [], inA: [HTTPCookie] = []
        b.httpCookieStore.getAllCookies { c in inB = c; a.httpCookieStore.getAllCookies { inA = $0; read.fulfill() } }
        wait(for: [read], timeout: 5)
        XCTAssertTrue(inA.contains { $0.name == "probe" })
        XCTAssertFalse(inB.contains { $0.name == "probe" }, "profile B must not see profile A cookies")
        let rm = expectation(description: "cleanup")
        WKWebsiteDataStore.remove(forIdentifier: idA) { _ in WKWebsiteDataStore.remove(forIdentifier: idB) { _ in rm.fulfill() } }
        wait(for: [rm], timeout: 10)
    }

    func testExternalPagesHaveNoPrivilegedBridge() {
        let (_, _, container) = makeWindow()
        let a = WebKitBrowserAdapter(pageContainer: container) { _ in }
        a.handle(["type": "createView", "tabId": "tab_probe0001", "dataStoreId": UUID().uuidString, "private": true, "url": "about:blank", "navigationId": "nav_1"])
        let v = a.view(for: "tab_probe0001")!
        waitFor { !v.isLoading }
        let exp = expectation(description: "js")
        v.evaluateJavaScript("typeof (window.webkit && window.webkit.messageHandlers && window.webkit.messageHandlers.justslide)") { r, _ in
            XCTAssertEqual(r as? String, "undefined"); exp.fulfill()
        }
        wait(for: [exp], timeout: 5)
    }

    /// Investigation: records whether WebKit can snapshot a view that is not in
    /// a window (background tab). The result feeds BrowserEngineCapabilities.
    func testBackgroundSnapshotBehaviourIsReported() {
        let (_, _, container) = makeWindow()
        var events: [[String: Any]] = []
        let a = WebKitBrowserAdapter(pageContainer: container) { events.append($0) }
        a.handle(["type": "createView", "tabId": "tab_snap0001", "dataStoreId": UUID().uuidString, "private": true, "url": "about:blank", "navigationId": "nav_1"])
        waitFor { events.contains { $0["type"] as? String == "navigationFinished" } }
        a.handle(["type": "screenshot", "requestId": "cap_1", "tabId": "tab_snap0001", "maxEdgePx": 640, "format": "png"])
        waitFor { events.contains { $0["type"] as? String == "screenshotResult" } }
        let r = events.first { $0["type"] as? String == "screenshotResult" }!
        print("JUSTSLIDE-EVIDENCE background-snapshot:", (r["capture"] as? [String: Any]).map { "image \($0["width"] ?? 0)x\($0["height"] ?? 0) possiblyStale=\($0["possiblyStale"] ?? "?")" } ?? "error \(r["error"] ?? "?")")
    }

    func testCoreRoundTripOverFFI() {
        guard let core = CoreBridge(dataDirectory: nil, capabilities: WebKitBrowserAdapter.capabilities, inMemory: true, enableAgentIPC: false) else { return XCTFail("core") }
        var commands: [[String: Any]] = []
        core.onEngineCommand = { commands.append($0) }
        var reply: [String: Any]?
        core.execute(id: "t1", command: "tabs.open", args: ["input": "example.com"]) { reply = $0 }
        waitFor { reply != nil && commands.contains { $0["type"] as? String == "show" } }
        XCTAssertEqual(reply?["ok"] as? Bool, true)
        XCTAssertTrue(commands.contains { $0["type"] as? String == "createView" && $0["url"] as? String == "https://example.com/" })
    }
}
