import { RULES, START_GOLD, factionGoalVp } from '../data';
import { coreFixture } from './coreFixture';
import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeAll, describe, expect, it } from 'vitest';
import { BUILDING_NAME, FACTIONS, GREAT_WORKS, TECHS, UNIT_KINDS, greatWorkRequirement } from '../data';
import { fromOffset, hexEq, neighbors, within } from '../hex';
import type { BuildingKind, Event, Order, ProductionItem, UnitKind } from '../types';
import { WasmClient } from '../wasmClient';
import { FixtureClient } from './fixtureEngine';
import { developmentForecast } from './fixtureEngine';
import { buildingBlocker, buyCost, canProduce, cityYields, growthThreshold, IDLE_PRODUCTION_CAP, idxOf, productionCost, swissInterest, upkeepBreakdown } from './fixtureEngine';
import { createState, makeStandardSetup, spawnUnit } from './fixtureEngine';
import type { MCity, MState } from './fixtureState';
import { upgradeCost } from './fixtureEngine';

const available = existsSync(fileURLToPath(new URL('../wasm-pkg/legatum.js', import.meta.url)));
let makeCore: () => WasmClient;
const spearman: ProductionItem = { kind: 'Unit', unit: 'Spearman' };
const settler: ProductionItem = { kind: 'Unit', unit: 'Settler' };
type Turn = Record<number, Order[]>;

function fixture() {
  const setup = makeStandardSetup('700001', 'Athens', 'India', 'P0', 'P1');
  setup.width = setup.height = 20;
  setup.players = setup.players.slice(0, 3).map((p) => ({ ...p, is_human: true }));
  const s = createState(setup);
  delete (s as Partial<MState>).diplomacy; // Legacy tactical fixture; diplomacy parity has its own fixtures.
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
    p.techs = p.id === 0 ? TECHS.slice(0, 30) : [];
    p.explored.fill(0);
  }
  s.cities = [[0, 4, 4], [0, 13, 4], [1, 4, 13], [2, 13, 13]].map(([owner, col, row], i): MCity => ({
    id: i + 1, owner, founder: owner, name: ['A', 'B', 'Ally', 'Enemy'][i], pos: fromOffset(col, row),
    population: 5, food: 0, prod: 15, producing: owner === 0 ? spearman : null, bought: null,
    buildings: [], hp: 200, grownTiles: [], tilesClaimed: 0, isCapital: i !== 1, foundedTurn: 1, resistance: 0,
  }));
  s.nextCityId = s.cities.length;
  for (const c of s.cities) for (const h of within(c.pos, 1)) {
    s.owner[idxOf(s, h)] = c.owner;
    s.tileCity[idxOf(s, h)] = c.id;
  }
  const [a, b, ally, enemy] = s.cities;
  return { s, a, b, ally, enemy };
}

function block(s: MState, c: MCity, owner: number, kind: UnitKind = 'Spearman') {
  const units = within(c.pos, 1).map((h) => spawnUnit(s, owner, kind, h, false));
  return units.find((u) => hexEq(u.pos, c.pos))!;
}

function greatWorkFixture(building: BuildingKind) {
  const result = fixture();
  if (building === 'IterFusionReactor') result.s.players[0].techs.push('FusionPower');
  const { a, b } = result;
  const required = greatWorkRequirement(building)?.[0];
  for (const c of [a, b]) {
    c.population = 8;
    c.buildings = required ? [required] : [];
    c.producing = null;
  }
  return { ...result, item: { kind: 'Wonder', building, tile: within(a.pos, 1).find((h) => !hexEq(h, a.pos))! } as const };
}

function tallGrowthFixture() {
  const result = fixture();
  const { s, a, b } = result;
  // Territory growth cannot change the food yields during resolution: claimed tiles are Mountains,
  // and Irrigation's city Food or Crop Rotation's Mill Food would let the pop-8 city regrow.
  s.players[0].techs = s.players[0].techs.filter((t) => t !== 'Irrigation' && t !== 'CropRotation');
  s.terrain.fill('Mountains');
  for (const c of [a, b]) {
    c.producing = null;
    for (const h of within(c.pos, 1)) s.terrain[idxOf(s, h)] = 'Plains';
  }
  a.population = 8;
  a.buildings = ['Mill'];
  a.resistance = 1;
  a.food = 0;
  b.population = 7;
  b.buildings = ['Mill'];
  // The partner once had an Arcology's +5 Food over the pop-8 city; five Grassland tiles around it give that back.
  within(b.pos, 1).filter((h) => !hexEq(h, b.pos)).slice(0, 5).forEach((h) => { s.terrain[idxOf(s, h)] = 'Grassland'; });
  // Food surplus per turn of the pop-7 partner, which the forecasts count in whole turns.
  const surplus = cityYields(s, b).total.food - 2 * b.population;
  expect(surplus).toBeGreaterThan(0);
  b.food = growthThreshold(7) - 4 * surplus;
  return { ...result, surplus };
}


// The fixtureClient engine has no hangar, so its production list omits aircraft (Fighter, Bomber).
const notAircraft = (o: { item: ProductionItem }) =>
  !(o.item.kind === 'Unit' && (o.item.unit === 'Fighter' || o.item.unit === 'Bomber'));
const purchases = (events: Event[]) => events.filter((e) => e.type === 'ItemBought');
const failures = (events: Event[]) => events.filter((e) => e.type === 'OrderFailed');
const completions = (events: Event[]) => events.filter((e) => e.type === 'ProductionCompleted');

function play(s: MState, turns: Turn[]) {
  const fixtureClient = new FixtureClient();
  fixtureClient.load(JSON.stringify(s));
  const core = makeCore();
  core.load(coreFixture(s));
  const reports: Event[][] = [];
  const nativeReports: Event[][] = [];
  for (const orders of turns) {
    const before = fixtureClient.save();
    expect(fixtureClient.save()).toBe(before);
    const actual = fixtureClient.endTurn(orders).events;
    nativeReports.push(core.endTurn(orders).events);
    reports.push(actual);
    fixtureClient.load(fixtureClient.save());
    core.load(core.save());
  }
  return { reports, nativeReports, state: fixtureClient.debugState(), fixtureClient, core };
}

