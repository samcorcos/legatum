import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';
import { readFileSync, realpathSync } from 'node:fs';
import { execSync } from 'node:child_process';
import { webpAssets } from './vite-plugin-webp.ts';

// The title screen's build label (src/buildLabel.ts): the release label if set, else git describe
// against the last playtest-* tag.
const describe = (() => {
  try {
    return execSync("git describe --tags --match 'playtest-*' --always", { stdio: ['ignore', 'pipe', 'ignore'] }).toString().trim();
  } catch {
    return '';
  }
})();

// The web build's public address, for the link-preview tags in index.html (they need absolute URLs).
// package.json's `homepage` is the one place it is written; LEGATUM_SITE_URL overrides it for another host.
const pkg = JSON.parse(readFileSync(new URL('./package.json', import.meta.url), 'utf8')) as { homepage: string; version: string };
const siteUrl = (process.env.LEGATUM_SITE_URL || pkg.homepage).replace(/\/+$/, '');

// The engine's rules version (crates/core/src/lib.rs), which an online game must match: read here so the
// title screen knows it without loading the engine. src/platform/online.test.ts checks it against the engine.
const rulesVersion = Number(/pub const RULES_VERSION: u32 = (\d+);/.exec(readFileSync(new URL('../crates/core/src/lib.rs', import.meta.url), 'utf8'))?.[1]);
if (!Number.isInteger(rulesVersion)) throw new Error('RULES_VERSION not found in crates/core/src/lib.rs');

export default defineConfig({
  // The website build (scripts/build-web.sh sets LEGATUM_WEBP=1) serves the bundled PNG art as WebP; the dev server and a plain `npm run build` keep PNG.
  plugins: [react(), ...(process.env.LEGATUM_WEBP === '1' ? [webpAssets()] : []), { name: 'legatum-site-url', transformIndexHtml: (html: string) => html.replaceAll('%SITE_URL%', siteUrl) }],
  define: {
    __BUILD_LABEL__: JSON.stringify(process.env.LEGATUM_BUILD_LABEL ?? ''),
    __BUILD_DESCRIBE__: JSON.stringify(describe),
    __APP_VERSION__: JSON.stringify(pkg.version),
    __RULES_VERSION__: JSON.stringify(rulesVersion),
  },
  // A relative base, so the build works from any folder of a static host.
  base: './',
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    // The bundled scenarios live at the repo root (scenarios/*.json, imported raw). node_modules is
    // allowed by its real path: in worktrees it's a symlink to the main checkout, and fonts 403 otherwise.
    fs: { allow: ['.', '../scenarios', realpathSync('node_modules')] },
  },
  build: { target: 'es2022', outDir: 'dist', chunkSizeWarningLimit: 1500 },
  // 30 s per test and 60 s per hook: several WASM-backed tests take 4 to 10 s on a loaded machine,
  // and the 5 s default failed them on busy shared hosts (2026-11-21 audit).
  test: { environment: 'node', setupFiles: ['src/test/setupOffline.ts', 'src/test/setupWasm.ts'], include: ['src/**/*.test.{ts,tsx}', 'scripts/**/*.test.mjs'], testTimeout: 30_000, hookTimeout: 60_000 },
});
