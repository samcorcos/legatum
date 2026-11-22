// @vitest-environment happy-dom
// The turn report can always be found again (Last turn, L), steps back through recent turns, marks
// each line good or bad for you, and a combat line shows where the attack came from.
import { act, StrictMode } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { CelebrationProvider } from '../celebrate/MomentLane';
import { FixtureClient } from '../test/fixtureEngine';
import { pushReport, useGame, type TurnReport as Report } from '../store';
import { contextFromViews, formatEvents } from '../format';
import type { Event, PlayerView } from '../types';
import { SeatBar } from './SeatBar';
import { TurnReport } from './TurnReport';
import { handleGameKey } from '../screens/gameKeyHandler';

let root: Root, host: HTMLDivElement;
let view: PlayerView;

function report(turn: number, events: Event[]): Report {
  return { turn, hash: `t${turn}`, events: [events], moments: [[]], lines: [formatEvents(events, contextFromViews(view, view))] };
}

beforeEach(async () => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  document.body.innerHTML = '';
  host = document.body.appendChild(document.createElement('div'));
  root = createRoot(host);
  document.body.addEventListener('keydown', handleGameKey);
  useGame.setState(useGame.getInitialState(), true);
  const client = new FixtureClient();
  const setup = await client.standardSetup('42', 'India', 'Byzantium', 'Your Dominion', 'Byzantium');
  setup.players[1].is_human = false;
  await client.newGame(setup);
  view = client.view(0);
  const mine = view.units.find((u) => u.owner === 0 && u.kind === 'Soldier')!;
  const target = { q: mine.pos.q + 1, r: mine.pos.r };
  const from = { q: mine.pos.q + 3, r: mine.pos.r };
  const enemy = view.players.find((p) => p.team !== view.team)!.id;
  // Turn 1: quiet. Turn 2: a Byzantium Ranger shot our Soldier from two tiles away.
  const quiet = report(1, [{ type: 'TechResearched', player: 0, tech: 'BronzeWorking' }]);
  const shot = report(2, [{
    type: 'Battle', attacker: 900, attacker_owner: enemy, defender: { kind: 'Unit', unit: mine.id, owner: 0, unit_kind: 'Soldier' }, at: target,
    attacker_at: from, attacker_strength: { total: 15, parts: [] }, defender_strength: { total: 10, parts: [] }, damage_to_attacker: 0,
    damage_to_defender: 35, attacker_roll: 100, defender_roll: 0, attacker_hp_after: 100, defender_hp_after: 65, ranged: true, meeting: false,
  }]);
  useGame.setState({ client, humans: [0], views: [view], screen: 'game', booted: true, seat: 0,
    orders: [[]], errors: [[]], ready: [false], selection: null, mode: 'normal', busy: null, error: null,
    report: shot, reportHistory: pushReport(pushReport([], quiet), shot), reportOpen: false });
});
afterEach(() => {
  act(() => root.unmount());
  document.body.removeEventListener('keydown', handleGameKey);
  useGame.setState(useGame.getInitialState(), true);
  vi.restoreAllMocks(); vi.unstubAllGlobals();
});
const render = () => act(() => root.render(<StrictMode><CelebrationProvider><SeatBar /><TurnReport /></CelebrationProvider></StrictMode>));
const key = (k: string) => act(() => { document.body.dispatchEvent(new KeyboardEvent('keydown', { key: k, bubbles: true })); });
const $ = <T extends Element = HTMLElement>(sel: string) => host.querySelector<T & HTMLElement>(sel);

it('Last turn in This Turn shows how the turn went and reopens the report', () => {
  render();
  expect($('.report')).toBeNull();
  const button = $<HTMLButtonElement>('.seatbar .last-turn')!;
  expect(button.textContent).toContain('Last turn · T2');
  expect(button.querySelector('.chip-count.bad')?.textContent).toBe('1 bad');
  expect(button.querySelector('.chip-count.bad .mask-icon')).not.toBeNull();
  act(() => button.click());
  expect($('.report')?.dataset.reportTurn).toBe('2');
  act(() => button.click());
  expect($('.report')).toBeNull();
});

it('shows the shared insignia on a promotion report line', () => {
  const mine = view.units.find((u) => u.owner === 0 && u.kind === 'Soldier')!;
  const promoted = report(3, [{ type: 'UnitPromoted', unit: mine.id, owner: 0, unit_kind: 'Soldier', level: 4, at: mine.pos }]);
  useGame.setState({ report: promoted, reportHistory: pushReport([], promoted), reportOpen: true });
  render();
  const mark = $('.report .rline-main .rank-insignia');
  expect(mark?.getAttribute('aria-label')).toBe('Level 4');
  expect(mark?.querySelectorAll('g')).toHaveLength(4);
});

