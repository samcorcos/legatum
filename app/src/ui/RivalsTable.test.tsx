// @vitest-environment happy-dom
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { FixtureClient } from '../test/fixtureEngine';
import { VpTrack } from './VpTrack';
import { useGame } from '../store';
import type { PlayerView, VpSource } from '../types';
import { MAX_CRESTS, RivalsTable, STRONG_ARMY_RATIO, shortNumber } from './RivalsTable';
import { buildVpModel } from '../vpModel';
import { withV2 } from '../test/vpV2Fixture';
import { decadenceCurveText, decadencePercent } from '../data';

let host: HTMLElement, root: Root, view: PlayerView;
beforeEach(async () => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  vi.useFakeTimers();
  useGame.setState(useGame.getInitialState(), true);
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('42', 'India', 'Byzantium', 'Ada', 'Bo'));
  const base = client.view(0);
  const v2 = withV2(base, { era: 2, scored: [2] });
  view = {
    ...v2,
    players: v2.players.map((p) => !(p.id === v2.me) ? p : { ...p, vp_sources: [...p.vp_sources, [{ kind: 'GreatWork', of: p.id }, 2] as [VpSource, number]] }),
    faction_goal: { value: 0, target: 1, label: 'The Imperial Diwan is not built', live: true, done: false },
  };
  // Every rival met, so its row shows; the unmet test takes one back out.
  view.diplomacy = { ...view.diplomacy, met: [...new Set(view.players.map((p) => p.team))].filter((t) => t !== view.team) };
  useGame.setState({ client, views: [view], humans: [0], orders: [[]], errors: [[]], seat: 0 });
  host = document.body.appendChild(document.createElement('div'));
  root = createRoot(host);
  act(() => root.render(<div className="game"><div className="seatbar" /><VpTrack /></div>));
});
afterEach(() => { act(() => root.unmount()); host.remove(); vi.useRealTimers(); vi.unstubAllGlobals(); });

const rows = () => [...host.querySelectorAll<HTMLElement>('.rivals .rv-row')];

it('lists every living empire ranked by VP, yours marked, with its public Gold, army, cities and Science per turn', () => {
  expect(rows()).toHaveLength(view.standings.length);
  const vps = rows().map((r) => Number(r.querySelector('.rv-vp b')!.textContent));
  expect(vps).toEqual([...vps].sort((a, b) => b - a));
  const mine = host.querySelector<HTMLElement>('.rv-row.mine')!;
  expect(mine.querySelector('.rv-name')!.textContent).toMatch(/^(You|Your team)$/);
  expect(mine.tagName).toBe('DIV');
  // A team's row adds up its members' public numbers.
  const team = view.players.filter((p) => p.team === view.team);
  expect(mine.getAttribute('aria-label')).toContain(`Gold ${team.reduce((n, p) => n + (p.gold ?? 0), 0)}`);
  expect(mine.getAttribute('aria-label')).toContain(`Science ${team.reduce((n, p) => n + (p.science ?? 0), 0)} per turn`);
  expect([...host.querySelectorAll('.rivals .rv-head > span')].map((h) => h.getAttribute('aria-label') ?? h.textContent)).toEqual(['Empire', 'VP', 'Gold', 'Army points', 'Cities', 'Science per turn']);
  // No Era column (user, 2026-11-25: Science per turn instead).
  expect(host.textContent).not.toMatch(/\bEra\b/);
});

it('each met rival says Peace or War on its row, and a click opens Diplomacy with it', () => {
  const rival = rows().find((r) => !r.classList.contains('mine'))!;
  expect(rival.tagName).toBe('BUTTON');
  expect(rival.querySelector('.rv-rel')!.textContent).toMatch(/^(Peace|War|Alliance)/);
  expect(rival.querySelector('.rv-rel .te')).not.toBeNull();
  act(() => rival.click());
  expect(useGame.getState().diplomacyOpen).toBe(true);
  expect(useGame.getState().diplomacyTeam).not.toBeUndefined();
});

