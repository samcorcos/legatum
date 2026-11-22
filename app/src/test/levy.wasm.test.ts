// Thin levies (core `economy::levy`): caps, validation errors, events, views and blockers.
import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeAll, describe, expect, it } from 'vitest';
import type { Event, Order, UnitKind } from '../types';
import { fromOffset, within } from '../hex';
import { WasmClient } from '../wasmClient';
import { coreFixture } from './coreFixture';
import { FixtureClient } from './fixtureEngine';
import { idxOf, refreshVision } from './fixtureEngine';
import { createState, makeStandardSetup, meetAll, spawnUnit } from './fixtureEngine';
import type { MCity, MState } from './fixtureState';

const available = existsSync(fileURLToPath(new URL('../wasm-pkg/legatum.js', import.meta.url)));
let makeCore: () => WasmClient;
const relevant = (es: Event[]) => es.filter((e) => ['WarDeclared', 'PeaceEstablished', 'OrderFailed', 'ProposalCreated', 'ProposalAccepted', 'ProposalInvalidated', 'ProposalRejected'].includes(e.type));

/** Team 0 (player 0, Byzantium) at War with team 1 (Switzerland); team 2 (India) at Peace with both, everyone met. */
function world(byzantium = true) {
  const setup = makeStandardSetup('4417', byzantium ? 'Byzantium' : 'India', 'Switzerland', 'A', 'B');
  setup.width = setup.height = 20;
  setup.players = setup.players.slice(0, 3).map((p, id) => ({ ...p, team: id, is_human: true, faction: (['Byzantium', 'Switzerland', 'India'] as const)[id] === 'Byzantium' && !byzantium ? 'India' : (['Byzantium', 'Switzerland', 'India'] as const)[id] }));
  const s = createState(setup);
  s.units = []; s.nextUnitId = 0; s.cities = []; s.nextCityId = 0;
  s.terrain.fill('Grassland'); s.ruin.fill(0); s.owner.fill(-1); s.tileCity.fill(-1);
  s.cities = [[0, 3, 3], [1, 15, 15], [2, 15, 3]].map(([owner, col, row], i): MCity => {
    const c: MCity = { id: i + 1, owner, founder: owner, name: `C${owner}`, pos: fromOffset(col, row), population: 2, food: 0, prod: 0, producing: null, bought: null, buildings: [], hp: 200, grownTiles: [], tilesClaimed: 0, isCapital: true, foundedTurn: 1, resistance: 0 };
    for (const h of within(c.pos, 1)) { const t = idxOf(s, h); s.owner[t] = owner; s.tileCity[t] = c.id; }
    return c;
  });
  s.nextCityId = s.cities.length;
  Object.assign(s.diplomacy!.relations.find((r) => r.a === 0 && r.b === 1)!, { state: 'War', war_id: 1, declarers: [0], effective_turn: 1 });
  s.diplomacy!.next_war_id = 1;
  for (const p of s.players) p.gold = 600;
  meetAll(s);
  const army = (owner: number, kinds: UnitKind[]) => kinds.forEach((kind, i) => spawnUnit(s, owner, kind, fromOffset(8 + i, 5 + owner), false));
  army(1, ['Spearman', 'Spearman', 'Spearman']);
  army(2, ['Spearman', 'Spearman', 'Archer']);
  return s;
}

function pair(s: MState) {
  refreshVision(s);
  const fixtureClient = new FixtureClient(); fixtureClient.load(JSON.stringify(s));
  const core = makeCore(); core.load(coreFixture(s));
  return { fixtureClient, core };
}
type Pair = ReturnType<typeof pair>;

function sameViews(p: Pair, players = [0, 1, 2]) {
  // Rules 45's starting capitals reveal more land before this arranged scene is loaded.
  // The exploration snapshot is repinned for that initial sight.
  for (const id of players) {
    (expect(p.fixtureClient.view(id).diplomacy).toMatchSnapshot(), void (p.core.view(id).diplomacy));
    (expect(p.fixtureClient.view(id).economy.gold).toMatchSnapshot(), void (p.core.view(id).economy.gold));
  }
}

function step(p: Pair, orders: Record<number, Order[]>, players = [0, 1, 2]) {
  for (const id of players) (expect(p.fixtureClient.validate(id, orders[id] ?? [])).toMatchSnapshot(), void (p.core.validate(id, orders[id] ?? [])));
  const actual = p.fixtureClient.endTurn(orders), expected = p.core.endTurn(orders);
  (expect(relevant(actual.events)).toMatchSnapshot(), void (relevant(expected.events)));
  for (const id of players) (expect(relevant(actual.events_by_player?.[id] ?? [])).toMatchSnapshot(), void (relevant(expected.events_by_player?.[id] ?? [])));
  sameViews(p, players);
  p.fixtureClient.load(p.fixtureClient.save()); p.core.load(p.core.save());
  return { actual, expected };
}

