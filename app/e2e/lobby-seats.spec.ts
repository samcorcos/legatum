// Choosing a map size snaps the number of empires into that size's recommended range; a new
// Single Player lobby starts at Standard's default of 6 (you and 5 AIs).
import { test, expect } from './fixtures';

test('lobby: size changes reset untouched seats to 4, 6 or 8 empires, and the warning clears', async ({ page }) => {
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Single Player/ }).click();
  await expect(page.getByRole('heading', { name: 'Single Player', exact: true })).toBeVisible();

  const rows = page.getByRole('table', { name: 'Seats' }).locator('.seat-row:not(.head)');
  const size = page.getByRole('radiogroup', { name: 'Map size', exact: true });
  await expect(rows).toHaveCount(6);
  await expect(page.getByText('6 of 8 seats')).toBeVisible();

  await size.getByRole('radio', { name: /^Small/ }).click();
  await expect(rows).toHaveCount(4);
  await expect(page.getByText('4 of 8 seats')).toBeVisible();
  await expect(page.locator('.lobby-note', { hasText: 'maps suit' })).toHaveCount(0);

  // Below Standard's minimum of 5, the count goes to its default (6), not to the minimum.
  await size.getByRole('radio', { name: /^Standard/ }).click();
  await expect(rows).toHaveCount(6);
  await expect(page.locator('.lobby-note', { hasText: 'maps suit' })).toHaveCount(0);

  await size.getByRole('radio', { name: /^Large/ }).click();
  await expect(rows).toHaveCount(8);

  // All AI seats are untouched, so Standard returns to its default even though 8 fits.
  await size.getByRole('radio', { name: /^Standard/ }).click();
  await expect(rows).toHaveCount(6);

  // A manual removal is reversible, and the Add button can still reach eight.
  await page.getByLabel('Remove seat 6', { exact: true }).click();
  await expect(rows).toHaveCount(5);
  await page.getByRole('button', { name: '+ Add an AI empire' }).click();
  await expect(rows).toHaveCount(6);
  await page.getByRole('button', { name: '+ Add an AI empire' }).click();
  await page.getByRole('button', { name: '+ Add an AI empire' }).click();
  await expect(rows).toHaveCount(8);
  await expect(page.getByRole('button', { name: '+ Add an AI empire' })).toHaveCount(0);

  // Going outside the range afterwards is still the player's call, and the warning says so.
  await size.getByRole('radio', { name: /^Small/ }).click();
  await expect(rows).toHaveCount(4);
  await page.getByRole('button', { name: '+ Add an AI empire' }).click();
  await expect(rows).toHaveCount(5);
  await expect(page.locator('.lobby-note', { hasText: 'Small maps suit 2-4 empires; you have 5' })).toBeVisible();
});

for (const [width, height] of [[1000, 600], [1440, 900]] as const) {
  test(`lobby: the all-AI level segments read in full with an 8 px gutter at ${width}x${height}`, async ({ page }) => {
    await page.setViewportSize({ width, height });
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^Single Player/ }).click();
    const group = page.getByRole('radiogroup', { name: 'AI level (all AI empires)', exact: true });
    await expect(group.getByRole('radio', { name: /^Hard/ })).toBeVisible();
    const gutters = await group.getByRole('radio').evaluateAll((buttons) =>
      buttons.map((b) => {
        const label = b.querySelector('span')!;
        const r = document.createRange();
        r.selectNodeContents(label);
        const text = r.getBoundingClientRect();
        const box = b.getBoundingClientRect();
        return { label: label.textContent, left: text.left - box.left, right: box.right - text.right, clipped: label.scrollWidth > label.clientWidth };
      }));
    expect(gutters.map((g) => g.label)).toEqual(['Easy', 'Normal', 'Hard']);
    for (const g of gutters) {
      expect(g.left, g.label ?? '').toBeGreaterThanOrEqual(8);
      expect(g.right, g.label ?? '').toBeGreaterThanOrEqual(8);
      expect(g.clipped, g.label ?? '').toBe(false);
    }
  });
}

for (const [width, height] of [[1000, 600], [1440, 900]] as const) {
  test(`faction sheet review shots at ${width}x${height}`, async ({ page }) => {
    await page.setViewportSize({ width, height });
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^Single Player/ }).click();
    const stage = page.locator('dialog[data-testid="faction-stage"][open]');
    const seat = page.getByRole('button', { name: /^Seat 1 faction:/ });
    await seat.click();
    await expect(stage).toBeVisible();
    for (const faction of ['Athens', 'Mongols']) {
      await stage.getByRole('option', { name: new RegExp(faction) }).click();
      await page.mouse.move(5, 5);
      await page.screenshot({ path: `test-results/faction-sheet-${faction.toLowerCase()}-${width}x${height}.png`, animations: 'disabled' });
    }
  });
}

