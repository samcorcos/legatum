// Per-item production banking and the population-2 Settler rule in the WASM engine.
import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeAll, describe, expect, it } from 'vitest';
import { FIRST_GROWTH, TECHS, greatWorkRequirement } from '../data';
import { fromOffset, within } from '../hex';
import type { BuildingKind, Event, Order, ProductionItem } from '../types';
import { FixtureClient } from './fixtureEngine';
import { cityYields, idxOf, productionCost } from './fixtureEngine';
import { createState, makeStandardSetup, spawnUnit } from './fixtureEngine';
import type { MCity, MState } from './fixtureState';

const available = existsSync(fileURLToPath(new URL('../wasm-pkg/legatum.js', import.meta.url)));
const spearman: ProductionItem = { kind: 'Unit', unit: 'Spearman' };
const scout: ProductionItem = { kind: 'Unit', unit: 'Scout' };
const settler: ProductionItem = { kind: 'Unit', unit: 'Settler' };
const monument: ProductionItem = { kind: 'Building', building: 'Granary' };
type Turn = Record<number, Order[]>;

function fixture() {
  const setup = makeStandardSetup('700001', 'Athens', 'India', 'P0', 'P1');
  setup.width = setup.height = 20;
  setup.players = setup.players.slice(0, 3).map((p) => ({ ...p, is_human: true }));
  const s = createState(setup);
  delete (s as Partial<MState>).diplomacy;
  s.units = [];
  s.nextUnitId = 0;
  s.terrain.fill('Grassland');
  s.ruin.fill(0);
  s.wonder = undefined; // These fixtures test economy rules on a plain grid: no natural wonder rewards or yields.
  s.wondersDiscovered = undefined;
  s.owner.fill(-1);
  s.tileCity.fill(-1);
  for (const p of s.players) {
    p.gold = 10_000;
    p.techs = p.id === 0 ? TECHS.slice(0, 25) : [];
    p.explored.fill(0);
  }
  s.cities = [[0, 4, 4], [0, 13, 4], [1, 4, 13], [2, 13, 13]].map(([owner, col, row], i): MCity => ({
    id: i + 1, owner, founder: owner, name: ['A', 'B', 'Ally', 'Enemy'][i], pos: fromOffset(col, row),
    population: 5, food: 0, prod: 15, producing: owner === 0 ? spearman : null, bought: null,
    buildings: [], hp: 200, tilesClaimed: 0, grownTiles: [], isCapital: i !== 1, foundedTurn: 1, resistance: 0,
  }));
  s.nextCityId = s.cities.length;
  for (const c of s.cities) for (const h of within(c.pos, 1)) {
    s.owner[idxOf(s, h)] = c.owner;
    s.tileCity[idxOf(s, h)] = c.id;
  }
  const [a, b, ally, enemy] = s.cities;
  return { s, a, b, ally, enemy };
}

const failures = (events: Event[]) => events.filter((e) => e.type === 'OrderFailed').map((e) => e.type === 'OrderFailed' && e.reason);

function play(s: MState, turns: Turn[]) {
  const fixtureClient = new FixtureClient();
  fixtureClient.load(JSON.stringify(s));
  const reports: Event[][] = [];
  for (const orders of turns) {
    const actual = fixtureClient.endTurn(orders).events;
    reports.push(actual);
    fixtureClient.load(fixtureClient.save());
  }
  return { reports, state: fixtureClient.debugState(), fixtureClient };
}

const detail = (client: FixtureClient, p: number, id: number) => client.view(p).cities.find((c) => c.id === id)!.detail!;
const option = (client: FixtureClient, p: number, id: number, item: ProductionItem) =>
  detail(client, p, id).options.find((o) => JSON.stringify(o.item) === JSON.stringify(item));

