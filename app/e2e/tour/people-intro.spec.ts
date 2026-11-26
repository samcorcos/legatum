// The opening of a new game: the people's introduction (celebrate/PeopleIntro.tsx) at both desktop sizes, for the
// people with the most text on it and the one with the least, then the first turn it gives way to.
import { test, expect } from '../fixtures';
import { SIZES, shot } from './helpers';
import { setMapSeed } from '../mapSeed';

test.describe.configure({ mode: 'parallel' });
for (const size of SIZES) test.describe(`people intro ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });
  // Every people at the laptop size (their text differs in length); the longest and shortest at the large one.
  const ALL = ['Switzerland', 'India', 'Byzantium', 'Germany', 'Celts', 'Mongols', 'Sparta', 'Athens', 'Aztecs', 'Egypt', 'Rome'];
  for (const people of size.height <= 640 ? ALL : ['Switzerland', 'Egypt', 'Aztecs', 'Rome']) test(`${people}: introduced, then Begin opens turn 1`, async ({ page }) => {
    // The fixtures start every other test without the introduction; this one is about it.
    await page.addInitScript(() => localStorage.setItem('legatum.intro.v1', 'on'));
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^Single Player/ }).click();
    await setMapSeed(page, '424242');
    await page.getByRole('button', { name: /^Seat 1 faction:/ }).click();
    const stage = page.locator('dialog[data-testid="faction-stage"][open]');
    await stage.getByRole('option', { name: new RegExp(people) }).click();
    await page.keyboard.press('Escape');
    await page.getByRole('button', { name: /^Start game/ }).click();
    const intro = page.getByTestId('people-intro');
    await expect(intro).toBeVisible({ timeout: 60_000 });
    await expect(page.locator('.loading-screen')).toHaveCount(0);
    await expect(intro.getByRole('heading', { name: people, exact: true })).toBeVisible();
    const begin = intro.getByRole('button', { name: /^Begin/ });
    await expect(begin).toBeFocused();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, `01-intro-${people.toLowerCase()}`);
    // Everything on the plate is inside the window: the columns are whole and Begin is in view.
    for (const part of await intro.locator('.et-col, .pi-head, .et-foot .btn').all()) {
      const box = (await part.boundingBox())!;
      expect(box.y, 'top inside the window').toBeGreaterThanOrEqual(0);
      expect(box.y + box.height, 'bottom inside the window').toBeLessThanOrEqual(size.height);
      // A column is a box of text: nothing inside it may be cut (the head's glow and the button's notch overflow by design).
      if (await part.evaluate((el) => el.classList.contains('et-col'))) expect(await part.evaluate((el) => el.scrollHeight - el.clientHeight), 'nothing is cut inside it').toBeLessThanOrEqual(1);
    }
    // The people's story is whole: never cut mid-sentence.
    expect(await intro.locator('.pi-lore').evaluate((el) => el.scrollHeight - el.clientHeight), 'the story is not cut').toBeLessThanOrEqual(1);
    // The advisor waits its turn: one Return on screen.
    await expect(page.getByTestId('advisor')).toHaveCount(0);
    if (people === 'Switzerland') {
      // How to Play opened over the plate keeps its own Esc; the introduction stays up underneath.
      // Return on the focused link follows it (a mouse click would not catch the plate taking Return for Begin).
      await intro.getByRole('button', { name: 'Full rules in How to Play' }).focus();
      await page.keyboard.press('Enter');
      const guide = page.getByRole('dialog', { name: 'How to Play' });
      await expect(guide).toBeVisible();
      await expect(guide.getByRole('button', { name: 'The peoples' })).toHaveAttribute('aria-current', 'page');
      await page.keyboard.press('Escape');
      await expect(guide).toHaveCount(0);
      await expect(intro).toBeVisible();
      await expect(begin).toBeFocused();
    }
    await page.keyboard.press('Enter');
    await expect(intro).toHaveCount(0);
    await expect(page.getByTestId('turn-number')).toHaveText('1');
    await expect(page.getByTestId('advisor')).toHaveAttribute('data-advisor-id', 'welcome');
    if (people === 'Switzerland') await shot(page, size, '02-after-begin');
  });
});
