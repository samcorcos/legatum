import type { Page } from '@playwright/test';
import type { Hex } from '../src/types';
import type { useGame } from '../src/store';
import { test, expect } from './fixtures';
import { coreFixture } from '../src/test/coreFixture';
import { createState, makeStandardSetup, spawnUnit } from '../src/test/fixtureEngine';
import { idxOf, refreshVision } from '../src/test/fixtureEngine';

const ranger: Hex = { q: 6, r: 5 };
const target: Hex = { q: 8, r: 5 };
const forest: Hex = { q: 7, r: 5 };
type GameWindow = Window & { legatum: typeof useGame; legatumMap: { toScreen(hex: Hex): { x: number; y: number } } };

function save() {
  const setup = makeStandardSetup('7301', 'Switzerland', 'Switzerland', 'Ada', 'Rival');
  setup.width = 20; setup.height = 12;
  setup.players = setup.players.slice(0, 2).map((p, i) => ({ ...p, team: i, is_human: i === 0 }));
  const s = createState(setup);
  for (const r of s.diplomacy!.relations) r.state = 'War';
  s.terrain.fill('Grassland'); s.terrain[idxOf(s, forest)] = 'Forest';
  s.owner.fill(-1); s.tileCity.fill(-1); s.ruin.fill(0);
  s.units = []; s.cities = []; s.nextUnitId = 0; s.nextCityId = 0;
  for (const p of s.players) spawnUnit(s, p.id, 'Settler', { q: 1 + 12 * p.id, r: 10 }, false);
  const shooter = spawnUnit(s, 0, 'Ranger', ranger, false);
  spawnUnit(s, 0, 'Scout', { q: 8, r: 4 }, false); // The rival remains visible from another source.
  spawnUnit(s, 1, 'Soldier', target, false);
  refreshVision(s);
  return { game: coreFixture(s), shooter: shooter.id };
}

async function start(page: Page) {
  const { game, shooter } = save();
  await page.addInitScript((game) => {
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn: 1, names: ['Ada'], savedAt: new Date().toISOString(), game }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, game);
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('1');
  await page.evaluate(({ id, at }) => {
    const s = (window as unknown as GameWindow).legatum.getState();
    s.set({ techOpen: false });
    s.select({ kind: 'unit', id });
    s.jumpTo(at);
    s.setMode('ranged');
  }, { id: shooter, at: ranger });
}

async function point(page: Page, hex: Hex) {
  let last = { x: NaN, y: NaN };
  await expect.poll(async () => {
    const p = await page.evaluate((h) => {
      const p = (window as unknown as GameWindow).legatumMap.toScreen(h);
      const b = document.querySelector('[data-game-map]')!.getBoundingClientRect();
      return { x: p.x + b.left, y: p.y + b.top };
    }, hex);
    const still = Math.hypot(p.x - last.x, p.y - last.y) < 0.5;
    last = p;
    return still;
  }).toBe(true);
  return last;
}

test('desktop blocked ranged target is grey and explains the obstruction', async ({ page }, testInfo) => {
  await start(page);
  await page.waitForTimeout(450); // Let the jump-to camera settle before the hover and click use the same target.
  const p = await point(page, target);
  await page.mouse.move(p.x, p.y);
  await expect(page.locator('.tilecard')).toContainText('No line of sight: the Forest');
  await page.screenshot({ path: testInfo.outputPath('terrain-targeting-desktop.png') });
  await page.mouse.click(p.x, p.y);
  await expect(page.locator('.toast[role="alert"]', { hasText: /No line of sight: the Forest/ })).toBeVisible();
  expect(await page.evaluate(() => {
    const s = (window as unknown as GameWindow).legatum.getState();
    return s.orders[s.seat].filter((o) => o.type === 'RangedAttack');
  })).toEqual([]);
});

