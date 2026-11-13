/** @jsxRuntime classic */
/** @jsx jsx */
// Conditional agent activity card (Figma 35:313). Rendered only when the core
// reports an authenticated agent that is working or paused; an icon in the
// dock never implies a connection. Pause/Resume call the real core command.
import { cssMap, jsx } from '@compiled/react';
import { token } from '@atlaskit/tokens';
import { useBridge } from '../bridge/bridge';
import type { AgentView } from '../bridge/types';
import { UI_ICONS, WORKSPACE_ICONS } from '../assets/icons';
import { Glyph, Img } from './Icon';

const styles = cssMap({
  card: {
    width: '100%', height: '56px', boxSizing: 'border-box', flexShrink: 0, display: 'flex', alignItems: 'center', gap: token('space.100'),
    padding: token('space.100'), paddingInlineStart: '9px', borderRadius: '10px', backgroundColor: token('elevation.surface'),
    boxShadow: `inset 0 0 0 1px ${token('color.border.bold')}`, color: token('color.text'),
  },
  text: { flexGrow: 1, minWidth: 0, display: 'flex', flexDirection: 'column', gap: token('space.025') },
  title: { font: 'var(--slide-font-section-title)', whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' },
  sub: { font: 'var(--slide-font-caption)', color: token('color.text.subtle'), opacity: 0.72, whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' },
  pause: {
    width: '24px', height: '24px', flexShrink: 0, display: 'grid', placeItems: 'center', border: 'none', padding: 0, margin: 0,
    borderRadius: token('radius.medium'), backgroundColor: token('elevation.surface'), color: token('color.icon'), cursor: 'pointer',
    '&:hover': { backgroundColor: token('color.background.neutral.subtle.hovered') },
    '&:focus-visible': { outline: `2px solid ${token('color.border.focused')}`, outlineOffset: '-2px' },
  },
});

const AGENT_ICON: Record<string, string> = { 'claude-code': WORKSPACE_ICONS['claude-code-activity'], codex: WORKSPACE_ICONS.codex, chatgpt: WORKSPACE_ICONS.chatgpt };

export function AgentActivitySummary({ agent }: { agent: AgentView }) {
  const b = useBridge();
  const state = agent.paused ? 'Paused' : 'Working';
  const icon = AGENT_ICON[agent.client];
  return (
    <section css={styles.card} aria-label={`${agent.label} activity`} aria-live="polite" data-figma="35:313">
      {icon ? <Img src={icon} width={16} height={16} /> : <span style={{ width: 16 }} />}
      <div css={styles.text}>
        <span css={styles.title}>{agent.label} · {state}</span>
        <span css={styles.sub}>{agent.tabCount} {agent.tabCount === 1 ? 'tab' : 'tabs'} in its workspace</span>
      </div>
      <button type="button" css={styles.pause} aria-pressed={agent.paused} aria-label={agent.paused ? `Resume ${agent.label}` : `Pause ${agent.label}`}
        onClick={() => void b.execute(agent.paused ? 'agents.resume' : 'agents.pause', { grantId: agent.grantId })}>
        <Glyph src={UI_ICONS.pause} size={14} />
      </button>
    </section>
  );
}
