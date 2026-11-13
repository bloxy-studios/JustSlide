//! End-to-end: the real `justslide-mcp` binary over stdio, talking to the
//! REAL JustSlide core + IPC server (with the TEST-DOUBLE rendering engine).
//! This is the M4 protocol harness: an agent creates three background tabs
//! while the human's selection/focus stays unchanged.

use justslide_core::engine::EngineCommand;
use justslide_core::test_engine::TestEngine;
use justslide_core::{Actor, Core, Request, ipc};
use serde_json::{Value, json};
use std::os::unix::fs::PermissionsExt;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};

struct Mcp { _child: Child, stdin: ChildStdin, out: tokio::io::Lines<BufReader<ChildStdout>>, next: i64 }

impl Mcp {
    async fn spawn(udd: &std::path::Path, token: &std::path::Path) -> Mcp {
        let mut child = Command::new(env!("CARGO_BIN_EXE_justslide-mcp"))
            .args(["--user-data-dir", udd.to_str().unwrap(), "--token-file", token.to_str().unwrap()])
            .env("RUST_LOG", "warn")
            .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::inherit())
            .kill_on_drop(true).spawn().unwrap();
        let stdin = child.stdin.take().unwrap();
        let out = BufReader::new(child.stdout.take().unwrap()).lines();
        let mut m = Mcp { _child: child, stdin, out, next: 1 };
        let init = m.request("initialize", json!({"protocolVersion": "2025-11-25", "capabilities": {}, "clientInfo": {"name": "justslide-e2e", "version": "0"}})).await;
        assert_eq!(init["result"]["serverInfo"]["name"], "justslide-mcp", "{init}");
        m.send(json!({"jsonrpc": "2.0", "method": "notifications/initialized", "params": {}})).await;
        m
    }
    async fn send(&mut self, v: Value) { self.stdin.write_all(format!("{v}\n").as_bytes()).await.unwrap(); self.stdin.flush().await.unwrap(); }
    async fn request(&mut self, method: &str, params: Value) -> Value {
        let id = self.next; self.next += 1;
        self.send(json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params})).await;
        loop {
            let line = tokio::time::timeout(std::time::Duration::from_secs(20), self.out.next_line()).await.expect("timeout").unwrap().expect("eof");
            let v: Value = serde_json::from_str(&line).unwrap_or_else(|_| panic!("non-JSON on stdout: {line}"));
            assert_eq!(v["jsonrpc"], "2.0", "{line}");
            if v.get("id") == Some(&json!(id)) { return v; }
        }
    }
    async fn call(&mut self, tool: &str, args: Value) -> Value { self.request("tools/call", json!({"name": tool, "arguments": args})).await["result"].clone() }
}

