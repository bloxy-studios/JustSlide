# Security and privacy

Trust boundaries: external pages → WebKit content processes (untrusted) · trusted chrome WKWebView · Swift host · Rust core · IPC socket · MCP bridge · agents.

| Threat | Mitigation | Test |
|---|---|---|
| Page calls privileged bridge | handler only on chrome config; origin check `justslide-chrome`; page views use fresh configs | `testExternalPagesHaveNoPrivilegedBridge` (native, not run) |
| Page navigates to internal scheme | adapter cancels non-web schemes; core accepts only http(s) from agents | core URL tests |
| Agent reads personal tabs | grant scope; out-of-scope = not_found; events filtered | `agents_cannot_see_or_touch_personal_tabs…`, `agent_event_stream_is_scoped` |
| Agent steals focus / changes viewed tab | no activation commands for agents; viewed tab protected; post-check | `agent_works_in_background…`, MCP e2e |
| Agent escalates itself | grants/approvals/pause are UI-only; identity from token | `human_only_and_unsafe_inputs…`, `injected_page_text_grants_nothing` |
| Revoked token keeps working | per-request authorisation + socket close | `revocation_is_immediate…`, MCP e2e |
| Local process impersonation | socket 0600 in 0700 dir, peer-UID check, 256-bit tokens hashed, constant-time compare | IPC code; endpoint perms checked by client |
| Chrome exfiltration | CSP `connect-src 'none'`, hash-pinned scripts | `tests/visual/csp_check.py` |
| Prompt injection via page text | snapshots marked `untrustedContent`; MCP instructions warn; no capability is derived from content | core test |

Privacy: activity logs keep redacted URLs (no query/fragment/credentials), capped at 2 000 rows; private profiles are never persisted; no telemetry.

Open items: App Sandbox is **off** in the development build (socket location); pairing UI + Keychain; approval prompts as native sheets; per-site permissions (camera/mic/geolocation) default-deny; download prompts; DNS/HTTPS-only policy.
