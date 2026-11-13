#!/usr/bin/env python3
"""Figma-vs-preview visual regression for the JustSlide chrome.

What this is: the React chrome rendered in a DESKTOP BROWSER (Playwright
Chromium, optionally WebKitGTK) against the development design fixture. It is
NOT a native macOS WKWebView capture; those live in docs/evidence/native/.

Checks
  1. Region geometry: every mapped Figma node's bounding box (from the Figma
     REST node tree saved in design/figma/) vs the DOM element's box. Fails
     when any edge differs by more than TOLERANCE_PX.
  2. Pixel comparison per region (mean absolute difference, share of pixels
     differing by > 32/255) for review, plus side-by-side composites.

Usage: python3 tests/visual/capture_and_compare.py [--engines chromium,webkit] [--out DIR]
"""
import argparse, datetime, functools, http.server, json, os, socketserver, sys, threading
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw, ImageFont
from playwright.sync_api import sync_playwright

ROOT = Path(__file__).resolve().parents[2]
DIST = ROOT / "apps/browser-chrome/dist/preview"
FIGMA = ROOT / "design/figma"
TOLERANCE_PX = 1.0

FRAMES = {
    "desktop": {"node": "1:6", "size": (1440, 1024), "render": "1-6"},
    "compact": {"node": "35:13", "size": (1100, 720), "render": "35-13"},
}

# Figma layer name -> DOM selector (same semantic slot in both frames).
REGIONS = {
    "Sidebar": '[data-figma="1:9"]',
    "Navigation Controls": '[data-figma="6:17"]',
    "Address Bar": '[data-figma="7:46"]',
    "Website favorites / 12 sites": '[data-figma="7:87"]',
    "Pinned Section": '[data-figma="25:4"]',
    "Browser Sidebar": '[data-figma="20:82"]',
    "Tabs / Independently scrolling": '[data-figma="12:289"]',
    "Background agent activity / Illustrative state": '[data-figma="35:313"]',
    "Workspace footer": '[data-figma="12:290"]',
    "Web Page Window": '[data-figma="1:7"]',
}
# Ordered children compared index-by-index.
LISTS = {
    "Website favorites / 12 sites": '[data-figma="7:87"] button',
    "Workspace controls": '[data-figma="12:290"] > div:last-child button',
}


def load_tree():
    d = json.loads((FIGMA / "nodes-1-6_30-52_35-13.json").read_text())
    return d


def figma_boxes(tree, frame_id):
    doc = tree["nodes"][frame_id]["document"]
    ox, oy = doc["absoluteBoundingBox"]["x"], doc["absoluteBoundingBox"]["y"]
    named, children = {}, {}

    def walk(n):
        if n.get("visible") is False:
            return
        bb = n.get("absoluteBoundingBox")
        if bb:
            box = (bb["x"] - ox, bb["y"] - oy, bb["width"], bb["height"])
            named.setdefault(n["name"], box)
            if n["name"] in ("Website favorites / 12 sites",):
                children[n["name"]] = [(c["absoluteBoundingBox"]["x"] - ox, c["absoluteBoundingBox"]["y"] - oy, c["absoluteBoundingBox"]["width"], c["absoluteBoundingBox"]["height"]) for c in n.get("children", []) if c.get("visible") is not False and c["type"] != "INSTANCE"]
            if n["name"] == "Workspace controls":
                out = []
                for c in n.get("children", []):
                    if c["name"] == "Workspace switcher":
                        out += [(g["absoluteBoundingBox"]["x"] - ox, g["absoluteBoundingBox"]["y"] - oy, 32, 32) for g in c["children"]]
                    else:
                        out.append((c["absoluteBoundingBox"]["x"] - ox, c["absoluteBoundingBox"]["y"] - oy, c["absoluteBoundingBox"]["width"], c["absoluteBoundingBox"]["height"]))
                children["Workspace controls"] = out
        for k in n.get("children", []):
            walk(k)

    walk(doc)
    # Tab rows: visible pinned items then tab rows in order.
    rows = []

    def rows_walk(n, inside=False):
        if n.get("visible") is False:
            return
        if n["name"] in ("Pinned Item", "Tab") or n["name"].startswith("Tab / "):
            bb = n["absoluteBoundingBox"]
            rows.append((bb["x"] - ox, bb["y"] - oy, bb["width"], bb["height"]))
            return
        for k in n.get("children", []):
            rows_walk(k)

    rows_walk(doc)
    sc = named.get("Tabs / Independently scrolling")
    if sc:
        rows = [r for r in rows if r[1] < sc[1] + sc[3]]
    return named, children, rows


