# ADR 0006 — MCP transport
Status: Accepted. `justslide-mcp` is a stdio MCP server (rmcp 3.9.0) that forwards to the app over an authenticated Unix socket; the app is the authority. No localhost HTTP listener, no blanket grants, identity from the pairing token. Remote clients need a separately designed, owner-approved relay (none exists).
