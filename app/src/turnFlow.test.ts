import { beforeEach, expect, it, vi } from 'vitest';
import { cancelWarPrompt, sleepSelected } from './commands';
import { pendingDecisions, nextDecision } from './turnFlow';
import { FixtureClient } from './test/fixtureEngine';
import { turnFlowFixture } from './test/turnFlowFixture';
import { describeOrder, unitsNeedingOrders } from './orders';
import type { Event, Order, PlayerView } from './types';
import { useGame } from './store';
import { queueDiplomacy } from './diplomacy';
import { releasedGarrison } from './releasedGarrison';
import { contextFromViews, releasedGarrisonLines } from './format';

let client: FixtureClient;
let view: PlayerView;
beforeEach(() => {
  client = new FixtureClient(); client.load(JSON.stringify(turnFlowFixture())); view = client.view(0);
  useGame.setState(useGame.getInitialState(), true);
  useGame.setState({ client, humans: [0], views: [view], screen: 'game', seat: 0, orders: [[]], errors: [[]], ready: [false] });
});

it('starts a new game with capital production as the first decision', async () => {
  const fresh = new FixtureClient();
  await fresh.newGame(await fresh.standardSetup('4545', 'Rome', 'India', 'Ada', 'Bo'));
  const opening = fresh.view(0);
  const decisions = pendingDecisions(opening, []);
  expect(decisions[0].kind).toBe('city');
  expect(decisions[0]).toMatchObject({ id: opening.cities.find((c) => c.owner === opening.me)!.id });
  expect(nextDecision(decisions, decisions[0])?.kind).toBe('unit');
  expect(opening.units.filter((u) => u.owner === opening.me && u.kind === 'Settler')).toHaveLength(0);
});

it('cycles every pending unit, then idle cities, then research, and wraps without readying', () => {
  const list = pendingDecisions(view, []);
  expect(list.map((d) => d.kind)).toEqual(['unit', 'unit', 'city', 'research']);
  let current = nextDecision(list, null);
  expect(current).toBe(list[0]);
  for (const decision of [...list.slice(1), list[0]]) {
    current = nextDecision(list, current); expect(current).toBe(decision);
  }
  const s = useGame.getState();
  for (const decision of list) {
    s.nextDecision();
    const after = useGame.getState();
    if (decision.kind === 'research') expect(after.techOpen).toBe(true);
    else if (!('id' in decision)) throw new Error('fixture has no treasury decision');
    else {
      expect(after.selection).toEqual({ kind: decision.kind, id: decision.id });
      const item = decision.kind === 'unit' ? view.units.find((u) => u.id === decision.id) : view.cities.find((c) => c.id === decision.id);
      expect(after.focus?.hex).toEqual(item!.pos);
    }
    expect(after.ready).toEqual([false]); expect(after.views![0].turn).toBe(view.turn);
  }
});

it('names a unit whose Heal just finished in the queue, from the last resolution\'s notice', () => {
  const unit = view.units.find((u) => u.owner === view.me && u.kind === 'Spearman')!;
  unit.hp = 100; unit.needs_orders = true;
  const unitLabel = (local?: Parameters<typeof pendingDecisions>[3]) => pendingDecisions(view, [], [], local).find((d) => d.kind === 'unit' && d.id === unit.id)!.label;
  expect(unitLabel()).toMatch(/: needs orders$/);
  const healed: Event = { type: 'UnitHealed', unit: unit.id, owner: view.me, unit_kind: 'Spearman', at: unit.pos };
  const local = { humans: [0], orders: [[]], errors: [[]], seat: 0, report: { events: [[healed]] } };
  expect(unitLabel(local)).toMatch(/: healed to full, needs orders$/);
  // Another unit's notice, another seat's notice and a missing report never rename it.
  expect(unitLabel({ ...local, report: { events: [[{ ...healed, unit: unit.id + 99 }]] } })).toMatch(/: needs orders$/);
  expect(unitLabel({ ...local, humans: [0, 1], seat: 1, report: { events: [[healed], []] } })).toMatch(/: needs orders$/);
  expect(unitLabel({ ...local, report: null })).toMatch(/: needs orders$/);
  // Return (the next decision) lands on it, and the store passes its own report through.
  useGame.setState({ report: { turn: view.turn, lines: [[]], events: [[healed]], hash: '' } });
  useGame.getState().nextDecision(true);
  expect(useGame.getState().selection).toEqual({ kind: 'unit', id: expect.any(Number) });
  const decisions = pendingDecisions(view, [], [], useGame.getState());
  expect(decisions.filter((d) => d.label.includes('healed to full')).map((d) => 'id' in d && d.id)).toEqual([unit.id]);
});