// Faction picker: the click is the selection. No hover preview, no confirm button, Close keeps the choice.
for (const [width, height] of [[1000, 600], [1440, 900]] as const) {
  test(`lobby: clicking a faction picks it live, hover never previews, and Close keeps the choice at ${width}x${height}`, async ({ page }) => {
    await page.setViewportSize({ width, height });
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^Single Player/ }).click();
    await expect(page.getByRole('heading', { name: 'Single Player', exact: true })).toBeVisible();

    const seat = page.getByRole('button', { name: /^Seat 1 faction:/ });
    const stage = page.locator('dialog[data-testid="faction-stage"][open]');
    const sheetName = stage.getByTestId('faction-sheet').locator('.fs-sheet-name');
    const tile = (name: RegExp) => stage.getByRole('option', { name });
    const open = async () => {
      await seat.click();
      await expect(stage).toBeVisible();
      await stage.evaluate((el) => Promise.all(el.getAnimations().map((a) => a.finished)));
    };

    // There is no confirm button, only Close.
    await open();
    await expect(stage.getByRole('button', { name: /^Play( as| Random)/ })).toHaveCount(0);
    const close = stage.getByRole('button', { name: 'Close faction picker' });
    await expect(close).toBeVisible();

    // Hovering never changes the sheet or the seat, however long the pointer rests.
    const before = await sheetName.innerText();
    await tile(/Mongols/).hover();
    await page.waitForTimeout(400);
    await expect(sheetName).toHaveText(before);
    await expect(seat).not.toHaveAccessibleName(/Mongols/);

    // A click is the selection: the sheet, the highlight and the seat row behind all change at once,
    // and the dialog stays open.
    await tile(/Mongols/).click();
    await expect(stage).toBeVisible();
    await expect(sheetName).toContainText('Mongols');
    await expect(tile(/Mongols/)).toHaveAttribute('aria-selected', 'true');
    await expect(stage.getByTestId('faction-chosen')).toContainText('Mongols');
    await expect(seat).toHaveAccessibleName(/Mongols/);

    // Clicking another entry toggles the choice, Random included.
    await tile(/Sparta/).click();
    await expect(sheetName).toContainText('Sparta');
    await expect(tile(/Mongols/)).toHaveAttribute('aria-selected', 'false');
    await expect(seat).toHaveAccessibleName(/Sparta/);
    await tile(/Random/).click();
    await expect(stage.getByTestId('faction-sheet')).toContainText('Could be any of these');
    await expect(seat).toHaveAccessibleName(/Random/);
    await tile(/Mongols/).click();

    // Close keeps the choice, and so does a reopen.
    await close.click();
    await expect(stage).toHaveCount(0);
    await expect(seat).toHaveAccessibleName(/Mongols/);
    await open();
    await expect(tile(/Mongols/)).toHaveAttribute('aria-selected', 'true');
    await expect(sheetName).toContainText('Mongols');

    // Escape closes too, with no revert, even after a change.
    await tile(/Byzantium/).click();
    await page.keyboard.press('Escape');
    await expect(stage).toHaveCount(0);
    await expect(seat).toHaveAccessibleName(/Byzantium/);

    // Backdrop click closes and keeps the choice.
    await open();
    await tile(/Mongols/).click();
    await page.mouse.click(3, 3);
    await expect(stage).toHaveCount(0);
    await expect(seat).toHaveAccessibleName(/Mongols/);

    // Tab stays inside the dialog (Close and the chosen tile are the only stops), and Return still closes.
    await open();
    await page.keyboard.press('Tab');
    expect(await page.evaluate(() => !!document.activeElement?.closest('dialog[open]'))).toBe(true);
    await page.keyboard.press('Tab');
    expect(await page.evaluate(() => !!document.activeElement?.closest('dialog[open]'))).toBe(true);
    await page.keyboard.press('Enter');
    await expect(stage).toHaveCount(0);
    await expect(seat).toHaveAccessibleName(/Mongols/);

    // Keyboard: arrows move the selection and pick; Return closes and keeps it.
    await open();
    await page.keyboard.press('ArrowDown');
    await expect(seat).not.toHaveAccessibleName(/Byzantium/);
    await expect(tile(/Byzantium/)).toHaveAttribute('aria-selected', 'false');
    const picked = await sheetName.innerText();
    await expect(seat).toHaveAccessibleName(new RegExp(picked.trim()));
    await page.keyboard.press('Enter');
    await expect(stage).toHaveCount(0);
    await expect(seat).toHaveAccessibleName(new RegExp(picked.trim()));
  });
}
