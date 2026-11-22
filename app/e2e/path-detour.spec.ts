import path from 'node:path';
import type { useGame } from '../src/store';
import type { Hex, Order } from '../src/types';
import { test, expect } from './fixtures';
import { coreFixture } from '../src/test/coreFixture';
import { pathDetourFixture } from '../src/test/pathDetourFixture';

type GameWindow = Window & { legatum: typeof useGame; legatumMap: { toScreen(hex: Hex): { x: number; y: number } } };
const setup = pathDetourFixture();
const shots = process.env.LEGATUM_SHOTS ?? path.resolve('test-results/path-detour');

test('Tessitura attack previews and confirms the direct shared hex', async ({ page }) => {
  await page.setViewportSize({ width: 1000, height: 600 });
  await page.addInitScript((game) => {
    if (!location.protocol.startsWith('http')) return;
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({
      format: 'legatum-save', version: 1, client: 'wasm', turn: 38,
      names: ['Player 1'], savedAt: new Date().toISOString(), game,
    }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, coreFixture(setup.fixture));
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('38');
  const pending: Order[] = [
    { type: 'MoveUnitPath', unit: setup.upper, to: { q: 22, r: 13 }, path: [setup.gap, { q: 22, r: 13 }] },
    { type: 'MoveUnitPath', unit: setup.left, to: { q: 22, r: 12 }, path: [{ q: 20, r: 12 }, { q: 21, r: 12 }, { q: 22, r: 12 }] },
  ];
  await page.evaluate((orders) => {
    const s = (window as unknown as GameWindow).legatum.getState();
    s.editOrders(() => orders);
  }, pending);
  await page.evaluate((id) => (window as unknown as GameWindow).legatum.getState().select({ kind: 'unit', id }, true), setup.attacker);
  const point = async (hex: Hex) => page.evaluate((h) => {
    const p = (window as unknown as GameWindow).legatumMap.toScreen(h);
    const b = document.querySelector('[data-game-map]')!.getBoundingClientRect();
    return { x: Math.round(p.x + b.left), y: Math.round(p.y + b.top) };
  }, hex);
  await expect.poll(async () => {
    const at = await point(setup.target);
    await page.mouse.move(at.x, at.y);
    return page.locator('.forecast').isVisible();
  }).toBe(true);
  await expect(page.getByTestId('route-warning')).toHaveCount(0);
  await page.screenshot({ path: path.join(shots, 'tessitura-direct-1000.png') });
  const at = await point(setup.target);
  await page.mouse.click(at.x, at.y, { button: 'right' });
  await expect.poll(async () => page.evaluate((id) => {
    const s = (window as unknown as GameWindow).legatum.getState();
    return s.orders[s.seat].find((o) => 'unit' in o && o.unit === id);
  }, setup.attacker)).toEqual({ type: 'MoveUnitPath', unit: setup.attacker, to: setup.target, path: [setup.gap, setup.target] });
  // A previously confirmed detour can be corrected by issuing the same destination again.
  await page.evaluate(({ id, to }) => {
    const s = (window as unknown as GameWindow).legatum.getState();
    s.editOrders((orders) => orders.map((o) => 'unit' in o && o.unit === id
      ? { type: 'MoveUnitPath' as const, unit: id, to, path: [{ q: 20, r: 13 }, { q: 21, r: 12 }, { q: 22, r: 12 }, { q: 22, r: 13 }, to] }
      : o));
    s.select({ kind: 'unit', id }, true);
  }, { id: setup.attacker, to: setup.target });
  await page.mouse.click(at.x, at.y, { button: 'right' });
  await expect.poll(async () => page.evaluate((id) => {
    const s = (window as unknown as GameWindow).legatum.getState();
    return s.orders[s.seat].find((o) => 'unit' in o && o.unit === id);
  }, setup.attacker)).toEqual({ type: 'MoveUnitPath', unit: setup.attacker, to: setup.target, path: [setup.gap, setup.target] });
});
