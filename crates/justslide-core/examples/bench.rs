//! Core baseline (TEST-DOUBLE engine; measures JustSlide's own command path,
//! persistence and snapshot cost — not WebKit). `cargo run --release -p
//! justslide-core --features test-engine --example bench`
use justslide_core::{Actor, Core, Request, test_engine::TestEngine};
use serde_json::json;
use std::time::Instant;

fn rss_kb() -> u64 {
    std::fs::read_to_string("/proc/self/status").ok().and_then(|s| s.lines().find(|l| l.starts_with("VmRSS")).and_then(|l| l.split_whitespace().nth(1)?.parse().ok())).unwrap_or(0)
}

#[tokio::main]
async fn main() {
    let dir = tempfile::tempdir().unwrap();
    let t0 = Instant::now();
    let core = Core::open(Some(dir.path())).unwrap();
    let _e = TestEngine::attach(&core);
    println!("open+migrate+seed: {:.2} ms, rss {} KiB", t0.elapsed().as_secs_f64() * 1e3, rss_kb());
    let h = Actor::Human;
    let pair = core.execute(&h, Request::new("p", "grants.create", json!({"client": "bench", "operations": ["read.metadata", "write.organize"]}))).await;
    let agent = core.authenticate(pair.result().unwrap()["pairingToken"].as_str().unwrap()).unwrap();
    let ws = core.execute(&agent, Request::new("w", "workspaces.create", json!({"name": "Bench"}))).await.result().unwrap()["workspace"]["id"].as_str().unwrap().to_string();
    let mut n = 0;
    for target in [1usize, 5, 20, 50] {
        let start = Instant::now();
        let mut created = 0;
        while n < target {
            core.execute(&h, Request::new(format!("h{n}"), "tabs.create", json!({"workspaceId": core.debug_state().personal_workspace_id().unwrap(), "url": format!("https://e{n}.example/")}))).await.result().unwrap();
            n += 1; created += 1;
        }
        let per = if created > 0 { start.elapsed().as_secs_f64() * 1e3 / created as f64 } else { 0.0 };
        let s = Instant::now();
        for _ in 0..20 { core.execute(&h, Request::new("s", "state.snapshot", json!({}))).await.result().unwrap(); }
        let snap = s.elapsed().as_secs_f64() * 1e3 / 20.0;
        let a = Instant::now();
        core.execute(&agent, Request::new(format!("a{target}"), "tabs.create", json!({"workspaceId": ws, "url": "https://agent.example/"}))).await.result().unwrap();
        println!("tabs={target:>3}: human tabs.create {per:.3} ms/tab (incl. SQLite txn), state.snapshot {snap:.3} ms, agent tabs.create {:.3} ms, rss {} KiB",
            a.elapsed().as_secs_f64() * 1e3, rss_kb());
    }
}
