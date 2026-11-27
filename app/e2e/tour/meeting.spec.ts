// First contact (celebrate/MeetingTakeover.tsx): the full-screen meeting with another empire, at both desktop sizes,
// for the peoples with the longest and shortest text, and what Return (the map goes to where you met them), Esc and Diplomacy do.
import { test, expect } from '../fixtures';
import { coreFixture } from '../../src/test/coreFixture';
import { wallsFixture } from '../../src/test/wallsFixture';
import { meetingsForTurn } from '../../src/celebrate/moments';
import type { Faction, PlayerView } from '../../src/types';
import { SIZES, loadFixture, shot, type GameWindow } from './helpers';

const ALL: Faction[] = ['Switzerland', 'India', 'Byzantium', 'Germany', 'Celts', 'Mongols', 'Sparta', 'Athens', 'Aztecs', 'Egypt', 'Rome'];

/** The walls fixture with its second empire played by `people`, an AI, and the turn it is met. */
function save(people: Faction, war = false) {
  const setup = wallsFixture();
  const f = setup.fixture;
  // The usual first meeting is at Peace; Sparta's shot keeps the War line in the tour.
  // The core records the contact too, so Diplomacy afterwards agrees with the plate.
  f.diplomacy!.contacts = [[0, 1]];
  if (!war) for (const r of f.diplomacy!.relations) if (r.a === 1 || r.b === 1) Object.assign(r, { state: 'Peace', war_id: null, declarers: [] });
  f.players[1].faction = people; f.setup.players[1].faction = people;
  for (const p of [1, 2]) { f.players[p].isHuman = false; f.setup.players[p].is_human = false; }
  return { game: coreFixture(f), turn: f.turn };
}

async function meet(page: import('@playwright/test').Page) {
  const view = await page.evaluate(() => { const s = (window as unknown as GameWindow).legatum.getState(); return s.views![s.seat]; }) as PlayerView;
  const before = { ...view, diplomacy: { ...view.diplomacy, met: [] } };
  const after = { ...view, diplomacy: { ...view.diplomacy, met: [1] } };
  const moments = meetingsForTurn(before, after, 0);
  expect(moments).toHaveLength(1);
  const report = { turn: view.turn, hash: '', events: [[]], moments: [moments], alarms: [[]], lines: [[]] };
  await page.evaluate((report) => (window as unknown as GameWindow).legatum.getState().set({ report, alarmSeen: [] }), report);
  return page.getByTestId('meeting-takeover');
}

