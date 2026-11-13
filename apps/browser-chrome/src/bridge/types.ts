// Mirrors the core's UI projection (crates/justslide-core/src/core.rs:
// tab_view, workspace_view, snapshot). The chrome never invents state.
export type LoadState = 'notLoaded' | 'loading' | 'loaded' | 'failed' | 'crashed';

export interface TabView {
  id: string; workspaceId: string; url: string; title: string; pinned: boolean; order: number;
  loading: boolean; loadState: LoadState; active: boolean; openedBy: 'user' | 'agent' | 'restore' | 'popup';
  navigationId: string; crashed: boolean; audible: boolean; faviconUrl?: string | null;
  canGoBack: boolean; canGoForward: boolean; secure: boolean; error?: string | null;
}

export interface WorkspaceView {
  id: string; name: string; icon: string; profileKind: 'personal' | 'agent-isolated' | 'private';
  agentOwned: boolean; order: number; pinnedCollapsed: boolean;
}

export interface Favorite { id: string; url: string; title: string; order: number }
export interface Settings { theme: 'light' | 'dark' | 'system'; backdrop: string; sidebarCollapsed: boolean; searchTemplate: string }
export interface AgentView { grantId: string; client: string; label: string; paused: boolean; working: boolean; tabCount: number; workspaceIds: string[] }
export interface Approval { id: string; grantId: string; client: string; command: string; summary: string; createdAt: number; status: string }

export interface Snapshot {
  revision: number; instanceId: string;
  window: { id: string; activeWorkspaceId: string; activeTabByWorkspace: Record<string, string> };
  workspaces: WorkspaceView[]; tabs: TabView[]; favorites: Favorite[]; settings: Settings;
  agents: AgentView[]; approvals: Approval[]; lastSeq: number;
  /** Present only in the development design fixture. */
  fixture?: { variant: 'desktop' | 'compact'; illustrativeAgent: boolean };
}

export interface CoreError { code: string; message: string; retryable: boolean; pendingRequestId?: string }
