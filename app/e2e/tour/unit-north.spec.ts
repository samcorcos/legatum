// A unit on any tile around a city never covers the city's banner (tour 2026-11-22, §4: a token on
// the hex above hid the name and the meters). Zoomed in, banners draw above units, as in Civ 6.
// Each case stands one unit around a garrisoned city, at the two zooms where the banner and the
// hex above overlap most, and checks the banner's pixels match the case with no unit beside it.
import { test, expect } from '../fixtures';
import type { Page } from '@playwright/test';
import { coreFixture } from '../../src/test/coreFixture';
import { improvementCity } from '../../src/test/improvementsFixture';
import { spawnUnit, refreshVision } from '../../src/test/fixtureEngine';
import { hexEq, neighbors } from '../../src/hex';
import type { Hex } from '../../src/types';
import { SIZES, frameHall, hexPoint, loadFixture, parkMouse, shot, type GameWindow } from './helpers';

// DIRECTIONS order: E, NE, NW, W, SW, SE; "two-up" is the tile straight above the hex above.
const CASES = ['none', 'e', 'ne', 'nw', 'w', 'sw', 'se', 'two-up'] as const;
type Case = (typeof CASES)[number];
const ZOOMS = [0.62, 0.7] as const;

function spot(city: Hex, c: Case): Hex | null {
  if (c === 'none') return null;
  if (c === 'two-up') return { q: city.q + 1, r: city.r - 2 };
  return neighbors(city)[['e', 'ne', 'nw', 'w', 'sw', 'se'].indexOf(c)];
}

function scene(c: Case) {
  const s = improvementCity();
  const city = s.cities[0];
  city.name = 'Ajmer';
  s.units = s.units.filter((u) => u.owner !== 0 || hexEq(u.pos, city.pos));
  if (!s.units.some((u) => hexEq(u.pos, city.pos))) spawnUnit(s, 0, 'Spearman', city.pos, false);
  const at = spot(city.pos, c);
  s.units = s.units.filter((u) => !at || hexEq(u.pos, city.pos) || !hexEq(u.pos, at));
  const unit = at ? spawnUnit(s, 0, 'Spearman', at, false) : null;
  refreshVision(s);
  return { s, city, unit };
}

type Box = { x: number; y: number; w: number; h: number };
type Banner = { city: number; rect: Box; cover: Box; far: boolean };

/** The canvas pixels of a box on the map (CSS pixels, relative to the canvas). */
const pixels = (page: Page, box: Box) => page.evaluate((b) => {
  const canvas = document.querySelector<HTMLCanvasElement>('[data-game-map]')!;
  const k = canvas.width / canvas.getBoundingClientRect().width;
  return Array.from(canvas.getContext('2d')!.getImageData(Math.round(b.x * k), Math.round(b.y * k), Math.round(b.w * k), Math.round(b.h * k)).data);
}, box);

/** The city's banner as last drawn: its name plate and its cover (plate and meters). */
const bannerOf = (page: Page, city: number) => page.evaluate((city) =>
  (window as unknown as { legatumMap: { banners: Banner[] } }).legatumMap.banners.find((x) => x.city === city)!, city);

/** The strip just above the banner over a hex's centre, where a token behind the banner shows its head. */
async function above(page: Page, banner: Banner, hex: Hex): Promise<Box> {
  const map = (await page.locator('[data-game-map]').boundingBox())!;
  const p = await hexPoint(page, hex);
  return { x: p.x - map.x - 6, y: banner.cover.y - 9, w: 12, h: 8 };
}

function differing(a: number[], b: number[]) {
  if (a.length !== b.length) return 1;
  let n = 0;
  for (let i = 0; i < a.length; i += 4) {
    if (Math.abs(a[i] - b[i]) + Math.abs(a[i + 1] - b[i + 1]) + Math.abs(a[i + 2] - b[i + 2]) > 48) n++;
  }
  return n / (a.length / 4);
}

for (const size of SIZES) test.describe(`unit north ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });
  for (const zoom of ZOOMS) test(`banner over units at zoom ${zoom}`, async ({ page }) => {
    test.setTimeout(240_000);
    let reference: number[] | null = null;
    const empty = new Map<string, number[]>();
    for (const c of CASES) {
      const { s, city, unit } = scene(c);
      await loadFixture(page, coreFixture(s), s.turn);
      await frameHall(page, city.pos, zoom);
      await parkMouse(page);
      await page.waitForTimeout(300);
      await shot(page, size, `un-${c}-z${Math.round(zoom * 100)}`);
      const banner = await bannerOf(page, city.id);
      expect(banner.far, 'zoomed in, the city wears a full banner').toBe(false);
      const drawn = await pixels(page, banner.cover);
      if (!reference) {
        reference = drawn;
        for (const k of ['ne', 'nw'] as const) empty.set(k, await pixels(page, await above(page, banner, spot(city.pos, k)!)));
        continue;
      }
      // The banner reads the same whatever stands around it. The slack is for the token showing through the
      // gap between the plate and the meters and past the plate's pointed ends (about 2-3% on the hex above).
      expect.soft(differing(reference, drawn), `${c}: banner pixels differ from the empty case`).toBeLessThan(0.05);
      // A token on the hex above is raised so its head shows above the banner, not hidden behind it.
      if (unit && (c === 'ne' || c === 'nw')) {
        expect.soft(differing(empty.get(c)!, await pixels(page, await above(page, banner, unit.pos))), `${c}: the token's head shows above the banner`).toBeGreaterThan(0.25);
      }
      if (c !== 'ne' || !unit) continue;
      // The banner stays clickable where it overhangs the hex above, and hovering it shows its card.
      const map = (await page.locator('[data-game-map]').boundingBox())!;
      const unitAt = await hexPoint(page, unit.pos);
      const over = { x: Math.round(Math.max(banner.rect.x + 4, unitAt.x - map.x)), y: Math.round(banner.rect.y + banner.rect.h / 2) };
      await page.mouse.move(map.x + over.x, map.y + over.y, { steps: 3 });
      await expect(page.getByTestId('banner-tooltip')).toBeVisible();
      await page.mouse.click(map.x + over.x, map.y + over.y);
      const picked = () => page.evaluate(() => (window as unknown as GameWindow).legatum.getState().selection);
      expect(await picked(), 'a click on the banner never picks the unit behind it').not.toEqual({ kind: 'unit', id: unit.id });
      // The token's top, clear of the banner, still selects the unit.
      const top = { x: unitAt.x, y: Math.round(map.y + banner.cover.y) - 4 };
      await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().select(null));
      await page.mouse.click(top.x, top.y);
      await expect.poll(picked).toEqual({ kind: 'unit', id: unit.id });
    }
  });
});
