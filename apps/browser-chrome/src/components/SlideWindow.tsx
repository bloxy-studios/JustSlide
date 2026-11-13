/** @jsxRuntime classic */
/** @jsx jsx */
import { cssMap, jsx } from '@compiled/react';
import { token } from '@atlaskit/tokens';
import { useEffect } from 'react';
import { useSnapshot } from '../bridge/bridge';
import { applyAppearance } from '../theme/slide-theme';
import { NavigationControls } from './NavigationControls';
import { AddressSearchBar } from './AddressSearchBar';
import { FavoriteGrid } from './FavoriteGrid';
import { BrowserSection, PinnedSection, SectionSeparator } from './TabSections';
import { AgentActivitySummary } from './AgentActivitySummary';
import { WorkspaceDock } from './WorkspaceDock';
import { WebContentHostAnchor } from './WebContentHostAnchor';
import { HoverLabelHost } from './HoverLabel';

const styles = cssMap({
  window: {
    position: 'fixed', inset: 0, display: 'flex', gap: token('space.150'), padding: token('space.150'), boxSizing: 'border-box',
    backgroundColor: 'var(--slide-backdrop)', color: token('color.text'), overflow: 'hidden',
  },
  sidebar: {
    position: 'relative', width: '240px', flexShrink: 0, height: '100%', display: 'flex', flexDirection: 'column', alignItems: 'center',
    gap: token('space.100'), minHeight: 0,
  },
  rail: { width: '40px' },
});

export function SlideWindow({ preview = false }: { preview?: boolean }) {
  const s = useSnapshot();
  useEffect(() => {
    if (s) applyAppearance(document.documentElement, s.settings.theme, s.settings.backdrop);
  }, [s?.settings.theme, s?.settings.backdrop]);
  if (!s) return null;
  const ws = s.window.activeWorkspaceId;
  const workspace = s.workspaces.find((w) => w.id === ws);
  const tabs = s.tabs.filter((t) => t.workspaceId === ws);
  const pinned = tabs.filter((t) => t.pinned).sort((a, b) => a.order - b.order);
  const regular = tabs.filter((t) => !t.pinned).sort((a, b) => a.order - b.order);
  const active = tabs.find((t) => t.active);
  const agent = s.agents.find((a) => a.working || a.paused);
  const collapsed = s.settings.sidebarCollapsed;
  return (
    <div css={styles.window} data-figma={s.fixture?.variant === 'compact' ? '35:13' : '1:6'} data-fixture={s.fixture ? s.fixture.variant : undefined}>
      {collapsed ? (
        <nav css={[styles.sidebar, styles.rail]} aria-label="Browser sidebar (collapsed)" data-sidebar>
          <NavigationControls tab={active} sidebarCollapsed />
        </nav>
      ) : (
        <HoverLabelHost>
          <nav css={styles.sidebar} aria-label="Browser sidebar" data-sidebar data-figma="1:9">
            <NavigationControls tab={active} sidebarCollapsed={false} />
            <AddressSearchBar tab={active} />
            <FavoriteGrid favorites={s.favorites} />
            <PinnedSection tabs={pinned} collapsed={!!workspace?.pinnedCollapsed} workspaces={s.workspaces} />
            <SectionSeparator />
            <BrowserSection tabs={regular} workspaces={s.workspaces} />
            {agent && <AgentActivitySummary agent={agent} />}
            <WorkspaceDock workspaces={s.workspaces} activeId={ws} agents={s.agents} />
          </nav>
        </HoverLabelHost>
      )}
      <WebContentHostAnchor tab={active} preview={preview} />
    </div>
  );
}
