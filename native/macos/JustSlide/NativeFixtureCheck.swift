#if DEBUG
import AppKit
import CryptoKit
import WebKit

/// DEBUG-ONLY native visual check. Runs after the design fixture loads in the
/// real chrome WKWebView and reports, to stdout (`JUSTSLIDE-NATIVE-CHECK …`):
///  1. region geometry vs the Figma node boxes (same selectors/tolerance as
///     tests/visual/capture_and_compare.py), and
///  2. pixel differences between a WebKit snapshot of the chrome and the
///     Figma reference render shipped in the DEBUG bundle.
/// The snapshot PNG is saved under Application Support/JustSlide Dev/evidence.
@MainActor
enum NativeFixtureCheck {
    static func run(chrome: WKWebView, variant: String, previewRoot: URL) {
        poll(chrome, variant: variant, attempts: 60) { ready in
            guard ready else { return log(["variant": variant, "error": "fixture did not become ready"]) }
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.6) { measure(chrome: chrome, variant: variant, previewRoot: previewRoot) }
        }
    }

    private static func poll(_ v: WKWebView, variant: String, attempts: Int, done: @escaping (Bool) -> Void) {
        v.evaluateJavaScript("document.readyState === 'complete' && !!document.querySelector('[data-fixture=\"\(variant)\"] [data-figma=\"1:9\"]') && document.fonts.status === 'loaded'") { r, _ in
            if (r as? Bool) == true { return done(true) }
            if attempts <= 0 { return done(false) }
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.2) { poll(v, variant: variant, attempts: attempts - 1, done: done) }
        }
    }

    private static let geometryJS = """
    const tol = expected.tolerancePx, exp = expected[variant];
    const box = (el) => { const r = el.getBoundingClientRect(); return [r.x, r.y, r.width, r.height]; };
    const delta = (a, b) => Math.max(Math.abs(a[0]-b[0]), Math.abs(a[1]-b[1]), Math.abs(a[0]+a[2]-b[0]-b[2]), Math.abs(a[1]+a[3]-b[1]-b[3]));
    const regions = [], lists = []; let failures = 0;
    for (const [name, fb] of Object.entries(exp.regions)) {
      const el = document.querySelector(expected.regionSelectors[name]);
      if (!el) { regions.push({name, missing: true}); failures++; continue; }
      const d = delta(fb, box(el)); if (d > tol) failures++;
      regions.push({name, d: Math.round(d * 100) / 100});
    }
    for (const [name, fl] of Object.entries(exp.lists)) {
      const dl = [...document.querySelectorAll(expected.listSelectors[name])].map(box);
      const ds = fl.map((b, i) => dl[i] ? delta(b, dl[i]) : 999);
      const ok = fl.length === dl.length && ds.every(d => d <= tol); if (!ok) failures++;
      lists.push({name, figma: fl.length, dom: dl.length, maxD: Math.round(Math.max(0, ...ds) * 100) / 100});
    }
    const sc = document.querySelector('[data-figma="12:289"]').getBoundingClientRect();
    const rows = [...document.querySelectorAll('[data-figma="25:4"] li, [data-figma="12:289"] li')].map(box).filter(b => b[1] < sc.bottom);
    const rds = exp.rows.map((b, i) => rows[i] ? delta(b, rows[i]) : 999);
    if (!(rows.length === exp.rows.length && rds.every(d => d <= tol))) failures++;
    lists.push({name: 'Tab rows (visible)', figma: exp.rows.length, dom: rows.length, maxD: Math.round(Math.max(0, ...rds) * 100) / 100});
    const title = document.querySelector('[aria-current="page"] span');
    return {failures, regions, lists, viewport: [innerWidth, innerHeight], dpr: devicePixelRatio,
            titleFont: title ? getComputedStyle(title).font : null, titleWidth: title ? title.getBoundingClientRect().width : null};
    """

    private static func measure(chrome: WKWebView, variant: String, previewRoot: URL) {
        guard let data = try? Data(contentsOf: previewRoot.appendingPathComponent("figma/figma-expected-boxes.json")),
              let expected = try? JSONSerialization.jsonObject(with: data) else { return log(["variant": variant, "error": "expected boxes missing from DEBUG bundle"]) }
        chrome.callAsyncJavaScript(geometryJS, arguments: ["expected": expected, "variant": variant], in: nil, in: .page) { result in
            var report: [String: Any] = ["variant": variant, "kind": "native WKWebView (macOS), design fixture"]
            switch result {
            case .success(let v): report["geometry"] = v
            case .failure(let e): report["geometryError"] = e.localizedDescription
            }
            let cfg = WKSnapshotConfiguration()
            chrome.takeSnapshot(with: cfg) { image, error in
                guard let image, let cg = image.cgImage(forProposedRect: nil, context: nil, hints: nil) else {
                    report["snapshotError"] = error?.localizedDescription ?? "no image"
                    return log(report)
                }
                let scale = Int((Double(cg.width) / Double(chrome.bounds.width)).rounded())
                report["snapshot"] = ["width": cg.width, "height": cg.height, "scale": scale]
                if let png = NSBitmapImageRep(cgImage: cg).representation(using: .png, properties: [:]) {
                    let dir = AppDelegate.dataDirectory.appendingPathComponent("evidence", isDirectory: true)
                    try? FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
                    let file = dir.appendingPathComponent("native-fixture-\(variant)@\(scale)x.png")
                    try? png.write(to: file)
                    report["png"] = ["path": file.path, "bytes": png.count, "sha256": SHA256.hash(data: png).map { String(format: "%02x", $0) }.joined()]
                }
                let node = variant == "compact" ? "35-13" : "1-6"
                if let ref = NSImage(contentsOf: previewRoot.appendingPathComponent("figma/\(node)@\(scale)x.png"))?.cgImage(forProposedRect: nil, context: nil, hints: nil) {
                    report["pixels"] = compare(cg, ref, regions: (expected as? [String: Any]).flatMap { ($0[variant] as? [String: Any])?["regions"] as? [String: [Double]] } ?? [:], scale: scale)
                } else {
                    report["pixelsError"] = "Figma render \(node)@\(scale)x.png not in bundle"
                }
                log(report)
            }
        }
    }

    private static func rgba(_ img: CGImage, _ w: Int, _ h: Int) -> [UInt8] {
        var buf = [UInt8](repeating: 0, count: w * h * 4)
        let ctx = CGContext(data: &buf, width: w, height: h, bitsPerComponent: 8, bytesPerRow: w * 4, space: CGColorSpace(name: CGColorSpace.sRGB)!, bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue)!
        ctx.draw(img, in: CGRect(x: 0, y: 0, width: w, height: h))
        return buf
    }

    /// Mean absolute difference and share of pixels whose max channel differs
    /// by more than 32/255, for the whole frame and each Figma region.
    private static func compare(_ a: CGImage, _ b: CGImage, regions: [String: [Double]], scale: Int) -> [String: Any] {
        let w = min(a.width, b.width), h = min(a.height, b.height)
        let pa = rgba(a, w, h), pb = rgba(b, w, h)
        func stats(_ x0: Int, _ y0: Int, _ x1: Int, _ y1: Int) -> [String: Double] {
            var sum = 0.0, over = 0, n = 0
            for y in max(0, y0)..<min(h, y1) { for x in max(0, x0)..<min(w, x1) {
                let i = (y * w + x) * 4
                let d = max(abs(Int(pa[i]) - Int(pb[i])), abs(Int(pa[i+1]) - Int(pb[i+1])), abs(Int(pa[i+2]) - Int(pb[i+2])))
                sum += Double(d); if d > 32 { over += 1 }; n += 1
            } }
            return n == 0 ? [:] : ["meanAbsDiff": (sum / Double(n) * 100).rounded() / 100, "pctOver32": (Double(over) / Double(n) * 10000).rounded() / 100]
        }
        var out: [String: Any] = ["sizeCompared": [w, h], "sizes": [[a.width, a.height], [b.width, b.height]], "whole": stats(0, 0, w, h)]
        var per: [String: Any] = [:]
        for (name, r) in regions where r.count == 4 {
            let s = Double(scale)
            per[name] = stats(Int(r[0] * s), Int(r[1] * s), Int((r[0] + r[2]) * s), Int((r[1] + r[3]) * s))
        }
        out["regions"] = per
        return out
    }

    private static func log(_ obj: [String: Any]) {
        let data = (try? JSONSerialization.data(withJSONObject: obj, options: [.sortedKeys])) ?? Data()
        print("JUSTSLIDE-NATIVE-CHECK " + (String(data: data, encoding: .utf8) ?? "{}"))
        fflush(stdout)
    }
}
#endif
