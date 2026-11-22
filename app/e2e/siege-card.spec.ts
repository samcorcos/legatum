// The Siege upgrade on the unit card (user decision 2026-10-22): a spearman without Siege offers it with its Gold price and the Iron it
// uses, says why it cannot be bought now, and queues the order when it can. WASM engine. SIEGE_SHOTS=<dir> picks where the shots go.
// SIEGE_SHOTS=<dir> picks where the shots go.
import { test as base, expect, type Page } from '@playwright/test';
import { createServer } from 'vite';
import { mkdir } from 'node:fs/promises';
import path from 'node:path';
import type { Hex } from '../src/types';

type MapWindow = Window & { legatumMap: { camera: { zoom: number }; redraw(): void; toScreen(h: Hex): { x: number; y: number } } };
const out = process.env.SIEGE_SHOTS ? path.resolve(process.env.SIEGE_SHOTS) : path.resolve('test-results/siege-card');
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
    (window as unknown as { siegeFx: unknown }).siegeFx = { centre: fx.centre, east: fx.east, spear: fx.spear, archer: fx.archer, bare: fx.bare, cityAt: fx.cityAt };
  }, tier);
  await expect(page.locator('[data-game-map]')).toBeVisible();
  await page.evaluate(() => {
    const w = window as unknown as MapWindow;
    const fx = (window as unknown as { siegeFx: { centre: Hex } }).siegeFx;
    w.legatumMap.camera.zoom = 1.25;
    (window as unknown as { legatum: { getState(): { jumpTo(h: Hex): void } } }).legatum.getState().jumpTo(fx.centre);
    w.legatumMap.redraw();
  });
  await page.evaluate(() => document.fonts.ready.then(() => undefined));
  await page.waitForTimeout(1200);
}
const shot = (page: Page, size: { width: number }, name: string) => page.screenshot({ path: path.join(out, `${name}-${size.width}.png`), animations: 'disabled' });

for (const size of [{ width: 1000, height: 600 }, { width: 1440, height: 900 }]) {
  test.describe(`${size.width}x${size.height}`, () => {
    test('a unit without Siege offers it with its price and Iron, blocked until Iron is revealed and spare', async ({ page }) => {
      test.setTimeout(120_000);
      await boot(page, size);
      // The spearman without Siege stands in your own land, beside the capital.
      await page.evaluate(() => {
        const g = (window as unknown as { legatum: { getState(): { client: { debugState(): { owner: number[]; width: number; units: { id: number; pos: { q: number; r: number } }[]; cities: { pos: { q: number; r: number } }[] }; view(p: number): unknown }; select(s: unknown, jump?: boolean): void; set(s: unknown): void } } }).legatum.getState();
        const fx = (window as unknown as { siegeFx: { bare: number } }).siegeFx;
        const f = g.client.debugState();
        const city = f.cities[0].pos;
        const bare = f.units.find((u) => u.id === fx.bare)!;
        bare.pos = { q: city.q + 1, r: city.r };
        g.set({ views: [g.client.view(0)] });
        g.select({ kind: 'unit', id: fx.bare }, true);
      });
      await page.waitForTimeout(600);
      const siege = page.getByRole('button', { name: /^Siege/ });
      await expect(siege).toBeVisible();
      const siegeRow = page.locator('.improve-row').filter({ has: siege });
      await expect(siegeRow.locator('.improve-cost')).toContainText('160');
      await expect(siegeRow.locator('.improve-supply')).toContainText('Uses 1 Iron');
      await expect(siege).toBeDisabled();
      await expect(page.locator('.act-reason').first()).toContainText('Iron');
      await shot(page, size, '01-siege-blocked');
      // A deposit remains hidden until Iron Working. Learning it reveals Iron and makes the spare supply usable.
      await page.evaluate(() => {
        const g = (window as unknown as { legatum: { getState(): { client: { debugState(): { resource: (string | null)[]; owner: number[]; players: { techs: string[] }[] }; view(p: number): unknown }; set(s: unknown): void } } }).legatum.getState();
        const f = g.client.debugState();
        // Four units with Siege (archer, spearman, Arquebusier, Assault Trooper) already use 4 Iron: three deposits (2 supply each) leave some to spare.
        let placed = 0;
        for (let k = 0; k < f.owner.length && placed < 3; k++) if (f.owner[k] === 0 && !f.resource[k]) { f.resource[k] = 'Iron'; placed++; }
        g.set({ views: [g.client.view(0)] });
      });
      await page.waitForTimeout(600);
      await expect(siege).toBeDisabled();
      await page.evaluate(() => {
        const g = (window as unknown as { legatum: { getState(): { client: { debugState(): { players: { techs: string[] }[] }; view(p: number): unknown }; set(s: unknown): void } } }).legatum.getState();
        g.client.debugState().players[0].techs.push('IronWorking');
        g.set({ views: [g.client.view(0)] });
      });
      await expect(siege).toBeEnabled();
      await shot(page, size, '02-siege-buyable');
      await siege.click();
      await expect(page.locator('.up-status')).toContainText('Buy Siege');
      await shot(page, size, '03-siege-ordered');
    });
  });
}
