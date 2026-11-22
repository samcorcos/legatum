// Map resources and strategic supply: the same scenarios through the fixtureClient and the real WASM
// core, comparing supply views, blockers, held builds, upgrade budgets, idle users and what they
// lose, resource fog, announcements, border pull and settlement scores.
import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeAll, describe, expect, it } from 'vitest';
import { FACTIONS, TECHS } from '../data';
import { fromOffset, tileIndex, within } from '../hex';
import type { Event, Order, PlayerView, ProductionItem, Resource, TechId, UnitView } from '../types';
import { WasmClient } from '../wasmClient';
import { coreFixture } from './coreFixture';
import { FixtureClient } from './fixtureEngine';
import { idxOf } from './fixtureEngine';
import { settlementSites } from './../settlement';
import { createState, makeStandardSetup, spawnUnit } from './fixtureEngine';
import type { MCity, MState } from './fixtureState';

const available = existsSync(fileURLToPath(new URL('../wasm-pkg/legatum.js', import.meta.url)));
type Game = {
  settlement_sites(view: string, unit: number, reserved: string): string;
  settlement_site(view: string, unit: number, q: number, r: number, reserved: string): string;
  site_blocker(view: string, q: number, r: number): string;
  new(setup: string): { view(p: number): string; free(): void };
  standard_setup(...args: string[]): string;
};
let makeCore: () => WasmClient;
let Game: Game;

const TROOPER: ProductionItem = { kind: 'Unit', unit: 'Pikeman' };
const SOLDIER: ProductionItem = { kind: 'Unit', unit: 'Spearman' };
const WALLS: ProductionItem = { kind: 'Building', building: 'Walls' };
type Turn = Record<number, Order[]>;

/** Grassland, three human empires (P0 and P1 allied), P0 knowing the first 25 techs. P0's city A owns an Iron tile (2 Iron supply). */
function fixture() {
  const setup = makeStandardSetup('700011', 'Athens', 'India', 'P0', 'P1');
  setup.width = setup.height = 20;
  setup.players = setup.players.slice(0, 3).map((p) => ({ ...p, is_human: true }));
  const s = createState(setup);
  delete (s as Partial<MState>).diplomacy;
  s.units = [];
  s.nextUnitId = 0;
  s.terrain.fill('Grassland');
  s.ruin.fill(0);
  s.owner.fill(-1);
  s.tileCity.fill(-1);
  s.resource = s.terrain.map(() => null);
  for (const p of s.players) {
    p.gold = 10_000;
    p.techs = p.id === 0 ? [...new Set<TechId>([...TECHS.slice(0, 25), 'BronzeWorking', 'Feudalism'])] : [];
    p.explored.fill(0);
  }
  s.cities = [[0, 4, 4], [0, 13, 4], [1, 4, 13], [2, 13, 13]].map(([owner, col, row], i): MCity => ({
    id: i + 1, owner, founder: owner, name: ['A', 'B', 'Ally', 'Enemy'][i], pos: fromOffset(col, row),
    population: 3, food: 0, prod: 0, producing: null, bought: null, productionQueue: [],
    buildings: [], hp: 200, grownTiles: [], tilesClaimed: 0, isCapital: i !== 1, foundedTurn: 1, resistance: 0,
  }));
  s.nextCityId = s.cities.length;
  for (const c of s.cities) for (const h of within(c.pos, 1)) {
    s.owner[idxOf(s, h)] = c.owner;
    s.tileCity[idxOf(s, h)] = c.id;
  }
  const [a, b] = s.cities;
  const iron = fromOffset(5, 4);
  put(s, iron, 'Iron');
  return { s, a, b, iron };
}

const put = (s: MState, h: { q: number; r: number }, r: Resource | null) => { s.resource![idxOf(s, h)] = r; };
const supplyOf = (v: PlayerView, r: Resource) => v.supply!.find((x) => x.resource === r)!;
const RELEVANT = ['ResourcesRevealed', 'SupplyShortage', 'OrderFailed', 'ItemBought', 'UnitUpgraded', 'ProductionCompleted', 'TileClaimed'];
const relevant = (events: Event[]) => events.filter((e) => RELEVANT.includes(e.type));

