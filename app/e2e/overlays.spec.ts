// Map overlays (Yields, Resources, Borders) stack over any lens. Shots and checks at 1000x600 and 1440x900:
//   Yields + Settle, Yields + Threat, and all three overlays + Default, plus the panel itself.
// Legibility is measured on the canvas: the yield icons of tiles that show them over the plain map must still be there
// (same ink pixels) under every lens, so a lens tint, a terrain fade or a lens chip never hides them.
// OVERLAY_SHOTS=<dir> picks where the shots go (default: the test's output folder).
import { test as base, expect, type Page } from '@playwright/test';
import { createServer } from 'vite';
import { mkdir } from 'node:fs/promises';
import path from 'node:path';
import type { useGame } from '../src/store';
import { coreFixture } from '../src/test/coreFixture';
import { mapArtFixture } from '../src/test/mapArtFixture';
import { lensRadio, overlayBox, setOverlay, showMapOptions } from './overlayHelpers';

type Hex = { q: number; r: number };
type MapWindow = Window & { legatum: typeof useGame; legatumMap: { camera: { zoom: number }; redraw(): void; toScreen(h: Hex): { x: number; y: number } } };

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

const setup = mapArtFixture();
const SIZES = [{ width: 1000, height: 600 }, { width: 1440, height: 900 }] as const;

async function open(page: Page) {
  await page.addInitScript(({ game }) => {
    if (!location.protocol.startsWith('http')) return;
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn: 12, names: ['Ada'], savedAt: new Date().toISOString(), game }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, { game: coreFixture(setup.fixture) });
  await page.goto('/?client=wasm');
  const resume = page.getByRole('button', { name: /^Continue/ });
  // Retry one blank initial Vite navigation; a persistent load failure still fails the test.
  try { await expect(resume).toBeVisible({ timeout: 15_000 }); }
  catch { await page.reload(); await expect(resume).toBeVisible({ timeout: 30_000 }); }
  await resume.click();
  await expect(page.getByTestId('turn-number')).toHaveText('12');
  await expect(page.locator('.loading-screen')).toHaveCount(0);
  for (let i = 0; i < 4 && await page.locator('.moment-card').first().isVisible(); i++) await page.keyboard.press('Escape');
  await page.evaluate(() => document.fonts.ready.then(() => undefined));
  await showMapOptions(page);
}

/** Centre the camera on a hex at a zoom, with the given unit selected (or nothing). */
async function frame(page: Page, at: Hex, zoom: number, select?: { kind: 'unit' | 'city'; id: number }) {
  await page.evaluate(({ at, zoom, select }) => {
    const w = window as unknown as MapWindow;
    const s = w.legatum.getState();
    s.set({ techOpen: false });
    s.select(select ?? null);
    s.jumpTo(at);
    w.legatumMap.camera.zoom = zoom;
    w.legatumMap.redraw();
  }, { at, zoom, select });
  await expect.poll(() => page.evaluate((at) => {
    const m = (window as unknown as MapWindow).legatumMap;
    const b = document.querySelector('[data-game-map]')!.getBoundingClientRect();
    const p = m.toScreen(at);
    return Math.hypot(p.x - b.width / 2, p.y - b.height / 2) < 6;
  }, at), { timeout: 5000 }).toBe(true).catch(() => undefined);
  await park(page);
  await page.waitForTimeout(400);
  await showMapOptions(page);
}

/** Copy the map canvas's pixels aside, for `yieldPixels` to compare against. */
const captureMap = (page: Page) => page.evaluate(() => {
  const canvas = document.querySelector('canvas[data-game-map]') as HTMLCanvasElement;
  (window as unknown as { __mapOff: ImageData }).__mapOff = canvas.getContext('2d')!.getImageData(0, 0, canvas.width, canvas.height);
});

/**
 * For up to `limit` tiles in view that have yields (no token, city, resource or wonder on them), the number of pixels in the icon cluster's strip that
 * are exactly an icon's ink as drawn over what the captured map (the Yields overlay off, same lens) shows there: the ink itself at full strength, or 0.7 of it on
 * a remembered tile. That finds the glyphs whatever the lens tint or terrain behind them, so a count that falls under a lens means something now covers the icons.
 */