fn body(r: &Value) -> Value { r["structuredContent"]["result"].clone() }
fn is_err(r: &Value) -> bool { r["isError"] == json!(true) }

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn agent_creates_three_background_tabs_without_interrupting_the_person() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let core = Core::open(Some(dir.path())).unwrap();
    let engine = TestEngine::attach(&core);
    let _ipc = ipc::start(core.clone(), dir.path()).await.unwrap();

    // The person is working in a personal tab.
    let h = Actor::Human;
    let personal = core.execute(&h, Request::new("h1", "tabs.open", json!({"input": "https://personal.example/editor"}))).await.result().unwrap()["tab"]["id"].as_str().unwrap().to_string();
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    let window_before = core.window();

    // The person pairs Claude Code with a scoped grant (token shown once).
    let g = core.execute(&h, Request::new("h2", "grants.create", json!({"client": "claude-code", "operations": ["read.metadata", "write.organize", "read.capture", "write.navigate"]}))).await;
    let g = g.result().unwrap().clone();
    let token_file = dir.path().join("token");
    std::fs::write(&token_file, g["pairingToken"].as_str().unwrap()).unwrap();
    std::fs::set_permissions(&token_file, std::fs::Permissions::from_mode(0o600)).unwrap();

    let mut m = Mcp::spawn(dir.path(), &token_file).await;
    let tools = m.request("tools/list", json!({})).await;
    let names: Vec<String> = tools["result"]["tools"].as_array().unwrap().iter().map(|t| t["name"].as_str().unwrap().to_string()).collect();
    for must in ["justslide_tabs_create", "justslide_workspaces_create", "justslide_page_screenshot", "justslide_tabs_pin"] {
        assert!(names.contains(&must.to_string()), "{must} missing from {names:?}");
    }
    for never in ["justslide_tabs_activate", "justslide_workspaces_switch", "justslide_grants_create", "justslide_approvals_resolve", "justslide_page_snapshot"] {
        assert!(!names.contains(&never.to_string()), "{never} must not be advertised (human-only or not granted)");
    }

    let ws = m.call("justslide_workspaces_create", json!({"name": "Research", "icon": "claude-code"})).await;
    assert!(!is_err(&ws), "{ws}");
    let wsid = body(&ws)["workspace"]["id"].as_str().unwrap().to_string();
    let mut tabs = vec![];
    for u in ["https://example.com/a", "https://example.org/b", "https://example.net/c"] {
        let r = m.call("justslide_tabs_create", json!({"workspaceId": wsid, "url": u})).await;
        assert!(!is_err(&r), "{r}");
        tabs.push(body(&r)["tab"]["id"].as_str().unwrap().to_string());
    }
    assert!(!is_err(&m.call("justslide_tabs_pin", json!({"tabId": tabs[0], "pinned": true})).await));
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
    let shot = m.call("justslide_page_screenshot", json!({"tabId": tabs[1]})).await;
    assert!(!is_err(&shot), "{shot}");
    assert_eq!(body(&shot)["tabId"], tabs[1].as_str());
    assert_eq!(body(&shot)["mimeType"], "image/x-justslide-test-capture", "test-double captures are never presented as real images");

    // Out-of-scope and malformed requests.
    let r = m.call("justslide_tabs_close", json!({"tabId": personal})).await;
    assert!(is_err(&r) && r.to_string().contains("not_found"), "{r}");
    let r = m.call("justslide_tabs_create", json!({"workspaceId": wsid, "url": "file:///etc/passwd"})).await;
    assert!(is_err(&r) && r.to_string().contains("invalid_argument"), "{r}");
    let r = m.call("justslide_tabs_activate", json!({"tabId": tabs[0]})).await;
    assert!(is_err(&r), "{r}");

    assert_eq!(core.window(), window_before, "human window selection unchanged");
    {
        let st = engine.state.lock().unwrap();
        assert_eq!(st.first_responder, personal, "keyboard focus unchanged");
        assert_eq!(st.shown.as_deref(), Some(personal.as_str()));
        let agent_shows = st.log.iter().filter(|c| matches!(c, EngineCommand::Show { tab_id: Some(t), .. } if tabs.contains(t))).count();
        assert_eq!(agent_shows, 0);
    }

    // Revocation: the open MCP session can no longer read or mutate.
    let gid = g["grant"]["id"].as_str().unwrap();
    core.execute(&h, Request::new("h3", "grants.revoke", json!({"grantId": gid}))).await.result().unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    let r = m.call("justslide_tabs_list", json!({})).await;
    assert!(is_err(&r), "revoked token must not read: {r}");
    let r = m.call("justslide_tabs_create", json!({"workspaceId": wsid, "url": "https://example.com/d"})).await;
    assert!(is_err(&r), "revoked token must not mutate: {r}");
    assert_eq!(core.debug_state().tabs.len(), 4);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bad_token_is_refused_and_only_capabilities_are_advertised() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let core = Core::open(Some(dir.path())).unwrap();
    let _ipc = ipc::start(core.clone(), dir.path()).await.unwrap();
    let token_file = dir.path().join("token");
    std::fs::write(&token_file, "x".repeat(43)).unwrap();
    std::fs::set_permissions(&token_file, std::fs::Permissions::from_mode(0o600)).unwrap();
    let mut m = Mcp::spawn(dir.path(), &token_file).await;
    let tools = m.request("tools/list", json!({})).await;
    let names: Vec<&str> = tools["result"]["tools"].as_array().unwrap().iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert_eq!(names, ["justslide_capabilities_get"]);
    let r = m.call("justslide_tabs_list", json!({})).await;
    assert!(is_err(&r));
}
