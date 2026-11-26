#!/usr/bin/env bash
# Bundle sizes and cold load through playable turn 1 of the release web build.
#   tools/cloud/remote.sh . --with-uncommitted --npm -- bash tools/perf/app-load.sh
set -euo pipefail
cd "$(dirname "$0")/../.."
scripts/build-wasm.sh
cd app
npm run build 2>&1 | tail -15
echo "dist total: $(du -sh dist | cut -f1); images: $(du -sh dist/assets 2>/dev/null | cut -f1)"
node --input-type=module <<'JS'
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { brotliCompressSync, gzipSync } from 'node:zlib';
import { chromium, expect } from '@playwright/test';
import { preview } from 'vite';

console.log('== dist sizes (raw / gzip / brotli)');
const files = readdirSync('dist', { recursive: true })
  .filter((file) => /\.(js|css|wasm|html)$/.test(file))
  .map((file) => `dist/${file}`)
  .map((path) => ({ path, size: statSync(path).size }))
  .sort((a, b) => b.size - a.size);
for (const { path, size } of files.slice(0, 15)) {
  const data = readFileSync(path);
  const sizes = [size, gzipSync(data, { level: 9 }).length, brotliCompressSync(data).length];
  console.log(`${sizes.map((n) => String(n).padStart(9)).join(' ')}  ${path}`);
}

const server = await preview({ preview: { host: '127.0.0.1', port: 0, strictPort: true } });
const viewport = { width: 1000, height: 600 };
const seed = '424242';
const samples = [];
try {
  const url = `http://127.0.0.1:${server.httpServer.address().port}/?client=wasm`;
  for (let run = 1; run <= 5; run++) {
    const browser = await chromium.launch();
    try {
      if (run === 1) console.log(`== Chromium ${browser.version()}, ${viewport.width}x${viewport.height}, Standard, seed ${seed}`);
      const page = await browser.newPage({ viewport });
      page.setDefaultTimeout(60_000);
      const t0 = performance.now();
      await page.goto(url, { waitUntil: 'commit' });
      const solo = page.getByRole('button', { name: /^Single Player/ });
      await solo.waitFor();
      const title = performance.now();
      const titleResources = await page.evaluate(() => {
        const nav = performance.getEntriesByType('navigation')[0];
        const res = performance.getEntriesByType('resource');
        return { dcl: Math.round(nav.domContentLoadedEventEnd), load: Math.round(nav.loadEventEnd),
          bytes: res.reduce((sum, r) => sum + r.transferSize, 0), n: res.length,
          heapMB: Math.round(performance.memory.usedJSHeapSize / 1048576) };
      });
      await solo.click();
      const size = page.getByRole('radiogroup', { name: 'Map size', exact: true });
      await expect(size.getByRole('radio', { name: /^Standard/ })).toBeChecked();
      await page.getByRole('textbox', { name: 'Map seed', exact: true }).fill(seed);
      const start = performance.now();
      await page.getByRole('button', { name: /^Start game/ }).click();
      const intro = page.getByTestId('people-intro');
      const begin = intro.getByRole('button', { name: /^Begin/ });
      await begin.waitFor();
      const blockingLoad = page.locator('.loading-screen:not(.leaving)');
      await blockingLoad.waitFor({ state: 'hidden' });
      const introVisible = performance.now();
      await begin.click();
      await intro.waitFor({ state: 'hidden' });
      await expect(page.getByTestId('turn-number')).toHaveText('1');
      await page.locator('canvas[data-game-map]').waitFor();
      // A mounted canvas is not a drawn map; this counter increments after a completed draw.
      await page.waitForFunction(() => window.legatumMap?.perf.frames > 0);
      const dock = page.locator('[data-overlay-slot="turn-dock"]');
      await dock.getByRole('button', { name: /^(Ready anyway|End turn)/ }).click({ trial: true });
      const playable = performance.now();
      const phases = {
        navToTitle: Math.round(title - t0),
        titleToStart: Math.round(start - title),
        startToIntro: Math.round(introVisible - start),
        introToPlayable: Math.round(playable - introVisible),
        startToPlayable: Math.round(playable - start),
        navToPlayable: Math.round(playable - t0),
      };
      samples.push(phases);
      console.log(`run ${run} (ms):`, JSON.stringify(phases), 'title resources:', JSON.stringify(titleResources));
    } finally {
      await browser.close();
    }
  }
  const medians = Object.fromEntries(Object.keys(samples[0]).map((key) => [key, samples.map((s) => s[key]).sort((a, b) => a - b)[2]]));
  console.log('median (ms):', JSON.stringify(medians));
} finally {
  await server.close();
}
JS