async function yieldPixels(page: Page, sample: number[] | null, limit = 60) {
  return page.evaluate(async ({ sample, limit }) => {
    const w = window as unknown as MapWindow & { __mapOff: ImageData };
    const storePath = '/src/store.ts', resPath = '/src/resources.ts';
    const { useGame } = await import(/* @vite-ignore */ storePath) as typeof import('../src/store');
    const { tileYields } = await import(/* @vite-ignore */ resPath) as typeof import('../src/resources');
    const s = useGame.getState(), view = s.views![s.seat];
    const faction = view.players.find((p) => p.id === view.me)!.faction;
    const canvas = document.querySelector('canvas[data-game-map]') as HTMLCanvasElement;
    const on = canvas.getContext('2d')!.getImageData(0, 0, canvas.width, canvas.height), off = w.__mapOff;
    const k = canvas.width / canvas.clientWidth, r = 36 * w.legatumMap.camera.zoom;
    const occupied = new Set([...view.units.map((u) => `${u.pos.q},${u.pos.r}`), ...view.cities.map((c) => `${c.pos.q},${c.pos.r}`)]);
    const cw = canvas.clientWidth, ch = canvas.clientHeight;
    const INK = [[78, 118, 46], [150, 80, 32], [128, 98, 13], [35, 107, 146]], PAPER = [247, 241, 227];
    const cb = canvas.getBoundingClientRect();
    const out: { index: number; diff: number }[] = [];
    for (const i of sample ?? view.tiles.map((_, n) => n)) {
      const t = view.tiles[i];
      if (!t || !t.explored || !t.terrain) continue;
      const row = Math.floor(i / view.width);
      const hex = { q: i % view.width - Math.floor((row - (row & 1)) / 2), r: row };
      if (occupied.has(`${hex.q},${hex.r}`) || t.wonder || t.resource) continue;
      const y = tileYields(faction, t);
      if (!(y.food > 0 || y.production > 0 || y.gold > 0 || y.science > 0)) continue;
      const p = w.legatumMap.toScreen(hex);
      if (p.x < r || p.x > cw - r || p.y < r || p.y > ch - r) continue;
      // Not hidden behind a panel: the canvas still holds those pixels, but nobody can see them.
      if ([-0.5, 0, 0.5].some((dx) => document.elementFromPoint(cb.left + p.x + dx * r, cb.top + p.y + r * 0.45) !== canvas)) continue;
      const hw = r * 0.6, x0 = Math.round((p.x - hw) * k), y0 = Math.round((p.y + r * 0.2) * k), wpx = Math.round(2 * hw * k), hpx = Math.round(r * 0.5 * k);
      const a = t.visible ? 1 : 0.7;
      let n = 0;
      for (let yy = y0; yy < y0 + hpx; yy++) for (let xx = x0; xx < x0 + wpx; xx++) {
        const o = (yy * canvas.width + xx) * 4;
        // Only pixels the icons changed (so brown terrain that happens to match a gear never counts) and that are an icon colour: the ink itself, or the
        // glyphs' parchment rim, which keeps an icon readable when its ink is close to a deep tint.
        const changed = Math.abs(on.data[o] - off.data[o]) + Math.abs(on.data[o + 1] - off.data[o + 1]) + Math.abs(on.data[o + 2] - off.data[o + 2]) > 24;
        const hasInk = changed && INK.some((c) => [0, 1, 2].every((ch) => Math.abs(on.data[o + ch] - (a * c[ch] + (1 - a) * off.data[o + ch])) <= 34));
        const k = 0.92 * a;
        const hasRim = changed && [0, 1, 2].every((ch) => Math.abs(on.data[o + ch] - (k * PAPER[ch] + (1 - k) * off.data[o + ch])) <= 22);
        if (hasInk || hasRim) n++;
      }
      out.push({ index: i, diff: n });
      if (!sample && out.length >= limit) break;
    }
    return out;
  }, { sample, limit });
}

/** A spot over plain panel text, so no control's tooltip or the map's tile card leaks into a shot. */
const park = (page: Page) => page.mouse.move(page.viewportSize()!.width < 1200 ? 820 : 1263, page.viewportSize()!.width < 1200 ? 355 : 345);

const outDir = (testInfo: { outputPath(n: string): string }) => process.env.OVERLAY_SHOTS ? path.resolve(process.env.OVERLAY_SHOTS) : path.dirname(testInfo.outputPath('x'));
async function shot(page: Page, testInfo: { outputPath(n: string): string }, name: string) {
  const dir = outDir(testInfo);
  await mkdir(dir, { recursive: true });
  await park(page);
  await page.screenshot({ path: path.join(dir, `${name}.png`), animations: 'disabled' });
}

