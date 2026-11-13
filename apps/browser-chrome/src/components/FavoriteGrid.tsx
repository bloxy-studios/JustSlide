/** @jsxRuntime classic */
/** @jsx jsx */
// Website favorites (Figma 7:87): 5-column grid of 43×34 white tiles. These
// are WEBSITE launchers — distinct from the workspace dock at the bottom.
import { cssMap, jsx } from '@compiled/react';
import { token } from '@atlaskit/tokens';
import { useBridge } from '../bridge/bridge';
import type { Favorite } from '../bridge/types';
import { UI_ICONS } from '../assets/icons';
import { Glyph } from './Icon';
import { SiteIcon, useIcons } from './Favicon';
import { offsetInSidebar, useHoverLabel } from './HoverLabel';

const styles = cssMap({
  grid: {
    width: '100%', boxSizing: 'border-box', padding: token('space.050'), display: 'grid', gridTemplateColumns: 'repeat(5, 43px)',
    gridAutoRows: '34px', gap: token('space.050'), flexShrink: 0, margin: 0, listStyle: 'none',
  },
  tile: {
    width: '43px', height: '34px', boxSizing: 'border-box', display: 'grid', placeItems: 'center', border: 'none', padding: 0, margin: 0,
    borderRadius: token('radius.large'), backgroundColor: token('elevation.surface'), cursor: 'pointer',
    '&:hover': { boxShadow: `inset 0 0 0 1px ${token('color.border')}` },
    '&:active': { transform: 'scale(0.97)' },
    '&:focus-visible': { outline: 'none', boxShadow: `inset 0 0 0 2px ${token('color.border.focused')}` },
  },
  add: { border: `1px dashed ${token('color.icon.subtle')}`, color: token('color.icon.subtle') },
});

/** Figma icon box sizes for the reference favorites (some marks are not square). */
const ICON_BOX: Record<string, [number, number]> = { youtube: [18, 13], discord: [18, 14], netflix: [10, 18], notion: [18, 19], supabase: [18, 19], convex: [18, 19] };

function Tile({ fav, index }: { fav: Favorite; index: number }) {
  const b = useBridge();
  const icons = useIcons();
  const key = fav.id.replace(/^fav_/, '');
  const [w, h] = ICON_BOX[key] ?? [18, 18];
  const label = useHoverLabel(`fav-${fav.id}`, `Open ${fav.title} website`, (el) => offsetInSidebar(el).bottom + 4);
  return (
    <li style={{ display: 'contents' }}>
      <button type="button" css={styles.tile} aria-label={`Open ${fav.title} website`} data-index={index}
        onClick={() => void b.execute('tabs.open', { input: fav.url })} {...label}>
        <SiteIcon src={icons.favorite(fav.id, fav.url)} size={w} height={h} />
      </button>
    </li>
  );
}

export function FavoriteGrid({ favorites }: { favorites: Favorite[] }) {
  const addLabel = useHoverLabel('fav-add', 'Add website favorite', (el) => offsetInSidebar(el).bottom + 4);
  const b = useBridge();
  const snapshotTab = () => {
    const s = b.getSnapshot();
    const t = s?.tabs.find((x) => x.active);
    if (t && /^https?:/.test(t.url)) void b.execute('favorites.add', { url: t.url, title: t.title });
  };
  return (
    <ul css={styles.grid} aria-label="Website favorites" data-figma="7:87">
      {favorites.map((f, i) => <Tile key={f.id} fav={f} index={i} />)}
      <li style={{ display: 'contents' }}>
        <button type="button" css={[styles.tile, styles.add]} aria-label="Add website favorite" onClick={snapshotTab} {...addLabel}>
          <Glyph src={UI_ICONS['add-favorite']} size={18} />
        </button>
      </li>
    </ul>
  );
}
