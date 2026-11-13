// PRODUCTION entry for the trusted chrome inside the macOS app. Never imports
// fixtures or brand assets (enforced by scripts/check-release-bundle.mjs).
import '@atlaskit/css-reset';
import './styles.css';
import { createRoot } from 'react-dom/client';
import { BridgeContext } from './bridge/bridge';
import { createNativeBridge } from './bridge/native';
import { SlideWindow } from './components/SlideWindow';

const bridge = createNativeBridge();
createRoot(document.getElementById('root')!).render(
  <BridgeContext.Provider value={bridge}>
    <SlideWindow />
  </BridgeContext.Provider>,
);
