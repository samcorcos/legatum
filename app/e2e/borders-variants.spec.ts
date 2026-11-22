// Border legibility shots: the current look and the three bold variants (A, B, C) on a 4-empire and an
// 8-empire mid-game map (src/test/bordersFixture.ts), at 1000x600 and 1440x900, at normal and far zoom.
// BORDERS_SHOTS=<dir> picks where they go; BORDERS_SHEET=<png path> also builds the portrait comparison
// sheet. Shots only.
import { test as base, expect, type Page } from '@playwright/test';
import { createServer } from 'vite';
import { mkdir } from 'node:fs/promises';
import path from 'node:path';
import type { useGame } from '../src/store';
import { coreFixture } from '../src/test/coreFixture';
import { bordersFixture } from '../src/test/bordersFixture';
import { BORDER_VARIANTS, type BorderVariant } from '../src/map/borderVariant';
import { mapBorderColor, mapPlayerColor } from '../src/map/style';
import { FACTION_PALETTE, setSeatFactions } from '../src/theme';
import type { Faction } from '../src/types';

type MapWindow = Window & { legatum: typeof useGame; legatumMap: { camera: { zoom: number }; redraw(): void; setBorderVariant(v: BorderVariant): void } };
const out = process.env.BORDERS_SHOTS ? path.resolve(process.env.BORDERS_SHOTS) : path.resolve('test-results/borders-variants');

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

const maps = [
  { name: '4-empire', opts: { players: 4, width: 40, height: 26, seed: '31337' }, far: 0.3 },
  { name: '8-empire', opts: { players: 8, width: 52, height: 34, seed: '777' }, far: 0.24 },
];

async function open(page: Page, f: ReturnType<typeof bordersFixture>) {
  await page.addInitScript(({ game }) => {
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn: 90, names: ['Ada'], savedAt: new Date().toISOString(), game }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, { game: coreFixture(f.fixture) });
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('90');
  await expect(page.locator('.loading-screen')).toHaveCount(0);
  await page.evaluate(() => document.fonts.ready.then(() => undefined));
}

async function frame(page: Page, at: { q: number; r: number }, zoom: number) {
  await page.evaluate(({ at, zoom }) => {
    const w = window as unknown as MapWindow;
    const s = w.legatum.getState();
    s.set({ techOpen: false });
    s.select(null);
    s.jumpTo(at);
    w.legatumMap.camera.zoom = zoom;
    w.legatumMap.redraw();
  }, { at, zoom });
  await page.waitForTimeout(700);
}

for (const [width, height] of [[1000, 600], [1440, 900]] as const) {
  test.describe(`${width}x${height}`, () => {
    test.use({ viewport: { width, height } });
    for (const m of maps) {
      test(`borders ${m.name}`, async ({ page }) => {
        test.setTimeout(240_000);
        const f = bordersFixture(m.opts);
        await open(page, f);
        await mkdir(`${out}/${width}x${height}`, { recursive: true });
        for (const [tag, zoom, at] of [['near', 0.9, f.meet], ['far', m.far, f.center]] as const) {
          await frame(page, at, zoom);
          for (const v of BORDER_VARIANTS) {
            await page.evaluate((v) => (window as unknown as MapWindow).legatumMap.setBorderVariant(v), v);
            await page.waitForTimeout(500);
            await page.screenshot({ path: `${out}/${width}x${height}/${m.name}-${tag}-${v}.png`, animations: 'disabled' });
          }
        }
      });
    }
  });
}

