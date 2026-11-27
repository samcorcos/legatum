import { test, expect } from '../fixtures';
import { coreFixture } from '../../src/test/coreFixture';
import { citiesFixture } from '../../src/test/citiesFixture';
import { SIZES, selectThing, shot } from './helpers';

for (const size of SIZES) test.describe(`advisor feature discovery ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });
  test('established city automation is explained and its real toggle is highlighted', async ({ page }) => {
    test.setTimeout(90_000);
    const state = citiesFixture();
    state.turn = 80;
    const city = state.cities.find((c) => c.owner === 0 && c.population >= 4)!;
    const game = coreFixture(state);
    await page.addInitScript(({ game }) => {
      if (!location.protocol.startsWith('http')) return;
      localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn: 80, names: ['Ada'], savedAt: new Date().toISOString(), game }));
      localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: true, seen: ['welcome', 'report', 'howToWin'], practice: {}, tips: {} }));
    }, { game });
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^Continue/ }).click();
    await expect(page.getByTestId('turn-number')).toHaveText('80', { timeout: 60_000 });
    await selectThing(page, 'city', city.id);
    const card = page.locator('[data-testid="advisor"][data-advisor-id="cityAutomation"]');
    const toggle = page.getByRole('button', { name: `Automate ${city.name}`, exact: true });
    await expect(card).toBeVisible();
    await expect(card).toContainText('tile improvements');
    await expect(card).toContainText('Click Automate');
    await expect(page.getByTestId('advisor-ring')).toBeVisible();
    await shot(page, size, '01-automate-lesson');
    const cardBox = await card.boundingBox(), buttonBox = await toggle.boundingBox();
    expect(cardBox).toBeTruthy(); expect(buttonBox).toBeTruthy();
    if (cardBox && buttonBox) expect(cardBox.x < buttonBox.x + buttonBox.width && cardBox.x + cardBox.width > buttonBox.x
      && cardBox.y < buttonBox.y + buttonBox.height && cardBox.y + cardBox.height > buttonBox.y, 'advisor does not cover Automate').toBe(false);
    await toggle.click();
    await expect(toggle).toHaveAttribute('aria-pressed', 'true');
    await expect(card).toHaveCount(0);
    await toggle.click();
    await expect(toggle).toHaveAttribute('aria-pressed', 'false');
    await expect(card).toHaveCount(0);
    await shot(page, size, '02-learned-once');
  });
});