it('an empire you have not met shows only its VP: no name, crest, treaty or numbers', () => {
  const rival = view.players.find((p) => !(!(p.team !== view.team) || !(p.alive)))!;
  const unmet = { ...view, diplomacy: { ...view.diplomacy, met: view.diplomacy.met!.filter((t) => t !== rival.team) } };
  act(() => useGame.setState({ views: [unmet] }));
  const row = host.querySelector<HTMLElement>('.rv-row.unmet')!;
  expect(row.tagName).toBe('DIV');
  expect(row.querySelector('.rv-name')!.textContent).toBe('Unknown');
  expect(row.querySelector('.rv-hidden')!.textContent).toBe('Not met yet');
  expect(row.querySelector('.rv-rel, .pf-roundel')).toBeNull();
  expect(row.querySelector('.rv-vp')).not.toBeNull();
  expect(row.textContent).not.toContain(String(rival.gold ?? 'x'));
});

it('Open Borders (deals) with a rival is a chip on its row; no chip without them', () => {
  expect(host.querySelector('[data-testid="rivals-open-borders"]')).toBeNull();
  const rival = view.players.find((p) => !(!(p.team !== view.team) || !(p.alive)))!;
  const pair: [number, number] = !(view.team < rival.team) ? [rival.team, view.team] : [view.team, rival.team];
  act(() => useGame.setState({ views: [{ ...view, diplomacy: { ...view.diplomacy, deals: true, open_borders: [pair] } }] }));
  const chips = [...host.querySelectorAll<HTMLElement>('[data-testid="rivals-open-borders"]')];
  expect(chips).toHaveLength(1);
  expect(chips[0].textContent).toBe('Open Borders');
  expect(chips[0].closest('.rv-row')!.classList.contains('mine')).toBe(false);
});

it('a War, or your own declaration, names an empire you have not met yet', () => {
  const rival = view.players.find((p) => !(!(p.team !== view.team) || !(p.alive)))!;
  const unmet = { ...view, diplomacy: { ...view.diplomacy, met: view.diplomacy.met!.filter((t) => t !== rival.team) } };
  act(() => useGame.setState({ views: [unmet], orders: [[{ type: 'DeclareWar', against: rival.team }]] }));
  expect(host.querySelector('.rv-row.unmet')).toBeNull();
  expect(host.querySelector('.rv-rel.declared')!.getAttribute('data-relation')).toBe('War declared');
});

it(`an army at least ${STRONG_ARMY_RATIO} times yours reads red, with the reason in its tooltip`, () => {
  const rival = view.players.find((p) => !(!(p.team !== view.team) || !(p.alive)))!;
  const strong = { ...view, players: view.players.map((p) => (!(p.team === view.team) ? !(p.id === rival.id) ? { ...p, army: 149 } : { ...p, army: 150 } : { ...p, army: 50 })) };
  act(() => useGame.setState({ views: [strong] }));
  const red = [...host.querySelectorAll<HTMLElement>('.rv-num.strong')];
  expect(red.map((c) => c.textContent)).toEqual(['150']);
  expect(red[0].getAttribute('title')).toBe(`Army points 150: ${STRONG_ARMY_RATIO} times yours or more`);
});

