import '@testing-library/react';
// jsdom lacks these browser APIs used by the chrome.
class RO { observe() {} unobserve() {} disconnect() {} }
(globalThis as any).ResizeObserver = (globalThis as any).ResizeObserver ?? RO;
(globalThis as any).matchMedia = (globalThis as any).matchMedia ?? ((q: string) => ({ matches: false, media: q, addEventListener() {}, removeEventListener() {}, addListener() {}, removeListener() {} }));
