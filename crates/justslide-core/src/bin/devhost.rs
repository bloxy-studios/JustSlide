//! DEVELOPMENT HOST — runs the real JustSlide core and IPC server with the
//! deterministic TEST-DOUBLE engine (no rendering). Used by the MCP protocol
//! harness and for local agent-integration work on machines without the
//! macOS app. Never shipped. Usage:
//!   justslide-devhost --data-dir DIR [--grant-ops read.metadata,write.organize,read.capture --token-file FILE]

use justslide_core::{Actor, Core, Request, ipc, test_engine::TestEngine};
use serde_json::json;
use std::os::unix::fs::PermissionsExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let get = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    let dir = std::path::PathBuf::from(get("--data-dir").ok_or_else(|| anyhow::anyhow!("--data-dir required"))?);
    let core = Core::open(Some(&dir))?;
    let _engine = TestEngine::attach(&core);
    if let (Some(ops), Some(tf)) = (get("--grant-ops"), get("--token-file")) {
        let ops: Vec<&str> = ops.split(',').collect();
        let r = core.execute(&Actor::Human, Request::new("devhost-pair", "grants.create", json!({"client": get("--client").unwrap_or("devhost-agent".into()), "operations": ops}))).await;
        let token = r.result().map_err(|e| anyhow::anyhow!("{e}"))?["pairingToken"].as_str().unwrap().to_string();
        std::fs::write(&tf, &token)?;
        std::fs::set_permissions(&tf, std::fs::Permissions::from_mode(0o600))?;
    }
    let h = ipc::start(core.clone(), &dir).await?;
    eprintln!("justslide-devhost (TEST-DOUBLE ENGINE) listening on {}", h.socket.display());
    tokio::signal::ctrl_c().await.ok();
    Ok(())
}
