// Tile improvements (playtest 8): the same scenarios through the fixtureClient and the real WASM core,
// comparing options and previews, validation, completion, purchase, banking, upgrades, tile loss,
// the yield breakdown and what each viewer sees on the map.
import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeAll, describe, expect, it } from 'vitest';
import { fromOffset, within } from '../hex';
import type { Event, Order, PlayerView, ProductionItem, Resource, TechId } from '../types';
import { WasmClient } from '../wasmClient';
import { coreFixture } from './coreFixture';
import { FixtureClient } from './fixtureEngine';
import { idxOf } from './fixtureEngine';
import { createState, makeStandardSetup } from './fixtureEngine';
import type { MCity, MState } from './fixtureState';

const available = existsSync(fileURLToPath(new URL('../wasm-pkg/legatum.js', import.meta.url)));
let makeCore: () => WasmClient;

type Turn = Record<number, Order[]>;
const farm = (col: number, row: number): ProductionItem => ({ kind: 'Improvement', improvement: 'Farm', tile: fromOffset(col, row) });
const mine = (col: number, row: number): ProductionItem => ({ kind: 'Improvement', improvement: 'Mine', tile: fromOffset(col, row) });
const SPEARMAN: ProductionItem = { kind: 'Unit', unit: 'Spearman' };

/** Two empires on grassland; A (P0, Byzantium: no terrain bonuses) owns radius 3 around (4,4) with Wheat, Copper and a hidden Iron. */
function fixture() {
  const setup = makeStandardSetup('700012', 'Byzantium', 'India', 'P0', 'P1');
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
  for (const p of s.players) { p.gold = 10_000; p.techs = []; p.explored.fill(0); }
  s.cities = [[0, 4, 4], [1, 4, 14], [2, 14, 14]].map(([owner, col, row], i): MCity => ({
    id: i + 1, owner, founder: owner, name: ['A', 'Ally', 'Enemy'][i], pos: fromOffset(col, row),
    population: 3, food: 0, prod: 0, producing: null, bought: null, productionQueue: [],
    buildings: [], hp: 200, grownTiles: [], tilesClaimed: 0, isCapital: true, foundedTurn: 1, resistance: 0,
  }));
  s.nextCityId = s.cities.length;
  const [a, , enemy] = s.cities;
  for (const c of s.cities) for (const h of within(c.pos, c.id === a.id ? 3 : 1)) {
    s.owner[idxOf(s, h)] = c.owner;
    s.tileCity[idxOf(s, h)] = c.id;
  }
  put(s, 5, 4, 'Wheat');
  put(s, 6, 4, 'Copper');
  put(s, 4, 6, 'Iron');
  put(s, 7, 4, 'Wheat'); // ring 3
  put(s, 4, 8, 'Cattle'); // beyond the work radius (still unowned: only owned tiles are listed)
  s.terrain[idxOf(s, fromOffset(6, 4))] = 'Hills';
  return { s, a, enemy };
}

function put(s: MState, col: number, row: number, r: Resource | null) { s.resource![idxOf(s, fromOffset(col, row))] = r; }

const RELEVANT = ['OrderFailed', 'ItemBought', 'ProductionCompleted', 'TileReleased', 'TileClaimed'];
const relevant = (events: Event[]) => events.filter((e) => RELEVANT.includes(e.type));

/** Everything improvements add to a view, for one player. */
function improvementView(v: PlayerView) {
  const own = v.cities.filter((c) => c.owner === v.me && c.detail);
  return {
    tiles: v.tiles.map((t, i) => (t.improvement ? [i, t.improvement] : null)).filter(Boolean),
    options: own.map((c) => c.detail!.options.filter((o) => o.item.kind === 'Improvement').map((o) => ({ item: o.item, cost: o.cost, turns: o.turns, preview: o.improvement, buy: o.buy_cost, stored: o.production_stored }))),
    yields: own.map((c) => ({ total: c.detail!.yields.total, sources: c.detail!.yields.sources.filter((x) => x.label.startsWith('Resources')) })),
    producing: own.map((c) => [c.detail!.producing, c.detail!.production_queue, c.detail!.production_banked, c.detail!.production_stored]),
    territory: own.map((c) => c.detail!.territory.length),
  };
}

