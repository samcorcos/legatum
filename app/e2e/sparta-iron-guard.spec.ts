import { test, expect } from './fixtures';
import { coreFixture } from '../src/test/coreFixture';
import { promotionFixture } from '../src/test/promotionFixture';
import { thresholdsFor } from '../src/promotion';

const setup = promotionFixture();
setup.fixture.players[0].faction = 'Sparta';
setup.fixture.units.find((unit) => unit.id === setup.trooper)!.xp =
  thresholdsFor('Sparta')[5];

test('Spartan Veteran shows its crest, six-part insignia and rank at desktop sizes', async ({ page }, testInfo) => {
  await page.addInitScript((game) => {
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({
      format: 'legatum-save', version: 1, client: 'wasm', turn: 2,
      names: ['Explorer'], savedAt: new Date().toISOString(), game,
    }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, coreFixture(setup.fixture));
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('2');

  for (const size of [{ width: 1000, height: 600 }, { width: 1280, height: 800 }, { width: 1440, height: 900 }]) {
    await page.setViewportSize(size);
    await page.evaluate((id) => {
      const game = (window as typeof window & { legatum: { getState: () => {
        select: (value: { kind: 'unit'; id: number }) => void;
        jumpTo: (hex: { q: number; r: number }) => void;
        views: { units: { id: number; pos: { q: number; r: number } }[] }[];
        seat: number;
      } } }).legatum.getState();
      game.select({ kind: 'unit', id });
      game.jumpTo(game.views[game.seat].units.find((unit) => unit.id === id)!.pos);
    }, setup.trooper);
    const rank = page.locator('.unit-panel .up-rank');
    await expect(rank).toContainText('Spartan Veteran');
    await expect(rank).toContainText('Level 6 · +60%');
    await expect(rank.locator('.rank-insignia')).toHaveAttribute('aria-label', 'Spartan Veteran, level 6');
    await expect(rank.locator('.promo-pips')).toHaveCount(0);
    await expect(rank.locator('.rank-insignia g')).toHaveCount(6);
    await expect(rank.locator('.rank-insignia path[d="M9 21 L12 16 L15 21"]')).toHaveCount(2);
    const box = await rank.boundingBox();
    expect(box).not.toBeNull();
    expect(box!.x + box!.width).toBeLessThanOrEqual(size.width);
    await page.screenshot({ path: testInfo.outputPath(`iron-guard-${size.width}x${size.height}.png`), animations: 'disabled' });
  }
});