class Quiet(http.server.SimpleHTTPRequestHandler):
    def log_message(self, *a):
        pass


def serve():
    handler = functools.partial(Quiet, directory=str(DIST))
    httpd = socketserver.TCPServer(("127.0.0.1", 0), handler)
    threading.Thread(target=httpd.serve_forever, daemon=True).start()
    return httpd


def dom_boxes(page):
    js = """([regions, lists]) => {
      const box = (el) => { const r = el.getBoundingClientRect(); return [r.x, r.y, r.width, r.height]; };
      const out = {regions: {}, lists: {}, rows: []};
      for (const [k, sel] of Object.entries(regions)) { const el = document.querySelector(sel); if (el) out.regions[k] = box(el); }
      for (const [k, sel] of Object.entries(lists)) out.lists[k] = [...document.querySelectorAll(sel)].map(box);
      const sc = document.querySelector('[data-figma="12:289"]').getBoundingClientRect();
      out.rows = [...document.querySelectorAll('[data-figma="25:4"] li, [data-figma="12:289"] li')].map(box).filter(b => b[1] < sc.bottom);
      return out; }"""
    return page.evaluate(js, [REGIONS, LISTS])


def edge_delta(a, b):
    return max(abs(a[0] - b[0]), abs(a[1] - b[1]), abs((a[0] + a[2]) - (b[0] + b[2])), abs((a[1] + a[3]) - (b[1] + b[3])))


def compare_pixels(fig, prev, box, scale):
    x, y, w, h = [int(round(v * scale)) for v in box]
    a = np.asarray(fig.crop((x, y, x + w, y + h)).convert("RGB"), dtype=np.int16)
    b = np.asarray(prev.crop((x, y, x + w, y + h)).convert("RGB"), dtype=np.int16)
    if a.size == 0:
        return None
    d = np.abs(a - b).max(axis=2)
    return {"meanAbsDiff": round(float(d.mean()), 2), "pctPixelsOver32": round(float((d > 32).mean() * 100), 2)}


