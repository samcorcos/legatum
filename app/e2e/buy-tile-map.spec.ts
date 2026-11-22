// Buying a tile on the map (Civ style): a selected city shows a Gold price on each tile it can buy; a click queues the
// purchase, a second click cancels it, and ending the turn gives the tile to the city and takes the Gold.
// Screenshots go to test-results/buy-tile-map/<size>/ (or LEGATUM_SHOTS) at the user's laptop size and 1440x900.
import { mkdir } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import type { Page } from '@playwright/test';
import type { useGame } from '../src/store';
import type { Hex } from '../src/types';
import { distance, tileIndex, within } from '../src/hex';
import { idxOf } from '../src/test/fixtureEngine';
import { test, expect } from './fixtures';
import { endPlanning } from './turnActions';
import { coreFixture } from '../src/test/coreFixture';
import { diplomacyState } from '../src/test/diplomacyFixture';

type GameWindow = Window & { legatum: typeof useGame; legatumMap: { toScreen(hex: Hex): { x: number; y: number } } };
const SIZES = [{ name: '1000x600', width: 1000, height: 600 }, { name: '1440x900', width: 1440, height: 900 }] as const;
const outRoot = process.env.LEGATUM_SHOTS ?? fileURLToPath(new URL('../test-results/buy-tile-map/', import.meta.url));

async function shot(page: Page, size: string, name: string) {
  await page.evaluate(() => document.fonts.ready.then(() => undefined));
  await page.waitForTimeout(300);
  await mkdir(`${outRoot}/${size}`, { recursive: true });
  await page.screenshot({ path: `${outRoot}/${size}/${name}.png`, animations: 'disabled' });
}

/** `busy`: the city is building a Soldier (the common case: its picker is far below the fold); `gold` sets the treasury. */
async function load(page: Page, { busy = false, gold }: { busy?: boolean; gold?: number } = {}) {
  const s = diplomacyState(12, { met: true });
  s.cities[0].buildings = ['Granary'];
  if (busy) Object.assign(s.cities[0], { producing: { kind: 'Unit', unit: 'Spearman' }, prod: 12 });
  if (gold !== undefined) {
    s.players[0].gold = gold;
    s.players[0].explored.fill(1);
    // The city already owns part of ring 2, so ring 2 (50 Gold) and ring 3 (60 Gold) are both buyable.
    for (const h of within(s.cities[0].pos, 2).filter((h) => distance(h, s.cities[0].pos) === 2).slice(0, 6)) { const i = idxOf(s, h); s.owner[i] = 0; s.tileCity[i] = 0; }
  }
  const game = coreFixture(s);
  await page.addInitScript((game) => {
    if (!location.protocol.startsWith('http')) return;
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn: 12, names: ['Ada'], savedAt: new Date().toISOString(), game }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, game);
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('12');
  await expect(page.locator('.loading-screen')).toHaveCount(0);
}

async function hexPoint(page: Page, hex: Hex) {
  const at = () => page.evaluate((hex) => {
    const p = (window as unknown as GameWindow).legatumMap.toScreen(hex);
    const b = document.querySelector('[data-game-map]')!.getBoundingClientRect();
    return { x: Math.round(p.x + b.left), y: Math.round(p.y + b.top) };
  }, hex);
  let last = await at();
  await expect.poll(async () => {
    const now = await at(), stable = now.x === last.x && now.y === last.y;
    last = now;
    return stable;
  }).toBe(true);
  return last;
}

const queued = (page: Page) => page.evaluate(() => {
  const s = (window as unknown as GameWindow).legatum.getState();
  return s.orders[s.seat].filter((o) => o.type === 'BuyTile').length;
});

const selectCity = (page: Page) => page.evaluate(() => {
  const s = (window as unknown as GameWindow).legatum.getState(), v = s.views![s.seat];
  const city = v.cities.find((c) => c.owner === v.me)!;
  s.set({ techOpen: false });
  s.select({ kind: 'city', id: city.id }, true);
  s.jumpTo(city.pos);
  // The last tile in the list, so the city's own growth claim (the best tile of the nearest ring) is likely another one.
  const buyable = city.detail!.buyable_tiles.at(-1)!;
  return { tile: buyable.tile, price: buyable.price, gold: v.economy.gold, income: v.economy.gold_per_turn.total, width: v.width, height: v.height };
});
const placing = (page: Page) => page.evaluate(() => (window as unknown as GameWindow).legatum.getState().place);

