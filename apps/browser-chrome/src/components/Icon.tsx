/** @jsxRuntime classic */
/** @jsx jsx */
import { cssMap, jsx } from '@compiled/react';

const styles = cssMap({
  mask: {
    display: 'inline-block',
    flexShrink: 0,
    backgroundColor: 'currentColor',
    WebkitMaskRepeat: 'no-repeat',
    maskRepeat: 'no-repeat',
    WebkitMaskPosition: 'center',
    maskPosition: 'center',
    WebkitMaskSize: 'contain',
    maskSize: 'contain',
  },
  img: { display: 'block', flexShrink: 0, objectFit: 'contain' },
});

/** Monochrome UI glyph (Figma export) tinted with `currentColor` via mask. */
export function Glyph({ src, size, color }: { src: string; size: number; color?: string }) {
  return (
    <span
      aria-hidden
      css={styles.mask}
      style={{ width: size, height: size, WebkitMaskImage: `url("${src}")`, maskImage: `url("${src}")`, color }}
    />
  );
}

/** Full-colour brand/site image. */
export function Img({ src, width, height, onError }: { src: string; width: number; height: number; onError?: () => void }) {
  return <img alt="" aria-hidden src={src} width={width} height={height} css={styles.img} draggable={false} onError={onError} />;
}
