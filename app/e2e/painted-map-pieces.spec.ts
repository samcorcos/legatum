// The last placeholder drawings on the map, replaced with painted art (legatum-ui, 2026-11-01: "no line drawings of objects on
// the map"): barbarians are painted raiders at normal zoom and class roundels at far zoom, a city's air wing wears the unit
// icon, and Walls go up through their painted stages. At the user's laptop window and a desktop one.
import { test, expect } from './fixtures';
import { wallsFixture } from '../src/test/wallsFixture';
import { coreFixture } from '../src/test/coreFixture';
import { fromOffset, hexToPixel } from '../src/hex';
import { HEX_SIZE } from '../src/map/grid';
import { spawnUnit, refreshVision } from '../src/test/fixtureEngine';
import { SIZES, loadFixture, shot, type GameWindow } from './tour/helpers';
import type { Page } from '@playwright/test';
import type { UnitKind } from '../src/types';

type MapWindow = GameWindow & { legatumMap: { camera: { x: number; y: number; zoom: number }; redraw(): void } };

/** Every kind a barbarian camp fields (crates/core/src/rules/barbarians.rs), around the own city at (3, 5). */
const RAIDERS: [UnitKind, number, number][] = [['Spearman', 5, 4], ['Pikeman', 5, 5], ['Musketman', 5, 6], ['Archer', 6, 4], ['Crossbowman', 6, 5], ['Scout', 6, 6]];

async function open(page: Page, opts: { raiders?: boolean; planes?: boolean; walls?: number } = {}) {
  const setup = wallsFixture(), f = setup.fixture;
  const own = f.cities.find((c) => c.id === setup.own)!;
  if (opts.walls !== undefined) {
    own.buildings = [];
    own.producing = { kind: 'Building', building: 'Walls' } as typeof own.producing;
    own.prod = opts.walls; own.productionQueue = [];
  }
  if (opts.raiders) {
    for (const [kind, col, row] of RAIDERS) spawnUnit(f, 255, kind, fromOffset(col, row), false);
    spawnUnit(f, 0, 'Scout', fromOffset(4, 5), false);
    refreshVision(f);
  }
  let game = coreFixture(f);
  if (opts.planes) {
    // The save holds u64 bitsets that JSON.parse would round, so the hangar is added as text (as air-wing.spec.ts does).
    game = game.replace(/"next_unit_id":\d+/, '"next_unit_id":902')
      .replace(/^\{/, `{"aircraft":${JSON.stringify([{ id: 900, kind: 'Bomber', city: setup.own, hp: 100 }, { id: 901, kind: 'Fighter', city: setup.own, hp: 100 }])},`);
  }
  await loadFixture(page, game, 2);
  return { setup, own };
}

async function frame(page: Page, at: { q: number; r: number }, zoom: number) {
  const p = hexToPixel(at, HEX_SIZE);
  await page.evaluate(({ x, y, zoom }) => {
    const w = window as unknown as MapWindow;
    w.legatum.getState().select(null);
    Object.assign(w.legatumMap.camera, { x, y, zoom });
    w.legatumMap.redraw();
  }, { x: p.x, y: p.y, zoom });
  await page.mouse.move(2, 2);
}

for (const size of SIZES) test.describe(`painted map pieces ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  test('barbarians are painted raiders, class roundels zoomed out', async ({ page }) => {
    test.setTimeout(120_000);
    await open(page, { raiders: true });
    const seen = await page.evaluate(() => {
      const s = (window as unknown as GameWindow).legatum.getState(), v = s.views![s.seat];
      return v.units.filter((u) => u.owner === 255).map((u) => u.kind).sort();
    });
    expect(seen).toEqual(RAIDERS.map(([k]) => k).sort());
    await frame(page, fromOffset(5, 5), 2.2);
    await shot(page, size, 'barbarians-near');
    await frame(page, fromOffset(5, 5), 0.5);
    await shot(page, size, 'barbarians-far');
  });

  test('a city air wing wears the unit icon', async ({ page }) => {
    test.setTimeout(120_000);
    const { own } = await open(page, { planes: true });
    await frame(page, own.pos, 2.2);
    await shot(page, size, 'air-wing-badge');
  });

  // Walls cost 100 Production here: 10, 40 and 75 stored are stages 1, 2 and 3 (10%, 40% and 75% are the thresholds).
  for (const [stage, stored] of [[1, 10], [2, 40], [3, 75]] as const) test(`walls stage ${stage}`, async ({ page }) => {
    test.setTimeout(120_000);
    const { own } = await open(page, { walls: stored });
    const k = await page.evaluate((id) => {
      const s = (window as unknown as GameWindow).legatum.getState(), c = s.views![s.seat].cities.find((x) => x.id === id)!;
      return c.detail!.production_stored / c.detail!.production_cost!;
    }, own.id);
    expect(k >= [0.1, 0.4, 0.75][stage - 1] && k < [0.4, 0.75, 1][stage - 1], `k=${k}`).toBe(true);
    await frame(page, own.pos, 2.6);
    await shot(page, size, `walls-stage${stage}`);
  });
});