def composite(fig, prev, out, title):
    w, h = fig.size
    diff = np.abs(np.asarray(fig.convert("RGB"), dtype=np.int16) - np.asarray(prev.convert("RGB"), dtype=np.int16)).max(axis=2)
    heat = np.zeros((h, w, 3), dtype=np.uint8)
    heat[..., 0] = np.clip(diff * 3, 0, 255)
    base = np.asarray(fig.convert("L"))[..., None].repeat(3, axis=2) // 3
    heat = np.maximum(heat, base.astype(np.uint8))
    pad = 36
    canvas = Image.new("RGB", (w * 3 + 40, h + pad), "white")
    canvas.paste(fig.convert("RGB"), (0, pad))
    canvas.paste(prev.convert("RGB"), (w + 20, pad))
    canvas.paste(Image.fromarray(heat), (2 * w + 40, pad))
    dr = ImageDraw.Draw(canvas)
    f = ImageFont.load_default()
    for i, lab in enumerate(["FIGMA (reference render)", f"PREVIEW ({title}) - not native", "DIFF (red = differs)"]):
        dr.text((i * (w + 20) + 8, 10), lab, fill="black", font=f)
    canvas.save(out)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--engines", default="chromium")
    ap.add_argument("--out", default=str(ROOT / "docs/evidence/visual" / datetime.date.today().isoformat()))
    args = ap.parse_args()
    out = Path(args.out)
    out.mkdir(parents=True, exist_ok=True)
    tree = load_tree()
    httpd = serve()
    base = f"http://127.0.0.1:{httpd.server_address[1]}/preview.html"
    report = {"generatedAt": datetime.datetime.now().isoformat(timespec="seconds"), "kind": "browser-preview (NOT native WKWebView)",
              "figmaFileVersion": tree.get("version"), "figmaLastModified": tree.get("lastModified"), "tolerancePx": TOLERANCE_PX, "results": []}
    failures = 0
    with sync_playwright() as p:
        for engine in args.engines.split(","):
            try:
                browser = getattr(p, engine).launch()
            except Exception as e:  # e.g. missing WebKitGTK system libraries
                report["results"].append({"engine": engine, "status": "not-run", "reason": str(e).splitlines()[0][:300]})
                continue
            ver = browser.version
            for variant, spec in FRAMES.items():
                named, kids, rows = figma_boxes(tree, spec["node"])
                for scale in (1, 2):
                    page = browser.new_page(viewport={"width": spec["size"][0], "height": spec["size"][1]}, device_scale_factor=scale)
                    page.goto(f"{base}?variant={variant}")
                    page.wait_for_selector('[data-figma="1:9"]')
                    page.wait_for_timeout(600)
                    shot = out / f"preview-{engine}-{variant}-{spec['size'][0]}x{spec['size'][1]}@{scale}x.png"
                    page.screenshot(path=str(shot))
                    dom = dom_boxes(page)
                    page.close()
                    fig = Image.open(FIGMA / "renders" / f"{spec['render']}@{scale}x.png")
                    prev = Image.open(shot)
                    if fig.size != prev.size:
                        fig = fig.resize(prev.size)
                    regions = []
                    for name, fbox in named.items():
                        if name not in REGIONS:
                            continue
                        dbox = dom["regions"].get(name)
                        if dbox is None:
                            regions.append({"region": name, "status": "missing-in-dom"}); failures += 1; continue
                        delta = edge_delta(fbox, dbox)
                        ok = delta <= TOLERANCE_PX
                        failures += 0 if ok else 1
                        regions.append({"region": name, "figma": [round(v, 1) for v in fbox], "dom": [round(v, 1) for v in dbox], "maxEdgeDeltaPx": round(delta, 2), "pass": ok, "pixels": compare_pixels(fig, prev, fbox, scale)})
                    lists = []
                    for name, fl in kids.items():
                        dl = dom["lists"].get(name, [])
                        deltas = [round(edge_delta(a, b), 2) for a, b in zip(fl, dl)]
                        ok = len(fl) == len(dl) and all(d <= TOLERANCE_PX for d in deltas)
                        failures += 0 if ok else 1
                        lists.append({"list": name, "figmaCount": len(fl), "domCount": len(dl), "maxDeltaPx": max(deltas) if deltas else None, "pass": ok})
                    n = min(len(rows), len(dom["rows"]))
                    rdeltas = [round(edge_delta(rows[i], dom["rows"][i]), 2) for i in range(n)]
                    rows_ok = all(d <= TOLERANCE_PX for d in rdeltas) and n > 0
                    failures += 0 if rows_ok else 1
                    lists.append({"list": "Tab rows (visible)", "figmaCount": len(rows), "domCount": len(dom["rows"]), "compared": n, "maxDeltaPx": max(rdeltas) if rdeltas else None, "pass": rows_ok})
                    comp = out / f"compare-{engine}-{variant}-{spec['size'][0]}x{spec['size'][1]}@{scale}x.png"
                    composite(fig, prev, comp, f"{engine} {ver}")
                    whole = compare_pixels(fig, prev, (0, 0, spec["size"][0], spec["size"][1]), scale)
                    report["results"].append({"engine": engine, "engineVersion": ver, "variant": variant, "figmaNode": spec["node"], "size": spec["size"], "scale": scale,
                                              "screenshot": shot.name, "composite": comp.name, "wholeFrame": whole, "regions": regions, "lists": lists})
            browser.close()
    httpd.shutdown()
    report["geometryFailures"] = failures
    (out / "report.json").write_text(json.dumps(report, indent=2))
    for r in report["results"]:
        if r.get("status") == "not-run":
            print(f"{r['engine']}: NOT RUN ({r['reason']})"); continue
        bad = [x for x in r["regions"] if not x.get("pass")] + [x for x in r["lists"] if not x["pass"]]
        print(f"{r['engine']} {r['variant']} @{r['scale']}x: {len(r['regions'])} regions, {len(r['lists'])} lists, {len(bad)} geometry failures; whole-frame {r['wholeFrame']}")
        for x in bad:
            print("   FAIL", json.dumps(x)[:400])
    print(f"report: {out / 'report.json'}")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
