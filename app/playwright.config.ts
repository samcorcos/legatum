import { defineConfig } from '@playwright/test';

const engine = process.env.LEGATUM_E2E_BROWSER;
const browserName = engine === 'firefox' || engine === 'webkit' ? engine : 'chromium';

// Playwright's WebKit on macOS captures screenshots through WindowServer, which leaked capture
// surfaces until it aborted and logged the user out (2026-10-04 23:17 and 2026-10-05 08:15:
// 46k IOSurfaces, 55 GB, request from "Playwright"). Host WebKit runs only in the release check,
// one worker at a time; everything else uses Chromium (or WebKit in a Linux container).
// Never run WebKit on the macOS host: its screenshots go through WindowServer, which aborted three
// times (2026-10-05, 2026-10-05, 2026-10-07: the last during the release check's one-worker run),
// logging the user out. WebKit runs only in the Linux container (tools/release/webkit-docker.sh).
if (engine === 'webkit' && process.platform === 'darwin') {
  throw new Error('WebKit never runs on the macOS host (it crashes WindowServer). Use Chromium, or tools/release/webkit-docker.sh for WebKit in a Linux container.');
}

export default defineConfig({
  testDir: './e2e',
  workers: 1,
  timeout: 45_000,
  expect: { timeout: 10_000 },
  use: { browserName, headless: true, viewport: { width: 1440, height: 1100 }, screenshot: 'only-on-failure', trace: 'retain-on-failure' },
  projects: [{ name: 'game' }],
});
