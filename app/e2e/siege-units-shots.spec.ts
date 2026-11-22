// Siege on the map (siege study, user 2026-10-18; Siege upgrade 2026-10-22; era caps 2026-10-28): an archer and a spearman with Siege wear siege
// gear beside them, a spearman without it does not, a Renaissance Arquebusier and AssaultTrooper with Siege wear a bombard and ladders, an
// Industrial Rifleman and Field Cannon (Siege built in) wear nothing, the forecast against the walled city says what Walls do, and the unit's
// own Siege note. One shot per era band (8.9.16 review). WASM engine. SIEGE_SHOTS=<dir> picks where the shots go.
import { test as base, expect, type Page } from '@playwright/test';
import { createServer } from 'vite';
import { mkdir } from 'node:fs/promises';
import path from 'node:path';
import type { Hex } from '../src/types';

type MapWindow = Window & { legatumMap: { camera: { zoom: number }; redraw(): void; toScreen(h: Hex): { x: number; y: number } } };
const out = process.env.SIEGE_SHOTS ? path.resolve(process.env.SIEGE_SHOTS) : path.resolve('test-results/siege');
const test = base.extend<{}, { devURL: string }>({
  devURL: [async ({}, use) => {
    const server = await createServer({ server: { host: '127.0.0.1', port: 0, strictPort: false } });
    await server.listen();
    try {
      const address = server.httpServer!.address();
      if (!address || typeof address === 'string') throw new Error('Vite did not bind');
      await use(`http://127.0.0.1:${address.port}`);
    } finally { await server.close(); }
  }, { scope: 'worker' }],
  baseURL: async ({ devURL }, use) => { await use(devURL); },
});

async function boot(page: Page, size: { width: number; height: number }, tier: 'Mathematics' | 'Machinery' | 'Metallurgy' = 'Mathematics') {
  await mkdir(out, { recursive: true });
  await page.addInitScript(() => localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] })));
  await page.setViewportSize(size);
  await page.goto('/?client=wasm');
  await expect(page.getByRole('button', { name: /^Single Player/ })).toBeVisible();
  await page.evaluate(async (t) => {
    const file = '/src/test/siegeUnitsFixture.ts';
    const { showSiegeUnitsFixture } = await import(/* @vite-ignore */ file);
    const fx = showSiegeUnitsFixture(t);
    (window as unknown as { siegeFx: unknown }).siegeFx = { centre: fx.centre, east: fx.east, spear: fx.spear, archer: fx.archer, rifleman: fx.rifleman, bare: fx.bare, cityAt: fx.cityAt };
  }, tier);
  await expect(page.locator('[data-game-map]')).toBeVisible();
  await page.evaluate(() => {
    const w = window as unknown as MapWindow;
    const fx = (window as unknown as { siegeFx: { centre: Hex } }).siegeFx;
    // Every unit of the fixture (one per era band) is framed between the unit panel and the sidebar at 1000x600.
    w.legatumMap.camera.zoom = 0.95;
    (window as unknown as { legatum: { getState(): { jumpTo(h: Hex): void } } }).legatum.getState().jumpTo(fx.centre);
    w.legatumMap.redraw();
  });
  await page.evaluate(() => document.fonts.ready.then(() => undefined));
  await page.waitForTimeout(1200);
}
const shot = (page: Page, size: { width: number }, name: string) => page.screenshot({ path: path.join(out, `${name}-${size.width}.png`), animations: 'disabled' });

