// City healing (core rules::cities::heal_cities): +20 HP at the end of a turn unless the city took
// damage that turn or an at-war military unit is adjacent. WASM checks for the rule and for the
// city card's forecast (CityDetail.turns_to_heal, heal_blocked).
import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeAll, describe, expect, it } from 'vitest';
import type { Event, Hex, Order } from '../types';
import { fromOffset } from '../hex';
import { WasmClient } from '../wasmClient';
import { coreFixture } from '../test/coreFixture';
import { FixtureClient } from './fixtureEngine';
import { createState, makeStandardSetup, spawnUnit } from './fixtureEngine';
import { idxOf, refreshVision } from './fixtureEngine';
import type { MCity, MState } from './fixtureState';

const available = existsSync(fileURLToPath(new URL('../wasm-pkg/legatum.js', import.meta.url)));
let makeCore: () => WasmClient;
const home = fromOffset(8, 5);

function fixture() {
  const setup = makeStandardSetup('7', 'Switzerland', 'Switzerland', 'A', 'B');
  setup.width = 20; setup.height = 12;
  setup.players = setup.players.slice(0, 2).map((p, i) => ({ ...p, faction: 'Switzerland', team: i, is_human: true }));
  const s = createState(setup);
  for (const relation of s.diplomacy!.relations) relation.state = 'War';
  s.units = []; s.cities = []; s.nextUnitId = 0; s.nextCityId = 0;
  s.terrain.fill('Grassland'); s.ruin.fill(0); s.owner.fill(-1); s.tileCity.fill(-1);
  s.knownOwner = {}; s.knownCities = {}; s.knownRuins = {};
  for (const p of s.players) { p.gold = 10000; p.explored.fill(0); spawnUnit(s, p.id, 'Settler', fromOffset(1 + 6 * p.id, 11), false); }
  return s;
}
function addCity(s: MState, owner: number, pos: Hex, hp: number) {
  const c: MCity = { id: s.nextCityId, owner, founder: owner, name: `City ${s.nextCityId}`, pos, population: 3, food: 0, prod: 0, producing: null,
    bought: null, buildings: [], hp, grownTiles: [], tilesClaimed: 0, isCapital: true, originalCapital: true, foundedTurn: 1, resistance: 0 };
  s.cities.push(c); s.nextCityId += 1; s.owner[idxOf(s, pos)] = owner; s.tileCity[idxOf(s, pos)] = c.id;
  return c;
}

describe.skipIf(!available)('city healing core behavior', () => {
  beforeAll(async () => {
    const pkg = '../wasm-pkg/legatum.js', mod = await import(/* @vite-ignore */ pkg);
    await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
    makeCore = () => new WasmClient(mod.Game);
  });

  /** Both engines from one state; `turn` plays the same orders in each and checks they agree. */
  function start(s: MState) {
    refreshVision(s);
    const fixtureClient = new FixtureClient(); fixtureClient.load(JSON.stringify(s));
    const core = makeCore(); core.load(coreFixture(s));
    const city = (c: FixtureClient | WasmClient) => c.view(0).cities.find((x) => x.id === 0)!;
    return {
      fixtureClient, core,
      city: () => { const m = city(fixtureClient), k = city(core); expect({ hp: m.hp, heal: [m.detail?.turns_to_heal ?? null, m.detail?.heal_blocked ?? false] })
        .toEqual({ hp: k.hp, heal: [k.detail?.turns_to_heal ?? null, k.detail?.heal_blocked ?? false] }); return k; },
      turn: (orders: Record<number, Order[]> = {}) => {
        const a = fixtureClient.endTurn(orders).events, b = core.endTurn(orders).events;
        const battles = (es: Event[]) => es.filter((e) => e.type === 'Battle');
        (expect(battles(a)).toMatchSnapshot(), void (battles(b)));
        return b;
      },
    };
  }

  it('an undamaged, unthreatened city heals 20 a turn, caps at full, and forecasts the turns to full', () => {
    const s = fixture(); addCity(s, 0, home, 130);
    const g = start(s);
    expect(g.city().detail).toMatchObject({ turns_to_heal: 4 });
    g.turn(); expect(g.city().hp).toBe(150);
    g.turn(); expect(g.city().hp).toBe(170);
    g.turn(); expect(g.city().hp).toBe(190);
    g.turn(); expect(g.city().hp).toBe(200);
    expect(g.city().detail?.turns_to_heal ?? null).toBeNull();
  });

  it('a city damaged this turn does not heal; the next quiet turn it does', () => {
    const s = fixture(); addCity(s, 0, home, 100);
    const archer = spawnUnit(s, 1, 'Archer', fromOffset(10, 5), false);
    const g = start(s);
    const events = g.turn({ 1: [{ type: 'RangedAttack', unit: archer.id, target: home }] });
    const dealt = events.flatMap((e) => e.type === 'Battle' && e.defender.kind === 'City' ? [e.damage_to_defender] : [])[0];
    expect(dealt).toBeGreaterThan(0);
    expect(g.city().hp).toBe(100 - dealt);
    g.turn(); expect(g.city().hp).toBe(120 - dealt);
  });

  it('an at-war military unit adjacent stops the heal and the forecast says so; an enemy Settler does not', () => {
    const s = fixture(); addCity(s, 0, home, 100);
    spawnUnit(s, 1, 'Settler', fromOffset(8, 6), false);
    const g = start(s);
    expect(g.city().detail?.heal_blocked ?? false).toBe(false);
    g.turn(); expect(g.city().hp).toBe(120);
    const s2 = fixture(); addCity(s2, 0, home, 100);
    spawnUnit(s2, 1, 'Spearman', fromOffset(8, 6), false);
    const blocked = start(s2);
    expect(blocked.city().detail).toMatchObject({ heal_blocked: true });
    blocked.turn(); expect(blocked.city().hp).toBe(100);
  });
});
