// Launch media capture (docs/media/PLAN.md). Not part of the e2e gate: its files end in .media.ts.
//   npx playwright test -c playwright.media.config.ts [-g clip-name]
// Env: MEDIA_OUT (where clips and frames go), MEDIA_ONLY (comma list of clip names).
import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './e2e/media',
  testMatch: '**/*.media.ts',
  workers: 1,
  timeout: 600_000,
  expect: { timeout: 30_000 },
  reporter: 'list',
  // 1440x810 CSS px at 4/3 device scale is a crisp 1920x1080 frame.
  use: { browserName: 'chromium', headless: true, viewport: { width: 1440, height: 810 }, deviceScaleFactor: 4 / 3 },
});