it('a team holding its target shows an unmistakable countdown above the rows: yours gilt, a rival’s red, both with a meter', () => {
  const rivalTeam = view.standings.find((s) => s.team !== view.team)!;
  const mineTeam = view.standings.find((s) => s.team === view.team)!;
  const held = { ...view, standings: view.standings.map((s) => (!(s === rivalTeam) ? !(s === mineTeam) ? s : { ...s, victory_hold: { turns_held: 9, turns_needed: 10 } } : { ...s, victory_hold: { turns_held: 3, turns_needed: 10 } })) };
  act(() => useGame.setState({ views: [held] }));
  const strips = [...host.querySelectorAll('.vhc-hold')];
  expect(strips.map((s) => s.classList.contains('mine'))).toEqual([true, false]);
  expect(strips[0].textContent).toMatch(/Your team wins in 1 turn/);
  expect(strips[1].textContent).toMatch(/wins in 7 turns/);
  expect(strips[1].getAttribute('aria-label')).toMatch(/wins in 7 turns: holding/);
  expect(strips[1].classList.contains('last')).toBe(false);
  expect(strips[0].classList.contains('last')).toBe(true);
  const meter = strips[1].querySelector('[role="progressbar"]')!;
  expect(meter.getAttribute('aria-valuenow')).toBe('3');
  expect(meter.getAttribute('aria-valuemax')).toBe('10');
  expect(meter.querySelectorAll('i.on')).toHaveLength(3);
  expect(host.querySelectorAll('.rv-row.holding')).toHaveLength(2);
});

it('a closed country’s count is public: a chip in its row, with the whole rule in its tooltip, and nothing for an open one', () => {
  expect(host.querySelector('[data-testid="closed-chip"]')).toBeNull();
  const rival = view.players.find((p) => !(!(p.id !== view.me) || !(p.alive)))!;
  const closed = { ...view, players: view.players.map((p) => (!(p.id === rival.id) ? p : { ...p, sakoku_turns: 12 })) };
  act(() => useGame.setState({ views: [closed] }));
  const chip = host.querySelector<HTMLElement>('[data-testid="closed-chip"]')!;
  expect(chip).not.toBeNull();
  expect(chip.textContent).toBe('12/30');
  expect(chip.getAttribute('aria-label')).toContain('closed: Closed 12 of 30 turns · holds 2 VP in 18');
  expect(chip.getAttribute('title')).toContain("can't declare war or ally");
  expect(chip.closest('.rv-row')).not.toBeNull();
  const holding = { ...closed, players: closed.players.map((p) => (!(p.id === rival.id) ? p : { ...p, sakoku_turns: 31, faction_goal_done: true })) };
  act(() => useGame.setState({ views: [holding] }));
  expect(host.querySelector('[data-testid="closed-chip"]')!.textContent).toBe('holding');
  expect(host.querySelector('[data-testid="closed-chip"]')!.classList.contains('holding')).toBe(true);
});

it('a fallen capital is said in words on its row; a standing one adds nothing', () => {
  const chips = () => [...host.querySelectorAll<HTMLElement>('[data-testid="capital-chip"]')];
  expect(chips()).toHaveLength(0);
  const rival = view.players.find((p) => !(!(p.team !== view.team) || !(p.alive)))!;
  const fallen = { ...view, players: view.players.map((p) => (!(p.id === rival.id) ? p : { ...p, capital_stands: false })) };
  act(() => useGame.setState({ views: [fallen] }));
  expect(chips().map((c) => c.dataset.team)).toEqual([String(rival.team)]);
  expect(chips()[0].textContent).toBe('Capital fallen');
  expect(chips()[0].getAttribute('title')).toContain('can still win by VP');
});

