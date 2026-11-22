// Greece Captive: the Athens sheet, a rival's Athenian city card and the free-tech report.
import { test, expect } from '../fixtures';
import { coreFixture } from '../../src/test/coreFixture';
import { researchFixture } from '../../src/test/researchFixture';
import { contextFromViews, formatEvents } from '../../src/format';
import { FACTION_INFO } from '../../src/data';
import { SIZES, loadFixture, selectThing, shot, type GameWindow } from './helpers';

for (const size of SIZES) test.describe(`Greece Captive ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  test('Athens sheet states the drawback', async ({ page }) => {
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^Single Player/ }).click();
    await page.getByRole('button', { name: /^Seat 1 faction:/ }).click();
    const stage = page.locator('dialog[data-testid="faction-stage"][open]');
    await page.getByRole('option', { name: FACTION_INFO.Athens.name }).click();
    const sheet = stage.getByTestId('faction-sheet');
    await expect(sheet).toContainText('A rival that captures one of your cities gets one free technology');
    await expect(sheet).toContainText('Capture an Athenian city for one free technology');
    await page.mouse.move(size.width - 2, size.height - 2);
    await shot(page, size, '01-athens-sheet');
  });

  test('rival city card and the free-technology turn report', async ({ page }) => {
    const fixture = researchFixture('Sparta');
    fixture.players[1].faction = 'Athens';
    fixture.setup.players[1].faction = 'Athens';
    const city = fixture.cities.find((c) => c.owner === 1)!;
    await loadFixture(page, coreFixture(fixture), fixture.turn);
    await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().set({ techOpen: false }));
    await selectThing(page, 'city', city.id);
    const card = page.getByTestId('foreign-city-actions');
    await expect(card).toContainText('Capture it: +1 free technology');
    await page.mouse.move(size.width - 2, size.height - 2);
    await shot(page, size, '02-rival-city-card');

    const view = await page.evaluate(() => {
      const s = (window as unknown as GameWindow).legatum.getState();
      return s.views![s.seat];
    });
    const events = [{ type: 'AthensCityCaptured' as const, city: city.id, from: 1, to: 0 }];
    const report = { turn: view.turn, hash: '', events: [events], moments: [[]], alarms: [[]], lines: [formatEvents(events, contextFromViews(view, view))] };
    await page.evaluate((report) => (window as unknown as GameWindow).legatum.getState().set({ report, alarmSeen: [], reportOpen: true }), report);
    const opened = page.getByRole('complementary', { name: 'Turn report' });
    await expect(opened).toContainText(`Captured ${city.name} from Athens: you gain one free technology`);
    await shot(page, size, '03-free-tech-turn-report');
  });
});
