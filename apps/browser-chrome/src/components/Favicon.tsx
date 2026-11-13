/** @jsxRuntime classic */
/** @jsx jsx */
import { cssMap, jsx } from '@compiled/react';
import { token } from '@atlaskit/tokens';
import GlobeIcon from '@atlaskit/icon/core/globe';
import Spinner from '@atlaskit/spinner';
import { createContext, useContext, useState } from 'react';
import { Img } from './Icon';

/**
 * Resolves icon references to image URLs. Production: favicons WebKit loaded
 * for the site, served by the native host's `justslide-favicon:` scheme.
 * Preview: the design fixture's Figma-exported brand icons.
 */
export interface IconResolver {
  tab(faviconUrl: string | null | undefined, pageUrl: string): string | null;
  favorite(id: string, url: string): string | null;
}

export const nativeResolver: IconResolver = {
  tab: (fav) => (fav && /^https?:/.test(fav) ? `justslide-favicon://icon/${encodeURIComponent(fav)}` : null),
  favorite: (_id, url) => { try { return `justslide-favicon://site/${new URL(url).host}`; } catch { return null; } },
};

export const IconResolverContext = createContext<IconResolver>(nativeResolver);

const styles = cssMap({
  fallback: { display: 'grid', placeItems: 'center', flexShrink: 0, color: token('color.icon.subtle') },
});

export function SiteIcon({ src, size, height, loading }: { src: string | null; size: number; height?: number; loading?: boolean }) {
  const [broken, setBroken] = useState(false);
  if (loading) return <span css={styles.fallback} style={{ width: size, height: size }}><Spinner size="small" label="Loading" /></span>;
  if (!src || broken) return <span css={styles.fallback} style={{ width: size, height: size }}><GlobeIcon label="" size="small" /></span>;
  return <Img src={src} width={size} height={height ?? size} onError={() => setBroken(true)} />;
}

export function useIcons() { return useContext(IconResolverContext); }
