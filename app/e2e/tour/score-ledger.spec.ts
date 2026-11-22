// Release tour of the score ledger at the two supported review sizes.
import type { Page } from '@playwright/test';
import { test, expect } from '../fixtures';
import { coreFixture } from '../../src/test/coreFixture';
import { progressFixture } from '../../src/test/progressFixture';
import { withV2 } from '../../src/test/vpV2Fixture';
import { openLateRace } from '../lateRaceSave';
import { stageHoldings } from '../vpStage';
import { SIZES, loadFixture, shot, type GameWindow } from './helpers';

test.describe.configure({ mode: 'parallel' });

async function openLedger(page: Page) {
  await page.locator('button.vh-chip').click();
  const sheet = page.getByTestId('objective-sheet');
  await expect(sheet.locator('.sl-table')).toBeVisible();
  await expect(sheet.locator('.vh-card')).toHaveCount(0);
  return sheet;
}

async function fitLedger(page: Page, height: number) {
  const metrics = await page.locator('.sl-body').evaluate((el) => {
    const last = el.querySelector('.sl-row.k-era:last-of-type') ?? [...el.querySelectorAll('.sl-row.k-era')].at(-1)!;
    const body = el.getBoundingClientRect(), row = last.getBoundingClientRect();
    return { lastBottom: row.bottom, bodyBottom: body.bottom, sideways: el.scrollWidth - el.clientWidth, scroll: el.scrollHeight - el.clientHeight,
      bodyHeight: el.clientHeight, tableHeight: el.querySelector('table')!.getBoundingClientRect().height,
      dialogHeight: el.closest('dialog')!.getBoundingClientRect().height, headHeight: el.previousElementSibling!.getBoundingClientRect().height };
  });
  expect(metrics.sideways, 'no horizontal scroll').toBeLessThanOrEqual(1);
  if (height === 600) {
    expect(metrics.scroll, `closed ledger fits vertically: ${JSON.stringify(metrics)}`).toBeLessThanOrEqual(1);
    expect(metrics.lastBottom, 'last era row above fold').toBeLessThanOrEqual(metrics.bodyBottom + 1);
  }
}

