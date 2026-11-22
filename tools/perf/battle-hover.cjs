#!/usr/bin/env node
// Late-game battle hover and click: load a save, centre the map on its densest fight zoomed in, select an own
// military unit, sweep the mouse over the fight for ~10 s (with a few clicks), and report frame gaps, input lag
// (event to the frame after it), long tasks, map draws (full and clipped) and WASM forecast times.
//   node tools/perf/battle-hover.cjs --url http://127.0.0.1:4173 --save out/perf-save-130-dense.json --turn 131 [--dpr 2]
//     [--zoom 2] [--channel chrome --headed] [--profile] [--out out/battle-hover-<tag>.json]
// A dense save: python3 tools/perf/dense_battle_save.py. On a Mac, `--channel chrome --headed` measures real Chrome with the GPU.
const { chromium } = require('playwright');
const fs = require('fs');

const arg = (name, fallback) => { const i = process.argv.indexOf(`--${name}`); return i < 0 ? fallback : process.argv[i + 1]; };
const flag = (name) => process.argv.includes(`--${name}`);
const base = arg('url', 'http://127.0.0.1:4173');
const savePath = arg('save');
const turn = Number(arg('turn'));
const dpr = Number(arg('dpr', 2));
const zoom = Number(arg('zoom', 2));
const out = arg('out');
if (!savePath || !turn) { console.error('usage: battle-hover.cjs --save <save.json> --turn <turn shown after Continue> [--url ...]'); process.exit(2); }

