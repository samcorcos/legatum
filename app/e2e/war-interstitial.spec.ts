// Playtest 6: "orange declared war on me ... I only just noticed it after several turns ... It needs
// a full interstitial like you get with Civ 6." The real Spartan AI declares war in turn 1 of the
// fixture; the declaration takes the whole screen before the turn report and the celebrations.
import type { Page } from '@playwright/test';
import type { useGame } from '../src/store';
import type { Hex } from '../src/types';
import { test, expect } from './fixtures';
import { endPlanning } from './turnActions';
import { warDeclaredCoreSave } from '../src/test/warDeclaredFixture';

type GameWindow = Window & { legatum: typeof useGame; legatumMap: { toScreen(hex: Hex): { x: number; y: number } } };

async function load(page: Page) {
  await page.addInitScript(({ game, names }) => {
    if (!location.protocol.startsWith('http')) return;
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn: 1, names, savedAt: new Date().toISOString(), game }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, { game: warDeclaredCoreSave(), names: ['Ada'] });
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('1');
}

const alarm = (page: Page) => page.getByRole('alertdialog', { name: 'The Spartans declare war on you' });
const report = (page: Page) => page.getByRole('complementary', { name: 'Turn report' });

test('War declared on you: a full-screen declaration before the report; Return centres the map on the threat', async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await load(page);
  await endPlanning(page);
  await expect(page.getByTestId('turn-number')).toHaveText('2');
  const screen = alarm(page);
  await expect(screen).toBeVisible();
  // Before anything else: no turn report, no celebration card under it.
  await expect(report(page)).toHaveCount(0);
  await expect(page.locator('.moment-card')).toHaveCount(0);
  // The declaring leader's face: their engraved portrait with the crest pinned on it.
  const portrait = screen.locator('.alarm-leader .pf > img');
  await expect(portrait).toHaveCount(1);
  expect(await portrait.evaluate((img: HTMLImageElement) => img.complete && img.naturalWidth > 0), 'leader portrait loaded').toBe(true);
  await expect(screen.locator('.alarm-leader .pf-crest')).toHaveCount(1);
  await expect(screen).toContainText('“Train in the agoge. Hold the line.”');
  await expect(screen).toContainText('War is in effect now.');
  await expect(screen).toContainText('No treaty stood between you, so the declaration took effect at once (a surprise war).');
  await expect(screen.locator('.alarm-where')).toContainText(/Pikeman/);
  await expect(screen.locator('.alarm-where')).toContainText(/Oathhold, \d+ tiles W of Waystation/);
  const primary = screen.locator('.alarm-primary');
  await expect(primary).toHaveAccessibleName(/^Show on map/);
  await expect(primary).toBeFocused();
  await expect(screen.getByRole('button', { name: 'Open diplomacy', exact: true })).toBeVisible();
  await expect(page.getByTestId('alarm-screen')).toHaveAttribute('data-armed', '');

  await page.keyboard.press('Enter');
  await expect(page.getByTestId('alarm-screen')).toHaveCount(0);
  // The map centres on their nearest force in sight.
  const focus = await page.evaluate(() => {
    const s = (window as unknown as GameWindow).legatum.getState(), v = s.views![s.seat];
    const hex = s.focus!.hex;
    return { hex, theirs: v.units.some((u) => u.owner === 1 && u.pos.q === hex.q && u.pos.r === hex.r) };
  });
  expect(focus.theirs).toBe(true);
  await expect.poll(() => page.evaluate((hex) => {
    const p = (window as unknown as GameWindow).legatumMap.toScreen(hex);
    const b = document.querySelector('[data-game-map]')!.getBoundingClientRect();
    return Math.hypot(p.x - b.width / 2, p.y - b.height / 2);
  }, focus.hex)).toBeLessThan(4);

  // The event stays reachable: the report line (red, down arrow) and a new red war pin in the race.
  await expect.poll(async () => {
    // A moment card holds the report back; L opens it (the card has no Turn report button).
    if (await page.getByTestId('moment-lane').locator('.moment-card').isVisible()) await page.keyboard.press('l');
    return report(page).isVisible();
  }).toBe(true);
  const line = report(page).locator('.rline', { hasText: /Sparta declared war/ });
  await expect(line).toHaveAttribute('data-tone', 'bad');
  await expect(line.locator('.rline-tone .rm.bad .rm-sign .mask-icon')).toHaveCount(1);
  await expect(page.locator('.vp-row .vhc-rel.war.new')).toHaveAttribute('data-relation', /War/);
  // Once seen, it does not come back this turn.
  await page.keyboard.press('Escape');
  await expect(page.getByTestId('alarm-screen')).toHaveCount(0);
});

test('War declared on you: Esc closes it where it is; Open diplomacy opens Sparta', async ({ page }) => {
  await page.setViewportSize({ width: 1000, height: 600 });
  await load(page);
  await endPlanning(page);
  await expect(alarm(page)).toBeVisible();
  // The primary action is visible at the laptop size without scrolling.
  const primary = alarm(page).locator('.alarm-primary');
  await expect(primary).toHaveAccessibleName(/^Show on map/);
  await expect(primary).toBeInViewport();
  await expect(page.getByTestId('alarm-screen')).toHaveAttribute('data-armed', '');
  const before = await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().focus);
  await page.keyboard.press('Escape');
  await expect(page.getByTestId('alarm-screen')).toHaveCount(0);
  expect(await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().focus)).toEqual(before);

  const other = await page.context().newPage();
  await load(other);
  await endPlanning(other);
  await expect(alarm(other)).toBeVisible();
  await alarm(other).getByRole('button', { name: 'Open diplomacy', exact: true }).click();
  await expect(other.getByTestId('alarm-screen')).toHaveCount(0);
  await expect(other.getByRole('complementary', { name: 'Diplomacy', exact: true })).toBeVisible();
  await other.close();
});