for (const size of SIZES) test.describe(`score ledger ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  for (const aboutToScore of [false, true]) test(`objective accounting ${aboutToScore ? 'about to score' : 'midway'}`, async ({ page }) => {
    await loadFixture(page, coreFixture(progressFixture().fixture), 2);
    const base = await page.evaluate(() => { const s = (window as unknown as GameWindow).legatum.getState(); return structuredClone(s.views![s.seat]); });
    const era = aboutToScore ? 3 : 2;
    const view = withV2(base, { era, scored: [], progress: aboutToScore ? { '3:1': [40, 40] } : { '2:1': [2, 4] } });
    view.objective_accounting = [{ era, slot: 1, baseline: 9, total: aboutToScore ? 49 : 11,
      counting_since_turn: 112, restart_eras: aboutToScore ? [2] : [] }];
    if (aboutToScore) view.objective_progress.find(p => p.era === 3 && p.slot === 1)!.ready = true;
    await page.evaluate((v) => { const store = (window as unknown as GameWindow).legatum, s = store.getState(); store.setState({ views: s.views!.map((x, i) => i === s.seat ? v : x) }); }, view);
    const sheet = await openLedger(page);
    const detail = sheet.locator(`#sl-detail-era-${era}`);
    const row = sheet.locator(`.sl-row[data-row="era-${era}"] button`);
    if (await row.getAttribute('aria-expanded') === 'false') await row.click();
    await expect(detail.getByTestId('era-count-account')).toContainText(aboutToScore ? 'You had 9 population on turn 112; now 49.' : 'You had 9 cities on turn 112; now 11.');
    if (aboutToScore) await expect(detail.getByTestId('era-count-restart')).toContainText('your Classical progress goes back to zero');
    await detail.getByTestId(aboutToScore ? 'era-count-restart' : 'era-count-account').scrollIntoViewIfNeeded();
    await expect(detail.getByTestId(aboutToScore ? 'era-count-restart' : 'era-count-account')).toBeInViewport();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, aboutToScore ? '80j-objectives-count-about-to-score' : '80i-objectives-count-midway');
  });

  test('2 empires: all sources, rank order and default expansion', async ({ page }) => {
    await loadFixture(page, coreFixture(progressFixture().fixture), 2);
    await stageHoldings(page, { oneCardPerAge: true });
    const sheet = await openLedger(page);
    await expect(sheet.getByRole('button', { name: 'Close objectives' })).toHaveClass(/\bbtn\b/);
    await expect(sheet.locator('.sl-col')).toHaveCount(2);
    // The two Ages that have begun in the staged view.
    await expect(sheet.locator('.sl-row.k-era')).toHaveCount(2);
    await expect(sheet.locator('.sl-section')).toContainText(['Kept for good', 'Held', 'Ages']);
    await expect(sheet.locator('.sl-row.k-heldGoal')).toHaveCount(1);
    for (const seal of await sheet.locator('.sl-row .sl-seal').all()) {
      const box = (await seal.boundingBox())!, dialog = (await sheet.boundingBox())!;
      expect(box.x, 'VP seal stays inside the ledger').toBeGreaterThanOrEqual(dialog.x);
      expect(box.x + box.width, 'VP seal stays inside the ledger').toBeLessThanOrEqual(dialog.x + dialog.width);
    }
    // The newest Age you have not scored opens with the ledger, at every window size.
    await expect(sheet.locator('.sl-row-detail')).toHaveCount(1);
    await fitLedger(page, size.height);
    await page.mouse.move(4, size.height / 2);
    await shot(page, size, '80a-score-ledger-2-empires');
  });

  test('8 empires: compact head and a whole final era row', async ({ page }) => {
    await openLateRace(page);
    const sheet = await openLedger(page);
    await expect(sheet.locator('.sl-col')).toHaveCount(8);
    // At most the newest Age you have not scored is open.
    expect(await sheet.locator('.sl-row-detail').count()).toBeLessThanOrEqual(1);
    await fitLedger(page, size.height);
    await page.mouse.move(4, size.height / 2);
    await shot(page, size, '80b-score-ledger-8-empires');
  });

  test('team: 20 VP shown for both partners', async ({ page }) => {
    await openLateRace(page);
    await page.evaluate(() => {
      const store = (window as unknown as GameWindow).legatum, s = store.getState(), v = structuredClone(s.views![s.seat]);
      const me = v.players.find((p) => p.id === v.me)!, mate = v.players.find((p) => p.id !== v.me && p.alive)!;
      const former = v.standings.find((r) => r.members.includes(mate.id))!;
      mate.team = me.team;
      const own = v.standings.find((r) => r.team === me.team)!;
      own.members.push(mate.id); own.vp += former.vp; own.target = 20;
      v.standings = v.standings.filter((r) => r !== former);
      store.setState({ views: s.views!.map((x, i) => i === s.seat ? v : x) });
    });
    const sheet = await openLedger(page);
    const mine = sheet.locator('.sl-col.own-team');
    await expect(mine).toHaveCount(2);
    await expect(sheet.locator('.sl-col.mine')).toHaveCount(1);
    await expect(mine.first().locator('.sl-col-vp')).toContainText('/20');
    await expect(mine.nth(1).locator('.sl-col-vp')).toContainText('/20 VP');
    await fitLedger(page, size.height);
    await page.mouse.move(4, size.height / 2);
    await shot(page, size, '80c-score-ledger-team');
  });

  test('a hold in progress: turns left under the holder VP', async ({ page }) => {
    await loadFixture(page, coreFixture(progressFixture().fixture), 2);
    const base = await page.evaluate(() => { const s = (window as unknown as GameWindow).legatum.getState(); return structuredClone(s.views![s.seat]); });
    const v = withV2(base, { era: 4, scored: [2, 3, 4] });
    v.great_works = (v.great_works ?? []).map((work) => work.player === v.me ? { ...work, completed: 'GreatLibrary' } : work);
    v.players = v.players.map((p) => {
      if (p.id !== v.me) return p;
      const vp_sources: typeof p.vp_sources = [
        [{ kind: 'GreatWork', of: v.me }, 2], [{ kind: 'FactionGoal', of: v.me }, 2],
        ...([2, 3, 4] as const).map((era): typeof p.vp_sources[number] => [{ kind: 'EraObjective', era }, 1]),
      ];
      return { ...p, vp: 10, faction_goal_done: true, vp_sources };
    });
    const own = v.standings.find((r) => r.team === v.team)!;
    own.vp = 10; own.victory_hold = { turns_held: 7, turns_needed: 10 };
    await page.evaluate((view) => { const store = (window as unknown as GameWindow).legatum, s = store.getState(); store.setState({ views: s.views!.map((x, i) => i === s.seat ? view : x) }); }, v);
    const sheet = await openLedger(page);
    await expect(sheet.locator('.sl-col.mine .sl-hold')).toContainText('win in 3');
    await expect(sheet.locator('.sl-col:not(.mine) .sl-hold')).toHaveCount(0);
    await fitLedger(page, size.height);
    await page.mouse.move(4, size.height / 2);
    await shot(page, size, '80d-score-ledger-hold');
  });

  test('held Heir: another empire scores Byzantium’s goal', async ({ page }) => {
    await openLateRace(page);
    const names = await page.evaluate(() => {
      const store = (window as unknown as GameWindow).legatum, s = store.getState(), v = structuredClone(s.views![s.seat]);
      const of = v.players.find((p) => p.faction === 'Byzantium' && p.alive)!, holder = v.players.find((p) => p.id !== of.id && p.alive)!;
      for (const p of v.players) {
        const had = p.vp_sources.some(([source]) => source.kind === 'FactionGoal' && source.of === of.id);
        if (had) {
          p.vp_sources = p.vp_sources.filter(([source]) => source.kind !== 'FactionGoal' || source.of !== of.id);
          p.vp -= 2;
          v.standings.find((r) => r.members.includes(p.id))!.vp -= 2;
        }
      }
      holder.vp += 2; holder.vp_sources.push([{ kind: 'FactionGoal', of: of.id }, 2]);
      v.standings.find((r) => r.members.includes(holder.id))!.vp += 2;
      store.setState({ views: s.views!.map((x, i) => i === s.seat ? v : x) });
      return { of: of.id, holder: holder.id };
    });
    const sheet = await openLedger(page);
    const goal = sheet.locator('.sl-row.k-heldGoal');
    await expect(goal.locator('.sl-token-of')).toHaveCount(1);
    await expect(goal.locator('.sl-token[title*="The Heir to the Throne"]')).toHaveCount(1);
    await fitLedger(page, size.height);
    await goal.locator('button').click();
    await expect(sheet.locator('#sl-detail-held-goal')).toContainText('Held by');
    await page.mouse.move(4, size.height / 2);
    await shot(page, size, '80e-score-ledger-held-heir');
    expect(names.holder).not.toBe(names.of);
  });
});