/** The selected unit's primary button is whole and in view (not under the scroll hint, not under the map panel). */
async function expectPrimaryWhole(page: Page, name: RegExp, why: string) {
  await expect.poll(() => page.evaluate((src) => {
    const re = new RegExp(src);
    const b = [...document.querySelectorAll<HTMLButtonElement>('.unit-panel button')].find((x) => re.test(x.textContent ?? ''));
    if (!b) return 'no such button';
    const r = b.getBoundingClientRect(), hit = (x: number, y: number) => b.contains(document.elementFromPoint(x, y));
    const d = document.querySelector('.map-lenses')!.getBoundingClientRect();
    if (!(hit(r.left + r.width / 2, r.top + 2) && hit(r.left + r.width / 2, r.bottom - 2))) return 'cut off';
    if (r.left < d.right && r.right > d.left && r.top < d.bottom && r.bottom > d.top) return 'under the map panel';
    return 'whole';
  }, name.source), { message: why, timeout: 5000 }).toBe('whole');
}

/** The panel stays whole: inside the window, over no other panel. */
async function expectPanelClear(page: Page) {
  const issues = await page.locator('.map-lenses').evaluate((panel) => {
    const a = panel.getBoundingClientRect();
    const intersects = (b: DOMRect) => a.left < b.right && a.right > b.left && a.top < b.bottom && a.bottom > b.top;
    const out: string[] = [];
    for (const el of document.querySelectorAll('.topbar, .unit-panel, .vp, .seatbar, .first-game-hint')) {
      const b = el.getBoundingClientRect();
      if (b.height > 0 && getComputedStyle(el).visibility !== 'hidden' && intersects(b)) out.push(String(el.className));
    }
    if (a.left < 0 || a.right > innerWidth || a.top < 0 || a.bottom > innerHeight) out.push('offscreen');
    if (panel.scrollWidth > panel.clientWidth + 1) out.push('sideways overflow');
    return out;
  });
  expect(issues).toEqual([]);
}

