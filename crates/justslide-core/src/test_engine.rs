//! TEST DOUBLE. Deterministic in-process engine used by core/MCP tests and
//! the dev host. It does not render anything: captures are labelled
//! `image/x-justslide-test-capture` and encode the tab id and URL so tests
//! can prove the *requested* tab was captured. Never shipped.

use crate::core::Core;
use crate::engine::*;
use base64::Engine as _;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};

#[derive(Default)]
pub struct TestEngineState {
    pub log: Vec<EngineCommand>,
    pub views: HashMap<String, String>,
    /// Tab currently attached to the human window.
    pub shown: Option<String>,
    /// Keyboard first responder: `chrome` or a tab id.
    pub first_responder: String,
    pub history: HashMap<String, Vec<String>>,
}

pub struct TestEngine {
    core: Mutex<Weak<Core>>,
    pub state: Mutex<TestEngineState>,
    /// When true, events are delivered synchronously on a helper thread
    /// (still never re-entrantly from `send`).
    pub fail_hosts: Mutex<Vec<String>>,
}

impl TestEngine {
    pub fn attach(core: &Arc<Core>) -> Arc<TestEngine> {
        let e = Arc::new(TestEngine { core: Mutex::new(Arc::downgrade(core)), state: Mutex::new(TestEngineState { first_responder: "chrome".into(), ..Default::default() }), fail_hosts: Mutex::new(vec![]) });
        core.set_engine(e.clone());
        e
    }

    fn deliver(&self, evs: Vec<EngineEvent>) {
        let core = self.core.lock().unwrap().clone();
        std::thread::spawn(move || {
            if let Some(c) = core.upgrade() {
                for ev in evs {
                    c.on_engine_event(ev);
                }
            }
        });
    }

    fn load(&self, tab_id: &str, url: &str, nav: &str) -> Vec<EngineEvent> {
        let host = crate::urls::host_of(url);
        let mut st = self.state.lock().unwrap();
        st.views.insert(tab_id.into(), url.into());
        let h = st.history.entry(tab_id.into()).or_default();
        h.push(url.into());
        let back = h.len() > 1;
        drop(st);
        let mut evs = vec![EngineEvent::NavigationStarted { tab_id: tab_id.into(), navigation_id: nav.into(), url: url.into() }];
        if self.fail_hosts.lock().unwrap().contains(&host) {
            evs.push(EngineEvent::NavigationFailed { tab_id: tab_id.into(), navigation_id: nav.into(), message: format!("A server with the specified hostname could not be found. ({host})") });
            return evs;
        }
        evs.push(EngineEvent::NavigationCommitted { tab_id: tab_id.into(), navigation_id: nav.into(), url: url.into(), secure: url.starts_with("https://"), can_go_back: back, can_go_forward: false });
        evs.push(EngineEvent::NavigationFinished { tab_id: tab_id.into(), navigation_id: nav.into(), title: format!("{host} (test page)") });
        evs
    }
}

impl BrowserEngine for TestEngine {
    fn capabilities(&self) -> EngineCapabilities {
        let yes = |n: &str| Feature { support: Support::Supported, note: n.into() };
        let mut c = EngineCapabilities::unknown("test-double");
        c.engine_version = "0".into();
        c.screenshots = yes("test double: returns a labelled non-image payload");
        c.background_capture = yes("test double");
        c.page_snapshot = yes("test double");
        c.profile_isolation = yes("test double");
        c.private_profiles = yes("test double");
        c
    }

    fn send(&self, cmd: EngineCommand) {
        self.state.lock().unwrap().log.push(cmd.clone());
        let evs = match cmd {
            EngineCommand::CreateView { tab_id, url, navigation_id, .. } | EngineCommand::Navigate { tab_id, url, navigation_id } => self.load(&tab_id, &url, &navigation_id),
            EngineCommand::DestroyView { tab_id } => {
                let mut st = self.state.lock().unwrap();
                st.views.remove(&tab_id);
                if st.shown.as_deref() == Some(tab_id.as_str()) { st.shown = None; }
                if st.first_responder == tab_id { st.first_responder = "chrome".into(); }
                vec![]
            }
            EngineCommand::Show { tab_id, focus } => {
                let mut st = self.state.lock().unwrap();
                st.shown = tab_id.clone();
                if focus { if let Some(t) = tab_id { st.first_responder = t; } }
                vec![]
            }
            EngineCommand::Screenshot { request_id, tab_id, .. } => {
                let st = self.state.lock().unwrap();
                let url = st.views.get(&tab_id).cloned();
                drop(st);
                let nav = self.core.lock().unwrap().upgrade().and_then(|c| c.debug_state().tab(&tab_id).map(|t| t.navigation_id.clone())).unwrap_or_default();
                vec![match url {
                    Some(u) => EngineEvent::ScreenshotResult {
                        request_id, navigation_id: nav,
                        capture: Some(Capture { mime_type: "image/x-justslide-test-capture".into(), data_base64: base64::engine::general_purpose::STANDARD.encode(format!("tab={tab_id};url={u}")), width: 1280, height: 800, possibly_stale: false }),
                        error: None,
                    },
                    None => EngineEvent::ScreenshotResult { request_id, navigation_id: nav, capture: None, error: Some("no live view".into()) },
                }]
            }
            EngineCommand::Snapshot { request_id, tab_id, .. } => {
                let core = self.core.lock().unwrap().upgrade();
                let t = core.and_then(|c| c.debug_state().tab(&tab_id).cloned());
                vec![EngineEvent::SnapshotResult {
                    request_id, navigation_id: t.as_ref().map(|t| t.navigation_id.clone()).unwrap_or_default(), url: t.as_ref().map(|t| t.url.clone()).unwrap_or_default(),
                    nodes: Some(vec![AxNode { role: "document".into(), name: t.map(|t| t.title).unwrap_or_default(), depth: 0 }]), truncated: false, error: None,
                }]
            }
            _ => vec![],
        };
        if !evs.is_empty() {
            self.deliver(evs);
        }
    }
}
