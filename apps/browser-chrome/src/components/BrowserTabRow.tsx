/** @jsxRuntime classic */
/** @jsx jsx */
// Figma component set "Slide / Tab" (30:100): Default, Hover (white, close
// visible), Selected (white, 1px ink border, 590 weight, close visible),
// Keyboard focus (white, 2px focus border). Pinned rows (25:4) share it.
import { cssMap, jsx } from '@compiled/react';
import { token } from '@atlaskit/tokens';
import DropdownMenu, { DropdownItem, DropdownItemGroup } from '@atlaskit/dropdown-menu';
import { useState } from 'react';
import { useBridge } from '../bridge/bridge';
import type { TabView, WorkspaceView } from '../bridge/types';
import { UI_ICONS } from '../assets/icons';
import { Glyph } from './Icon';
import { SiteIcon, useIcons } from './Favicon';

const styles = cssMap({
  row: {
    position: 'relative', height: '32px', width: '100%', boxSizing: 'border-box', flexShrink: 0, display: 'flex', alignItems: 'center', margin: 0,
    borderRadius: token('radius.large'), color: token('color.text'),
    '&:hover': { backgroundColor: token('elevation.surface') },
    '&:hover [data-close]': { visibility: 'visible' },
    '&:hover [data-main]': { paddingInlineEnd: '40px' },
  },
  selected: {
    backgroundColor: token('elevation.surface'), boxShadow: `inset 0 0 0 1px ${token('color.border.bold')}`,
    '& [data-close]': { visibility: 'visible', insetInlineEnd: '9px' },
    '& [data-main]': { paddingInlineEnd: '41px' },
  },
  main: {
    flexGrow: 1, minWidth: 0, height: '100%', display: 'flex', alignItems: 'center', gap: token('space.100'), border: 'none', margin: 0,
    paddingInlineStart: token('space.100'), paddingInlineEnd: token('space.100'), backgroundColor: 'transparent', color: 'inherit',
    borderRadius: token('radius.large'), cursor: 'pointer', textAlign: 'start', font: 'var(--slide-font-row)',
    '&:focus-visible': { outline: 'none', backgroundColor: token('elevation.surface'), boxShadow: `inset 0 0 0 2px ${token('color.border.focused')}` },
  },
  mainSelected: { font: 'var(--slide-font-row-strong)', paddingInlineStart: '9px' },
  title: { flexGrow: 1, minWidth: 0, overflow: 'hidden', whiteSpace: 'nowrap', textOverflow: 'ellipsis' },
  close: {
    visibility: 'hidden', position: 'absolute', insetInlineEnd: '8px', top: '4px', width: '24px', height: '24px', display: 'grid', placeItems: 'center',
    border: 'none', padding: 0, margin: 0, borderRadius: token('radius.medium'), backgroundColor: 'transparent', color: 'var(--slide-icon-close, currentColor)', cursor: 'pointer',
    '&:hover': { backgroundColor: token('color.background.neutral.subtle.hovered') },
    '&:focus-visible': { visibility: 'visible', outline: `2px solid ${token('color.border.focused')}`, outlineOffset: '-2px' },
  },
  status: { position: 'absolute', width: 1, height: 1, overflow: 'hidden', clip: 'rect(0 0 0 0)' },
  menuAnchor: { position: 'absolute', insetInlineEnd: 0, bottom: 0, width: 0, height: 0 },
});

export function BrowserTabRow({ tab, workspaces, figmaId }: { tab: TabView; workspaces: WorkspaceView[]; figmaId?: string }) {
  const b = useBridge();
  const icons = useIcons();
  const label = tab.title || tab.url;
  const status = tab.crashed ? 'page crashed' : tab.loadState === 'failed' ? 'failed to load' : tab.loading ? 'loading' : tab.loadState === 'notLoaded' ? 'not loaded' : '';
  const selected = tab.active;
  const iconSize = selected ? 16 : 14;
  const others = workspaces.filter((w) => w.id !== tab.workspaceId);
  const [menuOpen, setMenuOpen] = useState(false);
  return (
    <li css={[styles.row, selected && styles.selected]} onContextMenu={(e) => { e.preventDefault(); setMenuOpen(true); }} data-tab-id={tab.id} data-selected={selected || undefined} data-figma={figmaId}>
      <button
        type="button"
        data-main
        aria-haspopup="menu"
        css={[styles.main, selected && styles.mainSelected]}
        aria-current={selected ? 'page' : undefined}
        aria-label={`${label}${status ? `, ${status}` : ''}${tab.audible ? ', playing audio' : ''}${tab.openedBy === 'agent' ? ', opened by an agent' : ''}`}
        onClick={() => void b.execute('tabs.activate', { tabId: tab.id })}
        onAuxClick={(e) => { if (e.button === 1) void b.execute('tabs.close', { tabId: tab.id }); }}
        onKeyDown={(e) => {
          if (e.altKey && (e.key === 'ArrowUp' || e.key === 'ArrowDown')) {
            e.preventDefault();
            void b.execute('tabs.reorder', { tabId: tab.id, index: Math.max(0, tab.order + (e.key === 'ArrowUp' ? -1 : 1)) });
          }
          if ((e.metaKey && e.key === 'w') || e.key === 'Delete') { e.preventDefault(); void b.execute('tabs.close', { tabId: tab.id }); }
          if (e.key === 'ContextMenu' || (e.shiftKey && e.key === 'F10')) { e.preventDefault(); setMenuOpen(true); }
        }}
      >
        <SiteIcon src={icons.tab(tab.faviconUrl, tab.url)} size={iconSize} loading={tab.loading && !tab.faviconUrl} />
        <span css={styles.title}>{label}</span>
      </button>
      <button type="button" data-close css={styles.close} aria-label={`Close ${label}`} onClick={() => void b.execute('tabs.close', { tabId: tab.id })}>
        <Glyph src={UI_ICONS.close} size={12} />
      </button>
      <span css={styles.menuAnchor}>
        <DropdownMenu
          shouldRenderToParent
          isOpen={menuOpen}
          onOpenChange={({ isOpen }) => setMenuOpen(isOpen)}
          placement="bottom-end"
          trigger={({ triggerRef }) => <span ref={triggerRef as React.Ref<HTMLSpanElement>} aria-hidden />}
        >
          <DropdownItemGroup>
            <DropdownItem onClick={() => void b.execute('tabs.pin', { tabId: tab.id, pinned: !tab.pinned })}>{tab.pinned ? 'Unpin tab' : 'Pin tab'}</DropdownItem>
            <DropdownItem onClick={() => void b.execute('tabs.duplicate', { tabId: tab.id })}>Duplicate tab</DropdownItem>
            {others.map((w) => (
              <DropdownItem key={w.id} onClick={() => void b.execute('tabs.move', { tabId: tab.id, workspaceId: w.id })}>Move to {w.name}</DropdownItem>
            ))}
            <DropdownItem onClick={() => void b.execute('tabs.close', { tabId: tab.id })}>Close tab</DropdownItem>
          </DropdownItemGroup>
        </DropdownMenu>
      </span>
      {status && <span css={styles.status} role="status">{status}</span>}
    </li>
  );
}
