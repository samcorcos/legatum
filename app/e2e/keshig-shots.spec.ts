// The Mongol Keshig (user, 2026-10-17): its art beside the Horseman and the Cataphract it replaces, its build card, the Mongol faction sheet, the Iron
// Working card (the Keshig upgrades to the Cataphract and loses the bow) and the two-option forecast, at the user's
// laptop size and at 1440x900. KESHIG_SHOTS=<dir> picks where the shots go. WASM engine.
import { test as base, expect, type Page } from '@playwright/test';
import { createServer } from 'vite';
import { mkdir } from 'node:fs/promises';
import path from 'node:path';
import type { Hex } from '../src/types';
import { coreFixture } from '../src/test/coreFixture';
import { keshigFixture } from '../src/test/keshigFixture';
import { MELEE_VS_MOUNTED_LABEL } from '../src/combat';

type MapWindow = Window & { legatumMap: { camera: { zoom: number }; redraw(): void; toScreen(h: Hex): { x: number; y: number } } };
const out = process.env.KESHIG_SHOTS ? path.resolve(process.env.KESHIG_SHOTS) : path.resolve('test-results/keshig');
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

async function boot(page: Page, size: { width: number; height: number }) {
  await mkdir(out, { recursive: true });
  await page.addInitScript(() => {
    if (location.protocol.startsWith('http')) localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  });
  await page.setViewportSize(size);
  await page.goto('/?client=wasm');
  await expect(page.getByRole('button', { name: /^Single Player/ })).toBeVisible();
  await page.evaluate(() => document.fonts.ready);
}
const shot = (page: Page, size: { width: number }, name: string) => page.screenshot({ path: path.join(out, `${name}-${size.width}.png`), animations: 'disabled' });

