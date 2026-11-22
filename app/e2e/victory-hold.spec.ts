// The user changed the victory rule: a team wins by VP only after holding its target for N turns,
// "there should be a clear countdown so you know that someone hit 10 points and they're about to
// win. that should muster action quickly." The alarm comes to every player the moment a team
// reaches its target; the report keeps its line (reopenable) and a broken hold gets its own notice.
import type { Page } from '@playwright/test';
import type { useGame } from '../src/store';
import { test, expect } from './fixtures';
import { ALARM_TURN, alarmCoreSave } from '../src/test/alarmFixture';
import { holdBroken, holdStarted, stageHoldReport } from './holdFixture';

type GameWindow = Window & { legatum: typeof useGame };

const SIZES = [{ width: 1000, height: 600 }, { width: 1440, height: 900 }] as const;
const shots = process.env.HOLD_SHOTS;

async function load(page: Page) {
  await page.addInitScript(({ game, turn }) => {
    if (!location.protocol.startsWith('http')) return;
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn, names: ['Ada'], savedAt: new Date().toISOString(), game }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, { game: alarmCoreSave(), turn: ALARM_TURN });
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText(String(ALARM_TURN));
  await expect(page.locator('.loading-screen')).toHaveCount(0);
}

const alarm = (page: Page) => page.getByTestId('alarm-screen');
const report = (page: Page) => page.getByRole('complementary', { name: 'Turn report' });

/** The primary is visible without scrolling and nothing in the sheet overflows the window. */
async function expectFits(page: Page, primaryName: RegExp) {
  const size = page.viewportSize()!;
  await expect(alarm(page).getByRole('button', { name: primaryName })).toBeInViewport();
  const sheet = await alarm(page).locator('.alarm-sheet').boundingBox();
  expect(sheet!.x).toBeGreaterThanOrEqual(0);
  expect(sheet!.y).toBeGreaterThanOrEqual(0);
  expect(sheet!.x + sheet!.width).toBeLessThanOrEqual(size.width + 0.5);
  expect(sheet!.y + sheet!.height).toBeLessThanOrEqual(size.height + 0.5);
  const overflow = await alarm(page).locator('.alarm-sheet').evaluate((el) => ({ x: el.scrollWidth - el.clientWidth, y: el.scrollHeight - el.clientHeight }));
  expect(overflow.x, 'no horizontal overflow').toBeLessThanOrEqual(1);
  const body = await alarm(page).locator('.alarm-body').evaluate((el) => el.scrollHeight - el.clientHeight);
  expect(body, 'the whole body shows without scrolling').toBeLessThanOrEqual(1);
}

async function shot(page: Page, name: string) {
  if (!shots) return;
  await page.mouse.move(5, page.viewportSize()!.height / 2);
  await page.waitForTimeout(500);
  await page.screenshot({ path: `${shots}/${page.viewportSize()!.width}x${page.viewportSize()!.height}-${name}.png`, animations: 'disabled' });
}