for (const size of SIZES) {
  test.describe(size.name, () => {
    test.use({ viewport: { width: size.width, height: size.height } });

    test('Buy a tile opens a priced placement on the map; Escape cancels it', async ({ page }) => {
      await load(page);
      const target = await selectCity(page);
      const panel = page.locator('.city-panel');
      const button = panel.getByRole('button', { name: /^Buy a tile · from \d+ Gold$/ });
      await expect(button).toBeVisible();
      await expect(button).toBeEnabled();
      await shot(page, size.name, '01-buy-a-tile-button');
      expect(await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().orders.flat().length)).toBe(0);
      await button.click();
      await expect(page.getByTestId('city-placement')).toContainText('Click a lit tile to buy it for');
      expect(await placing(page)).toMatchObject({ intent: 'tile' });
      await hexPoint(page, target.tile);
      await shot(page, size.name, '02-choose-a-tile');
      await page.keyboard.press('Escape');
      await expect(page.getByTestId('city-placement')).toHaveCount(0);
      expect(await placing(page)).toBeNull();
      expect(await queued(page)).toBe(0);
    });

    test('select the city, Buy a tile, click a lit tile, press Ready: the city owns the tile and the Gold is spent', async ({ page }) => {
      await load(page);
      const before = await selectCity(page);
      const panel = page.locator('.city-panel');
      await panel.getByRole('button', { name: /^Buy a tile/ }).click();
      const point = await hexPoint(page, before.tile);
      await page.mouse.move(point.x, point.y);
      await expect(page.locator('.tilecard')).toContainText(`Click to buy for ${before.price} Gold`);
      await page.mouse.click(point.x, point.y);
      expect(await placing(page)).toBeNull();
      await expect.poll(() => queued(page)).toBe(1);
      await expect(panel.getByTestId('city-buy-tile')).toContainText('Buying ');
      await expect(panel.locator('.city-borders .cost-chip')).toContainText(`${before.price}`);
      await shot(page, size.name, '03-tile-purchase-queued');
      await endPlanning(page);
      await expect(page.getByTestId('turn-number')).toHaveText('13', { timeout: 30_000 });
      const index = tileIndex(before.tile, before.width, before.height);
      const after = await page.evaluate(({ tile, index }) => {
        const s = (window as unknown as GameWindow).legatum.getState(), v = s.views![s.seat];
        return { gold: v.economy.gold, owner: v.tiles[index].owner, me: v.me, mine: v.cities.find((c) => c.owner === v.me)!.detail!.territory.some((h) => h.q === tile.q && h.r === tile.r) };
      }, { tile: before.tile, index });
      expect(after.mine).toBe(true);
      expect(after.owner).toBe(after.me);
      expect(after.gold).toBe(before.gold - before.price + before.income);
    });

    test('a building city shows the one Buy a tile button at scroll 0, under the tile count', async ({ page }) => {
      await load(page, { busy: true });
      await selectCity(page);
      const panel = page.locator('.city-panel');
      const button = panel.getByRole('button', { name: /^Buy a tile · from \d+ Gold$/ });
      await expect(button).toHaveCount(1);
      await expect(panel.locator('[data-testid="city-buy-tile"]')).toHaveCount(1);
      await expect(button).toBeInViewport({ ratio: 1 });
      expect(await panel.evaluate((el) => el.scrollTop)).toBe(0);
      const box = (await button.boundingBox())!;
      expect(box.height).toBeGreaterThanOrEqual(24);
      const tiles = (await panel.getByTestId('city-territory').boundingBox())!;
      expect(box.y).toBeGreaterThan(tiles.y);
      expect(box.y - (tiles.y + tiles.height)).toBeLessThan(60);
      await shot(page, size.name, '04-busy-city-button');
    });

    test('a click on a lit tile the treasury cannot pay keeps the mode open and says why on the tile card', async ({ page }) => {
      await load(page, { gold: 50 });
      await selectCity(page);
      const dear = await page.evaluate(() => {
        const s = (window as unknown as GameWindow).legatum.getState(), v = s.views![s.seat];
        const b = v.cities.find((c) => c.owner === v.me)!.detail!.buyable_tiles.find((t) => t.price > v.economy.gold)!;
        return { tile: b.tile, price: b.price };
      });
      const panel = page.locator('.city-panel');
      await panel.getByRole('button', { name: /^Buy a tile/ }).click();
      const point = await hexPoint(page, dear.tile);
      await page.mouse.move(point.x, point.y);
      await expect(page.getByTestId('tile-refusal')).toContainText(`Costs ${dear.price} Gold. You have 50 Gold, ${dear.price - 50} short.`);
      await page.mouse.click(point.x, point.y);
      expect(await placing(page)).toMatchObject({ intent: 'tile' });
      expect(await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().selection)).toMatchObject({ kind: 'city' });
      await expect(page.getByTestId('city-placement')).toBeVisible();
      expect(await queued(page)).toBe(0);
      await shot(page, size.name, '05-unaffordable-refused-in-place');
    });

    test('an item purchase queued in the city disables Buy a tile and names it', async ({ page }) => {
      await load(page, { busy: true });
      await selectCity(page);
      await page.evaluate(() => {
        const s = (window as unknown as GameWindow).legatum.getState(), v = s.views![s.seat];
        s.editOrders((o) => [...o, { type: 'BuyItem', city: v.cities.find((c) => c.owner === v.me)!.id, item: { kind: 'Unit', unit: 'Spearman' } }], 'purchase');
      });
      const panel = page.locator('.city-panel');
      const button = panel.getByTestId('city-buy-tile');
      await expect(button).toBeDisabled();
      await expect(button).toHaveText('Buy a tile');
      await expect(panel.locator('.city-borders-reason')).toHaveText(/^Cancel the .+ purchase first$/);
      await shot(page, size.name, '06-item-queued-disables');
    });
  });
}
