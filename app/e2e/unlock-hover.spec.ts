// The unlock hovercard on the research screen (user 2026-10-11: "it should give me the feeling of 'ooh, that's cool, I
// want that'"): big art, what it is for, what it beats, how it compares with what it replaces. It appears on hover,
// on keyboard focus (pinned with Enter) and on a tap; it never covers the tech card it describes, and it fits at
// 1000x600. Classical cards: Mathematics (Phalangite, the Spearmen upgrade, Siege), Engineering, Iron Working.
// Shots go to test-results/unlock-hover/<size>/ for the UI reviewer.
import { mkdir } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import type { Locator, Page } from '@playwright/test';
import { test, expect } from './fixtures';
import { coreFixture } from '../src/test/coreFixture';
import { researchFixture } from '../src/test/researchFixture';

const SIZES = [{ width: 1000, height: 600 }, { width: 1440, height: 900 }];
const out = fileURLToPath(new URL('../test-results/unlock-hover/', import.meta.url));

/** Ada has finished the Ancient era and is in the Classical one. */
async function load(page: Page) {
  const f = researchFixture('India');
  f.players[0].techs = ['Irrigation', 'BronzeWorking', 'Masonry', 'Currency', 'Writing'];
  f.players[0].research = null as never;
  f.players[0].research_queue = [] as never;
  const game = coreFixture(f);
  await page.addInitScript(({ game }) => {
    if (!location.protocol.startsWith('http')) return;
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn: 2, names: ['Ada'], savedAt: new Date().toISOString(), game }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, { game });
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('2');
  await expect(page.locator('.loading-screen')).toHaveCount(0);
  await page.locator('.topbar .research-chip').click();
  await expect(page.locator('dialog.tech-modal')).toBeVisible();
  await page.waitForTimeout(300);
  return page.locator('dialog.tech-modal');
}

type Box = { x: number; y: number; width: number; height: number };
const intersects = (a: Box, b: Box) => a.x < b.x + b.width && b.x < a.x + a.width && a.y < b.y + b.height && b.y < a.y + a.height;

async function shot(page: Page, size: { width: number; height: number }, name: string) {
  await page.evaluate(() => document.fonts.ready.then(() => undefined));
  await page.waitForTimeout(250);
  await mkdir(`${out}${size.width}x${size.height}/`, { recursive: true });
  await page.screenshot({ path: `${out}${size.width}x${size.height}/${name}.png`, animations: 'disabled' });
}

/** The hovercard is whole on screen and clear of the tech card it describes (and of the row it hangs from). */
async function expectBeside(page: Page, card: Locator, row: Locator, size: { width: number; height: number }) {
  const c = (await page.getByTestId('unlock-card').boundingBox())!;
  const t = (await card.boundingBox())!;
  const r = (await row.boundingBox())!;
  expect(c.x, 'inside the window, left').toBeGreaterThanOrEqual(0);
  expect(c.y, 'inside the window, top').toBeGreaterThanOrEqual(0);
  expect(c.x + c.width, 'inside the window, right').toBeLessThanOrEqual(size.width + 0.5);
  expect(c.y + c.height, 'inside the window, bottom').toBeLessThanOrEqual(size.height + 0.5);
  expect(intersects(c, t), 'the hovercard never covers the tech card it describes').toBe(false);
  expect(intersects(c, r), 'nor the unlock row').toBe(false);
}

