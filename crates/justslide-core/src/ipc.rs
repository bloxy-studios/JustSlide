//! Authenticated local IPC for external agents (consumed by `justslide-mcp`).
//!
//! * Unix-domain socket in an owner-only (0700) directory, socket 0600.
//! * Peer UID must equal ours; then a `hello` with a pairing token.
//! * Each request is re-authorised by `Core::execute` (revocation is
//!   effective on open connections); revocation also closes the socket.
//! * No TCP/HTTP listener exists anywhere in JustSlide.

use crate::core::{Actor, Core, Precondition, Request};
use justslide_protocol::{Registry, validate};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::{Mutex, Semaphore, mpsc};

pub const AUTOMATION_DIR: &str = "JustSlideAutomation";
pub const ENDPOINT_FILE: &str = "endpoint.json";
const SUN_PATH_MAX: usize = 100;

pub struct IpcHandle {
    pub socket: PathBuf,
    pub endpoint_file: PathBuf,
    task: tokio::task::JoinHandle<()>,
}

impl Drop for IpcHandle {
    fn drop(&mut self) {
        self.task.abort();
        let _ = std::fs::remove_file(&self.endpoint_file);
        let _ = std::fs::remove_file(&self.socket);
    }
}

fn euid() -> u32 {
    // SAFETY: geteuid has no preconditions and cannot fail.
    unsafe { libc::geteuid() }
}

fn private_dir(p: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(p)?;
    std::fs::set_permissions(p, std::fs::Permissions::from_mode(0o700))?;
    Ok(())
}

async fn read_frame(r: &mut (impl AsyncReadExt + Unpin), max: usize) -> anyhow::Result<Value> {
    let len = r.read_u32().await? as usize;
    anyhow::ensure!(len <= max, "frame of {len} bytes exceeds {max}");
    let mut buf = vec![0u8; len];
    r.read_exact(&mut buf).await?;
    Ok(serde_json::from_slice(&buf)?)
}

async fn write_frame(w: &mut (impl AsyncWriteExt + Unpin), v: &Value) -> anyhow::Result<()> {
    let b = serde_json::to_vec(v)?;
    w.write_u32(b.len() as u32).await?;
    w.write_all(&b).await?;
    w.flush().await?;
    Ok(())
}

/// Starts the IPC listener and writes the discovery record
/// `<user_data_dir>/JustSlideAutomation/endpoint.json`.
pub async fn start(core: Arc<Core>, user_data_dir: &Path) -> anyhow::Result<IpcHandle> {
    let dir = user_data_dir.join(AUTOMATION_DIR);
    private_dir(&dir)?;
    let instance = core.instance_id();
    let mut socket = dir.join("automation.sock");
    if socket.as_os_str().len() > SUN_PATH_MAX {
        let short = PathBuf::from(format!("/tmp/justslide-{}", euid()));
        private_dir(&short)?;
        socket = short.join(format!("{}.sock", &instance[..instance.len().min(24)]));
    }
    if socket.exists() {
        std::fs::remove_file(&socket)?;
    }
    let listener = UnixListener::bind(&socket)?;
    std::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o600))?;
    let reg = Registry::get();
    let record = json!({"protocol": reg.protocol_name(), "protocolVersion": crate::PROTOCOL_VERSION, "instanceId": instance, "socket": socket, "pid": std::process::id()});
    let endpoint_file = dir.join(ENDPOINT_FILE);
    let tmp = dir.join(".endpoint.json.tmp");
    std::fs::write(&tmp, serde_json::to_vec_pretty(&record)?)?;
    std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o600))?;
    std::fs::rename(&tmp, &endpoint_file)?;
    let task = tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let c = core.clone();
                    tokio::spawn(async move {
                        if let Err(e) = handle(c, stream).await {
                            tracing::debug!("ipc connection ended: {e:#}");
                        }
                    });
                }
                Err(e) => {
                    tracing::warn!("ipc accept failed: {e}");
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        }
    });
    Ok(IpcHandle { socket, endpoint_file, task })
}