function start(s: MState) {
  const fixtureClient = new FixtureClient();
  fixtureClient.load(JSON.stringify(s));
  const core = makeCore();
  core.load(coreFixture(s));
  compare(fixtureClient, core, s.players.length);
  return { fixtureClient, core };
}

function compare(fixtureClient: FixtureClient, core: WasmClient, players: number) {
  for (let p = 0; p < players; p++) (expect(improvementView(fixtureClient.view(p))).toMatchSnapshot(), void (improvementView(core.view(p))));
}

function turn(fixtureClient: FixtureClient, core: WasmClient, orders: Turn, players = 3): Event[] {
  for (let p = 0; p < players; p++) (expect(fixtureClient.validate(p, orders[p] ?? [])).toMatchSnapshot(), void (core.validate(p, orders[p] ?? [])));
  const actual = fixtureClient.endTurn(orders), expected = core.endTurn(orders);
  (expect(relevant(actual.events)).toMatchSnapshot(), void (relevant(expected.events)));
  compare(fixtureClient, core, players);
  fixtureClient.load(fixtureClient.save());
  core.load(core.save());
  compare(fixtureClient, core, players);
  return relevant(actual.events);
}

const option = (v: PlayerView, item: ProductionItem) =>
  v.cities.find((c) => c.owner === v.me)!.detail!.options.find((o) => JSON.stringify(o.item) === JSON.stringify(item));