// The portrait comparison sheet: the same two crops (a junction of empires at normal zoom, the middle
// of the map zoomed out) for the current look and A, B, C, then the seat palette as colour-blind
// players see it. Shot at 2x so the lines read on a phone.
const SHEET = process.env.BORDERS_SHEET;
const LABELS: Record<BorderVariant, [string, string]> = {
  current: ['Now', 'Wash 0.07, a pale hairline in a toned-down seat colour'],
  a: ['A · Stronger line', 'Bold seat-colour line, thin ink edge, narrow inner band'],
  b: ['B · Civ-style two-tone', 'Seat line inside, lighter line outside, soft tint 0.20'],
  c: ['C · Ownership at a glance', 'A plus a clear tint 0.18 and a heavier ink rule between neighbours'],
};
test.describe('sheet', () => {
  test.skip(!SHEET, 'set BORDERS_SHEET to build the comparison sheet');
  test.use({ viewport: { width: 1440, height: 900 }, deviceScaleFactor: 2 });
  test('comparison sheet', async ({ page, browser }) => {
    test.setTimeout(240_000);
    const f = bordersFixture(maps[1].opts);
    await open(page, f);
    const W = 400, H = 270;
    const crops: Record<string, Buffer> = {};
    for (const [tag, zoom, at] of [['near', 0.9, f.meet], ['far', 0.3, f.center]] as const) {
      await frame(page, at, zoom);
      for (const v of BORDER_VARIANTS) {
        await page.evaluate((v) => (window as unknown as MapWindow).legatumMap.setBorderVariant(v), v);
        // Let the 2x chunks finish refining, or a half-sharpened crop is captured.
        await page.waitForTimeout(2500);
        const box = await page.evaluate(() => { const b = document.querySelector('[data-game-map]')!.getBoundingClientRect(); return { x: b.x, y: b.y, w: b.width, h: b.height }; });
        crops[`${tag}-${v}`] = await page.screenshot({ clip: { x: box.x + box.w / 2 - W / 2 - (tag === 'far' ? 120 : 290), y: box.y + box.h / 2 - H / 2 + (tag === 'far' ? 10 : 190), width: W, height: H }, animations: 'disabled' });
      }
    }
    const img = (b: Buffer) => `data:image/png;base64,${b.toString('base64')}`;
    const blocks = BORDER_VARIANTS.map((v) => `<section><h2>${LABELS[v][0]}</h2><p>${LABELS[v][1]}</p>
      <img src="${img(crops[`near-${v}`])}"><img src="${img(crops[`far-${v}`])}"></section>`).join('');
    setSeatFactions(Object.keys(FACTION_PALETTE) as Faction[]);
    const seats = [0, 1, 2, 3, 4, 5, 6, 7, 8];
    const modes: [string, string][] = [['Normal sight', 'none'], ['Protanopia', 'url(#protan)'], ['Deuteranopia', 'url(#deutan)'], ['Tritanopia', 'url(#tritan)']];
    const strip = (colors: string[], filter: string) => `<div class="strip" style="filter:${filter}">${colors.map((c) => `<i style="background:${c}"></i>`).join('')}</div>`;
    const m = (r: number[][]) => [...r.map((row) => [...row, 0, 0].map((n) => n.toFixed(6)).join(' ')), '0 0 0 1 0'].join(' ');
    const palette = modes.map(([name, filter]) => `<div class="row"><b>${name}</b>${strip(seats.map(mapPlayerColor), filter)}${strip(seats.map(mapBorderColor), filter)}</div>`).join('');
    const html = `<!doctype html><meta charset="utf-8"><style>
      body{margin:0;background:#f2ead8;color:#2b2419;font:15px Georgia,serif;width:800px}
      h1{font:600 30px Georgia,serif;margin:0;padding:22px 20px 4px;letter-spacing:.02em}
      .lede{margin:0;padding:0 20px 14px;color:#5b503f}
      section{border-top:2px solid #2b2419;padding-bottom:10px}
      h2{font:600 24px Georgia,serif;margin:12px 20px 0}
      section p{margin:2px 20px 10px;color:#5b503f}
      img{display:block;width:800px;height:540px;margin-bottom:6px}
      .pal{border-top:2px solid #2b2419;padding:12px 20px 20px}
      .row{display:grid;grid-template-columns:130px 1fr 1fr;gap:10px;align-items:center;margin:6px 0}
      .strip{display:flex;height:34px;border:1px solid #2b2419}.strip i{flex:1}
      .cap{display:grid;grid-template-columns:130px 1fr 1fr;gap:10px;color:#5b503f;margin-top:4px}
    </style>
    <svg width="0" height="0" style="position:absolute"><defs>
      <filter id="protan" color-interpolation-filters="linearRGB"><feColorMatrix type="matrix" values="${m([[0.152286, 1.052583, -0.204868], [0.114503, 0.786281, 0.099216], [-0.003882, -0.048116, 1.051998]])}"/></filter>
      <filter id="deutan" color-interpolation-filters="linearRGB"><feColorMatrix type="matrix" values="${m([[0.367322, 0.860646, -0.227968], [0.280085, 0.672501, 0.047413], [-0.01182, 0.04294, 0.968881]])}"/></filter>
      <filter id="tritan" color-interpolation-filters="linearRGB"><feColorMatrix type="matrix" values="${m([[1.255528, -0.076749, -0.178779], [-0.078411, 0.930809, 0.147602], [0.004733, 0.691367, 0.3039]])}"/></filter>
    </defs></svg>
    <h1>Legatum territory borders</h1><p class="lede">Same eight-empire map, mid-game. Each look at normal zoom, then zoomed out.</p>
    ${blocks}
    <div class="pal"><h2 style="margin:0 0 4px">Seat colours, as others see them</h2><p style="margin:0 0 8px;color:#5b503f">Left: the seat colours the map used before. Right: the border palette.</p>${palette}
      <div class="cap"><span></span><span>Before</span><span>Border palette</span></div></div>`;
    const sheet = await browser.newPage({ viewport: { width: 800, height: 1200 }, deviceScaleFactor: 1 });
    await sheet.setContent(html);
    await sheet.waitForTimeout(300);
    await mkdir(path.dirname(path.resolve(SHEET!)), { recursive: true });
    await sheet.screenshot({ path: path.resolve(SHEET!), fullPage: true });
    await sheet.close();
  });
});