it('the L key toggles the latest report and [ ] step through turns', () => {
  render();
  key('l');
  expect($('.report')?.dataset.reportTurn).toBe('2');
  expect($('.report-kicker')?.textContent).toBe('Last turn');
  key('[');
  expect($('.report')?.dataset.reportTurn).toBe('1');
  expect($('.report-kicker')?.textContent).toBe('1 turn ago');
  expect($<HTMLButtonElement>('.report-step[aria-label^="Previous"], .report-step[aria-label="No earlier report"]')!.disabled).toBe(true);
  act(() => $<HTMLButtonElement>('.report-step[aria-label^="Next turn"]')!.click());
  expect($('.report')?.dataset.reportTurn).toBe('2');
  // Closing and reopening always starts on the latest turn.
  key('[');
  key('L');
  key('L');
  expect($('.report')?.dataset.reportTurn).toBe('2');
});

it('marks the line bad with an icon and a signed red HP change; clicking it centres the map and draws the arrow', () => {
  useGame.setState({ reportOpen: true });
  render();
  const line = $('.rline[data-tone="bad"]')!;
  expect(line.querySelector('.rline-tone')?.textContent).toBe('');
  expect(line.querySelector('.rline-tone .rm.bad .rm-sign .mask-icon')).not.toBeNull();
  expect(line.querySelector('.sr-only')?.textContent).toBe('Bad for you: ');
  expect(line.querySelector('.hp-delta.bad')?.textContent).toBe('−35 HP');
  expect(line.textContent).toMatch(/bombarded your Spearman .*, from 2 tiles E: −35 HP \(65 left\)/);
  expect($('.chip[data-category="combat"]')?.textContent).toContain('1 bad');
  act(() => line.querySelector<HTMLButtonElement>('.rline-text')!.click());
  const { focus, reportArrow, report: r } = useGame.getState();
  const battle = r!.events[0][0] as Extract<Event, { type: 'Battle' }>;
  expect(focus?.hex).toEqual(battle.at);
  expect(focus?.clear).toBe(true);
  expect(reportArrow).toMatchObject({ from: battle.attacker_at, to: battle.at });
  // Closing the report removes the arrow.
  act(() => $<HTMLButtonElement>('[aria-label="Close report"]')!.click());
  expect(useGame.getState().reportArrow).toBeNull();
});

it('history keeps the last 20 turns and a replayed turn replaces its entry', () => {
  let h: Report[] = [];
  for (let t = 1; t <= 25; t++) h = pushReport(h, report(t, []));
  expect(h.map((r) => r.turn)).toEqual(Array.from({ length: 20 }, (_, i) => i + 6));
  // Loading an earlier save and resolving turn 10 again drops the now-stale later turns.
  h = pushReport(h, report(10, []));
  expect(h.map((r) => r.turn)).toEqual([6, 7, 8, 9, 10]);
});

it('every category header and every line wears a medallion with the line’s own icon; star dividers sit between sections', () => {
  const events: Event[] = [
    { type: 'CityCaptured', city: 1, from: view.players.find((p) => p.team !== view.team)!.id, to: 0 },
    { type: 'TechResearched', player: 0, tech: 'BronzeWorking' },
    { type: 'EraObjectiveScored', player: 0, era: 2, slot: 0, objective: 'Learn 9 technologies' },
  ];
  useGame.setState({ report: report(3, events), reportHistory: pushReport([], report(3, events)), reportOpen: true });
  render();
  const groups = [...host.querySelectorAll('.report-group')];
  expect(groups.length).toBeGreaterThanOrEqual(3);
  for (const g of groups) expect(g.querySelector('.report-cat .rm.neutral')).not.toBeNull();
  // One divider between sections, none before the first.
  expect(groups[0].querySelector('.report-sd')).toBeNull();
  for (const g of groups.slice(1)) expect(g.querySelector('.report-sd')).not.toBeNull();
  for (const line of host.querySelectorAll('.rline')) expect(line.querySelector('.rline-tone .rm')).not.toBeNull();
  // A capture is its own emblem; the tone ring and arrow stay (good for you).
  const capture = host.querySelector('.rline[data-tone="good"] .rm.good')!;
  expect(capture).not.toBeNull();
  expect(capture.querySelector('.rm-sign')).not.toBeNull();
});

it('a World Age line in the report opens the score ledger from its control', () => {
  const rival = view.players.find((p) => p.team !== view.team)!.id;
  const age = report(3, [{ type: 'AgeBegan', era: 2, first: rival }]);
  useGame.setState({ report: age, reportHistory: pushReport([], age), reportOpen: true, vpExpanded: false });
  render();
  const row = [...host.querySelectorAll<HTMLElement>('.rline')].find((r) => r.textContent?.includes('Age began'))!;
  expect(row.textContent).toContain('entered the');
  expect(row.textContent).toContain('Its card is open to every empire.');
  const links = [...row.querySelectorAll<HTMLButtonElement>('.rline-show-card')];
  expect(links.map((b) => b.textContent)).toEqual(['Show score check', 'See how you win']);
  act(() => links[1].click());
  expect(useGame.getState().vpExpanded).toBe(true);
  // Every player can see the Age's score check again, not only the empire that opened it.
  act(() => links[0].click());
  expect(useGame.getState().eraReopen).toEqual({ era: 2, first: false, age: rival });
});
