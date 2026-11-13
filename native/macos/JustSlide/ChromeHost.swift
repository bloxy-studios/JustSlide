import AppKit
import UniformTypeIdentifiers
import WebKit

/// Serves the bundled trusted chrome at `justslide-chrome://app/…`. Only the
/// chrome WKWebView registers this scheme; page views cannot load it.
final class ChromeSchemeHandler: NSObject, WKURLSchemeHandler {
    let root: URL
    init(root: URL) { self.root = root.standardizedFileURL }

    func webView(_ webView: WKWebView, start task: WKURLSchemeTask) {
        guard let url = task.request.url, url.host == "app" else { return fail(task) }
        let rel = url.path.isEmpty || url.path == "/" ? "index.html" : String(url.path.dropFirst())
        let file = root.appendingPathComponent(rel).standardizedFileURL
        guard file.path.hasPrefix(root.path + "/"), let data = try? Data(contentsOf: file) else { return fail(task) }
        let mime = UTType(filenameExtension: file.pathExtension)?.preferredMIMEType ?? "application/octet-stream"
        let resp = HTTPURLResponse(url: url, statusCode: 200, httpVersion: "HTTP/1.1", headerFields: ["Content-Type": mime, "Cache-Control": "no-store"])!
        task.didReceive(resp)
        task.didReceive(data)
        task.didFinish()
    }

    func webView(_ webView: WKWebView, stop task: WKURLSchemeTask) {}

    private func fail(_ task: WKURLSchemeTask) { task.didFailWithError(URLError(.fileDoesNotExist)) }
}

/// `justslide-favicon://icon/<url>` and `justslide-favicon://site/<host>`:
/// fetches favicons with an ephemeral, cookie-less session and a bounded cache.
final class FaviconSchemeHandler: NSObject, WKURLSchemeHandler {
    private let cache = NSCache<NSString, NSData>()
    private let session: URLSession = {
        let c = URLSessionConfiguration.ephemeral
        c.httpCookieStorage = nil
        c.timeoutIntervalForRequest = 10
        return URLSession(configuration: c)
    }()
    private var active = Set<ObjectIdentifier>()

    override init() { super.init(); cache.countLimit = 300; cache.totalCostLimit = 8 << 20 }

    func webView(_ webView: WKWebView, start task: WKURLSchemeTask) {
        guard let url = task.request.url else { return }
        let target: URL?
        switch url.host {
        case "icon": target = URL(string: String(url.path.dropFirst()).removingPercentEncoding ?? "")
        case "site": target = URL(string: "https://\(url.path.dropFirst())/favicon.ico")
        default: target = nil
        }
        guard let target, ["https", "http"].contains(target.scheme ?? "") else { return task.didFailWithError(URLError(.badURL)) }
        let key = target.absoluteString as NSString
        active.insert(ObjectIdentifier(task))
        if let d = cache.object(forKey: key) { return finish(task, url, d as Data, "image/x-icon") }
        session.dataTask(with: target) { [weak self] data, resp, _ in
            DispatchQueue.main.async {
                guard let self, self.active.contains(ObjectIdentifier(task)) else { return }
                guard let data, (resp as? HTTPURLResponse)?.statusCode == 200, data.count < 512 * 1024 else { self.active.remove(ObjectIdentifier(task)); return task.didFailWithError(URLError(.resourceUnavailable)) }
                self.cache.setObject(data as NSData, forKey: key, cost: data.count)
                self.finish(task, url, data, (resp as? HTTPURLResponse)?.mimeType ?? "image/x-icon")
            }
        }.resume()
    }

    private func finish(_ task: WKURLSchemeTask, _ url: URL, _ data: Data, _ mime: String) {
        active.remove(ObjectIdentifier(task))
        task.didReceive(URLResponse(url: url, mimeType: mime, expectedContentLength: data.count, textEncodingName: nil))
        task.didReceive(data)
        task.didFinish()
    }

    func webView(_ webView: WKWebView, stop task: WKURLSchemeTask) { active.remove(ObjectIdentifier(task)) }
}

/// The privileged bridge. Registered ONLY on the chrome's user content
/// controller; external page views are created with fresh configurations.
@MainActor
final class ChromeMessageHandler: NSObject, WKScriptMessageHandlerWithReply {
    let core: CoreBridge
    var onContentBounds: ((CGRect, CGFloat) -> Void)?
    init(core: CoreBridge) { self.core = core }

    func userContentController(_ ucc: WKUserContentController, didReceive message: WKScriptMessage, replyHandler: @escaping (Any?, String?) -> Void) {
        // Defence in depth: accept only the main frame of our own scheme.
        guard message.frameInfo.isMainFrame, message.frameInfo.securityOrigin.protocol == "justslide-chrome",
              let body = message.body as? [String: Any] else { return replyHandler(nil, "refused") }
        if body["kind"] as? String == "contentBounds", let r = body["rect"] as? [String: Double] {
            onContentBounds?(CGRect(x: r["x"] ?? 0, y: r["y"] ?? 0, width: r["width"] ?? 0, height: r["height"] ?? 0), CGFloat(r["radius"] ?? 16))
            return replyHandler(["ok": true], nil)
        }
        guard let command = body["command"] as? String else { return replyHandler(nil, "malformed") }
        core.execute(id: body["id"] as? String ?? "ui", command: command, args: body["args"] ?? [:]) { replyHandler($0, nil) }
    }
}
