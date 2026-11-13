---
trigger: glob
globs: "crates/**, scripts/gen-protocol-ts.mjs"
description: One command model, registry-first protocol, permissions and event ordering.
---
- Change commands in `crates/justslide-protocol/registry/justslide.protocol.v1.json` first, then `bun run protocol:gen`. `exposure: ["ui"]` commands are human-only; never expose activation, grants, approvals or settings to MCP.
- Permissions are enforced in `Core::execute`/`Tx::dispatch`, not in UI. Out-of-scope resources return `not_found`.
- Identity comes from the pairing token only; never trust caller-supplied actor fields.
- Every mutation bumps the revision and is persisted transactionally; private profiles never persist.
- Add a core scenario test for every new command (crates/justslide-core/tests/core_scenarios.rs) and MCP e2e coverage when agent-exposed.