for (const size of [{ width: 1000, height: 600 }, { width: 1440, height: 900 }]) {
  test.describe(`${size.width}x${size.height}`, () => {
    test('art: the Keshig reads apart from the Horseman and the Archer, as icon, roundel and miniature', async ({ page }) => {
      await boot(page, size);
      await page.evaluate(async () => {
        const svg = async (url: string) => (await fetch(url)).text();
        const names = ['unit-keshig', 'unit-horseman', 'unit-archer', 'unit-cataphract'];
        const roundels = ['keshig', 'horseman', 'archer', 'cataphract'];
        const icons = await Promise.all(names.map((n) => svg(`/src/assets/icons/${n}.svg`)));
        const rounds = await Promise.all(roundels.map((n) => svg(`/src/assets/units/${n}-roundel.svg`)));
        document.body.innerHTML = `<div style="background:#efe6d0;color:#2b2419;padding:16px;font:12px sans-serif">
          ${[16, 24, 48].map((px) => `<div style="display:flex;gap:18px;align-items:center;margin:8px 0">${icons.map((s) => `<span style="width:${px}px;height:${px}px;display:inline-block">${s.replace('<svg ', `<svg width="${px}" height="${px}" `)}</span>`).join('')}
            <span style="width:30px"></span>${rounds.map((s) => `<span style="color:#4a74a8;display:inline-block;width:${px}px;height:${px}px">${s.replace(/width="48" height="48"/, `width="${px}" height="${px}"`)}</span>`).join('')}</div>`).join('')}
          <div style="display:flex;gap:12px">${roundels.map((n) => `<img src="/src/assets/units/${n}-256.png" width="96">`).join('')}${roundels.map((n) => `<img src="/src/assets/units/${n}-card.png" width="64">`).join('')}</div>
          <div style="margin-top:6px">icons: Keshig, Horseman, Ranger, Cataphract (16, 24, 48 px, then the same four roundels) · miniatures and build-card crops below</div></div>`;
        await Promise.all([...document.images].map((i) => i.decode().catch(() => undefined)));
      });
      await shot(page, size, '01-art');
    });

    test('copy: the Mongol faction sheet and the build card', async ({ page }) => {
      test.setTimeout(90_000);
      await boot(page, size);
      await page.getByRole('button', { name: /^Single Player/ }).click();
      await page.getByRole('button', { name: /^Seat 1 faction:/ }).click();
      const stage = page.locator('dialog[data-testid="faction-stage"][open]');
      await expect(stage).toBeVisible();
      await page.getByRole('option', { name: 'Mongols' }).click();
      const sheet = stage.getByTestId('faction-sheet');
      await expect(sheet.locator('.fs-sheet-name')).toContainText('Mongols');
      await expect(sheet).not.toContainText('need no strategic resource');
      const card = sheet.locator('.pc').first();
      await expect(card).toContainText('Keshig');
      await expect(card).toContainText('replaces Cataphract');
      await expect(card).toContainText('Shoots or charges each turn, never both');
      await expect(sheet).not.toContainText(/Outrider|Market-family/);
      await page.mouse.move(size.width - 2, size.height - 2);
      await page.waitForTimeout(300);
      await shot(page, size, '02-faction-sheet');
      // The sheet fits the window without scrolling.
      expect(await sheet.evaluate((el) => el.scrollHeight - el.clientHeight)).toBeLessThanOrEqual(0);
      await page.keyboard.press('Escape');

      await page.evaluate(async () => {
        const file = '/src/test/pieceFixture.ts';
        const { pieceFixture } = await import(/* @vite-ignore */ file);
        await pieceFixture('Mongols');
      });
      const build = page.locator('.city-panel .production-choice[data-item="u:Cataphract"]');
      await build.scrollIntoViewIfNeeded();
      await expect(build).toContainText('Keshig');
      await expect(build.getByTestId('piece-vs')).toContainText('vs Cataphract: +17 ranged');
      expect(await build.locator('.prod').evaluate((el) => el.scrollWidth - el.clientWidth)).toBeLessThanOrEqual(0);
      await page.mouse.move(size.width - 2, size.height - 2);
      await page.waitForTimeout(250);
      await shot(page, size, '03-build-card');
    });

    test('research: Iron Working unlocks the Keshig and upgrades Horsemen into it', async ({ page }) => {
      test.setTimeout(90_000);
      await boot(page, size);
      await page.evaluate(async () => {
        const file = '/src/test/pieceFixture.ts';
        const { pieceFixture, setTechOpen } = await import(/* @vite-ignore */ file);
        await pieceFixture('Mongols', ['Irrigation', 'Currency']);
        setTechOpen(true);
      });
      const dialog = page.locator('dialog.tech-modal');
      await expect(dialog).toBeVisible();
      const iron = dialog.locator('.era-card[data-tech="IronWorking"]');
      await iron.scrollIntoViewIfNeeded();
      await expect(iron).toContainText('Keshig');
      await expect(iron.locator('.era-unlock.unit')).toContainText('From Horseman');
      await expect(iron).not.toContainText('Horsemen can upgrade to Keshigs');
      await expect(iron).not.toContainText('Cataphract');
      await iron.locator('.era-unlock.unit', { hasText: 'From Horseman' }).hover();
      const hover = page.getByTestId('unlock-card');
      await expect(hover).toBeVisible();
      await expect(hover).toContainText('Keshig');
      await expect(hover.getByTestId('uc-upgrade')).toContainText('Horseman');
      await page.waitForTimeout(300);
      await shot(page, size, '04-iron-working');
    });

    test('forecast: shoot or charge side by side, and why there is no shot when the target is too far', async ({ page }) => {
      test.setTimeout(120_000);
      await mkdir(out, { recursive: true });
      const fx = keshigFixture();
      await page.addInitScript(({ game }) => {
        localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn: 60, names: ['Temujin', 'Pericles'], savedAt: new Date().toISOString(), game }));
        localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
      }, { game: coreFixture(fx.fixture) });
      await page.setViewportSize(size);
      await page.goto('/?client=wasm');
      await page.getByRole('button', { name: /^Continue/ }).click();
      await expect(page.getByTestId('turn-number')).toHaveText('60');
      await expect(page.locator('.loading-screen')).toHaveCount(0);
      await page.evaluate(() => document.fonts.ready.then(() => undefined));
      await page.evaluate((id) => (window as unknown as { legatum: { getState(): { select(s: unknown): void } } }).legatum.getState().select({ kind: 'unit', id }), fx.keshig);
      const hoverOn = async (h: Hex, wanted: string) => {
        await page.evaluate((c) => { const w = window as unknown as MapWindow; w.legatumMap.camera.zoom = 1.0; (window as unknown as { legatum: { getState(): { jumpTo(h: Hex): void } } }).legatum.getState().jumpTo(c); w.legatumMap.redraw(); }, fx.centre);
        const b = await page.locator('[data-game-map]').boundingBox();
        // The camera glides to the centre: wait for it, then hover the target where it is now.
        await expect.poll(() => page.evaluate((c) => {
          const m = (window as unknown as MapWindow).legatumMap;
          const r = document.querySelector('[data-game-map]')!.getBoundingClientRect();
          const q = m.toScreen(c);
          return Math.hypot(q.x - r.width / 2, q.y - r.height / 2) < 6;
        }, fx.centre), { timeout: 8000 }).toBe(true).catch(() => undefined);
        await expect.poll(async () => {
          const p = await page.evaluate((hex) => (window as unknown as MapWindow).legatumMap.toScreen(hex), h);
          await page.mouse.move(b!.x + p.x + 3, b!.y + p.y + 3);
          await page.mouse.move(b!.x + p.x, b!.y + p.y, { steps: 3 });
          return page.locator(wanted).isVisible();
        }, { intervals: [400, 800, 1000, 2000] }).toBe(true);
        await page.waitForTimeout(300);
      };
      // Next to the Keshig: both options.
      await hoverOn(fx.spearmanAt, '[data-testid="forecast-dual"]');
      const dual = page.getByTestId('forecast-dual');
      await expect(page.getByTestId('forecast-shot')).toContainText('R, then click');
      await expect(page.getByTestId('forecast-charge')).toContainText('Right-click');
      await expect(page.getByTestId('forecast-shot-class')).toHaveText('No class bonus for either side.');
      await expect(page.getByTestId('forecast-charge-class')).toContainText(`${MELEE_VS_MOUNTED_LABEL}: +`);
      const box = (await dual.boundingBox())!;
      expect(box.y, 'inside the window, top').toBeGreaterThanOrEqual(0);
      expect(box.y + box.height, 'inside the window, bottom').toBeLessThanOrEqual(size.height + 0.5);
      expect(await dual.evaluate((el) => el.scrollHeight - el.clientHeight), 'the card is not cut off').toBeLessThanOrEqual(1);
      await shot(page, size, '05-forecast-shoot-or-charge');
      // Three hexes off: a charge only, and the card says why.
      await hoverOn(fx.archerAt, '[data-testid="forecast-shot-why"]');
      await expect(page.getByTestId('forecast-shot-why')).toContainText('No shot from here');
      await expect(page.getByTestId('forecast-shot-why')).toContainText('Right-click to charge it');
      await shot(page, size, '06-forecast-out-of-range');
      // The unit panel keeps the ranged order and its R shortcut beside movement.
      await page.mouse.move(size.width - 2, size.height - 2);
      await page.waitForTimeout(300);
      await expect(page.locator('.unit-panel').getByRole('button', { name: /Ranged Attack/ })).toBeVisible();
      await expect(page.locator('.unit-panel').getByRole('button', { name: /Move or attack/ })).toBeVisible();
      await shot(page, size, '07-unit-panel');
    });
  });
}
