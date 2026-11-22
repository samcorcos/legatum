// The capitals victory (core scoring::holds_own_capital): the only team still holding an original capital of its
// own wins at the end of the turn, and the public view says which empires' capitals stand.
import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeAll, describe, expect, it } from 'vitest';
import type { Hex } from '../types';
import { fromOffset } from '../hex';
import { WasmClient } from '../wasmClient';
import { coreFixture } from './coreFixture';
import { createState, makeStandardSetup, spawnUnit, idxOf, refreshVision } from './fixtureEngine';
import type { MCity, MState } from './fixtureState';

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
function addCapital(s: MState, owner: number, pos: Hex) {
  const c: MCity = { id: s.nextCityId, owner, founder: owner, name: `City ${s.nextCityId}`, pos, population: 3, food: 0, prod: 0, producing: null,
    bought: null, buildings: [], hp: 200, grownTiles: [], tilesClaimed: 0, isCapital: true, originalCapital: true, foundedTurn: 1, resistance: 0 };
  s.cities.push(c); s.nextCityId += 1; s.owner[idxOf(s, pos)] = owner; s.tileCity[idxOf(s, pos)] = c.id;
  return c;
}

describe.skipIf(!available)('capitals victory core behavior', () => {
  beforeAll(async () => {
    const pkg = '../wasm-pkg/legatum.js', mod = await import(/* @vite-ignore */ pkg);
    await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
    makeCore = () => new WasmClient(mod.Game);
  });

  it('both capitals standing: the public view says so and nobody wins', () => {
    const s = fixture(); addCapital(s, 0, fromOffset(4, 5)); addCapital(s, 1, fromOffset(14, 5));
    refreshVision(s);
    const core = makeCore(); core.load(coreFixture(s));
    expect(core.view(0).players.map((p) => p.capital_stands)).toEqual([true, true]);
    const r = core.endTurn({});
    expect(r.finished).toBe(false);
    expect(r.events.some((e) => e.type === 'GameWon')).toBe(false);
  });

  it('taking the last rival capital ends the game with the new reason, and the view shows it fallen', () => {
    const s = fixture(); addCapital(s, 0, fromOffset(4, 5)); const rival = addCapital(s, 1, fromOffset(14, 5));
    s.cities[rival.id].owner = 0;
    refreshVision(s);
    const core = makeCore(); core.load(coreFixture(s));
    expect(core.view(0).players.map((p) => p.capital_stands)).toEqual([true, false]);
    const r = core.endTurn({});
    expect(r.finished).toBe(true);
    expect(r.winner).toBe(0);
    expect(r.events.find((e) => e.type === 'GameWon')).toMatchObject({ team: 0, reason: expect.stringContaining('holds the last capital') });
  });
});
