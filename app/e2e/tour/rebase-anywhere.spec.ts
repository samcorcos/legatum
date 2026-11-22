// UI review of train-1103b: aircraft rebase at any distance. The picker lists every other city of yours, near and far, with its own hangar count.
import type { Page } from '@playwright/test';
import { test, expect } from '../fixtures';
import { coreFixture } from '../../src/test/coreFixture';
import { siegeFixture } from '../../src/test/siegeFixture';
import { fromOffset, toOffset } from '../../src/hex';
import { SIZES, shot, type GameWindow } from './helpers';

async function start(page: Page, extraCities: { name: string; dq: number }[]) {
  const setup = siegeFixture({ approach: true });
  const own = setup.fixture.cities.find((c) => c.id === setup.own)!;
  const [col, row] = toOffset(own.pos);
  const width = setup.fixture.width;
  const made = extraCities.map(({ name, dq }) => {
    const c = col + dq < width - 1 ? col + dq : Math.max(1, col - dq);
    const city = { ...own, id: setup.fixture.nextCityId++, name, pos: fromOffset(c, row) };
    setup.fixture.cities.push(city);
    return city;
  });
  const aircraft = JSON.stringify([{ id: 900, kind: 'Bomber', city: setup.own, hp: 100 }, { id: 901, kind: 'Fighter', city: setup.own, hp: 100 }]);
  const save = coreFixture(setup.fixture).replace(/"next_unit_id":\d+/, '"next_unit_id":902');
  const game = save.replace(/^\{/, `{"aircraft":${aircraft},`);
  await page.addInitScript((g) => {
    if (!location.protocol.startsWith('http')) return;
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn: 2, names: ['Ada'], savedAt: new Date().toISOString(), game: g }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, game);
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('2');
  await expect(page.locator('.loading-screen')).toHaveCount(0);
  return { setup, made };
}

for (const size of SIZES) test.describe(`rebase anywhere ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });
  test('the Rebase picker lists a near and a far city', async ({ page }) => {
    await start(page, [{ name: 'Emain Macha', dq: 2 }, { name: 'Alesia', dq: 14 }]);
    await page.evaluate((id) => (window as unknown as GameWindow).legatum.getState().select({ kind: 'air', id }, true), 901);
    await expect(page.getByTestId('air-wing')).toBeVisible();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '01-air-wing-card-with-far-city');
    await page.getByTestId('air-wing').getByRole('button', { name: /Rebase/ }).click();
    await expect(page.getByTestId('air-wing').locator('.air-targets')).toBeVisible();
    await page.mouse.move(5, size.height / 2);
    for (const city of ['Emain Macha', 'Alesia']) {
      const row = page.getByTestId('air-wing').locator('.air-target', { hasText: city });
      await expect(row, `${city} is in view without scrolling the card`).toBeInViewport({ ratio: 1 });
      await expect(row).toContainText('2 of 2 hangar slots free');
    }
    await shot(page, size, '02-rebase-picker-near-and-far');
    await page.getByTestId('air-wing').locator('.air-target', { hasText: 'Alesia' }).hover();
    await shot(page, size, '03-rebase-picker-far-hover');
    await page.getByTestId('air-wing').locator('.air-target', { hasText: 'Alesia' }).click();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '04-rebase-far-ordered');
  });
});
