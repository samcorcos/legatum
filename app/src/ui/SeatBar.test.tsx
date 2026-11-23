// @vitest-environment happy-dom
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { useGame } from '../store';
import { coopVsAiSetup } from '../gameSetup';
import { FixtureClient } from '../test/fixtureEngine';
import { readyAndPass, confirmEndTurn, requestReady, SeatBar } from './SeatBar';
import { turnFlowFixture } from '../test/turnFlowFixture';
import { skipSelected, sleepSelected } from '../commands';
import { decodeOrders } from '../orderDecoding';
import { wildState } from '../test/goalActionsFixture';
import { empireTitle } from '../empires';

let root: Root;
let host: HTMLDivElement;
beforeEach(async () => {
  vi.useFakeTimers();
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  const client = new FixtureClient();
  await client.newGame(coopVsAiSetup('42', [
    { name: 'Ada', faction: 'India' }, { name: 'Bo', faction: 'Byzantium' }, { name: 'Cy', faction: 'Celts' },
  ]));
  client.debugState().players[1].alive = false;
  useGame.setState(useGame.getInitialState(), true);
  useGame.setState({ screen: 'game', client, humans: [0, 1, 2], views: [0, 1, 2].map((id) => client.view(id)),
    ready: [false, false, false], orders: [[], [], []], errors: [[], [], []],
  });
  host = document.body.appendChild(document.createElement('div'));
  root = createRoot(host);
  act(() => root.render(<SeatBar />));
});
afterEach(() => {
  act(() => { root.unmount(); useGame.getState().stopResolving(); });
  host.remove();
  vi.clearAllTimers();
  vi.useRealTimers();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

it('renders an unknown queued order without blanking the page', () => {
  act(() => useGame.setState({ orders: [[{ type: 'FutureOrder', unit: 3 } as never], [], []] }));
  expect(host.querySelector('.orders-list')?.textContent).toContain('Unknown order (FutureOrder)');
  expect(host.querySelector('.orders-toggle')).not.toBeNull();
});

it('labels retired Hold and Heal orders from the turn-200 tape after decoding', () => {
  const orders = decodeOrders([{ type: 'HoldUnit', unit: 999 }, { type: 'HealUnit', unit: 998 }]);
  act(() => useGame.setState({ orders: [orders, [], []] }));
  const labels = [...host.querySelectorAll('.order-item span')].map((row) => row.textContent);
  expect(labels).toContain('Unit 999: waiting until woken');
  expect(labels).toContain('Unit 998: waiting until woken');
});

function soloFixture() {
  const client = new FixtureClient(); client.load(JSON.stringify(turnFlowFixture()));
  act(() => useGame.setState({ client, views: [client.view(0)], humans: [0], seat: 0, ready: [false], orders: [[]], errors: [[]] }));
  return client;
}

it('a broken unit order leads the CTA even when a faction action first becomes ready', () => {
  const client = new FixtureClient(); client.load(JSON.stringify(wildState()));
  const view = client.view(0), unit = view.units.find((u) => u.owner === view.me)!;
  act(() => useGame.setState({ client, views: [view], humans: [0], seat: 0, ready: [false],
    orders: [[{ type: 'SleepUnit', unit: unit.id }]], errors: [[{ index: 0, message: 'Invalid order' }]], selection: { kind: 'unit', id: unit.id } }));
  expect(host.querySelector('.cta-label')?.textContent).toBe('Fix order');
});

it('the incoming-offer primary action and End Turn review open the same inbox', () => {
  const client = soloFixture();
  const view = client.view(0);
  view.diplomacy.relations.find((r) => r.a === view.team && r.b === 1)!.state = 'War';
  view.diplomacy.proposals = [{ id: 30, from: 1, to: view.team, proposer: 1, kind: 'Peace',
    created_turn: view.turn, respond_by: view.turn + 2, relation_version: 0, required_teams: [0, 1] }];
  act(() => useGame.setState({ views: [view], selection: { kind: 'unit', id: 0 } }));
  expect(host.querySelector('.cta-label')?.textContent).toBe('Respond to offer');
  expect(host.querySelector('.cta-sub')?.textContent).toContain(`Reply by T${view.turn + 2}`);
  act(() => host.querySelector<HTMLButtonElement>('.cta')!.click());
  expect(useGame.getState()).toMatchObject({ diplomacyOpen: true, diplomacyTeam: 1, ready: [false] });
  act(() => { useGame.setState({ diplomacyOpen: false }); readyAndPass(); });
  expect(host.querySelector('dialog li')?.textContent).toBe(`Peace with ${empireTitle(view, 1)}: ends only your war with ${empireTitle(view, 1)} · respond by turn ${view.turn + 2}`);
  act(() => host.querySelector<HTMLButtonElement>('dialog .primary')!.click());
  expect(useGame.getState()).toMatchObject({ diplomacyOpen: true, diplomacyTeam: 1, endTurnConfirmation: null, ready: [false] });
  act(() => useGame.setState({ orders: [[{ type: 'RespondToProposal', proposal: 30, accept: true }]] }));
  expect(host.querySelector('.cta-label')?.textContent).not.toBe('Respond to offer');
});

it('touch turn controls omit Return hints for both pending decisions and a completed plan', () => {
  const client = soloFixture();
  act(() => root.unmount());
  vi.spyOn(window, 'matchMedia').mockImplementation((query) => Object.assign(new EventTarget(), { matches: query === '(pointer: coarse)' }) as MediaQueryList);
  root = createRoot(host);
  act(() => root.render(<SeatBar />));
  expect(host.querySelector('.cta-label')?.textContent).toContain('units need orders');
  expect(host.querySelector('.cta kbd')).toBeNull();
  expect(host.querySelector('.ready-anyway kbd')).toBeNull();
  const city = client.view(0).cities[0];
  act(() => useGame.getState().editOrders(() => [
    { type: 'SkipUnit', unit: 0 }, { type: 'SkipUnit', unit: 4 },
    { type: 'SetProduction', city: city.id, item: { kind: 'Unit', unit: 'Spearman' } },
    { type: 'SetResearch', tech: 'BronzeWorking' },
  ]));
  expect(host.querySelector('.cta-label')?.textContent).toBe('End turn');
  expect(host.querySelector('.cta kbd')).toBeNull();
  expect(host.querySelector('.touch-turn-controls')?.textContent).not.toContain('Return');
});

it('Ready anyway lists all pending decisions, focuses navigation, and offers one-click confirmation', () => {
  soloFixture();
  const turn = useGame.getState().views![0].turn;
  act(() => host.querySelector<HTMLButtonElement>('.ready-anyway')!.click());
  const dialog = host.querySelector<HTMLDialogElement>('dialog')!;
  expect(dialog.open).toBe(true);
  expect(dialog.textContent).toContain('Spearman #0: needs orders');
  expect(dialog.textContent).toContain('Scout #4: needs orders');
  expect(dialog.textContent).toMatch(/Choose what .+ builds/);
  expect(dialog.textContent).toContain('Choose research');
  // A waiting queue isn't empty: the dialog words each decision instead of claiming so.
  expect(dialog.textContent).not.toContain('will stay empty');
  // The primary button names the decision it leads to.
  expect(document.activeElement?.textContent).toBe('Give the Spearman orders');
  expect(dialog.querySelectorAll('.end-turn-actions button')).toHaveLength(2);
  expect(useGame.getState().ready).toEqual([false]);
  expect(useGame.getState().views![0].turn).toBe(turn);
  const confirm = [...dialog.querySelectorAll('button')].find((b) => b.textContent === 'End turn anyway')!;
  act(() => confirm.click());
  expect(useGame.getState().endTurnConfirmation).toBeNull();
  expect(useGame.getState().busy).toBe('Resolving turn…');
  act(() => vi.advanceTimersByTime(40));
  expect(useGame.getState().views![0].turn).toBe(turn + 1);
});

it('the primary review button selects and centers the first unit without ending the turn', () => {
  soloFixture();
  act(readyAndPass);
  act(() => [...host.querySelectorAll('dialog button')].find((b) => b.textContent === 'Give the Spearman orders')!.dispatchEvent(new MouseEvent('click', { bubbles: true })));
  expect(useGame.getState()).toMatchObject({ selection: { kind: 'unit', id: 0 }, ready: [false], endTurnConfirmation: null });
  expect(useGame.getState().focus?.hex).toEqual(useGame.getState().views![0].units.find((u) => u.id === 0)!.pos);
});

it('stale confirmations cannot ready an edited plan or a new view', () => {
  soloFixture(); act(readyAndPass);
  act(() => useGame.getState().editOrders(() => [{ type: 'SkipUnit', unit: 0 }]));
  act(confirmEndTurn);
  expect(useGame.getState().ready).toEqual([false]);
  act(readyAndPass);
  act(() => useGame.setState({ views: [structuredClone(useGame.getState().views![0])] }));
  act(confirmEndTurn);
  expect(useGame.getState().ready).toEqual([false]);
});

it('seat readiness buttons use the same review, and unready does not need confirmation', () => {
  act(() => requestReady(2));
  expect(useGame.getState().endTurnConfirmation?.seat).toBe(2);
  expect(useGame.getState().ready[2]).toBe(false);
  act(confirmEndTurn);
  expect(useGame.getState().ready[2]).toBe(true);
  act(() => requestReady(2));
  expect(useGame.getState().ready[2]).toBe(false);
  expect(useGame.getState().endTurnConfirmation).toBeNull();
});

it('Space skip returns next turn while a sleeping unit stays out of the cycle', () => {
  soloFixture();
  act(() => { useGame.getState().select({ kind: 'unit', id: 0 }); skipSelected(); });
  act(() => { useGame.getState().select({ kind: 'unit', id: 4 }); sleepSelected(); });
  act(() => vi.advanceTimersByTime(120));
  expect(useGame.getState().selection).toBeNull();
  act(() => { readyAndPass(); confirmEndTurn(); });
  act(() => vi.advanceTimersByTime(40));
  act(() => useGame.getState().nextDecision(true));
  expect(useGame.getState().selection).toEqual({ kind: 'unit', id: 0 });
  act(() => useGame.getState().nextDecision());
  expect(useGame.getState().selection?.kind).toBe('city');
});

it('offers empty observer turns and a working Stop control when all humans are dead', () => {
  act(() => useGame.setState({ views: useGame.getState().views!.map((v) => ({ ...v,
    players: v.players.map((p) => ({ ...p, alive: p.is_human ? false : p.alive })),
  })) }));
  expect(host.querySelector('.cta-label')?.textContent).toBe('End turn');
  expect(host.textContent).not.toContain('Choose research');
  const find = (label: string) => [...host.querySelectorAll('button')].find((b) => b.textContent === label)!;
  act(() => find('Resolve to the end').click());
  expect(useGame.getState().resolvingToEnd).toBe(true);
  act(() => find('Stop').click());
  expect(useGame.getState().resolvingToEnd).toBe(false);
  expect(find('Resolve to the end')).toBeDefined();
});

it('an online seat with living teammates observes without planning or a Resolve action', () => {
  const s = useGame.getState();
  const client = { ...s.client!, kind: 'remote' } as unknown as typeof s.client;
  const view = s.views![1];
  const seats = view.players.filter((p) => p.is_human).map((p, seat) => ({ seat, player: p.id, name: p.name,
    faction: p.faction, connected: true, ready: false, orders: 0,
  }));
  act(() => useGame.setState({ client, views: [view], humans: [1], seat: 0, ready: [true], orders: [[]], errors: [[]],
    net: { mySeat: 1, seats, conn: 'open', status: { mode: 'server', has_game: true, seats, turn: 1, is_local: false } },
  }));
  expect(host.querySelector('.cta-label')?.textContent).toBe('Observing');
  expect(host.querySelector<HTMLButtonElement>('.cta')?.disabled).toBe(true);
  expect(host.textContent).not.toContain('Choose research');
  expect(host.textContent).not.toContain('Resolve to the end');
  expect(host.querySelector('.cta-sub')?.textContent).toContain('Waiting for Ada, Cy');
  act(() => useGame.setState({ views: [{ ...view, players: view.players.map((p) => ({ ...p, alive: p.is_human ? false : p.alive })) }] }));
  expect(host.querySelector('.cta-sub')?.textContent).toContain('Ada controls the remaining turns');
});


it('counts the selected last unit and city until their decisions are actually handled', () => {
  const client = soloFixture();
  act(() => useGame.getState().select({ kind: 'unit', id: 4 }));
  const label = () => host.querySelector('.cta-label')?.textContent;
  const click = () => act(() => host.querySelector<HTMLButtonElement>('.cta')!.click());
  expect(label()).toBe('2 units need orders');
  act(() => useGame.getState().editOrders(() => [{ type: 'SkipUnit', unit: 0 }]));
  expect(label()).toBe('1 unit needs orders');
  click();
  expect(useGame.getState().selection).toEqual({ kind: 'unit', id: 4 });
  act(skipSelected);
  expect(label()).toBe('Choose production');
  click();
  const city = client.view(0).cities[0];
  expect(useGame.getState().selection).toEqual({ kind: 'city', id: city.id });
  expect(label()).toBe('Choose production');
  click();
  expect(useGame.getState().selection).toEqual({ kind: 'city', id: city.id });
  expect(useGame.getState().techOpen).toBe(false);
  act(() => useGame.getState().editOrders((orders) => [...orders,
    { type: 'SetProduction', city: city.id, item: { kind: 'Unit', unit: 'Spearman' } }]));
  expect(label()).toBe('Choose research');
  act(() => useGame.getState().editOrders((orders) => [...orders, { type: 'SetResearch', tech: 'BronzeWorking' }]));
  expect(label()).toBe('End turn');
  expect(host.querySelector('.cta kbd')?.textContent).toBe('Return');
  click();
  expect(useGame.getState().endTurnConfirmation).toBeNull();
  act(() => vi.advanceTimersByTime(40));
  expect(useGame.getState().views![0].turn).toBe(3);
});

it('the phone Next button goes where its label says, even with the last unit selected', () => {
  const client = soloFixture();
  act(() => root.unmount());
  vi.spyOn(window, 'matchMedia').mockReturnValue(Object.assign(new EventTarget(), { matches: true }) as MediaQueryList);
  root = createRoot(host);
  act(() => root.render(<SeatBar />));
  const next = () => host.querySelector<HTMLButtonElement>('.phone-turn-dock button[aria-label^="Next decision"]')!;
  // The Scout (id 4) is the last unit in the cycle; what follows it is the city, then research.
  act(() => useGame.getState().select({ kind: 'unit', id: 4 }));
  expect(next().textContent).toBe('Next: Unit');
  expect(next().getAttribute('aria-label')).toBe('Next decision: 2 units need orders');
  act(() => next().click());
  expect(useGame.getState().selection).toEqual({ kind: 'unit', id: 0 });
  expect(useGame.getState().techOpen).toBe(false);
  act(() => useGame.getState().editOrders(() => [{ type: 'SkipUnit', unit: 0 }, { type: 'SkipUnit', unit: 4 }]));
  expect(next().textContent).toBe('Next: Build');
  act(() => next().click());
  const city = client.view(0).cities[0];
  expect(useGame.getState().selection).toEqual({ kind: 'city', id: city.id });
  act(() => useGame.getState().editOrders((orders) => [...orders, { type: 'SetProduction', city: city.id, item: { kind: 'Unit', unit: 'Spearman' } }]));
  expect(next().textContent).toBe('Next: Research');
  act(() => next().click());
  expect(useGame.getState().techOpen).toBe(true);
});

it('on phones the turn row steps back while the diplomacy inbox is open (its Accept is the primary)', () => {
  const client = soloFixture();
  const view = client.view(0);
  view.diplomacy.proposals = [{ id: 30, from: 1, to: view.team, proposer: 1, kind: 'Peace', created_turn: view.turn, respond_by: view.turn + 2, relation_version: 0 }];
  act(() => root.unmount());
  vi.spyOn(window, 'matchMedia').mockReturnValue(Object.assign(new EventTarget(), { matches: true }) as MediaQueryList);
  root = createRoot(host);
  act(() => { useGame.setState({ views: [view], selection: null, diplomacyOpen: false }); root.render(<SeatBar />); });
  const primaries = () => [...host.querySelectorAll('.phone-turn-dock .btn.primary')].map((b) => b.textContent);
  expect(primaries()).toEqual(['Next: Offer']);
  act(() => useGame.setState({ diplomacyOpen: true }));
  expect(primaries()).toEqual([]);
});

it('with another unit selected, "Treasury runs dry" goes to the endangered unit', () => {
  const v = structuredClone(useGame.getState().views![0]);
  const mine = v.units.filter((u) => u.owner === v.me).sort((a, b) => a.id - b.id);
  // The endangered unit is the first; the selection is the second.
  const [endangered, selected] = mine;
  expect(mine.length).toBeGreaterThan(1);
  v.economy.bankruptcy = { gold: { total: -1, parts: [['Unit upkeep', -1]] }, disband: { unit: endangered.id, kind: endangered.kind, pos: endangered.pos, city: null } };
  act(() => { useGame.setState({ views: [v, ...useGame.getState().views!.slice(1)], seat: 0 }); useGame.getState().select({ kind: 'unit', id: selected.id }); });
  expect(host.querySelector('.cta-label')?.textContent).toBe('Treasury runs dry');
  act(() => host.querySelector<HTMLButtonElement>('.seatbar .cta')!.click());
  expect(useGame.getState().selection).toEqual({ kind: 'unit', id: endangered.id });
});
