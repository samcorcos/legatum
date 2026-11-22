import { defineConfig } from '@playwright/test';

// The hosted web build's smoke test: app/dist (built by scripts/build-web.sh) on serve.mjs, a plain
// static server with no backend, the way Cloudflare Pages serves it.
//   scripts/build-web.sh && cd app && npm run e2e:hosted
const port = Number(process.env.LEGATUM_HOSTED_PORT ?? 4178);

export default defineConfig({
  testDir: '.',
  workers: 1,
  timeout: 180_000,
  expect: { timeout: 15_000 },
  outputDir: '../test-results/hosted',
  use: { browserName: 'chromium', baseURL: `http://127.0.0.1:${port}`, viewport: { width: 1440, height: 900 }, headless: true, screenshot: 'only-on-failure', trace: 'off' },
  webServer: { command: `node e2e-hosted/serve.mjs ${port} dist`, url: `http://127.0.0.1:${port}`, reuseExistingServer: false, timeout: 30_000, cwd: '..' },
});
