import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeAll, describe, expect, it } from 'vitest';
import type { Event, Order } from '../types';
import { fromOffset, within } from '../hex';
import { WasmClient } from '../wasmClient';
import { coreFixture } from '../test/coreFixture';
import { FixtureClient } from './fixtureEngine';
import { createState, makeStandardSetup, spawnUnit } from './fixtureEngine';
import { idxOf, productionCost, refreshVision } from './fixtureEngine';
import type { MCity, MState } from './fixtureState';

const available = existsSync(fileURLToPath(new URL('../wasm-pkg/legatum.js', import.meta.url)));
let makeCore: () => WasmClient;
beforeAll(async () => {
  if (!available) return;
  const mod = await import(/* @vite-ignore */ '../wasm-pkg/legatum.js');
  await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
  makeCore = () => new WasmClient(mod.Game);
});

function fixture(): MState {
  const setup = makeStandardSetup('5300001', 'Sparta', 'Switzerland', 'Oath', 'Toll');
  setup.width = 20; setup.height = 12;
  setup.players = setup.players.slice(0, 2).map((p, id) => ({ ...p, faction: id === 0 ? 'Sparta' : 'Switzerland', team: id, is_human: true }));
  const s = createState(setup);
  s.units = []; s.cities = []; s.nextUnitId = 0; s.nextCityId = 0;
  s.terrain.fill('Grassland'); s.ruin.fill(0); s.owner.fill(-1); s.tileCity.fill(-1);
  for (const p of s.players) { p.explored.fill(1); p.gold = 100; }
  return s;
}

function city(s: MState, owner: number, col: number, row: number, pop = 3): MCity {
  const c: MCity = { id: ++s.nextCityId, owner, founder: owner, name: `City ${s.nextCityId}`,
    pos: fromOffset(col, row), population: pop, food: 0, prod: 0, producing: null, bought: null,
    buildings: [], hp: 200, grownTiles: [], tilesClaimed: 0, isCapital: !s.cities.some((x) => x.owner === owner), foundedTurn: 1, resistance: 0 };
  s.cities.push(c);
  for (const h of within(c.pos, 1)) { s.owner[idxOf(s, h)] = owner; s.tileCity[idxOf(s, h)] = c.id; }
  return c;
}

function pair(s: MState) {
  refreshVision(s);
  const fixtureClient = new FixtureClient(); fixtureClient.load(JSON.stringify(s));
  const core = makeCore(); core.load(coreFixture(s));
  return { fixtureClient, core };
}

const oathEvents = (events: Event[]) => events.filter((e) =>
  ['Battle', 'UnitDestroyed', 'UnitPromoted', 'SpartaKillHeal', 'CityCaptured', 'SpartaConquest'].includes(e.type));

describe.skipIf(!available)('Sparta WASM behavior', () => {
  it('starts every military type at level one and quotes the 50% home cost', () => {
    const s = fixture(); city(s, 0, 4, 4); city(s, 1, 14, 4);
    for (const kind of ['Soldier', 'Scout', 'Ranger', 'Trooper'] as const) spawnUnit(s, 0, kind, fromOffset(3, 4), false);
    const { fixtureClient, core } = pair(s);
    (expect(fixtureClient.view(0).units.filter((u) => u.owner === 0).map((u) => [u.kind, u.level, u.xp, u.xp_next])).toMatchSnapshot(), void (core.view(0).units.filter((u) => u.owner === 0).map((u) => [u.kind, u.level, u.xp, u.xp_next])));
    expect(productionCost(s, 0, { kind: 'Unit', unit: 'Soldier' })).toBe(80);
    expect(productionCost(s, 0, { kind: 'Unit', unit: 'Settler' })).toBe(105);
    const options = (client: FixtureClient | WasmClient) => client.view(0).cities.find((c) => c.owner === 0)!.detail!.options
      .filter((o) => o.item.kind === 'Unit' && ['Soldier', 'Settler'].includes(o.item.unit))
      .map((o) => [o.item, o.cost, o.buy_quote?.production, o.buy_quote?.additional_settler_production]);
    (expect(options(fixtureClient)).toMatchSnapshot(), void (options(core)));
    s.players[0].settlersTrained = 1;
    expect(productionCost(s, 0, { kind: 'Unit', unit: 'Settler' })).toBe(180);
    const second = pair(s);
    (expect(options(second.fixtureClient)).toMatchSnapshot(), void (options(second.core)));
  });

  it('heals nearby regiments on capture without a Gold payout', () => {
    const s = fixture(); city(s, 0, 3, 4); const target = city(s, 1, 8, 4); city(s, 1, 14, 4);
    s.players[1].faction = s.setup.players[1].faction = 'India';
    for (const r of s.diplomacy!.relations) r.state = 'War';
    target.hp = 0; target.prod = 13;
    const attacker = spawnUnit(s, 0, 'Soldier', fromOffset(7, 4), false);
    const near = spawnUnit(s, 0, 'Ranger', fromOffset(6, 4), false); near.hp = 45;
    const far = spawnUnit(s, 0, 'Ranger', fromOffset(2, 2), false); far.hp = 55;
    const { fixtureClient, core } = pair(s);
    const orders: Record<number, Order[]> = { 0: [{ type: 'MoveUnit', unit: attacker.id, to: target.pos }] };
    (expect(oathEvents(fixtureClient.endTurn(orders).events)).toMatchSnapshot(), void (oathEvents(core.endTurn(orders).events)));
    (expect(fixtureClient.view(0).economy.gold).toMatchSnapshot(), void (core.view(0).economy.gold));
    expect(fixtureClient.view(0).units.find((u) => u.id === near.id)?.hp).toBe(100);
    expect(fixtureClient.view(0).units.find((u) => u.id === far.id)?.hp).toBe(55 + 10); // a far regiment only rests: no capture heal
  });

  it('heals after a military kill', () => {
    const s = fixture(); city(s, 0, 3, 4); city(s, 1, 14, 4);
    const ranger = spawnUnit(s, 0, 'Ranger', fromOffset(6, 4), false);
    const enemy = spawnUnit(s, 1, 'Scout', fromOffset(8, 4), false); enemy.hp = 1;
    ranger.hp = 40;
    for (const r of s.diplomacy!.relations) r.state = 'War';
    const pair1 = pair(s);
    const orders: Record<number, Order[]> = { 0: [{ type: 'RangedAttack', unit: ranger.id, target: enemy.pos }] };
    (expect(oathEvents(pair1.fixtureClient.endTurn(orders).events)).toMatchSnapshot(), void (oathEvents(pair1.core.endTurn(orders).events)));
    expect(pair1.fixtureClient.view(0).units.find((u) => u.id === ranger.id)?.hp).toBe(100);
    (expect(pair1.fixtureClient.view(0).units.find((u) => u.id === ranger.id)?.hp).toMatchSnapshot(), void (pair1.core.view(0).units.find((u) => u.id === ranger.id)?.hp));
  });
});