function start(s: MState) {
  const fixtureClient = new FixtureClient();
  fixtureClient.load(JSON.stringify(s));
  const core = makeCore();
  core.load(coreFixture(s));
  return { fixtureClient, core };
}

/** Resolve a turn and return the resource events asserted by the test. */
function turn(fixtureClient: FixtureClient, core: WasmClient, orders: Turn): Event[] {
  const actual = fixtureClient.endTurn(orders);
  core.endTurn(orders);
  fixtureClient.load(fixtureClient.save());
  core.load(core.save());
  return relevant(actual.events);
}

describe.skipIf(!available)('WASM resource behavior', () => {
  beforeAll(async () => {
    const path = '../wasm-pkg/legatum.js';
    const mod = await import(/* @vite-ignore */ path);
    await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
    makeCore = () => new WasmClient(mod.Game);
    Game = mod.Game;
  });

  const option = (v: PlayerView, city: number, unit = 'Pikeman') => v.cities.find((c) => c.id === city)!.detail!.options.find((o) => o.item.kind === 'Unit' && o.item.unit === unit)!;

  it('supply needs the tech and the border, and a Mine adds one', () => {
    const { s, iron } = fixture();
    s.players[0].techs = s.players[0].techs.filter((t) => t !== 'IronWorking' && t !== 'Feudalism');
    let { fixtureClient, core } = start(s);
    turn(fixtureClient, core, {});
    expect(supplyOf(fixtureClient.view(0), 'Iron')).toMatchObject({ revealed: false, supplied: { total: 0 }, spare: 0 });

    s.players[0].techs.push('IronWorking');
    ({ fixtureClient, core } = start(s));
    expect(supplyOf(fixtureClient.view(0), 'Iron')).toMatchObject({ revealed: true, supplied: { total: 2, parts: [['Iron near A', 2]] }, used: { total: 0 }, spare: 2 });
    turn(fixtureClient, core, {});

    // A Mine on the deposit adds one; so does a city built on it.
    (s.improved ??= s.terrain.map(() => 0))[idxOf(s, iron)] = 1;
    ({ fixtureClient, core } = start(s));
    expect(supplyOf(fixtureClient.view(0), 'Iron').supplied).toEqual({ total: 3, parts: [['Iron near A, with a Mine', 3]] });

    // A second deposit in B's territory and one outside every border.
    put(s, fromOffset(12, 4), 'Iron');
    put(s, fromOffset(8, 8), 'Iron');
    ({ fixtureClient, core } = start(s));
    expect(supplyOf(fixtureClient.view(0), 'Iron').supplied).toEqual({ total: 5, parts: [['Iron near A, with a Mine', 3], ['Iron near B', 2]] });
    turn(fixtureClient, core, {});
    s.owner[idxOf(s, iron)] = -1;
    s.tileCity[idxOf(s, iron)] = -1;
    ({ fixtureClient, core } = start(s));
    expect(supplyOf(fixtureClient.view(0), 'Iron').supplied.total).toBe(2);
    turn(fixtureClient, core, {});
  });

  it('a Pikeman starts only with spare Iron and says why not', () => {
    const { s, a, b } = fixture();
    spawnUnit(s, 0, 'Pikeman', fromOffset(3, 3), false);
    spawnUnit(s, 0, 'Pikeman', fromOffset(3, 4), false);
    let { fixtureClient, core } = start(s);
    expect(option(fixtureClient.view(0), a.id).supply_need).toBe('Iron');
    expect(option(fixtureClient.view(0), a.id).resource_blocker).toBe('No Iron to spare: all 2 of your Iron are in use. Bring another Iron deposit inside your borders, or build a Mine on one for +1 Iron.');
    const errors = fixtureClient.validate(0, [{ type: 'SetProduction', city: a.id, item: TROOPER }, { type: 'SetProductionQueue', city: b.id, queue: [TROOPER, SOLDIER] }]);
    expect(errors.map((e) => e.index)).toEqual([0, 1]);
    expect(errors[0].message).toMatch(/^A cannot start Pikeman: No Iron to spare/);
    turn(fixtureClient, core, { 0: [{ type: 'SetProductionQueue', city: b.id, queue: [SOLDIER, TROOPER] }] });

    put(s, fromOffset(5, 4), null);
    ({ fixtureClient, core } = start(s));
    expect(option(fixtureClient.view(0), a.id).resource_blocker).toBe('Needs 1 Iron: you have none yet. Bring an Iron deposit inside your borders, or build a Mine on one for +1 Iron.');
    s.players[0].techs = s.players[0].techs.filter((t) => t !== 'IronWorking');
    ({ fixtureClient, core } = start(s));
    expect(option(fixtureClient.view(0), a.id).resource_blocker).toBe('Needs Iron: Iron Working reveals Iron on the map');
    turn(fixtureClient, core, { 0: [{ type: 'SetProduction', city: a.id, item: TROOPER }] });
  });

  it('a build in progress holds its supply and gives it back when replaced', () => {
    const { s, a, b } = fixture();
    spawnUnit(s, 0, 'Pikeman', fromOffset(3, 3), false);
    let { fixtureClient, core } = start(s);
    // One Iron is spare: A starts a Pikeman and holds it.
    expect(option(fixtureClient.view(0), a.id).resource_blocker).toBeUndefined();
    turn(fixtureClient, core, { 0: [{ type: 'SetProduction', city: a.id, item: TROOPER }] });
    const v = fixtureClient.view(0);
    expect(supplyOf(v, 'Iron')).toMatchObject({ held: ['Pikeman in A'], spare: 0 });
    expect(option(v, b.id).resource_blocker).toBe('No Iron to spare: all 2 of your Iron are in use. 1 is held for the Pikeman being built in A. Bring another Iron deposit inside your borders, or build a Mine on one for +1 Iron.');
    // The city's own build doesn't count against it; replacing it frees the supply.
    expect(option(v, a.id).resource_blocker).toBeUndefined();
    turn(fixtureClient, core, { 0: [{ type: 'SetProduction', city: a.id, item: WALLS }, { type: 'SetProduction', city: b.id, item: TROOPER }] });
    expect(supplyOf(fixtureClient.view(0), 'Iron')).toMatchObject({ held: ['Pikeman in B'], spare: 0 });

    // Finishing turns the hold into a user.
    const done = structuredClone(fixtureClient.debugState());
    done.cities[1].prod = 1000;
    const d = start(done);
    const events = turn(d.fixtureClient, d.core, {});
    expect(events).toContainEqual({ type: 'ProductionCompleted', city: b.id, owner: 0, item: TROOPER });
    expect(supplyOf(d.fixtureClient.view(0), 'Iron')).toMatchObject({ used: { total: 2, parts: [['Pikeman ×2', 2]] }, spare: 0 });
    expect(supplyOf(d.fixtureClient.view(0), 'Iron').held).toBeUndefined();
  });

  it('a queued Pikeman waits for supply and starts when it is spare', () => {
    const { s, a } = fixture();
    const u1 = spawnUnit(s, 0, 'Pikeman', fromOffset(3, 3), false);
    spawnUnit(s, 0, 'Pikeman', fromOffset(3, 4), false);
    let { fixtureClient, core } = start(s);
    turn(fixtureClient, core, { 0: [{ type: 'SetProductionQueue', city: a.id, queue: [SOLDIER, TROOPER] }] });
    const waiting = structuredClone(fixtureClient.debugState());
    waiting.cities[0].prod = 140;
    ({ fixtureClient, core } = start(waiting));
    const events = turn(fixtureClient, core, {});
    expect(events.filter((e) => e.type === 'OrderFailed')).toEqual([
      { type: 'OrderFailed', player: 0, reason: expect.stringMatching(/^A skipped queued Pikeman: No Iron to spare.*; kept in queue$/) },
    ]);
    // Notified once: the next turn keeps waiting silently.
    expect(turn(fixtureClient, core, {}).filter((e) => e.type === 'OrderFailed')).toEqual([]);
    const freed = structuredClone(fixtureClient.debugState());
    freed.units = freed.units.filter((u) => u.id !== u1.id);
    freed.cities[0].prod = 1000;
    ({ fixtureClient, core } = start(freed));
    const resumed = turn(fixtureClient, core, {});
    expect(resumed).toContainEqual(expect.objectContaining({ type: 'ProductionCompleted', item: TROOPER }));
  });

  it('buying a Pikeman needs spare Iron and takes over its own hold', () => {
    const { s, a, b } = fixture();
    spawnUnit(s, 0, 'Pikeman', fromOffset(3, 3), false);
    spawnUnit(s, 0, 'Pikeman', fromOffset(3, 4), false);
    let { fixtureClient, core } = start(s);
    const buy: Order[] = [{ type: 'BuyItem', city: a.id, item: TROOPER }];
    expect(fixtureClient.validate(0, buy)[0].message).toMatch(/^No Iron to spare/);
    turn(fixtureClient, core, { 0: buy });

    // With one spare Iron, B holds it for its build in progress; buying that build is allowed, buying one in A is not.
    put(s, fromOffset(12, 4), 'Iron');
    s.units.pop();
    ({ fixtureClient, core } = start(s));
    turn(fixtureClient, core, { 0: [{ type: 'SetProduction', city: b.id, item: TROOPER }] });
    expect(supplyOf(fixtureClient.view(0), 'Iron').spare).toBe(2);
    const second = structuredClone(fixtureClient.debugState());
    spawnUnit(second, 0, 'Pikeman', fromOffset(3, 5), false);
    spawnUnit(second, 0, 'Pikeman', fromOffset(2, 5), false);
    ({ fixtureClient, core } = start(second));
    expect(supplyOf(fixtureClient.view(0), 'Iron')).toMatchObject({ spare: 0, held: ['Pikeman in B'] });
    expect(fixtureClient.validate(0, buy)[0].message).toMatch(/^No Iron to spare/);
    const buyHeld: Order[] = [{ type: 'BuyItem', city: b.id, item: TROOPER }];
    expect(fixtureClient.validate(0, buyHeld)).toEqual([]);
    (expect(fixtureClient.validate(0, buy)).toMatchSnapshot(), void (core.validate(0, buy)));
    turn(fixtureClient, core, { 0: buyHeld });
  });

  it('Pikeman to Musketman upgrades take Niter in list order', () => {
    const { s, a, iron } = fixture();
    // Rifling would make one order reach Rifleman and require Coal instead.
    s.players[0].techs = s.players[0].techs.filter((tech) => tech !== 'Rifling');
    const [n1, n2] = within(a.pos, 1).filter((h) => !(h.q === a.pos.q && h.r === a.pos.r) && !(h.q === iron.q && h.r === iron.r));
    put(s, n1, 'Niter');
    put(s, n2, 'Iron');
    const u1 = spawnUnit(s, 0, 'Pikeman', fromOffset(4, 3), false);
    const u2 = spawnUnit(s, 0, 'Pikeman', fromOffset(3, 4), false);
    const u3 = spawnUnit(s, 0, 'Pikeman', fromOffset(3, 3), false);
    let { fixtureClient, core } = start(s);
    const unit = (v: PlayerView, id: number) => v.units.find((u) => u.id === id)! as UnitView;
    // One Niter deposit supplies 2: the third upgrade has none to spare.
    expect(unit(fixtureClient.view(0), u1.id).upgrade_need).toBe('Niter');
    const os: Order[] = [{ type: 'UpgradeUnit', unit: u1.id }, { type: 'UpgradeUnit', unit: u2.id }, { type: 'UpgradeUnit', unit: u3.id }];
    const errors = fixtureClient.validate(0, os);
    expect(errors.map((e) => e.index)).toEqual([2]);
    expect(errors[0].message).toMatch(/^upgrading to Musketman: no Niter to spare: all 2 of your Niter are in use\. Bring another Niter deposit/);
    (expect(errors).toMatchSnapshot(), void (core.validate(0, os)));
    // Upgrades take supply before production starts.
    const withStart = [...os.slice(0, 2), { type: 'SetProduction', city: a.id, item: { kind: 'Unit', unit: 'Musketman' } as ProductionItem } as Order];
    (expect(fixtureClient.validate(0, withStart)).toMatchSnapshot(), void (core.validate(0, withStart)));
    turn(fixtureClient, core, { 0: os.slice(0, 2) });
    expect(fixtureClient.view(0).units.find((u) => u.id === u1.id)!.kind).toBe('Musketman');
    // An upgraded unit leaves the Iron and joins the Niter: the Iron it used is spare again.
    expect(supplyOf(fixtureClient.view(0), 'Niter').used.total).toBe(2);
    expect(supplyOf(fixtureClient.view(0), 'Iron').used.total).toBe(1);

    // With no supply left the unit shows no upgrade price and the order says why.
    ({ fixtureClient, core } = start(fixtureClient.debugState()));
    expect(unit(fixtureClient.view(0), u3.id).upgrade_cost).toBeNull();
    expect(fixtureClient.validate(0, os.slice(2))[0].message).toMatch(/^upgrading to Musketman: no Niter to spare/);
    (expect(fixtureClient.validate(0, os.slice(2))).toMatchSnapshot(), void (core.validate(0, os.slice(2))));
  });

  it('a Market needs no Copper and a Bank needs only Niter', () => {
    const { s, a, iron } = fixture();
    const [niter] = within(a.pos, 1).filter((h) => !(h.q === a.pos.q && h.r === a.pos.r) && !(h.q === iron.q && h.r === iron.r));
    put(s, niter, 'Niter');
    a.buildings = ['Market'];
    a.population = 5;
    let { fixtureClient, core } = start(s);
    // No Copper deposit: the Market remains active and the Bank can start on top of it.
    let events = turn(fixtureClient, core, {});
    expect(events.some((e) => e.type === 'SupplyShortage' && e.resource === 'Copper')).toBe(false);
    const bank: Order[] = [{ type: 'SetProduction', city: a.id, item: { kind: 'Building', building: 'Bank' } }];
    expect(fixtureClient.validate(0, bank)).toEqual([]);
    (expect(fixtureClient.validate(0, bank)).toMatchSnapshot(), void (core.validate(0, bank)));
    // A Bank on a Market is supplied by Niter; neither tier uses Copper.
    const withBank = structuredClone(fixtureClient.debugState());
    withBank.cities.find((c) => c.id === a.id)!.buildings = ['Market', 'Bank'];
    ({ fixtureClient, core } = start(withBank));
    events = turn(fixtureClient, core, {});
    const detail = fixtureClient.view(0).cities.find((c) => c.id === a.id)!.detail!;
    expect(detail.idle_buildings ?? []).toEqual([]);
    expect(fixtureClient.view(0).supply!.some((entry) => entry.resource === 'Copper')).toBe(false);
  });

  it('short supply idles the newest users until it returns', () => {
    const { s, a, iron } = fixture();
    const [, , newest] = [[3, 3], [3, 4], [2, 4]].map(([col, row]) => spawnUnit(s, 0, 'Pikeman', fromOffset(col, row), false));
    let { fixtureClient, core } = start(s);
    // Three Pikemen, 2 Iron: the first turn counts supply and idles the newest.
    let events = turn(fixtureClient, core, {});
    expect(events).toContainEqual({ type: 'SupplyShortage', player: 0, resource: 'Iron', idle: 1, supplied: 2, used: 3 });
    const v = fixtureClient.view(0);
    expect(supplyOf(v, 'Iron')).toMatchObject({ idle: ['Pikeman'], used: { total: 3, parts: [['Pikeman ×3', 3]] }, spare: -1 });
    const idle = v.units.filter((u) => u.unsupplied);
    expect(idle.map((u) => u.id)).toEqual([newest.id]);
    // The strength it loses comes from the core: 25% of the Pikeman's strength, whole numbers.
    expect(idle[0].unsupplied_loss).toBeGreaterThan(0);
    // Nothing changes while the shortage stands.
    expect(turn(fixtureClient, core, {}).filter((e) => e.type === 'SupplyShortage')).toEqual([]);

    // Its fights show the lost strength in the forecast.
    const fight = structuredClone(fixtureClient.debugState());
    spawnUnit(fight, 2, 'Spearman', fromOffset(2, 5), false);
    ({ fixtureClient, core } = start(fight));
    const target = fromOffset(2, 5);
    const f = core.forecast(0, newest.id, target, 'Melee');
    expect(f).not.toBeNull();
    (expect(fixtureClient.forecast(0, newest.id, target, 'Melee')).toMatchSnapshot(), void (f));
    expect(JSON.stringify(f)).toContain('No Iron supply');

    // A Mine on the deposit adds one: supply covers everyone again.
    (s.improved ??= s.terrain.map(() => 0))[idxOf(s, iron)] = 1;
    const mined = structuredClone(fixtureClient.debugState());
    mined.improved = s.improved;
    ({ fixtureClient, core } = start(mined));
    events = turn(fixtureClient, core, {});
    expect(events).toContainEqual({ type: 'SupplyShortage', player: 0, resource: 'Iron', idle: 0, supplied: 3, used: 3 });
    expect(fixtureClient.view(0).units.some((u) => u.unsupplied)).toBe(false);
    void a;
  });

  it('an idle Forge yields nothing and a Factory needs Coal and a working Forge', () => {
    const { s, a } = fixture();
    a.buildings = ['Granary', 'Workshop', 'Forge', 'Factory'];
    s.players[0].techs.push('IronWorking');
    let { fixtureClient, core } = start(s);
    // Iron covers the Forge; no Coal deposit: the Factory is idle after the first count.
    let events = turn(fixtureClient, core, {});
    expect(events).toContainEqual({ type: 'SupplyShortage', player: 0, resource: 'Coal', idle: 1, supplied: 0, used: 1 });
    let detail = fixtureClient.view(0).cities.find((c) => c.id === a.id)!.detail!;
    expect(detail.idle_buildings).toEqual([{ building: 'Factory', resource: 'Coal' }]);
    expect(detail.yields.sources.some((x) => x.label === 'Factory idle: no Coal')).toBe(true);
    expect(supplyOf(fixtureClient.view(0), 'Coal')).toMatchObject({ idle: ['Factory in A'] });

    // Lose the Iron deposit to another empire: the Forge idles, and the Factory with it.
    const lost = structuredClone(fixtureClient.debugState());
    lost.owner[idxOf(lost, fromOffset(5, 4))] = 2;
    lost.tileCity[idxOf(lost, fromOffset(5, 4))] = 4;
    ({ fixtureClient, core } = start(lost));
    events = turn(fixtureClient, core, {});
    expect(events).toContainEqual({ type: 'SupplyShortage', player: 0, resource: 'Iron', idle: 1, supplied: 0, used: 1 });
    detail = fixtureClient.view(0).cities.find((c) => c.id === a.id)!.detail!;
    expect(detail.idle_buildings).toEqual([{ building: 'Forge', resource: 'Iron' }, { building: 'Factory', resource: 'Coal' }]);
  });

  it('shows strategic resources only to empires that know them, and remembers them in fog', () => {
    const { s } = fixture();
    put(s, fromOffset(3, 5), 'Wheat');
    put(s, fromOffset(15, 15), 'Coal');
    for (const p of s.players) p.explored.fill(1);
    s.players[0].techs = s.players[0].techs.filter((t) => t !== 'Industrialization');
    let { fixtureClient, core } = start(s);
    const at = (v: PlayerView, col: number, row: number) => v.tiles[tileIndex(fromOffset(col, row), v.width, v.height)].resource ?? null;
    expect([at(fixtureClient.view(0), 5, 4), at(fixtureClient.view(0), 3, 5), at(fixtureClient.view(0), 15, 15)]).toEqual(['Iron', 'Wheat', null]);
    expect([at(fixtureClient.view(2), 5, 4), at(fixtureClient.view(2), 3, 5)]).toEqual([null, 'Wheat']);
    s.players[2].techs = ['IronWorking', 'Industrialization'];
    ({ fixtureClient, core } = start(s));
    expect([at(fixtureClient.view(2), 5, 4), at(fixtureClient.view(2), 15, 15)]).toEqual(['Iron', 'Coal']);
    turn(fixtureClient, core, {});
  });

  it('announces a new source and a reveal with no deposit to its team only', () => {
    const { s } = fixture();
    const p0 = s.players[0];
    p0.techs = ['Irrigation', 'BronzeWorking', 'Masonry', 'Currency', 'Writing'];
    p0.research = 'IronWorking';
    p0.research_queue = ['IronWorking'];
    p0.science = 5000;
    let { fixtureClient, core } = start(s);
    const events = turn(fixtureClient, core, {});
    expect(events.filter((e) => e.type === 'ResourcesRevealed')).toEqual([
      { type: 'ResourcesRevealed', player: 0, resource: 'Iron', city: 1, sources: 1, supply: 2 },
    ]);
    put(s, fromOffset(5, 4), null);
    ({ fixtureClient, core } = start(s));
    expect(turn(fixtureClient, core, {})).toContainEqual({ type: 'ResourcesRevealed', player: 0, resource: 'Iron', city: null, sources: 0, supply: 0 });
  });

  it('growth leans toward a known deposit and its neighbours', () => {
    const { s, a } = fixture();
    put(s, fromOffset(5, 4), null);
    put(s, fromOffset(6, 5), 'Iron');
    a.population = 4; // Three claims, all by population growth.
    const { fixtureClient, core } = start(s);
    const claims = turn(fixtureClient, core, {}).filter((e) => e.type === 'TileClaimed');
    expect(claims.length).toBeGreaterThan(0);
    turn(fixtureClient, core, {});
  });

  it.each(FACTIONS)('settlement scores and breakdowns count known resources for %s', (faction) => {
    // Rules 45's founded capital changes starting sight and the known land around this
    // synthetic Settler; the site-score snapshots are repinned for that opening.
    const game = new Game(Game.standard_setup('42', faction, 'India', 'A', 'B'));
    try {
      const v: PlayerView = JSON.parse(game.view(0));
      const u = v.units.find((x) => x.owner === v.me && x.kind === 'Spearman')!;
      u.pos = { ...v.cities.find((city) => city.owner === v.me && city.is_capital)!.pos };
      u.kind = 'Settler'; u.military = false;
      v.cities = []; v.city_gap_blocked = [];
      v.tiles.forEach((tile) => { if (tile.owner === v.me) tile.owner = null; });
      const near = within(u.pos, 3).map((h) => tileIndex(h, v.width, v.height)).filter((i) => i >= 0 && v.tiles[i].explored);
      near.forEach((i, k) => { if (k % 4 === 1) v.tiles[i].resource = (['Iron', 'Coal', 'Wheat', 'Copper'] as Resource[])[k % 3]; });
      const expected = settlementSites(v, u, []);
      (expect(JSON.parse(Game.settlement_sites(JSON.stringify(v), u.id, '[]'))).toMatchSnapshot(), void (expected.map(({ pos, score, quality, band, border_risk, rank, travel_turns }) => ({ pos, score, quality, band, border_risk, rank, travel_turns }))));
      const strategic = expected.filter((site) => site.breakdown.some((p) => p.label.startsWith('Known strategic deposits')));
      expect(strategic.length).toBeGreaterThan(0);
      for (const site of [expected[0], strategic[0], expected.find((x) => x.rank === 1)].filter((x) => !!x)) {
        expect(JSON.parse(Game.settlement_site(JSON.stringify(v), u.id, site.pos.q, site.pos.r, '[]'))).toEqual({ ...site, rank: null });
      }
    } finally { game.free(); }
  });

  it('a generated fixtureClient map with resources loads into core with identical views', () => {
    const s = createState(makeStandardSetup('4242', 'Byzantium', 'Sparta', 'A', 'B'));
    expect(s.resource!.some((r) => r === 'Iron')).toBe(true);
    for (const p of s.players) p.explored.fill(1);
    s.players[0].techs = ['BronzeWorking'];
    start(s);
  });
});
