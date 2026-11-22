import { defineConfig, type Project } from '@playwright/test';

// The hosted web build's smoke test: app/dist (built by scripts/build-web.sh) on serve.mjs, a plain
// static server with no backend, the way Cloudflare Pages serves it.
//   scripts/build-web.sh && cd app && npm run e2e:hosted
//
// Chromium runs by default. Firefox and WebKit are opt-in, because their browser binaries are not
// installed everywhere (`npx playwright install firefox webkit` fetches them where that is allowed):
//   LEGATUM_E2E_BROWSERS=all npm run e2e:hosted                       Chromium, Firefox and WebKit
//   LEGATUM_E2E_BROWSERS=firefox,webkit npm run e2e:hosted            just those
//   LEGATUM_E2E_BROWSERS=all npm run e2e:hosted -- --project=firefox  one of them
// WebKit never runs on a macOS host (the same rule as ../playwright.config.ts: its screenshots go through
// WindowServer, which crashed and logged the user out three times). On a Mac, "all" leaves WebKit out
// and naming webkit fails; run WebKit in the Linux container (tools/release/webkit-docker.sh).
const port = Number(process.env.LEGATUM_HOSTED_PORT ?? 4178);

type Engine = 'chromium' | 'firefox' | 'webkit';
const ENGINES: Engine[] = ['chromium', 'firefox', 'webkit'];
const asked = (process.env.LEGATUM_E2E_BROWSERS ?? 'chromium').split(',').map((s) => s.trim().toLowerCase()).filter(Boolean);
const mac = process.platform === 'darwin';
if (mac && asked.includes('webkit')) {
  throw new Error('WebKit never runs on the macOS host (it crashes WindowServer). Use Chromium or Firefox, or tools/release/webkit-docker.sh for WebKit in a Linux container.');
}
const unknown = asked.filter((name) => name !== 'all' && !ENGINES.includes(name as Engine));
if (unknown.length) throw new Error(`LEGATUM_E2E_BROWSERS: unknown browser ${unknown.join(', ')} (use chromium, firefox, webkit or all)`);
const engines = asked.includes('all') ? ENGINES.filter((e) => !(mac && e === 'webkit')) : ENGINES.filter((e) => asked.includes(e));

const projects: Project[] = engines.map((browserName) => ({
  name: browserName,
  use: { browserName },
  // Each browser writes its own failure shots.
  outputDir: `../test-results/hosted/${browserName}`,
}));

export default defineConfig({
  testDir: '.',
  workers: 1,
  timeout: 180_000,
  expect: { timeout: 15_000 },
  outputDir: '../test-results/hosted',
  use: { baseURL: `http://127.0.0.1:${port}`, viewport: { width: 1440, height: 900 }, headless: true, screenshot: 'only-on-failure', trace: 'off' },
  projects,
  webServer: { command: `node e2e-hosted/serve.mjs ${port} dist`, url: `http://127.0.0.1:${port}`, reuseExistingServer: false, timeout: 30_000, cwd: '..' },
});