describe.skipIf(!available)('WASM purchase behavior', () => {
  beforeAll(async () => {
    const path = '../wasm-pkg/legatum.js';
    const mod = await import(/* @vite-ignore */ path);
    await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
    makeCore = () => new WasmClient(mod.Game);
  });

  // The tactical fixture models Production-funded buildings, so Egypt's Food economy is covered by native core tests.
  it.each(FACTIONS.filter((f) => f !== 'Egypt'))('%s unit strengths and health-weighted Army metric agree with native scoring', (faction) => {
    const { s, a } = fixture();
    s.players[0].faction = faction;
    s.players[0].techs = [...TECHS];
    for (const kind of UNIT_KINDS) {
      const unit = spawnUnit(s, 0, kind, a.pos, false);
      unit.hp = 63;
    }
    const fixtureClient = new FixtureClient();
    fixtureClient.load(JSON.stringify(s));
    const core = makeCore();
    core.load(coreFixture(s));
    const units = (client: FixtureClient | WasmClient) => client.view(0).units.map((u) => ({
      kind: u.kind, strength: u.strength, ranged: u.ranged, moves: u.max_moves, upgrade: u.upgrade_cost,
    }));
    (expect(units(fixtureClient)).toMatchSnapshot(), void (units(core)));
    const prices = (client: FixtureClient | WasmClient) => client.view(0).cities.find((c) => c.id === a.id)!.detail!.options
      .filter((o) => o.item.kind === 'Unit' && notAircraft(o)).map((o) => ({ item: o.item, cost: o.cost }));
    (expect(prices(fixtureClient)).toMatchSnapshot(), void (prices(core)));
    (expect(fixtureClient.view(0).players[0].army).toMatchSnapshot(), void (core.view(0).players[0].army));
  });

  it('Sparta healing forecasts resolve exactly on home, foreign and allied land', () => {
    for (const site of ['field', 'enemy', 'ally', 'home'] as const) {
      const { s, a, b } = fixture();
      s.players[0].faction = 'Sparta';
      a.producing = b.producing = null;
      const home = site === 'home';
      const pos = home ? a.pos : fromOffset(8, 8);
      s.owner[idxOf(s, pos)] = home ? 0 : site === 'enemy' ? 2 : site === 'ally' ? 1 : -1;
      a.buildings = [];
      a.resistance = 0;
      for (const kind of ['Spearman', 'Scout', 'Settler'] as const) spawnUnit(s, 0, kind, pos, false).hp = 40;
      const fixtureClient = new FixtureClient();
      fixtureClient.load(JSON.stringify(s));
      const core = makeCore();
      core.load(coreFixture(s));
      const forecast = fixtureClient.view(0).units.filter((u) => u.owner === 0);
      (expect(forecast.map((u) => u.resting_heal)).toMatchSnapshot(), void (core.view(0).units.filter((u) => u.owner === 0).map((u) => u.resting_heal)));
      for (const unit of forecast) {
        // Spartan soldiers, Scouts and civilians all rest-heal at the normal rate.
        const expected = home ? 20 : 10;
        (expect(unit.resting_heal!.total).toMatchSnapshot(), void (expected));
        (expect(unit.resting_heal!.parts.reduce((sum, p) => sum + p[1], 0)).toMatchSnapshot(), void (expected));
      }
      fixtureClient.endTurn({});
      core.endTurn({});
      for (const before of forecast) {
        const native = core.view(0).units.find((u) => u.id === before.id)!;
        expect(native.hp).toBe(40 + before.resting_heal!.total);
        (expect(fixtureClient.view(0).units.find((u) => u.id === before.id)!.hp).toMatchSnapshot(), void (native.hp));
      }
    }
  });

  it.each([6, 7, 8, 9])('India distant-city yields and their quoted boundary match native rules (distance %s)', (distance) => {
    const { s, a, b } = fixture();
    s.players[0].faction = 'India';
    a.producing = b.producing = null;
    b.pos = fromOffset(4 + distance, 4);
    const core = makeCore();
    core.load(coreFixture(s));
    const native = core.view(0).cities.find((c) => c.id === b.id)!.detail!.yields;
    (expect(cityYields(s, b)).toMatchSnapshot(), void (native));
    const loss = Math.min(100, (distance - RULES.indiaCapitalRadius) * RULES.indiaYieldPercentPerHex);
    expect(native.sources.some((x) => x.label === `Corruption: ${distance} tiles from the capital (-${loss}%)`)).toBe(distance > 4);
    expect(native.sources.find((x) => x.label === 'India network')?.yields).toEqual({ food: 1, production: 0, gold: 0, science: 0 }); // Population is territory: the city network adds Food.
  });

  it.each([7, 8, 9])('Mongols commerce begins at eight hexes, with upgraded Markets (distance %s)', (distance) => {
    const { s, a, b } = fixture();
    s.players[0].faction = 'Mongols';
    a.producing = b.producing = null;
    b.pos = fromOffset(4 + distance, 4);
    b.buildings = ['Bank'];
    const core = makeCore();
    core.load(coreFixture(s));
    const native = core.view(0).cities.find((c) => c.id === b.id)!.detail!.yields;
    (expect(cityYields(s, b)).toMatchSnapshot(), void (native));
    expect(native.sources.find((x) => x.label === 'Mongols: long-road commerce')?.yields.gold ?? 0).toBe(distance >= 8 ? 2 : 0);
    b.resistance = 1;
    expect(cityYields(s, b).total.gold).toBe(0);
  });

  it.each([
    ['India', 'Walls'], ['India', 'Castle'], ['India', 'StarFort'], ['Switzerland', 'Walls'],
  ] as const)('%s cannot buy the Walls-line building %s and matches native', (faction, building) => {
    const { s, a } = fixture();
    s.players[0].faction = s.setup.players[0].faction = faction;
    s.players[0].techs = ['Masonry', 'Feudalism', 'Chivalry', 'Machinery'];
    a.producing = null;
    a.buildings = building === 'Castle' ? ['Walls'] : building === 'StarFort' ? ['Walls', 'Castle'] : [];
    const item = { kind: 'Building' as const, building };
    const fixtureClient = new FixtureClient(); fixtureClient.load(JSON.stringify(s));
    const core = makeCore(); core.load(coreFixture(s));
    const pick = (view: ReturnType<typeof core.view>) => view.cities.find((c) => c.id === a.id)!.detail!.options.find((o) => o.item.kind === 'Building' && o.item.building === building)!;
    const native = pick(core.view(0)), mocked = pick(fixtureClient.view(0));
    expect([native.buy_cost, native.buy_quote, native.buy_blocker]).toEqual([
      null, null, "Walls must be built with Production; they can't be bought.",
    ]);
    (expect([mocked.buy_cost, mocked.buy_quote, mocked.buy_blocker, mocked.buy_block]).toMatchSnapshot(), void ([
      native.buy_cost, native.buy_quote, native.buy_blocker, native.buy_block,
    ]));
    const { reports } = play(s, [{ 0: [{ type: 'BuyItem', city: a.id, item }] }]);
    expect(failures(reports[0])[0].reason).toBe("Walls must be built with Production; they can't be bought.");
  });

  it.each(['Switzerland', 'India', 'Mongols'] as const)('Great Work purchase quotes in the city option list match native for %s', (faction) => {
    const { s, a, item } = greatWorkFixture('RoyalArsenal');
    s.players[0].faction = s.setup.players[0].faction = faction;
    const fixtureClient = new FixtureClient();
    fixtureClient.load(JSON.stringify(s));
    const core = makeCore();
    core.load(coreFixture(s));
    const pick = (view: ReturnType<typeof core.view>) => view.cities.find((c) => c.id === a.id)!.detail!.options.find((o) => o.item.kind === 'Wonder' && o.item.building === item.building && hexEq(o.item.tile, item.tile))!;
    const native = pick(core.view(0)), mocked = pick(fixtureClient.view(0));
    // No empire can buy a world wonder.
    expect([native.buy_cost, native.buy_quote]).toEqual([null, null]);
    (expect([mocked.buy_cost, mocked.buy_quote, mocked.buy_blocker, mocked.buy_block]).toMatchSnapshot(), void ([native.buy_cost, native.buy_quote, native.buy_blocker, native.buy_block]));
  });

  it('Swiss interest and public capital bounty match native capture resolution', () => {
    const { s, enemy } = fixture();
    s.players[2].faction = s.setup.players[2].faction = 'Switzerland';
    s.players[2].gold = 400;
    enemy.population = 5;
    enemy.hp = 0;
    const attacker = spawnUnit(s, 0, 'Spearman', { q: enemy.pos.q - 1, r: enemy.pos.r }, false);
    const core = makeCore(); core.load(coreFixture(s));
    expect(core.view(0).cities.find((c) => c.id === enemy.id)?.capture_bounty).toBe(400);
    const { reports, fixtureClient } = play(s, [{ 0: [{ type: 'MoveUnit', unit: attacker.id, to: enemy.pos }] }]);
    expect(reports[0]).toContainEqual({ type: 'CityCaptured', city: enemy.id, from: 2, to: 0, bounty: 400 });
    expect(fixtureClient.view(2).economy.gold).toBe(0);
  });

  it('Swiss interest has no maximum and fixtureClient income matches native', () => {
    for (const gold of [99, 100, 450, 1_234, 50_000]) {
      const { s } = fixture();
      s.players[0].faction = 'Switzerland';
      s.players[0].gold = gold;
      expect(swissInterest(s, 0)).toBe(Math.floor(gold / 100));
      const fixtureClient = new FixtureClient();
      fixtureClient.load(JSON.stringify(s));
      const core = makeCore();
      core.load(coreFixture(s));
      const line = (view: ReturnType<typeof core.view>) => view.economy.gold_per_turn.parts.find(([label]) => label === 'Switzerland: interest')?.[1] ?? 0;
      expect(line(core.view(0))).toBe(Math.floor(gold / 100));
      expect(line(fixtureClient.view(0))).toBe(Math.floor(gold / 100));
      (expect(fixtureClient.view(0).economy.gold_per_turn).toMatchSnapshot(), void (core.view(0).economy.gold_per_turn));
      // After a turn the treasury has moved; both clients still pay interest on the new treasury.
      const { core: after, fixtureClient: mocked } = play(s, [{}]);
      expect(line(mocked.view(0))).toBe(Math.floor(mocked.view(0).economy.gold / 100));
      expect(line(after.view(0))).toBe(Math.floor(after.view(0).economy.gold / 100));
    }
  });

  it('Swiss flat upkeep reconciles with native income and resolution', () => {
    for (let count = 1; count <= 7; count++) {
      const { s, a, b } = fixture();
      s.players[0].faction = 'Switzerland';
      s.players[0].techs = [];
      a.producing = b.producing = null;
      const spots = within(a.pos, 3).filter((h) => !hexEq(h, a.pos));
      for (let i = 0; i < count; i++) spawnUnit(s, 0, 'Spearman', spots[i], false);
      const paid = Math.max(0, count - 2), raw = paid * 3;
      expect(upkeepBreakdown(s, 0).total).toBe(raw);
      const { core, fixtureClient } = play(s, [{}]);
      (expect(fixtureClient.view(0).economy.gold_per_turn).toMatchSnapshot(), void (core.view(0).economy.gold_per_turn));
    }
  });

  it('native terrain food and its growth forecast match actual resolution', () => {
    const { s, a, b } = fixture();
    s.players[0].faction = 'Celts';
    s.players[0].techs = [];
    a.population = 2;
    a.food = growthThreshold(2) - 1;
    a.producing = null;
    b.producing = null;
    const core = makeCore();
    core.load(coreFixture(s));
    const forecast = core.view(0).cities.find((c) => c.id === a.id)!.detail!;
    expect(forecast.yields).toEqual(cityYields(s, a));
    expect(forecast.yields.total.food).toBe(16); // Every owned tile within three hexes is worked.
    expect(forecast.turns_to_grow).toBe(1);
    const played = play(s, [{}]);
    expect(played.state.cities.find((c) => c.id === a.id)!.population).toBe(3);
  });

  // The Swarm, Sakoku and the Athens' broadcast are actions: fixtureClient/actions.wasm.test.ts.
  it.each(['Mongols'] as const)('%s earns its Yam Relay Station goal award once, with fixtureClient and native VP parity', (faction) => {
    const { s, a } = fixture();
    s.players[0].faction = faction;
    s.players[0].techs = ['Philosophy'];
    const count = 3;
    s.cities = Array.from({ length: count }, (_, i) => ({
      ...a, id: i + 1, pos: fromOffset(2 + i * 8, 2),
      buildings: i === 1 ? ['YamRelayStation' as const] : [],
      producing: null, population: 1, prod: 0, isCapital: i === 0, originalCapital: i === 0,
    }));
    s.nextCityId = count;
    s.players[0].hallCity = 2;
    spawnUnit(s, 2, 'Settler', fromOffset(19, 19), false);
    s.owner.fill(-1);
    s.tileCity.fill(-1);
    for (const c of s.cities) for (const h of within(c.pos, 1)) {
      s.owner[idxOf(s, h)] = 0;
      s.tileCity[idxOf(s, h)] = c.id;
    }
    const result = play(s, [{}, {}]);
    expect(result.reports.flat().filter((e) => e.type === 'FactionGoalCompleted')).toEqual([
      { type: 'FactionGoalCompleted', player: 0, faction, vp: factionGoalVp(faction) },
    ]);
    expect(result.fixtureClient.view(0).standings).toMatchSnapshot();
  });

  it('pays the visible capital Production and Science on the founding turn for every faction', () => {
    for (const faction of FACTIONS) {
      const { s } = fixture();
      s.players[0].faction = faction;
      s.players[0].techs = [];
      s.cities = [];
      s.owner.fill(-1);
      s.tileCity.fill(-1);
      const first = spawnUnit(s, 0, 'Settler', fromOffset(4, 4), false);
      const second = spawnUnit(s, 0, 'Settler', fromOffset(13, 4), false);
      spawnUnit(s, 2, 'Settler', fromOffset(13, 13), false);
      const { state } = play(s, [{ 0: [{ type: 'FoundCity', unit: first.id }] }, { 0: [{ type: 'FoundCity', unit: second.id }] }]);
      const cities = state.cities.filter((c) => c.owner === 0);
      expect(cities.map((c) => cityYields(state, c).total.science)).toEqual([2, 0]);
      expect(state.players[0].science).toBe(4);
      expect(cityYields(state, cities[0]).sources).toContainEqual({ label: 'Capital', yields: { food: 0, production: 2, gold: 0, science: 2 } });
    }
  });

  it('removes captured capital Science and awards it to the next founded city after save/load', () => {
    const { s, enemy } = fixture();
    enemy.population = 1;
    enemy.hp = 0;
    const attacker = spawnUnit(s, 0, 'Spearman', { q: enemy.pos.q - 1, r: enemy.pos.r }, false);
    const settler = spawnUnit(s, 2, 'Settler', fromOffset(13, 8), false);
    const { reports, state } = play(s, [
      { 0: [{ type: 'MoveUnit', unit: attacker.id, to: enemy.pos }] },
      { 2: [{ type: 'FoundCity', unit: settler.id }] }, {}, {},
    ]);
    expect(reports[0]).toContainEqual({ type: 'CityCaptured', city: enemy.id, from: 2, to: 0 });
    const captured = state.cities.find((c) => c.id === enemy.id)!;
    expect(captured.resistance).toBeGreaterThan(0);
    expect(captured.isCapital).toBe(false);
    expect(cityYields(state, captured).sources.some((source) => source.label === 'Capital')).toBe(false);
    const replacement = state.cities.find((c) => c.owner === 2)!;
    expect(replacement.isCapital).toBe(true);
    expect(cityYields(state, replacement).sources.find((source) => source.label === 'Capital')!.yields.science).toBe(2);
  });

  it('research queue validation, advance, save/load and visibility match', () => {
    const { s, a, b } = fixture();
    s.players[0].techs = [];
    s.players[0].science = 1000;
    a.producing = b.producing = null;
    const fixtureClient = new FixtureClient(); fixtureClient.load(JSON.stringify(s));
    const core = makeCore(); core.load(coreFixture(s));
    const queue = ['BronzeWorking', 'Writing', 'Irrigation', 'Currency'] as const;
    const invalid: Order[][] = [
      [{ type: 'SetResearchQueue', techs: ['Mathematics'] }],
      [{ type: 'SetResearchQueue', techs: ['Writing', 'Mathematics'] }],
      [{ type: 'SetResearchQueue', techs: ['BronzeWorking', 'BronzeWorking'] }],
    ];
    for (const orders of invalid) {
      (expect(fixtureClient.validate(0, orders)).toMatchSnapshot(), void (core.validate(0, orders)));
      expect(fixtureClient.validate(0, orders)).toHaveLength(1);
    }
    for (let i = 0; i < queue.length; i++) {
      const orders: Turn = i === 0 ? { 0: [{ type: 'SetResearchQueue', techs: [...queue] }] } : {};
      const actual = fixtureClient.endTurn(orders), expected = core.endTurn(orders);
      const events = (ev: Event[]) => ev.filter((e) => e.type === 'TechResearched' || e.type === 'ResearchQueueAdvanced');
      (expect(events(actual.events)).toMatchSnapshot(), void (events(expected.events)));
      expect(events(actual.events)).toContainEqual({ type: 'ResearchQueueAdvanced', player: 0, next: queue[i + 1] ?? null });
      for (const player of s.players) {
        (expect(events(actual.events_by_player![player.id])).toMatchSnapshot(), void (events(expected.events_by_player![player.id])));
        const research = (client: FixtureClient | WasmClient) => {
          const r = client.view(player.id).research;
          return { current: r.current, queue: r.queue, progress: r.progress, known: [...r.known].sort(), turns: r.turns_left };
        };
        (expect(research(fixtureClient)).toMatchSnapshot(), void (research(core)));
      }
      expect(fixtureClient.view(0).research.queue).toEqual(queue.slice(i + 1));
      const known: Order[] = [{ type: 'SetResearchQueue', techs: ['BronzeWorking'] }];
      (expect(fixtureClient.validate(0, known)).toMatchSnapshot(), void (core.validate(0, known)));
      fixtureClient.load(fixtureClient.save()); core.load(core.save());
    }
    const clear: Turn = { 0: [{ type: 'SetResearchQueue', techs: [] }] };
    fixtureClient.endTurn(clear); core.endTurn(clear);
    (expect(fixtureClient.view(0).research.progress).toMatchSnapshot(), void (core.view(0).research.progress));
    expect(fixtureClient.view(0).research.queue).toEqual([]);
  });

  it.each(FACTIONS.filter((f) => f !== 'Egypt'))('Inheritance purchases and infantry upgrades retain %s', (faction) => {
    const { s, a, b } = fixture();
    s.players[0].faction = faction;
    // Keep the unit line at Musketman while retaining the Research Lab unlocks.
    s.players[0].techs = TECHS.filter((tech) => tech !== 'Rifling');
    a.population = 8;
    a.buildings = ['University'];
    a.producing = null;
    b.producing = null;
    // The Research Lab uses Oil and the Musketman Niter (and the Pikeman's upgrade into it): one deposit of each inside A's border.
    s.resource = s.terrain.map(() => null);
    const [niter, oil] = within(a.pos, 1).filter((h) => !hexEq(h, a.pos));
    s.resource[idxOf(s, niter)] = 'Niter';
    s.resource[idxOf(s, oil)] = 'Oil';
    const infantry = spawnUnit(s, 0, 'Pikeman', b.pos, false);
    const coreItem = { kind: 'Building', building: 'ResearchLab' } as const;
    const assault = { kind: 'Unit', unit: 'Musketman' } as const;
    const { reports, state } = play(s, [{ 0: [
      { type: 'SetProduction', city: a.id, item: coreItem }, { type: 'BuyItem', city: a.id, item: coreItem },
      { type: 'UpgradeUnit', unit: infantry.id },
    ] }, { 0: [{ type: 'SetProduction', city: b.id, item: assault }, { type: 'BuyItem', city: b.id, item: assault }] }]);
    expect(completions(reports[0])).toContainEqual({ type: 'ProductionCompleted', owner: 0, city: a.id, item: coreItem });
    expect(state.cities.find((c) => c.id === a.id)!.buildings).toEqual(['ResearchLab']);
    expect(state.units.find((u) => u.id === infantry.id)!.kind).toBe('Musketman');
    expect(completions(reports[1])).toContainEqual({ type: 'ProductionCompleted', owner: 0, city: b.id, item: assault });
  });

  it('Inference Core rechecks the population gate for purchases', () => {
    const { s, a } = fixture(); s.players[0].techs = [...TECHS];
    a.population = 7; a.buildings = ['University'];
    a.producing = { kind: 'Building', building: 'ResearchLab' }; // legal earlier commitment
    const core = makeCore(); core.load(coreFixture(s));
    const fixtureClient = new FixtureClient(); fixtureClient.load(JSON.stringify(s));
    for (const client of [core, fixtureClient]) {
      expect(client.view(0).cities.find((c) => c.id === a.id)!.detail!.buy_cost).toBeNull();
      expect(client.validate(0, [{ type: 'BuyItem', city: a.id, item: { kind: 'Building', building: 'ResearchLab' } }])).toHaveLength(1);
    }
    a.prod = productionCost(s, 0, a.producing);
    const { state } = play(s, [{}]);
    expect(state.cities.find((c) => c.id === a.id)!.buildings).toEqual(['ResearchLab']);
  });

  it('growth retains overflow and grows once, with identical forecasts', () => {
    const { s, a, b } = fixture(); s.players[0].techs = [...TECHS];
    a.population = 2; a.food = 1000; a.buildings = ['Mill']; a.producing = null; b.producing = null;
    const threshold = growthThreshold(2);
    const food = cityYields(s, a).total.food - 4;
    const { state } = play(s, [{}]);
    const grown = state.cities.find((c) => c.id === a.id)!;
    expect(grown.population).toBe(3);
    expect(grown.food).toBe(1000 + food - threshold);
  });

  it.each(FACTIONS.filter((f) => f !== 'Egypt'))('Full-tree quotes and faction effects match WASM for %s', (faction) => {
    const { s, a, b } = fixture(); s.players[0].faction = faction;
    a.population = 8; a.producing = null; b.producing = null;
    a.buildings = ['Mill', 'Factory', 'Bank', 'ResearchLab', 'Library', 'GreatLibrary'];
    spawnUnit(s, 0, 'Pikeman', b.pos, false);
    for (let i = 0; i < 6; i++) spawnUnit(s, 0, 'Scout', fromOffset(5 + i, 7), false);
    const snapshot = () => {
      const core = makeCore(); core.load(coreFixture(s));
      const fixtureClient = new FixtureClient(); fixtureClient.load(JSON.stringify(s));
      const native = core.view(0), demo = fixtureClient.view(0);
      (expect(demo.economy.upkeep).toMatchSnapshot(), void (native.economy.upkeep));
      (expect(demo.economy.upkeep_breakdown).toMatchSnapshot(), void (native.economy.upkeep_breakdown));
      for (const tech of TECHS.slice(25)) {
        const actual = demo.research.catalog.find((t) => t.tech === tech)!;
        const expected = native.research.catalog.find((t) => t.tech === tech)!;
        (expect(actual.cost).toMatchSnapshot(), void (expected.cost));
        (expect(actual.effects).toMatchSnapshot(), void (expected.effects));
        (expect(actual.income_change).toMatchSnapshot(), void (expected.income_change));
      }
      for (const city of demo.cities.filter((c) => c.owner === 0)) {
        const d = city.detail!, n = native.cities.find((c) => c.id === city.id)!.detail!;
        (expect(d.yields.total).toMatchSnapshot(), void (n.yields.total));
        (expect(d.growth_breakdown).toMatchSnapshot(), void (n.growth_breakdown));
      }
      for (const u of demo.units.filter((u) => u.owner === 0)) {
        const n = native.units.find((x) => x.id === u.id)!;
        (expect(u.upgrade_breakdown).toMatchSnapshot(), void (n.upgrade_breakdown));
        (expect(u.upgrade_cost).toMatchSnapshot(), void (n.upgrade_cost));
      }
    };
    snapshot();
    s.players[0].techs = [...TECHS];
    snapshot();
    play(s, [{}]);
  });

  // G53: every build card shows its ongoing cost. The preview (free slots left, paid rate, Gold/
  // turn after) must agree between the demo engine and native core at every point around the
  // two-free-unit threshold, for a faction with the ordinary rate and one with the higher rate.
  it.each(['India', 'Switzerland'] as const)('unit card upkeep previews match WASM for %s around the free-slot threshold', (faction) => {
    const { s, a } = fixture();
    s.players[0].faction = faction;
    a.producing = null;
    const positions = within(a.pos, 3).filter((h) => !hexEq(h, a.pos));
    for (let military = 0; military <= 3; military++) {
      s.units = s.units.filter((u) => u.owner !== 0);
      for (let i = 0; i < military; i++) spawnUnit(s, 0, 'Spearman', positions[i], false);
      const core = makeCore(); core.load(coreFixture(s));
      const fixtureClient = new FixtureClient(); fixtureClient.load(JSON.stringify(s));
      const demo = fixtureClient.view(0).cities.find((c) => c.id === a.id)!.detail!;
      const native = core.view(0).cities.find((c) => c.id === a.id)!.detail!;
      const options = (detail: typeof demo) => detail.options
        .filter((o) => o.item.kind === 'Unit' && notAircraft(o)).map((o) => ({ item: o.item, upkeep: o.upkeep ?? null }));
      (expect(options(demo)).toMatchSnapshot(), void (options(native)));
      (expect(fixtureClient.view(0).economy.gold_per_turn.total).toMatchSnapshot(), void (core.view(0).economy.gold_per_turn.total));
    }
  });

  // Scouts never pay upkeep and never use a free slot (user, 2026-10-06): the upkeep total, the
  // card previews (a Scout's card says "No upkeep") and the Gold/turn forecast match core.
  it.each(['India', 'Switzerland'] as const)('Scouts are upkeep-free and use no free slot, matching WASM for %s', (faction) => {
    const { s, a } = fixture();
    s.players[0].faction = faction;
    a.producing = null;
    const positions = within(a.pos, 3).filter((h) => !hexEq(h, a.pos));
    for (let military = 0; military <= 3; military++) {
      s.units = s.units.filter((u) => u.owner !== 0);
      for (let i = 0; i < military; i++) spawnUnit(s, 0, 'Spearman', positions[i], false);
      for (let i = 0; i < 2; i++) spawnUnit(s, 0, 'Scout', positions[3 + i], false);
      const core = makeCore(); core.load(coreFixture(s));
      const fixtureClient = new FixtureClient(); fixtureClient.load(JSON.stringify(s));
      const demo = fixtureClient.view(0), native = core.view(0);
      const options = (v: typeof demo) => v.cities.find((c) => c.id === a.id)!.detail!.options
        .filter((o) => o.item.kind === 'Unit' && notAircraft(o)).map((o) => ({ item: o.item, upkeep: o.upkeep ?? null }));
      (expect(options(demo)).toMatchSnapshot(), void (options(native)));
      expect(options(demo).find((o) => o.item.kind === 'Unit' && o.item.unit === 'Scout')?.upkeep).toBeNull();
      (expect(demo.economy.upkeep).toMatchSnapshot(), void (native.economy.upkeep));
      (expect(demo.economy.upkeep_breakdown).toMatchSnapshot(), void (native.economy.upkeep_breakdown));
      expect(demo.economy.upkeep).toBe(Math.max(0, military - 2) * (faction === 'Switzerland' ? 3 : 1));
      (expect(demo.economy.gold_per_turn.total).toMatchSnapshot(), void (native.economy.gold_per_turn.total));
    }
  });

  it('a new game starts every empire with the starting Gold, matching WASM', async () => {
    const setup = makeStandardSetup('700001', 'Switzerland', 'India', 'P0', 'P1');
    const fixtureClient = new FixtureClient(); await fixtureClient.newGame(setup);
    const core = makeCore(); await core.newGame(setup);
    const expected = START_GOLD;
    expect(expected).toBe(60);
    (expect(fixtureClient.view(0).economy.gold).toMatchSnapshot(), void (expected));
    (expect(core.view(0).economy.gold).toMatchSnapshot(), void (expected));
  });

  it('astra-master-042c841: a city starves after resistance before its partner reaches eight', () => {
    const { s, a, b } = tallGrowthFixture();
    const forecast = developmentForecast(s, 0);
    expect(forecast.growth.map((g) => g.turns_to_eight)).toEqual([0, 4]);
    expect(forecast.tall_turns).toBeNull();
    const { reports, state } = play(s, [{}, {}, {}, {}]);
    expect(reports[0].filter((e) => e.type === 'CityStarved' && e.city === a.id)).toEqual([]);
    expect(reports[1]).toContainEqual({ type: 'CityStarved', city: a.id, owner: 0, population: 7 });
    expect(reports[3]).toContainEqual({ type: 'CityGrew', city: b.id, owner: 0, population: 8 });
    expect(state.cities.slice(0, 2).map((c) => c.population)).toEqual([7, 8]);
  });

  it.each([
    { food: 4, resistance: 1, expected: null },
    { food: 6, resistance: 1, expected: 4 },
    { food: 0, resistance: 4, expected: 4 },
  ])('tall overlap with $food stored food and $resistance resistance forecasts $expected turns', ({ food, resistance, expected }) => {
    const { s, a } = tallGrowthFixture();
    a.food = food;
    a.resistance = resistance;
    expect(developmentForecast(s, 0).tall_turns).toBe(expected);
    const { state } = play(s, [{}, {}, {}, {}]);
    expect(state.cities.slice(0, 2).every((c) => c.population >= 8)).toBe(expected === 4);
  });

  it('forecasts a later tall overlap after starvation and regrowth', () => {
    const { s, a } = tallGrowthFixture();
    s.terrain[idxOf(s, a.pos)] = 'Grassland';
    const expected = 2 + growthThreshold(7);
    expect(developmentForecast(s, 0).tall_turns).toBe(expected);
  });

  it('includes the forecast horizon and an existing pair, and excludes other owners', () => {
    const { s, a, b, surplus } = tallGrowthFixture();
    // This pop-8 city keeps its size for the whole horizon: the same five Grassland tiles that give the partner its surplus.
    within(a.pos, 1).filter((h) => !hexEq(h, a.pos)).slice(0, 5).forEach((h) => { s.terrain[idxOf(s, h)] = 'Grassland'; });
    a.resistance = 0;
    b.food = growthThreshold(7) - surplus;
    const horizon = 300;
    const check = (expected: number | null) => {
      expect(developmentForecast(s, 0).tall_turns).toBe(expected);
    };
    b.resistance = horizon - 1;
    check(horizon);
    b.resistance = horizon;
    check(null);
    b.population = 8;
    check(0);
    b.owner = 1;
    check(null);
  });

  it('buys any unit while a market keeps progressing, with a persisted queue', () => {
    const { s, a } = fixture();
    const market = { kind: 'Building', building: 'Workshop' } as const;
    const monument = { kind: 'Building', building: 'Granary' } as const;
    const price = productionCost(s, 0, spearman) * 2;
    const { reports: [events], state } = play(s, [{ 0: [
      { type: 'SetProductionQueue', city: a.id, queue: [market, monument] },
      { type: 'BuyItem', city: a.id, item: spearman },
    ] }, {}]);
    expect(purchases(events)).toEqual([{ type: 'ItemBought', city: a.id, owner: 0, item: spearman, gold: price }]);
    expect(state.cities[0].producing).toEqual(market);
    expect(state.cities[0].productionQueue).toEqual([monument]);
    // The Spearman's 15 stays banked with it; the Market starts from its own (empty) bank and
    // has two turns of Production.
    expect(state.cities[0].productionBanked).toEqual([{ item: spearman, progress: 15 }]);
    expect(state.cities[0].prod).toBe(2 * cityYields(state, state.cities[0]).total.production);
    expect(state.units.filter((u) => u.owner === 0 && u.kind === 'Spearman')).toHaveLength(1);
  });

  it('does not charge a blocked purchase, including after save/load', () => {
    const { s, a, b } = fixture();
    block(s, a, 0);
    s.players[0].gold = productionCost(s, 0, spearman) * 2;
    const { reports } = play(s, [{ 0: [{ type: 'BuyItem', city: a.id, item: spearman }, { type: 'BuyItem', city: b.id, item: spearman }] },
      { 0: [{ type: 'BuyItem', city: a.id, item: spearman }] }]);
    (expect(purchases(reports[0]).map((e) => e.city)).toMatchSnapshot(), void ([b.id]));
    expect(failures(reports[0])[0].reason).toContain('spawn');
    expect(failures(reports[1])[0].reason).toContain('spawn');
  });

  it('a pending move cannot free spawn space until after the purchase phase', () => {
    const { s, a } = fixture();
    block(s, a, 0);
    const u = s.units[0];
    const { reports: [events] } = play(s, [{ 0: [
      { type: 'BuyItem', city: a.id, item: spearman }, { type: 'MoveUnit', unit: u.id, to: { q: u.pos.q - 1, r: u.pos.r } },
    ] }]);
    expect(purchases(events)).toEqual([]);
    expect(failures(events)[0].reason).toContain('spawn');
  });

  it('a disband frees spawn space before buying, regardless of list position', () => {
    const { s, a } = fixture();
    const u = block(s, a, 0);
    const { reports: [events] } = play(s, [{ 0: [{ type: 'BuyItem', city: a.id, item: spearman }, { type: 'DisbandUnit', unit: u.id }] }]);
    expect(failures(events)).toEqual([]);
    expect(purchases(events)).toHaveLength(1);
  });

  it.each(GREAT_WORKS)('starting %s announces its actual route once and reserves the empire slot', (building) => {
    const { s, a, b, item } = greatWorkFixture(building);
    const production = cityYields(s, a, item).total.production;
    const { reports: [events], state } = play(s, [{ 0: [
      { type: 'SetProduction', city: a.id, item },
      { type: 'SetProduction', city: a.id, item },
      { type: 'SetProduction', city: b.id, item },
    ] }]);
    expect(events.filter((e) => e.type === 'MegaprojectStarted')).toEqual([
      { type: 'MegaprojectStarted', city: a.id, owner: 0, building },
    ]);
    expect(failures(events)).toEqual([
      { type: 'OrderFailed', player: 0, reason: `B cannot produce ${BUILDING_NAME[building]}: World wonder under construction in A; only one per empire` },
    ]);
    expect(state.cities[0].producing).toEqual(item);
    expect(state.cities[0].prod).toBe(a.prod + production);
  });

  it.each(GREAT_WORKS)('%s cannot be bought even with sufficient gold and support', (building) => {
    const { s, a, item } = greatWorkFixture(building);
    const { reports: [events] } = play(s, [{ 0: [
      { type: 'SetProduction', city: a.id, item }, { type: 'BuyItem', city: a.id, item },
    ] }]);
    expect(failures(events)).toEqual([
      { type: 'OrderFailed', player: 0, reason: `${BUILDING_NAME[building]} cannot be bought` },
    ]);
    expect(purchases(events)).toEqual([]);
    expect(completions(events)).toEqual([]);
  });

  it.each(['GreatLibrary', 'GrandBazaar', 'RoyalArsenal'] as const)('%s loses support without adding production and banks its earned progress', (building) => {
    const { s, a, b, item } = greatWorkFixture(building);
    a.producing = item;
    a.prod = 100;
    expect(canProduce(s, a, item)).toBe(true);
    if (building === 'GrandBazaar') b.owner = 1;
    else a.buildings = [];
    const reason = buildingBlocker(s, a, building, false);
    expect(reason).toBeTruthy();
    const { reports, state } = play(s, [{}, {}]);
    expect(reports.flatMap(failures)).toEqual([
      { type: 'OrderFailed', player: 0, reason: `A can no longer produce ${BUILDING_NAME[building]}: ${reason}; its 100 Production stays banked` },
    ]);
    expect(reports.flatMap(completions)).toEqual([]);
    expect(state.cities[0].producing).toBeNull();
    // The 100 waits for the Great Work; the idle city banks new Production up to the idle cap.
    expect(state.cities[0].productionBanked).toEqual([{ item, progress: 100 }]);
    expect(state.cities[0].prod).toBeLessThanOrEqual(IDLE_PRODUCTION_CAP);
    expect(state.cities[0].bought).toBeNull();
  });

  it.each(GREAT_WORKS)('%s completes with held VP and stays locked after save/load', (building) => {
    const { s, a, b, item } = greatWorkFixture(building);
    a.prod = productionCost(s, 0, item) - 1;
    const { reports, nativeReports, state, core } = play(s, [
      { 0: [{ type: 'SetProduction', city: a.id, item }] },
      { 0: [{ type: 'SetProduction', city: b.id, item }] },
    ]);
    expect(reports.flatMap(completions)).toEqual([{ type: 'ProductionCompleted', city: a.id, owner: 0, item }]);
    expect(nativeReports.flat().filter((e) => e.type === 'GreatWorkCompleted')).toEqual([
      { type: 'GreatWorkCompleted', player: 0, of: 0, building, vp: 2, again: false },
    ]);
    expect(core.view(0).players[0].vp_sources).toContainEqual([{ kind: 'GreatWork', of: 0 }, 2]);
    expect(failures(reports[1])).toHaveLength(1);
    expect(state.players[0].greatWorkCompleted).toBe(building);
  });

  it('buy then switch then buy still spends only once and banks the earned progress', () => {
    const { s, a } = fixture();
    const { reports: [events], state } = play(s, [{ 0: [
      { type: 'BuyItem', city: a.id, item: spearman }, { type: 'SetProduction', city: a.id, item: settler }, { type: 'BuyItem', city: a.id, item: settler },
    ] }]);
    expect(purchases(events)).toHaveLength(1);
    expect(failures(events)[0].reason).toContain('already bought');
    expect(state.cities[0].productionBanked).toEqual([{ item: spearman, progress: 15 }]);
    expect(state.cities[0].prod).toBe(5);
  });

  it.each([false, true])('Settlers escalate immediately; first blocked=%s', (blocked) => {
    const { s, a, b } = fixture();
    if (blocked) block(s, a, 0, 'Settler');
    const { reports, state } = play(s, [{ 0: [{ type: 'BuyItem', city: a.id, item: settler }, { type: 'BuyItem', city: b.id, item: settler }] }]);
    expect(purchases(reports[0]).map((e) => e.gold)).toEqual(blocked ? [126] : [126, 216]); // Military Science discounts land units by 10%
    expect(state.players[0].settlersTrained).toBe(blocked ? 1 : 2);
  });

  it('Sparta pays its premium on every Settler, the first after the empire discount', () => {
    const { s, a, b } = fixture();
    s.players[0].faction = 'Sparta';
    const { reports, state } = play(s, [{ 0: [
      { type: 'BuyItem', city: a.id, item: settler },
      { type: 'BuyItem', city: b.id, item: settler },
    ] }]);
    expect(purchases(reports[0]).map((e) => e.gold)).toEqual([188, 324]);
    expect(state.players[0].settlersTrained).toBe(2);
  });

  it('a Settler purchase requires population two before growth', () => {
    const { s, a } = fixture();
    a.population = 1;
    a.food = growthThreshold(a.population);
    const { reports } = play(s, [{ 0: [{ type: 'BuyItem', city: a.id, item: settler }] }, { 0: [{ type: 'BuyItem', city: a.id, item: settler }] }]);
    expect(purchases(reports[0])).toEqual([]);
    expect(failures(reports[0])[0].reason).toContain('population');
    expect(purchases(reports[1])).toHaveLength(1);
  });

  it('a bought building supplies yields this turn', () => {
    const { s, a } = fixture();
    a.population = 4;
    a.buildings = ['Granary'];
    const hydro = { kind: 'Building', building: 'Mill' } as const;
    a.food = growthThreshold(a.population) - (cityYields(s, a).total.food - 2 * a.population) - 1;
    const { reports } = play(s, [{ 0: [{ type: 'BuyItem', city: a.id, item: hydro }] }]);
    expect(completions(reports[0])).toContainEqual({ type: 'ProductionCompleted', city: a.id, owner: 0, item: hydro });
    expect(reports[0]).toContainEqual({ type: 'CityGrew', city: a.id, owner: 0, population: 5 });
  });

  it('upgrades spend before purchases even when listed after BuyItem', () => {
    const { s, a } = fixture();
    const u = spawnUnit(s, 0, 'Spearman', a.pos, false);
    s.players[0].gold = upgradeCost(s, u)! + buyCost(s, a)! - 1;
    // Spearman→Pikeman also uses 1 Iron supply: an Iron deposit inside A's border.
    (s.resource ??= s.terrain.map(() => null))[idxOf(s, fromOffset(5, 4))] = 'Iron';
    s.players[0].techs.push('BronzeWorking');
    const { reports: [events] } = play(s, [{ 0: [{ type: 'BuyItem', city: a.id, item: spearman }, { type: 'UpgradeUnit', unit: u.id }] }]);
    expect(purchases(events)).toEqual([]);
    expect(failures(events)).toHaveLength(1);
  });

  it('resistance blocks buying, then full-price buying preserves natural overflow', () => {
    const { s, a } = fixture();
    a.resistance = 1;
    a.prod = 100;
    const { reports, state } = play(s, [
      { 0: [{ type: 'BuyItem', city: a.id, item: spearman }] },
      { 0: [{ type: 'BuyItem', city: a.id, item: spearman }, { type: 'BuyItem', city: a.id, item: spearman }] },
    ]);
    expect(failures(reports[0])[0].reason).toContain('resisting');
    expect(purchases(reports[1])[0].gold).toBe(144);
    expect(failures(reports[1])).toHaveLength(1);
    expect(state.cities[0].prod).toBe(33);
  });

  it('buying a building unlocks its successor during the order phase', () => {
    const { s, a } = fixture();
    a.buildings = ['Workshop'];
    a.population = 6;
    // The Forge uses 1 Iron and the Factory 1 Coal: a deposit of each inside the border.
    (s.resource ??= s.terrain.map(() => null))[idxOf(s, neighbors(a.pos)[0])] = 'Iron';
    s.resource[idxOf(s, neighbors(a.pos)[1])] = 'Coal';
    const machine = { kind: 'Building', building: 'Forge' } as const;
    const auto = { kind: 'Building', building: 'Factory' } as const;
    const { reports: [events], state } = play(s, [{ 0: [{ type: 'BuyItem', city: a.id, item: machine }, { type: 'SetProduction', city: a.id, item: auto }] }]);
    expect(failures(events)).toEqual([]);
    expect(completions(events)[0].item).toEqual(machine);
    expect(state.cities[0].producing).toEqual(auto);
  });

  it('Swiss full-price purchases and upkeep match', () => {
    const { s, a } = fixture();
    s.players[0].faction = s.setup.players[0].faction = 'Switzerland';
    spawnUnit(s, 0, 'Spearman', a.pos, false);
    spawnUnit(s, 0, 'Spearman', fromOffset(5, 4), false);
    const { reports: [events] } = play(s, [{ 0: [{ type: 'BuyItem', city: a.id, item: spearman }] }]);
    expect(purchases(events)[0].gold).toBe(Math.floor(productionCost(s, 0, spearman) * 1.75));
    expect(completions(events)).toHaveLength(1);
  });
  // A Settler needs population 2 even to continue: at population 1 it is banked and dropped
  // (with its reason), not deferred like a spawn-blocked head.
  it.each(['population', 'spawn'] as const)('a %s-blocked head lets the queued building proceed', (blocker) => {
    const { s } = fixture();
    const city = s.cities[0];
    city.producing = blocker === 'population' ? settler : spearman;
    city.productionQueue = [{ kind: 'Building', building: 'Granary' }];
    city.prod = 0;
    city.population = 1;
    city.food = 0;
    if (blocker === 'spawn') for (const h of within(city.pos, 1)) spawnUnit(s, city.owner, 'Spearman', h, false);
    const { fixtureClient, core, reports } = play(s, [{}]);
    if (blocker === 'population') expect(failures(reports[0]).map((e) => e.reason)).toEqual(['A can no longer produce Settler: Settler needs population 2 (currently 1)']);
    for (const client of [fixtureClient, core]) {
      const detail = client.view(0).cities.find((c) => c.id === city.id)!.detail!;
      expect(detail.producing).toEqual({ kind: 'Building', building: 'Granary' });
      expect(detail.production_queue).toEqual(blocker === 'population' ? [] : [spearman]);
      expect(detail.production_stored).toBeGreaterThan(0);
    }
  });

  it('banks blocked-only production and reports the skip once across reloads', () => {
    for (const blocker of ['population', 'spawn']) {
      const { s, a } = fixture();
      a.producing = null;
      a.productionQueue = [blocker === 'population' ? settler : spearman];
      a.population = 1;
      a.food = 0;
      a.prod = IDLE_PRODUCTION_CAP - 1;
      if (blocker === 'spawn') block(s, a, a.owner);
      const before = a.prod;
      const production = cityYields(s, a).total.production;
      const { reports, fixtureClient, core } = play(s, [{}, {}, {}]);
      expect(reports.map((events) => failures(events).filter((e) => e.reason.includes('skipped queued')).length)).toEqual([1, 0, 0]);
      if (blocker === 'population') {
        // A queued Settler at population 1 is invalid, not waiting: it is dropped once, and the
        // idle city banks unassigned Production up to the idle cap.
        expect(failures(reports[0]).map((e) => e.reason)).toEqual(['A skipped queued Settler: Settler needs population 2 (currently 1)']);
        for (const client of [fixtureClient, core]) {
          const detail = client.view(0).cities.find((c) => c.id === a.id)!.detail!;
          expect(detail.producing).toBeNull();
          expect(detail.production_queue).toEqual([]);
          expect(detail.idle_production_cap).toBe(IDLE_PRODUCTION_CAP);
          expect(detail.production_stored).toBe(IDLE_PRODUCTION_CAP);
        }
        continue;
      }
      for (const client of [fixtureClient, core]) {
        const detail = client.view(0).cities.find((c) => c.id === a.id)!.detail!;
        expect(detail.producing).toBeNull();
        expect(detail.production_queue).toEqual(a.productionQueue);
        // The cap is the item's price; stored progress is never reduced.
        const item = a.productionQueue![0];
        let stored = before;
        for (let step = 0; step < 3; step++) {
          const cap = productionCost({ ...s, turn: s.turn + step }, 0, item);
          if (stored < cap) stored = Math.min(stored + production, cap);
        }
        const cap = productionCost(fixtureClient.debugState(), 0, item);
        expect(detail.production_stored).toBe(stored);
        expect(detail.idle_production_cap).toBe(cap);
        expect(detail.idle_production_waste).toBe(Math.min(detail.yields.total.production, Math.max(0, detail.yields.total.production - (cap - stored))));
      }
    }
  });

  // Only spawn space blocks a queue head temporarily now; a population-1 Settler is dropped
  // (covered above).
  it.each(['spawn'] as const)('a %s-blocked queue banks to its rising first-item cost, then shows the waste', (blocker) => {
    const { s, a } = fixture();
    const item = spearman;
    const cost = productionCost(s, 0, item);
    a.producing = null;
    a.productionQueue = [item];
    a.population = 1;
    a.food = 0;
    a.prod = cost - 2;
    if (blocker === 'spawn') block(s, a, a.owner);
    const production = cityYields(s, a).total.production;
    const { fixtureClient, core } = play(s, [{}, {}]);
    for (const client of [fixtureClient, core]) {
      const detail = client.view(0).cities.find((c) => c.id === a.id)!.detail!;
      expect(detail.producing).toBeNull();
      let stored = cost - 2;
      for (let step = 0; step < 2; step++) {
        const cap = productionCost({ ...s, turn: s.turn + step }, 0, item);
        if (stored < cap) stored = Math.min(stored + production, cap);
      }
      const cap = productionCost(fixtureClient.debugState(), 0, item);
      expect(detail.production_stored).toBe(stored);
      expect(detail.idle_production_cap).toBe(cap);
      expect(detail.idle_production_waste).toBe(Math.min(detail.yields.total.production, Math.max(0, detail.yields.total.production - (cap - stored))));
    }
  });

});