it('returns a garrison the occupation lock held to the queue once, when its city is secure', () => {
  const city = view.cities.find((c) => c.owner === view.me)!;
  const held = view.units.find((u) => u.owner === view.me && u.kind === 'Spearman')!;
  const other = view.units.find((u) => u.owner === view.me && u.id !== held.id)!;
  // Before: held in the city under the lock (the core fortified it). After: the lock lifted, still fortified.
  const before = structuredClone(view); const after = structuredClone(view);
  for (const v of [before, after]) {
    const u = v.units.find((x) => x.id === held.id)!;
    u.pos = city.pos; u.standing_order = 'Sleep'; u.needs_orders = false; u.occupation_lock = v === before ? 1 : null;
  }
  expect(unitsNeedingOrders(after, []).some((u) => u.id === held.id)).toBe(false);
  expect(releasedGarrison(before, after)).toEqual([{ unit: held.id, city: city.name }]);
  const released = releasedGarrison(before, after);
  const local = { humans: [0], orders: [[]], errors: [[]], seat: 0, report: { events: [[]], released: [released] } };
  const queued = (l: Parameters<typeof pendingDecisions>[3], orders: Order[] = []) => pendingDecisions(after, orders, [], l).filter((d) => d.kind === 'unit' && d.id === held.id);
  expect(queued(local)).toEqual([expect.objectContaining({ released: city.name, label: expect.stringMatching(new RegExp(`#${held.id}: ${city.name} is secure, needs orders$`)) })]);
  // Any order answers it: Fortify, Skip this turn or a move.
  for (const answer of [{ type: 'SleepUnit', unit: held.id }, { type: 'SkipUnit', unit: held.id }] as Order[]) expect(queued(local, [answer])).toEqual([]);
  // Only this seat's own released units, and only the turn the lock lifted.
  expect(queued({ ...local, humans: [0, 1], seat: 1, report: { events: [[], []], released: [released, []] } })).toEqual([]);
  expect(queued({ ...local, report: null })).toEqual([]);
  expect(queued({ ...local, report: { events: [[]], released: [[{ unit: other.id, city: city.name }]] } })).toEqual([]);
  // A unit still under the lock, one fortified by choice, and a teammate's unit are never released.
  expect(releasedGarrison(before, before)).toEqual([]);
  expect(releasedGarrison(after, after)).toEqual([]);
  const theirs = structuredClone(after); theirs.units.find((u) => u.id === held.id)!.owner = view.me + 1;
  expect(releasedGarrison(before, theirs)).toEqual([]);
  expect(releasedGarrison(null, after)).toEqual([]);
  // The report line names the city and says it needs orders, in the healed line's tone.
  const lines = releasedGarrisonLines(before, after, contextFromViews(before, after));
  expect(lines).toHaveLength(1);
  expect(lines[0]).toMatchObject({ text: expect.stringMatching(new RegExp(`: ${city.name} is secure\\. Needs orders\\.$`)), tone: 'good', category: 'orders' });
});

it('keeps an auto-firing garrison out of the Return queue', () => {
  const unit = view.units.find((u) => u.owner === view.me)!;
  unit.standing_order = 'AutoFire';
  unit.needs_orders = true;
  expect(unitsNeedingOrders(view, []).some((u) => u.id === unit.id)).toBe(false);
});

it('decision navigation alone does nothing with no remaining decisions and never readies', () => {
  const orders: Order[] = [
    ...view.units.filter((u) => u.owner === 0).map((u): Order => ({ type: 'SkipUnit', unit: u.id })),
    { type: 'SetProduction', city: view.cities[0].id, item: { kind: 'Unit', unit: 'Spearman' } },
    { type: 'SetResearchQueue', techs: ['BronzeWorking'] },
  ];
  useGame.setState({ orders: [orders] });
  expect(pendingDecisions(view, orders)).toEqual([]);
  for (let i = 0; i < 20; i++) useGame.getState().nextDecision();
  expect(useGame.getState()).toMatchObject({ ready: [false], busy: null, selection: null, techOpen: false });
});

