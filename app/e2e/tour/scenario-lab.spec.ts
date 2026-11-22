// Screen tour, Scenario Lab cards. Every new Lab card gets a shot scrolled into view, with its briefing (legatum-ui,
// "Tour hygiene"), Hold Biel's objective ring on Biel, and the dropped-in file cards ("Added file", "Cannot be played").
// Run alone with `npx playwright test e2e/tour/scenario-lab`. Shots land in test-results/screen-tour/<size>/scenario-lab/.
import { test, expect } from '../fixtures';
import { SIZES, shot } from './helpers';

test.describe.configure({ mode: 'parallel' });

const TACTICS = [
  ['sparta-walled-nicomedia', 'Six Spartans and a Wall'],
  ['sparta-before-the-walls', 'Before the Walls'],
  ['germany-two-cities-one-turn', 'Two Cities, Two Bombers'],
  ['swiss-hold-biel', 'Hold Biel'],
] as const;

for (const size of SIZES) test.describe(`screen tour ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  test('the four tactics cards and their briefings', async ({ page }) => {
    await page.addInitScript(() => {
      if (!location.protocol.startsWith('http')) return;
      localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
    });
    for (const [slug, title] of TACTICS) {
      await page.goto('/?client=wasm');
      await page.getByRole('button', { name: 'Scenarios', exact: true }).click();
      await expect(page.getByRole('heading', { name: 'Scenario Lab', exact: true })).toBeVisible();
      const option = page.getByRole('option', { name: new RegExp(`^${title}|${title}`) }).first();
      await option.scrollIntoViewIfNeeded();
      await page.mouse.move(5, size.height / 2);
      await shot(page, size, `sl1-card-${slug}`);
      await option.click();
      await expect(page.getByTestId('lab-briefing')).toBeVisible();
      await page.mouse.move(5, size.height / 2);
      await shot(page, size, `sl2-briefing-${slug}`);
    }
  });

  test('Hold Biel in play: the target is Biel', async ({ page }) => {
    await page.addInitScript(() => {
      if (!location.protocol.startsWith('http')) return;
      localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
    });
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: 'Scenarios', exact: true }).click();
    await page.getByRole('option', { name: /Hold Biel/ }).first().click();
    await expect(page.getByTestId('lab-briefing')).toBeVisible();
    await page.getByRole('button', { name: /^Begin/ }).first().click();
    await expect(page.getByTestId('scenario-bar')).toBeVisible({ timeout: 120_000 });
    await expect(page.locator('.loading-screen')).toHaveCount(0, { timeout: 120_000 });
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'sl3-hold-biel-in-play');
    await page.getByTestId('scenario-show-target').click();
    await page.waitForTimeout(800);
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'sl4-hold-biel-target');
    await page.getByTestId('scenario-bar').locator('.sb-main').click();
    await expect(page.locator('#scenario-details')).toBeVisible();
    await shot(page, size, 'sl5-hold-biel-details');
  });

  test('dropped-in files: Added file and Cannot be played', async ({ page }) => {
    // This tour runs against the built app in the WebKit container, which has no repo scenario files.
    // The card only needs catalogue metadata; no dropped-in game is started here.
    const bundled = {
      format: 'legatum-scenario', version: 1, revision: 1, id: 'dropped-in-test',
      title: 'Dropped-In Test', faction: 'Athens', seat: 0, minutes: 7,
      objective: { turns: 9 }, lab: { order: 0, question: 'Can you break the hold?', blurb: 'A new scenario from the folder.' },
    };
    const files = [
      { file: 'dropped-in-test.json', text: JSON.stringify(bundled), reason: null },
      { file: 'sparta-next-rush.json', text: JSON.stringify({ ...bundled, id: 'sparta-next-rush', title: 'Next Rush' }), reason: 'made for rules version 24; this app is 25' },
      { file: 'notes.json', text: '{ not json', reason: 'invalid scenario: expected value at line 1 column 3' },
    ];
    await page.addInitScript((folder) => {
      if (!location.protocol.startsWith('http')) return;
      const w = window as unknown as { isTauri?: boolean; __TAURI_INTERNALS__?: unknown };
      w.isTauri = true;
      w.__TAURI_INTERNALS__ = { invoke: async (cmd: string) => (cmd === 'scenario_dropins' ? folder : null), transformCallback: () => 0 };
      localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
    }, files);
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: 'Scenarios', exact: true }).click();
    await expect(page.getByRole('heading', { name: 'Scenario Lab', exact: true })).toBeVisible();
    const added = page.getByTestId('lab-card-dropped-in-test');
    await expect(added).toContainText('Added file');
    await added.scrollIntoViewIfNeeded();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'sl6-dropin-added');
    const old = page.getByTestId('lab-unavailable-sparta-next-rush.json');
    await expect(old).toContainText('Cannot be played');
    await old.scrollIntoViewIfNeeded();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'sl7-dropin-cannot-play');
  });
});
