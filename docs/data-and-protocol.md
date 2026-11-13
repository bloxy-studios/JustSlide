# Data and protocol

**Registry**: `crates/justslide-protocol/registry/justslide.protocol.v1.json` (v1.1.0) — commands with `operation` (permission), `idempotency`, `activates`, `exposure`, JSON-schema arguments/results; events; limits; handshake/envelope. TypeScript: `bun run protocol:gen` (drift check `protocol:check`).

**Wire (agents)**: Unix socket, frames = u32 BE length + JSON (≤64 KiB in, ≤8 MiB out). `hello{client, clientVersion, pairingToken, expectInstanceId}` → `welcome{instanceId, protocolVersion, grantId, commands}`. Requests `{v,id,command,args,precondition?}` → `{v,id,ok,result,revision}` | `{v,id,ok:false,error{code,message,retryable,pendingRequestId?}}`. Cancel `{v,cancel:id}`. Pushed `grant.revoked` then close. Discovery: `<data-dir>/JustSlideAutomation/endpoint.json` (0600, dir 0700).

**Entities** (`crates/justslide-core/src/model.rs`): Profile (kind personal/agent-isolated/private, dataStoreId), Workspace (icon, profile, agentOwned, ownerGrantId, order, pinnedCollapsed), Tab (url, title, favicon, pinned/order, loadState notLoaded/loading/loaded/failed/crashed, navigationId, canGoBack/Forward, secure, openedBy), Favorite, Settings, WindowState, Grant (hashed token, operations, workspaceIds, expiry, revoked, paused), Approval, ActivityRecord (redacted URL), ClosedTab, HistoryEntry.

**Semantics**: revision++ per committed mutation; `precondition.revision`/`navigationId` → `precondition_failed`; request-id idempotency cache (10 min, 1024); out-of-scope → `not_found`; event log ring (1024) with `resyncRequired`.

**Storage**: SQLite (WAL), `PRAGMA user_version` migrations; refuses newer schemas. Private profiles never written. History only for persisted profiles. No cookies/passwords/WebKit data copied. Pairing tokens stored as SHA-256 only.
