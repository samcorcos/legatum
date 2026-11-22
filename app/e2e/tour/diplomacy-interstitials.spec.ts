import { test, expect } from '../fixtures';
import { coreFixture } from '../../src/test/coreFixture';
import { siegeFixture } from '../../src/test/siegeFixture';
import { alarmsForTurn } from '../../src/celebrate/alarms';
import type { Event, PlayerView, Relation } from '../../src/types';
import { SIZES, loadFixture, shot, type GameWindow } from './helpers';
import { endPlanning } from '../turnActions';

for (const size of SIZES) test.describe(`diplomacy moments ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  test('waits for an open action and Escape clears the queue so the turn is playable', async ({ page }) => {
    const setup = siegeFixture({ approach: true });
    await loadFixture(page, coreFixture(setup.fixture), 2);
    await page.locator('.topbar .research-chip').click();
    const research = page.getByRole('dialog', { name: 'Research', exact: true });
    const close = research.getByRole('button', { name: 'Close research', exact: true });
    await close.focus();
    await expect(close).toBeFocused();
    const view = await page.evaluate(() => {
      const s = (window as unknown as GameWindow).legatum.getState();
      return s.views![s.seat];
    }) as PlayerView;
    const relation = (a: number, b: number, state: Relation['state'], declarers: number[] = []): Relation => ({
      a, b, state, effective_turn: view.turn, guarantee_until: view.turn + 10, version: 2,
      security: null, war_id: 2, declarers,
    });
    const cards = alarmsForTurn([
      { type: 'WarDeclared', relation: relation(0, 1, 'War', [1]) },
      { type: 'PeaceEstablished', relation: relation(0, 2, 'Peace') },
    ], view, view);
    await page.evaluate((cards) => {
      const s = (window as unknown as GameWindow).legatum.getState();
      const report = s.report ?? { turn: s.views![s.seat].turn, lines: [], events: [], hash: 'tour' };
      s.set({ report: { ...report, alarms: s.views!.map((_, i) => i === s.seat ? cards : []) }, alarmSeen: [], alarmStep: {} });
    }, cards);
    await expect(page.getByTestId('alarm-screen')).toHaveCount(0);
    await expect(close).toBeFocused();
    await close.click();
    const modal = page.getByTestId('alarm-screen');
    await expect(modal).toBeVisible();
    await page.keyboard.press('Escape'); // no arming delay and no repeated key presses for a queue
    await expect(modal).toHaveCount(0);
    await expect(page.locator('.seatbar .cta').first()).toBeVisible();
    expect(await page.evaluate(() => {
      const s = (window as unknown as GameWindow).legatum.getState();
      return { seen: s.alarmSeen.includes(s.seat), step: s.alarmStep[s.seat], total: s.report?.alarms?.[s.seat]?.length };
    })).toEqual({ seen: true, step: 2, total: 2 });
    await endPlanning(page);
    await expect(page.getByTestId('turn-number')).toHaveText('3');
  });

  test('war and peace show one leader moment at a time, in priority order', async ({ page }) => {
    const setup = siegeFixture({ approach: true });
    await loadFixture(page, coreFixture(setup.fixture), 2);
    const view = await page.evaluate(() => {
      const s = (window as unknown as GameWindow).legatum.getState();
      return s.views![s.seat];
    }) as PlayerView;
    const relation = (a: number, b: number, state: Relation['state'], declarers: number[] = []): Relation => ({
      a, b, state, effective_turn: view.turn, guarantee_until: view.turn + 10, version: 2,
      security: null, war_id: 2, declarers,
    });
    await page.evaluate((war) => {
      const s = (window as unknown as GameWindow).legatum.getState();
      s.set({ views: s.views!.map((v) => ({ ...v, diplomacy: { ...v.diplomacy,
        relations: [...v.diplomacy.relations.filter((r) => !(r.a === war.a && r.b === war.b)), war] } })) });
    }, relation(0, 1, 'War'));
    const events: Event[] = [
      { type: 'PeaceEstablished', relation: relation(0, 2, 'Peace') },
      { type: 'WarDeclared', relation: relation(0, 1, 'War', [1]) },
    ];
    const alarms = alarmsForTurn(events, view, view);
    expect(alarms.map((a) => a.kind)).toEqual(['war', 'peace']);
    await page.evaluate((cards) => {
      const s = (window as unknown as GameWindow).legatum.getState();
      const report = s.report ?? { turn: s.views![s.seat].turn, lines: [], events: [], hash: 'tour' };
      s.set({ report: { ...report, alarms: s.views!.map((_, i) => i === s.seat ? cards : []) }, alarmSeen: [], alarmStep: {}, reportOpen: false, techOpen: false });
    }, alarms);
    const modal = page.getByTestId('alarm-screen');
    for (const [kind, label] of [['war', 'war-declared'], ['peace', 'peace-signed']] as const) {
      await expect(modal).toHaveAttribute('data-alarm', kind);
      await expect(modal.locator('.alarm-leader')).toHaveCount(1);
      await expect(modal.getByRole('heading', { level: 3, name: 'What it means now' })).toBeVisible();
      if (kind === 'peace') await expect(modal).toContainText('Your war with');
      await page.mouse.move(1, 330);
      await shot(page, size, `32-${label}`);
      await expect(modal).toHaveAttribute('data-armed', '');
      await page.keyboard.press('Enter');
    }
    await expect(modal).toHaveCount(0);
    expect(await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().alarmStep[0])).toBe(2);
  });

  test('your declaration, Alliance changes and answered calls each get a leader moment', async ({ page }) => {
    const setup = siegeFixture({ approach: true });
    await loadFixture(page, coreFixture(setup.fixture), 2);
    const view = await page.evaluate(() => {
      const s = (window as unknown as GameWindow).legatum.getState();
      return s.views![s.seat];
    }) as PlayerView;
    const relation = (a: number, b: number, state: Relation['state'], declarers: number[] = []): Relation => ({
      a, b, state, effective_turn: view.turn, guarantee_until: view.turn + 10, version: 2,
      security: null, war_id: 2, declarers,
    });
    const call = (id: number, joined: boolean): Event => ({ type: 'CallAnswered', joined, effective_turn: view.turn,
      call: { id, team: 2, ally: 0, target: 1, created_turn: view.turn - 1, respond_by: view.turn, alliance_version: 1, war_version: 1 } });
    const cases: { kind: string; label: string; event: Event }[] = [
      { kind: 'warByYou', label: 'war-declared-by-you', event: { type: 'WarDeclared', relation: relation(0, 1, 'War', [0]) } },
      { kind: 'callJoined', label: 'call-joined', event: call(8, true) },
      { kind: 'allianceEnded', label: 'alliance-ended', event: { type: 'TreatyEnded', treaty_id: 9, a: 0, b: 2, effective_turn: view.turn, reason: 'Alliance ended' } },
      { kind: 'alliance', label: 'alliance-formed', event: { type: 'TreatyActivated', relation: relation(0, 2, 'Alliance') } },
      { kind: 'callDeclined', label: 'call-declined', event: call(7, false) },
    ];
    const modal = page.getByTestId('alarm-screen');
    for (const { kind, label, event } of cases) {
      const alarms = alarmsForTurn([event], view, view);
      expect(alarms.map((a) => a.kind)).toEqual([kind]);
      await page.evaluate((cards) => {
        const s = (window as unknown as GameWindow).legatum.getState();
        const report = s.report ?? { turn: s.views![s.seat].turn, lines: [], events: [], hash: 'tour' };
        s.set({ report: { ...report, alarms: s.views!.map((_, i) => i === s.seat ? cards : []) }, alarmSeen: [], alarmStep: {}, reportOpen: false, techOpen: false });
      }, alarms);
      await expect(modal).toHaveAttribute('data-alarm', kind);
      await expect(modal.locator('.alarm-leader')).toHaveCount(1);
      await expect(modal.getByRole('heading', { level: 3, name: 'What it means now' })).toBeVisible();
      if (kind === 'warByYou') {
        await expect(modal.locator('.alarm-primary')).toContainText('Continue');
        await expect(modal.locator('.alarm-diplomacy', { hasText: 'Show on map' })).not.toHaveClass(/primary/);
        await expect(modal).toContainText('No treaty stood between you, so the war began at once and the attacks you ordered took place this turn.');
      }
      if (kind === 'alliance') {
        await expect(modal.locator('.alarm-fact.bad')).toContainText('Each empire pays 1 Gold a turn for every city it owns, and never less than 1 Gold.');
      }
      if (kind === 'allianceEnded') {
        await expect(modal.locator('.alarm-title')).toContainText('Alliance ended');
        expect((await modal.innerText()).match(/\bended\b/gi)).toHaveLength(1);
      }
      await page.mouse.move(1, 330);
      await shot(page, size, `33-${label}`);
      await expect(modal).toHaveAttribute('data-armed', '');
      await page.keyboard.press('Escape');
      await expect(modal).toHaveCount(0);
    }
  });

  test('Map animations Off shows an interstitial at once', async ({ page }) => {
    await page.addInitScript(() => localStorage.setItem('legatum.combat-fx', JSON.stringify({ enabled: false })));
    const setup = siegeFixture({ approach: true });
    await loadFixture(page, coreFixture(setup.fixture), 2);
    const view = await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().views![0]) as PlayerView;
    const event: Event = { type: 'WarDeclared', relation: { a: 0, b: 1, state: 'War', effective_turn: view.turn,
      guarantee_until: view.turn + 10, version: 2, security: null, war_id: 2, declarers: [0] } };
    const alarms = alarmsForTurn([event], view, view);
    await page.evaluate((cards) => {
      const s = (window as unknown as GameWindow).legatum.getState();
      const report = s.report ?? { turn: s.views![s.seat].turn, lines: [], events: [], hash: 'tour' };
      s.set({ report: { ...report, alarms: s.views!.map((_, i) => i === s.seat ? cards : []) }, alarmSeen: [], alarmStep: {}, reportOpen: false, techOpen: false });
    }, alarms);
    const modal = page.getByTestId('alarm-screen');
    await expect(modal).toHaveClass(/reduced/);
    for (const selector of ['.alarm-sheet', '.alarm-banner', '.alarm-crest', '.alarm-rays', '.alarm-ribbon']) {
      await expect.poll(() => modal.locator(selector).first().evaluate((el) => getComputedStyle(el).animationName)).toBe('none');
    }
  });
});
