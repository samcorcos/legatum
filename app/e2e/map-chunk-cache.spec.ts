// Playtest 8.9.2: "weird blue rectangle artifacts" on the map, and a hard-edged block of sea drawn with a greenish land
// wash. Both are chunk-shaped holes in the terrain cache: after a zoom the visible chunks of the new level are drawn a
// few per frame, and the holes showed the overview (drawn for a far smaller scale, so its pixel-floored borders were
// 5-10x too fat) or the previous level's chunks from an earlier turn. Worse, the cache cap (110) was smaller than a
// big window's working set once the previous level's chunks were counted as used, so it evicted the view's own ring
// of chunks every frame and never went idle.
//
// A second failure, found reviewing the first fix: with no stand-in for a big zoom-out the view was a blank parchment
// hole (no terrain, no borders) for about 2 s. The cache now keeps the whole map as a stand-in layer drawn for a
// typical zoom, so the frame right after a zoom in or out has terrain everywhere the settled frame does.
//
// A third failure, in 8.9.3 on the Mac: while he zoomed, chunk-shaped patches of the soft stand-in layer (a blurred coast
// that does not match the sharp one) showed beside sharp chunks. Each crossing of a zoom level needs a whole new set of
// chunks, drawn a few per frame, and a quick zoom leaves several levels half filled. The cache now holds its level
// through a zoom gesture and uses every close cached level as a stand-in, so a zoom in never shows the soft layer alone.
//
// A fifth, in 8.9.4 on the Mac (the user's T36 save): patches of sea, with coast lines and wave rings, over land. Chromium and
// Playwright's Linux WebKit never show it; real WebKit on macOS did in about half of fresh loads, because it defers canvas
// operations and a chunk's scratch canvases were overwritten by the next chunk in the same frame. `tools/ui/webkit-mac/check.sh`
// reproduces it in the real engine; this spec keeps the save in the live-versus-fresh comparison for the other engines.
//
// A fourth, in 8.9.4: the quiet check failed on a loaded box ("the cache keeps rendering chunks") though the cache was not
// evicting anything. Each city sprite that finishes decoding marked every cached chunk stale, so on a slow machine, where the
// sprites trickle in while the view fills, each arrival repainted the whole view and its ring again (about 90 chunks at
// about 100 ms each) and the cache never went quiet. Now a chunk that drew a town without its art repaints once, when the last
// sprite it waited for arrives, and no other chunk does.
//
// The saves are copies of the user's autosaves (crates/wasm/tests/fixtures/saves/p892-*.legatum). The check: after zooms,
// a pan and a turn, the cache fills every visible chunk, goes quiet, and the map equals the same map drawn fresh.
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import type { Browser, Page } from '@playwright/test';
import { test, expect } from './fixtures';

const dir = fileURLToPath(new URL('../../crates/wasm/tests/fixtures/saves/', import.meta.url));
const SAVES = [
  { name: 'p892-blue-rects-t117', sizes: [[1440, 900], [1000, 600]] },
  { name: 'p892-sea-rect-t141', sizes: [[1440, 900]] },
  { name: 'p894-rects-t36', sizes: [[1440, 900]] },
] as const;

test.use({ deviceScaleFactor: 2 });

type Stats = { chunks: number; renders: number; missing: number; visible: number };
type Win = Window & { legatum: any; legatumMap: any };