(async () => {
  const browser = await chromium.launch({ channel: arg('channel'), headless: !flag('headed') });
  const page = await browser.newPage({ viewport: { width: 1440, height: 900 }, deviceScaleFactor: dpr });
  const game = fs.readFileSync(savePath, 'utf8');
  await page.addInitScript(({ game, turn }) => {
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn, names: ['Me'], savedAt: new Date().toISOString(), game }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
    const p = window.__probe = { tasks: [], frames: [], lat: [] };
    try { new PerformanceObserver((l) => { for (const e of l.getEntries()) p.tasks.push(e.duration); }).observe({ type: 'longtask' }); } catch { /* not Chromium */ }
    const tick = (t) => { p.frames.push(t); requestAnimationFrame(tick); };
    requestAnimationFrame(tick);
    for (const kind of ['pointermove', 'pointerdown', 'pointerup']) {
      document.addEventListener(kind, (e) => { const ts = e.timeStamp; requestAnimationFrame(() => setTimeout(() => p.lat.push({ kind, ms: performance.now() - ts }), 0)); }, true);
    }
  }, { game, turn });
  await page.goto(`${base}/?client=wasm`);
  await page.getByRole('button', { name: /^Continue/ }).click();
  await page.getByTestId('turn-number').filter({ hasText: String(turn) }).waitFor({ timeout: 120000 });
  await page.waitForTimeout(1500);
  for (let i = 0; i < 15; i++) { await page.keyboard.press('Escape'); await page.waitForTimeout(100); }
  const scene = await page.evaluate(() => {
    const s = window.legatum.getState();
    const v = s.views[s.seat];
    const d = (a, b) => (Math.abs(a.q - b.q) + Math.abs(a.r - b.r) + Math.abs(a.q + a.r - b.q - b.r)) / 2;
    let best = null;
    for (const u of v.units) {
      const near = v.units.filter((x) => d(x.pos, u.pos) <= 3);
      const rivals = near.filter((x) => x.owner !== v.me).length, mine = near.length - rivals;
      const score = near.length + (rivals && mine ? 10 : 0);
      if (!best || score > best.score) best = { score, pos: u.pos, units: near.length, rivals };
    }
    return { best, units: v.units.length, cities: v.cities.length, turn: v.turn };
  });
  const centre = (pos, z) => page.evaluate(({ pos, z }) => {
    const m = window.legatumMap, c = m.camera;
    if (z) c.zoom = z;
    const at = m.toScreen(pos), canvas = document.querySelector('.map-canvas').getBoundingClientRect();
    c.x += (at.x - canvas.width / 2) / c.zoom; c.y += (at.y - canvas.height / 2) / c.zoom;
    m.redraw();
  }, { pos, z });
  await centre(scene.best.pos, zoom);
  await centre(scene.best.pos);
  const forecastMs = await page.evaluate((pos) => {
    const s = window.legatum.getState(), v = s.views[s.seat], c = s.client;
    const d = (a, b) => (Math.abs(a.q - b.q) + Math.abs(a.r - b.r) + Math.abs(a.q + a.r - b.q - b.r)) / 2;
    const mine = v.units.filter((u) => u.owner === v.me && u.military).sort((a, b) => d(a.pos, pos) - d(b.pos, pos))[0];
    const foe = v.units.find((u) => u.owner !== v.me && u.military);
    if (mine) s.select({ kind: 'unit', id: mine.id });
    if (!mine || !foe) return null;
    const t0 = performance.now();
    for (let i = 0; i < 20; i++) c.forecast(v.me, mine.id, foe.pos, 'Melee');
    return (performance.now() - t0) / 20;
  }, scene.best.pos);
  await page.waitForTimeout(3000);
  const box = await page.locator('.map-canvas').first().boundingBox();
  let cdp = null;
  if (flag('profile')) { cdp = await page.context().newCDPSession(page); await cdp.send('Profiler.enable'); await cdp.send('Profiler.start'); }
  await page.evaluate(() => { const p = window.__probe; p.tasks.length = 0; p.frames.length = 0; p.lat.length = 0; window.legatumMap.perf.reset(); });
  const cx = box.x + box.width / 2, cy = box.y + box.height / 2;
  const t0 = Date.now();
  for (let i = 0; i < 600; i++) {
    const a = i / 600 * Math.PI * 4;
    await page.mouse.move(cx + Math.sin(a) * 260, cy + Math.sin(2 * a) * 140);
    await page.waitForTimeout(16);
    if (i % 150 === 75) { await page.mouse.down(); await page.mouse.up(); }
  }
  const wallMs = Date.now() - t0;
  await page.waitForTimeout(500);
  const r = await page.evaluate(() => {
    const p = window.__probe, perf = window.legatumMap.perf;
    const q = (a, k) => { const s = [...a].sort((x, y) => x - y); return s.length ? s[Math.min(s.length - 1, Math.floor(s.length * k))] : 0; };
    const gaps = p.frames.slice(1).map((t, i) => t - p.frames[i]);
    const moves = p.lat.filter((x) => x.kind === 'pointermove').map((x) => x.ms), clicks = p.lat.filter((x) => x.kind !== 'pointermove').map((x) => x.ms);
    return { frames: gaps.length, frameP50: q(gaps, .5), frameP95: q(gaps, .95), frameMax: Math.max(0, ...gaps), slowFrames: gaps.filter((g) => g > 50).length,
      longTasks: p.tasks.length, longTaskMs: Math.round(p.tasks.reduce((a, b) => a + b, 0)),
      moveLatP50: q(moves, .5), moveLatP95: q(moves, .95), clickLatMax: Math.max(0, ...clicks),
      mapDraws: perf.frames, clippedDraws: perf.partialFrames ?? null, mapDrawMeanMs: perf.drawMs / (perf.frames || 1), mapDrawMaxMs: perf.maxMs };
  });
  const result = { scene, dpr, zoom, forecastMs, wallMs, ...r };
  if (cdp) { const { profile } = await cdp.send('Profiler.stop'); fs.writeFileSync((out ?? 'out/battle-hover.json').replace(/\.json$/, '.cpuprofile'), JSON.stringify(profile)); }
  if (out) fs.writeFileSync(out, JSON.stringify(result, null, 2));
  console.log(JSON.stringify(result, null, 2));
  await browser.close();
})().catch((e) => { console.error(e); process.exit(1); });
