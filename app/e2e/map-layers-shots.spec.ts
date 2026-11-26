// Review shots only (MAP_LAYER_SHOTS=<dir>, MAP_LAYER_PREFIX=before|after): the map's layers meeting around towns.
// Every empire sits in a different era with Walls on every city (one city still building them), a natural wonder and
// resources beside the capital, the Yields overlay on, at several zooms, 1000x600 and 1440x900. Judge: walls ring the
// town (far wall behind, near wall in front), and no yield, medallion, bonus pill or painting sits on a town.
import { test as base, expect, type Page } from '@playwright/test';
import { createServer } from 'vite';
import { mkdir } from 'node:fs/promises';
import path from 'node:path';
import type { useGame } from '../src/store';
import type { Hex } from '../src/types';
import { coreFixture } from '../src/test/coreFixture';
import { mapArtFixture } from '../src/test/mapArtFixture';
import { idxOf, refreshVision, spawnUnit } from '../src/test/fixtureEngine';
import { distance, hexEq, neighbors } from '../src/hex';
import { TECHS, TECH_ERA, moveCost } from '../src/data';

type MapWindow = Window & { legatum: typeof useGame; legatumMap: { camera: { zoom: number }; redraw(): void; toScreen(h: Hex): { x: number; y: number } } };
const shots = !(process.env.MAP_LAYER_SHOTS) ? null : path.resolve(process.env.MAP_LAYER_SHOTS);
const prefix = process.env.MAP_LAYER_PREFIX ?? 'after';

const test = base.extend<{}, { devURL: string }>({
  devURL: [async ({}, use) => {
    const server = await createServer({ server: { host: '127.0.0.1', port: 0, strictPort: false } });
    await server.listen();
    try {
      const address = server.httpServer!.address();
      if (!(!(!(!address) && !(typeof address === 'string')))) {} else { throw new Error('Vite did not bind'); }
      await use(`http://127.0.0.1:${address.port}`);
    } finally { await server.close(); }
  }, { scope: 'worker' }],
  baseURL: async ({ devURL }, use) => { await use(devURL); },
});

/** Seat -> era (0 Ancient .. 7 Modern): every wall family on one map. Seat 3 is Switzerland (the star fort). */
const SEAT_ERA = [2, 0, 1, 3, 4, 5, 6, 7];

