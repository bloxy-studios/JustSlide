//! The JustSlide command handler. The trusted chrome (human) and MCP agents
//! call the *same* `Core::execute`; permissions are enforced here, not in UI
//! affordances. Invariant: an agent command never changes the human window's
//! selected workspace/tab and never asks the engine to show or focus a view.

use crate::engine::{BrowserEngine, DetachedEngine, EngineCommand, EngineEvent, Support};
use crate::error::{CoreError, CoreResult, ErrorCode};
use crate::events::{EventLog, StoredEvent};
use crate::ids::{new_id, new_secret, now_ms};
use crate::model::*;
use crate::state::State;
use crate::store::Store;
use crate::urls;
use justslide_protocol::{CommandSpec, Exposure, Registry, validate};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::oneshot;

pub const PROTOCOL_VERSION: &str = "1.1.0";
const IDEMPOTENCY_TTL: Duration = Duration::from_secs(600);
const IDEMPOTENCY_CAP: usize = 1024;
const MAX_CAPTURES_IN_FLIGHT_PER_AGENT: u32 = 2;
const WORKING_WINDOW_MS: i64 = 15_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Actor {
    /// The person, via the trusted browser chrome.
    Human,
    /// An authenticated agent. Identity comes from the pairing token resolved
    /// server-side, never from caller-supplied fields.
    Agent { grant_id: String, client: String },
}

