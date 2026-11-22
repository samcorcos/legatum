// From the Classical era, strategic supply has an era resource: the Resources overlay, the era's supply chip with its breakdown, and the
// Classical and Medieval units that use 1 Iron (blocked, with the core's reason, when no Iron is supplied), in the real WASM engine.
import { test as base, expect, type Page } from '@playwright/test';
import { createServer } from 'vite';
import { mkdirSync } from 'node:fs';
import { endPlanning } from './turnActions';
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

const SHOTS = process.env.LEGATUM_SHOTS_DIR;

/**
 * Advance the founded capital without production, then edit the engine save losslessly (u64 bitset words stay exact) so the
 * capital owns an Iron deposit (or, with `deposit: false`, none, though Iron is revealed) and every Ancient technology
 * is known, so the empire is in the Classical era and Iron is its era resource (its chip is always in the top bar).
 * With `coal`, it also owns two Coal deposits and knows every technology up to the Renaissance plus
 * Industrialization (which reveals Coal), so it is in the Industrial era and Coal is its era resource.
 */
async function ironScenario(page: Page, { coal = false, deposit = true }: { coal?: boolean; deposit?: boolean } = {}) {
  // Screenshots should show the resource UI, not the first-game hints.
  await page.addInitScript(() => {
    if (location.protocol.startsWith('http')) localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  });
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Single Player/ }).click();
  await page.getByRole('button', { name: /^Start game/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('1');
  await endPlanning(page);
  await expect(page.getByTestId('turn-number')).toHaveText('2');
  if (await page.locator('.moment-card, [data-celebrate-delta]').count()) await page.keyboard.press('Escape');
  const city = await page.evaluate(async ({ coal, deposit }) => {
    const { useGame } = await import('/src/store.ts' as string) as typeof import('../src/store');
    const { eraTechs } = await import('/src/data.ts' as string) as typeof import('../src/data');
    const s = useGame.getState();
    const raw = s.client!.save();
    // Quote big integers before parsing so nothing loses precision, then unquote them.
    const state = JSON.parse(raw.replace(/([:,[]\s*)(-?\d{16,})(?=\s*[,\]}])/g, '$1"__n$2"'));
    delete state.archive;
    const me = state.players[0];
    // The era is the first with an unknown technology: every Ancient one known is the Classical era, every one up to the Renaissance the Industrial.
    const known = [0].concat(coal ? [1, 2, 3] : []).flatMap((era) => eraTechs(era));
    me.techs = [...new Set([...me.techs, ...known, 'IronWorking', 'BronzeWorking', 'Masonry', 'Feudalism', ...(coal ? ['Industrialization'] : [])])];
    const capital = Object.values(state.cities as Record<string, { id: number; owner: number; pos: { q: number; r: number } }>).find((c) => c.owner === 0)!;
    const w = state.map.width;
    const centre = capital.pos.r * w + capital.pos.q + Math.floor(capital.pos.r / 2);
    const owned = state.map.tiles.map((t: { city: number | null }, i: number) => (t.city === capital.id && i !== centre ? i : -1)).filter((i: number) => i >= 0);
    if (deposit) state.map.tiles[owned[0]].resource = 'Iron';
    if (coal) for (const i of owned.slice(1, 3)) state.map.tiles[i].resource = 'Coal';
    const game = JSON.stringify(state).replace(/"__n(-?\d+)"/g, '$1');
    await s.loadSave({ format: 'legatum-save', version: 1, client: 'wasm', savedAt: '', turn: state.turn, names: ['Player 1'], game });
    return capital.id;
  }, { coal, deposit });
  await expect(page.getByTestId('turn-number')).toHaveText('2');
  await expect(page.locator('.loading-screen')).toHaveCount(0);
  return city;
}

async function shot(page: Page, name: string) {
  if (!SHOTS) return;
  // Let entrance animations (tooltips, sheets) finish before capturing.
  await page.waitForFunction(() => document.getAnimations().every((a) => a.playState !== 'running' || a.effect?.getTiming().iterations === Infinity));
  mkdirSync(SHOTS, { recursive: true });
  await page.screenshot({ path: `${SHOTS}/${name}.png` });
}