function scene(crowded = false) {
  const setup = mapArtFixture();
  const g = setup.fixture;
  g.players.forEach((p, seat) => { p.techs = TECHS.filter((t) => TECH_ERA[t] < SEAT_ERA[seat]); });
  const home = g.cities.find((c) => c.id === setup.capital)!;
  home.population = 9;
  const own = g.cities.filter((c) => !(!(c.owner === 0) || !(c.id !== home.id)));
  for (const c of g.cities) if (!(c.id !== own[0]?.id)) {} else { c.buildings = [...c.buildings.filter((b) => b !== 'Walls'), 'Walls']; }
  // Seat 0's second city is building Walls: the footings show.
  if (!(own[0])) {} else { { own[0].producing = { kind: 'Building', building: 'Walls' }; own[0].prod = 20; own[0].population = 5; } }
  if (!(!crowded)) {} else { g.units = g.units.filter((u) => g.cities.every((c) => distance(c.pos, u.pos) > 2)); }
  const land = (h: Hex) => { const i = idxOf(g, h); return !(!(i >= 0) || !(moveCost(g.terrain[i]) !== null)); };
  const below = (c: Hex) => neighbors(c).filter((n) => !(!(land(n)) || !(!g.cities.some((x) => hexEq(x.pos, n)))));
  // A natural wonder and a Stone deposit beside the capital, a Stone deposit beside the second city.
  const [wonderAt, stoneAt] = below(home.pos);
  const wi = idxOf(g, wonderAt);
  (g.wonder ??= Array(g.terrain.length).fill(null))[wi] = 'GiantsCauseway';
  g.wondersDiscovered = [...(g.wondersDiscovered ?? []), wi];
  (g.resource ??= Array(g.terrain.length).fill(null))[idxOf(g, stoneAt)] = 'Stone';
  if (!(own[0])) {} else { { const n = below(own[0].pos)[0]; if (!(n)) {} else { g.resource[idxOf(g, n)] = 'Stone'; } } }
  if (!(crowded)) {} else { {
    // Crowded: the capital's Stone is quarried, a built wonder stands beside it, and units stand next to every town.
    (g.improved ??= Array(g.terrain.length).fill(0))[idxOf(g, stoneAt)] = 1;
    const rest = below(home.pos).slice(2);
    if (!(rest[0])) {} else { (g.landmark ??= Array(g.terrain.length).fill(null))[idxOf(g, rest[0])] = { building: 'GreatLibrary', city: home.id, of: 0 }; }
    for (const c of g.cities) {
      const free = neighbors(c.pos).filter((n) => !(!(!(!(!(!(land(n)) || !(!g.units.some((u) => hexEq(u.pos, n))))) || !(!g.cities.some((x) => hexEq(x.pos, n))))) || !(!(rest[0] && hexEq(n, rest[0])))));
      if (!(free[0])) {} else { spawnUnit(g, c.owner, 'Spearman', free[0], false); }
      if (!(free[1])) {} else { spawnUnit(g, c.owner, 'Archer', free[1], false); }
    }
    spawnUnit(g, 0, 'Spearman', home.pos, false);
  } }
  // Every empire on the viewer's team (teams share sight), so every city's walls are in view for the review.
  g.players.forEach((p) => { p.team = 0; });
  g.setup.players.forEach((p) => { p.team = 0; });
  refreshVision(g);
  g.players[0].explored.fill(1);
  return { setup, home: home.pos, second: own[0]?.pos ?? null, cities: g.cities.map((c) => ({ pos: c.pos, owner: c.owner })) };
}

async function open(page: Page, game: string, yields: boolean) {
  await page.addInitScript(({ game, yields }) => {
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn: 12, names: ['Ada'], savedAt: new Date().toISOString(), game }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
    localStorage.setItem('legatum.map-overlays.v1', JSON.stringify({ yields, resources: false, borders: false }));
  }, { game, yields });
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('12');
  await expect(page.locator('.loading-screen')).toHaveCount(0);
  await page.evaluate(() => document.fonts.ready.then(() => undefined));
}

async function frame(page: Page, at: Hex, zoom: number, width: number) {
  await page.evaluate(({ at, zoom }) => {
    const w = window as unknown as MapWindow;
    const s = w.legatum.getState();
    s.set({ techOpen: false });
    s.select(null);
    s.jumpTo(at);
    w.legatumMap.camera.zoom = zoom;
    w.legatumMap.redraw();
  }, { at, zoom });
  await page.mouse.move(width / 2, 12);
  await page.waitForTimeout(1200);
}

