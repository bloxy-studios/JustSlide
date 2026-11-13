# Architecture

```
Person ─▶ Trusted chrome (React/ADS, local WKWebView, justslide-chrome://app)
              │  WKScriptMessageHandlerWithReply "justslide" (chrome only)
              ▼
         CoreBridge.swift ──C ABI (JSON)──▶ justslide-core (Rust)  ◀── IPC (Unix socket, token) ◀── justslide-mcp (stdio) ◀── MCP clients
              ▲                                   │ EngineCommand
              │ EngineEvent (real WebKit callbacks)▼
         WebKitBrowserAdapter.swift ── one WKWebView per tab, WKWebsiteDataStore per profile
```

| Component | Path | Owns | Verified here |
|---|---|---|---|
| Protocol registry | `crates/justslide-protocol` | 40 commands, 17 events, limits, error codes; exposure `ui`/`mcp` | Rust tests |
| Core | `crates/justslide-core` | windows, workspaces, tabs, favorites, settings, grants, approvals, activity, history; permission engine; SQLite; event log; IPC server | 21 tests |
| FFI | `crates/justslide-ffi` | C ABI for the Swift host | C-ABI round-trip test |
| MCP bridge | `crates/justslide-mcp` | stdio MCP (rmcp 3.9.0) → IPC; no state | 2 e2e tests vs real core |
| Chrome UI | `apps/browser-chrome` | sidebar UI; renders core snapshots; sends commands | 11 tests, geometry 0 fail, CSP check |
| macOS host | `native/macos` | NSWindow, chrome WKWebView, page container, WebKit adapter, schemes | **not compiled (no Mac build path this session)** |

Key rules
- **Single authority**: every UI action and MCP tool is a registry command executed by `Core::execute(actor, request)`. The chrome re-renders from `state.snapshot` after each core event (coalesced).
- **Engine boundary**: core emits typed `EngineCommand`s (`createView`, `navigate`, `show`, `screenshot`, …); the adapter reports `EngineEvent`s (`navigationCommitted`, `processTerminated`, `popupRequested`, …). Core never marks a load successful without an engine event.
- **Focus invariant**: only human commands emit `show`/`focus`. Agent commands are checked post-hoc (window state must be identical).
- **Laziness**: restored tabs are `notLoaded`; only the active tab gets a view on restore.
- **Isolation**: personal and each agent-oriented workspace use separate profiles/data stores; agent-created workspaces share one isolated profile per grant.
- Threading: core runs on its own tokio runtime; Swift callbacks copy JSON and hop to the main queue; WebKit is touched only on the main actor.
