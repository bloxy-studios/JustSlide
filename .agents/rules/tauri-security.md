---
trigger: model_decision
description: Read when touching the host process model, Tauri evaluation, IPC, entitlements or any trust boundary between the trusted chrome and untrusted webpages.
---
- Current host: AppKit + trusted chrome in a local WKWebView (ADR 0001, pending owner approval). If Tauri v2 is revisited, scope capabilities to the chrome webview label only — never a window/webview wildcard that includes page content.
- No TCP/HTTP automation listener. Agent IPC is a 0600 Unix socket in a 0700 directory with peer-UID check and token handshake.
- Chrome CSP: `script-src 'self'` + hash-pinned import map, `connect-src 'none'` (enforced by `scripts/check-release-bundle.mjs`).
- Threat model and open items: `docs/security-and-privacy.md`.
