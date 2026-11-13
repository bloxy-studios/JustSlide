/** @jsxRuntime classic */
/** @jsx jsx */
// Workspace footer (12:290): WORKSPACES heading + current name, Downloads,
// workspace switchers (Personal/Codex/Claude Code/ChatGPT…) and New
// workspace. These SWITCH TAB COLLECTIONS; website launchers live above.
import { cssMap, jsx } from '@compiled/react';
import { token } from '@atlaskit/tokens';
import { useBridge } from '../bridge/bridge';
import type { AgentView, WorkspaceView } from '../bridge/types';
import { UI_ICONS, WORKSPACE_ICONS } from '../assets/icons';
import { Glyph, Img } from './Icon';
import { offsetInSidebar, useHoverLabel } from './HoverLabel';

const styles = cssMap({
  footer: { width: '100%', height: '64px', boxSizing: 'border-box', flexShrink: 0, paddingTop: token('space.100'), display: 'flex', flexDirection: 'column', gap: token('space.100') },
  heading: { height: '16px', display: 'flex', alignItems: 'center', justifyContent: 'space-between', paddingInline: token('space.100') },
  overline: { font: 'var(--slide-font-overline)', letterSpacing: '0.6px', textTransform: 'uppercase', color: token('color.text.subtlest'), opacity: 0.68, margin: 0 },
  current: { font: 'var(--slide-font-caption)', color: token('color.text.subtle') },
  controls: { height: '32px', display: 'flex', alignItems: 'center', justifyContent: 'space-between', paddingInline: token('space.050') },
  switcher: { display: 'flex', alignItems: 'center', gap: token('space.050'), margin: 0, padding: 0, listStyle: 'none' },
  btn: {
    position: 'relative', width: '32px', height: '32px', boxSizing: 'border-box', display: 'grid', placeItems: 'center', border: 'none',
    padding: 0, margin: 0, borderRadius: token('radius.large'), backgroundColor: 'transparent', color: token('color.icon'), cursor: 'pointer',
    '&:hover': { backgroundColor: token('color.background.neutral.subtle.hovered') },
    '&:focus-visible': { outline: 'none', boxShadow: `inset 0 0 0 2px ${token('color.border.focused')}` },
    '&[aria-disabled="true"]': { cursor: 'default' },
  },
  selected: { backgroundColor: token('elevation.surface'), boxShadow: `inset 0 0 0 1px ${token('color.border.bold')}`, '&:hover': { backgroundColor: token('elevation.surface') } },
  dot: {
    position: 'absolute', insetInlineStart: '24px', top: '2px', width: '7px', height: '7px', boxSizing: 'border-box', borderRadius: '50%',
    backgroundColor: token('color.background.success.bold'), boxShadow: 'inset 0 0 0 1.5px var(--slide-backdrop)',
  },
  letter: { width: '16px', height: '16px', borderRadius: token('radius.small'), display: 'grid', placeItems: 'center', font: 'var(--slide-font-caption)', backgroundColor: token('color.background.neutral'), color: token('color.text') },
});

function WorkspaceIcon({ ws }: { ws: WorkspaceView }) {
  const src = WORKSPACE_ICONS[ws.icon];
  if (ws.icon === 'personal') return <Glyph src={src} size={16} color="var(--ds-icon-subtle)" />;
  if (src) return <Img src={src} width={16} height={16} />;
  return <span css={styles.letter} aria-hidden>{ws.name.slice(0, 1).toUpperCase()}</span>;
}

function SwitchButton({ ws, selected, active }: { ws: WorkspaceView; selected: boolean; active: AgentView | undefined }) {
  const b = useBridge();
  const label = `Switch to ${ws.name} workspace`;
  const hover = useHoverLabel(`ws-${ws.id}`, label, (el) => offsetInSidebar(el).top - 68);
  return (
    <li style={{ display: 'contents' }}>
      <button type="button" css={[styles.btn, selected && styles.selected]} aria-label={label + (active ? `, ${active.label} is ${active.paused ? 'paused' : 'working'} here` : '')}
        aria-pressed={selected} onClick={() => void b.execute('workspaces.switch', { workspaceId: ws.id })} {...hover}>
        <WorkspaceIcon ws={ws} />
        {active && !active.paused && <span css={styles.dot} data-testid="activity-dot" />}
      </button>
    </li>
  );
}

export function WorkspaceDock({ workspaces, activeId, agents }: { workspaces: WorkspaceView[]; activeId: string; agents: AgentView[] }) {
  const b = useBridge();
  const current = workspaces.find((w) => w.id === activeId);
  const dl = useHoverLabel('downloads', 'Downloads — not available in this build yet', (el) => offsetInSidebar(el).top - 68);
  const nw = useHoverLabel('new-ws', 'Create a new workspace', (el) => offsetInSidebar(el).top - 68);
  return (
    <footer css={styles.footer} data-figma="12:290">
      <div css={styles.heading}>
        <h2 css={styles.overline}>Workspaces</h2>
        <span css={styles.current} aria-live="polite">{current?.name}</span>
      </div>
      <div css={styles.controls}>
        <button type="button" css={styles.btn} aria-label="Downloads" aria-disabled="true" {...dl}>
          <Glyph src={UI_ICONS.downloads} size={16} />
        </button>
        <ul css={styles.switcher} aria-label="Workspaces">
          {workspaces.map((w) => (
            <SwitchButton key={w.id} ws={w} selected={w.id === activeId} active={agents.find((a) => a.workspaceIds.includes(w.id) && (a.working || a.paused))} />
          ))}
        </ul>
        <button type="button" css={styles.btn} aria-label="Create new workspace" {...nw}
          onClick={async () => {
            const r = (await b.execute('workspaces.create', { name: 'New workspace' })) as { workspace?: { id: string } };
            if (r?.workspace) void b.execute('workspaces.switch', { workspaceId: r.workspace.id });
          }}>
          <Glyph src={UI_ICONS['new-workspace']} size={16} />
        </button>
      </div>
    </footer>
  );
}