it('a broken order is the next decision, ahead of other units, cities and research, from anywhere in the cycle', () => {
  const units = unitsNeedingOrders(view, []);
  const last = units[units.length - 1];
  const orders: Order[] = [{ type: 'RangedAttack', unit: last.id, target: last.pos }];
  const errors = [{ index: 0, message: 'no enemy unit or city to attack there' }];
  const list = pendingDecisions(view, orders, errors);
  const broken = list.find((d) => d.kind === 'unit' && d.id === last.id)!;
  expect(broken.issue).toBeDefined();
  expect(list[0]).not.toBe(broken);
  for (const current of [null, { kind: 'unit' as const, id: units[0].id }, { kind: 'city' as const, id: view.cities[0].id }, { kind: 'research' as const }])
    expect(nextDecision(list, current)).toBe(broken);
});

it('keeps an invalid ranged order on its fix path with the validation reason', () => {
  const unit = view.units.find((u) => u.owner === view.me)!;
  const orders: Order[] = [{ type: 'RangedAttack', unit: unit.id, target: unit.pos }];
  const errors = [{ index: 0, message: 'no enemy unit or city to attack there' }];
  useGame.setState({ orders: [orders], errors: [errors], selection: { kind: 'unit', id: unit.id } });
  const decision = nextDecision(pendingDecisions(view, orders, errors), { kind: 'unit', id: unit.id });
  expect(decision?.label).toContain('Change or remove its order');
  expect(decision).toMatchObject({ kind: 'unit', id: unit.id });
  useGame.getState().nextDecision();
  expect(useGame.getState()).toMatchObject({ selection: { kind: 'unit', id: unit.id }, mode: 'ranged', error: decision?.label });
});

it('an invalid attack on a team at Peace is fixed by the Declare War prompt; cancelling drops the attack', () => {
  const unit = view.units.find((u) => u.owner === view.me)!;
  const rival = view.players.find((p) => p.team !== view.team)!;
  // A rival city in sight (the fixture's view has none): a copy of ours, theirs, next to the unit.
  const city = { ...view.cities[0], id: 99, owner: rival.id, name: 'Epoch', detail: null, pos: { q: unit.pos.q + 1, r: unit.pos.r } };
  view.cities.push(city);
  expect(view.diplomacy.relations.find((r) => r.a === view.team || r.b === view.team)?.state).toBe('Peace');
  const orders: Order[] = [{ type: 'RangedAttack', unit: unit.id, target: city.pos }];
  const errors = [{ index: 0, message: 'you are not at War with its owner; declare War first to attack there' }];
  useGame.setState({ orders: [orders], errors: [errors], selection: { kind: 'unit', id: unit.id } });
  const label = pendingDecisions(view, orders, errors).find((d) => d.kind === 'unit' && d.id === unit.id)?.label;
  expect(label).toMatch(/declare War first to attack there\. Change or remove its order\.$/);
  useGame.getState().nextDecision();
  const team = view.players.find((p) => p.id === city.owner)!.team;
  expect(useGame.getState()).toMatchObject({ mode: 'normal', warPrompt: { unit: unit.id, target: city.pos, team, ranged: true } });
  cancelWarPrompt();
  expect(useGame.getState()).toMatchObject({ warPrompt: null, orders: [[]] });
});

