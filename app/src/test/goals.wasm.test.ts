import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeAll, describe, expect, it } from 'vitest';
import { RULES } from '../data';
import type { Faction } from '../types';
import { fromOffset, within } from '../hex';
import { WasmClient } from '../wasmClient';
import { coreFixture } from './coreFixture';
import { FixtureClient } from './fixtureEngine';
import { newBarbarians } from './fixtureEngine';
import { BARBARIAN, idxOf, refreshVision } from './fixtureEngine';
import { createState, makeStandardSetup, spawnUnit } from './fixtureEngine';
import type { MCity, MState } from './fixtureState';

const available = existsSync(fileURLToPath(new URL('../wasm-pkg/legatum.js', import.meta.url)));
let makeCore: () => WasmClient;

function fixture(first: Faction, second: Faction) {
  const setup = makeStandardSetup('90210', first, second, 'A', 'B');
  setup.width = setup.height = 20;
  setup.players = setup.players.slice(0, 2).map((p, team) => ({ ...p, team, is_human: true }));
  const s = createState(setup);
  s.units = []; s.nextUnitId = 0; s.cities = []; s.nextCityId = 0;
  s.terrain.fill('Grassland'); s.ruin.fill(0); s.owner.fill(-1); s.tileCity.fill(-1);
  for (const relation of s.diplomacy?.relations ?? []) relation.state = 'Peace';
  return s;
}

function city(s: MState, owner: number, col: number, row: number, radius = 1) {
  const c: MCity = {
    id: ++s.nextCityId, owner, founder: owner, name: `C${owner}`, pos: fromOffset(col, row),
    population: 2, food: 0, prod: 0, producing: null, bought: null, buildings: [],
    hp: 200, grownTiles: [], tilesClaimed: 0, isCapital: true, foundedTurn: 1, resistance: 0,
  };
  s.cities.push(c);
  for (const h of within(c.pos, radius)) { const i = idxOf(s, h); s.owner[i] = owner; s.tileCity[i] = c.id; }
  return c;
}

function pair(s: MState) {
  refreshVision(s);
  const fixtureClient = new FixtureClient(); fixtureClient.load(JSON.stringify(s));
  const core = makeCore(); core.load(coreFixture(s));
  return { fixtureClient, core };
}

const goalEvents = (events: { type: string }[]) => events.filter((e) => e.type.startsWith('FactionGoal'));

