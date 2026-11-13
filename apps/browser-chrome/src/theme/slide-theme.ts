// Slide theme layer. Components only use ADS semantic tokens (`token()`);
// this module sets the *values* of a small set of ADS tokens on the chrome
// root so neutrals harmonise with the user-selected backdrop, exactly as the
// Figma file does (its inks are teal-tinted against #ACFFF7). With the Figma
// example backdrop the output equals the Figma hexes; other backdrops rotate
// hue/scale saturation. Dark mode keeps ADS dark values. See DESIGN.md.
import { setGlobalTheme } from '@atlaskit/tokens/set-global-theme';

export const FIGMA_EXAMPLE_BACKDROP = '#ACFFF7';

/** Figma-measured ink values at the example backdrop (file Slide, 1:6/35:13/30:52). */
const FIGMA_INKS: Record<string, string> = {
  '--ds-text': '#0E1B1B',
  '--ds-text-subtle': '#304B48',
  '--ds-text-subtlest': '#45615F',
  '--ds-border': '#627F87',
  '--ds-border-bold': '#0E1B1B',
  '--ds-border-focused': '#10574F',
  '--ds-background-neutral-bold': '#0E1B1B',
  '--ds-background-success-bold': '#128066',
};
/** Neutral (untinted) icon inks used by the Figma UI icons. */
const FIGMA_ICONS: Record<string, string> = { '--ds-icon': '#171717', '--ds-icon-subtle': '#393939', '--slide-icon-close': '#304B48' };

function hexToHsl(hex: string): [number, number, number] {
  const n = parseInt(hex.slice(1), 16);
  const r = ((n >> 16) & 255) / 255, g = ((n >> 8) & 255) / 255, b = (n & 255) / 255;
  const max = Math.max(r, g, b), min = Math.min(r, g, b), l = (max + min) / 2;
  if (max === min) return [0, 0, l];
  const d = max - min;
  const s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
  const h = max === r ? (g - b) / d + (g < b ? 6 : 0) : max === g ? (b - r) / d + 2 : (r - g) / d + 4;
  return [h * 60, s, l];
}

function hslToHex(h: number, s: number, l: number): string {
  const f = (n: number) => {
    const k = (n + h / 30) % 12;
    const a = s * Math.min(l, 1 - l);
    return Math.round(255 * (l - a * Math.max(-1, Math.min(k - 3, 9 - k, 1))));
  };
  return '#' + [f(0), f(8), f(4)].map((x) => x.toString(16).padStart(2, '0')).join('').toUpperCase();
}

export function tintFor(backdrop: string): Record<string, string> {
  const b = backdrop.toUpperCase();
  if (b === FIGMA_EXAMPLE_BACKDROP) return { ...FIGMA_INKS };
  const [bh, bs] = hexToHsl(b);
  const [fh, fs] = hexToHsl(FIGMA_EXAMPLE_BACKDROP);
  const out: Record<string, string> = {};
  for (const [k, v] of Object.entries(FIGMA_INKS)) {
    const [h, s, l] = hexToHsl(v);
    out[k] = hslToHex((h + bh - fh + 360) % 360, Math.min(1, s * (bs / (fs || 1))), l);
  }
  return out;
}

/** WCAG relative-luminance contrast ratio. */
export function contrast(a: string, b: string): number {
  const lum = (hex: string) => {
    const n = parseInt(hex.slice(1), 16);
    const c = [(n >> 16) & 255, (n >> 8) & 255, n & 255].map((v) => { const x = v / 255; return x <= 0.03928 ? x / 12.92 : ((x + 0.055) / 1.055) ** 2.4; });
    return 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
  };
  const [x, y] = [lum(a), lum(b)].sort((p, q) => q - p);
  return (x + 0.05) / (y + 0.05);
}

let appliedMode: string | null = null;
export function applyAppearance(root: HTMLElement, theme: 'light' | 'dark' | 'system', backdrop: string) {
  const mode = theme === 'system' ? (matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light') : theme;
  if (appliedMode !== theme) {
    appliedMode = theme;
    void setGlobalTheme({ light: 'light', dark: 'dark', colorMode: theme === 'system' ? 'auto' : theme });
  }
  root.style.setProperty('--slide-backdrop', backdrop);
  const vars = mode === 'light' ? { ...tintFor(backdrop), ...FIGMA_ICONS } : {};
  for (const k of [...Object.keys(FIGMA_INKS), ...Object.keys(FIGMA_ICONS)]) root.style.removeProperty(k);
  for (const [k, v] of Object.entries(vars)) root.style.setProperty(k, v);
  root.dataset.colorMode = mode;
}