describe.skipIf(!available)('WASM production banking', () => {
  beforeAll(async () => {
    const path = '../wasm-pkg/legatum.js';
    const mod = await import(/* @vite-ignore */ path);
    await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
  });

  it('switching away banks progress per item and switching back resumes it', () => {
    const { s, a } = fixture();
    const perTurn = cityYields(s, a).total.production;
    const { fixtureClient, state } = play(s, [
      { 0: [{ type: 'SetProduction', city: a.id, item: monument }] },
      { 0: [{ type: 'SetProduction', city: a.id, item: spearman }] },
      {},
    ]);
    const city = state.cities.find((c) => c.id === a.id)!;
    expect(city.producing).toEqual(spearman);
    // The Spearman kept its 15 while the Granary was built for a turn, then resumed.
    expect(city.prod).toBe(15 + 2 * perTurn);
    expect(city.productionBanked).toEqual([{ item: monument, progress: perTurn }]);
    expect(option(fixtureClient, 0, a.id, monument)).toMatchObject({ production_stored: perTurn, production_banked: perTurn });
    expect(option(fixtureClient, 0, a.id, spearman)).toMatchObject({ production_banked: 0 });
  });

  it('overflow after a completion joins the next queued item and its own bank', () => {
    const { s, a } = fixture();
    const cost = productionCost(s, 0, spearman);
    a.prod = cost - 1;
    a.productionQueue = [monument];
    a.productionBanked = [{ item: monument, progress: 10 }];
    const perTurn = cityYields(s, a).total.production;
    const { reports, state } = play(s, [{}, {}]);
    expect(reports[0]).toContainEqual({ type: 'ProductionCompleted', city: a.id, owner: 0, item: spearman });
    const city = state.cities.find((c) => c.id === a.id)!;
    expect(city.producing).toEqual(monument);
    expect(city.productionBanked).toBeUndefined();
    expect(city.prod).toBe(cost - 1 + perTurn - cost + 10 + perTurn);
  });

  it('a Settler whose city drops to population 1 banks, rejects new orders with the reason, and resumes at 2', () => {
    const { s, a } = fixture();
    const progress = 10;
    // Desert starves a population-2 city to 1 before its Settler can complete.
    s.terrain.fill('Desert');
    a.population = 2;
    a.food = 0;
    a.producing = settler;
    a.prod = progress;
    a.productionQueue = [settler];
    const first = play(s, [
      {},
      { 0: [{ type: 'SetProduction', city: a.id, item: settler }] },
      { 0: [{ type: 'SetProductionQueue', city: a.id, queue: [scout, settler] }] },
      { 0: [{ type: 'BuyItem', city: a.id, item: settler }] },
    ]);
    const reasons = first.reports.flatMap(failures);
    expect(reasons).toContain(`A can no longer produce Settler: Settler needs population 2 (currently 1); its ${progress} Production stays banked`);
    expect(reasons).toContain('A skipped queued Settler: Settler needs population 2 (currently 1)');
    expect(reasons).toContain('A cannot produce Settler: Settler needs population 2 (currently 1)');
    expect(reasons).toContain('Queue item 2: A cannot produce Settler: Settler needs population 2 (currently 1)');
    expect(reasons).toContain('Settler needs population 2 (currently 1)');
    const unavailable = detail(first.fixtureClient, 0, a.id).unavailable.find((u) => u.item.kind === 'Unit' && u.item.unit === 'Settler');
    expect(unavailable).toMatchObject({ reason: 'Settler needs population 2 (currently 1)', production_banked: progress });
    expect(first.state.cities.find((c) => c.id === a.id)!.productionBanked).toContainEqual({ item: settler, progress });

    // Grown back to 2, the Settler resumes from its bank.
    const s2 = first.state;
    const city = s2.cities.find((c) => c.id === a.id)!;
    s2.terrain.fill('Grassland');
    city.population = 2;
    city.producing = null;
    city.productionQueue = [];
    const unassigned = city.prod;
    const perTurn = cityYields(s2, city, settler).total.production;
    const second = play(s2, [{ 0: [{ type: 'SetProduction', city: a.id, item: settler }] }]);
    const resumed = second.state.cities.find((c) => c.id === a.id)!;
    expect(resumed.productionBanked?.find((b) => b.item.kind === 'Unit' && b.item.unit === 'Settler')).toBeUndefined();
    expect(resumed.producing).toEqual(settler);
    expect(resumed.prod).toBe(progress + unassigned + perTurn);
  });

  it('a Great Work bank goes stale once the empire completes its Great Work', () => {
    const { s, a, b } = fixture();
    const building: BuildingKind = 'GrandBazaar';
    const [required] = greatWorkRequirement(building)!;
    for (const c of [a, b]) {
      c.population = 8;
      c.buildings = [required];
    }
    const item = { kind: 'Wonder', building, tile: within(a.pos, 1).find((h) => h.q !== a.pos.q || h.r !== a.pos.r)! } as const;
    const built = { kind: 'Wonder', building, tile: within(b.pos, 1).find((h) => h.q !== b.pos.q || h.r !== b.pos.r)! } as const;
    a.producing = null;
    a.prod = 0;
    a.productionBanked = [{ item, progress: 50 }];
    b.producing = built;
    b.prod = productionCost(s, 0, built) - 1;
    const { reports, state } = play(s, [{}, {}]);
    expect(reports[0]).toContainEqual({ type: 'ProductionCompleted', city: b.id, owner: 0, item: built });
    expect(state.cities.find((c) => c.id === a.id)!.productionBanked).toBeUndefined();
  });

  it('a bank the owner can never use goes stale (an older save)', () => {
    const { s, a } = fixture();
    s.players[0].faction = s.setup.players[0].faction = 'India';
    const project = { kind: 'Project', project: 'Gold' } as const;
    a.productionBanked = [{ item: scout, progress: 6 }, { item: project, progress: 20 }];
    const { state } = play(s, [{}]);
    expect(state.cities.find((c) => c.id === a.id)!.productionBanked).toEqual([{ item: scout, progress: 6 }]);
  });

  it('buying a banked building adds its bank to the current build', () => {
    const { s, a } = fixture();
    a.prod = 0;
    a.productionBanked = [{ item: monument, progress: 20 }];
    const { reports, state } = play(s, [{ 0: [{ type: 'BuyItem', city: a.id, item: monument }] }]);
    expect(reports[0]).toContainEqual({ type: 'ProductionCompleted', city: a.id, owner: 0, item: monument });
    const city = state.cities.find((c) => c.id === a.id)!;
    expect(city.productionBanked).toBeUndefined();
    expect(city.producing).toEqual(spearman);
    expect(city.prod).toBe(20 + cityYields(state, city).total.production);
  });

  it('buying the active unit is a separate copy and never touches its progress', () => {
    const { s, a } = fixture();
    a.productionBanked = [{ item: scout, progress: 7 }];
    const perTurn = cityYields(s, a).total.production;
    const { state } = play(s, [{ 0: [{ type: 'BuyItem', city: a.id, item: spearman }] }]);
    const city = state.cities.find((c) => c.id === a.id)!;
    expect(city.producing).toEqual(spearman);
    expect(city.prod).toBe(15 + perTurn);
    expect(city.productionBanked).toEqual([{ item: scout, progress: 7 }]);
    expect(state.units.filter((u) => u.owner === 0 && u.kind === 'Spearman')).toHaveLength(1);
  });

  it('capture clears the captured city bank', () => {
    const { s, enemy } = fixture();
    enemy.hp = 0;
    enemy.producing = spearman;
    enemy.productionBanked = [{ item: scout, progress: 12 }, { item: monument, progress: 9 }];
    const attacker = spawnUnit(s, 0, 'Spearman', { q: enemy.pos.q - 1, r: enemy.pos.r }, false);
    const { reports, state } = play(s, [{ 0: [{ type: 'MoveUnit', unit: attacker.id, to: enemy.pos }] }]);
    expect(reports[0]).toContainEqual({ type: 'CityCaptured', city: enemy.id, from: 2, to: 0 });
    expect(state.cities.find((c) => c.id === enemy.id)!.productionBanked).toBeUndefined();
  });

  it('a project never banks; unassigned Production joins the next unit', () => {
    const { s, a } = fixture();
    s.players[0].faction = s.setup.players[0].faction = 'Germany';
    const project = { kind: 'Project', project: 'Gold' } as const;
    play(s, [
      { 0: [{ type: 'SetProduction', city: a.id, item: project }] },
      {},
      { 0: [{ type: 'SetProduction', city: a.id, item: monument }] },
      { 0: [{ type: 'SetProduction', city: a.id, item: spearman }] },
    ]);
  });

  it('faction-unavailable items are rejected with the reason', () => {
    const { s, a } = fixture();
    s.players[0].faction = s.setup.players[0].faction = 'India';
    const project = { kind: 'Project', project: 'Grow' } as const;
    const { reports } = play(s, [
      { 0: [{ type: 'SetProduction', city: a.id, item: project }] },
      { 0: [{ type: 'SetProductionQueue', city: a.id, queue: [spearman, project] }] },
    ]);
    const reasons = reports.flatMap(failures).map(String);
    expect(reasons.some((r) => /^A cannot produce .+: Only Egyptian cities can choose Grow$/.test(r))).toBe(true);
    expect(reasons.some((r) => r.startsWith('Queue item 2: ') && r.endsWith('Only Egyptian cities can choose Grow'))).toBe(true);
  });

  it('the first growth costs FIRST_GROWTH', () => {
    const { s, a, b } = fixture();
    a.population = 1;
    a.food = 0;
    b.population = 2;
    const before = new FixtureClient();
    before.load(JSON.stringify(s));
    expect(detail(before, 0, a.id).growth_threshold).toBe(FIRST_GROWTH);
    expect(detail(before, 0, a.id).growth_breakdown?.parts).toEqual([['First growth', FIRST_GROWTH]]);
  });
});
