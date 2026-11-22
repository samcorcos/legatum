// Yield amounts carry their icon everywhere (legatum-ui, user 2026-10-06): a Library's "+3 Science"
// is a number and the Science icon on the build card, the tech tree and the unlock card, never words.
// Set YIELD_SHOTS=<dir> to also write screenshots (the same capture runs on master for before/after).
import type { Page } from '@playwright/test';
import { test, expect } from './fixtures';
import { endPlanning } from './turnActions';
import { coreFixture } from '../src/test/coreFixture';
import { researchFixture } from '../src/test/researchFixture';
import type { useGame } from '../src/store';

type GameWindow = Window & { legatum: typeof useGame };
const SIZES = [{ width: 1000, height: 600 }, { width: 1440, height: 900 }];
const shots = process.env.YIELD_SHOTS;
const before = process.env.YIELD_BEFORE === '1';

async function start(page: Page, edit: (f: ReturnType<typeof researchFixture>) => void) {
  const state = researchFixture('India');
  edit(state);
  await page.addInitScript((game) => {
    if (!location.protocol.startsWith('http')) return;
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn: 2, names: ['Ada'], savedAt: new Date().toISOString(), game }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, coreFixture(state));
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('2');
  await expect(page.locator('.loading-screen')).toHaveCount(0);
}

const known = (f: ReturnType<typeof researchFixture>) => { f.players[0].techs = [...f.players[0].techs, 'Writing', 'Currency']; };
const shoot = async (page: Page, name: string, size: { width: number }, target?: ReturnType<Page['locator']>) => {
  if (!shots) return;
  await page.mouse.move(0, 0);
  await (target ?? page).screenshot({ path: `${shots}/${name}-${size.width}.png` });
};

/** Text outside every amount icon: what is left must hold no "+N Word" for a yield. */
const bareAmounts = (page: Page, root: string) => page.evaluate((sel) => {
  const re = /(?<![\w/.,\-–−+])[+−-]?\d[\d,]*(?:[–-]\d+|\/\d[\d,]*| → [+−-]?\d[\d,]*)?[  ](Food|Production|Gold|Science|VP|Copper|Iron|Niter|Coal|Oil|Silicon)\b(?!-)/;
  const found: string[] = [];
  for (const el of document.querySelectorAll(sel)) {
    const clone = el.cloneNode(true) as HTMLElement;
    clone.querySelectorAll('.amt').forEach((a) => a.remove());
    clone.querySelectorAll('.sr-only').forEach((a) => a.remove());
    const text = clone.textContent ?? '';
    if (re.test(text)) found.push(text.slice(0, 160));
  }
  return found;
}, root);

for (const size of SIZES) test.describe(`yield icons ${size.width}x${size.height}`, () => {
  test.use({ viewport: size });

  test('a city build card shows the Library effect as a number and an icon', async ({ page }) => {
    await start(page, known);
    await page.evaluate(() => {
      const s = (window as unknown as GameWindow).legatum.getState();
      const city = s.views![s.seat].cities.find((c) => c.owner === s.views![s.seat].me && c.detail)!;
      s.set({ techOpen: false });
      s.select({ kind: 'city', id: city.id }, true);
    });
    const panel = page.locator('.city-panel');
    const card = panel.locator('.production-choice[data-item="b:Library"]');
    await expect(card).toBeVisible();
    await card.scrollIntoViewIfNeeded();
    if (!before) {
      const effect = card.locator('.prod-effect');
      await expect(effect.locator('.amt[data-amount="Science"]')).toHaveCount(1);
      await expect(effect.locator('.amt .amt-n')).toHaveText('+3');
      await expect(effect).toContainText('+3 Science');
      expect(await bareAmounts(page, '.production-choice .prod-effect')).toEqual([]);
    }
    await shoot(page, 'build-card', size, card);
    await shoot(page, 'city-panel', size, panel);
  });

  test('the era card and its detail show unlock amounts as icons', async ({ page }) => {
    await start(page, (f) => { f.players[0].techs = f.players[0].techs.filter((t) => t !== 'Writing'); });
    await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().set({ techOpen: true }));
    const node = page.locator('.era-card[data-tech="Writing"]');
    await expect(node).toBeVisible();
    await node.locator('.era-details-chevron').click();
    const dialog = page.locator('dialog.tech-panel, dialog').filter({ has: node }).first();
    if (!before) {
      await expect(node.locator('.era-unlock-gain .amt[data-amount="Science"]')).toHaveCount(1);
      await expect(node.locator('.era-cost-main .amt[data-amount="Science"]')).toHaveCount(1);
      await expect(page.locator('.research-unlock .amt[data-amount="Production"]').first()).toBeVisible();
      expect(await bareAmounts(page, '.era-unlock-label, .era-cost, .research-unlock p, .research-detail p')).toEqual([]);
    }
    await shoot(page, 'tech-tree', size, dialog);
  });

  test('the unlock card shows the building effect and cost as icons', async ({ page }) => {
    await start(page, (f) => { f.players[0].research = 'Writing'; f.players[0].science = 5000; });
    await endPlanning(page);
    const card = page.locator('.moment-card.unlock');
    await expect(card).toBeVisible();
    await expect(card).toContainText('Library');
    if (!before) {
      await expect(card.locator('.amt[data-amount="Science"]').first()).toBeVisible();
      await expect(card.locator('.unlock-cost .amt[data-amount="Production"]').first()).toBeVisible();
      expect(await bareAmounts(page, '.moment-card.unlock p, .moment-card.unlock .unlock-cost')).toEqual([]);
    }
    await shoot(page, 'unlock-card', size, card);
  });

  test('a turn report line shows the price of a purchase as a number and an icon', async ({ page }) => {
    await start(page, (f) => { f.players[0].gold = 400; });
    await page.evaluate(() => {
      const s = (window as unknown as GameWindow).legatum.getState();
      const city = s.views![s.seat].cities.find((c) => c.owner === s.views![s.seat].me && c.detail)!;
      s.set({ techOpen: false });
      s.editOrders((o) => [...o, { type: 'BuyItem', city: city.id, item: { kind: 'Building', building: 'Granary' } }]);
    });
    await endPlanning(page);
    await expect(page.getByTestId('turn-number')).toHaveText('3');
    await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().set({ reportOpen: true }));
    const report = page.getByRole('complementary', { name: 'Turn report' });
    await expect.poll(async () => {
      // A moment card holds the report back; L opens it (the card has no Turn report button).
      if (await page.getByTestId('moment-lane').locator('.moment-card').isVisible()) await page.keyboard.press('l');
      return report.isVisible();
    }).toBe(true);
    const line = report.locator('.rline', { hasText: /bought Granary/ });
    await expect(line).toBeVisible();
    if (!before) {
      await expect(line.locator('.amt[data-amount="Gold"]')).toHaveCount(1);
      expect(await bareAmounts(page, '.rline')).toEqual([]);
    }
    await shoot(page, 'report-line', size, line);
    await shoot(page, 'report', size, report);
  });
});
