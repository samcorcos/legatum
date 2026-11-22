#!/usr/bin/env bash
# Bundle sizes and cold load of the release web build, in Chromium on the box.
#   tools/cloud/remote.sh . --with-uncommitted --npm -- bash tools/perf/app-load.sh
set -e
scripts/build-wasm.sh
cd app && npm run build 2>&1 | tail -15
echo "== dist sizes (raw / gzip / brotli)"
find dist -type f \( -name '*.js' -o -name '*.css' -o -name '*.wasm' -o -name '*.html' \) | while read f; do
  printf '%9d %9d %9d  %s\n' "$(stat -c %s "$f")" "$(gzip -9 -c "$f" | wc -c)" "$(brotli -c "$f" 2>/dev/null | wc -c)" "$f"
done | sort -rn | head -15
echo "dist total: $(du -sh dist | cut -f1); images: $(du -sh dist/assets 2>/dev/null | cut -f1)"
(npx vite preview --port 4173 --strictPort >/tmp/preview.log 2>&1 &) ; sleep 3
cat > /tmp/load.mjs <<'JS'
import { chromium } from '@playwright/test';
const browser = await chromium.launch();
for (let run = 0; run < 5; run++) {
  const ctx = await browser.newContext({ viewport: { width: 1000, height: 600 } });
  const page = await ctx.newPage();
  const t0 = Date.now();
  await page.goto('http://localhost:4173/?client=wasm', { waitUntil: 'commit' });
  await page.getByRole('button', { name: /^Single Player/ }).waitFor();
  const title = Date.now() - t0;
  const m = await page.evaluate(() => {
    const nav = performance.getEntriesByType('navigation')[0];
    const res = performance.getEntriesByType('resource');
    return { dcl: Math.round(nav.domContentLoadedEventEnd), load: Math.round(nav.loadEventEnd),
      bytes: res.reduce((a, r) => a + r.transferSize, 0), n: res.length,
      heapMB: Math.round(performance.memory.usedJSHeapSize / 1048576) };
  });
  console.log(`run ${run}: title screen ${title} ms`, JSON.stringify(m));
  await ctx.close();
}
await browser.close();
JS
cp /tmp/load.mjs ./perf-load.mjs && node perf-load.mjs; rm -f perf-load.mjs
