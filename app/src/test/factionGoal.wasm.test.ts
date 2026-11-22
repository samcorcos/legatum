// Your own faction goal's progress for every faction, from an empty empire to one part-way through its goal.
import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeAll, describe, expect, it } from 'vitest';
import type { BuildingKind, Faction } from '../types';
import { fromOffset, within } from '../hex';
import { WasmClient } from '../wasmClient';
import { coreFixture } from './coreFixture';
import { FixtureClient } from './fixtureEngine';
import { idxOf, refreshVision } from './fixtureEngine';
import { createState, makeStandardSetup, spawnUnit } from './fixtureEngine';
import type { MCity, MState } from './fixtureState';

const available = existsSync(fileURLToPath(new URL('../wasm-pkg/legatum.js', import.meta.url)));
let makeCore: () => WasmClient;

function fixture(first: Faction) {
  const setup = makeStandardSetup('73531', first, first === 'Celts' ? 'Mongols' : 'Celts', 'A', 'B');
  setup.width = setup.height = 40;
  setup.players = setup.players.slice(0, 2).map((p, team) => ({ ...p, team, is_human: true }));
  const s = createState(setup);
  s.units = []; s.nextUnitId = 0; s.cities = []; s.nextCityId = 0;
  s.terrain.fill('Grassland'); s.ruin.fill(0); s.owner.fill(-1); s.tileCity.fill(-1);
  return s;
}

function city(s: MState, owner: number, col: number, row: number, population: number, buildings: BuildingKind[], capital = false) {
  const c: MCity = {
    id: ++s.nextCityId, owner, founder: owner, name: `C${s.nextCityId}`, pos: fromOffset(col, row),
    population, food: 0, prod: 0, producing: null, bought: null, buildings,
    hp: 200, grownTiles: [], tilesClaimed: 0, isCapital: capital, foundedTurn: 1, resistance: 0,
  };
  s.cities.push(c);
  for (const h of within(c.pos, 1)) { const i = idxOf(s, h); s.owner[i] = owner; s.tileCity[i] = c.id; }
  return c;
}

function pair(s: MState) {
  refreshVision(s);
  const fixtureClient = new FixtureClient(); fixtureClient.load(JSON.stringify(s));
  const core = makeCore(); core.load(coreFixture(s));
  return { fixtureClient, core };
}

const FACTIONS: Faction[] = ['Switzerland', 'India', 'Byzantium', 'Germany', 'Celts', 'Mongols', 'Sparta', 'Athens'];

describe.skipIf(!available)('faction goal progress: core behavior', () => {
  beforeAll(async () => {
    const mod = await import(/* @vite-ignore */ '../wasm-pkg/legatum.js');
    await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
    makeCore = () => new WasmClient(mod.Game);
  });

  for (const faction of FACTIONS) {
    it(`${faction}: an empty empire and a part-way one report the same progress`, () => {
      const s = fixture(faction);
      city(s, 1, 36, 36, 2, [], true);
      let { fixtureClient, core } = pair(s);
      const empty = core.view(0).faction_goal!;
      expect(empty.value).toBeLessThan(empty.target); // (Switzerland start with Gold)
      expect(fixtureClient.view(0).faction_goal).toMatchSnapshot();
      // Part-way: cities with the buildings the goal asks about, spread out and walled where it matters.
      const all: BuildingKind[] = ['Granary', 'Workshop', 'Market', 'Library', 'Walls'];
      const own = fixture(faction);
      city(own, 1, 36, 36, 2, [], true);
      city(own, 0, 4, 4, 8, all, true);
      city(own, 0, 14, 4, 5, ['Granary', 'Market', 'Library', 'Walls']);
      city(own, 0, 4, 14, 5, ['Granary', 'Market']);
      if (faction === 'Sparta') spawnUnit(own, 0, 'Spearman', fromOffset(4, 4), false);
      if (faction === 'Athens') own.players[0].techs = ['Philosophy'];
      own.players[0].gold = 123;
      ({ fixtureClient, core } = pair(own));
      const part = core.view(0).faction_goal!;
      expect(part.target).toBeGreaterThan(0);
      expect(part.label.length).toBeGreaterThan(5);
      expect(fixtureClient.view(0).faction_goal).toMatchSnapshot();
      // A rival's progress is never in your view: only your own goal is.
      expect(core.view(1).faction_goal?.label).not.toBe(part.label);
    });
  }

  it('is null once the goal is done, and typed sources name what scored', () => {
    const s = fixture('India');
    city(s, 1, 36, 36, 2, [], true);
    s.players[0].goalRule = 'Swarm';
    s.players[0].factionGoalDone = true;
    s.players[0].greatWorkCompleted = 'GreatLibrary';
    const workTile = idxOf(s, fromOffset(36, 36));
    s.owner[workTile] = 0;
    s.landmark = Array(s.terrain.length).fill(null);
    s.landmark[workTile] = { building: 'GreatLibrary', city: 1, of: 0 };
    const { fixtureClient, core } = pair(s);
    expect(core.view(0).faction_goal ?? null).toBeNull();
    expect(fixtureClient.view(0).faction_goal ?? null).toBeNull();
    const sources = core.view(0).players[0].vp_sources;
    expect(sources).toContainEqual([{ kind: 'GreatWork', of: 0 }, 2]);
    expect(sources).toContainEqual([{ kind: 'FactionGoal', of: 0 }, expect.any(Number)]);
    expect(fixtureClient.view(0).players[0].vp_sources).toMatchSnapshot();
  });
});
