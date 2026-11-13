#!/usr/bin/env python3
"""Loads the built chrome (preview and app) in Chromium under its own CSP and
asserts: zero CSP violations, zero page errors, zero outbound network
requests, and ADS theme variables resolved. Browser preview only."""
import functools, http.server, json, socketserver, sys, threading
from pathlib import Path
from playwright.sync_api import sync_playwright
ROOT = Path(__file__).resolve().parents[2]
class Q(http.server.SimpleHTTPRequestHandler):
    def log_message(self, *a): pass
def run(dist, page_name, expect_render):
    httpd = socketserver.TCPServer(("127.0.0.1", 0), functools.partial(Q, directory=str(dist)))
    threading.Thread(target=httpd.serve_forever, daemon=True).start()
    origin = f"http://127.0.0.1:{httpd.server_address[1]}"
    out = {"page": page_name, "cspViolations": [], "pageErrors": [], "externalRequests": []}
    with sync_playwright() as p:
        b = p.chromium.launch(); pg = b.new_page()
        pg.on("console", lambda m: out["cspViolations"].append(m.text) if "Content Security Policy" in m.text else None)
        pg.on("pageerror", lambda e: out["pageErrors"].append(str(e)))
        pg.on("request", lambda r: out["externalRequests"].append(r.url) if not r.url.startswith(origin) and not r.url.startswith("data:") else None)
        pg.goto(f"{origin}/{page_name}"); pg.wait_for_timeout(1200)
        out["dsSurface"] = pg.evaluate("getComputedStyle(document.documentElement).getPropertyValue('--ds-surface').trim()")
        out["dsText"] = pg.evaluate("getComputedStyle(document.documentElement).getPropertyValue('--ds-text').trim()")
        out["rendered"] = pg.evaluate("!!document.querySelector('[data-figma=\"1:9\"]')")
        b.close()
    httpd.shutdown()
    ok = not out["cspViolations"] and not out["externalRequests"] and (out["rendered"] == expect_render) and (not expect_render or (out["dsSurface"] and not out["pageErrors"]))
    return ok, out
results = []
ok1, r1 = run(ROOT / "apps/browser-chrome/dist/preview", "preview.html", True); results.append(r1)
# The app entry needs the native bridge; in a browser it must fail closed (no render, no network, no CSP violations).
ok2, r2 = run(ROOT / "apps/browser-chrome/dist/app", "index.html", False); results.append(r2)
print(json.dumps(results, indent=1)[:2500])
sys.exit(0 if ok1 and ok2 else 1)