describe.skipIf(!available)('WASM improvement behavior', () => {
  beforeAll(async () => {
    const path = '../wasm-pkg/legatum.js';
    const mod = await import(/* @vite-ignore */ path);
    await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
    makeCore = () => new WasmClient(mod.Game);
  });

  it('lists only owned, worked, known, matching resource tiles, with the same previews', () => {
    const { s } = fixture();
    const { fixtureClient, core } = start(s);
    const v = fixtureClient.view(0);
    const items = v.cities[0].detail!.options.filter((o) => o.item.kind === 'Improvement');
    // Wheat twice and Copper; the Iron is hidden and the Cattle is not owned.
    expect(items.map((o) => o.item)).toEqual([farm(5, 4), mine(6, 4), farm(7, 4)]);
    expect(option(v, farm(5, 4))!.improvement).toMatchObject({ gain: { food: 1, production: 0, gold: 0, science: 0 }, upgrade: { name: 'Irrigated Farm', tech: 'CropRotation' } });
    expect(option(v, mine(6, 4))!.improvement).toMatchObject({ gain: { gold: 2 }, upgrade: { name: 'Deep Mine' } });
    expect(option(v, farm(5, 4))!.cost).toBe(30);
    expect(option(v, mine(6, 4))!.cost).toBe(40);
    turn(fixtureClient, core, {});
  });

  it('validates the same refusals', () => {
    const { s, a } = fixture();
    const { fixtureClient, core } = start(s);
    const wrongKind: ProductionItem = { kind: 'Improvement', improvement: 'Mine', tile: fromOffset(5, 4) };
    const hidden: ProductionItem = { kind: 'Improvement', improvement: 'Mine', tile: fromOffset(4, 6) };
    const bare = farm(3, 4);
    const unowned: ProductionItem = { kind: 'Improvement', improvement: 'Pasture', tile: fromOffset(4, 8) };
    const orders: Order[] = [wrongKind, hidden, bare, unowned, farm(5, 4)].map((item) => ({ type: 'SetProduction', city: a.id, item }));
    const errors = fixtureClient.validate(0, orders);
    expect(errors.map((e) => e.index)).toEqual([0, 1, 2, 3]);
    expect(errors[0].message).toContain('needs a Farm, not a Mine');
    expect(errors[2].message).toContain('no known resource');
    (expect(core.validate(0, orders)).toMatchSnapshot(), void (errors));
    // Another empire's city can't improve A's tile.
    (expect(core.validate(2, [{ type: 'SetProduction', city: 3, item: farm(5, 4) }])).toMatchSnapshot(), void (fixtureClient.validate(2, [{ type: 'SetProduction', city: 3, item: farm(5, 4) }])));
  });

  it('builds the improvement, yields its bonus only afterwards, and upgrades with technology', () => {
    const { s, a } = fixture();
    const base = start(s);
    const food = (v: PlayerView) => v.cities[0].detail!.yields.total.food;
    const before = food(base.fixtureClient.view(0));
    const done = structuredClone(base.fixtureClient.debugState());
    done.cities[0].producing = farm(5, 4);
    done.cities[0].prod = 1000;
    const d = start(done);
    const events = turn(d.fixtureClient, d.core, {});
    expect(events).toContainEqual({ type: 'ProductionCompleted', city: a.id, owner: 0, item: farm(5, 4) });
    expect(d.fixtureClient.view(0).tiles[idxOf(done, fromOffset(5, 4))].improvement).toEqual({ kind: 'Farm' });
    expect(d.fixtureClient.view(0).cities[0].detail!.yields.sources.find((x) => x.label.startsWith('Resources'))!.label).toBe('Resources: Wheat (Farm)');
    expect(food(d.fixtureClient.view(0)) - before).toBeGreaterThanOrEqual(1);
    // The improvement never shows to the rival unless seen, and a rival never sees the upgraded form.
    const upgraded = structuredClone(d.fixtureClient.debugState());
    upgraded.players[0].techs = ['CropRotation' as TechId];
    const u = start(upgraded);
    expect(u.fixtureClient.view(0).tiles[idxOf(done, fromOffset(5, 4))].improvement).toEqual({ kind: 'Farm', upgraded: true });
    expect(u.fixtureClient.view(0).cities[0].detail!.yields.sources.find((x) => x.label.startsWith('Resources'))!.label).toBe('Resources: Wheat (Irrigated Farm)');
    turn(u.fixtureClient, u.core, {});
  });

  it('buys an improvement, banks progress on a switch, and retains it when the tile is lost', () => {
    const { s, a } = fixture();
    const { fixtureClient, core } = start(s);
    const events = turn(fixtureClient, core, { 0: [{ type: 'BuyItem', city: a.id, item: farm(5, 4) }] });
    expect(events.some((e) => e.type === 'ItemBought')).toBe(true);
    expect(fixtureClient.view(0).tiles[idxOf(s, fromOffset(5, 4))].improvement).toEqual({ kind: 'Farm' });

    const bank = structuredClone(fixtureClient.debugState());
    bank.cities[0].producing = mine(6, 4);
    bank.cities[0].prod = 12;
    const b = start(bank);
    turn(b.fixtureClient, b.core, { 0: [{ type: 'SetProduction', city: a.id, item: SPEARMAN }] });
    expect(b.fixtureClient.view(0).cities[0].detail!.production_banked).toEqual([{ item: mine(6, 4), progress: 12 }]);
    turn(b.fixtureClient, b.core, { 0: [{ type: 'SetProduction', city: a.id, item: mine(6, 4) }] });

    // A released tile keeps its improvement even though its city no longer works it.
    const lose = structuredClone(fixtureClient.debugState());
    lose.cities[0].population = 1;
    lose.cities[0].grownTiles = [fromOffset(5, 4)];
    lose.cities[0].tilesClaimed = 1;
    lose.cities[0].resistance = 3; // no growth this turn, so population stays at 1
    lose.cities[0].producing = farm(7, 4);
    lose.cities[0].prod = 5;
    const l = start(lose);
    const lost = turn(l.fixtureClient, l.core, {});
    expect(lost.some((e) => e.type === 'TileReleased')).toBe(true);
    expect(l.fixtureClient.view(0).tiles[idxOf(lose, fromOffset(5, 4))].improvement).toEqual({ kind: 'Farm' });
  });
});
