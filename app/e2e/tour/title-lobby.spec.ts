// Screen tour, title, lobby, hot seat and scenarios. One of the domain slices of the old e2e/screen-tour.spec.ts; run it alone with
// `npx playwright test e2e/tour/title-lobby`. Shots land in test-results/screen-tour/<size>/title-lobby/.
import { test, expect } from '../fixtures';
import { endPlanning } from '../turnActions';
import { SIZES, shot } from './helpers';

// Each test starts from its own save, so tests can run on separate workers.
test.describe.configure({ mode: 'parallel' });

for (const size of SIZES) test.describe(`screen tour ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });


  test('title, settings, lobby, faction picker and guide', async ({ page }) => {
    await page.goto('/?client=wasm');
    await expect(page.getByRole('heading', { name: 'Legatum', exact: true })).toBeVisible();
    await shot(page, size, '01-title');

    await page.getByRole('button', { name: 'Settings', exact: true }).click();
    await expect(page.getByRole('button', { name: 'Close settings', exact: true })).toBeVisible();
    await shot(page, size, '02-title-settings');
    await page.getByRole('button', { name: 'Close settings', exact: true }).click();

    await page.getByRole('button', { name: /^How to Play/ }).click();
    await expect(page.getByRole('dialog', { name: 'How to Play', exact: true })).toBeVisible();
    await shot(page, size, '15-guide');
    // The chapters that carry the hold-to-win rule and every faction goal.
    const guide = page.getByRole('dialog', { name: 'How to Play', exact: true });
    await guide.getByRole('button', { name: /The victory race/ }).click();
    await page.mouse.move(5, 5);
    await shot(page, size, '15b-guide-victory-race');
    await guide.getByRole('button', { name: /The peoples/ }).click();
    await page.mouse.move(5, 5);
    await shot(page, size, '15c-guide-eight-heirs');
    await page.keyboard.press('Escape');

    await page.getByRole('button', { name: /^Single Player/ }).click();
    await expect(page.getByRole('heading', { name: 'Single Player', exact: true })).toBeVisible();
    await shot(page, size, '03-lobby');
    // The all-AI control offers Easy, Normal and Hard only (Training is a research tool, 2026-11-21),
    // then the seat list with the level name closed in every seat (nothing clipped).
    const levels = page.getByRole('radiogroup', { name: 'AI level (all AI empires)', exact: true });
    await expect(levels.getByRole('radio')).toHaveCount(3);
    await levels.scrollIntoViewIfNeeded();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '03d-lobby-levels');
    await page.getByRole('combobox', { name: 'Seat 2 AI level', exact: true }).scrollIntoViewIfNeeded();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '03e-lobby-seat-list');
    await page.getByRole('button', { name: /^Seat 1 faction:/ }).click();
    // Every seat has its own (closed) picker dialog in the DOM; scope to the one that's open.
    const stage = page.locator('dialog[data-testid="faction-stage"][open]');
    await expect(stage).toBeVisible();
    // The picker now changes the seat as you click; remember the default so the tour can put it back.
    const defaultFaction = (await stage.locator('[role=option][aria-selected=true]').getAttribute('data-faction'))!;
    await shot(page, size, '04-faction-picker');
    // A selected faction's full sheet: lore, pips, strengths paired with their price, the unique
    // piece, and how you win/lose. Clicking an entry chooses it; the dialog stays open.
    await page.getByRole('option', { name: /Sparta/ }).click();
    await expect(stage.getByTestId('faction-sheet')).toContainText('Sparta');
    await shot(page, size, '04b-faction-sheet');
    // Every gallery option's sheet, Random included (premium bar: all of them must reach the reference).
    // Clicking a tile chooses it and shows its sheet; the dialog stays open until Close.
    const names = await stage.locator('.fs-tile-name').allTextContents();
    for (const [i, label] of names.entries()) {
      const id = (await stage.getByRole('option').nth(i).getAttribute('data-faction'))!.toLowerCase();
      await stage.getByRole('option').nth(i).click();
      await expect(stage.locator('.fs-sheet-name')).toContainText(label.trim());
      await page.mouse.move(5, size.height / 2);
      await shot(page, size, `04c-sheet-${String(i + 1).padStart(2, '0')}-${id}`);
    }
    await stage.locator(`[role=option][data-faction="${defaultFaction}"]`).click();
    await page.keyboard.press('Escape');
    await expect(stage).toHaveCount(0);
    // Map size snaps the empire count: Small trims to 4, Large fills all 8 seat rows.
    const mapSize = page.getByRole('radiogroup', { name: 'Map size', exact: true });
    await mapSize.getByRole('radio', { name: /^Small/ }).click();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '03f-lobby-small-snapped');
    await mapSize.getByRole('radio', { name: /^Large/ }).click();
    await page.locator('.seat-row:not(.head)').last().scrollIntoViewIfNeeded();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '03g-lobby-large-eight-seats');
  });

  test('hot seat: two humans on turn one', async ({ page }) => {
    await page.addInitScript(() => {
      if (location.protocol.startsWith('http')) localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
    });
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^Multiplayer/ }).click();
    await page.getByRole('button', { name: /^Hot Seat/ }).click();
    await expect(page.getByRole('heading', { name: 'Hot Seat', exact: true })).toBeVisible();
    await page.getByRole('radiogroup', { name: 'Map size', exact: true }).getByRole('radio', { name: /^Small/ }).click();
    await page.getByLabel('Map seed', { exact: true }).fill('9001');
    await expect(page.getByLabel('Seat 2 type', { exact: true })).toHaveValue('human');
    await page.getByRole('button', { name: /^Start game/ }).click();
    await expect(page.getByTestId('turn-number')).toHaveText('1', { timeout: 60_000 });
    await expect(page.locator('.loading-screen')).toHaveCount(0);
    await shot(page, size, '05b-hotseat-first-turn');
  });

  test('six empires: the default single-player race on turn one', async ({ page }) => {
    await page.addInitScript(() => {
      if (location.protocol.startsWith('http')) localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
    });
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^Single Player/ }).click();
    await page.getByLabel('Map seed', { exact: true }).fill('424242');
    await page.getByRole('button', { name: /^Start game/ }).click();
    await expect(page.getByTestId('turn-number')).toHaveText('1', { timeout: 60_000 });
    await expect(page.locator('.loading-screen')).toHaveCount(0);
    await shot(page, size, '05d-six-empires-first-turn');
  });

  test('scenarios: list, briefing, objective bar, result and feedback', async ({ page }) => {
    await page.addInitScript(() => {
      if (!location.protocol.startsWith('http')) return;
      (globalThis as { legatumScenarioOverrides?: unknown }).legatumScenarioOverrides = { 'war-sparta-nicaea': { objective: { turns: 1 } } };
      localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
    });
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: 'Scenarios', exact: true }).click();
    await expect(page.getByRole('heading', { name: 'Scenario Lab', exact: true })).toBeVisible();
    await page.mouse.move(2, 2);
    await shot(page, size, '32-scenarios');
    for (const [slug, title] of [['celts-reserve', 'The Unwalled Reserve'], ['india-niter', 'An Army Without Niter']] as const) {
      await page.goto('/?client=wasm');
      await page.getByRole('button', { name: 'Scenarios', exact: true }).click();
      await expect(page.getByRole('heading', { name: 'Scenario Lab', exact: true })).toBeVisible();
      const option = page.getByRole('option', { name: new RegExp(title) });
      await option.scrollIntoViewIfNeeded();
      await page.mouse.move(2, 2);
      await shot(page, size, `32b-scenario-card-${slug}`);
      await option.click();
      await expect(page.getByTestId('lab-briefing')).toBeVisible();
      await page.mouse.move(2, 2);
      await shot(page, size, `32c-scenario-briefing-${slug}`);
    }
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: 'Scenarios', exact: true }).click();
    await expect(page.getByRole('heading', { name: 'Scenario Lab', exact: true })).toBeVisible();
    await page.getByRole('option', { name: /Nicaea Strike/ }).click();
    await expect(page.getByTestId('lab-briefing')).toBeVisible();
    await page.mouse.move(2, 2);
    await shot(page, size, '33-scenario-briefing');
    await page.getByRole('button', { name: /^Begin Sparta's Nicaea Strike/ }).click();
    await expect(page.getByTestId('scenario-bar')).toBeVisible();
    await expect(page.locator('.loading-screen')).toHaveCount(0);
    await shot(page, size, '34-scenario-in-play');
    // The objective's city is ringed on the map; Show on map centres on it.
    await page.getByTestId('scenario-show-target').click();
    await page.waitForTimeout(500);
    await shot(page, size, '34b-scenario-target-on-map');
    await page.getByTestId('scenario-bar').locator('.sb-main').click();
    await expect(page.locator('#scenario-details')).toBeVisible();
    await shot(page, size, '35-scenario-objective-details');
    await page.getByTestId('scenario-bar').locator('.sb-main').click();
    await endPlanning(page);
    await expect(page.getByTestId('scenario-result')).toBeVisible();
    await expect(page.getByTestId('scenario-saved')).not.toHaveText('Saving your run…');
    await page.mouse.move(2, 2);
    await shot(page, size, '36-scenario-result');
    await page.getByTestId('scenario-feel').click();
    await expect(page.getByTestId('scenario-feedback')).toBeVisible();
    await page.mouse.move(2, 2);
    await shot(page, size, '36b-scenario-feedback');
    const feedback = page.getByTestId('scenario-feedback');
    await feedback.getByRole('radiogroup', { name: 'Fun' }).getByRole('radio', { name: '4' }).click();
    await feedback.getByRole('radiogroup', { name: 'Fair' }).getByRole('radio', { name: '3' }).click();
    await feedback.getByRole('radio', { name: 'Too easy' }).click();
    await feedback.locator('textarea').fill('Four Hoplites at the gate felt too easy.');
    await page.mouse.move(2, 2);
    await shot(page, size, '36c-scenario-feedback-filled');
    await page.getByTestId('scenario-save-feedback').click();
    await expect(page.getByRole('heading', { name: 'Thank you' })).toBeVisible();
    await expect(page.getByTestId('scenario-saved')).not.toHaveText('Saving your run…');
    await page.mouse.move(2, 2);
    await shot(page, size, '36d-scenario-thanks');
    // Back on the list the card shows your best result.
    await page.getByRole('button', { name: /^Back to scenarios/ }).click();
    await page.mouse.move(2, 2);
    await shot(page, size, '36e-scenarios-after-play');
  });

  test('scenarios: the objective is marked on the map from turn 1, even in fog', async ({ page }) => {
    await page.addInitScript(() => {
      if (!location.protocol.startsWith('http')) return;
      localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
    });
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: 'Scenarios', exact: true }).click();
    await page.getByRole('option', { name: /Nicaea Strike/ }).click();
    await expect(page.getByTestId('scenario-briefing-target')).toContainText(/Take \w+: \d+ tiles? /);
    await shot(page, size, '37-scenario-briefing-target');
    await page.getByRole('button', { name: /^Begin Sparta's Nicaea Strike/ }).click();
    await expect(page.locator('.loading-screen')).toHaveCount(0);
    await expect(page.getByTestId('scenario-target')).toContainText('Take');
    await page.getByTestId('scenario-show-target').click();
    await page.waitForTimeout(500);
    await shot(page, size, '38-scenario-target-in-fog');
  });
});