async fn handle(core: Arc<Core>, stream: UnixStream) -> anyhow::Result<()> {
    let reg = Registry::get();
    let cred = stream.peer_cred()?;
    anyhow::ensure!(cred.uid() == euid(), "peer uid {} rejected", cred.uid());
    let (mut r, mut w) = stream.into_split();
    let hello = tokio::time::timeout(Duration::from_secs(5), read_frame(&mut r, reg.max_frame_bytes())).await??;
    let refuse = |code: &str, msg: &str| json!({"v": 1, "error": {"code": code, "message": msg, "retryable": false}});
    if let Err(e) = validate(&reg.raw["handshake"]["hello"], &hello) {
        write_frame(&mut w, &refuse("invalid_argument", &e)).await?;
        return Ok(());
    }
    let h = &hello["hello"];
    let Some(actor) = core.authenticate(h["pairingToken"].as_str().unwrap_or_default()) else {
        tokio::time::sleep(Duration::from_millis(250)).await;
        write_frame(&mut w, &refuse("unauthenticated", "pairing token is unknown, revoked or expired")).await?;
        return Ok(());
    };
    if let Some(expect) = h["expectInstanceId"].as_str() {
        if expect != core.instance_id() {
            write_frame(&mut w, &refuse("precondition_failed", "instance id mismatch")).await?;
            return Ok(());
        }
    }
    let Actor::Agent { grant_id, .. } = &actor else { anyhow::bail!("unexpected actor") };
    let grant_id = grant_id.clone();
    write_frame(&mut w, &json!({"v": 1, "welcome": {"instanceId": core.instance_id(), "protocolVersion": crate::PROTOCOL_VERSION, "grantId": grant_id, "commands": core.commands_for(&actor)}})).await?;
    tracing::info!(grant = %grant_id, client = %h["client"], "agent connected");

    let (out_tx, mut out_rx) = mpsc::channel::<Value>(64);
    let max_out = reg.max_result_frame_bytes();
    let writer = tokio::spawn(async move {
        while let Some(v) = out_rx.recv().await {
            let size = serde_json::to_vec(&v).map(|b| b.len()).unwrap_or(0);
            let v = if size > max_out {
                json!({"v": 1, "id": v["id"], "ok": false, "error": {"code": "payload_too_large", "message": "result exceeds frame limit", "retryable": false}})
            } else {
                v
            };
            if write_frame(&mut w, &v).await.is_err() {
                break;
            }
        }
    });
    let permits = Arc::new(Semaphore::new(reg.limit("maxConcurrentRequestsPerConnection", 8) as usize));
    let inflight: Arc<Mutex<HashMap<String, tokio::task::AbortHandle>>> = Arc::default();
    let mut events = core.subscribe();
    let actor = Arc::new(actor);
    loop {
        tokio::select! {
            ev = events.recv() => {
                if let Ok(ev) = ev {
                    if ev.event == "grant.revoked" && ev.data["grantId"].as_str() == Some(grant_id.as_str()) {
                        let _ = out_tx.send(json!({"v": 1, "seq": ev.seq, "event": "grant.revoked", "data": ev.data})).await;
                        break;
                    }
                }
            }
            frame = read_frame(&mut r, reg.max_frame_bytes()) => {
                let frame = frame?;
                if let Some(id) = frame.get("cancel").and_then(Value::as_str) {
                    if let Some(h) = inflight.lock().await.remove(id) { h.abort(); }
                    let _ = out_tx.send(json!({"v": 1, "id": id, "ok": false, "error": {"code": "cancelled", "message": "cancelled by caller", "retryable": false}})).await;
                    continue;
                }
                if let Err(e) = validate(&reg.raw["envelope"]["request"], &frame) {
                    let _ = out_tx.send(json!({"v": 1, "id": frame["id"], "ok": false, "error": {"code": "invalid_argument", "message": e, "retryable": false}})).await;
                    continue;
                }
                let id = frame["id"].as_str().unwrap_or_default().to_string();
                let req = Request {
                    id: id.clone(), command: frame["command"].as_str().unwrap_or_default().into(), args: frame["args"].clone(),
                    precondition: frame.get("precondition").and_then(|p| serde_json::from_value::<Precondition>(p.clone()).ok()),
                };
                let (core, actor, out, permits, inflight2) = (core.clone(), actor.clone(), out_tx.clone(), permits.clone(), inflight.clone());
                let rid = id.clone();
                let task = tokio::spawn(async move {
                    let Ok(_p) = permits.try_acquire() else {
                        let _ = out.send(json!({"v": 1, "id": rid, "ok": false, "error": {"code": "rate_limited", "message": "too many concurrent requests", "retryable": true}})).await;
                        return;
                    };
                    let resp = core.execute(&actor, req).await;
                    inflight2.lock().await.remove(&rid);
                    let _ = out.send(resp.to_wire()).await;
                });
                inflight.lock().await.insert(id, task.abort_handle());
            }
        }
    }
    drop(out_tx);
    let _ = tokio::time::timeout(Duration::from_secs(1), writer).await;
    tracing::info!(grant = %grant_id, "agent connection closed");
    Ok(())
}
