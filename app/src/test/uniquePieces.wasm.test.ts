// WASM checks for the Spartan Hoplite phalanx.
import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeAll, describe, expect, it } from 'vitest';
import type { Faction, Order } from '../types';
import { fromOffset, neighbors, within } from '../hex';
import { HOPLITE_PHALANX_STRENGTH } from '../data';
import { PHALANX_LABEL } from '../combat';
import { WasmClient } from '../wasmClient';
import { coreFixture } from '../test/coreFixture';
import { FixtureClient } from './fixtureEngine';
import { idxOf, refreshVision } from './fixtureEngine';
import { createState, makeStandardSetup, spawnUnit } from './fixtureEngine';
import type { MCity, MState } from './fixtureState';

const available = existsSync(fileURLToPath(new URL('../wasm-pkg/legatum.js', import.meta.url)));
let makeCore: () => WasmClient;
beforeAll(async () => {
  if (!available) return;
  const mod = await import(/* @vite-ignore */ '../wasm-pkg/legatum.js');
  await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
  makeCore = () => new WasmClient(mod.Game);
});

function fixture(first: Faction, second: Faction, war: boolean): MState {
  const setup = makeStandardSetup('5300007', first, second, 'A', 'B');
  setup.width = 20; setup.height = 12;
  setup.players = setup.players.slice(0, 2).map((p, id) => ({ ...p, faction: id === 0 ? first : second, team: id, is_human: true }));
  const s = createState(setup);
  s.units = []; s.cities = []; s.nextUnitId = 0; s.nextCityId = 0;
  s.terrain.fill('Grassland'); s.ruin.fill(0); s.owner.fill(-1); s.tileCity.fill(-1);
  for (const p of s.players) { p.explored.fill(1); p.gold = 100; }
  for (const r of s.diplomacy?.relations ?? []) r.state = war ? 'War' : 'Peace';
  return s;
}

function city(s: MState, owner: number, col: number, row: number, buildings: MCity['buildings'] = []): MCity {
  const c: MCity = { id: ++s.nextCityId, owner, founder: owner, name: `City ${s.nextCityId}`,
    pos: fromOffset(col, row), population: 3, food: 0, prod: 0, producing: null, bought: null,
    buildings, hp: 200, grownTiles: [], tilesClaimed: 0, isCapital: !s.cities.some((x) => x.owner === owner), foundedTurn: 1, resistance: 0 };
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

describe.skipIf(!available)('Hoplite phalanx: core agree', () => {
  function board(withPartner: boolean, partnerKind: 'Spearman' | 'Phalangite' = 'Spearman') {
    const s = fixture('Sparta', 'Athens', true);
    city(s, 0, 2, 5); city(s, 1, 17, 5);
    const hoplite = spawnUnit(s, 0, 'Spearman', fromOffset(9, 5), false);
    const enemy = spawnUnit(s, 1, 'Spearman', neighbors(hoplite.pos).filter((h) => h.q > hoplite.pos.q)[0], false);
    if (withPartner) spawnUnit(s, 0, partnerKind, neighbors(hoplite.pos).filter((h) => h.q < hoplite.pos.q)[0], false);
    return { s, hoplite, enemy };
  }
  const line = [PHALANX_LABEL, HOPLITE_PHALANX_STRENGTH] as [string, number];

  it('the forecast carries the same phalanx line on both sides of the attack', () => {
    for (const [partner, kind] of [[false, 'Spearman'], [true, 'Spearman'], [true, 'Phalangite']] as const) {
      const { s, hoplite, enemy } = board(partner, kind);
      const { fixtureClient, core } = pair(s);
      const attack = core.forecast(0, hoplite.id, enemy.pos, 'Melee')!;
      (expect(fixtureClient.forecast(0, hoplite.id, enemy.pos, 'Melee'), `${partner} ${kind}`).toMatchSnapshot(), void (attack));
      expect(attack.attacker_strength.parts.some(([l, n]) => l === line[0] && n === line[1])).toBe(partner && kind === 'Spearman');
      // Defending: the rival's forecast against the Hoplite.
      const defence = core.forecast(1, enemy.id, hoplite.pos, 'Melee')!;
      (expect(fixtureClient.forecast(1, enemy.id, hoplite.pos, 'Melee'), `defence ${partner} ${kind}`).toMatchSnapshot(), void (defence));
      expect(defence.defender_strength.parts.some(([l, n]) => l === line[0] && n === line[1])).toBe(partner && kind === 'Spearman');
    }
  });

  it('a resolved fight reports the same strengths in the fixtureClient as in the core', () => {
    const { s, hoplite, enemy } = board(true);
    const { fixtureClient, core } = pair(s);
    const orders: Record<number, Order[]> = { 0: [], 1: [{ type: 'MoveUnit', unit: enemy.id, to: hoplite.pos }] };
    const coreEvents = core.endTurn(orders).events;
    const mockEvents = fixtureClient.endTurn(orders).events;
    const battle = (events: typeof coreEvents) => events.find((e) => e.type === 'Battle');
    expect(battle(mockEvents)).toEqual(battle(coreEvents));
    expect(JSON.stringify(battle(coreEvents))).toContain(PHALANX_LABEL);
  });
});
