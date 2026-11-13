// DEVELOPMENT-ONLY fixture bridge for the browser preview and visual tests.
// It reproduces the Figma reference content and a small subset of core
// semantics in memory. It is NOT the browser: nothing is rendered or loaded,
// and the production entry (main.tsx) never imports this module.
import fixture from '../../../../tests/fixtures/design/slide-fixture.json';
import type { ChromeBridge } from './bridge';
import type { Snapshot, TabView, WorkspaceView } from './types';

let seq = 0;
const id = (p: string) => `${p}_fixture${(++seq).toString().padStart(8, '0')}`;

export function buildFixtureSnapshot(variant: 'desktop' | 'compact'): Snapshot {
  const v = fixture.variants[variant];
  const icons = ['personal', 'codex', 'claude-code', 'chatgpt'];
  const names = ['Personal', 'Codex', 'Claude Code', 'ChatGPT'];
  const workspaces: WorkspaceView[] = icons.map((icon, i) => ({
    id: id('ws'), name: names[i], icon, profileKind: i === 0 ? 'personal' : 'agent-isolated', agentOwned: false, order: i,
    pinnedCollapsed: i === 0 ? v.pinnedCollapsed : false,
  }));
  const personal = workspaces[0].id;
  const tab = (t: { icon: string; title: string; url: string; active?: boolean }, pinned: boolean, order: number): TabView => ({
    id: id('tab'), workspaceId: personal, url: t.url, title: t.title, pinned, order, loading: false, loadState: 'loaded',
    active: !!t.active, openedBy: 'restore', navigationId: id('nav'), crashed: false, audible: false, faviconUrl: `fixture:${t.icon}`,
    canGoBack: true, canGoForward: true, secure: true, error: null,
  });
  const tabs = [...fixture.pinned.map((t, i) => tab(t, true, i)), ...fixture.tabs.map((t, i) => tab(t, false, i))];
  const active = tabs.find((t) => t.active)!;
  const agent = v.agent;
  return {
    revision: 1, instanceId: 'inst_designfixture', lastSeq: 0,
    window: { id: 'win_fixture', activeWorkspaceId: personal, activeTabByWorkspace: { [personal]: active.id } },
    workspaces, tabs,
    favorites: fixture.favorites.map((f, i) => ({ id: `fav_${f.key}`, url: f.url, title: f.title, order: i })),
    settings: { theme: 'light', backdrop: '#ACFFF7', sidebarCollapsed: false, searchTemplate: 'https://duckduckgo.com/?q={query}' },
    agents: agent ? [{ grantId: 'grant_fixture', client: agent.client, label: agent.label, paused: false, working: true, tabCount: agent.tabs.length, workspaceIds: [workspaces[2].id] }] : [],
    approvals: [],
    fixture: { variant, illustrativeAgent: !!agent },
  };
}

export function createFixtureBridge(variant: 'desktop' | 'compact'): ChromeBridge {
  let snap = buildFixtureSnapshot(variant);
  const listeners = new Set<() => void>();
  const commit = (next: Snapshot) => { snap = { ...next, revision: next.revision + 1 }; listeners.forEach((l) => l()); };
  const withActive = (s: Snapshot, ws: string, tabId?: string) => {
    const map = { ...s.window.activeTabByWorkspace };
    if (tabId) map[ws] = tabId; else delete map[ws];
    return { ...s, window: { ...s.window, activeWorkspaceId: ws, activeTabByWorkspace: map }, tabs: s.tabs.map((t) => ({ ...t, active: t.id === map[ws] })) };
  };
  return {
    kind: 'fixture',
    getSnapshot: () => snap,
    subscribe: (l) => { listeners.add(l); return () => listeners.delete(l); },
    reportContentBounds: () => {},
    async execute(command, args: any) {
      const s = snap;
      switch (command) {
        case 'tabs.activate': { const t = s.tabs.find((x) => x.id === args.tabId); if (t) commit(withActive(s, t.workspaceId, t.id)); return {}; }
        case 'tabs.close': {
          const rest = s.tabs.filter((t) => t.id !== args.tabId);
          const ws = s.window.activeWorkspaceId;
          const wasActive = s.window.activeTabByWorkspace[ws] === args.tabId;
          commit(withActive({ ...s, tabs: rest }, ws, wasActive ? rest.find((t) => t.workspaceId === ws && !t.pinned)?.id : s.window.activeTabByWorkspace[ws]));
          return {};
        }
        case 'tabs.pin': commit({ ...s, tabs: s.tabs.map((t) => (t.id === args.tabId ? { ...t, pinned: args.pinned } : t)) }); return {};
        case 'ui.setPinnedCollapsed': commit({ ...s, workspaces: s.workspaces.map((w) => (w.id === s.window.activeWorkspaceId ? { ...w, pinnedCollapsed: args.collapsed } : w)) }); return {};
        case 'workspaces.switch': commit(withActive(s, args.workspaceId, s.window.activeTabByWorkspace[args.workspaceId])); return {};
        case 'settings.update': commit({ ...s, settings: { ...s.settings, ...args } }); return {};
        case 'agents.pause': case 'agents.resume':
          commit({ ...s, agents: s.agents.map((a) => (a.grantId === args.grantId ? { ...a, paused: command === 'agents.pause' } : a)) }); return {};
        case 'tabs.open': {
          const input = String(args.input);
          const t: TabView = { ...s.tabs[0], id: id('tab'), pinned: false, order: 999, title: input === 'about:blank' ? 'New Tab' : input, url: input, faviconUrl: null, active: true, workspaceId: s.window.activeWorkspaceId };
          commit(withActive({ ...s, tabs: [...s.tabs, t] }, s.window.activeWorkspaceId, t.id));
          return { tab: t };
        }
        default: return {};
      }
    },
  };
}
