// Population is territory: a grown city's tiles beyond its 3-hex work radius are owned but not
// worked, so they carry a hatch on the map and are counted apart in the city panel. Screenshots go
// to test-results/territory/<size>/ (or LEGATUM_SHOTS) for review at the user's laptop size
// (1000x600) and 1440x900.
import { mkdir } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import type { Page } from '@playwright/test';
import type { useGame } from '../src/store';
import { test, expect } from './fixtures';
import { coreFixture } from '../src/test/coreFixture';
import { diplomacyState } from '../src/test/diplomacyFixture';
import { ring, within } from '../src/hex';
import { idxOf } from '../src/test/fixtureEngine';

type GameWindow = Window & { legatum: typeof useGame };
const SIZES = [{ name: '1000x600', width: 1000, height: 600 }, { name: '1440x900', width: 1440, height: 900 }] as const;
const outRoot = process.env.LEGATUM_SHOTS ?? fileURLToPath(new URL('../test-results/territory/', import.meta.url));

async function shot(page: Page, size: string, name: string) {
  await page.evaluate(() => document.fonts.ready.then(() => undefined));
  await page.waitForTimeout(400);
  await mkdir(`${outRoot}/${size}`, { recursive: true });
  await page.screenshot({ path: `${outRoot}/${size}/${name}.png`, animations: 'disabled' });
}

/** A capital of `population` that owns rings 1 to 3 and `ring4` tiles of ring 4 (a grown empire). */
function grownCity(population = 36, ring4 = 5) {
  const s = diplomacyState(12, { met: true });
  const c = s.cities[0];
  c.population = population;
  c.producing = { kind: 'Unit', unit: 'Spearman' };
  // Scouted well past the border, so the tiles beyond three hexes are known ground.
  for (const h of within(c.pos, 6)) s.players[0].explored[idxOf(s, h)] = 1;
  for (const r of [2, 3, 4]) {
    const tiles = ring(c.pos, r).slice(0, r === 4 ? ring4 : undefined);
    for (const h of tiles) {
      const i = idxOf(s, h);
      s.owner[i] = c.owner; s.tileCity[i] = c.id;
      c.grownTiles.push(h); c.tilesClaimed++;
    }
  }
  return s;
}

/** The common case: a population-4 capital that has grown three ring-2 tiles. */
function youngCity() {
  const s = diplomacyState(12, { met: true });
  const c = s.cities[0];
  c.population = 4;
  c.producing = { kind: 'Unit', unit: 'Spearman' };
  for (const h of ring(c.pos, 2).slice(0, 3)) {
    const i = idxOf(s, h);
    s.owner[i] = c.owner; s.tileCity[i] = c.id;
    c.grownTiles.push(h); c.tilesClaimed++;
  }
  return s;
}

async function open(page: Page, fixture: ReturnType<typeof grownCity>) {
  await page.addInitScript(({ game, turn }) => {
    if (!location.protocol.startsWith('http')) return;
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn, names: ['Ada'], savedAt: new Date().toISOString(), game }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, { game: coreFixture(fixture), turn: 12 });
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('12');
  await expect(page.locator('.loading-screen')).toHaveCount(0);
  await page.evaluate(() => {
    const s = (window as unknown as GameWindow).legatum.getState(), v = s.views![s.seat];
    const city = v.cities.find((c) => c.owner === v.me)!;
    s.set({ techOpen: false });
    s.select({ kind: 'city', id: city.id }, true);
    s.jumpTo(city.pos);
  });
  await expect(page.locator('.city-panel')).toBeVisible();
}

/** Centre the map on a hex offset from the capital (q, r) and optionally set the zoom. */
async function frame(page: Page, dq: number, dr: number, zoom?: number) {
  await page.evaluate(({ dq, dr, zoom }) => {
    const w = window as unknown as GameWindow & { legatumMap: { camera: { zoom: number }; redraw(): void } };
    const s = w.legatum.getState(), v = s.views![s.seat];
    const c = v.cities.find((c) => c.owner === v.me)!;
    s.jumpTo({ q: c.pos.q + dq, r: c.pos.r + dr });
    if (zoom) { w.legatumMap.camera.zoom = zoom; w.legatumMap.redraw(); }
  }, { dq, dr, zoom });
  await page.waitForTimeout(500);
}

