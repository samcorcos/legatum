// The unit triangle ends at the World War era (user, 2026-10-20; core combat::triangle_applies): a class bonus
// needs both units from before it. WASM checks of the forecast and the armour's moves.
import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeAll, describe, expect, it } from 'vitest';
import type { Hex } from '../types';
import { fromOffset } from '../hex';
import { WasmClient } from '../wasmClient';
import { coreFixture } from '../test/coreFixture';
import { FixtureClient } from './fixtureEngine';
import { createState, makeStandardSetup, spawnUnit } from './fixtureEngine';
import { refreshVision } from './fixtureEngine';
import type { MState } from './fixtureState';
import { AUTONOMOUS_ARMOUR_MOVES, RULES, TANK_MOVES, unitStatsFor } from '../data';
import { MELEE_VS_MOUNTED_LABEL, MOUNTED_VS_RANGED_LABEL, classBonusStrength } from '../combat';

const available = existsSync(fileURLToPath(new URL('../wasm-pkg/legatum.js', import.meta.url)));
let makeCore: () => WasmClient;

function fixture() {
  const setup = makeStandardSetup('7', 'Switzerland', 'Switzerland', 'A', 'B');
  setup.width = 20; setup.height = 12;
  setup.players = setup.players.slice(0, 2).map((p, i) => ({ ...p, faction: 'Switzerland', team: i, is_human: true }));
  const s = createState(setup);
  for (const relation of s.diplomacy!.relations) relation.state = 'War';
  s.units = []; s.cities = []; s.nextUnitId = 0; s.nextCityId = 0;
  s.terrain.fill('Grassland'); s.ruin.fill(0); s.owner.fill(-1); s.tileCity.fill(-1);
  s.knownOwner = {}; s.knownCities = {}; s.knownRuins = {};
  for (const p of s.players) { p.explored.fill(0); spawnUnit(s, p.id, 'Settler', fromOffset(1 + 6 * p.id, 11), false); }
  return s;
}
const forecasts = (s: MState, id: number, to: Hex) => {
  refreshVision(s);
  const fixtureClient = new FixtureClient(); fixtureClient.load(JSON.stringify(s));
  const core = makeCore(); core.load(coreFixture(s));
  const f = core.forecast(0, id, to, 'Melee');
  expect(fixtureClient.forecast(0, id, to, 'Melee')).toMatchSnapshot();
  return f!;
};

describe.skipIf(!available)('triangle core behavior', () => {
  beforeAll(async () => {
    const pkg = '../wasm-pkg/legatum.js', mod = await import(/* @vite-ignore */ pkg);
    await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
    makeCore = () => new WasmClient(mod.Game);
  });

  it('a class bonus needs both units from before the Modern era, in the forecast of both engines', () => {
    const at = { q: 8, r: 5 }, there = { q: 9, r: 5 };
    for (const [meleeKind, other, melee, mounted] of [
      ['Rifleman', 'Cavalry', true, false], ['Rifleman', 'Tank', false, false], ['Infantry', 'Cavalry', false, false], ['Infantry', 'Tank', false, false],
      ['Cavalry', 'FieldCannon', false, true], ['Cavalry', 'Artillery', false, false], ['Tank', 'FieldCannon', false, false],
    ] as const) {
      const s = fixture();
      const attacker = spawnUnit(s, 0, meleeKind, at, false);
      spawnUnit(s, 1, other, there, false);
      const parts = forecasts(s, attacker.id, there).attacker_strength.parts;
      const label = `${meleeKind} vs ${other}`;
      const bonus = classBonusStrength(unitStatsFor('Switzerland', meleeKind).strength);
      expect(parts.some(([l, v]) => l === MELEE_VS_MOUNTED_LABEL && v === bonus), label).toBe(melee);
      expect(parts.some(([l, v]) => l === MOUNTED_VS_RANGED_LABEL && v === bonus), label).toBe(mounted);
    }
  });

  it('the Tank has five moves, the Autonomous Armour six and the rest of the mounted class four', () => {
    expect(unitStatsFor('India', 'Tank').moves).toBe(TANK_MOVES);
    expect(TANK_MOVES).toBe(RULES.tankMoves);
    expect(unitStatsFor('India', 'AutonomousArmour').moves).toBe(AUTONOMOUS_ARMOUR_MOVES);
    expect(AUTONOMOUS_ARMOUR_MOVES).toBeGreaterThan(TANK_MOVES);
    expect(unitStatsFor('India', 'Cavalry').moves).toBe(RULES.mountedMoves);
  });
});