impl Actor {
    fn key(&self) -> String {
        match self {
            Actor::Human => "human".into(),
            Actor::Agent { grant_id, .. } => format!("agent:{grant_id}"),
        }
    }
    fn is_agent(&self) -> bool { matches!(self, Actor::Agent { .. }) }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Precondition {
    pub revision: Option<i64>,
    pub navigation_id: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Request {
    pub id: String,
    pub command: String,
    pub args: Value,
    pub precondition: Option<Precondition>,
}

impl Request {
    pub fn new(id: impl Into<String>, command: impl Into<String>, args: Value) -> Self {
        Request { id: id.into(), command: command.into(), args, precondition: None }
    }
}

#[derive(Debug, Clone)]
pub struct Response {
    pub id: String,
    pub outcome: Result<(Value, i64), CoreError>,
}

impl Response {
    pub fn to_wire(&self) -> Value {
        match &self.outcome {
            Ok((result, revision)) => json!({"v": 1, "id": self.id, "ok": true, "result": result, "revision": revision}),
            Err(e) => json!({"v": 1, "id": self.id, "ok": false, "error": e}),
        }
    }
    pub fn result(&self) -> CoreResult<&Value> {
        self.outcome.as_ref().map(|(v, _)| v).map_err(Clone::clone)
    }
}

struct Inner {
    state: State,
    store: Store,
    events: EventLog,
}

enum Pending {
    Capture(oneshot::Sender<EngineEvent>),
}

pub struct Core {
    inner: Mutex<Inner>,
    engine: RwLock<Arc<dyn BrowserEngine>>,
    pending: Mutex<HashMap<String, Pending>>,
    idem: Mutex<HashMap<(String, String), (Instant, Response)>>,
}

impl Core {
    /// Opens (or creates) the app database under `data_dir`, or an in-memory
    /// store when `None` (tests, design fixture).
    pub fn open(data_dir: Option<&Path>) -> anyhow::Result<Arc<Core>> {
        let store = match data_dir {
            Some(d) => {
                std::fs::create_dir_all(d)?;
                Store::open(&d.join("justslide.sqlite3"))?
            }
            None => Store::in_memory()?,
        };
        let loaded = store.load()?;
        let mut state = if loaded.workspaces.is_empty() {
            State::seeded()
        } else {
            let mut s = State { instance_id: loaded.instance_seed.clone().unwrap_or_else(|| new_id("inst")), ..Default::default() };
            s.profiles = loaded.profiles.into_iter().map(|p| (p.id.clone(), p)).collect();
            s.workspaces = loaded.workspaces;
            s.tabs = loaded.tabs;
            s.favorites = loaded.favorites;
            s.settings = loaded.settings.unwrap_or_default();
            s.window = loaded.window.unwrap_or_default();
            s
        };
        state.grants = loaded.grants;
        // Recovery: repair a window that points at missing workspaces/tabs.
        if state.workspace(&state.window.active_workspace_id).is_none() {
            state.window.active_workspace_id = state.personal_workspace_id().map(str::to_string).unwrap_or_default();
        }
        if state.window.id.is_empty() {
            state.window.id = new_id("win");
        }
        let tab_ids: HashSet<String> = state.tabs.iter().map(|t| t.id.clone()).collect();
        state.window.active_tab_by_workspace.retain(|_, t| tab_ids.contains(t));
        state.revision = 1;
        let mut inner = Inner { state, store, events: EventLog::new(Registry::get().limit("maxQueuedEventsPerConnection", 1024) as usize) };
        inner.store.save(&inner.state)?;
        Ok(Arc::new(Core {
            inner: Mutex::new(inner),
            engine: RwLock::new(Arc::new(DetachedEngine)),
            pending: Mutex::new(HashMap::new()),
            idem: Mutex::new(HashMap::new()),
        }))
    }

    pub fn set_engine(&self, engine: Arc<dyn BrowserEngine>) {
        *self.engine.write().unwrap() = engine;
    }

    fn engine(&self) -> Arc<dyn BrowserEngine> { self.engine.read().unwrap().clone() }

    pub fn instance_id(&self) -> String { self.inner.lock().unwrap().state.instance_id.clone() }

    pub fn revision(&self) -> i64 { self.inner.lock().unwrap().state.revision }

    pub fn window(&self) -> WindowState { self.inner.lock().unwrap().state.window.clone() }

    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<StoredEvent> {
        self.inner.lock().unwrap().events.tx.subscribe()
    }

    /// Resolves a pairing token to an agent actor. Constant-time over the
    /// hash comparison; unknown, revoked and expired tokens all fail alike.
    pub fn authenticate(&self, token: &str) -> Option<Actor> {
        if token.len() < 43 || token.len() > 128 {
            return None;
        }
        let h = hash_token(token);
        let g = self.inner.lock().unwrap();
        let now = now_ms();
        g.state.grants.iter().find(|x| ct_eq(x.token_hash.as_bytes(), h.as_bytes()) && x.is_live(now)).map(|x| Actor::Agent { grant_id: x.id.clone(), client: x.client.clone() })
    }

    /// Commands an actor may call, for discovery (`welcome.commands`).
    pub fn commands_for(&self, actor: &Actor) -> Vec<String> {
        let g = self.inner.lock().unwrap();
        Registry::get()
            .commands
            .iter()
            .filter(|c| match actor {
                Actor::Human => c.exposed_to(Exposure::Ui),
                Actor::Agent { grant_id, .. } => {
                    c.exposed_to(Exposure::Mcp)
                        && g.state.grant(grant_id).is_some_and(|gr| c.operation.as_deref().is_none_or(|op| gr.operations.iter().any(|o| o == op)))
                }
            })
            .map(|c| c.name.clone())
            .collect()
    }

    pub fn grant_for(&self, actor: &Actor) -> Option<Grant> {
        match actor {
            Actor::Agent { grant_id, .. } => self.inner.lock().unwrap().state.grant(grant_id).cloned(),
            Actor::Human => None,
        }
    }

    pub async fn execute(&self, actor: &Actor, req: Request) -> Response {
        let id = req.id.clone();
        let reg = Registry::get();
        let Some(spec) = reg.by_command(&req.command) else {
            return Response { id, outcome: Err(CoreError::new(ErrorCode::UnknownCommand, format!("unknown command {}", req.command))) };
        };
        let exposure = if actor.is_agent() { Exposure::Mcp } else { Exposure::Ui };
        if !spec.exposed_to(exposure) {
            return Response { id, outcome: Err(CoreError::new(ErrorCode::UnknownCommand, format!("unknown command {}", req.command))) };
        }
        let args = if req.args.is_null() { json!({}) } else { req.args.clone() };
        if let Err(e) = validate(&spec.arguments, &args) {
            return Response { id, outcome: Err(CoreError::invalid(e)) };
        }
        let idem_key = (actor.key(), req.id.clone());
        if spec.idempotency == "request-id" || spec.idempotency == "safe" {
            if let Some((at, cached)) = self.idem.lock().unwrap().get(&idem_key) {
                if at.elapsed() < IDEMPOTENCY_TTL && spec.idempotency == "request-id" {
                    return cached.clone();
                }
            }
        }
        let before = self.window();
        let outcome = match spec.name.as_str() {
            "page.screenshot" | "page.snapshot" => self.capture(actor, spec, &args, req.precondition.as_ref()).await,
            _ => self.run_sync(actor, spec, &args, req.precondition.as_ref()),
        };
        if actor.is_agent() {
            let after = self.window();
            if before != after {
                // Defence in depth: this would be a core bug, never expected.
                tracing::error!(command = %spec.name, "agent command changed the human window state");
                debug_assert_eq!(before, after, "agent command {} changed the human window", spec.name);
            }
            self.record_activity(actor, spec, &args, &outcome);
        }
        let resp = Response { id, outcome };
        if spec.idempotency == "request-id" && resp.outcome.is_ok() {
            let mut m = self.idem.lock().unwrap();
            if m.len() >= IDEMPOTENCY_CAP {
                m.retain(|_, (at, _)| at.elapsed() < IDEMPOTENCY_TTL);
                if m.len() >= IDEMPOTENCY_CAP {
                    m.clear();
                }
            }
            m.insert(idem_key, (Instant::now(), resp.clone()));
        }
        resp
    }

    fn authorize(state: &State, actor: &Actor, spec: &CommandSpec) -> CoreResult<()> {
        let Actor::Agent { grant_id, .. } = actor else { return Ok(()) };
        let g = state.grant(grant_id).filter(|g| g.is_live(now_ms())).ok_or_else(|| CoreError::new(ErrorCode::Unauthenticated, "grant is revoked or expired"))?;
        if let Some(op) = spec.operation.as_deref() {
            if op == "human" || !g.operations.iter().any(|o| o == op) {
                return Err(CoreError::denied(format!("grant does not include {op}")));
            }
            if g.paused && op != "read.metadata" {
                return Err(CoreError::needs_user("agent is paused by the person in JustSlide", None));
            }
        }
        Ok(())
    }

    fn run_sync(&self, actor: &Actor, spec: &CommandSpec, args: &Value, pre: Option<&Precondition>) -> CoreResult<(Value, i64)> {
        let mut cmds = Vec::new();
        let out = {
            let mut g = self.inner.lock().unwrap();
            Self::authorize(&g.state, actor, spec)?;
            if let Some(rev) = pre.and_then(|p| p.revision) {
                if rev != g.state.revision {
                    return Err(CoreError::new(ErrorCode::PreconditionFailed, format!("revision is {} not {rev}", g.state.revision)));
                }
            }
            let mut tx = Tx { inner: &mut g, actor, cmds: &mut cmds, changed: false, grant_changed: None };
            let r = tx.dispatch(spec, args, pre);
            let changed = tx.changed;
            let grant_changed = tx.grant_changed.take();
            match r {
                Ok(v) => {
                    if changed {
                        g.state.revision += 1;
                        let Inner { state, store, .. } = &mut *g;
                        store.save(state)?;
                    }
                    if let Some(gid) = grant_changed {
                        let Inner { state, store, .. } = &mut *g;
                        if let Some(gr) = state.grant(&gid) {
                            store.save_grant(gr)?;
                        }
                    }
                    Ok((v, g.state.revision))
                }
                Err(e) => Err(e),
            }
        };
        if out.is_ok() {
            let engine = self.engine();
            for c in cmds {
                engine.send(c);
            }
        }
        out
    }

    async fn capture(&self, actor: &Actor, spec: &CommandSpec, args: &Value, pre: Option<&Precondition>) -> CoreResult<(Value, i64)> {
        let tab_id = args["tabId"].as_str().unwrap_or_default().to_string();
        let request_id = new_id("cap");
        let is_shot = spec.name == "page.screenshot";
        let (nav_id, cmd) = {
            let mut g = self.inner.lock().unwrap();
            Self::authorize(&g.state, actor, spec)?;
            let tab = visible_tab(&g.state, actor, &tab_id)?.clone();
            let caps = self.engine().capabilities();
            let feat = if is_shot { &caps.screenshots } else { &caps.page_snapshot };
            if matches!(feat.support, Support::Unsupported | Support::Unknown) {
                return Err(CoreError::unsupported(format!("{} is not available: {}", spec.name, feat.note)));
            }
            if tab.load_state == LoadState::NotLoaded {
                return Err(CoreError::new(ErrorCode::Unavailable, "tab has no live page yet (not loaded); retry after it loads"));
            }
            if let Some(n) = pre.and_then(|p| p.navigation_id.as_ref()) {
                if n != &tab.navigation_id {
                    return Err(CoreError::new(ErrorCode::PreconditionFailed, "tab navigated since navigationId"));
                }
            }
            if let Actor::Agent { grant_id, .. } = actor {
                let rt = g.state.agent_runtime.entry(grant_id.clone()).or_default();
                if rt.in_flight >= MAX_CAPTURES_IN_FLIGHT_PER_AGENT {
                    return Err(CoreError::new(ErrorCode::RateLimited, "too many captures in flight for this agent"));
                }
                rt.in_flight += 1;
            }
            let cmd = if is_shot {
                let edge = args["maxEdgePx"].as_u64().unwrap_or(1280).min(Registry::get().limit("maxScreenshotEdgePx", 2560)) as u32;
                EngineCommand::Screenshot { request_id: request_id.clone(), tab_id: tab_id.clone(), max_edge_px: edge, format: args["format"].as_str().unwrap_or("png").into() }
            } else {
                let n = args["maxNodes"].as_u64().unwrap_or(500).min(Registry::get().limit("maxSnapshotNodes", 2000)) as u32;
                EngineCommand::Snapshot { request_id: request_id.clone(), tab_id: tab_id.clone(), max_nodes: n }
            };
            let _ = &mut g;
            (tab.navigation_id.clone(), cmd)
        };
        let (txs, rx) = oneshot::channel();
        self.pending.lock().unwrap().insert(request_id.clone(), Pending::Capture(txs));
        self.engine().send(cmd);
        let timeout = Duration::from_millis(Registry::get().request_timeout_ms());
        let res = tokio::time::timeout(timeout, rx).await;
        self.pending.lock().unwrap().remove(&request_id);
        if let Actor::Agent { grant_id, .. } = actor {
            if let Some(rt) = self.inner.lock().unwrap().state.agent_runtime.get_mut(grant_id) {
                rt.in_flight = rt.in_flight.saturating_sub(1);
            }
        }
        let ev = match res {
            Ok(Ok(ev)) => ev,
            Ok(Err(_)) => return Err(CoreError::new(ErrorCode::Unavailable, "engine dropped the capture request")),
            Err(_) => return Err(CoreError::new(ErrorCode::Timeout, "capture timed out")),
        };
        let current_nav = self.inner.lock().unwrap().state.tab(&tab_id).map(|t| t.navigation_id.clone());
        let rev = self.revision();
        match ev {
            EngineEvent::ScreenshotResult { navigation_id, capture, error, .. } => {
                if let Some(e) = error {
                    return Err(CoreError::new(ErrorCode::Unavailable, e));
                }
                let c = capture.ok_or_else(|| CoreError::internal("engine returned neither capture nor error"))?;
                if current_nav.as_deref() != Some(navigation_id.as_str()) || navigation_id != nav_id {
                    return Err(CoreError::new(ErrorCode::Conflict, "tab navigated during capture"));
                }
                Ok((json!({"tabId": tab_id, "mimeType": c.mime_type, "dataBase64": c.data_base64, "width": c.width, "height": c.height, "navigationId": navigation_id, "possiblyStale": c.possibly_stale}), rev))
            }
            EngineEvent::SnapshotResult { navigation_id, url, nodes, truncated, error, .. } => {
                if let Some(e) = error {
                    return Err(CoreError::new(ErrorCode::Unavailable, e));
                }
                if current_nav.as_deref() != Some(navigation_id.as_str()) {
                    return Err(CoreError::new(ErrorCode::Conflict, "tab navigated during snapshot"));
                }
                Ok((json!({"tabId": tab_id, "url": url, "navigationId": navigation_id, "truncated": truncated, "nodes": nodes.unwrap_or_default(), "untrustedContent": true}), rev))
            }
            _ => Err(CoreError::internal("unexpected engine event for capture")),
        }
    }

    fn record_activity(&self, actor: &Actor, spec: &CommandSpec, args: &Value, outcome: &Result<(Value, i64), CoreError>) {
        let Actor::Agent { grant_id, client } = actor else { return };
        let mut g = self.inner.lock().unwrap();
        let now = now_ms();
        let tab_id = args["tabId"].as_str().map(str::to_string).or_else(|| outcome.as_ref().ok().and_then(|(v, _)| v["tab"]["id"].as_str().map(str::to_string)));
        let workspace_id = args["workspaceId"].as_str().map(str::to_string).or_else(|| tab_id.as_ref().and_then(|t| g.state.tab(t).map(|t| t.workspace_id.clone())));
        let rec = ActivityRecord {
            at: now, grant_id: grant_id.clone(), client: client.clone(), command: spec.name.clone(),
            outcome: match outcome { Ok(_) => "ok".into(), Err(e) => serde_json::to_value(e.code).ok().and_then(|v| v.as_str().map(str::to_string)).unwrap_or_default() },
            workspace_id, tab_id, url: args["url"].as_str().map(urls::redact),
        };
        let rt = g.state.agent_runtime.entry(grant_id.clone()).or_default();
        let was_working = now - rt.last_command_ms < WORKING_WINDOW_MS;
        rt.last_command_ms = now;
        let _ = g.store.append_activity(&rec);
        if !was_working {
            g.events.push("activity.changed", json!({"grantId": grant_id, "working": true}), None, true);
        }
    }

    /// Entry point for real WebKit callbacks from the platform adapter.
    pub fn on_engine_event(&self, ev: EngineEvent) {
        if let EngineEvent::ScreenshotResult { request_id, .. } | EngineEvent::SnapshotResult { request_id, .. } = &ev {
            if let Some(Pending::Capture(tx)) = self.pending.lock().unwrap().remove(request_id) {
                let _ = tx.send(ev);
            }
            return;
        }
        let mut cmds = Vec::new();
        {
            let mut g = self.inner.lock().unwrap();
            let mut tx = Tx { inner: &mut g, actor: &Actor::Human, cmds: &mut cmds, changed: false, grant_changed: None };
            let persist = tx.apply_engine_event(ev);
            if tx.changed {
                g.state.revision += 1;
                if persist {
                    let Inner { state, store, .. } = &mut *g;
                    let _ = store.save(state);
                }
            }
        }
        let engine = self.engine();
        for c in cmds {
            engine.send(c);
        }
    }

    /// Replays persisted session state into a freshly attached adapter:
    /// only the human-active tab gets a live view; others stay `notLoaded`.
    pub fn restore_session(&self) {
        self.run_human_internal(|tx| {
            let ws = tx.inner.state.window.active_workspace_id.clone();
            let tab = tx.inner.state.window.active_tab_by_workspace.get(&ws).cloned();
            if let Some(t) = tab {
                tx.ensure_view(&t);
                tx.cmds.push(EngineCommand::Show { tab_id: Some(t), focus: false });
            } else {
                tx.cmds.push(EngineCommand::Show { tab_id: None, focus: false });
            }
        });
    }

    fn run_human_internal(&self, f: impl FnOnce(&mut Tx)) {
        let mut cmds = Vec::new();
        {
            let mut g = self.inner.lock().unwrap();
            let mut tx = Tx { inner: &mut g, actor: &Actor::Human, cmds: &mut cmds, changed: false, grant_changed: None };
            f(&mut tx);
            if tx.changed {
                g.state.revision += 1;
            }
        }
        let engine = self.engine();
        for c in cmds {
            engine.send(c);
        }
    }

    #[cfg(any(test, feature = "test-engine"))]
    pub fn debug_state(&self) -> State { self.inner.lock().unwrap().state.clone() }

    /// Development-only: replace state with the deterministic design fixture.
    #[cfg(any(test, feature = "test-engine", feature = "design-fixture"))]
    pub fn load_fixture(&self, fixture: &Value) -> anyhow::Result<()> {
        let mut g = self.inner.lock().unwrap();
        crate::fixture::apply(&mut g.state, fixture)?;
        g.state.revision += 1;
        Ok(())
    }
}

fn hash_token(t: &str) -> String {
    let d = Sha256::digest(t.as_bytes());
    d.iter().map(|b| format!("{b:02x}")).collect()
}

fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

fn agent_scope(state: &State, grant_id: &str) -> HashSet<String> {
    let mut s: HashSet<String> = state.workspaces.iter().filter(|w| w.owner_grant_id.as_deref() == Some(grant_id)).map(|w| w.id.clone()).collect();
    if let Some(g) = state.grant(grant_id) {
        s.extend(g.workspace_ids.iter().filter(|id| state.workspace(id).is_some()).cloned());
    }
    s
}

fn in_scope(state: &State, actor: &Actor, ws: &str) -> bool {
    match actor {
        Actor::Human => state.workspace(ws).is_some(),
        Actor::Agent { grant_id, .. } => agent_scope(state, grant_id).contains(ws),
    }
}

/// Out-of-scope resources are reported as not found so agents cannot probe
/// for the existence of the person's workspaces or tabs.
fn visible_tab<'a>(state: &'a State, actor: &Actor, tab_id: &str) -> CoreResult<&'a Tab> {
    state.tab(tab_id).filter(|t| in_scope(state, actor, &t.workspace_id)).ok_or_else(|| CoreError::not_found("no such tab"))
}

