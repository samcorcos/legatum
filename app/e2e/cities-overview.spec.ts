// The Cities overview and "where to build this" (user, 2026-10-17: "to find that now I have to click on every single city to
// see how many turns it would take"). Five cities of yours, each different on purpose (src/test/citiesFixture.ts).
// Shots go to test-results/cities-overview/<size>/ for the UI reviewer.
import { mkdir } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import type { Page } from '@playwright/test';
import { test, expect } from './fixtures';
import { auditLayout } from './layoutAudit';
import { coreFixture } from '../src/test/coreFixture';
import { citiesFixture } from '../src/test/citiesFixture';
import type { useGame } from '../src/store';

type GameWindow = Window & { legatum: typeof useGame };
const SIZES = [{ width: 1000, height: 600 }, { width: 1024, height: 600 }, { width: 1440, height: 900 }];
const out = fileURLToPath(new URL('../test-results/cities-overview/', import.meta.url));

async function load(page: Page, mutate?: (f: ReturnType<typeof citiesFixture>) => void) {
  const f = citiesFixture();
  mutate?.(f);
  const game = coreFixture(f);
  await page.addInitScript(({ game }) => {
    if (!location.protocol.startsWith('http')) return;
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn: 2, names: ['Ada'], savedAt: new Date().toISOString(), game }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
    localStorage.removeItem('legatum.cities-sort.v1');
  }, { game });
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('2');
  await expect(page.locator('.loading-screen')).toHaveCount(0);
}

interface CityFacts { id: number; name: string; pos: { q: number; r: number }; population: number; production: number; gl: number | null }
const facts = (page: Page) => page.evaluate((): CityFacts[] => {
  const s = (window as unknown as GameWindow).legatum.getState();
  const v = s.views![s.seat];
  return v.cities.filter((c) => c.owner === v.me && c.detail).map((c) => ({
    id: c.id, name: c.name, pos: c.pos, population: c.population, production: c.detail!.yields.total.production,
    gl: c.detail!.options.find((o) => o.item.kind === 'Wonder' && o.item.building === 'GreatLibrary')?.turns ?? null,
  }));
});
const orders = (page: Page) => page.evaluate(() => { const s = (window as unknown as GameWindow).legatum.getState(); return s.orders[s.seat]; });
const select = (page: Page, id: number) => page.evaluate((id) => (window as unknown as GameWindow).legatum.getState().select({ kind: 'city', id }, true), id);

async function shot(page: Page, size: { width: number; height: number }, name: string, park = true) {
  if (park) await page.mouse.move(size.width - 2, size.height - 2);
  await page.evaluate(() => document.fonts.ready.then(() => undefined));
  await page.waitForTimeout(400);
  await mkdir(`${out}${size.width}x${size.height}/`, { recursive: true });
  await page.screenshot({ path: `${out}${size.width}x${size.height}/${name}.png`, animations: 'disabled' });
}

const names = (page: Page) => page.locator('[data-testid="city-row"] .co-city').allTextContents().then((t) => t.map((x) => x.trim()));
/** A numeric cell of every row, in order: td 1 is Food, 2 Production, 3 Gold, 4 Science (td 0 is population). */
const column = (page: Page, nth: number) => page.locator('[data-testid="city-row"]').evaluateAll((rows, nth) => rows.map((r) => parseInt(r.querySelectorAll('td:not(.co-select)')[nth].firstChild?.textContent ?? '', 10)), nth);

