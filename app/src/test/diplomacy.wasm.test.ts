import { newDiplomacy, NOT_MET } from './fixtureEngine';
import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeAll, describe, expect, it } from 'vitest';
import { FixtureClient } from './fixtureEngine';
import { WasmClient } from '../wasmClient';
import { createState, makeStandardSetup, meetAll, spawnUnit } from './fixtureEngine';
import { exploreStep } from './fixtureEngine';
import { coreFixture } from './coreFixture';
import type { Event, Order, PlayerView } from '../types';
import { fromOffset, within } from '../hex';
import { idxOf, refreshVision } from './fixtureEngine';
import type { MState } from './fixtureState';
import { contextFromViews, formatEvents } from '../format';

const available = existsSync(fileURLToPath(new URL('../wasm-pkg/legatum.js', import.meta.url)));
let makeCore: () => WasmClient;
let planExplore: (view: PlayerView, unit: number) => Order;
const offer = (to: number): Order => ({ type: 'ProposeTreaty', to, kind: 'Alliance' });
const accept = (proposal: number): Order => ({ type: 'RespondToProposal', proposal, accept: true });
const leave = (to: number): Order => ({ type: 'BreakTreaty', with: to, agreement: 'Security' });
const war = (against: number, breach = false): Order => ({ type: 'DeclareWar', against, breach });
const answer = (proposal: number, accept: boolean): Order => ({ type: 'RespondToProposal', proposal, accept });
const isWar = (client: { view(p: number): PlayerView }, a: number, b: number) => client.view(0).diplomacy.relations.find((r) => r.a === Math.min(a, b) && r.b === Math.max(a, b))?.state === 'War';
const relevant = (es: Event[]) => es.filter((e) => ['ProposalCreated', 'ProposalAccepted', 'ProposalRejected', 'ProposalWithdrawn', 'ProposalExpired', 'ProposalInvalidated', 'OrderSuperseded', 'TreatyActivated', 'TreatyEnded', 'TreatyBreached', 'TreatyPayment', 'AllianceBetrayed', 'WarDeclared', 'WarJoined', 'CallToArms', 'CallAnswered', 'CallLapsed', 'PeaceEstablished', 'TileClaimed', 'ItemBought'].includes(e.type));
function fixture(cities = false, arrange?: (s: MState) => void, contact = true) {
  const setup = makeStandardSetup('9002', 'Germany', 'India', 'Ada', 'Bo'); setup.width = setup.height = 24;
  setup.players = setup.players.slice(0, 4).map((p, id) => ({ ...p, is_human: true, team: id === 3 ? 0 : id }));
  // Treaty accounting has a fixed roster, independent of the lobby's rotating AI choices.
  setup.players[2].faction = 'Byzantium'; setup.players[3].faction = 'Germany';
  const s = createState(setup); s.terrain.fill('Grassland'); s.ruin.fill(0);
  const starts = s.players.map((p) => s.cities.find((c) => c.owner === p.id)!.pos);
  s.cities = []; s.nextCityId = 0; s.owner.fill(-1); s.tileCity.fill(-1);
  for (const p of s.players) spawnUnit(s, p.id, 'Settler', starts[p.id], false);
  for (const p of s.players) p.gold = 300;
  if (cities) {
    s.cities = [0, 1].map((owner) => ({ id: owner + 1, owner, founder: owner, name: `C${owner}`, pos: fromOffset(4 + owner * 12, 6), population: 1, food: 0, prod: 0, producing: null, bought: null, buildings: [], hp: 200, grownTiles: [], tilesClaimed: 0, isCapital: true, foundedTurn: 1, resistance: 0 }));
    s.nextCityId = 2;
    for (const c of s.cities) for (const h of within(c.pos, 1)) { const i = idxOf(s, h); s.owner[i] = c.owner; s.tileCity[i] = c.id; }
    refreshVision(s);
  }
  arrange?.(s);
  // Treaty rules are tested apart from first contact unless a case asks for strangers.
  if (contact) meetAll(s);
  const fixtureClient = new FixtureClient(); fixtureClient.load(JSON.stringify(s)); const core = makeCore(); core.load(coreFixture(s));
  return { fixtureClient, core };
}
function step(pair: ReturnType<typeof fixture>, orders: Record<number, Order[]>) {
  pair.fixtureClient.endTurn(orders);
  pair.core.endTurn(orders);
  pair.fixtureClient.load(pair.fixtureClient.save()); pair.core.load(pair.core.save());
}
describe.skipIf(!available)('WASM diplomacy behavior', () => {
  beforeAll(async () => {
    const path = '../wasm-pkg/legatum.js'; const mod = await import(/* @vite-ignore */ path);
    await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) }); makeCore = () => new WasmClient(mod.Game);
    planExplore = (view, unit) => JSON.parse(mod.explore_step(JSON.stringify(view), unit));
  });
  it('validates and resolves a ranged city attack with a same-turn surprise war', () => {
    const target = fromOffset(16, 6);
    let ranger = 0;
    const p = fixture(true, (s) => {
      ranger = spawnUnit(s, 0, 'Archer', fromOffset(14, 6), false).id;
      refreshVision(s);
    });
    const shot: Order = { type: 'RangedAttack', unit: ranger, target };
    const orders = [war(1), shot];
    for (const client of [p.fixtureClient, p.core]) {
      expect(client.validate(0, [shot])).toHaveLength(1);
      expect(client.validate(0, orders)).toEqual([]);
    }
    const reports = [p.fixtureClient, p.core].map((client) => client.endTurn({ 0: orders }));
    for (const report of reports) expect(report.events.some((e) => e.type === 'Battle' && e.attacker === ranger && e.defender.kind === 'City')).toBe(true);
  });
  it('rejects an Alliance between empires that have not met, then allows it once they have', () => {
    const p = fixture(false, (s) => { s.diplomacy!.contacts = []; }, false);
    const order: Order = { type: 'ProposeTreaty', to: 1, kind: 'Alliance' };
    for (const client of [p.fixtureClient, p.core]) {
      expect(client.validate(0, [order]).map((e) => e.message)).toEqual([NOT_MET]);
      expect(client.view(0).diplomacy.actions.find((a) => JSON.stringify(a.order) === JSON.stringify(order))?.blocked_reason).toBe(NOT_MET);
      expect(client.view(0).diplomacy.met).toEqual([]);
      // Peace and War need no contact.
      expect(client.validate(0, [war(1)])).toEqual([]);
    }
    // A unit seen is contact for both teams, recorded at the end of the turn.
    const met = fixture(false, (s) => { const at = s.units.find((u) => u.owner === 0)!.pos; spawnUnit(s, 1, 'Scout', { q: at.q + 1, r: at.r }, false); refreshVision(s); s.diplomacy!.contacts = []; }, false);
    met.fixtureClient.endTurn({}); met.core.endTurn({});
    for (const client of [met.fixtureClient, met.core]) {
      expect(client.view(0).diplomacy.met).toEqual([1]);
      expect(client.view(1).diplomacy.met).toEqual([0]);
      expect(client.validate(0, [offer(1)])).toEqual([]);
    }
  });
  it('an incoming Alliance offer from an empire not met cannot be accepted', () => {
    const p = fixture(false, (s) => {
      s.diplomacy!.contacts = [];
      s.diplomacy!.next_proposal_id = 1;
      s.diplomacy!.proposals = [{ id: 1, kind: 'Alliance', from: 1, to: 0, proposer: 1, created_turn: 1, respond_by: 3, relation_version: 0 }];
    }, false);
    for (const client of [p.fixtureClient, p.core]) {
      expect(client.validate(0, [accept(1)])).toEqual([{ index: 0, message: NOT_MET }]);
      expect(client.validate(0, [{ type: 'RespondToProposal', proposal: 1, accept: false }])).toEqual([]);
    }
  });
  it('reports Alliance activation with the same core and fixtureClient terms', () => {
    const p = fixture();
    step(p, { 0: [offer(1)] });
    const reports = [p.fixtureClient, p.core].map((client) => {
      const before = client.view(0);
      const result = client.endTurn({ 1: [accept(1)] });
      const activation = result.events.find((e) => e.type === 'TreatyActivated');
      expect(activation).toMatchObject({ relation: { state: 'Alliance', guarantee_until: 0, security: { protected_until: before.turn + 1 + 10 } } });
      return formatEvents(result.events_by_player![0], contextFromViews(before, client.view(0)));
    });
    expect(reports[0]).toEqual(reports[1]);
  });
  it('sorts several same-player offers and declarations independently of submission order', () => {
    const p = fixture(false, (s) => { s.players[3].team = 3; s.setup.players[3].team = 3;
      s.diplomacy = newDiplomacy([0, 1, 2, 3]); });
    step(p, { 0: [offer(3), offer(2), offer(1)] });
    expect(p.core.view(0).diplomacy.proposals.map((p) => [p.id, p.to])).toEqual([[1, 1], [2, 2], [3, 3]]);
    step(p, { 0: [{ type: 'WithdrawProposal', proposal: 3 }, { type: 'WithdrawProposal', proposal: 1 }, { type: 'WithdrawProposal', proposal: 2 }] });
    step(p, { 0: [war(3), war(1), war(2)] });
    expect(p.core.view(0).diplomacy.relations.filter((r) => r.a === 0).map((r) => [r.b, r.war_id])).toEqual([[1, 1], [2, 2], [3, 3]]);
  });
  it('clears stranded fortification at the same boundary that peace closes borders', () => {
    const p = fixture(true, (s) => {
      s.diplomacy!.relations.find((r) => r.a === 0 && r.b === 1)!.state = 'War';
      Object.assign(s.units.find((u) => u.owner === 0 && u.kind === 'Spearman')!, { pos: { ...s.cities[1].pos }, hp: 100, fortify: 6 });
    });
    step(p, { 0: [{ type: 'ProposeTreaty', to: 1, kind: 'Peace' }] });
    step(p, { 1: [accept(1)] });
    for (const client of [p.fixtureClient, p.core]) expect(client.view(0).units.find((u) => u.owner === 0 && u.kind === 'Spearman')?.fortify).toBe(0);
  });
  it('rejects auto-explore across Peace borders in validation and resolution', () => {
    const to = fromOffset(5, 5);
    const p = fixture(false, (s) => {
      Object.assign(s.units.find((u) => u.owner === 0 && u.kind === 'Scout')!, { pos: fromOffset(4, 5) });
      s.owner[idxOf(s, to)] = 1; refreshVision(s);
    });
    const scout = p.core.view(0).units.find((u) => u.owner === 0 && u.kind === 'Scout')!;
    const order: Order = { type: 'SetAutomation', unit: scout.id, automation: 'Explore', path: [to] };
    for (const client of [p.fixtureClient, p.core]) expect(client.validate(0, [order])).toHaveLength(1);
    step(p, { 0: [order] });
    for (const client of [p.fixtureClient, p.core]) expect(client.view(0).units.find((u) => u.id === scout.id)?.pos).toEqual(scout.pos);
  });
  it.each(['Settler', 'Spearman'] as const)('auto-explore avoids a peaceful foreign %s without treating it as a threat', (kind) => {
    const blocked = fromOffset(5, 5);
    const p = fixture(false, (s) => {
      Object.assign(s.units.find((u) => u.owner === 0 && u.kind === 'Scout')!, { pos: fromOffset(4, 5) });
      Object.assign(s.units.find((u) => u.owner === 1 && u.kind === kind)!, { pos: blocked });
      s.ruin[idxOf(s, blocked)] = 1; refreshVision(s);
    });
    const view = p.core.view(0), scout = view.units.find((u) => u.owner === 0 && u.kind === 'Scout')!;
    const planned = planExplore(view, scout.id);
    expect(exploreStep(view, scout.id)).toEqual(planned);
    if (planned.type !== 'SetAutomation') throw new Error('expected automation');
    expect(planned.stop_reason).not.toBe('Threatened');
    expect(planned.path ?? []).not.toContainEqual(blocked);
    step(p, { 0: [planned] });
    (expect(p.fixtureClient.view(0).units.find((u) => u.id === scout.id)?.pos).toMatchSnapshot(), void (p.core.view(0).units.find((u) => u.id === scout.id)?.pos));
  });
  it('closed territory prevents healing and fortification while asleep', () => {
    const p = fixture(true, (s) => {
      const unit = s.units.find((u) => u.owner === 0 && u.kind === 'Spearman')!;
      Object.assign(unit, { pos: { ...s.cities[1].pos }, hp: 50, fortify: 6 });
    });
    const unit = (client: WasmClient | FixtureClient) => client.view(0).units.find((u) => u.owner === 0 && u.kind === 'Spearman')!;
    for (const client of [p.fixtureClient, p.core]) {
      expect(unit(client).resting_heal).toEqual({ total: 0, parts: [['Closed foreign territory', 0]] });
    }
    step(p, { 0: [{ type: 'SleepUnit', unit: unit(p.core).id }] });
    step(p, {});
    for (const client of [p.fixtureClient, p.core]) expect(unit(client)).toMatchObject({ hp: 50, fortify: 0, standing_order: 'Sleep' });
  });
  it('an Alliance opens borders: units heal and move inside the ally’s territory', () => {
    const p = fixture(true, (s) => {
      const r = s.diplomacy!.relations.find((r) => r.a === 0 && r.b === 1)!;
      r.state = 'Alliance'; r.security = { id: 1, started_turn: 1, protected_until: 11 }; s.diplomacy!.next_treaty_id = 1;
      Object.assign(s.units.find((u) => u.owner === 0 && u.kind === 'Spearman')!, { pos: fromOffset(15, 6), hp: 50 });
    });
    const unit = (client: WasmClient | FixtureClient) => client.view(0).units.find((u) => u.owner === 0 && u.kind === 'Spearman')!;
    for (const client of [p.fixtureClient, p.core]) expect(unit(client).resting_heal?.parts).not.toContainEqual(['Closed foreign territory', 0]);
    (expect(unit(p.fixtureClient).resting_heal).toMatchSnapshot(), void (unit(p.core).resting_heal));
    const move: Order = { type: 'MoveUnit', unit: unit(p.core).id, to: fromOffset(16, 7) };
    for (const client of [p.fixtureClient, p.core]) expect(client.validate(0, [move])).toEqual([]);
    step(p, { 0: [move] });
    (expect(unit(p.fixtureClient).pos).toMatchSnapshot(), void (unit(p.core).pos));
  });
  it.each(['fixtureClient', 'core'] as const)('%s explicit attack forecasts reject closed territory with the validation reason', (engine) => {
    const origin = fromOffset(6, 5), target = fromOffset(7, 5);
    for (const owner of [1, null, 0, 2, 3]) {
      let ranger = 0;
      const p = fixture(false, (s) => {
        s.units = s.units.filter((u) => u.kind === 'Settler');
        ranger = spawnUnit(s, 0, 'Archer', origin, false).id;
        spawnUnit(s, 2, 'Spearman', target, false);
        s.owner[idxOf(s, origin)] = owner ?? -1;
        s.owner[idxOf(s, target)] = -1;
        s.diplomacy!.relations.find((r) => r.a === 0 && r.b === 2)!.state = 'War';
        refreshVision(s);
      });
      const client = p[engine], closed = owner === 1;
      for (const mode of ['Melee', 'Ranged'] as const) {
        const forecast = client.forecast(0, ranger, target, mode);
        if (closed) expect(forecast).toBeNull();
        else expect(forecast?.expected_damage_to_defender).toBeGreaterThan(0);
      }
      expect(client.validate(0, [{ type: 'RangedAttack', unit: ranger, target }])).toEqual(closed
        ? [{ index: 0, message: 'Cannot attack from closed foreign territory' }] : []);
    }
  });
  it('keeps pending offers and teammate conflicts identical; leaving after the protected window is free', () => {
    const p = fixture(); step(p, { 0: [offer(1)], 3: [war(1)] }); step(p, { 1: [accept(1)] });
    const until = p.core.view(0).diplomacy.relations.find((r) => r.a === 0 && r.b === 1)!.security!.protected_until;
    (expect(until).toMatchSnapshot(), void (p.core.view(0).turn + 10));
    while (p.fixtureClient.view(0).turn + 1 < until) step(p, {});
    for (const client of [p.fixtureClient, p.core]) {
      const action = client.view(0).diplomacy.actions.find((a) => a.order.type === 'BreakTreaty' && a.order.with === 1)!;
      expect(action).toMatchObject({ cost: 0, blocked_reason: null });
      expect(action.betrayal).toBeUndefined();
    }
    step(p, { 0: [leave(1)] });
    const d = p.core.view(0).diplomacy;
    expect(d.relations.find((r) => r.a === 0 && r.b === 1)?.state).toBe('Peace');
    expect(d.betrayals).toEqual([]);
  });
  it('charges each empire 1 Gold upkeep per owned city after activation', () => {
    const p = fixture();
    step(p, { 0: [offer(1)] });
    const before = [0, 1, 2, 3].map((id) => p.core.view(id).economy);
    const expected = p.core.endTurn({ 1: [accept(1)] });
    const actual = p.fixtureClient.endTurn({ 1: [accept(1)] });
    (expect(relevant(actual.events)).toMatchSnapshot(), void (relevant(expected.events)));
    expect(expected.events.some((e) => e.type === 'TreatyPayment')).toBe(false);
    for (const id of [0, 1, 2, 3]) {
      expect(p.core.view(id).economy.gold).toBe(before[id].gold + before[id].gold_per_turn.total);
      (expect(p.fixtureClient.view(id).economy).toMatchSnapshot(), void (p.core.view(id).economy));
    }
    // Upkeep: one line per Alliance in each member's gold breakdown, and in total upkeep.
    const label = (team: number) => p.core.view(0).players.filter((x) => x.team === team).map((x) => x.name).join(' & ');
    for (const id of [0, 1, 3]) {
      const v = p.core.view(id);
      expect(v.economy.gold_per_turn.parts.filter(([l]) => l.startsWith('Treaty upkeep: '))).toEqual([[`Treaty upkeep: Alliance with ${label(id === 1 ? 0 : 1)}`, -1]]);
      expect(v.economy.upkeep_breakdown?.parts).toContainEqual(['Treaties', 1]);
    }
    const now = [0, 1, 3].map((id) => p.core.view(id).economy);
    step(p, {});
    [0, 1, 3].forEach((id, i) => expect(p.core.view(id).economy.gold).toBe(now[i].gold + now[i].gold_per_turn.total));
  });
  it('breaking an Alliance inside its protected window costs Gold and records a public betrayal', () => {
    const p = fixture(); step(p, { 0: [offer(1)] }); step(p, { 1: [accept(1)] }); step(p, {});
    for (const client of [p.fixtureClient, p.core]) {
      const action = client.view(0).diplomacy.actions.find((a) => a.order.type === 'BreakTreaty' && a.order.with === 1)!;
      expect(action).toMatchObject({ cost: 120, betrayal: true, blocked_reason: null });
    }
    const turn = p.core.view(0).turn + 1, before = p.core.view(0).economy;
    step(p, { 0: [leave(1)] });
    const v = p.core.view(2);
    expect(v.diplomacy.betrayals).toEqual([{ by: 0, against: 1, turn, until: turn + 20 }]);
    // Upkeep is charged with the turn's income, before the exit at the boundary.
    expect(p.core.view(0).economy.gold).toBe(before.gold + before.gold_per_turn.total - 120);
    // The victim refuses the betrayer's Alliance offers until the record expires; not the reverse.
    const team0 = v.players.filter((x) => x.team === 0).map((x) => x.name).join(' & ');
    const reason = `Bo won't ally with ${team0} until turn ${turn + 20}: betrayed on turn ${turn}`;
    for (const client of [p.fixtureClient, p.core]) {
      expect(client.view(0).diplomacy.actions.find((a) => a.order.type === 'ProposeTreaty' && a.order.to === 1 && a.order.kind === 'Alliance')?.blocked_reason).toBe(reason);
      expect(client.validate(0, [offer(1)])).toEqual([{ index: 0, message: reason }]);
      expect(client.validate(1, [offer(0)])).toEqual([]);
    }
  });
  it('war on an ally needs an explicit betrayal, takes effect next turn and is always a betrayal', () => {
    const p = fixture(); step(p, { 0: [offer(1)] }); step(p, { 1: [accept(1)] });
    while (p.core.view(0).turn < 16) step(p, {});
    for (const client of [p.fixtureClient, p.core]) {
      const v = client.view(0);
      expect(v.diplomacy.actions.find((a) => a.order.type === 'DeclareWar' && !a.order.breach && a.order.against === 1)?.blocked_reason)
        .toBe('You are allied with Bo: leave the Alliance first, or declare war on your ally as a betrayal');
      expect(v.diplomacy.actions.find((a) => a.order.type === 'DeclareWar' && a.order.breach && a.order.against === 1)).toMatchObject({ cost: 0, betrayal: true, effective_turn: v.turn + 1 });
    }
    step(p, { 0: [war(1, true)] });
    const d = p.core.view(1).diplomacy;
    expect(d.relations.find((r) => r.a === 0 && r.b === 1)?.state).toBe('War');
    expect(d.betrayals).toHaveLength(1);
  });
  it('population is territory: growth claims a tile at once, in fixtureClient and core alike', () => {
    const p = fixture(true, (s) => { s.cities[1].population = 3; });
    const detail = (client: WasmClient | FixtureClient, id: number) => client.view(id).cities.find((c) => c.owner === id)!.detail!;
    for (const client of [p.fixtureClient, p.core]) {
      expect(detail(client, 0).next_claim).not.toBeNull();
      expect(detail(client, 0).yields.worked).toHaveLength(6);
    }
    (expect(detail(p.fixtureClient, 0)).toMatchSnapshot(), void (detail(p.core, 0)));
    const claims = (client: WasmClient | FixtureClient) => client.endTurn({}).events.filter((e) => e.type === 'TileClaimed').map((e) => e.type === 'TileClaimed' && [e.city, e.tile.q, e.tile.r]);
    const [core, fixtureClient] = [claims(p.core), claims(p.fixtureClient)];
    (expect(fixtureClient).toMatchSnapshot(), void (core));
    // The population-3 city claims two tiles; the population-1 city none.
    expect(core.filter((c) => c && c[0] === 2)).toHaveLength(2);
    expect(core.filter((c) => c && c[0] === 1)).toHaveLength(0);
    (expect(detail(p.fixtureClient, 1)).toMatchSnapshot(), void (detail(p.core, 1)));
  });
  it('an ally that is attacked is defended at once, an ally\'s own war is a call, and leaving keeps the wars', () => {
    const p=fixture(false,(s)=>{s.players[3].team=3;s.setup.players[3].team=3;s.diplomacy=newDiplomacy([0,1,2,3]);});
    step(p,{0:[offer(1)]});
    step(p,{1:[accept(1)]});
    expect(p.core.view(0).diplomacy.relations.find((r)=>r.a===0&&r.b===1)?.state).toBe('Alliance');
    expect(p.core.view(0).diplomacy.alliances).toEqual([[0, 1]]);
    for (const client of [p.fixtureClient, p.core]) {
      const action = client.view(0).diplomacy.actions.find((a) => JSON.stringify(a.order) === JSON.stringify(war(2)))!;
      expect(action.new_wars).toBeUndefined();
      expect(action.calls).toEqual([{ team: 1, enemy: 2 }]);
    }
    step(p,{0:[war(2)]});
    for (const client of [p.fixtureClient, p.core]) {
      expect(isWar(client, 0, 2)).toBe(true);
      expect(isWar(client, 1, 2)).toBe(false);
      expect(client.view(1).diplomacy.calls).toHaveLength(1);
      expect(client.view(0).diplomacy.calls).toHaveLength(0);
    }
    const call = p.core.view(1).diplomacy.calls![0];
    for (const client of [p.fixtureClient, p.core]) {
      const join = client.view(1).diplomacy.actions.find((a) => JSON.stringify(a.order) === JSON.stringify(answer(call.id, true)))!;
      expect(join.blocked_reason).toBeNull();
      expect(join.effective_turn).toBe(client.view(1).turn + 1);
    }
    step(p,{1:[answer(call.id, true)]});
    for (const client of [p.fixtureClient, p.core]) {
      expect(isWar(client, 1, 2)).toBe(true);
      expect(client.view(1).diplomacy.calls).toHaveLength(0);
    }
    step(p,{0:[leave(1)]});
    expect(p.core.view(0).diplomacy.relations.find((r)=>r.a===0&&r.b===1)?.state).toBe('Peace');
    expect(p.core.view(0).diplomacy.relations.find((r)=>r.a===1&&r.b===2)?.state).toBe('War');
  });
  it('a war on an ally brings the other ally in the same turn, and declining a call keeps the Alliance', () => {
    const p=fixture(false,(s)=>{s.players[3].team=3;s.setup.players[3].team=3;s.diplomacy=newDiplomacy([0,1,2,3]);});
    step(p,{0:[offer(1)]});
    step(p,{1:[accept(1)]});
    step(p,{2:[war(0)]});
    for (const client of [p.fixtureClient, p.core]) {
      expect(isWar(client, 0, 2) && isWar(client, 1, 2)).toBe(true);
      expect(client.view(0).diplomacy.calls).toHaveLength(0);
    }
    // 0 declares on 3: 1 is called and declines.
    step(p,{0:[war(3)]});
    const call = p.core.view(1).diplomacy.calls![0];
    expect([call.ally, call.target]).toEqual([0, 3]);
    step(p,{1:[answer(call.id, false)]});
    for (const client of [p.fixtureClient, p.core]) {
      expect(isWar(client, 1, 3)).toBe(false);
      expect(client.view(0).diplomacy.relations.find((r) => r.a === 0 && r.b === 1)?.state).toBe('Alliance');
      expect(client.view(1).diplomacy.calls).toHaveLength(0);
      expect(client.view(0).diplomacy.betrayals).toEqual([]);
    }
  });
  it('a call nobody answers lapses and the Peace stands', () => {
    const p=fixture(false,(s)=>{s.players[3].team=3;s.setup.players[3].team=3;s.diplomacy=newDiplomacy([0,1,2,3]);});
    step(p,{0:[offer(1)]});
    step(p,{1:[accept(1)]});
    step(p,{0:[war(2)]});
    expect(p.core.view(1).diplomacy.calls).toHaveLength(1);
    step(p,{});
    step(p,{});
    for (const client of [p.fixtureClient, p.core]) {
      expect(client.view(1).diplomacy.calls).toHaveLength(0);
      expect(isWar(client, 1, 2)).toBe(false);
    }
  });
  it('signs an Alliance during a war: it joins the wars its partner defends and calls for the ones it declared', () => {
    const p=fixture(false,(s)=>{s.players[3].team=3;s.setup.players[3].team=3;s.diplomacy=newDiplomacy([0,1,2,3]);});
    step(p,{0:[war(2)]});
    step(p,{3:[war(1)]});
    for (const client of [p.fixtureClient, p.core]) {
      const action = client.view(0).diplomacy.actions.find((a) => JSON.stringify(a.order) === JSON.stringify(offer(1)))!;
      expect(action.new_wars).toEqual([{ team: 0, enemy: 3 }]);
      expect(action.calls).toEqual([{ team: 1, enemy: 2 }]);
    }
    step(p,{0:[offer(1)]});
    step(p,{1:[accept(1)]});
    for (const client of [p.fixtureClient, p.core]) {
      expect(isWar(client, 0, 3)).toBe(true);
      expect(isWar(client, 1, 2)).toBe(false);
      expect(client.view(1).diplomacy.calls).toHaveLength(1);
    }
  });
  it.each(['fixtureClient', 'core'] as const)('%s remembers newly allied sight when its source disbands next turn', (engine) => {
    let scout = 0;
    const p = fixture(false, (s) => {
      s.units = s.units.filter((u) => u.kind === 'Settler');
      for (const u of s.units) u.pos = fromOffset(2 + u.owner * 5, 22);
      scout = spawnUnit(s, 1, 'Scout', fromOffset(18, 2), false).id;
      for (const player of s.players) player.explored.fill(0);
      s.knownOwner = {}; s.knownRuins = {}; s.knownCities = {}; s.knownTerrain = {};
      refreshVision(s);
    });
    const native = p[engine].view(0).diplomacy.exploration!.native_land;
    step(p, { 0: [{ type: 'ProposeTreaty', to: 1, kind: 'Alliance' }] });
    step(p, { 1: [accept(1)] });
    const shared = p[engine].view(0).tiles.flatMap((tile, i) => tile.visible ? [i] : []);
    expect(p[engine].view(0).units.some((u) => u.id === scout)).toBe(true);
    step(p, { 1: [{ type: 'DisbandUnit', unit: scout }] });
    const after = p[engine].view(0);
    expect(after.units.some((u) => u.id === scout)).toBe(false);
    expect(shared.filter((i) => !after.tiles[i].visible).length).toBeGreaterThan(0);
    for (const i of shared) expect(after.tiles[i], `forgot shared tile ${i}`).toMatchObject({ explored: true, terrain: 'Grassland' });
    (expect(after.diplomacy.exploration!.native_land).toMatchSnapshot(), void (native));
  });

  it('makes Peace between only the named teams while an ally keeps fighting', () => {
    const p = fixture();
    step(p, { 0: [offer(1)] });
    step(p, { 1: [accept(1)] });
    step(p, { 2: [war(0)] });
    step(p, { 2: [{ type: 'ProposeTreaty', to: 0, kind: 'Peace' }] });
    for (const client of [p.fixtureClient, p.core]) {
      expect(client.view(0).diplomacy.proposals[0].needs).toEqual([0]);
      expect(client.view(0).diplomacy.proposals[0].required_teams).toEqual([0, 2]);
      expect(client.view(1).diplomacy.proposals).toEqual([]);
    }
    step(p, { 0: [accept(2)] });
    for (const client of [p.fixtureClient, p.core]) {
      const rels = client.view(0).diplomacy.relations;
      expect(rels.find((r) => r.a === 0 && r.b === 2)?.state).toBe('Peace');
      expect(rels.find((r) => r.a === 1 && r.b === 2)?.state).toBe('War');
      expect(rels.find((r) => r.a === 0 && r.b === 1)?.state).toBe('Alliance');
      expect(client.view(0).diplomacy.proposals).toEqual([]);
    }
  });
  it('a Peace does not need the ally that declined the war', () => {
    const p = fixture();
    step(p, { 0: [offer(1)] });
    step(p, { 1: [accept(1)] });
    step(p, { 0: [war(2)] });
    const call = p.core.view(1).diplomacy.calls![0];
    step(p, { 1: [answer(call.id, false)] });
    step(p, { 2: [{ type: 'ProposeTreaty', to: 0, kind: 'Peace' }] });
    for (const client of [p.fixtureClient, p.core]) {
      expect(client.view(0).diplomacy.proposals[0].needs).toEqual([0]);
      expect(client.view(1).diplomacy.proposals).toEqual([]);
    }
    step(p, { 0: [accept(3)] });
    for (const client of [p.fixtureClient, p.core]) expect(isWar(client, 0, 2)).toBe(false);
  });

  it('refuses a declaration that an Alliance chain forbids, with the same words', () => {
    const p = fixture();
    step(p, { 0: [offer(1)] });
    step(p, { 1: [accept(1)] });
    step(p, { 1: [offer(2)] });
    step(p, { 2: [accept(2)] });
    for (const client of [p.fixtureClient, p.core]) {
      expect(client.validate(0, [war(2)])).toEqual([{ index: 0, message: expect.stringMatching(/another Alliance/) }]);
    }
  });

  it('keeps borrowed exploration separate and never forwards an ally’s borrowed sight',()=>{
    const p=fixture(false,(s)=>{
      s.players[3].team=3;s.setup.players[3].team=3;s.diplomacy=newDiplomacy([0,1,2,3]);
      s.units=s.units.filter((u)=>u.kind==='Settler');
      for(const u of s.units)u.pos=fromOffset([2,9,16,22][u.owner],4);
      for(const p of s.players)p.explored.fill(0);
      s.knownOwner={};s.knownRuins={};s.knownCities={};s.knownTerrain={};refreshVision(s);
    });
    const native=p.core.view(0).diplomacy.exploration!.native_land;
    step(p,{0:[{type:'ProposeTreaty',to:1,kind:'Alliance'}]});step(p,{1:[accept(1)]});
    step(p,{1:[{type:'ProposeTreaty',to:2,kind:'Alliance'}]});step(p,{2:[accept(2)]});
    for(const client of [p.fixtureClient,p.core]){
      const view=client.view(0);expect(view.units.some((u)=>u.owner===1)).toBe(true);expect(view.units.some((u)=>u.owner===2)).toBe(false);
      (expect(view.diplomacy.exploration!.native_land).toMatchSnapshot(), void (native));
    }
    step(p,{0:[leave(1)]});
    (expect(p.fixtureClient.view(0).tiles).toMatchSnapshot(), void (p.core.view(0).tiles));
    for(const client of [p.fixtureClient,p.core]){expect(client.view(0).units.some((u)=>u.owner===1)).toBe(false);(expect(client.view(0).diplomacy.exploration!.native_land).toMatchSnapshot(), void (native));}
  });

});