for (const size of SIZES) test.describe(`unlock hovercard ${size.width}x${size.height}`, () => {
  test.use({ viewport: size });

  test('a unit sells itself: art, class, history, matchup, the comparison with its predecessor and the upgrade price', async ({ page }) => {
    const d = await load(page);
    const card = d.locator('.era-card[data-tech="Mathematics"]');
    const row = card.locator('.era-unlock.unit', { hasText: 'Phalangite' });
    await row.hover();
    const hover = page.getByTestId('unlock-card');
    await expect(hover).toBeVisible();
    await expect(hover.locator('.uc-name')).toHaveText('Phalangite');
    await expect(hover.getByTestId('uc-class')).toHaveText('Melee line');
    const art = hover.locator('.uc-art img');
    await expect(art).toBeVisible();
    expect((await art.boundingBox())!.height, 'a large miniature, not a glyph').toBeGreaterThanOrEqual(120);
    await expect(hover.getByTestId('uc-flavour')).toContainText('sarissas');
    await expect(hover.getByTestId('uc-matchup')).toContainText('mounted');
    await expect(hover.getByTestId('uc-matchup')).toContainText('Weak to');
    await expect(hover.locator('[data-stat="strength"]')).toContainText('20');
    await expect(hover.locator('[data-stat="strength"]')).toContainText('25');
    await expect(hover.locator('[data-stat="strength"] .uc-delta')).toHaveText('+25%');
    await expect(hover).toContainText('Compared with Spearman');
    await expect(hover.getByTestId('uc-upgrade')).toContainText('Upgrade from Spearman');
    await expect(hover.getByTestId('uc-upgrade').locator('img')).toHaveCount(2);
    await expectBeside(page, card, row, size);
    await shot(page, size, '01-phalangite');
    // Moving away takes it down; the pointer is never trapped by it.
    await page.mouse.move(size.width / 2, 8);
    await expect(hover).toHaveCount(0);
  });

  test('a unit and a building each have their own card', async ({ page }) => {
    const d = await load(page);
    const maths = d.locator('.era-card[data-tech="Mathematics"]');
    await maths.locator('.era-unlock.unit', { hasText: 'Phalangite' }).hover();
    const hover = page.getByTestId('unlock-card');
    await expect(hover).toBeVisible();
    await expect(hover).toContainText('Phalangite');
    await expectBeside(page, maths, maths.locator('.era-unlock.unit', { hasText: 'Phalangite' }), size);
    await shot(page, size, '02-phalangite');
    await page.mouse.move(size.width / 2, 8);
    await expect(hover).toHaveCount(0);

    const phil = d.locator('.era-card[data-tech="Philosophy"]');
    const rowB = phil.locator('.era-unlock.building').first();
    await rowB.scrollIntoViewIfNeeded();
    await rowB.hover();
    await expect(hover).toBeVisible();
    await expect(hover).toHaveAttribute('data-type', 'building');
    await expect(hover.getByTestId('uc-gives')).toContainText('What it gives');
    await expect(hover.getByTestId('uc-why')).toContainText('Why build it');
    await expect(hover.getByTestId('uc-flavour')).not.toBeEmpty();
    await expect(hover.locator('.uc-art')).toBeVisible();
    await expectBeside(page, phil, rowB, size);
    await shot(page, size, '03-building');
  });

  test('a unit unlock folds its upgrade path into one row and shows both miniatures in the hovercard', async ({ page }) => {
    const d = await load(page);
    const card = d.locator('.era-card[data-tech="Mathematics"]');
    const row = card.locator('.era-unlock.unit', { hasText: 'From Spearman' });
    await expect(row).toHaveCount(1);
    await expect(card).not.toContainText('can upgrade');
    await expect(row.locator('.era-unlock-art[data-art="unit"] img')).toBeVisible();
    await row.hover();
    const hover = page.getByTestId('unlock-card');
    await expect(hover).toBeVisible();
    await expect(hover).toHaveAttribute('data-type', 'unit');
    await expect(hover.locator('.uc-upgrade .uc-pair .uc-figure')).toHaveCount(2);
    await expect(hover).toContainText('Spearman');
    await expect(hover).toContainText('Phalangite');
    await expectBeside(page, card, row, size);
    await shot(page, size, '04-upgrade-effect');
  });

  test('no placeholder: every unlock row on every Classical card has art or a real mark', async ({ page }) => {
    const d = await load(page);
    for (const row of await d.locator('.era-unlock').all()) {
      const art = row.locator('.era-unlock-art');
      await expect(art).toBeVisible();
      expect(await art.locator('.icon, img').count(), 'the row has a painting or glyph').toBeGreaterThan(0);
      for (const img of await art.locator('img').all()) {
        expect(await img.evaluate((el: HTMLImageElement) => el.naturalWidth), 'paintings load').toBeGreaterThan(0);
      }
    }
  });

  test('Classical with nothing queued: the queue strip is one row high and the first card starts above the fold', async ({ page }) => {
    const d = await load(page);
    const strip = (await d.locator('.research-queue').boundingBox())!;
    expect(strip.height, 'one row of slots').toBeLessThan(size.height === 600 ? 130 : 150);
    expect((await d.locator('.era-card').first().boundingBox())!.y, 'cards start above the fold').toBeLessThan(size.height - 200);
    await shot(page, size, '05-classical-idle');
  });

  test('keyboard: Tab to a row shows the card, Enter pins it, Escape closes it and leaves the research screen open', async ({ page }) => {
    const d = await load(page);
    const row = d.locator('.era-card[data-tech="Mathematics"] .era-unlock.unit', { hasText: 'Phalangite' });
    for (let i = 0; i < 60 && !(await row.evaluate((el) => el === document.activeElement)); i++) await page.keyboard.press('Tab');
    expect(await row.evaluate((el) => el === document.activeElement), 'Tab reaches the row').toBe(true);
    const hover = page.getByTestId('unlock-card');
    await expect(hover).toBeVisible();
    await expect(hover.locator('.uc-name')).toHaveText('Phalangite');
    await page.keyboard.press('Enter');
    await expect(hover).toHaveAttribute('role', 'dialog');
    await page.keyboard.press('Escape');
    await expect(hover).toHaveCount(0);
    await expect(d).toBeVisible();
    // Enter on the row never closed the screen or researched the tech.
    await expect(d.locator('.era-card[data-tech="Mathematics"]')).not.toHaveAttribute('data-state', 'researching');
  });
});

test.describe('touch', () => {
  test.use({ viewport: SIZES[0], hasTouch: true });
  test('a tap pins the card with a Close button instead of researching the tech', async ({ page }) => {
    const d = await load(page);
    const card = d.locator('.era-card[data-tech="Mathematics"]');
    const row = card.locator('.era-unlock.unit', { hasText: 'Phalangite' });
    await row.tap();
    const hover = page.getByTestId('unlock-card');
    await expect(hover).toHaveAttribute('role', 'dialog');
    await expect(card).not.toHaveAttribute('data-state', 'researching');
    await expectBeside(page, card, row, SIZES[0]);
    await hover.getByRole('button', { name: 'Close details' }).tap();
    await expect(hover).toHaveCount(0);
  });
});