test.describe.configure({ mode: 'parallel' });
test.use({ meetings: 'show' });
for (const size of SIZES) test.describe(`first contact ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });
  for (const people of size.height <= 640 ? ALL : ['Mongols', 'Sparta', 'Byzantium', 'Rome'] as Faction[]) test(`${people}: met, then Return closes it`, async ({ page }) => {
    const { game, turn } = save(people, people === 'Sparta');
    await loadFixture(page, game, turn);
    const plate = await meet(page);
    await expect(plate).toBeVisible();
    await expect(plate).toHaveAttribute('data-people', people);
    await expect(plate.getByRole('button', { name: /^Continue/ })).toBeFocused();
    await expect.poll(() => page.evaluate(() => (window as unknown as GameWindow).legatum.getState().meetingPeople)).toBe(people);
    await page.mouse.move(5, size.height / 2);
    await page.waitForTimeout(2600);
    await shot(page, size, `01-meet-${people.toLowerCase()}`);
    // Everything on the plate is inside the window, and no box of text is cut.
    for (const part of await plate.locator('.et-col, .mt-head, .et-foot .btn').all()) {
      const box = (await part.boundingBox())!;
      expect(box.y, 'top inside the window').toBeGreaterThanOrEqual(0);
      expect(box.y + box.height, 'bottom inside the window').toBeLessThanOrEqual(size.height);
      if (await part.evaluate((el) => el.classList.contains('et-col'))) expect(await part.evaluate((el) => el.scrollHeight - el.clientHeight), 'nothing is cut inside it').toBeLessThanOrEqual(1);
    }
    const where = await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().report!.moments![0]![0]!.meeting!.where!.at);
    await page.keyboard.press('Enter');
    await expect(plate).toHaveCount(0);
    // Closing it takes the map to where they were met.
    expect(await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().focus?.hex)).toEqual(where);
    await expect.poll(() => page.evaluate(() => (window as unknown as GameWindow).legatum.getState().meetingPeople)).toBeNull();
  });

  test('a team of two: both portraits, and each column describes both peoples', async ({ page }) => {
    const { game, turn } = save('Egypt');
    await loadFixture(page, game, turn);
    const view = await page.evaluate(() => { const s = (window as unknown as GameWindow).legatum.getState(); return s.views![s.seat]; }) as PlayerView;
    const before = { ...view, diplomacy: { ...view.diplomacy, met: [] } };
    const after = { ...view, diplomacy: { ...view.diplomacy, met: [1] } };
    const [moment] = meetingsForTurn(before, after, 0);
    // The fixture's empires are each their own team; a card naming two members is what a team of two produces.
    moment.meeting!.members = [{ id: 1, faction: 'Egypt' }, { id: 2, faction: 'Switzerland' }];
    const report = { turn: view.turn, hash: '', events: [[]], moments: [[moment]], alarms: [[]], lines: [[]] };
    await page.evaluate((report) => (window as unknown as GameWindow).legatum.getState().set({ report, alarmSeen: [] }), report);
    const plate = page.getByTestId('meeting-takeover');
    await expect(plate).toBeVisible();
    await expect(plate.locator('.mt-portrait')).toHaveCount(2);
    for (const col of ['meeting-strengths', 'meeting-prices']) {
      await expect(plate.getByTestId(col).locator('.mt-people-name')).toHaveText([/Egypt/i, /Switzerland/i]);
      for (const people of ['Egypt', 'Switzerland']) await expect(plate.getByTestId(col).locator(`[data-people=${people}] .mr`).first()).toBeVisible();
    }
    await page.mouse.move(5, size.height / 2);
    await page.waitForTimeout(2600);
    // Both peoples fit: every column whole and inside the window.
    for (const part of await plate.locator('.et-col, .mt-head, .et-foot .btn').all()) {
      const box = (await part.boundingBox())!;
      expect(box.y + box.height, 'bottom inside the window').toBeLessThanOrEqual(size.height);
      if (await part.evaluate((el) => el.classList.contains('et-col'))) expect(await part.evaluate((el) => el.scrollHeight - el.clientHeight), 'nothing is cut inside it').toBeLessThanOrEqual(1);
    }
    await shot(page, size, '03-meet-team-of-two');
  });

  test('the sealed letter breaks open', async ({ page }) => {
    const { game, turn } = save('Rome');
    await loadFixture(page, game, turn);
    await page.mouse.move(5, size.height / 2);
    const plate = await meet(page);
    await expect(plate).toBeVisible();
    const t0 = Date.now();
    for (const at of [250, 800, 1300, 1800]) {
      await page.waitForTimeout(Math.max(0, at - (Date.now() - t0)));
      await page.screenshot({ path: test.info().outputPath(`unwrap-${at}ms.png`) });
      await test.info().attach(`unwrap-${at}ms`, { path: test.info().outputPath(`unwrap-${at}ms.png`), contentType: 'image/png' });
    }
  });

  test('Continue goes to where they were met; Esc leaves the map where it was', async ({ page }) => {
    const { game, turn } = save('Athens');
    await loadFixture(page, game, turn);
    const focusOf = () => page.evaluate(() => (window as unknown as GameWindow).legatum.getState().focus?.seq ?? 0);
    let plate = await meet(page);
    await expect(plate).toBeVisible();
    const before = await focusOf();
    await page.keyboard.press('Escape');
    await expect(plate).toHaveCount(0);
    expect(await focusOf()).toBe(before);
    plate = await meet(page);
    await expect(plate).toBeVisible();
    const where = await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().report!.moments![0]![0]!.meeting!.where!.at);
    await plate.getByRole('button', { name: /^Continue/ }).click();
    await expect(plate).toHaveCount(0);
    expect(await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().focus?.hex)).toEqual(where);
    await page.waitForTimeout(600);
    await shot(page, size, '04-after-meeting-map');
  });

  test('Diplomacy opens on the empire just met', async ({ page }) => {
    const { game, turn } = save('Egypt');
    await loadFixture(page, game, turn);
    const plate = await meet(page);
    await plate.getByRole('button', { name: 'Diplomacy' }).click();
    await expect(plate).toHaveCount(0);
    await expect(page.getByRole('complementary', { name: 'Diplomacy' })).toBeVisible();
    expect(await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().diplomacyTeam)).toBe(1);
    await shot(page, size, '02-diplomacy-after-meeting');
  });
});
