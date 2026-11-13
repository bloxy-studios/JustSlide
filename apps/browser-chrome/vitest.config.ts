import { defineConfig, type Plugin } from 'vitest/config';
import compiled from '@compiled/vite-plugin';

// In serve/test mode @compiled/vite-plugin always extracts local rules via a
// virtual CSS module that Vitest's module runner cannot load. Styles are not
// evaluated in jsdom tests (css: false), so resolve it to an empty module.
// Production CSS extraction is exercised by the Parcel build, not by Vitest.
const stubCompiledLocalCss: Plugin = {
  name: 'justslide:stub-compiled-local-css',
  enforce: 'pre',
  resolveId(id) { return id.startsWith('virtual:@compiled/vite-plugin/local-css:') ? '\0justslide-empty-css' : null; },
  load(id) { return id === '\0justslide-empty-css' ? 'export {};' : null; },
};

export default defineConfig({
  plugins: [stubCompiledLocalCss, compiled({ importReact: false, transformerBabelPlugins: [['@atlaskit/tokens/babel-plugin', { shouldUseAutoFallback: false }]] })],
  test: { environment: 'jsdom', globals: true, include: ['src/__tests__/**/*.test.{ts,tsx}'], css: false, setupFiles: ['src/__tests__/setup.ts'] },
});