for (const size of SIZES) {
  test.describe(`${size.width}x${size.height}`, () => {
    test.use({ viewport: size });

    test('a rival reaches its target: every player gets the alarm with the countdown and one primary', async ({ page }) => {
      await load(page);
      await stageHoldReport(page, [holdStarted(3)], { holding: { team: 3, held: 1, needed: 10 } });
      const screen = page.getByRole('alertdialog', { name: 'The Swiss have 10 VP' });
      await expect(screen).toBeVisible();
      await expect(alarm(page)).toHaveAttribute('data-alarm', 'victoryHold');
      await expect(alarm(page)).toHaveAttribute('data-armed', '');
      await expect(screen).toContainText('They must hold it for 10 turns unless someone stops them.');
      await expect(screen).toContainText('A rival nears victory');
      // The holder's engraved portrait, the countdown as a meter and a plain statement of the rule.
      const portrait = screen.locator('.alarm-leader .pf > img');
      await expect(portrait).toHaveCount(1);
      expect(await portrait.evaluate((img: HTMLImageElement) => img.complete && img.naturalWidth > 0)).toBe(true);
      await expect(screen.getByRole('meter', { name: 'Turns held' })).toHaveAttribute('aria-valuetext', '1 of 10 turns held');
      await expect(screen.locator('.hold-pips i')).toHaveCount(10);
      await expect(screen.locator('.hold-pips i.on')).toHaveCount(1);
      await expect(screen.getByTestId('hold-count')).toContainText('9 to go');
      await expect(screen).toContainText('Below 10 VP at any turn end, the count goes back to zero.');
      await expect(screen).not.toContainText(/fastest|best way|should (attack|declare)/i);
      // Against a rival the one primary is Declare war while it is legal (rules 27, 2026-11-11); Return opens its confirmation.
      const primary = screen.getByRole('button', { name: /^Declare war on the Swiss/ });
      await expect(primary).toBeFocused();
      await expectFits(page, /^Declare war on the Swiss/);
      await shot(page, 'hold-rival');
      await page.keyboard.press('Enter');
      await expect(alarm(page)).toHaveCount(0);
      await expect(page.getByRole('dialog', { name: /^Declare War on .*\?$/ })).toBeVisible();
    });

    test('you reach your target: your own variant, Esc closes it, and the report line reopens it', async ({ page }) => {
      await load(page);
      await stageHoldReport(page, [holdStarted(0)], { holding: { team: 0, held: 1, needed: 10 } });
      const screen = page.getByRole('alertdialog', { name: 'You have 10 VP' });
      await expect(screen).toBeVisible();
      await expect(alarm(page)).toHaveAttribute('data-armed', '');
      await expect(screen).toContainText('Hold it for 10 turns to win; every rival will now come for you.');
      await expect(screen).toContainText('You win the game when the count reaches 10 turns.');
      await expectFits(page, /^See the race/);
      await shot(page, 'hold-self');
      await page.keyboard.press('Escape');
      await expect(alarm(page)).toHaveCount(0);
      expect(await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().vpExpanded)).toBe(false);
      // The turn report keeps the line; its Show card brings the same card back with the live count.
      await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().set({ reportOpen: true }));
      const line = report(page).locator('.rline', { hasText: /You reached 10 VP and must hold it for 10 turns to win/ });
      await expect(line).toHaveAttribute('data-tone', 'good');
      await line.getByRole('button', { name: 'Show card' }).click();
      await expect(screen).toBeVisible();
      await expect(alarm(page)).toHaveAttribute('data-armed', '');
      await expect(screen.getByRole('meter', { name: 'Turns held' })).toHaveAttribute('aria-valuenow', '1');
      await page.keyboard.press('Escape');
      await expect(alarm(page)).toHaveCount(0);
    });

    test('an ally reaches its target: the ally variant says an Alliance does not share victory', async ({ page }) => {
      await load(page);
      await stageHoldReport(page, [holdStarted(1)], { holding: { team: 1, held: 1, needed: 10 } });
      const screen = page.getByRole('alertdialog', { name: /your allies, have 10 VP$/ });
      await expect(screen).toBeVisible();
      await expect(screen).toContainText('an Alliance does not share victory');
      await expectFits(page, /^See the race/);
      await shot(page, 'hold-ally');
    });

    test('a broken hold is a notice in the report, from the viewer\'s side', async ({ page }) => {
      await load(page);
      await stageHoldReport(page, [holdBroken(3)], { reportOpen: true });
      await expect(alarm(page)).toHaveCount(0);
      const rivalLine = report(page).locator('.rline', { hasText: /hold on 10 VP was broken after 6 of 10 turns/ });
      await expect(rivalLine).toHaveAttribute('data-tone', 'good');
      await expect(rivalLine.locator('.rm.good .rm-sign')).toHaveCount(1);
      await shot(page, 'hold-broken-rival');
      await stageHoldReport(page, [holdBroken(0)], { reportOpen: true });
      const mine = report(page).locator('.rline', { hasText: /Your hold on 10 VP was broken after 6 of 10 turns/ });
      await expect(mine).toHaveAttribute('data-tone', 'bad');
      await expect(mine.locator('.rm.bad .rm-sign')).toHaveCount(1);
    });
  });
}
