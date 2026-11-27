import { test, expect } from './fixtures';
import { setMapSeed } from './mapSeed';

// Game panels never highlight as text. A modal dialog sits in the browser's top layer, where
// Chrome does not carry the shell's user-select: none down, so shift+click, a drag or
// select-all used to paint the whole research panel as a selection.
test('shift+click, drag and select-all in the research panel select no text', async ({ page }) => {
  test.setTimeout(120_000);
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Single Player/ }).click();
  await setMapSeed(page, '424242');
  await page.getByRole('button', { name: /^Start game/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('1', { timeout: 60_000 });
  const dialog = page.locator('dialog.tech-modal');
  await page.locator('.topbar .research-chip').click();
  await expect(dialog).toBeVisible();

  const selected = () => page.evaluate(() => getSelection()?.toString() ?? '');
  const title = (await dialog.locator('h2').first().boundingBox())!;
  const far = (await dialog.boundingBox())!;
  const end = { x: far.x + far.width - 40, y: far.y + far.height - 40 };

  await page.mouse.click(title.x + 5, title.y + 5);
  await page.keyboard.down('Shift');
  await page.mouse.click(end.x, end.y);
  await page.keyboard.up('Shift');
  expect(await selected()).toBe('');

  await page.mouse.move(title.x + 2, title.y + 5);
  await page.mouse.down();
  await page.mouse.move(end.x, end.y, { steps: 8 });
  await page.mouse.up();
  expect(await selected()).toBe('');

  await page.mouse.dblclick(title.x + 10, title.y + 10);
  expect(await selected()).toBe('');

  await page.keyboard.press('ControlOrMeta+a');
  expect(await selected()).toBe('');
});