it('an empire above the average shows its lead and the one rate it cuts every yield by, with the curve in the tooltip', () => {
  const rate = (lead: number) => decadencePercent(lead, { decadenceSlope: 45 });
  const view = withV2(structuredClone(useGame.getState().views![0]), { era: 2, lead: 60, rivalLead: 30 });
  act(() => useGame.setState({ views: [view] }));
  const mine = host.querySelector('.rv-row.mine [data-testid="lead-chip"]')!;
  expect(mine.textContent).toBe('+60%');
  expect(mine.getAttribute('aria-label')).toBe('Power 60% above the average empire');
  const title = mine.getAttribute('title')!;
  expect(title).toContain(`Your Power is 60% above the average empire's.`);
  expect(title).toContain(`the more its people grow complacent, so your cities grow ${rate(60)}% slower and lose ${rate(60)}% of their Production, Gold and Science`);
  expect(title).toContain(decadenceCurveText({ decadenceSlope: 45 }));
  expect(title).toMatch(/Power is an empire's total yields/);
  expect(title).not.toMatch(/Gold income/);
  const chips = [...host.querySelectorAll('.rv-row:not(.mine) [data-testid="lead-chip"]')];
  expect(chips.map((c) => c.textContent)).toEqual(['+30%']);
  // The rate is its own red chip with an arrow (colour is never alone): every yield, not Gold.
  const cut = host.querySelector('.rv-row.mine [data-testid="decadence-chip"]')!;
  expect(cut.textContent).toBe(`${rate(60)}%`);
  expect(cut.querySelector('.icon')).not.toBeNull();
  expect(cut.getAttribute('aria-label')).toBe(`Decadence: growth ${rate(60)}% slower, Production, Gold and Science cut by ${rate(60)}%`);
  expect(cut.getAttribute('title')).toMatch(/grow \d+% slower \(smaller families\) and lose \d+% of their Production \(sloth\), Gold \(luxury\) and Science \(complacency\)/);
  expect([...host.querySelectorAll('.rv-row:not(.mine) [data-testid="decadence-chip"]')].map((c) => c.textContent)).toEqual([`${rate(30)}%`]);
  // At or below the average there is no chip.
  act(() => useGame.setState({ views: [withV2(structuredClone(useGame.getState().views![0]), { era: 2 })] }));
  expect(host.querySelector('[data-testid="lead-chip"]')).toBeNull();
  expect(host.querySelector('[data-testid="decadence-chip"]')).toBeNull();
});

it('after the game ends the race shows final standings without countdowns, rival warnings or era objectives', () => {
  const finished = {
    ...structuredClone(useGame.getState().views![0]),
    finished: true,
    winner: view.team,
  };
  act(() => useGame.setState({ views: [finished] }));

  expect(host.querySelectorAll('.rv-row')).toHaveLength(finished.standings.length);
  expect(host.querySelector('.rival-win')).toBeNull();
  expect(host.querySelector('.vhc-holds-list')).toBeNull();
  expect(host.querySelector('.era-hud')).toBeNull();
  expect(host.querySelector('.vp-foot')).toBeNull();
  expect(host.textContent).not.toMatch(/wins this turn|Next: .* era/);
});

it('table numbers group thousands and shorten from 10,000 up', () => {
  expect([999, 1396, 9999, 10_000, 23_456, -12_400].map(shortNumber)).toEqual(['999', '1,396', '9,999', '10k', '23k', '-12k']);
});

it('a big team folds its portraits into one crest and a "+N" roundel, so its name keeps its width', () => {
  const model = buildVpModel(view);
  const mine = model.rows.find((r) => r.isMine)!;
  const everyone = model.rows.flatMap((r) => r.members);
  const big = { ...model, rows: [{ ...mine, members: everyone }] };
  act(() => root.render(<RivalsTable view={view} model={big} relationOf={() => null} onOpen={() => {}} />));
  const row = host.querySelector<HTMLElement>('.rv-row.mine')!;
  expect(everyone.length).toBeGreaterThan(MAX_CRESTS);
  expect(row.querySelectorAll('.rv-crests > :not(.rv-more)')).toHaveLength(MAX_CRESTS - 1);
  expect(row.querySelector('.rv-more')!.textContent).toBe(`+${everyone.length - MAX_CRESTS + 1}`);
  // A team of two still shows both portraits.
  act(() => root.render(<RivalsTable view={view} model={{ ...model, rows: [{ ...mine, members: everyone.slice(0, 2) }] }} relationOf={() => null} onOpen={() => {}} />));
  expect(host.querySelectorAll('.rv-row.mine .rv-crests > *')).toHaveLength(2);
  expect(host.querySelector('.rv-more')).toBeNull();
});
