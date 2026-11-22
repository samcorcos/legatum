import { test as base, expect } from '@playwright/test';
import { createServer } from 'vite';
import { coreFixture } from '../src/test/coreFixture';
import { hallFixture } from '../src/test/hallFixture';
import { overlayBox } from './overlayHelpers';

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

const sizes = [
  { name: 'desktop', width: 1440, height: 900, touch: false },
  { name: 'iPad', width: 1024, height: 768, touch: true },
];
for (const size of sizes) for (const look of ['atlas'] as const) test.describe(`${look} ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height }, hasTouch: size.touch, isMobile: size.touch });
  test('lenses stay reachable with the turn-one production hint', async ({ page }, testInfo) => {
    test.setTimeout(120_000);
    const errors: string[] = [];
    page.on('pageerror', (e) => errors.push(e.message));
    page.on('console', (m) => { if (m.type() === 'error') errors.push(m.text()); });
    await page.goto(`/?client=${process.env.LEGATUM_LENS_CLIENT ?? 'wasm'}`);
    await page.getByRole('button', { name: /^Single Player/ }).click();
    await page.getByRole('button', { name: /^Start game/ }).click();
    await expect(page.getByTestId('turn-number')).toHaveText('1');
    await expect(page.getByTestId('first-game-hint')).toContainText('Pick production');
    // Select the starting Scout to exercise reachable unit actions alongside the capital hint.
    await page.evaluate(async () => {
      const storeSource = '/src/store.ts';
      const { useGame } = await import(storeSource) as typeof import('../src/store');
      const s = useGame.getState(), v = s.views![s.seat];
      const u = v.units.find((u) => u.owner === v.me && u.kind === 'Scout')!;
      s.select({ kind: 'unit', id: u.id }); s.jumpTo(u.pos);
    });
    const phone = size.width < 700;
    const group = page.getByRole('radiogroup', { name: 'Map lens', exact: true });
    const toggle = page.getByRole('button', { name: /^Lens: / });
    const openLenses = async () => {
      if (await toggle.isVisible() && await toggle.getAttribute('aria-expanded') !== 'true') await toggle.click();
      await expect(group).toBeVisible();
    };
    // Poll measured layout (ResizeObserver and arrival animations settle asynchronously).
    const assertDockClear = async () => {
      await expect.poll(() => page.locator('.map-lenses').evaluate((panel) => {
        const a = panel.getBoundingClientRect();
        const visible = (el: Element) => el.getBoundingClientRect().height > 0 && getComputedStyle(el).visibility !== 'hidden';
        const intersects = (a: DOMRect, b: DOMRect) => a.left < b.right && a.right > b.left && a.top < b.bottom && a.bottom > b.top;
        const issues = [...document.querySelectorAll('.first-game-hint, .topbar, .unit-panel, .vp, .seatbar, .phone-selection-slot, .phone-turn-dock, .phone-vp-strip')]
          .filter((el) => visible(el) && intersects(a, el.getBoundingClientRect())).map((el) => el.className);
        if (a.left < 0 || a.right > innerWidth || a.top < 0 || a.bottom > innerHeight) issues.push('offscreen');
        const unit = document.querySelector('.phone-selection-slot, .unit-panel')!.getBoundingClientRect();
        const turn = document.querySelector('.phone-turn-dock, .seatbar')!.getBoundingClientRect();
        if (intersects(unit, turn)) issues.push('unit actions covered by This Turn');
        for (const el of document.querySelectorAll('.settle-persistent-legend, .map-touch-card, .settlement-card')) {
          if (!visible(el)) continue;
          const b = el.getBoundingClientRect();
          if (b.left < 0 || b.right > innerWidth || b.top < 0 || b.bottom > innerHeight) issues.push(`${el.className} offscreen`);
          for (const other of document.querySelectorAll('.first-game-hint, .topbar, .unit-panel, .vp, .seatbar, .phone-selection-slot, .phone-turn-dock, .phone-vp-strip, .map-lenses, .lens-content')) {
            if (!visible(other) || other.contains(el)) continue;
            if (intersects(b, other.getBoundingClientRect())) issues.push(`${el.className} overlaps ${other.className}`);
          }
        }
        if (innerWidth < 700) {
          if (Math.abs(a.height - 44) > 1) issues.push('lens dock must be 44px');
          const map = document.querySelector('.map-canvas')!, canvas = map.getBoundingClientRect();
          if (document.elementFromPoint(canvas.left + canvas.width / 2, canvas.top + canvas.height / 2) !== map) issues.push('map center covered');
          if (unit.top < canvas.bottom) issues.push('unit panel extends into the canvas');
          for (const el of document.querySelectorAll('.lens-content, .map-touch-card, .settle-persistent-legend')) {
            if (!visible(el)) continue;
            const b = el.getBoundingClientRect();
            if (b.bottom > canvas.top) issues.push('inspector extends into canvas');
          }
        }
        return issues;
      })).toEqual([]);
    };
    await openLenses();
    await group.getByRole('radio', { name: 'Settle', exact: true }).click();
    if (phone) await expect(toggle).toHaveText('Lens: Settle');
    const assertSettleLegend = async () => {
      const wasOpen = await group.isVisible();
      await openLenses();
      await expect(page.locator('.settle-legend')).toBeVisible();
      await expect(page.locator('.settle-legend p').first()).toHaveText(`Better places to found a city are darker green. ${size.touch ? 'Tap' : 'Hover'} a tile for why. No tint: not a site.`);
      await expect(page.locator('.settle-scale [data-settle-band]')).toHaveText([/^Poor/, /^Fair/, /^Good/, /^Great/, /^Exceptional/]);
      await expect(page.locator('.map-lenses')).not.toContainText(/scores in points|0–59|60–119|120\+/);
      if (!wasOpen) await toggle.click();
    };
    await assertSettleLegend(); // On a phone this is visible before opening the chooser.
    await openLenses();
    await expect(group.getByRole('radio', { name: 'Settle', exact: true })).toBeChecked();
    // A random opening may have zero or three known safe sites. Both states
    // retain a useful explanation; the explored-map fixture below requires ranks.
    if (await page.locator('[data-settle-rank]').count()) {
      await expect(page.locator('[data-settle-rank]')).toHaveText(['Best spot', '#2', '#3']);
    } else {
      await expect(page.getByText('No safe reachable recommendations. Explore more land or clear nearby threats.')).toBeVisible();
    }
    if (await toggle.isVisible()) await page.getByRole('button', { name: 'Close map options' }).click();
    // Every action, including Disband, is either in view or in a labelled scroller with
    // a persistent hint. Check real hit targets without issuing an irreversible order.
    if (phone) {
      await page.locator('.phone-sheet-handle').click();
      await page.locator('.phone-sheet-handle').click();
      await expect(page.locator('.phone-sheet')).toHaveAttribute('data-detent', 'full');
    }
    const actions = page.locator('.unit-panel .actions button.act');
    const scroll = page.getByRole('region', { name: 'Unit details and actions' });
    if (await scroll.evaluate((el) => el.scrollHeight > el.clientHeight + 1 && el.clientHeight > 0)) {
      await expect(page.locator('.unit-scroll-hint')).toBeVisible();
    }
    for (const action of await actions.all()) {
      await action.scrollIntoViewIfNeeded();
      await expect.poll(() => action.evaluate((el) => {
        const b = el.getBoundingClientRect();
        return el.contains(document.elementFromPoint(b.x + b.width / 2, b.y + b.height / 2));
      })).toBe(true);
    }
    if (phone) {
      await page.locator('.phone-sheet-handle').click();
      await expect(page.locator('.phone-sheet')).toHaveAttribute('data-detent', 'peek');
    }
    // Every overlay on: each lens is then checked with Yields, Resources and Borders stacked over it.
    await openLenses();
    for (const overlay of ['Yields', 'Resources', 'Borders']) {
      const box = overlayBox(page, overlay);
      if (size.touch) await box.tap(); else await box.click();
      await expect(box).toBeChecked();
    }
    await expect(page.locator('.lens-group-label').first()).toContainText('1 icon per point');
    for (const lens of ['Settle', 'Movement', 'Threat', 'Discovery', 'Default']) {
      await openLenses();
      await assertDockClear();
      const radio = group.getByRole('radio', { name: lens, exact: true });
      if (size.touch) await radio.tap(); else await radio.click();
      if (await toggle.isVisible()) {
        await expect(toggle).toHaveText(new RegExp(`^Lens: ${lens} · Yields, Resources, Borders$`));
        await expect(toggle).toHaveAttribute('aria-expanded', 'false');
        await expect(group).toBeHidden();
      } else await expect(radio).toBeChecked();
      if (lens === 'Settle') await assertSettleLegend();
      await expect(page.getByTestId('first-game-hint')).toBeVisible();
      await page.mouse.move(1, 1);
      await assertDockClear();
      await page.screenshot({ path: testInfo.outputPath(`${look}-${size.width}-${lens.toLowerCase()}.png`) });
    }
    await openLenses();
    const defaultRadio = group.getByRole('radio', { name: 'Default', exact: true });
    await defaultRadio.focus();
    await page.keyboard.press('ArrowRight');
    await expect(group.getByRole('radio', { name: 'Settle', exact: true })).toBeChecked();
    await expect(group.getByRole('radio', { name: 'Settle', exact: true })).toBeFocused();
    await expect(overlayBox(page, 'Yields')).toBeChecked(); // the overlays stayed on through every lens
    if (phone) { await page.keyboard.press('Escape'); await expect(toggle).toBeFocused(); }
    await expect(page.getByTestId('turn-number')).toHaveText('1');
    for (const zoom of [0.28, 2.4, 0.9]) {
      await page.evaluate((z) => { const m = (window as unknown as { legatumMap: { camera: { zoom: number }; redraw(): void } }).legatumMap; m.camera.zoom = z; m.redraw(); }, zoom);
      await page.waitForTimeout(80);
    }
    await openLenses();
    await group.getByRole('radio', { name: 'Settle', exact: true }).click();
    await openLenses();
    await assertDockClear();
    expect(errors).toEqual([]);
  });
});

// Playtest 8.4: "when i press the settle button it doesn't appear to do anything". With no Settler on the
// map the lens drew nothing; it now washes the explored land and ranks three sites from the capital.
for (const size of [{ width: 1000, height: 600 }, { width: 1440, height: 900 }]) test.describe(`settle lens without a Settler ${size.width}x${size.height}`, () => {
  test.use({ viewport: size });
  test('washes the explored map and marks the best sites', async ({ page }, testInfo) => {
    const errors: string[] = [];
    page.on('pageerror', (e) => errors.push(e.message));
    // Mid-game fixture: the whole island explored, three cities of yours, no Settler anywhere.
    const game = coreFixture(hallFixture('buildable').fixture);
    await page.addInitScript((game) => {
      if (!location.protocol.startsWith('http')) return;
      localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn: 12, names: ['Ada'], savedAt: new Date().toISOString(), game }));
      localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
    }, game);
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^Continue/ }).click();
    await expect(page.getByTestId('turn-number')).toHaveText('12');
    await expect(page.locator('.loading-screen')).toHaveCount(0);
    const hasSettler = await page.evaluate(async () => {
      const storeSource = '/src/store.ts';
      const { useGame } = await import(storeSource) as typeof import('../src/store');
      const s = useGame.getState(), v = s.views![s.seat];
      s.select(null);
      return v.units.some((u) => u.owner === v.me && u.kind === 'Settler');
    });
    expect(hasSettler).toBe(false);
    // Dismiss any celebration and the hint so the map is bare.
    for (let i = 0; i < 4 && await page.locator('.moment-card').first().isVisible(); i++) await page.keyboard.press('Escape');
    const group = page.getByRole('radiogroup', { name: 'Map lens', exact: true });
    const grab = () => {
      const c = document.querySelector<HTMLCanvasElement>('canvas[data-game-map]')!;
      return c.getContext('2d')!.getImageData(0, 0, c.width, c.height).data;
    };
    await page.mouse.move(500, 20);
    await page.evaluate((grabSrc) => { (window as unknown as { __before: Uint8ClampedArray }).__before = new Function(`return (${grabSrc})()`)(); }, grab.toString());
    await group.getByRole('radio', { name: 'Settle', exact: true }).click();
    await expect(page.locator('[data-settle-rank]')).toHaveText(['Best spot', '#2', '#3']);
    await page.mouse.move(500, 20);
    await page.waitForTimeout(500);
    // Candidates render: the wash changes a large share of the map's pixels, and visibly so.
    const changed = await page.evaluate((grabSrc) => {
      const before = (window as unknown as { __before: Uint8ClampedArray }).__before, after = new Function(`return (${grabSrc})()`)() as Uint8ClampedArray;
      let n = 0;
      for (let i = 0; i < before.length; i += 4) if (Math.abs(before[i] - after[i]) + Math.abs(before[i + 1] - after[i + 1]) + Math.abs(before[i + 2] - after[i + 2]) > 30) n++;
      return n / (before.length / 4);
    }, grab.toString());
    expect(changed).toBeGreaterThan(0.1);
    // Hovering still explains why, and the legend says where travel is counted from.
    await page.locator('[data-settle-rank="1"]').click();
    await expect(page.getByLabel('Settlement suitability').locator('[data-settle-band]')).toHaveText(/^(Poor|Fair|Good|Great|Exceptional) spot for a city: /);
    await expect(page.getByLabel('Settlement suitability').locator('[data-settle-route]')).toContainText('Best spot');
    await expect(page.locator('.settle-legend p').first()).toContainText('Better places to found a city are darker green');
    await expect(page.getByText('Travel is counted from your capital')).toBeVisible();
    await page.screenshot({ path: testInfo.outputPath(`settle-no-settler-${size.width}x${size.height}.png`) });
    expect(errors).toEqual([]);
  });
});
