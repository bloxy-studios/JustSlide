/** @jsxRuntime classic */
/** @jsx jsx */
// The rounded page window (Figma 1:7). In the macOS app the real page is a
// separate, unprivileged WKWebView that the native host positions over this
// rectangle (reported via the bridge). React never touches native views and
// never fakes page content; it renders only chrome-owned states.
import { cssMap, jsx } from '@compiled/react';
import { token } from '@atlaskit/tokens';
import { useLayoutEffect, useRef } from 'react';
import { useBridge } from '../bridge/bridge';
import type { TabView } from '../bridge/types';

const styles = cssMap({
  panel: {
    position: 'relative', flexGrow: 1, minWidth: 0, height: '100%', boxSizing: 'border-box', borderRadius: token('radius.xxlarge'),
    backgroundColor: token('elevation.surface'), boxShadow: `inset 0 0 0 1px ${token('elevation.surface')}`, overflow: 'hidden',
  },
  center: { position: 'absolute', inset: 0, display: 'grid', placeItems: 'center', color: token('color.text'), font: 'var(--slide-font-row)', textAlign: 'center', padding: token('space.300') },
  stack: { display: 'flex', flexDirection: 'column', gap: token('space.100'), alignItems: 'center', maxWidth: '420px' },
  h: { font: token('font.heading.small'), margin: 0 },
  sub: { color: token('color.text.subtle'), font: 'var(--slide-font-caption)', margin: 0 },
  btn: {
    marginTop: token('space.100'), height: '32px', paddingInline: token('space.150'), border: 'none', borderRadius: token('radius.large'),
    backgroundColor: token('color.background.neutral'), color: token('color.text'), font: 'var(--slide-font-row-strong)', cursor: 'pointer',
    '&:hover': { backgroundColor: token('color.background.neutral.hovered') },
    '&:focus-visible': { outline: `2px solid ${token('color.border.focused')}`, outlineOffset: '2px' },
  },
});

export function WebContentHostAnchor({ tab, preview }: { tab?: TabView; preview: boolean }) {
  const b = useBridge();
  const ref = useRef<HTMLDivElement>(null);
  useLayoutEffect(() => {
    const el = ref.current;
    if (!el) return;
    const report = () => {
      const r = el.getBoundingClientRect();
      b.reportContentBounds({ x: r.left, y: r.top, width: r.width, height: r.height, radius: 16 });
    };
    report();
    const ro = new ResizeObserver(report);
    ro.observe(el);
    return () => ro.disconnect();
  }, [b]);
  let overlay: React.ReactNode = null;
  if (!tab || tab.url === 'about:blank') {
    overlay = (
      <div css={styles.center}><div css={styles.stack}>
        <p css={styles.h}>New Tab</p>
        <p css={styles.sub}>Type an address or search in the bar on the left.</p>
      </div></div>
    );
  } else if (!preview && (tab.crashed || tab.loadState === 'failed')) {
    overlay = (
      <div css={styles.center} role="alert"><div css={styles.stack}>
        <p css={styles.h}>{tab.crashed ? 'This page stopped working' : 'This page couldn’t load'}</p>
        {tab.error && <p css={styles.sub}>{tab.error}</p>}
        <button type="button" css={styles.btn} onClick={() => void b.execute('page.reload', { tabId: tab.id })}>Reload</button>
      </div></div>
    );
  }
  return (
    <main ref={ref} css={styles.panel} aria-label="Web page" data-figma="1:7" data-page-host data-tab-id={tab?.id}>
      {overlay}
    </main>
  );
}