it('incoming offers lead the decision cycle; only a valid team response clears them', () => {
  const offer = { id: 30, from: 1, to: view.team, proposer: 1, kind: 'Alliance' as const, created_turn: view.turn,
    respond_by: view.turn + 2, relation_version: 0 };
  view.diplomacy.proposals = [offer, { ...offer, id: 31, from: view.team, to: 1, proposer: view.me }];
  const pending = () => pendingDecisions(view, []);
  expect(pending().filter((d) => d.kind === 'diplomacy')).toEqual([
    { kind: 'diplomacy', id: 30, team: 1, label: `Alliance offer: respond by turn ${view.turn + 2}` },
  ]);
  expect(nextDecision(pending(), { kind: 'unit', id: 0 })?.kind).toBe('diplomacy');
  useGame.getState().nextDecision();
  expect(useGame.getState()).toMatchObject({ diplomacyOpen: true, diplomacyTeam: 1, ready: [false] });
  const response: Order = { type: 'RespondToProposal', proposal: 30, accept: true };
  // Named by the empire that sent it, never by the internal offer id.
  expect(describeOrder(response, view)).toMatch(/^Accept Alliance offer from [A-Z][^#]+ · resolves turn \d+$/);
  expect(describeOrder({ type: 'WithdrawProposal', proposal: 31 }, view)).toMatch(/^Withdraw Alliance offer to [A-Z][^#]+ · resolves turn \d+$/);
  expect(pendingDecisions(view, [response])[0].kind).toBe('unit');
  expect(pendingDecisions(view, [response], [{ index: 0, message: 'Invalid' }])[0].kind).toBe('diplomacy');
  view.players.push({ ...view.players[0], id: 9 });
  const local = { humans: [view.me, 9], orders: [[], [response]], errors: [[], []] };
  expect(pendingDecisions(view, [], [], local)[0].kind).toBe('unit');
  expect(pendingDecisions(view, [], [], { ...local, errors: [[], [{ index: 0, message: 'Invalid' }]] })[0].kind).toBe('diplomacy');
});

// Playtest 8.9.2 T20: Switzerland offered an Alliance, he declined, then declared War on it to
// attack. The war replaced the decline (one diplomatic order per empire), so the offer came back as
// "Respond to offer"; answering it replaced the war, and his attacks failed. A war or an Alliance left
// toward the sender ends the offer at resolution, so it is no longer a decision.
it('a queued war or exit toward the sender settles its offer, and declining keeps the war', () => {
  const offer = { id: 30, from: 1, to: view.team, proposer: 1, kind: 'Alliance' as const, created_turn: view.turn,
    respond_by: view.turn + 2, relation_version: 0 };
  view.diplomacy.proposals = [offer];
  const offers = (orders: Order[]) => pendingDecisions(view, orders).filter((d) => d.kind === 'diplomacy');
  const decline: Order = { type: 'RespondToProposal', proposal: 30, accept: false };
  const war: Order = { type: 'DeclareWar', against: 1, breach: false };
  const attack: Order = { type: 'MoveUnit', unit: 7, to: { q: 3, r: 3 } };
  expect(offers([])).toHaveLength(1);
  expect(offers([decline])).toEqual([]);
  // Decline, then declare War: the war replaces the reply and still settles the offer.
  const declared = queueDiplomacy([decline], war, view);
  expect(declared).toEqual([war]);
  expect(offers([...declared, attack])).toEqual([]);
  // Answering the offer again (inbox, A/D keys) must never discard the war or the attacks.
  expect(queueDiplomacy([war, attack], decline, view)).toEqual([war, attack]);
  for (const o of [{ type: 'BreakTreaty', with: 1, agreement: 'Security' }] as Order[]) expect(offers([o])).toEqual([]);
  // A war on someone else, or an invalid war, leaves the offer open.
  expect(offers([{ type: 'DeclareWar', against: 2, breach: false }])).toHaveLength(1);
  expect(pendingDecisions(view, [war], [{ index: 0, message: 'Invalid' }]).filter((d) => d.kind === 'diplomacy')).toHaveLength(1);
  // Accepting is still an explicit choice that replaces the war.
  const accept: Order = { type: 'RespondToProposal', proposal: 30, accept: true };
  expect(queueDiplomacy([war, attack], accept, view)).toEqual([attack, accept]);
});

it('treats a blocked-only queue as a pending city decision until a valid build is chosen', () => {
  const city = view.cities.find((c) => c.owner === view.me)!;
  city.population = 1;
  city.detail!.producing = null;
  city.detail!.production_queue = [{ kind: 'Unit', unit: 'Settler' }];
  // Say what the queue waits for; it isn't empty, and "choose a build" would be wrong.
  const decision = { kind: 'city', id: city.id, label: `${city.name}: queue waiting (Settler needs population 2)` };
  expect(pendingDecisions(view, [])).toContainEqual(decision);
  const orders: Order[] = [{ type: 'SetProductionQueue', city: city.id, queue: [{ kind: 'Unit', unit: 'Spearman' }, ...city.detail!.production_queue] }];
  expect(pendingDecisions(view, orders)).not.toContainEqual(decision);
  expect(pendingDecisions(view, orders, [{ index: 0, message: 'Invalid' }])).toContainEqual(decision);
  expect(pendingDecisions(view, [])).toContainEqual(decision);
  city.population = 2;
  city.detail!.production_queue = [{ kind: 'Unit', unit: 'Spearman' }];
  expect(pendingDecisions(view, [])).toContainEqual({ ...decision, label: `${city.name}: queue waiting (Spearman has no free spawn tile)` });
  city.detail!.production_queue = [];
  expect(pendingDecisions(view, [])).toContainEqual({ ...decision, label: `Choose what ${city.name} builds` });
});

it('invalid production, research and unit orders cannot hide decisions; a cleared queue needs attention', () => {
  const orders: Order[] = [
    { type: 'SleepUnit', unit: 0 },
    { type: 'SetProduction', city: view.cities[0].id, item: { kind: 'Unit', unit: 'Pikeman' } },
    { type: 'SetResearchQueue', techs: ['Feudalism'] },
  ];
  expect(pendingDecisions(view, orders, orders.map((_, index) => ({ index, message: 'Invalid' })))).toHaveLength(4);
  view.research.current = 'BronzeWorking'; view.research.queue = ['BronzeWorking'];
  expect(pendingDecisions(view, []).some((d) => d.kind === 'research')).toBe(false);
  expect(pendingDecisions(view, [{ type: 'SetResearchQueue', techs: [] }]).at(-1)?.kind).toBe('research');
});

it('skipped and unordered units return next turn, Sleep persists until another order, and needs_orders overrides automation', () => {
  const orders: Order[] = [{ type: 'SkipUnit', unit: 0 }, { type: 'SleepUnit', unit: 4 }];
  expect(unitsNeedingOrders(view, orders)).toHaveLength(0);
  client.endTurn({ 0: orders });
  view = client.view(0);
  expect(unitsNeedingOrders(view, []).map((u) => u.id)).toEqual([0]);
  expect(view.units.find((u) => u.id === 0)!.fortify).toBeGreaterThan(0);
  client.endTurn({});
  view = client.view(0);
  expect(unitsNeedingOrders(view, []).map((u) => u.id)).toEqual([0]);
  client.endTurn({ 0: [{ type: 'SkipUnit', unit: 4 }] });
  view = client.view(0);
  expect(unitsNeedingOrders(view, []).map((u) => u.id)).toEqual([0, 4]);
  const scout = view.units.find((u) => u.id === 4)!;
  scout.automation = 'Explore'; scout.needs_orders = true;
  expect(unitsNeedingOrders(view, []).map((u) => u.id)).toEqual([0, 4]);
});

it('a quick Return after an order is not undone by the delayed auto-advance', () => {
  vi.useFakeTimers();
  try {
    const s = useGame.getState();
    s.nextDecision(); s.nextDecision(); // the second unit, the last one needing orders
    sleepSelected();
    useGame.getState().nextDecision(); // Return within 120 ms: the idle city
    const city = useGame.getState().selection;
    expect(city?.kind).toBe('city');
    vi.advanceTimersByTime(200);
    expect(useGame.getState().selection).toBe(city);
  } finally { vi.useRealTimers(); }
});


it('a forecast disband is a pending decision until production or upkeep changes', () => {
  const city = view.cities.find((c) => c.owner === view.me && c.detail)!;
  const v = structuredClone(view);
  const c = v.cities.find((x) => x.id === city.id)!;
  c.detail!.producing = { kind: 'Unit', unit: 'Spearman' }; c.detail!.production_queue = [];
  v.economy.bankruptcy = { gold: { total: -12, parts: [['Unit upkeep', -12]] }, disband: { unit: null, kind: 'Spearman', pos: c.pos, city: c.id } };
  const treasury = pendingDecisions(v, []).find((d) => d.kind === 'treasury');
  expect(treasury).toMatchObject({ target: { kind: 'city', id: c.id } });
  expect(treasury!.label).toBe(`Treasury -12 next turn: Spearman (completing in ${c.name}) will be disbanded; change production or upkeep`);
  // It comes first after diplomacy, and Return opens the city whose build causes it.
  expect(pendingDecisions(v, [])[0].kind).toBe('treasury');
  useGame.setState({ views: [v] });
  useGame.getState().nextDecision();
  expect(useGame.getState().selection).toEqual({ kind: 'city', id: c.id });
  // Changing that city's build, or disbanding a unit, deals with it.
  expect(pendingDecisions(v, [{ type: 'SetProduction', city: c.id, item: { kind: 'Building', building: 'Granary' } }]).some((d) => d.kind === 'treasury')).toBe(false);
  const unit = v.units.find((u) => u.owner === v.me)!;
  expect(pendingDecisions(v, [{ type: 'DisbandUnit', unit: unit.id }]).some((d) => d.kind === 'treasury')).toBe(false);
  // An existing unit at risk targets that unit.
  v.economy.bankruptcy.disband = { unit: unit.id, kind: unit.kind, pos: unit.pos, city: null };
  expect(pendingDecisions(v, []).find((d) => d.kind === 'treasury')).toMatchObject({ target: { kind: 'unit', id: unit.id } });
});
