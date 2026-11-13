//! App-owned domain entities. WebKit remains authoritative for rendering and
//! real navigation; these records are what JustSlide itself owns and persists.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProfileKind {
    Personal,
    AgentIsolated,
    Private,
}

impl ProfileKind {
    pub fn as_str(self) -> &'static str {
        match self { Self::Personal => "personal", Self::AgentIsolated => "agent-isolated", Self::Private => "private" }
    }
    pub fn parse(s: &str) -> Option<Self> {
        match s { "personal" => Some(Self::Personal), "agent-isolated" => Some(Self::AgentIsolated), "private" => Some(Self::Private), _ => None }
    }
    /// Private profiles never persist tabs or history.
    pub fn persists(self) -> bool { self != Self::Private }
}

/// A browser profile = one WebKit website-data-store boundary.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub id: String,
    pub kind: ProfileKind,
    pub name: String,
    /// UUID passed to `WKWebsiteDataStore(forIdentifier:)` (macOS 14+).
    pub data_store_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub profile_id: String,
    pub profile_kind: ProfileKind,
    /// True when created by an agent grant (shown as agent-owned in the dock).
    pub agent_owned: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_grant_id: Option<String>,
    pub order: i64,
    pub pinned_collapsed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LoadState {
    /// Restored or background metadata only; no WebKit view exists yet.
    NotLoaded,
    Loading,
    Loaded,
    Failed,
    Crashed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OpenedBy {
    User,
    Agent,
    Restore,
    Popup,
}

impl OpenedBy {
    pub fn as_str(self) -> &'static str {
        match self { Self::User => "user", Self::Agent => "agent", Self::Restore => "restore", Self::Popup => "popup" }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tab {
    pub id: String,
    pub workspace_id: String,
    pub url: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favicon_url: Option<String>,
    pub pinned: bool,
    pub order: i64,
    pub load_state: LoadState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub opened_by: OpenedBy,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opened_by_grant: Option<String>,
    pub navigation_id: String,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub audible: bool,
    /// Set when WebKit reports a secure (HTTPS, valid TLS) committed load.
    pub secure: bool,
    pub created_at: i64,
}

impl Tab {
    pub fn loading(&self) -> bool { self.load_state == LoadState::Loading }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Favorite {
    pub id: String,
    pub url: String,
    pub title: String,
    pub order: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub theme: String,
    /// User-selected backdrop. #ACFFF7 is the Figma *example*, not a brand colour.
    pub backdrop: String,
    pub sidebar_collapsed: bool,
    pub search_template: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings { theme: "system".into(), backdrop: "#ACFFF7".into(), sidebar_collapsed: false, search_template: "https://duckduckgo.com/?q={query}".into() }
    }
}

/// Human window selection. Agents can never change this.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WindowState {
    pub id: String,
    pub active_workspace_id: String,
    pub active_tab_by_workspace: BTreeMap<String, String>,
}

impl WindowState {
    pub fn active_tab(&self) -> Option<&String> { self.active_tab_by_workspace.get(&self.active_workspace_id) }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Grant {
    pub id: String,
    pub client: String,
    pub label: String,
    #[serde(skip)]
    pub token_hash: String,
    pub operations: Vec<String>,
    /// Pre-existing workspaces explicitly granted by the person. Workspaces an
    /// agent creates are owned by its grant and implicitly in scope.
    pub workspace_ids: Vec<String>,
    pub created_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
    pub revoked: bool,
    pub paused: bool,
}

impl Grant {
    pub fn is_live(&self, now: i64) -> bool { !self.revoked && self.expires_at.is_none_or(|e| e > now) }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityRecord {
    pub at: i64,
    pub grant_id: String,
    pub client: String,
    pub command: String,
    pub outcome: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tab_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Approval {
    pub id: String,
    pub grant_id: String,
    pub client: String,
    pub command: String,
    pub summary: String,
    #[serde(skip)]
    pub args: serde_json::Value,
    pub created_at: i64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClosedTab {
    pub workspace_id: String,
    pub url: String,
    pub title: String,
    pub pinned: bool,
    pub closed_at: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    pub url: String,
    pub title: String,
    pub at: i64,
}
