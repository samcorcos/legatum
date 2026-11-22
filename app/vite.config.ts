import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';
import { readFileSync, realpathSync } from 'node:fs';
import { execSync } from 'node:child_process';

// Tauri expects a fixed dev port and a relative base for the bundled app.
// LEGATUM_SERVER=http://localhost:7878 npm run dev  -> the dev UI talks to a running legatum-server.
const server = process.env.LEGATUM_SERVER;
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

export default defineConfig({
  plugins: [react(), { name: 'legatum-site-url', transformIndexHtml: (html: string) => html.replaceAll('%SITE_URL%', siteUrl) }],
  define: {
    __BUILD_LABEL__: JSON.stringify(process.env.LEGATUM_BUILD_LABEL ?? ''),
    __BUILD_DESCRIBE__: JSON.stringify(describe),
    __APP_VERSION__: JSON.stringify(pkg.version),
  },
  base: './',
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    // The bundled scenarios live at the repo root (scenarios/*.json, imported raw). node_modules is
    // allowed by its real path: in worktrees it's a symlink to the main checkout, and fonts 403 otherwise.
    fs: { allow: ['.', '../scenarios', realpathSync('node_modules')] },
    proxy: server ? { '/api': server, '/ws': { target: server.replace(/^http/, 'ws'), ws: true } } : undefined,
  },
  build: { target: 'es2022', outDir: 'dist', chunkSizeWarningLimit: 1500 },
  // 30 s per test and 60 s per hook: several WASM-backed tests take 4 to 10 s on a loaded machine,
  // and the 5 s default failed them on busy shared hosts (2026-11-21 audit).
  test: { environment: 'node', setupFiles: ['src/test/setupOffline.ts', 'src/test/setupWasm.ts'], include: ['src/**/*.test.{ts,tsx}', 'scripts/**/*.test.mjs'], testTimeout: 30_000, hookTimeout: 60_000 },
});