fn visible_workspace<'a>(state: &'a State, actor: &Actor, ws: &str) -> CoreResult<&'a Workspace> {
    state.workspace(ws).filter(|w| in_scope(state, actor, &w.id)).ok_or_else(|| CoreError::not_found("no such workspace"))
}

pub fn tab_view(state: &State, t: &Tab) -> Value {
    json!({
        "id": t.id, "workspaceId": t.workspace_id, "url": t.url, "title": t.title, "pinned": t.pinned, "order": t.order,
        "loading": t.loading(), "loadState": t.load_state, "active": state.is_active_tab(&t.id), "openedBy": t.opened_by.as_str(),
        "navigationId": t.navigation_id, "crashed": t.load_state == LoadState::Crashed, "audible": t.audible, "faviconUrl": t.favicon_url,
        "canGoBack": t.can_go_back, "canGoForward": t.can_go_forward, "secure": t.secure, "error": t.error,
    })
}

pub fn workspace_view(w: &Workspace) -> Value {
    json!({"id": w.id, "name": w.name, "icon": w.icon, "profileKind": w.profile_kind.as_str(), "agentOwned": w.agent_owned, "order": w.order, "pinnedCollapsed": w.pinned_collapsed})
}

struct Tx<'a> {
    inner: &'a mut Inner,
    actor: &'a Actor,
    cmds: &'a mut Vec<EngineCommand>,
    changed: bool,
    grant_changed: Option<String>,
}