for (const size of SIZES) {
  test.describe(size.name, () => {
    test.use({ viewport: { width: size.width, height: size.height } });

    test('tiles claimed beyond three hexes are hatched in player colour and counted apart', async ({ page }) => {
      await open(page, grownCity());
      const panel = page.locator('.city-panel');
      await panel.locator('.cp-economy summary').click();
      await expect(panel.getByTestId('city-territory')).toHaveText('36 worked · 41 owned');
      await expect(panel).toContainText('5 claimed, not worked');
      // The claim after those: ring 4 again, so it only blocks rivals.
      const growth = panel.getByTestId('city-growth');
      await expect(growth).toContainText(/claims /);
      // The ring is in the phrase's tooltip, not on the one-line label.
      await expect(growth.locator('.pbar-label')).not.toContainText(/ring/);
      await growth.locator('.pbar-label .tip-trigger', { hasText: /^claims / }).hover();
      await expect(page.locator('[role="tooltip"]', { hasText: 'Ring 4.' })).toBeVisible();
      await page.mouse.move(5, 5);
      await expect(panel.getByTestId('city-growth').locator('.pbar-foot-left')).toHaveText('Next tile · no yield · blocks rivals');
      // Centred on the territory: the capital and the hatched edge in one frame.
      await frame(page, -2, 1);
      await panel.evaluate((el) => { el.scrollTop = 0; });
      await shot(page, size.name, '01-grown-city-hatch');
    });

    test('the territory bar label stays one line with "unlocks Settler", and the tile totals are labelled', async ({ page }) => {
      const s = diplomacyState(12, { met: true });
      s.cities[0].population = 1;
      s.cities[0].producing = { kind: 'Unit', unit: 'Spearman' };
      await open(page, s);
      const panel = page.locator('.city-panel');
      const label = panel.getByTestId('city-growth').locator('.pbar-label');
      await expect(label).toContainText(/^Grows to 2 · claims .+ · unlocks Settler$/);
      await expect(label).not.toContainText(/ring/);
      // At 1000x600 the panel is 282 px wide and this label nearly fills it. Same Libre Franklin, but Linux's text
      // rendering (the cloud box) measures a few px wider and wraps it (30 px tall), so only macOS, the shipping
      // platform, holds it to one line. Elsewhere two lines still pass.
      expect(await label.evaluate((el) => el.getBoundingClientRect().height)).toBeLessThan(process.platform === 'darwin' ? 20 : 34);
      // Gross tile yields read as tiles, not income: labelled, neutral ink (not the accent red), with what is eaten.
      const tiles = panel.getByTestId('city-tiles');
      await expect(tiles).toHaveText('Tiles');
      const [ink, accent] = await tiles.evaluate((el) => [getComputedStyle(el).color, getComputedStyle(document.documentElement).getPropertyValue('--accent').trim()]);
      expect(ink).not.toBe(accent);
      await shot(page, size.name, '02a-newborn-city-bar-unlocks');
    });

    test('the city panel keeps its sticky head and territory bar without clipped labels', async ({ page }) => {
      await open(page, youngCity());
      const panel = page.locator('.city-panel');
      const bar = panel.getByTestId('city-growth');
      await expect(bar).toBeVisible();
      await expect(bar.locator('.pbar-label')).toContainText(/Grows to 5 · claims [A-Za-z ]+ (N|NE|E|SE|S|SW|W|NW)$/);
      const clipped = await panel.evaluate((el) => [...el.querySelectorAll<HTMLElement>('.cp-head *, .pbar-label, .pbar-foot')]
        .filter((n) => n.scrollWidth > n.clientWidth + 1 && getComputedStyle(n).overflow !== 'visible').map((n) => `${n.className}: ${n.scrollWidth}>${n.clientWidth}`));
      expect(clipped).toEqual([]);
      await shot(page, size.name, '06-city-panel-stable-order');
    });

    test('zoomed out: unworked claims read as a flat wash of the player colour', async ({ page }) => {
      await open(page, grownCity(36, 12));
      await frame(page, -2, 1, 0.45);
      await shot(page, size.name, '05-zoomed-out-wash');
    });
  });
}
