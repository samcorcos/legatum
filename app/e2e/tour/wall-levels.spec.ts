// City banners show each city's wall level after its name (the Walls mark, then II for a Castle or III for a Star Fort),
// so an attacker sees how strong a city's Walls are before attacking. A rival city building Walls shows only footings.
// Capitals carry the star seal, so the numeral must clear it. At the user's laptop window and a desktop one.
import { test } from '../fixtures';
import { wallsFixture } from '../../src/test/wallsFixture';
import { coreFixture } from '../../src/test/coreFixture';
import { fromOffset, hexToPixel } from '../../src/hex';
import { HEX_SIZE } from '../../src/map/grid';
import { SIZES, loadFixture, shot, type GameWindow } from './helpers';
import type { Page } from '@playwright/test';
import type { BuildingKind } from '../../src/types';

type MapWindow = GameWindow & { legatumMap: { camera: { x: number; y: number; zoom: number }; redraw(): void } };

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

for (const size of SIZES) test.describe(`wall levels ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  test('banners show Walls, Castle (II) and Star Fort (III); a rival building Walls shows footings', async ({ page }) => {
    test.setTimeout(120_000);
    const setup = wallsFixture(), f = setup.fixture;
    const city = (id: number) => f.cities.find((c) => c.id === id)!;
    // Own capital: Star Fort (III) beside its star seal. Rival capital Berea: Castle (II). Rival town: building Walls.
    city(setup.own).buildings = ['Walls', 'Castle', 'StarFort'] as BuildingKind[];
    city(setup.walled).buildings = ['Walls', 'Castle'] as BuildingKind[];
    const open = city(setup.open);
    open.buildings = [];
    open.producing = { kind: 'Building', building: 'Walls' } as typeof open.producing;
    open.prod = 40; open.productionQueue = [];
    await loadFixture(page, coreFixture(f), 2);
    for (const [name, zoom] of [['normal', 1.3], ['near', 2.2], ['far', 0.6]] as const) {
      await frame(page, fromOffset(10, 5), zoom);
      await shot(page, size, `wall-levels-${name}`);
    }
    // Hover the Castle capital's name plate: the banner tooltip is where the wall level gets its words.
    await frame(page, city(setup.walled).pos, 2.2);
    const plate = await page.evaluate((id) => {
      const w = window as unknown as { legatumMap: { banners: { city: number; rect: { x: number; y: number; w: number; h: number } }[] } };
      return w.legatumMap.banners.find((b) => b.city === id)?.rect ?? null;
    }, setup.walled);
    if (plate) {
      const box = (await page.locator('[data-game-map]').boundingBox())!;
      await page.mouse.move(box.x + plate.x + plate.w * 0.75, box.y + plate.y + plate.h / 2);
      await page.waitForTimeout(900);
      await shot(page, size, 'wall-levels-banner-hover');
    }
    await frame(page, open.pos, 2.6);
    await shot(page, size, 'wall-levels-rival-building');
    await frame(page, city(setup.own).pos, 2.6);
    await shot(page, size, 'wall-levels-capital-iii');
  });
});
