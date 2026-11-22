import type { useGame } from '../src/store';
import type { Hex } from '../src/types';
import { test, expect } from './fixtures';
import { coreFixture } from '../src/test/coreFixture';
import { explorationFixture } from '../src/test/explorationFixture';

type GameWindow = Window & {
  legatum: typeof useGame;
  legatumMap: { toScreen(hex: Hex): { x: number; y: number }; redraw(): void };
};

test.use({ viewport: { width: 1000, height: 600 } });

test('Found City clears selection before the next click on a tile shared with an escort', async ({ page }) => {
  const fixture = explorationFixture();
  const settler = fixture.units.find((unit) => unit.owner === 0 && unit.kind === 'Settler')!;
  const escort = fixture.units.find((unit) => unit.owner === 0 && unit.kind === 'Scout')!;
  escort.pos = { ...settler.pos };
  await page.addInitScript((game) => {
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({
      format: 'legatum-save', version: 1, client: 'wasm', turn: 1,
      names: ['Ada'], savedAt: new Date().toISOString(), game,
    }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, coreFixture(fixture));
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.locator('.loading-screen')).toHaveCount(0);
  // The game selects its first unit once it has loaded; choose the Settler only after that, or it replaces the choice.
  await expect.poll(() => page.evaluate(() => (window as unknown as GameWindow).legatum.getState().selection)).not.toBeNull();
  await page.evaluate((id) => {
    const game = (window as unknown as GameWindow).legatum;
    game.getState().select({ kind: 'unit', id }, true);
  }, settler.id);
  await page.getByRole('button', { name: /^Found City/ }).click();

  const orders = await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().orders[0]);
  expect(orders).toContainEqual({ type: 'FoundCity', unit: settler.id });
  expect(await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().selection)).toBeNull();

  const point = await page.evaluate((pos) => {
    const win = window as unknown as GameWindow;
    win.legatum.getState().jumpTo(pos);
    win.legatumMap.redraw();
    return win.legatumMap.toScreen(pos);
  }, settler.pos);
  await page.mouse.click(point.x, point.y);
  await expect.poll(() => page.evaluate(() => (window as unknown as GameWindow).legatum.getState().orders[0])).toEqual(orders);
});
