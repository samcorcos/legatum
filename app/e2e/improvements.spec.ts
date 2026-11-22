// City-built improvements (playtest 8): the production picker's Improvements group, the map-first
// tile highlight, dormant resources, and the painted land art with far-zoom badges. Screenshots go to
// test-results/improvements/<size>/ (or LEGATUM_SHOTS) for review at 1000x600 and 1440x900.
import { mkdir } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import type { Page } from '@playwright/test';
import type { useGame } from '../src/store';
import { test, expect } from './fixtures';
import { coreFixture } from '../src/test/coreFixture';
import { improvementCity, UPGRADE_TECHS } from '../src/test/improvementsFixture';
import { endPlanning } from './turnActions';

type GameWindow = Window & { legatum: typeof useGame };
const SIZES = [{ name: '1000x600', width: 1000, height: 600 }, { name: '1440x900', width: 1440, height: 900 }] as const;
const outRoot = process.env.LEGATUM_SHOTS ?? fileURLToPath(new URL('../test-results/improvements/', import.meta.url));

async function shot(page: Page, size: string, name: string) {
  await page.evaluate(() => document.fonts.ready.then(() => undefined));
  await page.waitForTimeout(500);
  await mkdir(`${outRoot}/${size}`, { recursive: true });
  await page.screenshot({ path: `${outRoot}/${size}/${name}.png`, animations: 'disabled' });
}

async function open(page: Page, fixture: ReturnType<typeof improvementCity>) {
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

async function frame(page: Page, zoom: number, dq = 0, dr = 0) {
  await page.evaluate(({ dq, dr, zoom }) => {
    const w = window as unknown as GameWindow & { legatumMap: { camera: { zoom: number }; redraw(): void } };
    const s = w.legatum.getState(), v = s.views![s.seat];
    const c = v.cities.find((c) => c.owner === v.me)!;
    s.jumpTo({ q: c.pos.q + dq, r: c.pos.r + dr });
    w.legatumMap.camera.zoom = zoom; w.legatumMap.redraw();
  }, { dq, dr, zoom });
  await page.waitForTimeout(600);
}

for (const size of SIZES) {
  test.describe(size.name, () => {
    test.use({ viewport: { width: size.width, height: size.height } });

    test('the picker lists every eligible tile with its forecast, lights the tile on hover and builds it', async ({ page }) => {
      await open(page, improvementCity());

      const group = page.getByTestId('improvements-group');
      await expect(group).toBeVisible();
      // Iron stays hidden until Iron Working. At Bronze Working the two Wheat, Cattle, Copper and Stone are eligible.
      const rows = group.locator('.production-choice');
      await expect(rows).toHaveCount(5);
      await expect(group).toContainText(/Farm, Wheat\s*1\stile/);
      await expect(group).not.toContainText('Mine, Iron');
      await expect(group).toContainText('5 resource tiles unimproved');
      const wheat = rows.filter({ hasText: /Farm, Wheat\s*1\stile\s/ });
      await wheat.locator('.production-more > summary').click();
      await expect(wheat.locator('.production-more-text')).toContainText(/\+1 Food when built · Later: Irrigated Farm with Crop Rotation, \+1 Food/);
      await wheat.locator('.production-more > summary').click();
      await expect(group).not.toContainText(/\(\d+, ?\d+\)/);
      await group.scrollIntoViewIfNeeded();
      await frame(page, 1.0);
      // Map first: hovering a row rings its tile on the map.
      await rows.nth(2).locator('button.prod').hover();
      expect(await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().improve.hover)).not.toBeNull();
      await shot(page, size.name, '01-picker-hover');
      // Choosing it queues the build; the order names the tile.
      await rows.filter({ hasText: /Farm, Wheat\s*1\stile\s/ }).locator('button.prod').click();
      const queued = await page.evaluate(() => { const s = (window as unknown as GameWindow).legatum.getState(); return JSON.stringify(s.orders[s.seat]); });
      expect(queued).toContain('"improvement":"Farm"');
      const food = () => page.evaluate(() => { const s = (window as unknown as GameWindow).legatum.getState(), v = s.views![s.seat]; return v.cities.find((c) => c.owner === v.me)!.detail!.yields.total.food; });
      const before = await food();
      // End turns until it completes: the tile then shows the improvement and the city gains its Food.
      for (let turn = 0; turn < 8; turn++) {
        const done = await page.evaluate(() => { const s = (window as unknown as GameWindow).legatum.getState(), v = s.views![s.seat]; return v.tiles.some((t) => t.improvement); });
        if (done) break;
        await endPlanning(page);
        await expect(page.getByTestId('turn-number')).not.toHaveText(String(12 + turn), { timeout: 15_000 });
      }
      expect(await page.evaluate(() => { const s = (window as unknown as GameWindow).legatum.getState(), v = s.views![s.seat]; return v.tiles.filter((t) => t.improvement).map((t) => t.improvement!.kind); })).toEqual(['Farm']);
      expect(await food()).toBeGreaterThan(before);
    });

    test('the land art at a comfortable zoom: base forms', async ({ page }) => {
      await open(page, improvementCity({ improved: true }));
      await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().select(null));
      await frame(page, 1.25);
      await shot(page, size.name, '03-art-base');
    });

    test('the land art at a comfortable zoom: upgraded forms', async ({ page }) => {
      await open(page, improvementCity({ improved: true, techs: ['CropRotation', 'IronWorking'] }));
      await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().select(null));
      await frame(page, 1.25);
      await shot(page, size.name, '04-art-upgraded');
    });

    test('zoomed out: a solid ink ring on the resource marker is the improvement badge', async ({ page }) => {
      await open(page, improvementCity({ improved: true }));
      await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().select(null));
      await frame(page, 0.55);
      await shot(page, size.name, '05-badges');
    });

    test('the far tier: improved resource dots wear a solid ring, dormant ones a dashed one', async ({ page }) => {
      await open(page, improvementCity({ improved: true, techs: UPGRADE_TECHS }));
      await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().select(null));
      await frame(page, 0.38);
      await shot(page, size.name, '07-badges-far');
    });

    test('dormant resources: your own unimproved resources read as waiting', async ({ page }) => {
      await open(page, improvementCity({ techs: UPGRADE_TECHS }));
      await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().select(null));
      await frame(page, 1.25);
      await shot(page, size.name, '06-dormant');
    });
  });
}
