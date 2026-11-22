import type { Page } from '@playwright/test';
import type { useGame } from '../src/store';
import { test, expect } from './fixtures';
import { turnFlowFixture } from '../src/test/turnFlowFixture';
import { coreFixture } from '../src/test/coreFixture';

type GameWindow = Window & { legatum: typeof useGame };

// A military unit's idle orders: Fortify, Heal (a damaged unit) and Skip this turn, at the user's two window sizes.
const SIZES = [{ width: 1000, height: 600 }, { width: 1440, height: 900 }];

async function start(page: Page) {
  const fixture = turnFlowFixture();
  fixture.players[1].isHuman = false;
  fixture.setup.players[1].is_human = false;
  await page.addInitScript((game) => {
    if (!location.protocol.startsWith('http')) return;
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({
      format: 'legatum-save', version: 1, client: 'wasm', turn: 2,
      names: ['Explorer'], savedAt: new Date().toISOString(), game,
    }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, coreFixture(fixture));
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('2');
  await page.locator('[data-game-map]').focus();
}

for (const size of SIZES) {
  test(`the unit card has Fortify, Heal and Skip this turn, at ${size.width}x${size.height}`, async ({ page }, testInfo) => {
    await page.setViewportSize(size);
    await start(page);
    await expect(page.locator('.unit-panel .up-kind')).toHaveText('Spearman');
    const buttons = page.locator('.unit-panel .actions button.act');
    const names = (await buttons.allTextContents()).map((t) => t.replace(/\s+/g, ' ').trim());
    expect(names.some((n) => /^Fortify/.test(n))).toBe(true);
    expect(names.some((n) => /^Skip this turn/.test(n))).toBe(true);
    // The Spearman is at 75 HP, so Heal is offered beside Fortify; Wake and Hold are gone.
    expect(names.some((n) => /^Heal/.test(n))).toBe(true);
    expect(names.some((n) => /^Wake/.test(n))).toBe(false);
    // Movement leads the side panel; both idle buttons remain on screen without scrolling.
    await expect(page.locator('.unit-panel .actions .act-wrap.primary')).toHaveCount(1);
    await expect(page.locator('.unit-panel .actions .act-wrap.primary')).toContainText('Move or attack');
    for (const label of [/^Fortify/, /^Heal/, /^Skip this turn/]) {
      const box = await page.locator('.unit-panel button.act', { hasText: label }).boundingBox();
      expect(box).not.toBeNull();
      expect(box!.y + box!.height).toBeLessThanOrEqual(size.height);
    }
    // The row stays whole: no label is clipped or wraps out of its button.
    for (const button of await page.locator('.unit-panel .act-pair button.act').all()) {
      expect(await button.evaluate((el) => el.scrollWidth <= el.clientWidth + 1)).toBe(true);
    }
    await page.mouse.move(5, size.height - 5);
    await page.screenshot({ path: testInfo.outputPath(`idle-orders-spearman-${size.width}x${size.height}.png`), animations: 'disabled' });

    // H heals: the order says "until healed", the card says it is healing, and the unit stays out of the Return queue.
    await page.keyboard.press('h');
    await expect.poll(() => page.evaluate(() => {
      const s = (window as unknown as GameWindow).legatum.getState();
      return s.orders[s.seat].filter((o) => o.type === 'SleepUnit' && o.until_healed).length;
    })).toBe(1);
    await page.evaluate(() => {
      const s = (window as unknown as GameWindow).legatum.getState();
      const o = s.orders[s.seat].find((o) => o.type === 'SleepUnit')!;
      s.select({ kind: 'unit', id: (o as { unit: number }).unit });
    });
    await expect(page.locator('.unit-panel .up-status')).toContainText('Healing until full');
    await expect(page.locator('.unit-panel button.act', { hasText: /^Heal/ })).toHaveAttribute('aria-pressed', 'true');
    await page.mouse.move(5, size.height - 5);
    await page.screenshot({ path: testInfo.outputPath(`idle-orders-healing-${size.width}x${size.height}.png`), animations: 'disabled' });
    // Clear it and select the Spearman again for the Fortify check below.
    await page.evaluate(() => {
      const s = (window as unknown as GameWindow).legatum.getState();
      const o = s.orders[s.seat].find((o) => o.type === 'SleepUnit')!;
      s.editOrders((os) => os.filter((x) => x !== o));
      s.select({ kind: 'unit', id: (o as { unit: number }).unit });
    });
    await page.locator('[data-game-map]').focus();

    // F fortifies the Spearman (and steps to the next unit); selecting it again shows the status.
    await page.keyboard.press('f');
    await expect.poll(() => page.evaluate(() => {
      const s = (window as unknown as GameWindow).legatum.getState();
      return s.orders[s.seat].filter((o) => o.type === 'SleepUnit').length;
    })).toBe(1);
    await page.evaluate(() => {
      const s = (window as unknown as GameWindow).legatum.getState();
      const o = s.orders[s.seat].find((o) => o.type === 'SleepUnit')!;
      s.select({ kind: 'unit', id: (o as { unit: number }).unit });
    });
    await expect(page.locator('.unit-panel .up-status')).toContainText('Fortified until woken');
    await expect(page.locator('.unit-panel button.act', { hasText: /^Fortify/ })).toHaveAttribute('aria-pressed', 'true');
    await page.mouse.move(5, size.height - 5);
    await page.screenshot({ path: testInfo.outputPath(`idle-orders-fortifying-${size.width}x${size.height}.png`), animations: 'disabled' });
  });
}
