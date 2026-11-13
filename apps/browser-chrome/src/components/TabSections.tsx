/** @jsxRuntime classic */
/** @jsx jsx */
// Pinned section (25:4), separator (12:139), New Tab (12:140) and the
// independently scrolling tab list (12:289).
import { cssMap, jsx } from '@compiled/react';
import { token } from '@atlaskit/tokens';
import { useBridge } from '../bridge/bridge';
import type { TabView, WorkspaceView } from '../bridge/types';
import { UI_ICONS } from '../assets/icons';
import { Glyph } from './Icon';
import { BrowserTabRow } from './BrowserTabRow';

const styles = cssMap({
  pinned: { width: '100%', display: 'flex', flexDirection: 'column', gap: token('space.050'), flexShrink: 0 },
  trigger: {
    height: '28px', width: '100%', boxSizing: 'border-box', display: 'flex', alignItems: 'center', gap: token('space.100'),
    paddingInline: token('space.100'), border: 'none', margin: 0, borderRadius: token('radius.large'), backgroundColor: 'transparent',
    color: token('color.text'), cursor: 'pointer', textAlign: 'start',
    '&:hover': { backgroundColor: token('color.background.neutral.subtle.hovered') },
    '&:focus-visible': { outline: 'none', boxShadow: `inset 0 0 0 2px ${token('color.border.focused')}` },
  },
  triggerTitle: { flexGrow: 1, font: 'var(--slide-font-section-title)' },
  count: { font: 'var(--slide-font-count)', color: token('color.text.subtle'), opacity: 0.6 },
  list: { margin: 0, padding: 0, listStyle: 'none', display: 'flex', flexDirection: 'column', gap: token('space.050') },
  separator: { width: 'calc(100% - 16px)', height: 0, margin: 0, border: 'none', borderTop: `1px solid ${token('color.border')}`, marginBottom: '-1px', flexShrink: 0 },
  browser: { width: '100%', flexGrow: 1, minHeight: 0, display: 'flex', flexDirection: 'column', gap: token('space.100') },
  newTab: {
    height: '32px', width: '100%', boxSizing: 'border-box', flexShrink: 0, display: 'flex', alignItems: 'center', gap: token('space.100'),
    paddingInline: token('space.100'), border: 'none', margin: 0, borderRadius: token('radius.large'), backgroundColor: 'transparent',
    color: token('color.text'), font: 'var(--slide-font-row-strong)', cursor: 'pointer', textAlign: 'start',
    '&:hover': { backgroundColor: token('color.background.neutral.subtle.hovered') },
    '&:focus-visible': { outline: 'none', boxShadow: `inset 0 0 0 2px ${token('color.border.focused')}` },
  },
  scroller: {
    flexGrow: 1, minHeight: 0, overflowY: 'auto', overflowX: 'hidden', overscrollBehavior: 'contain', scrollbarWidth: 'none',
    '&:hover, &:focus-within': { scrollbarWidth: 'thin' },
  },
});

export function PinnedSection({ tabs, collapsed, workspaces }: { tabs: TabView[]; collapsed: boolean; workspaces: WorkspaceView[] }) {
  const b = useBridge();
  return (
    <section css={styles.pinned} aria-label="Pinned tabs" data-figma="25:4">
      <button type="button" css={styles.trigger} aria-expanded={!collapsed} aria-controls="slide-pinned-list"
        onClick={() => void b.execute('ui.setPinnedCollapsed', { collapsed: !collapsed })}>
        <Glyph src={UI_ICONS.pin} size={12} />
        <span css={styles.triggerTitle}>Pinned</span>
        <span css={styles.count} aria-label={`${tabs.length} pinned`}>{tabs.length}</span>
        <Glyph src={collapsed ? UI_ICONS['chevron-collapsed'] : UI_ICONS['chevron-expanded']} size={14} />
      </button>
      {!collapsed && (
        <ul id="slide-pinned-list" css={styles.list}>
          {tabs.map((t) => <BrowserTabRow key={t.id} tab={t} workspaces={workspaces} />)}
        </ul>
      )}
    </section>
  );
}

export function SectionSeparator() {
  return <hr css={styles.separator} data-figma="12:139" />;
}

export function BrowserSection({ tabs, workspaces }: { tabs: TabView[]; workspaces: WorkspaceView[] }) {
  const b = useBridge();
  return (
    <div css={styles.browser} data-figma="20:82">
      <button type="button" css={styles.newTab} onClick={() => void b.execute('tabs.open', { input: 'about:blank' })} aria-keyshortcuts="Meta+T">
        <Glyph src={UI_ICONS['new-tab']} size={12} color="var(--ds-icon-subtle)" />
        New Tab
      </button>
      <div css={styles.scroller} data-figma="12:289" data-testid="tab-scroller">
        <ul css={styles.list} aria-label="Tabs">
          {tabs.map((t) => <BrowserTabRow key={t.id} tab={t} workspaces={workspaces} />)}
        </ul>
      </div>
    </div>
  );
}