for (const size of SIZES) {
  test.describe(`${size.width}x${size.height}`, () => {
    test.use({ viewport: size });

    test('the panel: two labelled groups, toggles that look like toggles, remembered per viewer', async ({ page }, testInfo) => {
      test.setTimeout(240_000);
      const errors: string[] = [];
      page.on('pageerror', (e) => { errors.push(e.message); console.log('PAGEERROR', e.message, e.stack); });
      page.on('console', (m) => { if (['error','warning'].includes(m.type())) console.log('CONSOLE', m.text()); });
      await open(page);
      const overlays = page.getByRole('group', { name: /^Overlays/ });
      await expect(overlays).toBeVisible();
      await expect(overlays.getByRole('checkbox')).toHaveCount(3);
      await expect(page.getByRole('radiogroup', { name: 'Map lens', exact: true }).getByRole('radio')).toHaveText(['Default', 'Settle', 'Movement', 'Threat', 'Discovery']);
      // First run: everything off, so the map starts as it always has.
      for (const name of ['Yields', 'Resources', 'Borders']) await expect(overlayBox(page, name)).not.toBeChecked();
      await expect(page.locator('.overlay-legend')).toHaveCount(0);
      await expectPanelClear(page);
      const baseHeight = (await page.locator('.map-lenses').boundingBox())!.height;
      await shot(page, testInfo, `${size.width}-panel-off`);

      // Click turns an overlay on, with its legend, and leaves the lens alone.
      await overlayBox(page, 'Yields').click();
      await expect(overlayBox(page, 'Yields')).toBeChecked();
      await expect(page.locator('.lens-group-label').first()).toContainText('1 icon per point · a number from 5');
      await expect(lensRadio(page, 'Default')).toBeChecked();
      await expectPanelClear(page);
      const oneOn = (await page.locator('.map-lenses').boundingBox())!.height;
      await shot(page, testInfo, `${size.width}-panel-yields`);

      // Keys: Y toggles Yields, O Resources, N Borders; R still belongs to the ranged attack.
      await page.keyboard.press('y');
      await expect(overlayBox(page, 'Yields')).not.toBeChecked();
      await page.keyboard.press('o');
      await expect(overlayBox(page, 'Resources')).toBeChecked();
      await page.keyboard.press('n');
      await expect(overlayBox(page, 'Borders')).toBeChecked();
      await page.keyboard.press('y');
      await expect(overlayBox(page, 'Yields')).toBeChecked();
      await expect(page.locator('.overlay-toggle kbd')).toHaveText(['Y', 'O', 'N']);
      await expectPanelClear(page);
      const allOn = (await page.locator('.map-lenses').boundingBox())!.height;
      await shot(page, testInfo, `${size.width}-panel-all`);
      if (size.height <= 700) {
        // Short window: the keys for Resources and Borders open on request.
        await expect(page.locator('.overlay-legend')).toHaveCount(0);
        await page.getByRole('button', { name: 'Overlay key' }).click();
        await expect(page.locator('.overlay-legend .lens-resource-key')).toBeVisible();
        await expect(page.locator('.lens-owner-key span')).toHaveCount(8);
        await expect(page.locator('.lens-border-key')).toBeInViewport();
        await expectPanelClear(page);
        await shot(page, testInfo, `${size.width}-panel-all-key`);
        await page.getByRole('button', { name: 'Overlay key' }).click();
        await expect(page.locator('.overlay-legend')).toHaveCount(0);
      } else {
        await expect(page.locator('.overlay-legend .lens-resource-key')).toBeVisible();
        await expect(page.locator('.lens-owner-key span')).toHaveCount(8);
      }
      testInfo.annotations.push({ type: 'panel height', description: `off ${baseHeight} · yields ${oneOn} · all three ${allOn} (px)` });

      // A lens is one at a time, and picking one leaves the overlays on.
      await lensRadio(page, 'Threat').click();
      await showMapOptions(page);
      await expect(lensRadio(page, 'Threat')).toBeChecked();
      await expect(lensRadio(page, 'Default')).not.toBeChecked();
      for (const name of ['Yields', 'Resources', 'Borders']) await expect(overlayBox(page, name)).toBeChecked();
      await expectPanelClear(page);

      // Remembered: a reload keeps the three overlays on; the lens starts again at Default.
      await page.reload();
      await page.getByRole('button', { name: /^Continue/ }).click();
      await expect(page.getByTestId('turn-number')).toHaveText('12');
      await showMapOptions(page);
      for (const name of ['Yields', 'Resources', 'Borders']) await expect(overlayBox(page, name)).toBeChecked();
      expect(await page.evaluate(() => JSON.parse(localStorage.getItem('legatum.map-overlays.v1')!))).toEqual({ yields: true, resources: true, borders: true });
      await setOverlay(page, 'Resources', false);
      await setOverlay(page, 'Borders', false);
      await setOverlay(page, 'Yields', false);
      expect(errors).toEqual([]);
    });

    test('Yields over Settle, Yields over Threat, and all three overlays over Default stay legible', async ({ page }, testInfo) => {
      test.setTimeout(300_000);
      const errors: string[] = [];
      page.on('pageerror', (e) => errors.push(e.message));
      await open(page);
      const home = await page.evaluate((id) => (window as unknown as MapWindow).legatum.getState().views![0].cities.find((c) => c.id === id)!.pos, setup.capital);
      expect(setup.settler && setup.settlerAt).toBeTruthy();
      const settlerAt = setup.settlerAt!, settler = setup.settler!;
      const problems: string[] = [];
      // The icons' pixels with the overlay on versus off, on the same frame: `measure` reads them for the given tiles (or picks tiles).
      const measure = async (at: Hex, zoom: number, select: { kind: 'unit' | 'city'; id: number } | undefined, sample: number[] | null) => {
        await frame(page, at, zoom, select);
        await setOverlay(page, 'Yields', false);
        await park(page);
        await page.waitForTimeout(450);
        await captureMap(page);
        await setOverlay(page, 'Yields', true);
        await park(page);
        await page.waitForTimeout(450);
        return yieldPixels(page, sample);
      };
      // The same frame (camera and selection) for the baseline and the lens, so each tile sits where it did.
      const baseline = async (label: string, at: Hex, select?: { kind: 'unit' | 'city'; id: number }) => {
        await frame(page, at, 0.9, select);
        await lensRadio(page, 'Default').click();
        const plain = (await measure(at, 0.9, select, null)).filter((t) => t.diff >= 30);
        if (plain.length < 6) problems.push(`${label}: only ${plain.length} tiles with icons to compare`);
        return { eligible: plain };
      };
      const under = async (label: string, lens: string, at: Hex, base: { index: number; diff: number }[], select?: { kind: 'unit' | 'city'; id: number }, zoom = 0.9) => {
        if (lens !== 'Default') await lensRadio(page, lens).click();
        const now = await measure(at, zoom, select, base.map((b) => b.index));
        const byIndex = new Map(now.map((t) => [t.index, t.diff]));
        // A lens that covered the icons would take their pixels away: no tile may lose two thirds of its icon pixels, and on average they keep nine in ten
        // (sub-pixel shifts of the camera and a neighbour's medallion ring move a few).
        let worst = 1, sum = 0, count = 0;
        for (const b of base) {
          const n = byIndex.get(b.index);
          if (n === undefined) continue; // the tile is now under a token (the selected Settler)
          const ratio = Math.min(1.2, n / b.diff);
          worst = Math.min(worst, ratio); sum += ratio; count++;
          if (ratio < 0.35) problems.push(`${label}: tile ${b.index} icon pixels ${n} of ${b.diff}`);
        }
        if (count && sum / count < 0.85) problems.push(`${label}: icons keep only ${(sum / count).toFixed(2)} of their pixels on average`);
        testInfo.annotations.push({ type: label, description: `${now.length} tiles compared, worst ratio ${worst.toFixed(2)}, mean ${(sum / Math.max(1, count)).toFixed(2)}` });
        return now.length;
      };

      // Yields + Settle (a Settler selected, as the game does by itself).
      await setOverlay(page, 'Yields', true);
      await frame(page, settlerAt, 0.9, { kind: 'unit', id: settler });
      await expect(lensRadio(page, 'Settle')).toBeChecked(); // switched on by itself, overlay still on
      await expect(overlayBox(page, 'Yields')).toBeChecked();
      const a = await baseline('yields+settle', settlerAt, { kind: 'unit', id: settler });
      expect(await under('yields+settle', 'Settle', settlerAt, a.eligible, { kind: 'unit', id: settler })).toBeGreaterThan(4);
      await shot(page, testInfo, `${size.width}-yields-settle`);
      await expectPanelClear(page);
      // The dock yields height to the unit panel: Found City stays whole with Yields on, and with every overlay on.
      await expectPrimaryWhole(page, /^Found City/, 'Found City with Yields over Settle');
      await setOverlay(page, 'Resources', true);
      await setOverlay(page, 'Borders', true);
      // A short window keeps the overlay keys behind a button, so the unit panel keeps its room.
      await expect(page.locator('.overlay-legend')).toHaveCount(size.height <= 700 ? 0 : 1);
      await expectPrimaryWhole(page, /^Found City/, 'Found City with every overlay over Settle');
      if (size.height <= 700) {
        await page.getByRole('button', { name: 'Overlay key' }).click();
        await expect(page.locator('.lens-border-key')).toBeInViewport();
        await expect(page.locator('.lens-owner-key span')).toHaveCount(8);
      }
      await shot(page, testInfo, `${size.width}-settler-all-on`);
      await setOverlay(page, 'Resources', false);
      await setOverlay(page, 'Borders', false);

      // Yields + Threat.
      await page.evaluate(() => (window as unknown as MapWindow).legatum.getState().select(null));
      const b = await baseline('yields+threat', home);
      expect(await under('yields+threat', 'Threat', home, b.eligible)).toBeGreaterThan(4);
      await shot(page, testInfo, `${size.width}-yields-threat`);
      for (const lens of ['Movement', 'Discovery']) {
        expect(await under(`yields+${lens.toLowerCase()}`, lens, home, b.eligible)).toBeGreaterThan(4);
        await shot(page, testInfo, `${size.width}-yields-${lens.toLowerCase()}`);
      }

      // All three overlays over Default, near and far.
      await lensRadio(page, 'Default').click();
      await setOverlay(page, 'Resources', true);
      await setOverlay(page, 'Borders', true);
      expect(await under('all+default', 'Default', home, b.eligible)).toBeGreaterThan(4);
      await shot(page, testInfo, `${size.width}-all-default`);
      await expectPanelClear(page);
      await frame(page, home, 1.5);
      await shot(page, testInfo, `${size.width}-all-default-near`);
      await frame(page, home, 0.45);
      await shot(page, testInfo, `${size.width}-all-default-far`);
      // And every lens with all three on: the map redraws without errors and the panel stays whole.
      for (const lens of ['Settle', 'Movement', 'Threat', 'Discovery']) {
        await lensRadio(page, lens).click();
        await frame(page, home, 0.9);
        await expectPanelClear(page);
        await shot(page, testInfo, `${size.width}-all-${lens.toLowerCase()}`);
      }
      expect(problems).toEqual([]);
      expect(errors).toEqual([]);
    });
  });
}
