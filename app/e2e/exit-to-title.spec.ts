// Reproduction and regression test for playtest 6.3: "Exit to title" in the in-game menu did
// nothing. Root cause: GameMenu's onBlur handler trusted FocusEvent.relatedTarget to tell whether
// focus left the menu. WebKit (Safari, and the Mac app's WKWebView at the time) fires blur with relatedTarget null when a
// mousedown moves focus from the already-focused first menu item ("Save to file...") to another
// button in the same menu (e.g. "Exit to title"), before the new element takes focus. The handler
// then treated that as focus leaving the menu and closed it between mousedown and mouseup, so the
// browser never saw a matching mouseup on the same element and no click fired. This only shows up
// on WebKit (Chromium gives a real relatedTarget), which is why it matched what the user saw in
// WebKit but not in a quick Chromium check.
//
// Playtest 8.4 repeat: the first fix (check document.activeElement after requestAnimationFrame)
// still failed in WebKit, because Safari/WKWebView never focuses a <button> on click: mousedown
// moves focus to <body>, so activeElement is outside the menu too. The Safari-click tests below
// reproduce that on any engine (including Chromium and the Linux WebKit container, which do focus
// buttons on click) by cancelling mousedown's default focus change and blurring the focused element.
import { test, expect } from './fixtures';

async function openGameMenuAndExit(page: import('@playwright/test').Page) {
  await page.getByRole('button', { name: 'Menu', exact: true }).click();
  await page.getByRole('button', { name: 'Exit to title', exact: true }).click();
}

test('normal game: Exit to title returns to the title screen', async ({ page }) => {
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Single Player/ }).click();
  await page.getByRole('button', { name: /^Start game/ }).click();
  await expect(page.getByTestId('turn-number')).toBeVisible();

  await openGameMenuAndExit(page);
  await expect(page.getByRole('heading', { name: 'Legatum', exact: true })).toBeVisible();
  // Exiting keeps the autosave: Resume shows the game was kept, not lost.
  await expect(page.getByRole('button', { name: /^Resume/ })).toBeVisible();
});

test('scenario: Exit to title returns to the title screen', async ({ page }) => {
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: 'Scenarios', exact: true }).click();
  await page.getByRole('option', { name: /Nicaea Strike/ }).click();
  await page.getByRole('button', { name: /^Begin Sparta's Nicaea Strike/ }).click();
  await expect(page.getByTestId('scenario-bar')).toBeVisible();

  await openGameMenuAndExit(page);
  await expect(page.getByRole('heading', { name: 'Legatum', exact: true })).toBeVisible();
});

// Safari/WKWebView: pressing a <button> does not focus it. Mousedown blurs the focused element
// (relatedTarget null, activeElement = body) and focus never lands on the button.
async function behaveLikeSafari(page: import('@playwright/test').Page) {
  await page.evaluate(() => {
    document.addEventListener('mousedown', (e) => {
      if (!(e.target as Element | null)?.closest('button')) return;
      e.preventDefault(); // Chromium and Linux WebKit would focus the button; Safari does not
      (document.activeElement as HTMLElement | null)?.blur();
    }, true);
  });
}

async function pressSlowly(page: import('@playwright/test').Page, name: string) {
  const button = page.getByRole('button', { name, exact: true });
  await button.hover();
  await page.mouse.down();
  // Let focus, rAF and React settle between mousedown and mouseup, as a human click does.
  await page.waitForTimeout(150);
  await page.mouse.up();
}

test('Safari click (no focus on mousedown): Exit to title still fires', async ({ page }) => {
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Single Player/ }).click();
  await page.getByRole('button', { name: /^Start game/ }).click();
  await expect(page.getByTestId('turn-number')).toBeVisible();
  await behaveLikeSafari(page);

  await page.getByRole('button', { name: 'Menu', exact: true }).click();
  await expect(page.locator('.menu-pop')).toBeVisible();
  await pressSlowly(page, 'Exit to title');
  await expect(page.getByRole('heading', { name: 'Legatum', exact: true })).toBeVisible();
});

test('Safari click: pressing a menu button keeps the menu open until the click', async ({ page }) => {
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Single Player/ }).click();
  await page.getByRole('button', { name: /^Start game/ }).click();
  await expect(page.getByTestId('turn-number')).toBeVisible();
  await behaveLikeSafari(page);

  await page.getByRole('button', { name: 'Menu', exact: true }).click();
  const menu = page.locator('.menu-pop');
  await expect(menu).toBeVisible();
  await page.getByRole('button', { name: 'Exit to title', exact: true }).hover();
  await page.mouse.down();
  await page.waitForTimeout(150);
  await expect(menu).toBeVisible();
  await page.mouse.up();
});

test('menu still closes on Escape, an outside press, and keyboard focus leaving it', async ({ page }) => {
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Single Player/ }).click();
  await page.getByRole('button', { name: /^Start game/ }).click();
  await expect(page.getByTestId('turn-number')).toBeVisible();
  const menu = page.locator('.menu-pop');
  const open = () => page.getByRole('button', { name: 'Menu', exact: true }).click();

  await open();
  await expect(menu).toBeVisible();
  await page.keyboard.press('Escape');
  await expect(menu).toBeHidden();

  await open();
  await expect(menu).toBeVisible();
  await page.mouse.click(5, 5);
  await expect(menu).toBeHidden();

  // Keyboard focus moving to a real element outside the menu (blur with a relatedTarget).
  await open();
  await expect(menu).toBeVisible();
  await page.evaluate(() => {
    const outside = document.createElement('button');
    outside.id = 'outside-focus-target';
    document.body.appendChild(outside);
    outside.focus();
  });
  await expect(menu).toBeHidden();
});
