//! Authenticated, length-framed client for the browser's local automation
//! socket. One connection per bridge process; requests are multiplexed by id.

use crate::endpoint::Endpoint;
use justslide_protocol::Registry;
use anyhow::{Context, Result, anyhow, bail, ensure};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::net::unix::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::{Mutex, Semaphore, oneshot};

#[derive(Debug, Clone)]
pub struct Welcome {
    pub instance_id: String,
    pub protocol_version: String,
    pub grant_id: String,
    pub commands: Vec<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct RemoteError {
    pub code: String,
    pub message: String,
    #[serde(default)]
    pub retryable: bool,
    #[serde(default, rename = "pendingRequestId")]
    pub pending_request_id: Option<String>,
}

#[derive(Debug)]
pub enum CallOutcome {
    Ok { result: Value, revision: i64 },
    Err(RemoteError),
}

type Pending = Arc<Mutex<HashMap<String, oneshot::Sender<Value>>>>;

pub async fn write_frame(w: &mut OwnedWriteHalf, v: &Value, max: usize) -> Result<()> {
    let bytes = serde_json::to_vec(v)?;
    ensure!(bytes.len() <= max, "frame of {} bytes exceeds limit {}", bytes.len(), max);
    w.write_u32(bytes.len() as u32).await?;
    w.write_all(&bytes).await?;
    w.flush().await?;
    Ok(())
}

pub async fn read_frame(r: &mut OwnedReadHalf, max: usize) -> Result<Value> {
    let len = r.read_u32().await? as usize;
    ensure!(len <= max, "incoming frame of {len} bytes exceeds limit {max}");
    let mut buf = vec![0u8; len];
    r.read_exact(&mut buf).await?;
    Ok(serde_json::from_slice(&buf)?)
}

pub struct Connection {
    writer: Mutex<OwnedWriteHalf>,
    pending: Pending,
    pub welcome: Welcome,
    inflight: Semaphore,
    closed: Arc<std::sync::atomic::AtomicBool>,
    next: AtomicU64,
}

impl Connection {
    pub async fn connect(ep: &Endpoint, client: &str, token: &str) -> Result<Arc<Connection>> {
        let reg = Registry::get();
        let stream = tokio::time::timeout(Duration::from_secs(5), UnixStream::connect(&ep.record.socket))
            .await
            .context("timed out connecting to JustSlide")?
            .with_context(|| format!("connecting to {}", ep.record.socket.display()))?;
        let (mut r, mut w) = stream.into_split();
        let hello = json!({"v": 1, "hello": {
            "client": client,
            "clientVersion": env!("CARGO_PKG_VERSION"),
            "pairingToken": token,
            "expectInstanceId": ep.record.instance_id,
        }});
        write_frame(&mut w, &hello, reg.max_frame_bytes()).await?;
        let reply = tokio::time::timeout(Duration::from_secs(5), read_frame(&mut r, reg.max_frame_bytes()))
            .await
            .context("timed out waiting for JustSlide handshake")??;
        if let Some(err) = reply.get("error") {
            bail!("JustSlide refused the connection: {}", err.get("message").and_then(Value::as_str).unwrap_or("unknown"));
        }
        let wv = reply.get("welcome").ok_or_else(|| anyhow!("malformed handshake reply"))?;
        let welcome = Welcome {
            instance_id: wv["instanceId"].as_str().unwrap_or_default().to_string(),
            protocol_version: wv["protocolVersion"].as_str().unwrap_or_default().to_string(),
            grant_id: wv["grantId"].as_str().unwrap_or_default().to_string(),
            commands: wv["commands"].as_array().map(|a| a.iter().filter_map(|c| c.as_str().map(String::from)).collect()).unwrap_or_default(),
        };
        ensure!(welcome.instance_id == ep.record.instance_id, "connected instance {} does not match discovery record {}", welcome.instance_id, ep.record.instance_id);
        let pending: Pending = Arc::default();
        let closed = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let conn = Arc::new(Connection {
            writer: Mutex::new(w),
            pending: pending.clone(),
            welcome,
            inflight: Semaphore::new(reg.raw["limits"]["maxConcurrentRequestsPerConnection"].as_u64().unwrap_or(8) as usize),
            closed: closed.clone(),
            next: AtomicU64::new(1),
        });
        let max_in = reg.max_result_frame_bytes();
        tokio::spawn(async move {
            loop {
                match read_frame(&mut r, max_in).await {
                    Ok(frame) => {
                        if let Some(id) = frame.get("id").and_then(Value::as_str) {
                            if let Some(tx) = pending.lock().await.remove(id) {
                                let _ = tx.send(frame);
                            }
                        } else if frame.get("event").is_some() {
                            tracing::debug!("ignoring pushed event (bridge resyncs via events.poll)");
                        }
                    }
                    Err(e) => {
                        tracing::warn!("JustSlide connection closed: {e:#}");
                        break;
                    }
                }
            }
            closed.store(true, Ordering::SeqCst);
            pending.lock().await.clear();
        });
        Ok(conn)
    }

    pub fn is_closed(&self) -> bool {
        self.closed.load(Ordering::SeqCst)
    }

    pub fn next_request_id(&self, prefix: &str) -> String {
        format!("{prefix}-{}", self.next.fetch_add(1, Ordering::SeqCst))
    }

    pub async fn call(&self, id: &str, command: &str, args: Value, cancel: tokio_util_like::Cancel) -> Result<CallOutcome> {
        let reg = Registry::get();
        let _permit = self.inflight.acquire().await?;
        let (tx, rx) = oneshot::channel();
        self.pending.lock().await.insert(id.to_string(), tx);
        let req = json!({"v": 1, "id": id, "command": command, "args": args});
        write_frame(&mut *self.writer.lock().await, &req, reg.max_frame_bytes()).await?;
        let timeout = Duration::from_millis(reg.request_timeout_ms());
        let frame = tokio::select! {
            r = rx => r.map_err(|_| anyhow!("connection to JustSlide closed"))?,
            _ = cancel.cancelled() => {
                self.pending.lock().await.remove(id);
                let _ = write_frame(&mut *self.writer.lock().await, &json!({"v": 1, "cancel": id}), reg.max_frame_bytes()).await;
                bail!("cancelled")
            }
            _ = tokio::time::sleep(timeout) => {
                self.pending.lock().await.remove(id);
                let _ = write_frame(&mut *self.writer.lock().await, &json!({"v": 1, "cancel": id}), reg.max_frame_bytes()).await;
                bail!("timed out after {} ms", timeout.as_millis())
            }
        };
        if frame.get("ok") == Some(&Value::Bool(true)) {
            Ok(CallOutcome::Ok { result: frame.get("result").cloned().unwrap_or(Value::Null), revision: frame["revision"].as_i64().unwrap_or(0) })
        } else {
            let err: RemoteError = serde_json::from_value(frame.get("error").cloned().unwrap_or(Value::Null))
                .unwrap_or(RemoteError { code: "internal".into(), message: "malformed error".into(), retryable: false, pending_request_id: None });
            Ok(CallOutcome::Err(err))
        }
    }
}

/// Tiny cancellation adapter so this module does not depend on rmcp types.
pub mod tokio_util_like {
    use std::future::Future;
    use std::pin::Pin;
    pub struct Cancel(pub Pin<Box<dyn Future<Output = ()> + Send>>);
    impl Cancel {
        pub fn never() -> Self { Cancel(Box::pin(std::future::pending())) }
        pub async fn cancelled(self) { self.0.await }
    }
}
