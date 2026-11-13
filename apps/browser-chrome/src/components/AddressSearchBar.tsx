/** @jsxRuntime classic */
/** @jsx jsx */
import { cssMap, jsx } from '@compiled/react';
import { token } from '@atlaskit/tokens';
import { useEffect, useRef, useState } from 'react';
import { useBridge } from '../bridge/bridge';
import type { TabView } from '../bridge/types';

const styles = cssMap({
  wrap: {
    width: '100%', height: '32px', boxSizing: 'border-box', flexShrink: 0, display: 'flex', alignItems: 'center', gap: token('space.050'),
    paddingInline: token('space.100'), borderRadius: token('radius.large'), backgroundColor: token('elevation.surface'),
    '&:focus-within': { boxShadow: `inset 0 0 0 2px ${token('color.border.focused')}` },
  },
  input: {
    flexGrow: 1, minWidth: 0, height: '18px', border: 'none', outline: 'none', padding: 0, margin: 0, backgroundColor: 'transparent',
    color: token('color.text'), font: 'var(--slide-font-field)',
    '&::placeholder': { color: token('color.text.subtlest') },
  },
  insecure: { color: token('color.text.subtle'), font: 'var(--slide-font-caption)', flexShrink: 0 },
});

/** Human-readable host shown at rest ("figma.com"), full URL while editing. */
export function displayUrl(url: string): string {
  try {
    const u = new URL(url);
    if (u.protocol === 'about:') return '';
    return u.host.replace(/^www\./, '');
  } catch {
    return url;
  }
}

export function AddressSearchBar({ tab }: { tab?: TabView }) {
  const b = useBridge();
  const [editing, setEditing] = useState(false);
  const [text, setText] = useState('');
  const ref = useRef<HTMLInputElement>(null);
  // Never overwrite what the person is typing on navigation events.
  useEffect(() => { if (!editing) setText(tab ? displayUrl(tab.url) : ''); }, [tab?.url, tab?.id, editing]);
  const submit = () => {
    const input = text.trim();
    if (!input) return;
    if (tab) void b.execute('page.navigate', { tabId: tab.id, url: input });
    else void b.execute('tabs.open', { input });
    setEditing(false);
    ref.current?.blur();
  };
  const insecure = tab && tab.url.startsWith('http://') && tab.loadState !== 'notLoaded';
  return (
    <div css={styles.wrap} data-figma="7:46">
      {insecure && <span css={styles.insecure}>Not secure</span>}
      <input
        ref={ref}
        css={styles.input}
        aria-label="Address and search"
        aria-busy={tab?.loading || undefined}
        placeholder="Search or enter address"
        spellCheck={false}
        autoCapitalize="off"
        autoCorrect="off"
        value={text}
        onFocus={(e) => { setEditing(true); setText(tab && tab.url !== 'about:blank' ? tab.url : ''); requestAnimationFrame(() => e.target.select()); }}
        onBlur={() => setEditing(false)}
        onChange={(e) => setText(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === 'Enter') submit();
          if (e.key === 'Escape') { setEditing(false); setText(tab ? displayUrl(tab.url) : ''); ref.current?.blur(); }
        }}
      />
    </div>
  );
}
