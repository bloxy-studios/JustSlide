// DEVELOPMENT PREVIEW entry: renders the chrome against the Figma design
// fixture in a desktop browser (Chromium/WebKit via Playwright). This is NOT
// JustSlide: no pages load and there is no native WKWebView.
import '@atlaskit/css-reset';
import './styles.css';
import { createRoot } from 'react-dom/client';
import { BridgeContext } from './bridge/bridge';
import { createFixtureBridge } from './bridge/fixture';
import { SlideWindow } from './components/SlideWindow';
import { IconResolverContext, type IconResolver } from './components/Favicon';
import { FIXTURE_FAVICONS, FIXTURE_FAVORITE_ICONS } from './assets/fixture-icons';

const params = new URLSearchParams(location.search);
const variant = params.get('variant') === 'compact' ? 'compact' : 'desktop';
const bridge = createFixtureBridge(variant);
const theme = params.get('theme');
if (theme === 'dark' || theme === 'light') void bridge.execute('settings.update', { theme });
const backdrop = params.get('backdrop');
if (backdrop && /^[0-9A-Fa-f]{6}$/.test(backdrop)) void bridge.execute('settings.update', { backdrop: `#${backdrop.toUpperCase()}` });

const fixtureIcons: IconResolver = {
  tab: (fav) => (fav?.startsWith('fixture:') ? FIXTURE_FAVICONS[fav.slice(8)] ?? null : null),
  favorite: (id) => FIXTURE_FAVORITE_ICONS[id.replace(/^fav_/, '')] ?? null,
};
document.title = `JustSlide — DESIGN FIXTURE PREVIEW (${variant})`;
console.info('[JustSlide] Development design fixture. Not a browser; no page is rendered.');

createRoot(document.getElementById('root')!).render(
  <BridgeContext.Provider value={bridge}>
    <IconResolverContext.Provider value={fixtureIcons}>
      <SlideWindow preview />
    </IconResolverContext.Provider>
  </BridgeContext.Provider>,
);