impl Tx<'_> {
    fn emit(&mut self, event: &str, data: Value, ws: Option<&str>, ui_only: bool) {
        self.inner.events.push(event, data, ws.map(str::to_string), ui_only);
    }

    fn human_only(&self) -> CoreResult<()> {
        if self.actor.is_agent() { Err(CoreError::denied("only the person can do this")) } else { Ok(()) }
    }

    fn grant_id(&self) -> Option<&str> {
        match self.actor { Actor::Agent { grant_id, .. } => Some(grant_id), Actor::Human => None }
    }

    fn arg_str<'v>(args: &'v Value, k: &str) -> &'v str { args[k].as_str().unwrap_or_default() }

    /// Ensure a live WebKit view exists for a tab (lazy restore).
    fn ensure_view(&mut self, tab_id: &str) {
        let st = &mut self.inner.state;
        let Some(t) = st.tab(tab_id).cloned() else { return };
        if t.load_state != LoadState::NotLoaded {
            return;
        }
        let Some(p) = st.profile_of_workspace(&t.workspace_id).cloned() else { return };
        let nav = new_id("nav");
        if let Some(tm) = st.tab_mut(tab_id) {
            tm.load_state = LoadState::Loading;
            tm.navigation_id = nav.clone();
        }
        self.changed = true;
        self.cmds.push(EngineCommand::CreateView { tab_id: t.id, profile_id: p.id, data_store_id: p.data_store_id, private: p.kind == ProfileKind::Private, url: t.url, navigation_id: nav });
    }

    fn show_active(&mut self, focus: bool) {
        let ws = self.inner.state.window.active_workspace_id.clone();
        let tab = self.inner.state.window.active_tab_by_workspace.get(&ws).cloned().or_else(|| {
            let mut ts: Vec<&Tab> = self.inner.state.tabs_in(&ws).collect();
            ts.sort_by_key(|t| (!t.pinned, t.order));
            ts.first().map(|t| t.id.clone())
        });
        if let Some(t) = &tab {
            self.inner.state.window.active_tab_by_workspace.insert(ws, t.clone());
            self.ensure_view(t);
        }
        self.cmds.push(EngineCommand::Show { tab_id: tab, focus });
    }

    fn create_tab(&mut self, ws: &str, url: &str, pinned: bool, by: OpenedBy) -> Tab {
        let grant = self.grant_id().map(str::to_string);
        let t = self.inner.state.new_tab(ws, url, pinned, by, grant);
        self.inner.state.tabs.push(t.clone());
        self.changed = true;
        self.ensure_view(&t.id);
        let t = self.inner.state.tab(&t.id).cloned().unwrap();
        let view = tab_view(&self.inner.state, &t);
        self.emit("tab.created", json!({"tab": view}), Some(ws), false);
        t
    }

    fn agent_rate_limit_tabs(&mut self) -> CoreResult<()> {
        let Some(gid) = self.grant_id().map(str::to_string) else { return Ok(()) };
        let limit = Registry::get().limit("maxTabsCreatedPerMinute", 30) as usize;
        let now = now_ms();
        let rt = self.inner.state.agent_runtime.entry(gid).or_default();
        rt.tab_creations.retain(|t| now - *t < 60_000);
        if rt.tab_creations.len() >= limit {
            return Err(CoreError::new(ErrorCode::RateLimited, "tab creation limit reached; retry later"));
        }
        rt.tab_creations.push(now);
        Ok(())
    }

    fn deny_if_human_viewing(&self, tab_id: &str) -> CoreResult<()> {
        if self.actor.is_agent() && self.inner.state.is_active_tab(tab_id) {
            return Err(CoreError::needs_user("the person is viewing this tab; agents cannot change it", None));
        }
        Ok(())
    }

    fn dispatch(&mut self, spec: &CommandSpec, a: &Value, pre: Option<&Precondition>) -> CoreResult<Value> {
        let st_actor = self.actor.clone();
        match spec.name.as_str() {
            "capabilities.get" => {
                let grant = match &st_actor {
                    Actor::Agent { grant_id, .. } => {
                        let g = self.inner.state.grant(grant_id).cloned();
                        let scope: Vec<String> = agent_scope(&self.inner.state, grant_id).into_iter().collect();
                        json!({"grantId": grant_id, "workspaces": scope, "operations": g.as_ref().map(|g| g.operations.clone()), "paused": g.map(|g| g.paused)})
                    }
                    Actor::Human => json!({"grantId": "human"}),
                };
                let cmds: Vec<String> = Registry::get()
                    .commands
                    .iter()
                    .filter(|c| c.exposed_to(if st_actor.is_agent() { Exposure::Mcp } else { Exposure::Ui }))
                    .filter(|c| match &st_actor {
                        Actor::Agent { grant_id, .. } => c.operation.as_deref().is_none_or(|op| self.inner.state.grant(grant_id).is_some_and(|g| g.operations.iter().any(|o| o == op))),
                        Actor::Human => true,
                    })
                    .map(|c| c.name.clone())
                    .collect();
                Ok(json!({"protocolVersion": PROTOCOL_VERSION, "instanceId": self.inner.state.instance_id, "commands": cmds, "grant": grant}))
            }
            "state.snapshot" => {
                self.human_only()?;
                Ok(self.snapshot())
            }
            "workspaces.list" => {
                let st = &self.inner.state;
                let ws: Vec<Value> = st.workspaces.iter().filter(|w| in_scope(st, &st_actor, &w.id)).map(workspace_view).collect();
                Ok(json!({"workspaces": ws}))
            }
            "workspaces.create" => {
                let name = Self::arg_str(a, "name").trim().to_string();
                if name.is_empty() {
                    return Err(CoreError::invalid("name is empty"));
                }
                let icon = a["icon"].as_str().unwrap_or(if st_actor.is_agent() { "agent" } else { "generic" }).to_string();
                let st = &mut self.inner.state;
                let (profile_id, kind, owner) = match &st_actor {
                    Actor::Agent { grant_id, client } => {
                        // One isolated WebKit data store per agent grant.
                        let existing = st.workspaces.iter().find(|w| w.owner_grant_id.as_deref() == Some(grant_id)).map(|w| w.profile_id.clone());
                        let pid = existing.unwrap_or_else(|| st.new_profile(ProfileKind::AgentIsolated, &format!("{client} (agent)")));
                        (pid, ProfileKind::AgentIsolated, Some(grant_id.clone()))
                    }
                    Actor::Human => {
                        let pid = st.workspaces.iter().find(|w| w.profile_kind == ProfileKind::Personal).map(|w| w.profile_id.clone()).unwrap_or_else(|| st.new_profile(ProfileKind::Personal, "Personal"));
                        (pid, ProfileKind::Personal, None)
                    }
                };
                let w = Workspace {
                    id: new_id("ws"), name, icon, profile_id, profile_kind: kind, agent_owned: owner.is_some(), owner_grant_id: owner,
                    order: st.workspaces.iter().map(|w| w.order + 1).max().unwrap_or(0), pinned_collapsed: false,
                };
                st.workspaces.push(w.clone());
                self.changed = true;
                let v = workspace_view(&w);
                self.emit("workspace.created", json!({"workspace": v}), Some(&w.id), false);
                Ok(json!({"workspace": v}))
            }
            "workspaces.switch" => {
                self.human_only()?;
                let ws = Self::arg_str(a, "workspaceId").to_string();
                visible_workspace(&self.inner.state, &st_actor, &ws)?;
                if self.inner.state.window.active_workspace_id != ws {
                    self.inner.state.window.active_workspace_id = ws.clone();
                    self.changed = true;
                }
                self.show_active(false);
                self.emit("window.changed", json!({"window": self.inner.state.window}), None, true);
                Ok(json!({"window": self.inner.state.window}))
            }
            "workspaces.rename" => {
                self.human_only()?;
                let ws = Self::arg_str(a, "workspaceId").to_string();
                let name = Self::arg_str(a, "name").trim().to_string();
                let w = self.inner.state.workspace_mut(&ws).ok_or_else(|| CoreError::not_found("no such workspace"))?;
                w.name = name;
                let v = workspace_view(w);
                self.changed = true;
                self.emit("workspace.updated", json!({"workspace": v}), Some(&ws), false);
                Ok(json!({"workspace": v}))
            }
            "workspaces.reorder" => {
                self.human_only()?;
                let ids: Vec<String> = a["workspaceIds"].as_array().unwrap().iter().filter_map(|v| v.as_str().map(str::to_string)).collect();
                for (i, id) in ids.iter().enumerate() {
                    let w = self.inner.state.workspace_mut(id).ok_or_else(|| CoreError::not_found("no such workspace"))?;
                    w.order = i as i64;
                }
                self.inner.state.workspaces.sort_by_key(|w| w.order);
                self.changed = true;
                self.emit("workspace.updated", json!({"order": ids}), None, true);
                Ok(json!({"ok": true}))
            }
            "workspaces.delete" => {
                self.human_only()?;
                let ws = Self::arg_str(a, "workspaceId").to_string();
                let w = self.inner.state.workspace(&ws).cloned().ok_or_else(|| CoreError::not_found("no such workspace"))?;
                let personal_count = self.inner.state.workspaces.iter().filter(|x| x.profile_kind == ProfileKind::Personal).count();
                if w.profile_kind == ProfileKind::Personal && personal_count <= 1 {
                    return Err(CoreError::new(ErrorCode::Conflict, "the last personal workspace cannot be deleted"));
                }
                let tabs: Vec<String> = self.inner.state.tabs_in(&ws).map(|t| t.id.clone()).collect();
                for t in &tabs {
                    self.cmds.push(EngineCommand::DestroyView { tab_id: t.clone() });
                }
                self.inner.state.tabs.retain(|t| t.workspace_id != ws);
                self.inner.state.workspaces.retain(|x| x.id != ws);
                self.inner.state.window.active_tab_by_workspace.remove(&ws);
                if self.inner.state.window.active_workspace_id == ws {
                    self.inner.state.window.active_workspace_id = self.inner.state.personal_workspace_id().unwrap_or_default().to_string();
                    self.show_active(false);
                }
                self.changed = true;
                self.emit("workspace.deleted", json!({"workspaceId": ws}), Some(&ws), false);
                Ok(json!({"ok": true}))
            }
            "tabs.list" => {
                let st = &self.inner.state;
                let filter_ws = a["workspaceId"].as_str();
                if let Some(ws) = filter_ws {
                    visible_workspace(st, &st_actor, ws)?;
                }
                let mut tabs: Vec<&Tab> = st.tabs.iter().filter(|t| in_scope(st, &st_actor, &t.workspace_id) && filter_ws.is_none_or(|w| w == t.workspace_id)).collect();
                tabs.sort_by_key(|t| (t.workspace_id.clone(), !t.pinned, t.order));
                Ok(json!({"tabs": tabs.into_iter().map(|t| tab_view(st, t)).collect::<Vec<_>>()}))
            }
            "tabs.create" => {
                let ws = Self::arg_str(a, "workspaceId").to_string();
                visible_workspace(&self.inner.state, &st_actor, &ws)?;
                let url = urls::require_http(Self::arg_str(a, "url"))?;
                self.agent_rate_limit_tabs()?;
                let by = if st_actor.is_agent() { OpenedBy::Agent } else { OpenedBy::User };
                let t = self.create_tab(&ws, &url, a["pinned"].as_bool().unwrap_or(false), by);
                Ok(json!({"tab": tab_view(&self.inner.state, &t)}))
            }
            "tabs.open" => {
                self.human_only()?;
                let url = urls::resolve_user_input(Self::arg_str(a, "input"), &self.inner.state.settings.search_template)?;
                let ws = a["workspaceId"].as_str().map(str::to_string).unwrap_or_else(|| self.inner.state.window.active_workspace_id.clone());
                visible_workspace(&self.inner.state, &st_actor, &ws)?;
                let t = self.create_tab(&ws, &url, a["pinned"].as_bool().unwrap_or(false), OpenedBy::User);
                self.inner.state.window.active_workspace_id = ws.clone();
                self.inner.state.window.active_tab_by_workspace.insert(ws, t.id.clone());
                self.cmds.push(EngineCommand::Show { tab_id: Some(t.id.clone()), focus: true });
                self.emit("window.changed", json!({"window": self.inner.state.window}), None, true);
                Ok(json!({"tab": tab_view(&self.inner.state, &t)}))
            }
            "tabs.activate" => {
                self.human_only()?;
                let tab = visible_tab(&self.inner.state, &st_actor, Self::arg_str(a, "tabId"))?.clone();
                let w = &mut self.inner.state.window;
                w.active_workspace_id = tab.workspace_id.clone();
                w.active_tab_by_workspace.insert(tab.workspace_id.clone(), tab.id.clone());
                self.changed = true;
                self.ensure_view(&tab.id);
                self.cmds.push(EngineCommand::Show { tab_id: Some(tab.id.clone()), focus: true });
                self.emit("window.changed", json!({"window": self.inner.state.window}), None, true);
                Ok(json!({"window": self.inner.state.window}))
            }
            "tabs.pin" => {
                let tab = visible_tab(&self.inner.state, &st_actor, Self::arg_str(a, "tabId"))?.clone();
                let pinned = a["pinned"].as_bool().unwrap_or(true);
                if tab.pinned != pinned {
                    let order = self.inner.state.next_tab_order(&tab.workspace_id, pinned);
                    let t = self.inner.state.tab_mut(&tab.id).unwrap();
                    t.pinned = pinned;
                    t.order = order;
                    self.inner.state.renumber(&tab.workspace_id);
                    self.changed = true;
                }
                let t = self.inner.state.tab(&tab.id).unwrap();
                let v = tab_view(&self.inner.state, t);
                self.emit("tab.updated", json!({"tab": v}), Some(&tab.workspace_id), false);
                Ok(json!({"tab": v}))
            }
            "tabs.close" => {
                let tab = visible_tab(&self.inner.state, &st_actor, Self::arg_str(a, "tabId"))?.clone();
                self.deny_if_human_viewing(&tab.id)?;
                let was_active = self.inner.state.is_active_tab(&tab.id);
                let siblings: Vec<String> = {
                    let mut ts: Vec<&Tab> = self.inner.state.tabs_in(&tab.workspace_id).collect();
                    ts.sort_by_key(|t| (!t.pinned, t.order));
                    ts.into_iter().map(|t| t.id.clone()).collect()
                };
                self.inner.state.tabs.retain(|t| t.id != tab.id);
                self.cmds.push(EngineCommand::DestroyView { tab_id: tab.id.clone() });
                if self.inner.state.tab_persists(&tab) || self.inner.state.workspace(&tab.workspace_id).is_some_and(|w| w.profile_kind.persists()) {
                    let _ = self.inner.store.push_closed(&ClosedTab { workspace_id: tab.workspace_id.clone(), url: tab.url.clone(), title: tab.title.clone(), pinned: tab.pinned, closed_at: now_ms() });
                }
                let w = &mut self.inner.state.window;
                if w.active_tab_by_workspace.get(&tab.workspace_id) == Some(&tab.id) {
                    w.active_tab_by_workspace.remove(&tab.workspace_id);
                    let idx = siblings.iter().position(|s| s == &tab.id).unwrap_or(0);
                    let next = siblings.get(idx + 1).or_else(|| idx.checked_sub(1).and_then(|i| siblings.get(i))).cloned();
                    if let Some(n) = next {
                        w.active_tab_by_workspace.insert(tab.workspace_id.clone(), n);
                    }
                }
                self.inner.state.renumber(&tab.workspace_id);
                self.changed = true;
                if was_active {
                    self.show_active(false);
                    self.emit("window.changed", json!({"window": self.inner.state.window}), None, true);
                }
                self.emit("tab.closed", json!({"tabId": tab.id, "workspaceId": tab.workspace_id}), Some(&tab.workspace_id), false);
                Ok(json!({"closed": true}))
            }
            "tabs.move" => {
                let tab = visible_tab(&self.inner.state, &st_actor, Self::arg_str(a, "tabId"))?.clone();
                let to = Self::arg_str(a, "workspaceId").to_string();
                let dest = visible_workspace(&self.inner.state, &st_actor, &to)?.clone();
                self.deny_if_human_viewing(&tab.id)?;
                let src_profile = self.inner.state.profile_of_workspace(&tab.workspace_id).map(|p| p.id.clone());
                let cross_profile = src_profile.as_deref() != Some(dest.profile_id.as_str());
                if cross_profile && st_actor.is_agent() {
                    return Err(CoreError::denied("agents cannot move tabs across profiles"));
                }
                let order = self.inner.state.next_tab_order(&to, tab.pinned);
                if let Some(t) = self.inner.state.tab_mut(&tab.id) {
                    t.workspace_id = to.clone();
                    t.order = order;
                }
                if cross_profile {
                    // A WKWebView cannot change data store; reload in the destination profile.
                    self.cmds.push(EngineCommand::DestroyView { tab_id: tab.id.clone() });
                    if let Some(t) = self.inner.state.tab_mut(&tab.id) { t.load_state = LoadState::NotLoaded; }
                }
                let w = &mut self.inner.state.window;
                if w.active_tab_by_workspace.get(&tab.workspace_id) == Some(&tab.id) {
                    w.active_tab_by_workspace.remove(&tab.workspace_id);
                }
                self.inner.state.renumber(&tab.workspace_id);
                self.changed = true;
                let v = tab_view(&self.inner.state, self.inner.state.tab(&tab.id).unwrap());
                self.emit("tab.updated", json!({"tab": v, "fromWorkspaceId": tab.workspace_id}), Some(&to), false);
                Ok(json!({"tab": v}))
            }
            "tabs.reorder" => {
                self.human_only()?;
                let tab = visible_tab(&self.inner.state, &st_actor, Self::arg_str(a, "tabId"))?.clone();
                let idx = a["index"].as_i64().unwrap_or(0);
                let mut ids: Vec<(i64, String)> = self.inner.state.tabs_in(&tab.workspace_id).filter(|t| t.pinned == tab.pinned && t.id != tab.id).map(|t| (t.order, t.id.clone())).collect();
                ids.sort();
                let pos = (idx as usize).min(ids.len());
                ids.insert(pos, (0, tab.id.clone()));
                for (i, (_, id)) in ids.iter().enumerate() {
                    if let Some(t) = self.inner.state.tab_mut(id) { t.order = i as i64; }
                }
                self.changed = true;
                let v = tab_view(&self.inner.state, self.inner.state.tab(&tab.id).unwrap());
                self.emit("tab.updated", json!({"tab": v}), Some(&tab.workspace_id), false);
                Ok(json!({"tab": v}))
            }
            "tabs.duplicate" => {
                self.human_only()?;
                let tab = visible_tab(&self.inner.state, &st_actor, Self::arg_str(a, "tabId"))?.clone();
                let t = self.create_tab(&tab.workspace_id, &tab.url, false, OpenedBy::User);
                Ok(json!({"tab": tab_view(&self.inner.state, &t)}))
            }
            "tabs.reopen" => {
                self.human_only()?;
                let ws = self.inner.state.window.active_workspace_id.clone();
                let closed = self.inner.store.pop_closed(&ws)?.ok_or_else(|| CoreError::not_found("no recently closed tab in this workspace"))?;
                let t = self.create_tab(&ws, &closed.url, closed.pinned, OpenedBy::User);
                self.inner.state.window.active_tab_by_workspace.insert(ws, t.id.clone());
                self.cmds.push(EngineCommand::Show { tab_id: Some(t.id.clone()), focus: true });
                self.emit("window.changed", json!({"window": self.inner.state.window}), None, true);
                Ok(json!({"tab": tab_view(&self.inner.state, &t)}))
            }
            "page.navigate" => {
                let tab = visible_tab(&self.inner.state, &st_actor, Self::arg_str(a, "tabId"))?.clone();
                self.deny_if_human_viewing(&tab.id)?;
                if let Some(n) = pre.and_then(|p| p.navigation_id.as_ref()) {
                    if n != &tab.navigation_id {
                        return Err(CoreError::new(ErrorCode::PreconditionFailed, "tab navigated since navigationId"));
                    }
                }
                let url = if st_actor.is_agent() {
                    urls::require_http(Self::arg_str(a, "url"))?
                } else {
                    urls::resolve_user_input(Self::arg_str(a, "url"), &self.inner.state.settings.search_template)?
                };
                if tab.load_state == LoadState::NotLoaded {
                    if let Some(t) = self.inner.state.tab_mut(&tab.id) { t.url = url.clone(); }
                    self.ensure_view(&tab.id);
                } else {
                    let nav = new_id("nav");
                    let t = self.inner.state.tab_mut(&tab.id).unwrap();
                    t.load_state = LoadState::Loading;
                    t.navigation_id = nav.clone();
                    t.error = None;
                    self.cmds.push(EngineCommand::Navigate { tab_id: tab.id.clone(), url, navigation_id: nav });
                }
                self.changed = true;
                let v = tab_view(&self.inner.state, self.inner.state.tab(&tab.id).unwrap());
                self.emit("tab.updated", json!({"tab": v}), Some(&tab.workspace_id), false);
                Ok(json!({"tab": v}))
            }
            "page.back" | "page.forward" | "page.reload" | "page.stop" => {
                let tab = visible_tab(&self.inner.state, &st_actor, Self::arg_str(a, "tabId"))?.clone();
                self.deny_if_human_viewing(&tab.id)?;
                if tab.load_state == LoadState::NotLoaded {
                    if spec.name == "page.reload" {
                        self.ensure_view(&tab.id);
                        return Ok(json!({"sent": true}));
                    }
                    return Err(CoreError::new(ErrorCode::PreconditionFailed, "tab is not loaded"));
                }
                if tab.load_state == LoadState::Crashed && spec.name == "page.reload" {
                    // Renderer recovery: recreate the view at the last committed URL.
                    self.cmds.push(EngineCommand::DestroyView { tab_id: tab.id.clone() });
                    if let Some(t) = self.inner.state.tab_mut(&tab.id) { t.load_state = LoadState::NotLoaded; }
                    self.ensure_view(&tab.id);
                    if self.inner.state.is_active_tab(&tab.id) {
                        self.cmds.push(EngineCommand::Show { tab_id: Some(tab.id.clone()), focus: false });
                    }
                    return Ok(json!({"sent": true, "recovered": true}));
                }
                let id = tab.id.clone();
                self.cmds.push(match spec.name.as_str() {
                    "page.back" => EngineCommand::GoBack { tab_id: id },
                    "page.forward" => EngineCommand::GoForward { tab_id: id },
                    "page.reload" => EngineCommand::Reload { tab_id: id },
                    _ => EngineCommand::Stop { tab_id: id },
                });
                Ok(json!({"sent": true}))
            }
            "favorites.list" => {
                if let Actor::Agent { grant_id, .. } = &st_actor {
                    let personal = self.inner.state.personal_workspace_id().map(str::to_string).unwrap_or_default();
                    if !agent_scope(&self.inner.state, grant_id).contains(&personal) {
                        return Err(CoreError::denied("favorites belong to the personal profile, which is not in this grant"));
                    }
                }
                Ok(json!({"favorites": self.inner.state.favorites}))
            }
            "favorites.add" => {
                let url = urls::require_http(Self::arg_str(a, "url"))?;
                if let Some(f) = self.inner.state.favorites.iter().find(|f| f.url == url) {
                    return Ok(json!({"favorite": f}));
                }
                let title = a["title"].as_str().map(str::to_string).unwrap_or_else(|| urls::host_of(&url));
                if let Actor::Agent { grant_id, client } = &st_actor {
                    // Favorites live in the person's sidebar: require approval.
                    let ap = Approval {
                        id: new_id("apr"), grant_id: grant_id.clone(), client: client.clone(), command: "favorites.add".into(),
                        summary: format!("Add “{title}” ({}) to Favorites", urls::host_of(&url)), args: json!({"url": url, "title": title}),
                        created_at: now_ms(), status: "pending".into(),
                    };
                    let id = ap.id.clone();
                    self.inner.state.approvals.push(ap.clone());
                    self.changed = true;
                    self.emit("approval.requested", json!({"approval": ap}), None, true);
                    return Err(CoreError::needs_user("waiting for the person to approve this favorite in JustSlide", Some(id)));
                }
                Ok(json!({"favorite": self.add_favorite(&url, &title)}))
            }
            "favorites.remove" => {
                self.human_only()?;
                let id = Self::arg_str(a, "favoriteId");
                let n = self.inner.state.favorites.len();
                self.inner.state.favorites.retain(|f| f.id != id);
                if n == self.inner.state.favorites.len() {
                    return Err(CoreError::not_found("no such favorite"));
                }
                for (i, f) in self.inner.state.favorites.iter_mut().enumerate() { f.order = i as i64; }
                self.changed = true;
                self.emit("favorite.removed", json!({"favoriteId": id}), None, true);
                Ok(json!({"ok": true}))
            }
            "favorites.reorder" => {
                self.human_only()?;
                let ids: Vec<&str> = a["favoriteIds"].as_array().unwrap().iter().filter_map(Value::as_str).collect();
                for (i, id) in ids.iter().enumerate() {
                    if let Some(f) = self.inner.state.favorites.iter_mut().find(|f| &f.id == id) { f.order = i as i64; }
                }
                self.inner.state.favorites.sort_by_key(|f| f.order);
                self.changed = true;
                self.emit("favorite.updated", json!({}), None, true);
                Ok(json!({"ok": true}))
            }
            "ui.setPinnedCollapsed" => {
                self.human_only()?;
                let ws = self.inner.state.window.active_workspace_id.clone();
                if let Some(w) = self.inner.state.workspace_mut(&ws) { w.pinned_collapsed = a["collapsed"].as_bool().unwrap_or(false); }
                self.changed = true;
                self.emit("window.changed", json!({"workspaceId": ws}), None, true);
                Ok(json!({"ok": true}))
            }
            "settings.update" => {
                self.human_only()?;
                let s = &mut self.inner.state.settings;
                if let Some(t) = a["theme"].as_str() { s.theme = t.into(); }
                if let Some(b) = a["backdrop"].as_str() { s.backdrop = b.to_ascii_uppercase(); }
                if let Some(c) = a["sidebarCollapsed"].as_bool() { s.sidebar_collapsed = c; }
                let v = json!(s);
                self.changed = true;
                self.emit("settings.changed", json!({"settings": v}), None, true);
                Ok(json!({"settings": v}))
            }
            "grants.list" => {
                self.human_only()?;
                Ok(json!({"grants": self.inner.state.grants}))
            }
            "grants.create" => {
                self.human_only()?;
                let ws: Vec<String> = a["workspaceIds"].as_array().map(|v| v.iter().filter_map(|x| x.as_str().map(str::to_string)).collect()).unwrap_or_default();
                for w in &ws {
                    self.inner.state.workspace(w).ok_or_else(|| CoreError::not_found(format!("no such workspace {w}")))?;
                }
                let token = new_secret();
                let now = now_ms();
                let g = Grant {
                    id: new_id("grant"), client: Self::arg_str(a, "client").into(), label: a["label"].as_str().unwrap_or(Self::arg_str(a, "client")).into(),
                    token_hash: hash_token(&token),
                    operations: a["operations"].as_array().unwrap().iter().filter_map(|x| x.as_str().map(str::to_string)).collect(),
                    workspace_ids: ws, created_at: now, expires_at: a["ttlSeconds"].as_i64().map(|s| now + s * 1000), revoked: false, paused: false,
                };
                self.inner.state.grants.push(g.clone());
                self.grant_changed = Some(g.id.clone());
                self.changed = true;
                self.emit("grant.created", json!({"grant": g}), None, true);
                Ok(json!({"grant": g, "pairingToken": token}))
            }
            "grants.revoke" => {
                self.human_only()?;
                let id = Self::arg_str(a, "grantId").to_string();
                let g = self.inner.state.grant_mut(&id).ok_or_else(|| CoreError::not_found("no such grant"))?;
                g.revoked = true;
                self.inner.state.approvals.retain(|ap| ap.grant_id != id);
                self.grant_changed = Some(id.clone());
                self.changed = true;
                self.emit("grant.revoked", json!({"grantId": id}), None, false);
                Ok(json!({"revoked": true}))
            }
            "agents.pause" | "agents.resume" => {
                self.human_only()?;
                let id = Self::arg_str(a, "grantId").to_string();
                let paused = spec.name == "agents.pause";
                let g = self.inner.state.grant_mut(&id).ok_or_else(|| CoreError::not_found("no such grant"))?;
                g.paused = paused;
                self.grant_changed = Some(id.clone());
                self.changed = true;
                self.emit("activity.changed", json!({"grantId": id, "paused": paused}), None, true);
                Ok(json!({"paused": paused}))
            }
            "approvals.resolve" => {
                self.human_only()?;
                let id = Self::arg_str(a, "approvalId");
                let approve = a["approve"].as_bool().unwrap_or(false);
                let pos = self.inner.state.approvals.iter().position(|x| x.id == id).ok_or_else(|| CoreError::not_found("no such approval"))?;
                let ap = self.inner.state.approvals.remove(pos);
                let live = self.inner.state.grant(&ap.grant_id).is_some_and(|g| g.is_live(now_ms()));
                let mut result = json!(null);
                if approve && live && ap.command == "favorites.add" {
                    let url = ap.args["url"].as_str().unwrap_or_default().to_string();
                    let title = ap.args["title"].as_str().unwrap_or_default().to_string();
                    result = json!({"favorite": self.add_favorite(&url, &title)});
                }
                self.changed = true;
                let status = if approve && live { "approved" } else { "denied" };
                self.emit("approval.resolved", json!({"approvalId": ap.id, "status": status, "grantId": ap.grant_id, "result": result}), None, false);
                Ok(json!({"status": status, "result": result}))
            }
            "activity.list" => {
                self.human_only()?;
                Ok(json!({"activity": self.inner.store.activity(a["limit"].as_i64().unwrap_or(100))?}))
            }
            "history.search" => {
                self.human_only()?;
                let pid = self.inner.state.profile_of_workspace(&self.inner.state.window.active_workspace_id).map(|p| p.id.clone()).unwrap_or_default();
                Ok(json!({"entries": self.inner.store.search_history(&pid, Self::arg_str(a, "query"), a["limit"].as_i64().unwrap_or(50))?}))
            }
            "events.poll" => {
                let after = a["afterSeq"].as_u64().unwrap_or(0);
                let max = a["max"].as_u64().unwrap_or(50) as usize;
                let st = &self.inner.state;
                let scope = match &st_actor { Actor::Agent { grant_id, .. } => Some((grant_id.clone(), agent_scope(st, grant_id))), Actor::Human => None };
                let (evs, resync) = self.inner.events.poll(after, max, |e| match &scope {
                    None => true,
                    Some((gid, s)) => {
                        !e.ui_only
                            && match &e.workspace_id {
                                Some(w) => s.contains(w),
                                None => e.data["grantId"].as_str() == Some(gid.as_str()),
                            }
                    }
                });
                Ok(json!({"events": evs, "lastSeq": self.inner.events.last_seq(), "resyncRequired": resync}))
            }
            other => Err(CoreError::unsupported(format!("{other} is declared but not implemented in this build"))),
        }
    }

    fn add_favorite(&mut self, url: &str, title: &str) -> Favorite {
        if let Some(f) = self.inner.state.favorites.iter().find(|f| f.url == url) {
            return f.clone();
        }
        let f = Favorite { id: new_id("fav"), url: url.into(), title: title.into(), order: self.inner.state.favorites.len() as i64 };
        self.inner.state.favorites.push(f.clone());
        self.changed = true;
        self.emit("favorite.added", json!({"favorite": f}), None, true);
        f
    }

    fn snapshot(&self) -> Value {
        let st = &self.inner.state;
        let now = now_ms();
        let agents: Vec<Value> = st
            .grants
            .iter()
            .filter(|g| g.is_live(now))
            .map(|g| {
                let scope = agent_scope(st, &g.id);
                let rt = st.agent_runtime.get(&g.id);
                let working = rt.is_some_and(|r| r.in_flight > 0 || now - r.last_command_ms < WORKING_WINDOW_MS) && !g.paused;
                let tab_count = st.tabs.iter().filter(|t| scope.contains(&t.workspace_id)).count();
                json!({"grantId": g.id, "client": g.client, "label": g.label, "paused": g.paused, "working": working, "tabCount": tab_count, "workspaceIds": scope})
            })
            .collect();
        let mut tabs: Vec<&Tab> = st.tabs.iter().collect();
        tabs.sort_by_key(|t| (!t.pinned, t.order));
        json!({
            "revision": st.revision, "instanceId": st.instance_id, "window": st.window,
            "workspaces": st.workspaces.iter().map(workspace_view).collect::<Vec<_>>(),
            "tabs": tabs.into_iter().map(|t| tab_view(st, t)).collect::<Vec<_>>(),
            "favorites": st.favorites, "settings": st.settings, "agents": agents,
            "approvals": st.approvals, "lastSeq": self.inner.events.last_seq(),
        })
    }

    /// Returns whether persistent fields changed.
    fn apply_engine_event(&mut self, ev: EngineEvent) -> bool {
        let tab_id = match &ev {
            EngineEvent::NavigationStarted { tab_id, .. } | EngineEvent::NavigationCommitted { tab_id, .. } | EngineEvent::NavigationFinished { tab_id, .. }
            | EngineEvent::NavigationFailed { tab_id, .. } | EngineEvent::TitleChanged { tab_id, .. } | EngineEvent::UrlChanged { tab_id, .. }
            | EngineEvent::FaviconChanged { tab_id, .. } | EngineEvent::AudibleChanged { tab_id, .. } | EngineEvent::ProcessTerminated { tab_id } => tab_id.clone(),
            EngineEvent::PopupRequested { opener_tab_id, .. } => opener_tab_id.clone(),
            _ => return false,
        };
        let Some(tab) = self.inner.state.tab(&tab_id).cloned() else { return false };
        let mut persist = false;
        let mut history: Option<(String, String)> = None;
        if let EngineEvent::PopupRequested { url, user_gesture, .. } = &ev {
            let Ok(url) = urls::require_http(url) else { return false };
            if !*user_gesture {
                self.emit("popup.blocked", json!({"tabId": tab_id, "url": urls::redact(&url)}), None, true);
                return false;
            }
            let opener_active = self.inner.state.is_active_tab(&tab_id);
            let t = self.create_tab(&tab.workspace_id, &url, false, OpenedBy::Popup);
            if opener_active {
                self.inner.state.window.active_tab_by_workspace.insert(tab.workspace_id.clone(), t.id.clone());
                self.cmds.push(EngineCommand::Show { tab_id: Some(t.id), focus: true });
                self.emit("window.changed", json!({"window": self.inner.state.window}), None, true);
            }
            return true;
        }
        {
            let t = self.inner.state.tab_mut(&tab_id).unwrap();
            match ev {
                EngineEvent::NavigationStarted { navigation_id, .. } => {
                    t.load_state = LoadState::Loading;
                    t.navigation_id = navigation_id;
                    t.error = None;
                }
                EngineEvent::NavigationCommitted { navigation_id, url, secure, can_go_back, can_go_forward, .. } => {
                    t.navigation_id = navigation_id;
                    t.url = url;
                    t.secure = secure;
                    t.can_go_back = can_go_back;
                    t.can_go_forward = can_go_forward;
                    t.favicon_url = None;
                    persist = true;
                }
                EngineEvent::NavigationFinished { navigation_id, title, .. } => {
                    if navigation_id == t.navigation_id {
                        t.load_state = LoadState::Loaded;
                        if !title.is_empty() { t.title = title; }
                        history = Some((t.url.clone(), t.title.clone()));
                        persist = true;
                    }
                }
                EngineEvent::NavigationFailed { navigation_id, message, .. } => {
                    if navigation_id == t.navigation_id {
                        t.load_state = LoadState::Failed;
                        t.error = Some(message.chars().take(512).collect());
                    }
                }
                EngineEvent::TitleChanged { title, .. } => {
                    t.title = title.chars().take(512).collect();
                    persist = true;
                }
                EngineEvent::UrlChanged { url, can_go_back, can_go_forward, .. } => {
                    t.url = url;
                    t.can_go_back = can_go_back;
                    t.can_go_forward = can_go_forward;
                    persist = true;
                }
                EngineEvent::FaviconChanged { favicon_url, .. } => {
                    t.favicon_url = urls::require_http(&favicon_url).ok();
                    persist = true;
                }
                EngineEvent::AudibleChanged { audible, .. } => t.audible = audible,
                EngineEvent::ProcessTerminated { .. } => t.load_state = LoadState::Crashed,
                _ => {}
            }
        }
        self.changed = true;
        if let Some((url, title)) = history {
            let ws = self.inner.state.workspace(&tab.workspace_id).cloned();
            if let Some(w) = ws.filter(|w| w.profile_kind.persists() && url.starts_with("http")) {
                let _ = self.inner.store.add_history(&w.profile_id, &url, &title, now_ms());
            }
        }
        let v = tab_view(&self.inner.state, self.inner.state.tab(&tab_id).unwrap());
        self.emit("tab.updated", json!({"tab": v}), Some(&tab.workspace_id), false);
        persist
    }
}
