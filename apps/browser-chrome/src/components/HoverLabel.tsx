/** @jsxRuntime classic */
/** @jsx jsx */
// Figma component "Slide / Tooltip" (30:122): a sidebar-wide dark label that
// distinguishes website launchers from workspace switchers. ADS Tooltip
// anchors to the trigger centre, which does not match this geometry, so the
// label is composed from tokens; semantics follow ADS (role=tooltip,
// aria-describedby, hover/focus delay, Escape dismiss).
import { cssMap, jsx } from '@compiled/react';
import { token } from '@atlaskit/tokens';
import { createContext, useCallback, useContext, useEffect, useRef, useState } from 'react';

export type LabelAnchor = { text: string; top: number; key: string };
const Ctx = createContext<{ show: (a: LabelAnchor) => void; hide: (key: string) => void } | null>(null);

const styles = cssMap({
  label: {
    position: 'absolute',
    insetInlineStart: token('space.100'),
    insetInlineEnd: token('space.100'),
    height: '32px',
    display: 'flex',
    alignItems: 'center',
    paddingInline: '10px',
    borderRadius: token('radius.large'),
    backgroundColor: token('color.background.neutral.bold'),
    color: token('color.text.inverse'),
    font: 'var(--slide-font-field)',
    pointerEvents: 'none',
    zIndex: 10,
    whiteSpace: 'nowrap',
    overflow: 'hidden',
    textOverflow: 'ellipsis',
  },
});

export const HOVER_LABEL_ID = 'slide-hover-label';

export function HoverLabelHost({ children }: { children: React.ReactNode }) {
  const [anchor, setAnchor] = useState<LabelAnchor | null>(null);
  const timer = useRef<number | undefined>(undefined);
  const show = useCallback((a: LabelAnchor) => {
    window.clearTimeout(timer.current);
    timer.current = window.setTimeout(() => setAnchor(a), 300);
  }, []);
  const hide = useCallback((key: string) => {
    window.clearTimeout(timer.current);
    setAnchor((cur) => (cur && cur.key === key ? null : cur));
  }, []);
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => { if (e.key === 'Escape') setAnchor(null); };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, []);
  return (
    <Ctx.Provider value={{ show, hide }}>
      {children}
      {anchor && (
        <div id={HOVER_LABEL_ID} role="tooltip" css={styles.label} style={{ top: anchor.top }} data-testid="hover-label">
          {anchor.text}
        </div>
      )}
    </Ctx.Provider>
  );
}

/** Props to spread on a trigger. `place` returns the label's top (px, sidebar coordinates). */
export function useHoverLabel(key: string, text: string, place: (el: HTMLElement) => number) {
  const ctx = useContext(Ctx);
  const on = (e: React.SyntheticEvent<HTMLElement>) => ctx?.show({ key, text, top: place(e.currentTarget) });
  const off = () => ctx?.hide(key);
  return { onMouseEnter: on, onFocus: on, onMouseLeave: off, onBlur: off, 'aria-describedby': HOVER_LABEL_ID };
}

export function offsetInSidebar(el: HTMLElement): { top: number; bottom: number } {
  const host = el.closest('[data-sidebar]') as HTMLElement | null;
  const r = el.getBoundingClientRect();
  const h = host?.getBoundingClientRect() ?? { top: 0 };
  return { top: r.top - h.top, bottom: r.bottom - h.top };
}
