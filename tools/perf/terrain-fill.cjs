#!/usr/bin/env node
// How the map fills in on a slow machine: long tasks while terrain chunks paint after camera jumps and zooms.
//
// Headless Chromium here has no graphics card, so canvases are drawn on the CPU, as on a laptop whose GPU Chrome
// does not use; `--cpu` slows the main thread further (Chrome's CPU throttling). The script loads a save, jumps the
// camera to six places and through four zoom levels, waits each time for the view to fill, and reports the long
// tasks (over 50 ms) the browser recorded meanwhile.
//
//   node tools/perf/terrain-fill.cjs --dist app/dist --save save.json --turn 121 [--cpu 4] [--w 1366 --h 768]
//
// `--save` is an engine save (see first-open.cjs for how to make one); `--turn` is the turn the game shows once it
// is loaded (the save's turn + 1). Prints one JSON object: chunk renders, long tasks, their total and longest, and
// the same per step.
const { chromium } = require('playwright');
const fs = require('fs');
const http = require('http');
const path = require('path');

const arg = (name, fallback) => { const i = process.argv.indexOf(`--${name}`); return i < 0 ? fallback : process.argv[i + 1]; };
const dist = path.resolve(arg('dist', 'app/dist'));
const save = arg('save');
const turn = Number(arg('turn'));
const rate = Number(arg('cpu', 4));
const W = Number(arg('w', 1366));
const H = Number(arg('h', 768));
if (!save || !turn) { console.error('usage: terrain-fill.cjs --dist app/dist --save save.json --turn N [--cpu 4]'); process.exit(2); }
const TYPES = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css', '.wasm': 'application/wasm', '.png': 'image/png', '.jpg': 'image/jpeg', '.svg': 'image/svg+xml', '.json': 'application/json' };

(async () => {
  const server = http.createServer((req, res) => {
    let file = path.join(dist, decodeURIComponent(new URL(req.url, 'http://x').pathname));
    if (!file.startsWith(dist) || !fs.existsSync(file) || fs.statSync(file).isDirectory()) file = path.join(dist, 'index.html');
    res.writeHead(200, { 'Content-Type': TYPES[path.extname(file)] || 'application/octet-stream' });
    fs.createReadStream(file).pipe(res);
  });
  await new Promise((done) => server.listen(0, '127.0.0.1', done));
  const browser = await chromium.launch();
  const page = await browser.newPage({ viewport: { width: W, height: H }, deviceScaleFactor: 1 });
  const game = fs.readFileSync(save, 'utf8');
  await page.addInitScript(({ game, turn }) => {
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn, names: ['Me'], savedAt: new Date().toISOString(), game }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
    window.__long = [];
    try { new PerformanceObserver((list) => { for (const e of list.getEntries()) window.__long.push([e.startTime, e.duration]); }).observe({ type: 'longtask' }); } catch { /* no long task API */ }
  }, { game, turn });
  await page.goto(`http://127.0.0.1:${server.address().port}/?client=wasm`);
  await page.getByRole('button', { name: /^Continue/ }).click();
  await page.getByTestId('turn-number').filter({ hasText: String(turn) }).waitFor({ timeout: 180000 });
  for (let i = 0; i < 15; i++) { await page.keyboard.press('Escape'); await page.waitForTimeout(100); }
  await page.waitForTimeout(3000);
  const cdp = await page.context().newCDPSession(page);
  await cdp.send('Emulation.setCPUThrottlingRate', { rate });

  const missing = () => page.evaluate(() => window.legatumMap.terrain?.missing ?? 0);
  // The view is full when no visible chunk is missing, twice, 300 ms apart.
  const settle = async () => {
    const t0 = Date.now();
    while (Date.now() - t0 < 20000) {
      if (!(await missing())) { await page.waitForTimeout(300); if (!(await missing())) break; }
      await page.waitForTimeout(50);
    }
    return Date.now() - t0;
  };
  await settle();
  await page.evaluate(() => { window.__long.length = 0; });
  const world = await page.evaluate(() => { const s = window.legatum.getState(); const v = s.views[s.seat]; return { w: v.width, h: v.height }; });
  const steps = [];
  const step = async (kind, move, value) => {
    const t0 = await page.evaluate(() => performance.now());
    const r0 = await page.evaluate(() => window.legatumMap.terrain.renders);
    await page.evaluate(move, value);
    const settle_ms = await settle();
    const renders = (await page.evaluate(() => window.legatumMap.terrain.renders)) - r0;
    const long = await page.evaluate((t0) => window.__long.filter((e) => e[0] >= t0 - 5).map((e) => Math.round(e[1])), t0);
    steps.push({ kind, renders, settle_ms, long });
  };
  // A hex is about 62 by 54 world pixels.
  for (const [fx, fy] of [[0.2, 0.2], [0.8, 0.3], [0.5, 0.8], [0.3, 0.6], [0.7, 0.7], [0.5, 0.5]])
    await step('jump', ({ fx, fy, world }) => { const c = window.legatumMap.camera; c.x = fx * world.w * 62.35; c.y = fy * world.h * 54; window.legatumMap.redraw(); }, { fx, fy, world });
  for (const z of [1.4, 0.7, 2.2, 1.0])
    await step(`zoom ${z}`, (z) => { window.legatumMap.camera.zoom = z; window.legatumMap.redraw(); }, z);
  const all = steps.flatMap((s) => s.long);
  console.log(JSON.stringify({
    cpu: rate, w: W, h: H,
    renders: steps.reduce((a, s) => a + s.renders, 0),
    long_tasks: all.length,
    long_ms: all.reduce((a, b) => a + b, 0),
    long_max: Math.max(0, ...all),
    over_200: all.filter((x) => x > 200).length,
    settle_ms: steps.reduce((a, s) => a + s.settle_ms, 0),
    steps,
  }, null, 1));
  await browser.close();
  server.close();
})();