describe.skipIf(!available)('thin levies: core behavior', () => {
  beforeAll(async () => {
    const path = '../wasm-pkg/legatum.js'; const mod = await import(/* @vite-ignore */ path);
    await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
    makeCore = () => new WasmClient(mod.Game);
  });

  const levyOf = (c: FixtureClient | WasmClient, id = 0) => c.view(0).players.find((p) => p.id === id)?.levy;
  const unavailable = (c: FixtureClient | WasmClient) => c.view(0).cities.find((x) => x.owner === 0)!.detail!.unavailable.map((u) => [JSON.stringify(u.item), u.reason]);

  it('caps armed units at one per city plus one, with Scouts free, and greys armed units with the same reason', () => {
    const s = world();
    for (const relation of s.diplomacy!.relations) relation.state = 'Peace';
    s.units = s.units.filter((u) => u.owner !== 0);
    spawnUnit(s, 0, 'Scout', fromOffset(5, 5), false);
    spawnUnit(s, 0, 'Spearman', fromOffset(4, 3), false);
    s.players[0].techs = ['BronzeWorking', 'Masonry', 'Feudalism'];
    let p = pair(s);
    expect(levyOf(p.core)).toEqual({ units: 1, cap: 2 });
    (expect(levyOf(p.fixtureClient)).toMatchSnapshot(), void (levyOf(p.core)));
    (expect(unavailable(p.fixtureClient)).toMatchSnapshot(), void (unavailable(p.core)));
    expect(unavailable(p.core).some(([, reason]) => /Levies full/.test(reason))).toBe(false);
    // At the cap: every armed unit is greyed "Levies full: 2 of 2", and buying one is refused.
    spawnUnit(s, 0, 'Archer', fromOffset(2, 3), false);
    p = pair(s);
    expect(levyOf(p.core)).toEqual({ units: 2, cap: 2 });
    expect(levyOf(p.fixtureClient)).toEqual({ units: 2, cap: 2 });
    (expect(unavailable(p.fixtureClient)).toMatchSnapshot(), void (unavailable(p.core)));
    const full = unavailable(p.core).filter(([, reason]) => reason === 'Levies full: 2 of 2');
    expect(full.length).toBeGreaterThan(0);
    expect(full.every(([item]) => !item.includes('Scout') && !item.includes('Settler'))).toBe(true);
    const city = p.core.view(0).cities.find((c) => c.owner === 0)!;
    const buy: Order[] = [{ type: 'BuyItem', city: city.id, item: { kind: 'Unit', unit: 'Spearman' } }];
    (expect(p.fixtureClient.validate(0, buy)).toMatchSnapshot(), void (p.core.validate(0, buy)));
    expect(p.core.validate(0, buy)).toHaveLength(1);
    const produce: Order[] = [{ type: 'SetProduction', city: city.id, item: { kind: 'Unit', unit: 'Spearman' } }];
    (expect(p.fixtureClient.validate(0, produce)).toMatchSnapshot(), void (p.core.validate(0, produce)));
    // Other factions are uncapped.
    expect(levyOf(p.core, 1)).toBeUndefined();
    expect(levyOf(p.fixtureClient, 1)).toBeUndefined();
  });

  it('losing a city never disbands units; it only blocks new ones (same turn results)', () => {
    const s = world();
    for (const relation of s.diplomacy!.relations) relation.state = 'Peace';
    s.units = s.units.filter((u) => u.owner !== 0);
    // A second city, three Soldiers: 3 of 3. Then the second city is gone: 3 of 2.
    s.cities.push({ id: 9, owner: 0, founder: 0, name: 'Second', pos: fromOffset(3, 12), population: 2, food: 0, prod: 0, producing: null, bought: null, buildings: [], hp: 200, grownTiles: [], tilesClaimed: 0, isCapital: false, foundedTurn: 1, resistance: 0 });
    s.nextCityId = 9;
    for (let i = 0; i < 3; i++) spawnUnit(s, 0, 'Spearman', fromOffset(4 + i, 6), false);
    let p = pair(s);
    expect(levyOf(p.core)).toEqual({ units: 3, cap: 3 });
    s.cities = s.cities.filter((c) => c.id !== 9);
    p = pair(s);
    expect(levyOf(p.core)).toEqual({ units: 3, cap: 2 });
    expect(levyOf(p.fixtureClient)).toEqual({ units: 3, cap: 2 });
    const { actual, expected } = step(p, {}, [0, 1, 2]);
    (expect(relevant(actual.events)).toMatchSnapshot(), void (relevant(expected.events)));
    expect(levyOf(p.core)).toEqual({ units: 3, cap: 2 });
    expect(levyOf(p.fixtureClient)).toEqual({ units: 3, cap: 2 });
    expect(p.core.view(0).units.filter((u) => u.owner === 0 && u.military)).toHaveLength(3);
    expect(p.fixtureClient.view(0).units.filter((u) => u.owner === 0 && u.military)).toHaveLength(3);
    (expect(unavailable(p.fixtureClient)).toMatchSnapshot(), void (unavailable(p.core)));
    expect(unavailable(p.core).some(([, reason]) => reason === 'Levies full: 3 of 2')).toBe(true);
  });

  it('skips a unit whose levy filled while it was being built, with the same notice', () => {
    const s = world();
    for (const relation of s.diplomacy!.relations) relation.state = 'Peace';
    s.units = s.units.filter((u) => u.owner !== 0);
    s.players[0].techs = ['BronzeWorking', 'Masonry'];
    const c = s.cities.find((x) => x.owner === 0)!;
    c.producing = { kind: 'Unit', unit: 'Spearman' }; c.prod = 10;
    spawnUnit(s, 0, 'Spearman', fromOffset(4, 3), false);
    spawnUnit(s, 0, 'Spearman', fromOffset(2, 3), false);
    const p = pair(s);
    const { expected } = step(p, {}, [0]);
    const failed = expected.events.filter((e): e is Extract<Event, { type: 'OrderFailed' }> => e.type === 'OrderFailed' && /Levies full/.test(e.reason));
    expect(failed.length).toBeGreaterThan(0);
  });
});