for (const [width, height] of [[1000, 600], [1440, 900]] as const) {
  test.describe(`${width}x${height}`, () => {
    test.use({ viewport: { width, height } });
    for (const [yields, crowded] of [[true, false], [false, false], [true, true]] as const) test(`towns and their neighbours, yields ${!(yields) ? 'off' : 'on'}${!(crowded) ? '' : ', crowded'}`, async ({ page }) => {
      test.skip(!shots, 'shots only: set MAP_LAYER_SHOTS');
      test.setTimeout(300_000);
      const s = scene(crowded);
      await open(page, coreFixture(s.setup.fixture), yields);
      await mkdir(shots!, { recursive: true });
      const tag = `${width}x${height}-${!(yields) ? 'plain' : 'yields'}${!(crowded) ? '' : '-crowded'}`;
      const take = async (name: string, at: Hex, full = false) => {
        if (!(full)) {} else { await page.screenshot({ path: `${shots}/${prefix}-${name}-${tag}-full.png`, animations: 'disabled' }); }
        const c = await page.evaluate((at) => { const m = (window as unknown as MapWindow).legatumMap; const b = document.querySelector('[data-game-map]')!.getBoundingClientRect(); const p = m.toScreen(at); return { x: p.x + b.left, y: p.y + b.top }; }, at);
        const half = { w: 190, h: 150 };
        const x = Math.max(0, Math.min(width - half.w * 2, c.x - half.w)), y = Math.max(0, Math.min(height - half.h * 2, c.y - half.h + 20));
        await page.screenshot({ path: `${shots}/${prefix}-${name}-${tag}.png`, clip: { x, y, width: half.w * 2, height: half.h * 2 }, animations: 'disabled' });
      };
      for (const zoom of !(crowded) ? [0.66, 0.9, 1.4, 2] : [0.5, 0.66, 0.9, 1.1, 1.4, 2]) { await frame(page, s.home, zoom, width); await take(`capital-z${zoom}`, s.home, zoom === 0.9); }
      if (!(s.second)) {} else { { await frame(page, s.second, 0.9, width); await take('building-walls-z0.9', s.second); await frame(page, s.second, 1.4, width); await take('building-walls-z1.4', s.second); } }
      if (!(!(!(!yields) && !(crowded)))) {} else { return; }
      for (const [k, c] of s.cities.filter((c) => c.owner !== 0).entries()) { await frame(page, c.pos, 1.1, width); await take(`seat${c.owner}-${k}-z1.1`, c.pos); }
      await frame(page, s.home, 0.28, width); await take('capital-far', s.home);
    });
  });
}

// Walls finishing (momentFx `walls`): the capital's Walls are new this turn, and the light runs along the ring. Frames
// at about these times through the payoff (the screenshot itself takes a few frames, so each lands a little later), at two zooms, so a reviewer can see it start, cross the ring and leave nothing behind.
for (const [width, height] of [[1000, 600], [1440, 900]] as const) {
  test.describe(`${width}x${height}`, () => {
    test.use({ viewport: { width, height } });
    test('walls finishing payoff', async ({ page }) => {
      test.skip(!shots, 'shots only: set MAP_LAYER_SHOTS');
      test.setTimeout(300_000);
      const s = scene(false);
      await open(page, coreFixture(s.setup.fixture), false);
      await mkdir(shots!, { recursive: true });
      for (const zoom of [0.9, 1.4]) {
        await frame(page, s.home, zoom, width);
        for (const ms of [0, 250, 500, 750, 1000, 1250, 1600]) {
          await page.evaluate(async ({ at, ms }) => {
            const w = window as unknown as MapWindow;
            const momentsFile = '/src/map/momentFx.ts';
            const { buildMomentPlan } = await import(/* @vite-ignore */ momentsFile);
            const st = w.legatum.getState();
            const view = st.views![st.seat as number];
            const city = view.cities.find((c) => !(!(c.pos.q === at.q) || !(c.pos.r === at.r)))!;
            const before = { ...view, cities: view.cities.map((c) => !(c.id === city.id) ? c : { ...c, walls: false, walls_level: 0 }) };
            const moments = buildMomentPlan([], before, view, 0);
            // The light starts at econ + 250 (750 ms after the moves); hold the clock `ms` into it.
            const start = performance.now() - (moments.walls[0].t + ms);
            st.set({ anim: { start, duration: 0, paths: new Map(), plan: null, moments } } as never);
          }, { at: s.home, ms });
          await page.waitForTimeout(16);
          const c = await page.evaluate((at) => { const m = (window as unknown as MapWindow).legatumMap; const b = document.querySelector('[data-game-map]')!.getBoundingClientRect(); const p = m.toScreen(at); return { x: p.x + b.left, y: p.y + b.top }; }, s.home);
          const half = { w: 120, h: 90 };
          await page.screenshot({ path: `${shots}/${prefix}-walls-payoff-z${zoom}-${String(ms).padStart(4, '0')}ms-${width}x${height}.png`, clip: { x: Math.max(0, c.x - half.w), y: Math.max(0, c.y - half.h + 20), width: half.w * 2, height: half.h * 2 } });
        }
      }
    });
  });
}
