// Screen tour, title and lobby. One of the domain slices of the old e2e/screen-tour.spec.ts; run it alone with
// `npx playwright test e2e/tour/title-lobby`. Shots land in test-results/screen-tour/<size>/title-lobby/.
import { test, expect } from '../fixtures';
import { SIZES, shot } from './helpers';

// Each test starts from its own save, so tests can run on separate workers.
test.describe.configure({ mode: 'parallel' });

for (const size of SIZES) test.describe(`screen tour ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });


  test('title, settings, lobby, faction picker and guide', async ({ page }) => {
    test.setTimeout(120_000); // A long tour: it timed out at 45 s with 4 workers (UI review, 2026-11-25).
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
    // The one level applies to every AI seat.
    const levels = page.getByRole('radiogroup', { name: 'AI level', exact: true });
    await expect(levels.getByRole('radio')).toHaveCount(3);
    await levels.scrollIntoViewIfNeeded();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '03d-lobby-levels');
    await page.getByRole('table', { name: 'Seats' }).scrollIntoViewIfNeeded();
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
    await page.getByRole('button', { name: 'Menu', exact: true }).click();
    await shot(page, size, '05e-game-menu');
    await page.getByRole('button', { name: 'Settings', exact: true }).click();
    await expect(page.getByRole('dialog', { name: 'Settings', exact: true })).toBeVisible();
    await shot(page, size, '05f-game-settings');
    await page.getByRole('button', { name: 'Close settings' }).click();
  });


});
