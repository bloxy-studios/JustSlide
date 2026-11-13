use crate::ids::{new_id, now_ms};
use crate::model::*;
use std::collections::{BTreeMap, HashMap};

/// In-memory projection of app-owned state. Mutated only by `Core` while
/// holding its lock; every committed mutation bumps `revision`.
#[derive(Debug, Clone, Default)]
pub struct State {
    pub instance_id: String,
    pub revision: i64,
    pub profiles: BTreeMap<String, Profile>,
    pub workspaces: Vec<Workspace>,
    pub tabs: Vec<Tab>,
    pub favorites: Vec<Favorite>,
    pub settings: Settings,
    pub window: WindowState,
    pub grants: Vec<Grant>,
    pub approvals: Vec<Approval>,
    pub agent_runtime: HashMap<String, AgentRuntime>,
}

#[derive(Debug, Clone, Default)]
pub struct AgentRuntime {
    pub last_command_ms: i64,
    pub in_flight: u32,
    pub tab_creations: Vec<i64>,
}

/// Default workspaces shown in the Figma dock. Agent-oriented workspaces get
/// their own isolated WebKit data store; the icon alone never implies that an
/// agent is connected.
const DEFAULT_WORKSPACES: &[(&str, &str, ProfileKind)] = &[
    ("Personal", "personal", ProfileKind::Personal),
    ("Codex", "codex", ProfileKind::AgentIsolated),
    ("Claude Code", "claude-code", ProfileKind::AgentIsolated),
    ("ChatGPT", "chatgpt", ProfileKind::AgentIsolated),
];

impl State {
    pub fn seeded() -> State {
        let mut s = State { instance_id: new_id("inst"), ..Default::default() };
        for (i, (name, icon, kind)) in DEFAULT_WORKSPACES.iter().enumerate() {
            let p = s.new_profile(*kind, name);
            s.workspaces.push(Workspace {
                id: new_id("ws"), name: name.to_string(), icon: icon.to_string(), profile_id: p, profile_kind: *kind,
                agent_owned: false, owner_grant_id: None, order: i as i64, pinned_collapsed: false,
            });
        }
        s.window = WindowState { id: new_id("win"), active_workspace_id: s.workspaces[0].id.clone(), active_tab_by_workspace: BTreeMap::new() };
        s
    }

    pub fn new_profile(&mut self, kind: ProfileKind, name: &str) -> String {
        let id = new_id("prof");
        let ds = crate::ids::uuid_v4();
        self.profiles.insert(id.clone(), Profile { id: id.clone(), kind, name: name.to_string(), data_store_id: ds });
        id
    }

    pub fn personal_workspace_id(&self) -> Option<&str> {
        self.workspaces.iter().find(|w| w.profile_kind == ProfileKind::Personal).map(|w| w.id.as_str())
    }

    pub fn workspace(&self, id: &str) -> Option<&Workspace> { self.workspaces.iter().find(|w| w.id == id) }
    pub fn workspace_mut(&mut self, id: &str) -> Option<&mut Workspace> { self.workspaces.iter_mut().find(|w| w.id == id) }
    pub fn tab(&self, id: &str) -> Option<&Tab> { self.tabs.iter().find(|t| t.id == id) }
    pub fn tab_mut(&mut self, id: &str) -> Option<&mut Tab> { self.tabs.iter_mut().find(|t| t.id == id) }
    pub fn grant(&self, id: &str) -> Option<&Grant> { self.grants.iter().find(|g| g.id == id) }
    pub fn grant_mut(&mut self, id: &str) -> Option<&mut Grant> { self.grants.iter_mut().find(|g| g.id == id) }

    pub fn tabs_in<'a>(&'a self, ws: &'a str) -> impl Iterator<Item = &'a Tab> + 'a {
        self.tabs.iter().filter(move |t| t.workspace_id == ws)
    }

    pub fn profile_of_workspace(&self, ws: &str) -> Option<&Profile> {
        self.workspace(ws).and_then(|w| self.profiles.get(&w.profile_id))
    }

    pub fn tab_persists(&self, t: &Tab) -> bool {
        self.workspace(&t.workspace_id).is_some_and(|w| w.profile_kind.persists())
    }

    pub fn next_tab_order(&self, ws: &str, pinned: bool) -> i64 {
        self.tabs_in(ws).filter(|t| t.pinned == pinned).map(|t| t.order + 1).max().unwrap_or(0)
    }

    pub fn is_active_tab(&self, tab_id: &str) -> bool {
        self.window.active_tab().is_some_and(|t| t == tab_id)
    }

    pub fn new_tab(&self, ws: &str, url: &str, pinned: bool, by: OpenedBy, grant: Option<String>) -> Tab {
        Tab {
            id: new_id("tab"), workspace_id: ws.to_string(), url: url.to_string(), title: String::new(), favicon_url: None, pinned,
            order: self.next_tab_order(ws, pinned), load_state: LoadState::NotLoaded, error: None, opened_by: by, opened_by_grant: grant,
            navigation_id: String::new(), can_go_back: false, can_go_forward: false, audible: false, secure: false, created_at: now_ms(),
        }
    }

    /// Normalises ordering after removals/moves so orders stay dense.
    pub fn renumber(&mut self, ws: &str) {
        for pinned in [true, false] {
            let mut ids: Vec<(i64, String)> = self.tabs.iter().filter(|t| t.workspace_id == ws && t.pinned == pinned).map(|t| (t.order, t.id.clone())).collect();
            ids.sort();
            for (i, (_, id)) in ids.into_iter().enumerate() {
                if let Some(t) = self.tab_mut(&id) { t.order = i as i64; }
            }
        }
    }
}