for (const size of [{ width: 1000, height: 600 }, { width: 1440, height: 900 }]) {
  test.describe(`${size.width}x${size.height}`, () => {
    for (const tier of ['Mathematics', 'Machinery', 'Metallurgy'] as const) {
      test(`map: units with Siege wear siege gear, tier ${tier}`, async ({ page }) => {
        test.setTimeout(90_000);
        await boot(page, size, tier);
        await page.mouse.move(size.width - 2, size.height - 2);
        await page.waitForTimeout(400);
        await shot(page, size, `01-map-${tier.toLowerCase()}`);
      });
    }

    test('map: every unit of the fixture is on screen, clear of the panels', async ({ page }) => {
      test.setTimeout(90_000);
      await boot(page, size, 'Metallurgy');
      const boxes = await page.evaluate(() => {
        const w = window as unknown as MapWindow & { legatum: { getState(): { views: { units: { kind: string; pos: Hex }[] }[] } } };
        const map = document.querySelector('[data-game-map]')!.getBoundingClientRect();
        return w.legatum.getState().views[0].units.map((u) => {
          const p = w.legatumMap.toScreen(u.pos), x = map.x + p.x, y = map.y + p.y;
          // The tile's centre and a point 0.8 of a hex to its right (where the gear stands) both show the map, not a panel.
          const open = [x, x + 40].every((px) => !!document.elementFromPoint(px, y)?.closest('[data-game-map]'));
          return { kind: u.kind, open };
        });
      });
      expect(boxes.map((b) => b.kind).sort()).toEqual(['Arquebusier', 'AssaultTrooper', 'FieldCannon', 'Ranger', 'Rifleman', 'Scout', 'Soldier', 'Soldier']);
      for (const b of boxes) expect(b, `${b.kind} is framed at ${size.width}x${size.height}`).toMatchObject({ open: true });
    });

    test('unit panel: an Industrial Rifleman has Siege built in and no Siege row', async ({ page }) => {
      test.setTimeout(90_000);
      await boot(page, size, 'Metallurgy');
      // A bare Soldier can still buy Siege; the Industrial Rifleman has it built in and offers no row.
      await page.evaluate(() => {
        const fx = (window as unknown as { siegeFx: { bare: number } }).siegeFx;
        (window as unknown as { legatum: { getState(): { select(s: { kind: 'unit'; id: number }, jump?: boolean): void } } }).legatum.getState().select({ kind: 'unit', id: fx.bare }, true);
      });
      await expect(page.locator('.improve-row .improve-copy', { hasText: /^Siege/ })).toHaveCount(1);
      await page.evaluate(() => {
        const fx = (window as unknown as { siegeFx: { rifleman: number } }).siegeFx;
        (window as unknown as { legatum: { getState(): { select(s: { kind: 'unit'; id: number }, jump?: boolean): void } } }).legatum.getState().select({ kind: 'unit', id: fx.rifleman }, true);
      });
      await expect(page.getByTestId('siege-note')).toContainText('Siege built in');
      await expect(page.locator('.improve-row .improve-copy', { hasText: /^Siege/ })).toHaveCount(0);
      await page.mouse.move(size.width - 2, size.height - 2);
      await page.waitForTimeout(400);
      const note = page.getByTestId('siege-note');
      expect(await note.evaluate((el) => el.scrollWidth <= el.clientWidth + 1), 'the note is whole').toBe(true);
      await shot(page, size, '03-unit-panel-rifleman');
    });

    test('forecast and unit panel', async ({ page }) => {
      test.setTimeout(90_000);
      await boot(page, size);
      await page.evaluate(() => {
        const w = window as unknown as MapWindow;
        const fx = (window as unknown as { siegeFx: { east: Hex } }).siegeFx;
        w.legatumMap.camera.zoom = 0.9;
        (window as unknown as { legatum: { getState(): { jumpTo(h: Hex): void } } }).legatum.getState().jumpTo(fx.east);
        w.legatumMap.redraw();
      });
      await page.waitForTimeout(1200);
      await expect(page.getByTestId('siege-note')).toBeVisible();
      const b = await page.locator('[data-game-map]').boundingBox();
      const hoverCity = async () => {
        const p = await page.evaluate(() => { const fx = (window as unknown as { siegeFx: { cityAt: Hex } }).siegeFx; return (window as unknown as MapWindow).legatumMap.toScreen(fx.cityAt); });
        await page.mouse.move(b!.x + p.x + 3, b!.y + p.y + 3);
        await page.mouse.move(b!.x + p.x, b!.y + p.y, { steps: 3 });
      };
      await expect.poll(async () => { await hoverCity(); return page.getByTestId('forecast-walls').first().isVisible(); }, { intervals: [400, 800, 1000, 2000] }).toBe(true);
      await expect(page.getByTestId('forecast-walls').first()).toContainText('Siege: your Spearman ignores');
      await shot(page, size, '02-forecast-covered');
    });

    test('forecast: an Industrial Rifleman against the walled city says Siege is built in', async ({ page }) => {
      test.setTimeout(90_000);
      await boot(page, size, 'Metallurgy');
      await page.evaluate(() => {
        const w = window as unknown as MapWindow;
        const fx = (window as unknown as { siegeFx: { east: Hex; rifleman: number } }).siegeFx;
        (window as unknown as { legatum: { getState(): { select(s: { kind: 'unit'; id: number }, jump?: boolean): void; jumpTo(h: Hex): void } } }).legatum.getState().select({ kind: 'unit', id: fx.rifleman }, true);
        w.legatumMap.camera.zoom = 0.9;
        (window as unknown as { legatum: { getState(): { jumpTo(h: Hex): void } } }).legatum.getState().jumpTo(fx.east);
        w.legatumMap.redraw();
      });
      await page.waitForTimeout(1200);
      const b = await page.locator('[data-game-map]').boundingBox();
      const hoverCity = async () => {
        const p = await page.evaluate(() => { const fx = (window as unknown as { siegeFx: { cityAt: Hex } }).siegeFx; return (window as unknown as MapWindow).legatumMap.toScreen(fx.cityAt); });
        await page.mouse.move(b!.x + p.x + 3, b!.y + p.y + 3);
        await page.mouse.move(b!.x + p.x, b!.y + p.y, { steps: 3 });
      };
      await expect.poll(async () => { await hoverCity(); return page.getByTestId('forecast-walls').first().isVisible(); }, { intervals: [400, 800, 1000, 2000] }).toBe(true);
      await expect(page.getByTestId('forecast-walls').first()).toContainText('Siege built in: your Rifleman ignores');
      await shot(page, size, '04-forecast-rifleman');
    });
  });
}
