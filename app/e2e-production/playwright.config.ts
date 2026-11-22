import { defineConfig } from '@playwright/test';
export default defineConfig({
  testDir: '.', workers: 1, timeout: 90_000,
  use: { browserName: process.env.LEGATUM_E2E_BROWSER === 'webkit' ? 'webkit' : 'chromium', baseURL: 'http://127.0.0.1:5186', viewport: { width: 1280, height: 800 }, headless: true, screenshot: 'only-on-failure', trace: 'retain-on-failure' },
  webServer: { command: 'npm run dev -- --host 127.0.0.1 --port 5186 --strictPort', url: 'http://127.0.0.1:5186', reuseExistingServer: false, timeout: 60_000, cwd: '..' },
});