const sizes = [
  { name: '1000', width: 1000, height: 600, touch: false },
  { name: '1440', width: 1440, height: 900, touch: false },
];
for (const size of sizes) for (const look of ['atlas'] as const) test.describe(`${look} ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height }, hasTouch: size.touch, isMobile: size.touch });
  test('two Coal deposits supply 4 Coal, and the chip and tooltip show every line of the sum', async ({ page }) => {
    test.setTimeout(90_000);
    const errors: string[] = [];
    page.on('pageerror', (e) => errors.push(e.message));
    await ironScenario(page, { coal: true });
    const chip = page.locator('.supply-stat.supply-coal');
    await expect(chip).toBeVisible();
    await expect(page.getByTestId('supply-Coal')).toHaveText('4/4');
    const bar = await page.locator('.topbar').boundingBox();
    expect(bar!.width).toBeLessThanOrEqual(size.width);
    expect(await page.locator('.topbar').evaluate((el) => el.scrollWidth - el.clientWidth)).toBeLessThanOrEqual(0);
    await chip.hover();
    await expect(page.locator('.tip-summary').filter({ hasText: 'Coal:' })).toContainText('Coal: 0 of 4 at work, 4 to spare');
    if (SHOTS) {
      await page.waitForTimeout(400);
      mkdirSync(SHOTS, { recursive: true });
      await page.screenshot({ path: `${SHOTS}/supply-hover-${size.name}.png` });
    }
    await chip.click();
    const tip = page.locator('.supply-tip');
    await expect(tip).toBeVisible();
    await expect(tip).toContainText('0 of 4 Coal at work');
    await expect(tip).toContainText(/Coal(?: with a Mine)?, (?:\d+ tiles? [NESW]+ of |in your land)/);
    await expect(tip).toContainText('Spare');
    await expect(tip).not.toContainText(/stockpile|stored|a turn/i);
    await shot(page, `supply-click-${size.name}`);
    expect(errors).toEqual([]);
  });

  test('resources overlay, Iron supply chip, and the units that use Iron', async ({ page }) => {
    test.setTimeout(90_000);
    const errors: string[] = [];
    page.on('pageerror', (e) => errors.push(e.message));
    const city = await ironScenario(page);
    const phone = size.width < 700;

    // Top bar: the Iron chip shows what the buildings and units want over what the deposit supplies; the tooltip names the deposit.
    const chip = page.locator('.supply-stat.supply-iron');
    await expect(chip).toBeVisible();
    await expect(page.getByTestId('supply-Iron')).toHaveText('2/2');
    const bar = await page.locator('.topbar').boundingBox();
    expect(bar!.width).toBeLessThanOrEqual(size.width);
    const overflow = await page.locator('.topbar').evaluate((el) => el.scrollWidth - el.clientWidth);
    expect(overflow).toBeLessThanOrEqual(0);
    if (phone) await chip.tap(); else await chip.click();
    const tip = page.locator('.supply-tip');
    await expect(tip).toBeVisible();
    await expect(tip).toContainText('0 of 2 Iron at work');
    await expect(tip).toContainText(/Iron(?: with a Mine)?, (?:\d+ tiles? [NESW]+ of |in your land)/);
    await shot(page, `topbar-chip-${look}-${size.name}`);
    await page.keyboard.press('Escape');

    // The production picker offers the Iron units with "Uses 1 Iron": a spare Iron is supplied, so none is blocked.
    await page.evaluate(async (id) => {
      const { useGame } = await import('/src/store.ts' as string) as typeof import('../src/store');
      useGame.getState().select({ kind: 'city', id }, true);
    }, city);
    if (phone) {
      const select = page.getByRole('combobox', { name: /^Production for / });
      await expect(select).toBeVisible();
      const options = select.locator('option', { hasText: 'Uses 1 Iron' });
      expect(await options.count()).toBeGreaterThan(0);
      for (const option of await options.all()) await expect(option).toBeEnabled();
    } else {
      const ironUnits = page.locator('.production-choice').filter({ hasText: 'Uses 1 Iron' });
      expect(await ironUnits.count()).toBeGreaterThan(0);
      for (const card of await ironUnits.all()) await expect(card.getByRole('button', { name: /^Build / })).toBeEnabled();
      await expect(ironUnits.first().locator('.prod-supply-need')).toContainText('Uses 1 Iron');
      await ironUnits.first().scrollIntoViewIfNeeded();
    }
    await shot(page, `pikeman-uses-iron-${look}-${size.name}`);

    // The Resources overlay lists the deposit and the tech that reveals the resources still hidden (Niter, Coal, Oil, Silicon).
    await page.evaluate(async () => {
      const { useGame } = await import('/src/store.ts' as string) as typeof import('../src/store');
      useGame.getState().select(null);
    });
    const toggle = page.getByRole('button', { name: /^Lens: / });
    if (phone && await toggle.getAttribute('aria-expanded') !== 'true') await toggle.click();
    const box = overlayBox(page, 'Resources');
    await box.click();
    // On phones the sheet stays open for a checkbox, but close and reopen it to read the key from rest.
    await expect(box).toBeChecked();
    // A short window keeps the key behind a button.
    const opener = page.getByRole('button', { name: 'Overlay key' });
    if (await opener.isVisible()) await opener.click();
    if (phone) await expect(page.locator('.map-lenses')).toContainText('Coal: revealed by Industrialization');
    else {
      // Desktop: what reveals a hidden strategic resource is the ring key's tip.
      await page.locator('.map-lenses .lens-key-row').hover();
      await expect(page.locator('[role="tooltip"]:not([hidden])')).toContainText('Coal: revealed by Industrialization');
      await page.mouse.move(1, 1);
    }
    await shot(page, `resources-overlay-${look}-${size.name}`);
    expect(errors).toEqual([]);
  });

  test('with no Iron supplied the Iron unit cards are greyed out with the reason where their price is', async ({ page }) => {
    test.setTimeout(90_000);
    const errors: string[] = [];
    page.on('pageerror', (e) => errors.push(e.message));
    const city = await ironScenario(page, { deposit: false });
    const phone = size.width < 700;
    await expect(page.getByTestId('supply-Iron')).toHaveText('0/0');
    await page.evaluate(async (id) => {
      const { useGame } = await import('/src/store.ts' as string) as typeof import('../src/store');
      useGame.getState().select({ kind: 'city', id }, true);
    }, city);
    const reason = 'Needs 1 Iron: you have none yet. Bring an Iron deposit inside your borders, or build a Mine on one for +1 Iron.';
    if (phone) {
      const select = page.getByRole('combobox', { name: /^Production for / });
      await expect(select).toBeVisible();
      await expect(select.locator('option', { hasText: reason }).first()).toBeDisabled();
    } else {
      const blocked = page.locator('.production-choice').filter({ hasText: reason });
      expect(await blocked.count()).toBeGreaterThan(0);
      for (const card of await blocked.all()) {
        await expect(card).toHaveClass(/supply-blocked/);
        await expect(card.getByRole('button', { name: /^Build / })).toHaveAttribute('aria-description', /Needs 1 Iron.*Open Iron supply/);
        await expect(card.locator('.prod-go')).toHaveText('Blocked');
        await expect(card.locator('.prod-supply-need')).toContainText('Uses 1 Iron');
        await expect(card.locator('.production-blocker')).toHaveText(reason);
        await expect(card.locator('.prod-effect')).not.toHaveText(reason);
      }
      await blocked.first().scrollIntoViewIfNeeded();
      // The card deliberately exposes aria-disabled while still opening its supply reason on click.
      await blocked.first().getByRole('button', { name: /^Build / }).click({ force: true });
      await expect(page.locator('.supply-tip')).toBeVisible();
      await expect(page.locator('.supply-tip')).toContainText('No Iron deposit inside your borders yet');
      await page.keyboard.press('Escape');
    }
    await shot(page, `blocked-pikeman-${look}-${size.name}`);
    expect(errors).toEqual([]);
  });
});