for (const size of SIZES) test.describe(`Cities overview ${size.width}x${size.height}`, () => {
  test.use({ viewport: size });

  test('opens from the top bar and from C; lists every city; sorts by any column; a row goes to the city', async ({ page }) => {
    await load(page);
    const sheet = page.getByTestId('cities-overview');
    await expect(sheet).toHaveCount(0);
    const citiesButton = page.getByTestId('cities-button');
    await expect(citiesButton).toBeVisible();
    await expect(citiesButton.locator('.icon')).toBeVisible();
    await expect(citiesButton).toHaveAttribute('title', /Every city of yours/);

    // Opens from the top bar, and C opens and closes it.
    await citiesButton.click();
    await expect(sheet).toBeVisible();
    await page.keyboard.press('Escape');
    await expect(sheet).toHaveCount(0);
    await page.locator('[data-game-map]').focus();
    await page.keyboard.press('c');
    await expect(sheet).toBeVisible();
    await page.keyboard.press('c');
    await expect(sheet).toHaveCount(0);
    await citiesButton.click();
    await expect(sheet).toBeVisible();

    await expect(page.getByTestId('cities-count')).toHaveText('5 cities · 1 idle');
    await expect(page.getByTestId('city-row')).toHaveCount(5);
    for (const heading of ['City', 'Population', 'Food', 'Production', 'Gold', 'Science', 'Strength', 'Building']) await expect(sheet.getByRole('columnheader', { name: heading })).toBeVisible();

    // It fits: inside the window, no sideways scroll, no layout problem, the primary in view.
    const box = (await sheet.boundingBox())!;
    expect(box.x).toBeGreaterThanOrEqual(0); expect(box.y).toBeGreaterThanOrEqual(0);
    expect(box.x + box.width).toBeLessThanOrEqual(size.width); expect(box.y + box.height).toBeLessThanOrEqual(size.height);
    const fit = await page.locator('.co-scroll').evaluate((el) => ({ scrollWidth: el.scrollWidth, clientWidth: el.clientWidth, columns: [...el.querySelectorAll('thead th')].map((th) => Math.round(th.getBoundingClientRect().width)) }));
    expect(fit.scrollWidth <= fit.clientWidth + 1, `no sideways scroll: ${JSON.stringify(fit)}`).toBe(true);
    await expect(sheet.getByRole('button', { name: 'Close cities' })).toBeInViewport();
    const issues = (await auditLayout(page, { minTarget: 24 })).filter((i) => i.kind !== 'overlap');
    expect(issues).toEqual([]);

    // Opens sorted by Production, highest first; the numbers are the view's own.
    const f = await facts(page);
    await expect(sheet.getByRole('columnheader', { name: 'Production' })).toHaveAttribute('aria-sort', 'descending');
    const prod = await column(page, 2);
    expect(prod).toEqual([...prod].sort((a, b) => b - a));
    expect([...prod].sort()).toEqual(f.map((c) => c.production).sort());
    await shot(page, size, '01-overview-by-production');

    // Population, highest first; a second click flips it.
    await sheet.locator('[data-sort="population"]').click();
    expect(await names(page)).toEqual(['Basel', 'Bern', 'Zurich', 'Lugano', 'Geneva']);
    await sheet.locator('[data-sort="population"]').click();
    expect(await names(page)).toEqual(['Geneva', 'Lugano', 'Zurich', 'Bern', 'Basel']);
    await expect(sheet.getByRole('columnheader', { name: 'Population' })).toHaveAttribute('aria-sort', 'ascending');
    // Every other column sorts too: each one orders its own numbers.
    for (const [key, nth] of [['food', 1], ['gold', 3], ['science', 4], ['defence', 5]] as const) {
      await sheet.locator(`[data-sort="${key}"]`).click();
      const values = key === 'defence'
        ? await page.locator('[data-testid="city-row"] .co-def b').allTextContents().then((t) => t.map(Number))
        : await column(page, nth);
      expect(values, key).toEqual([...values].sort((a, b) => b - a));
    }
    await sheet.locator('[data-sort="name"]').click();
    expect(await names(page)).toEqual(['Basel', 'Bern', 'Geneva', 'Lugano', 'Zurich']);
    // The idle city is flagged, and "Building" puts it first.
    await expect(page.locator('[data-testid="city-row"] [data-testid="city-idle"]')).toHaveCount(1);
    await sheet.locator('[data-sort="build"]').click();
    expect((await names(page))[0]).toBe('Geneva');
    expect(await page.evaluate(() => localStorage.getItem('legatum.cities-sort.v1'))).toBe('{"key":"build","dir":"asc"}');
    await shot(page, size, '02-sorted-by-building');

    // A row goes to the city: the overview closes, the map centres on it and its panel opens.
    await sheet.locator('[data-sort="production"]').click();
    const bern = f.find((c) => c.name === 'Bern')!;
    await page.locator('[data-testid="city-row"]', { hasText: 'Bern' }).locator('td.co-buildcell').click();
    await expect(sheet).toHaveCount(0);
    await expect(page.locator('.city-panel .cp-name')).toHaveText('Bern');
    const state = await page.evaluate(() => { const s = (window as unknown as GameWindow).legatum.getState(); return { selection: s.selection, focus: s.focus?.hex }; });
    expect(state.selection).toEqual({ kind: 'city', id: bern.id });
    expect(state.focus).toEqual(bern.pos);
    await shot(page, size, '03-after-going-to-bern');
  });

  test('the header stays put while many cities scroll', async ({ page }) => {
    await load(page);
    await page.evaluate(() => {
      const store = (window as unknown as GameWindow).legatum;
      const s = store.getState(), v = structuredClone(s.views![s.seat]);
      const base = v.cities.find((c) => c.owner === v.me && c.detail)!;
      for (let i = 0; i < 14; i++) v.cities.push({ ...structuredClone(base), id: 800 + i, name: `Newtown ${i + 1}`, is_capital: false });
      store.setState({ views: [v] });
    });
    await page.keyboard.press('c');
    await expect(page.getByTestId('city-row')).toHaveCount(19);
    const scroller = page.locator('.co-scroll');
    expect(await scroller.evaluate((el) => el.scrollHeight > el.clientHeight)).toBe(true);
    await scroller.evaluate((el) => { el.scrollTop = el.scrollHeight; });
    const head = (await page.locator('.co-table thead th').first().boundingBox())!;
    const area = (await scroller.boundingBox())!;
    expect(Math.abs(head.y - area.y)).toBeLessThan(1.5);
    await expect(page.locator('[data-testid="city-row"]').last()).toBeInViewport();
    await shot(page, size, '04-many-cities-scrolled');
    // Return closes it (the primary), and the map has the keys again.
    await page.keyboard.press('Enter');
    await expect(page.getByTestId('cities-overview')).toHaveCount(0);
  });

  /** A tile-chosen build's placement is open: pick its first lit tile, as a click on the map would. */
  const pickFirstLit = (page: Page) => page.evaluate(() => {
    const w = window as unknown as { legatum: { getState(): { place: object | null; set(p: object): void } }; legatumMap: { place: { candidates: { q: number; r: number }[] } | null } };
    const s = w.legatum.getState();
    s.set({ place: { ...s.place, pick: w.legatumMap.place!.candidates[0] } });
  });

  test('a Great Work is one card; Build lights its tiles on the map, and the pick records the chosen site', async ({ page }) => {
    await load(page);
    const data = await facts(page);
    const ready = data.filter((c) => c.gl !== null && c.gl > 0);
    expect(ready.map((c) => c.name).sort()).toEqual(['Bern', 'Zurich']);
    const zurich = data.find((c) => c.name === 'Zurich')!;
    await select(page, zurich.id);
    const sites = page.locator('.cp-great-works .production-choice[data-item^="w:GreatLibrary:"]');
    await expect(sites).toHaveCount(1);
    const chosen = sites.first();
    await expect(chosen).toContainText('one in the world');
    await chosen.scrollIntoViewIfNeeded();
    await chosen.locator('button.prod').click();
    await expect(page.getByTestId('city-placement')).toBeVisible();
    await shot(page, size, '05-great-library-tile-picker');
    await pickFirstLit(page);
    expect(await orders(page)).toEqual([{ type: 'SetProductionQueue', city: zurich.id, queue: [{ kind: 'Wonder', building: 'GreatLibrary', tile: expect.objectContaining({ q: expect.any(Number), r: expect.any(Number) }) }] }]);
    await select(page, zurich.id);
    await expect(chosen).toContainText('Chosen');
    await expect(chosen).toContainText('gives up');
    await shot(page, size, '06-great-library-site-chosen');
  });

  test('Queue adds a chosen wonder tile behind current production; a costly building keeps its city comparison', async ({ page }) => {
    await load(page, (f) => { f.players[0].techs.push('IronWorking'); });
    const data = await facts(page);
    const bern = data.find((c) => c.name === 'Bern')!;
    await select(page, bern.id);
    const site = page.locator('.cp-great-works .production-choice[data-item^="w:GreatLibrary:"]').first();
    await site.locator('button.queue-add').click();
    await pickFirstLit(page);
    const queue = (await orders(page)).find((o) => o.type === 'SetProductionQueue' && o.city === bern.id);
    expect(queue && queue.type === 'SetProductionQueue' ? queue.queue.map((i) => i.kind === 'Building' || i.kind === 'Wonder' ? i.building : i.kind) : []).toEqual(['Market', 'GreatLibrary']);
    await expect(site).toContainText('Queued');

    // The Forge costs more than 150 Production: its own card says where it is fastest.
    const card = page.locator('.production-choice[data-item="b:Forge"]');
    await card.scrollIntoViewIfNeeded();
    const line = card.getByTestId('where-line');
    await expect(line).toHaveText(/^Best city for this: (here|\w+), done in \d+ turns?\./);
    await card.locator('[data-testid="where-to-build"] > summary').click();
    await expect(card.getByTestId('wtb-row')).toHaveCount(5);
    await card.scrollIntoViewIfNeeded();
    await shot(page, size, '07-costly-building-card');
    // A cheap building has no such line (the Market is Bern's own current build, a card in the list).
    await expect(page.locator('.production-choice[data-item="b:Market"]')).toHaveCount(1);
    await expect(page.locator('.production-choice[data-item="b:Market"] [data-testid="where-to-build"]')).toHaveCount(0);
  });

  test('before the technology is researched, its card and hovercard say where it will be fastest, with no buttons', async ({ page }) => {
    await load(page, (f) => { f.players[0].techs = f.players[0].techs.filter((t) => t !== 'Philosophy'); });
    await page.keyboard.press('t');
    const d = page.locator('dialog.tech-modal');
    await expect(d).toBeVisible();
    const card = d.locator('.era-card[data-tech="Philosophy"]');
    await card.scrollIntoViewIfNeeded();
    await card.locator('li[data-unlock-hover="building"]').first().hover();
    const hover = page.getByTestId('unlock-card');
    await expect(hover).toBeVisible();
    await expect(hover.getByTestId('uc-where')).toContainText(/Best city for this once researched: \w+, done in \d+ turns?/);
    await shot(page, size, '08-unlock-hovercard', false);
    await page.mouse.move(size.width / 2, 4);
    await card.getByRole('button', { name: /^Show details of Philosophy/ }).click();
    const where = d.locator('.research-detail [data-testid="where-to-build"][data-item="b:GreatLibrary"]');
    await where.scrollIntoViewIfNeeded();
    await expect(where.getByTestId('where-line')).toHaveText(/^Best city for this once researched: \w+, done in \d+ turns?\./);
    await where.locator('summary').click();
    await expect(where.getByTestId('wtb-row')).toHaveCount(5);
    await expect(where.getByRole('button', { name: /^(Build|Queue) here/ })).toHaveCount(0);
    await shot(page, size, '09-research-details');
  });
});