describe.skipIf(!available)('faction goal WASM behavior', () => {
  beforeAll(async () => {
    const mod = await import(/* @vite-ignore */ '../wasm-pkg/legatum.js');
    await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
    makeCore = () => new WasmClient(mod.Game);
  });

  it('Switzerland hoard is live: gained, lost below the hoard, regained with `again`', () => {
    const phases: { gold: number; done: boolean; before: boolean; expected: object | null }[] = [];
    const hoard = RULES.factionGoals.Switzerland.target;
    expect(RULES.factionGoals.Switzerland.kind).toBe('Live');
    phases.push(
      { gold: hoard + 100, done: false, before: false, expected: { type: 'FactionGoalCompleted', player: 0, faction: 'Switzerland', vp: 2 } },
      { gold: hoard + 100, done: true, before: true, expected: null },
      { gold: hoard - 100, done: true, before: true, expected: { type: 'FactionGoalLost', player: 0, faction: 'Switzerland', vp: 2 } },
      { gold: hoard + 100, done: false, before: true, expected: { type: 'FactionGoalCompleted', player: 0, faction: 'Switzerland', vp: 2, again: true } },
      { gold: hoard - 100, done: false, before: true, expected: null },
    );
    for (const phase of phases) {
      const s = fixture('Switzerland', 'Mongols');
      city(s, 0, 4, 4); city(s, 1, 16, 16);
      s.players[0].gold = phase.gold; s.players[0].factionGoalDone = phase.done; s.players[0].factionGoalReachedBefore = phase.before;
      const { fixtureClient, core } = pair(s);
      (expect(fixtureClient.view(0).faction_goal).toMatchSnapshot(), void (core.view(0).faction_goal));
      expect(core.view(0).faction_goal).toMatchObject({ target: hoard, live: true, done: phase.done });
      const a = fixtureClient.endTurn({}), b = core.endTurn({});
      (expect(goalEvents(a.events)).toMatchSnapshot(), void (goalEvents(b.events)));
      (expect(goalEvents(b.events)).toMatchSnapshot(), void (phase.expected ? [phase.expected] : []));
      (expect(fixtureClient.view(0).players[0].vp_sources).toMatchSnapshot(), void (core.view(0).players[0].vp_sources));
      (expect(fixtureClient.view(0).players[0].faction_goal_done).toMatchSnapshot(), void (core.view(0).players[0].faction_goal_done));
    }
  });

  it('a Permanent goal is never lost and vp_sources are typed', () => {
    const s = fixture('India', 'Mongols');
    city(s, 0, 4, 4); city(s, 1, 16, 16);
    s.players[0].goalRule = 'Swarm'; s.players[0].factionGoalDone = true; s.players[0].factionGoalReachedBefore = true;
    const { fixtureClient, core } = pair(s);
    const a = fixtureClient.endTurn({}), b = core.endTurn({});
    expect(goalEvents(a.events)).toEqual([]);
    expect(goalEvents(b.events)).toEqual([]);
    expect(core.view(0).players[0].vp_sources).toContainEqual([{ kind: 'FactionGoal', of: 0 }, 2]);
    (expect(fixtureClient.view(0).players[0].vp_sources).toMatchSnapshot(), void (core.view(0).players[0].vp_sources));
    (expect(fixtureClient.view(0).faction_goal).toMatchSnapshot(), void (core.view(0).faction_goal));
  });

  it('every faction goal view agrees', () => {
    for (const f of ['India', 'Germany', 'Celts', 'Mongols', 'Sparta', 'Athens', 'Byzantium', 'Switzerland'] as Faction[]) {
      const s = fixture(f, f === 'Mongols' ? 'India' : 'Mongols');
      city(s, 0, 4, 4); city(s, 1, 16, 16);
      const { fixtureClient, core } = pair(s);
      (expect(fixtureClient.view(0).faction_goal).toMatchSnapshot(), void (core.view(0).faction_goal));
    }
  });

  it('a barbarian unit or camp on a worked tile removes its yields, in fixtureClient and core alike', () => {
    for (const kind of ['unit', 'camp'] as const) {
      const s = fixture('Switzerland', 'Mongols');
      const c = city(s, 0, 8, 8, 3);
      city(s, 1, 17, 17);
      const tile = fromOffset(9, 8);
      const before = pair(s);
      const baseline = before.core.view(0).cities.find((x) => x.id === c.id)!.detail!.yields;
      (expect(before.fixtureClient.view(0).cities.find((x) => x.id === c.id)!.detail!.yields).toMatchSnapshot(), void (baseline));
      if (kind === 'unit') spawnUnit(s, BARBARIAN, 'Soldier', tile, false);
      else {
        s.barbarians = newBarbarians();
        s.barbarians.camps.push({ id: 1, pos: tile, founded_turn: 1, guard: null, scout: null, raiders: [], scout_report: null, raid: null, next_raid_turn: 0, next_spawn_turn: 99 });
        s.barbarians.next_camp_id = 2;
      }
      const { fixtureClient, core } = pair(s);
      const coreYields = core.view(0).cities.find((x) => x.id === c.id)!.detail!.yields;
      (expect(fixtureClient.view(0).cities.find((x) => x.id === c.id)!.detail!.yields, kind).toMatchSnapshot(), void (coreYields));
      expect(coreYields.total).not.toEqual(baseline.total);
      expect(coreYields.sources).toEqual(expect.arrayContaining([expect.objectContaining({ label: expect.stringContaining('Barbarians hold the tile') })]));
    }
  });
});
