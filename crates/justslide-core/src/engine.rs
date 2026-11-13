//! Platform rendering adapter boundary. Core issues typed `EngineCommand`s;
//! the adapter (macOS: Swift `WebKitBrowserAdapter` over the FFI) performs
//! them on the main thread and reports real WebKit callbacks back as
//! `EngineEvent`s. Core never fabricates a successful page load.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum EngineCommand {
    #[serde(rename_all = "camelCase")]
    CreateView { tab_id: String, profile_id: String, data_store_id: String, private: bool, url: String, navigation_id: String },
    #[serde(rename_all = "camelCase")]
    DestroyView { tab_id: String },
    #[serde(rename_all = "camelCase")]
    Navigate { tab_id: String, url: String, navigation_id: String },
    #[serde(rename_all = "camelCase")]
    GoBack { tab_id: String },
    #[serde(rename_all = "camelCase")]
    GoForward { tab_id: String },
    #[serde(rename_all = "camelCase")]
    Reload { tab_id: String },
    #[serde(rename_all = "camelCase")]
    Stop { tab_id: String },
    /// Attach `tab_id` (or nothing) to the human window's content area.
    /// Only ever emitted for human actions. `focus` asks the adapter to make
    /// the page first responder (e.g. after the person picks a tab).
    #[serde(rename_all = "camelCase")]
    Show { tab_id: Option<String>, focus: bool },
    #[serde(rename_all = "camelCase")]
    Screenshot { request_id: String, tab_id: String, max_edge_px: u32, format: String },
    #[serde(rename_all = "camelCase")]
    Snapshot { request_id: String, tab_id: String, max_nodes: u32 },
}

impl EngineCommand {
    pub fn tab_id(&self) -> Option<&str> {
        match self {
            Self::CreateView { tab_id, .. } | Self::DestroyView { tab_id } | Self::Navigate { tab_id, .. } | Self::GoBack { tab_id }
            | Self::GoForward { tab_id } | Self::Reload { tab_id } | Self::Stop { tab_id } | Self::Screenshot { tab_id, .. }
            | Self::Snapshot { tab_id, .. } => Some(tab_id),
            Self::Show { tab_id, .. } => tab_id.as_deref(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Capture {
    pub mime_type: String,
    pub data_base64: String,
    pub width: u32,
    pub height: u32,
    /// True when the adapter knows the pixels may be stale (e.g. a hidden
    /// view WebKit has not re-rendered).
    #[serde(default)]
    pub possibly_stale: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AxNode {
    pub role: String,
    pub name: String,
    pub depth: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum EngineEvent {
    #[serde(rename_all = "camelCase")]
    NavigationStarted { tab_id: String, navigation_id: String, url: String },
    #[serde(rename_all = "camelCase")]
    NavigationCommitted { tab_id: String, navigation_id: String, url: String, secure: bool, can_go_back: bool, can_go_forward: bool },
    #[serde(rename_all = "camelCase")]
    NavigationFinished { tab_id: String, navigation_id: String, title: String },
    #[serde(rename_all = "camelCase")]
    NavigationFailed { tab_id: String, navigation_id: String, message: String },
    #[serde(rename_all = "camelCase")]
    TitleChanged { tab_id: String, title: String },
    #[serde(rename_all = "camelCase")]
    UrlChanged { tab_id: String, url: String, can_go_back: bool, can_go_forward: bool },
    #[serde(rename_all = "camelCase")]
    FaviconChanged { tab_id: String, favicon_url: String },
    #[serde(rename_all = "camelCase")]
    AudibleChanged { tab_id: String, audible: bool },
    #[serde(rename_all = "camelCase")]
    ProcessTerminated { tab_id: String },
    /// A page asked for a new window (window.open / target=_blank).
    #[serde(rename_all = "camelCase")]
    PopupRequested { opener_tab_id: String, url: String, user_gesture: bool },
    #[serde(rename_all = "camelCase")]
    ScreenshotResult { request_id: String, navigation_id: String, capture: Option<Capture>, error: Option<String> },
    #[serde(rename_all = "camelCase")]
    SnapshotResult { request_id: String, navigation_id: String, url: String, nodes: Option<Vec<AxNode>>, truncated: bool, error: Option<String> },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Support {
    Supported,
    Partial,
    Unsupported,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Feature {
    pub support: Support,
    pub note: String,
}

/// Typed, adapter-reported feature registry (never guessed by core).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EngineCapabilities {
    /// `webkit` for the shipping adapter; `test-double` for tests.
    pub engine: String,
    pub engine_version: String,
    pub screenshots: Feature,
    pub background_capture: Feature,
    pub page_snapshot: Feature,
    pub profile_isolation: Feature,
    pub private_profiles: Feature,
    pub downloads: Feature,
    pub web_inspector: Feature,
    pub extensions: Feature,
    pub tab_suspension: Feature,
    pub page_automation: Feature,
}

impl EngineCapabilities {
    pub fn unknown(engine: &str) -> Self {
        let u = || Feature { support: Support::Unknown, note: "not reported by adapter".into() };
        EngineCapabilities {
            engine: engine.into(), engine_version: String::new(), screenshots: u(), background_capture: u(), page_snapshot: u(),
            profile_isolation: u(), private_profiles: u(), downloads: u(), web_inspector: u(), extensions: u(), tab_suspension: u(), page_automation: u(),
        }
    }
}

pub trait BrowserEngine: Send + Sync {
    fn capabilities(&self) -> EngineCapabilities;
    /// Must not block and must not call back into `Core` synchronously.
    fn send(&self, cmd: EngineCommand);
}

/// Engine used before an adapter attaches (e.g. headless tooling). Reports
/// every rendering feature as unsupported and drops commands.
pub struct DetachedEngine;

impl BrowserEngine for DetachedEngine {
    fn capabilities(&self) -> EngineCapabilities {
        let mut c = EngineCapabilities::unknown("detached");
        let no = || Feature { support: Support::Unsupported, note: "no rendering adapter attached".into() };
        c.screenshots = no();
        c.background_capture = no();
        c.page_snapshot = no();
        c
    }
    fn send(&self, _cmd: EngineCommand) {}
}
