# MCP, plugins and extensions

`justslide-mcp` (Rust, rmcp **3.9.0**, stdio; stdout = protocol only). It discovers the running app via `endpoint.json`, authenticates with a pairing token, and advertises **only** tools the grant permits (`welcome.commands`); when the app is unreachable only `justslide_capabilities_get` is listed.

Agent tools (MCP-exposed): capabilities.get, workspaces.list/create, tabs.list/create/pin/close/move, favorites.list/add (add → person approval), page.navigate/back/forward/reload/stop, page.snapshot, page.screenshot, events.poll. Never exposed: activation/switching, grants, approvals, pause/resume, settings, state.snapshot.

| Tool category | Default | Scope | Approval | Audit |
|---|---|---|---|---|
| read.metadata | per grant | own + granted workspaces | — | activity log |
| write.organize | per grant | own + granted; never the viewed tab | favorites.add needs approval | activity log |
| write.navigate | per grant | not the tab the person is viewing | — | redacted URL logged |
| read.capture / read.content | per grant | own + granted; ≤2 in flight | — | logged, content untrusted |
| write.interact / write.sensitive | not implemented | — | always per-action (planned) | — |

Pairing (today): the chrome calls `grants.create` (token shown once). Pairing UI, Keychain token storage and pending-approval controls in the chrome are M4/M5 items.

Client setup (after pairing; token file mode 0600):
```json
{ "mcpServers": { "justslide": { "command": "/path/to/justslide-mcp", "args": ["--token-file", "/Users/you/.config/justslide/claude-code.token"] } } }
```
Same shape for Antigravity (`.agents/mcp_config.json`), Cursor (`.cursor/mcp.json`); Claude Code: `claude mcp add justslide -- /path/to/justslide-mcp --token-file …`; Codex: `[mcp_servers.justslide]` with `command`/`args` in `~/.codex/config.toml`. None of these clients were connected in this session. Remote/cloud clients cannot reach the local socket; no relay exists.

Three distinct surfaces: WebExtensions in WebKit (M6), a JustSlide plugin SDK (future), and this MCP API.
