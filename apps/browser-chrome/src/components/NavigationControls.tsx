/** @jsxRuntime classic */
/** @jsx jsx */
import { cssMap, jsx } from '@compiled/react';
import { token } from '@atlaskit/tokens';
import { UI_ICONS } from '../assets/icons';
import { Glyph } from './Icon';
import type { TabView } from '../bridge/types';
import { useBridge } from '../bridge/bridge';

const styles = cssMap({
  row: {
    height: '40px', width: '100%', boxSizing: 'border-box', display: 'flex', alignItems: 'center', justifyContent: 'space-between',
    paddingInline: token('space.100'), flexShrink: 0,
  },
  history: { display: 'flex', alignItems: 'center', gap: token('space.050') },
  btn: {
    width: '32px', height: '32px', display: 'grid', placeItems: 'center', border: 'none', padding: 0, margin: 0, cursor: 'pointer',
    borderRadius: token('radius.large'), backgroundColor: 'transparent', color: token('color.icon'),
    '&:hover': { backgroundColor: token('color.background.neutral.subtle.hovered') },
    '&:active': { backgroundColor: token('color.background.neutral.subtle.pressed') },
    '&:focus-visible': { outline: `2px solid ${token('color.border.focused')}`, outlineOffset: '-2px' },
    '&:disabled': { cursor: 'default', opacity: 0.38, backgroundColor: 'transparent' },
  },
});

export function NavigationControls({ tab, sidebarCollapsed }: { tab?: TabView; sidebarCollapsed: boolean }) {
  const b = useBridge();
  const loading = tab?.loading ?? false;
  const go = (command: 'page.back' | 'page.forward' | 'page.reload' | 'page.stop') => tab && void b.execute(command, { tabId: tab.id });
  return (
    <div css={styles.row} data-figma="6:17">
      <button type="button" css={styles.btn} aria-label={sidebarCollapsed ? 'Show sidebar' : 'Hide sidebar'} aria-pressed={sidebarCollapsed}
        onClick={() => void b.execute('settings.update', { sidebarCollapsed: !sidebarCollapsed })}>
        <Glyph src={UI_ICONS['sidebar-toggle']} size={16} />
      </button>
      <div css={styles.history}>
        <button type="button" css={styles.btn} aria-label="Back" disabled={!tab?.canGoBack} onClick={() => go('page.back')}>
          <Glyph src={UI_ICONS.back} size={16} />
        </button>
        <button type="button" css={styles.btn} aria-label="Forward" disabled={!tab?.canGoForward} onClick={() => go('page.forward')}>
          <Glyph src={UI_ICONS.forward} size={16} />
        </button>
        <button type="button" css={styles.btn} aria-label={loading ? 'Stop loading' : 'Reload'} disabled={!tab} onClick={() => go(loading ? 'page.stop' : 'page.reload')}>
          <Glyph src={loading ? UI_ICONS.close : UI_ICONS.reload} size={loading ? 12 : 16} />
        </button>
      </div>
    </div>
  );
}