async function open(page: Page, save: string) {
  await page.emulateMedia({ reducedMotion: 'reduce' });
  await page.addInitScript((s) => {
    localStorage.setItem('legatum.autosave.v1', s);
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, save);
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await page.waitForFunction(() => (window as unknown as Win).legatum?.getState().views?.length);
  await page.evaluate(() => {
    const g = (window as unknown as Win).legatum;
    g.setState({ reportOpen: false, techOpen: false });
    g.getState().select(null);
  });
}

const camTo = (page: Page, pos: unknown, zoom: number) => page.evaluate(([p, z]: any) => {
  const m = (window as unknown as Win).legatumMap, s = m.toScreen(p), cam = m.camera;
  cam.x += (s.x - innerWidth / 2) / cam.zoom; cam.y += (s.y - innerHeight / 2) / cam.zoom; cam.zoom = z; m.redraw();
}, [pos, zoom]);

const stats = (page: Page) => page.evaluate(() => (window as unknown as Win).legatumMap.terrain as Stats);

/** Every visible chunk is drawn and the cache stops rendering: the map is final, not mid-fill. A failure carries the cache's own counters. */
async function settle(page: Page) {
  const withStats = async (run: () => Promise<void>) => {
    try { await run(); } catch (e) { throw new Error(`${(e as Error).message}\n${JSON.stringify(await stats(page))}`); }
  };
  await withStats(() => expect.poll(async () => (await stats(page)).missing, { timeout: 40_000, message: 'visible chunks still missing' }).toBe(0));
  await withStats(() => expect.poll(async () => {
    const a = (await stats(page)).renders;
    await page.waitForTimeout(1500);
    return (await stats(page)).renders - a;
  }, { timeout: 40_000, message: 'the cache keeps rendering chunks (evicting what it shows)' }).toBe(0));
}

const cityPositions = (page: Page) => page.evaluate(() => {
  const s = (window as unknown as Win).legatum.getState(), v = s.views[s.seat];
  return v.cities.filter((c: any) => c.owner === v.me).map((c: any) => c.pos);
});

const mapPng = async (page: Page) => (await page.evaluate(() => (document.querySelector('canvas[data-game-map]') as HTMLCanvasElement).toDataURL('image/png')));

/** A frame captured two animation frames after the camera moved: the chunks of the new level are still filling in. */
const frameAfter = (page: Page) => page.evaluate(() => new Promise<{ png: string; missing: number }>((done) => {
  requestAnimationFrame(() => requestAnimationFrame(() => {
    done({ png: (document.querySelector('canvas[data-game-map]') as HTMLCanvasElement).toDataURL('image/png'), missing: (window as unknown as Win).legatumMap.terrain.missing });
  }));
}));

/**
 * Cells (32 px) that are blank veil in `early` but terrain in `settled`: the parchment hole. The veil is #ebe2cb with a
 * faint hatch and vignette; a cell counts as blank when its mean is near that and as terrain when the settled mean is not.
 */
async function blankHoles(page: Page, early: string, settled: string) {
  return page.evaluate(async ([x, y]) => {
    const load = (u: string) => new Promise<ImageData>((ok) => {
      const img = new Image();
      img.onload = () => { const c = document.createElement('canvas'); c.width = img.width; c.height = img.height; const g = c.getContext('2d')!; g.drawImage(img, 0, 0); ok(g.getImageData(0, 0, c.width, c.height)); };
      img.src = u;
    });
    const [a, b] = [await load(x), await load(y)];
    const cell = 32, cols = Math.floor(a.width / cell), rows = Math.floor(a.height / cell);
    const mean = (d: ImageData, cx: number, cy: number) => {
      let r = 0, g = 0, bl = 0;
      for (let yy = cy * cell; yy < (cy + 1) * cell; yy += 2) for (let xx = cx * cell; xx < (cx + 1) * cell; xx += 2) { const i = 4 * (yy * d.width + xx); r += d.data[i]; g += d.data[i + 1]; bl += d.data[i + 2]; }
      const n = (cell / 2) ** 2;
      return [r / n, g / n, bl / n];
    };
    const veil = [235, 226, 203];
    const dist = (m: number[]) => Math.hypot(m[0] - veil[0], m[1] - veil[1], m[2] - veil[2]);
    let holes = 0;
    for (let cy = 0; cy < rows; cy++) for (let cx = 0; cx < cols; cx++) if (dist(mean(a, cx, cy)) < 14 && dist(mean(b, cx, cy)) > 30) holes++;
    return holes;
  }, [early, settled]);
}

/** Pixels that differ, and 32 px cells where a quarter of the pixels differ (a chunk-shaped hole). */
async function compare(page: Page, a: string, b: string) {
  return page.evaluate(async ([x, y]) => {
    const load = (u: string) => new Promise<ImageData>((ok) => {
      const img = new Image();
      img.onload = () => { const c = document.createElement('canvas'); c.width = img.width; c.height = img.height; const g = c.getContext('2d')!; g.drawImage(img, 0, 0); ok(g.getImageData(0, 0, c.width, c.height)); };
      img.src = u;
    });
    const [p, q] = [await load(x), await load(y)];
    const cell = 32, cols = Math.floor(p.width / cell), rows = Math.floor(p.height / cell);
    const bad = new Uint32Array(cols * rows);
    let diff = 0;
    for (let yy = 0; yy < p.height; yy++) for (let xx = 0; xx < p.width; xx++) {
      const i = 4 * (yy * p.width + xx);
      if (Math.abs(p.data[i] - q.data[i]) + Math.abs(p.data[i + 1] - q.data[i + 1]) + Math.abs(p.data[i + 2] - q.data[i + 2]) > 60) {
        diff++;
        const cx = Math.floor(xx / cell), cy = Math.floor(yy / cell);
        if (cx < cols && cy < rows) bad[cy * cols + cx]++;
      }
    }
    const holes = [...bad].filter((n) => n > cell * cell * 0.25).length;
    return { diff, total: p.width * p.height, holes };
  }, [a, b]);
}

for (const { name, sizes } of SAVES) {
  for (const [width, height] of sizes) {
    test(`terrain chunks fill, go quiet and match a fresh map: ${name} at ${width}x${height}`, async ({ page, browser, serverURL }: { page: Page; browser: Browser; serverURL: string }) => {
      test.setTimeout(480_000);
      await page.setViewportSize({ width, height });
      const save = readFileSync(`${dir}${name}.current.legatum`, 'utf8');
      await open(page, save);
      const cities = await cityPositions(page);
      const zooms = [1.25, 0.6, 2.0, 0.35, 0.9];
      for (const [i, z] of zooms.entries()) { await camTo(page, cities[i % cities.length], z); await settle(page); }
      // The frame right after a zoom out (three levels) and a zoom in has terrain wherever the settled frame does.
      for (const [from, to] of [[1.25, 0.4], [0.4, 1.4]] as const) {
        await camTo(page, cities[0], from); await settle(page);
        await camTo(page, cities[0], to);
        const early = await frameAfter(page);
        await settle(page);
        const holes = await blankHoles(page, early.png, await mapPng(page));
        expect(holes, `${name}: blank parchment where terrain should be, right after zooming ${from} -> ${to} (${early.missing} chunks missing at capture)`).toBeLessThanOrEqual(3);
      }
      // A turn changes borders and fog under the cached chunks: they repaint in place.
      await page.evaluate(() => { const s = (window as unknown as Win).legatum.getState(); (window as any).__t0 = s.views[s.seat].turn; s.toggleReady(); });
      await page.waitForFunction(() => { const s = (window as unknown as Win).legatum.getState(); return s.views[s.seat].turn > (window as any).__t0 && !s.busy; }, null, { timeout: 120_000 });
      // The turn's celebrations (an era takeover, moment cards) paint on the map while they are open: close them all.
      for (let i = 0; i < 10 && await page.locator('.era-takeover, .moment-card').count() > 0; i++) { await page.keyboard.press('Escape'); await page.waitForTimeout(400); }
      await page.evaluate(() => { const g = (window as unknown as Win).legatum; g.setState({ reportOpen: false, techOpen: false }); g.getState().select(null); });
      const targets = cities.slice(0, 3);
      const cams = [0.9, 1.4, 0.6];
      const live: string[] = [];
      for (const [k, c] of targets.entries()) { await camTo(page, c, cams[k]); await page.mouse.move(2, 2); await settle(page); live.push(await mapPng(page)); }
      const autosave = await page.evaluate(() => localStorage.getItem('legatum.autosave.v1'));

      const context = await browser.newContext({ baseURL: serverURL, viewport: { width, height }, deviceScaleFactor: 2 });
      const fresh = await context.newPage();
      await open(fresh, autosave!);
      mkdirSync('test-results/chunk-cache', { recursive: true });
      for (const [k, c] of targets.entries()) {
        await camTo(fresh, c, cams[k]); await fresh.mouse.move(2, 2); await settle(fresh);
        const f = await mapPng(fresh);
        const r = await compare(fresh, live[k], f);
        writeFileSync(`test-results/chunk-cache/${name}-${width}-${k}-live.png`, Buffer.from(live[k].split(',')[1], 'base64'));
        writeFileSync(`test-results/chunk-cache/${name}-${width}-${k}-fresh.png`, Buffer.from(f.split(',')[1], 'base64'));
        expect(r.holes, `${name} view ${k}: chunk-shaped blocks differ from a fresh map`).toBe(0);
        expect(r.diff / r.total, `${name} view ${k}: share of pixels that differ from a fresh map`).toBeLessThan(0.002);
      }
      await context.close();
    });
  }
}

type Rec = { t: number; exposed: number; level: number; wanted: number; missing: number; chunks: number }[];

/** Wheel-zoom `ticks` steps over a settled view and record, every frame, how many visible chunks show only the soft stand-in layer. */
async function wheelZoom(page: Page, dy: number, ticks: number, gapMs: number, afterMs: number): Promise<{ frames: Rec; endedAt: number }> {
  await page.evaluate(() => {
    const w = window as unknown as Win & { __rec: Rec; __t0: number; __on: boolean };
    w.__rec = []; w.__t0 = performance.now(); w.__on = true;
    const loop = () => { const st = w.legatumMap.terrain; w.__rec.push({ t: Math.round(performance.now() - w.__t0), exposed: st.exposed, level: st.level, wanted: st.wanted, missing: st.missing, chunks: st.chunks }); if (w.__on) requestAnimationFrame(loop); };
    requestAnimationFrame(loop);
  });
  await page.mouse.move(page.viewportSize()!.width / 2, page.viewportSize()!.height / 2);
  for (let i = 0; i < ticks; i++) { await page.mouse.wheel(0, dy); await page.waitForTimeout(gapMs); }
  const endedAt = await page.evaluate(() => performance.now() - (window as any).__t0);
  await page.waitForTimeout(afterMs);
  const frames = await page.evaluate(() => { (window as any).__on = false; return (window as any).__rec as Rec; });
  return { frames, endedAt };
}

test('a wheel zoom holds its chunk level until it rests, and the soft stand-in layer clears soon after the camera stops', async ({ page, browserName }: { page: Page; browserName: string }) => {
  test.setTimeout(300_000);
  await page.setViewportSize({ width: 1000, height: 700 });
  await open(page, readFileSync(`${dir}p893-zoom-scout-t34.current.legatum`, 'utf8'));
  const pos = await page.evaluate(() => { const s = (window as unknown as Win).legatum.getState(), v = s.views[s.seat]; return v.units.find((u: any) => u.owner === v.me).pos; });

  // 1. A quick zoom across a level boundary keeps the level in use (no new set of chunks) until the zoom rests.
  await camTo(page, pos, 0.5); await settle(page);
  const quick = await wheelZoom(page, -120, 2, 30, 1500);
  const held = quick.frames.filter((f) => f.level !== f.wanted).length;
  expect(held, 'frames where the held level differed from the wanted one (a zoom across a level boundary must hold)').toBeGreaterThan(0);
  const last = quick.frames[quick.frames.length - 1];
  expect(last.level, 'the level settles on the wanted one once the zoom rests').toBe(last.wanted);
  await settle(page);

  // 2. On a slow machine (Safari's WebKit under load; Chromium is slowed to match) a longer zoom in and out
  // leaves no chunk showing only the soft stand-in layer for more than about 0.6 s (2.5 s at 4x slower) after the camera stops.
  if (browserName === 'chromium') await (await page.context().newCDPSession(page)).send('Emulation.setCPUThrottlingRate', { rate: 4 });
  for (const dy of [-120, 120]) {
    const run = await wheelZoom(page, dy, 14, 60, 4500); // records well past the 2.5 s limit, or a stuck patch would just end the recording
    const lastBad = Math.max(-1, ...run.frames.filter((f) => f.exposed > 0).map((f) => f.t));
    expect(lastBad - run.endedAt, `ms after the zoom ${dy < 0 ? 'in' : 'out'} stopped that a chunk still showed only the soft stand-in (rate 4x slower)`).toBeLessThanOrEqual(2500);
    await settle(page);
  }
});

// Sprites that arrive over half a minute (a slow machine; the box under a full verify) repaint the chunks that hold a town, once each.
test('late city art repaints the chunks that hold a town once each: p892-sea-rect-t141 at 1440x900', async ({ page }: { page: Page }) => {
  test.setTimeout(300_000);
  await page.setViewportSize({ width: 1440, height: 900 });
  let asked = 0, served = 0;
  await page.route(/\/assets\/(?:[a-z]+-(?:hamlet|town|city|metropolis|walls)|(?:hamlet|town|city|metropolis)-mark)[^/]*\.png$/, async (route) => {
    await new Promise((done) => setTimeout(done, 3000 + 2500 * asked++));
    await route.continue();
    served++;
  });
  await open(page, readFileSync(`${dir}p892-sea-rect-t141.current.legatum`, 'utf8'));
  const cities = await cityPositions(page);
  await camTo(page, cities[0], 1.25);
  // Every sprite has been asked for and has arrived (a few seconds past the last response to decode).
  await expect.poll(async () => { const a = asked; await page.waitForTimeout(4000); return asked === a && served === asked && asked > 0; }, { timeout: 120_000, message: 'the city sprites never all arrived' }).toBe(true);
  await page.waitForTimeout(3000);
  await settle(page);
  const st = await stats(page) as Stats & { chunks: number; evictions: number; epochs: number };
  // Every render beyond one per chunk ever cached (those held now plus those evicted) is a repaint.
  const repaints = st.renders - st.chunks - st.evictions;
  expect(st.epochs, 'sprite arrivals the cache saw (the test needs several spread out)').toBeGreaterThanOrEqual(3);
  // Each arrival used to repaint every cached chunk (273 repaints for 10 arrivals); now a town's chunk repaints about once (76).
  expect(repaints, `chunk repaints across ${st.epochs} sprite arrivals, for ${st.chunks} cached chunks: ${JSON.stringify(st)}`).toBeLessThanOrEqual(st.chunks);
});
