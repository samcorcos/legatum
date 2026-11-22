// Screen tour: a newcomer's first screens with the first-game hints ON, from an empty browser (nothing in storage).
// Every other tour turns the hints off, so this is the only place the hint panel is judged beside the real first screens.
// Run alone with `npx playwright test e2e/tour/first-run-hints`; shots land in test-results/screen-tour/<size>/first-run-hints/.
import { test, expect } from '../fixtures';
import { SIZES, shot, type GameWindow } from './helpers';

// One game per test; the two sizes run on their own workers.
test.describe.configure({ mode: 'parallel' });

for (const size of SIZES) test.describe(`first run with hints ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  test('title, a people, the first two turns, every hint shown', async ({ page }) => {
    // No addInitScript: storage starts empty, so the hints are on and none has been seen.
    await page.goto('/?client=wasm');
    await expect(page.getByRole('heading', { name: 'Legatum', exact: true })).toBeVisible();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '01-title');

    await page.getByRole('button', { name: /^Single Player/ }).click();
    await expect(page.getByRole('heading', { name: 'Single Player', exact: true })).toBeVisible();
    await page.getByLabel('Map seed', { exact: true }).fill('424242');
    await page.getByRole('button', { name: /^Seat 1 faction:/ }).click();
    const stage = page.locator('dialog[data-testid="faction-stage"][open]');
    await expect(stage).toBeVisible();
    await stage.getByRole('option', { name: /Athens/ }).click();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '02-pick-a-people');
    await page.keyboard.press('Escape');
    await expect(stage).toHaveCount(0);
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '03-lobby');
    await page.getByRole('button', { name: /^Start game/ }).click();
    await expect(page.getByTestId('turn-number')).toHaveText('1', { timeout: 60_000 });
    await expect(page.locator('.loading-screen')).toHaveCount(0);

    const hint = page.getByTestId('first-game-hint');
    const seen: string[] = [];
    // Shoots the hint on screen (if it is a new one) and checks it never covers the turn's primary button.
    const captureHint = async (label: string) => {
      await page.mouse.move(5, size.height / 2);
      if (!(await hint.isVisible())) return false;
      const title = (await hint.locator('h3').textContent())!.trim();
      if (seen.includes(title)) return false;
      seen.push(title);
      await shot(page, size, `${label}-hint-${title.toLowerCase().replace(/[^a-z]+/g, '-').replace(/^-|-$/g, '')}`);
      const cta = await page.locator('.seatbar .cta').first().boundingBox();
      const box = (await hint.boundingBox())!;
      if (cta) {
        const overlap = box.x < cta.x + cta.width && box.x + box.width > cta.x && box.y < cta.y + cta.height && box.y + box.height > cta.y;
        expect.soft(overlap, `the "${title}" hint must not cover the turn's primary button`).toBe(false);
      }
      return true;
    };

    // Turn 1, first screen: the capital has no production and no research is chosen.
    await expect(hint).toBeVisible();
    await captureHint('04-turn1-first');

    // Choose production in the capital: the first build card's Build button.
    await page.evaluate(() => {
      const s = (window as unknown as GameWindow).legatum.getState();
      const v = s.views![s.seat];
      const city = v.cities.find((c) => c.owner === v.me)!;
      s.select({ kind: 'city', id: city.id }, true);
      s.jumpTo(city.pos);
    });
    await expect(page.locator('.city-panel')).toBeVisible();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '05-turn1-city-panel-with-hint');
    await page.locator('.city-panel').getByRole('button', { name: /^(Build|Queue)\b/ }).first().click();
    await page.keyboard.press('Escape');
    await captureHint('06-turn1-after-production');

    // Research: the next hint, then pick a technology from the technologies panel.
    await page.keyboard.press('t');
    const tech = page.locator('dialog.tech-modal[open]');
    await expect(tech).toBeVisible();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '07-turn1-technologies');
    await tech.getByRole('button', { name: /^Research now/ }).first().click();
    if (await tech.isVisible()) await page.keyboard.press('Escape');
    await expect(tech).toHaveCount(0);
    await captureHint('08-turn1-after-research');
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '09-turn1-ready');

    // End the turn the way a newcomer would: through the turn dock.
    await page.locator('.seatbar').getByRole('button', { name: /^(Ready anyway|End turn|Ready\b)/ }).click();
    const confirm = page.getByRole('dialog', { name: 'End turn with decisions pending?' });
    if (await confirm.isVisible().catch(() => false)) {
      await page.mouse.move(5, size.height / 2);
      await shot(page, size, '10-turn1-end-turn-question');
      await confirm.getByRole('button', { name: 'End turn anyway', exact: true }).click();
    }
    await expect(page.getByTestId('turn-number')).toHaveText('2', { timeout: 60_000 });
    await expect(page.locator('.busy')).toHaveCount(0, { timeout: 60_000 });
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '11-turn2-first');
    await captureHint('12-turn2');
    expect(seen.length, 'at least the production hint is shown on the first screen').toBeGreaterThan(0);
  });
});
