// One-time empire actions pinned against literal snapshots of the Rust engine.
import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeAll, describe, expect, it } from 'vitest';
import { RULES, TECHS, TECH_ERA, HORSE_MOUNTED, AZTEC_NO_HORSES } from '../data';
import type { Faction, GameAction, Order, TechId } from '../types';
import { fromOffset, within } from '../hex';
import { WasmClient } from '../wasmClient';
import { coreFixture } from './coreFixture';
import { BARBARIAN, idxOf, refreshVision } from './fixtureEngine';
import { createState, makeStandardSetup, spawnUnit } from './fixtureEngine';
import { thresholdsFor } from '../promotion';
import type { MCity, MState } from './fixtureState';

const available = existsSync(fileURLToPath(new URL('../wasm-pkg/legatum.js', import.meta.url)));
let makeCore: () => WasmClient;

function fixture(factions: Faction[]) {
  const setup = makeStandardSetup('90210', factions[0], factions[1], 'A', 'B');
  setup.width = setup.height = 24;
  setup.players = factions.map((faction, team) => ({ ...(setup.players[team % 2]), faction, name: `P${team}`, team, is_human: true }));
  const s = createState(setup);
  s.units = []; s.nextUnitId = 0; s.cities = []; s.nextCityId = 0;
  s.terrain.fill('Grassland'); s.ruin.fill(0); s.owner.fill(-1); s.tileCity.fill(-1);
  // This action fixture deliberately replaces the generated world with plain land.
  s.wonder?.fill(null); s.wondersDiscovered = [];
  for (const relation of s.diplomacy?.relations ?? []) relation.state = 'Peace';
  return s;
}

/** A city whose territory matches its population: one grown tile per population above 1. */
function city(s: MState, owner: number, col: number, row: number, population = 2) {
  const c: MCity = {
    id: ++s.nextCityId, owner, founder: owner, name: `C${s.nextCityId}`, pos: fromOffset(col, row),
    population, food: 3, prod: 0, producing: null, bought: null, buildings: [],
    hp: 200, grownTiles: [], tilesClaimed: 0, isCapital: s.cities.every((x) => x.owner !== owner), foundedTurn: 1, resistance: 0,
  };
  s.cities.push(c);
  for (const h of within(c.pos, 1)) { const i = idxOf(s, h); s.owner[i] = owner; s.tileCity[i] = c.id; }
  for (const h of within(c.pos, 6)) {
    if (c.grownTiles.length >= population - 1) break;
    const i = idxOf(s, h);
    if (i < 0 || s.owner[i] >= 0) continue;
    s.owner[i] = owner; s.tileCity[i] = c.id; c.grownTiles.push(h); c.tilesClaimed++;
  }
  return c;
}

function pair(s: MState) {
  refreshVision(s);
  const core = makeCore(); core.load(coreFixture(s));
  return core;
}

const action = (a: GameAction): Order => ({ type: 'Action', action: a });
const goalEvents = (events: { type: string }[]) => events.filter((e) => ['FactionGoalCompleted', 'FactionGoalLost', 'SwarmUnleashed', 'SignalBroadcast', 'FreeTechsChosen', 'TileReleased', 'TechResearched', 'OrderFailed', 'VeteranRetired', 'WildCalled', 'BarbarianCampCleared'].includes(e.type));

const SWARM = RULES.swarmPopulation;
const RETIRE_LEVEL = RULES.spartaRetireLevel;

describe.skipIf(!available)('one-time actions core behavior', () => {
  beforeAll(async () => {
    const mod = await import(/* @vite-ignore */ '../wasm-pkg/legatum.js');
    await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
    makeCore = () => new WasmClient(mod.Game);
  });

  function aztecs() {
    const s = fixture(['Aztecs', 'Mongols']);
    city(s, 0, 4, 4, 22); city(s, 0, 12, 4, 22); city(s, 0, 4, 12, 9); city(s, 0, 12, 12, 1);
    city(s, 1, 20, 20, 3);
    return s;
  }

  it('previews the New Fire city by city, tiles included', () => {
    const core = pair(aztecs());
    const view = core.view(0).actions!.swarm!;
    expect(view.blocker).toBeNull();
    expect(view.cities.reduce((n, c) => n + c.lost, 0)).toBe(SWARM);
    expect(view.cities.every((c) => c.tiles.length > 0)).toBe(true);
    expect(core.view(0).actions).toMatchSnapshot();
    expect(core.view(0).faction_goal).toMatchSnapshot();
    expect(core.view(1).actions).toEqual({ swarm: null, sakoku: null, broadcast: null, free_techs: null, retire: null, wild: null });
  });

  it('explains why the New Fire is not available, identically', () => {
    const s = fixture(['Aztecs', 'Mongols']);
    // Two equal cities that together can spare one population short of the offering.
    const size = Math.floor(SWARM / 2);
    city(s, 0, 4, 4, size); city(s, 0, 12, 4, size); city(s, 1, 20, 20, 3);
    const core = pair(s);
    const blocker = core.view(0).actions!.swarm!.blocker;
    expect(blocker).toContain(`can spare ${2 * (size - 1)}`);
    const order = [action({ action: 'Swarm' })];
    expect(core.validate(0, order)).toMatchSnapshot();
    expect(core.validate(0, order)).toHaveLength(1);
    expect(core.validate(1, order)[0].message).toContain('goal is the New Fire');
    expect(core.validate(1, order)).toMatchSnapshot();
    expect(core.validate(0, [...order, ...order])).toMatchSnapshot();
  });

  it('resolves the New Fire the same way: population, territory, goal and report', () => {
    const core = pair(aztecs());
    const orders: Record<number, Order[]> = { 0: [action({ action: 'Swarm' })] };
    expect(core.validate(0, orders[0])).toEqual([]);
    const b = core.endTurn(orders);
    expect(goalEvents(b.events)).toMatchSnapshot();
    const swarm = b.events.find((e) => e.type === 'SwarmUnleashed');
    expect(swarm).toBeTruthy();
    expect(b.events.filter((e) => e.type === 'TileReleased').length).toBeGreaterThan(0);
    const cv = core.view(0);
    expect(cv.cities.map((c) => [c.id, c.population])).toMatchSnapshot();
    expect(cv.players[0].vp_sources).toMatchSnapshot();
    expect(cv.players[0].vp_sources.some(([src]) => src.kind === 'FactionGoal')).toBe(true);
    expect(cv.faction_goal).toMatchSnapshot();
    expect(cv.actions!.swarm ?? null).toBeNull();
    expect(cv.tiles.map((t) => t.owner)).toMatchSnapshot();
    // Permanent: it stays after the cities regrow.
    for (let i = 0; i < 3; i++) core.endTurn({});
    expect(core.view(0).players[0].faction_goal_done).toBe(true);
  });

  it('lists all five horse cards as unavailable and rejects production, queue and purchase', () => {
    const s = aztecs(); s.players[0].techs = [...TECHS];
    const core = pair(s), home = core.view(0).cities.find((c) => c.owner === 0)!;
    for (const unit of HORSE_MOUNTED) {
      const item = { kind: 'Unit' as const, unit };
      expect(home.detail!.unavailable.find((o) => o.item.kind === 'Unit' && o.item.unit === unit)?.reason).toBe(AZTEC_NO_HORSES);
      for (const order of [
        { type: 'SetProduction', city: home.id, item },
        { type: 'SetProductionQueue', city: home.id, queue: [item] },
        { type: 'BuyItem', city: home.id, item },
      ] as Order[]) expect(core.validate(0, [order])[0].message).toContain(AZTEC_NO_HORSES);
    }
    expect(core.view(0).players[0].faction).toBe('Aztecs');
    const restored = makeCore(); restored.load(core.save());
    expect(restored.view(0).actions?.swarm?.population).toBe(SWARM);
  });

  function signal(rival: Faction[] = ['Celts']) {
    const s = fixture(['Athens', ...rival] as Faction[]);
    city(s, 0, 4, 4, 3);
    rival.forEach((_, i) => city(s, i + 1, 18, 4 + 6 * i, 3));
    return s;
  }

  it('broadcasts to every rival outside the team and persists the prompt', () => {
    const s = signal(['Celts', 'Mongols']);
    s.players[2].team = 1; // the two rivals share a team: both still receive
    s.setup.players[2].team = 1;
    const core = pair(s);
    const view = core.view(0).actions!.broadcast!;
    expect(view.blocker).toBeNull();
    expect(view.receivers).toEqual([1, 2]);
    expect(core.view(0).actions).toMatchSnapshot();
    const orders: Record<number, Order[]> = { 0: [action({ action: 'Broadcast' })] };
    const b = core.endTurn(orders);
    expect(goalEvents(b.events)).toMatchSnapshot();
    expect(b.events.find((e) => e.type === 'SignalBroadcast')).toMatchObject({ from: 0, receivers: [1, 2], techs_each: 3 });
    for (const me of [1, 2]) {
      const prompt = core.view(me).actions!.free_techs!;
      expect(prompt).toMatchObject({ pending: 3, from: 0 });
      expect(core.view(me).actions).toMatchSnapshot();
    }
    expect(core.view(0).actions).toEqual({ swarm: null, sakoku: null, broadcast: null, free_techs: null, retire: null, wild: null });
    // Nothing is chosen for a human, however many turns pass.
    for (let i = 0; i < 3; i++) core.endTurn({});
    expect(core.view(1).actions!.free_techs!.pending).toBe(3);
    expect(core.view(1).actions).toMatchSnapshot();
  });

  it('validates and spends the free technology choice the same way', () => {
    const core = pair(signal());
    core.endTurn({ 0: [action({ action: 'Broadcast' })] });
    const open = core.view(1).actions!.free_techs!.choices;
    expect(open.length).toBeGreaterThan(1);
    const first = open[0];
    // Free picks come only from the receiver's current era: a later-era tech is refused.
    const follower = TECHS.find((t) => TECH_ERA[t] > core.view(1).research.era);
    expect(follower).toBeTruthy();
    const choose = (techs: TechId[]) => [action({ action: 'ChooseFreeTechs', techs })];
    for (const techs of [[], [follower!], [first, first], [first, follower!, open[1], open[open.length - 1]], core.view(1).research.known.slice(0, 1)]) {
      const expected = core.validate(1, choose(techs as TechId[]));
      expect(expected).toMatchSnapshot();
    }
    const second = open[1];
    expect(core.validate(1, choose([first, second]))).toEqual([]);
    expect(core.validate(0, choose([first]))[0].message).toContain('no free technologies');
    const orders: Record<number, Order[]> = { 1: choose([first, second]) };
    const b = core.endTurn(orders);
    expect(goalEvents(b.events)).toMatchSnapshot();
    expect(b.events.find((e) => e.type === 'FreeTechsChosen')).toMatchObject({ player: 1, from: 0, techs: [first, second] });
    const cv = core.view(1);
    expect(cv.research.known).toEqual(expect.arrayContaining([first, second]));
    expect(cv.research.known).toMatchSnapshot();
    expect(cv.actions).toMatchSnapshot();
    expect(cv.actions!.free_techs!.pending).toBe(1);
    // Spending the last one clears the prompt.
    const next = core.view(1).actions!.free_techs!.choices[0];
    core.endTurn({ 1: choose([next]) });
    expect(core.view(1).actions!.free_techs ?? null).toBeNull();
    expect(core.view(1).actions).toMatchSnapshot();
  });

  /** An Sparta with a War Shrine city, a plain city, and soldiers of the given promotion levels standing in the shrine city (a level-5 one in the plain city). */
  function sparta(levels: number[]) {
    const s = fixture(['Sparta', 'Mongols']);
    const shrine = city(s, 0, 4, 4, 3), plain = city(s, 0, 12, 4, 3);
    shrine.buildings.push('WarShrine');
    city(s, 1, 20, 20, 3);
    const xp = (level: number) => (level ? thresholdsFor('Sparta')[level - 1] : 0);
    const ids = levels.map((level) => { const u = spawnUnit(s, 0, 'Soldier', shrine.pos, false); u.xp = xp(level); return u.id; });
    const away = spawnUnit(s, 0, 'Soldier', plain.pos, false); away.xp = xp(5);
    const scout = spawnUnit(s, 0, 'Scout', shrine.pos, false);
    return { s, shrine, plain, ids, away: away.id, scout: scout.id };
  }
  const retire = (unit: number) => action({ action: 'RetireVeteran', unit });

  it('previews Sparta retirement identically: shrines, veterans and why not', () => {
    const { s, shrine, ids, away } = sparta([RETIRE_LEVEL, RETIRE_LEVEL + 2, RETIRE_LEVEL - 1]);
    const core = pair(s);
    const view = core.view(0).actions!.retire!;
    expect(view).toMatchObject({ retired: 0, target: 3, level: RETIRE_LEVEL, shrines: [shrine.id] });
    expect(view.veterans.map((v) => [v.unit, v.blocker === null])).toEqual([[ids[0], true], [ids[1], true], [away, false]]);
    expect(core.view(0).actions).toMatchSnapshot();
    expect(core.view(0).faction_goal).toMatchSnapshot();
    expect(core.view(0).faction_goal).toMatchObject({ value: 0, target: 3, label: '0 of 3 veterans retired', live: false });
    expect(core.view(1).actions!.retire ?? null).toBeNull();
  });

  it('validates retirement the same way', () => {
    const { s, ids, away, scout } = sparta([RETIRE_LEVEL, RETIRE_LEVEL + 1, RETIRE_LEVEL + 2, RETIRE_LEVEL, RETIRE_LEVEL - 1]);
    const core = pair(s);
    const cases: Order[][] = [
      [retire(ids[0])], [retire(ids[4])], [retire(away)], [retire(scout)], [retire(9999)],
      [retire(ids[0]), retire(ids[0])], [retire(ids[0]), retire(ids[1]), retire(ids[2])], [retire(ids[0]), retire(ids[1]), retire(ids[2]), retire(ids[3])],
    ];
    for (const orders of cases) expect(core.validate(0, orders)).toMatchSnapshot();
    expect(core.validate(0, cases[1])[0].message).toContain(`promotion level ${RETIRE_LEVEL}`);
    expect(core.validate(0, cases[2])[0].message).toContain('no Agoge');
    expect(core.validate(0, cases[7])).toHaveLength(1);
    expect(core.validate(1, [retire(ids[0])])[0].message).toContain('Only Sparta');
  });

  it('retires three veterans the same way: units, events, progress and the permanent goal', () => {
    const { s, ids } = sparta([4, 5, 6, 4]);
    const core = pair(s);
    const first: Record<number, Order[]> = { 0: [retire(ids[0]), retire(ids[1])] };
    const b1 = core.endTurn(first);
    expect(b1.events.filter((e) => e.type === 'VeteranRetired')).toHaveLength(2);
    let cv = core.view(0);
    expect(cv.units.map((u) => u.id)).toMatchSnapshot();
    expect(cv.units.some((u) => u.id === ids[0] || u.id === ids[1])).toBe(false);
    expect(cv.actions).toMatchSnapshot();
    expect(cv.faction_goal).toMatchObject({ value: 2, target: 3, done: false });
    expect(cv.faction_goal).toMatchSnapshot();
    const second: Record<number, Order[]> = { 0: [retire(ids[2])] };
    const b2 = core.endTurn(second);
    expect(b2.events.find((e) => e.type === 'FactionGoalCompleted')).toMatchObject({ player: 0, faction: 'Sparta' });
    cv = core.view(0);
    expect(cv.players[0].vp_sources.some(([src]) => src.kind === 'FactionGoal')).toBe(true);
    expect(cv.players[0].vp_sources).toMatchSnapshot();
    expect(cv.faction_goal ?? null).toBeNull();
    expect(cv.actions!.retire ?? null).toBeNull();
    expect(cv.actions).toMatchSnapshot();
    expect(core.validate(0, [retire(ids[3])])[0].message).toContain('already complete');
    expect(core.validate(0, [retire(ids[3])])).toMatchSnapshot();
  });

  /** A Celts with two cities that own territory to radius 3, and a rival. `barbarians` off by default. */
  function celts(barbarians: boolean) {
    const s = fixture(['Celts', 'Mongols']);
    const cities = [city(s, 0, 5, 5, 3), city(s, 0, 15, 5, 3)];
    city(s, 1, 20, 20, 3);
    for (const c of cities) for (const h of within(c.pos, 3)) { const i = idxOf(s, h); if (s.owner[i] < 0 || s.tileCity[i] === c.id) { s.owner[i] = 0; s.tileCity[i] = c.id; } }
    s.setup.barbarians = barbarians;
    if (barbarians) s.barbarians = { camps: [], next_camp_id: 0, last_camp_turn: 0, known: {} }; else delete s.barbarians;
    return { s, cities };
  }
  const call = () => action({ action: 'CallTheWild' });

  for (const barbarians of [false, true]) {
    it(`calls the warbands the same way with barbarians ${barbarians ? 'on' : 'off'}: camps, guards, events and the live goal`, () => {
      const { s, cities } = celts(barbarians);
      const core = pair(s);
      const w = core.view(0).actions!.wild!;
      expect(w).toMatchObject({ blocker: null, with_camp: 0, cities: 2 });
      expect(w.sites.map((x) => x.city)).toEqual(cities.map((c) => c.id));
      expect(core.view(0).actions).toMatchSnapshot();
      expect(core.view(1).actions!.wild ?? null).toBeNull();
      expect(core.validate(0, [call()])).toMatchSnapshot();
      expect(core.validate(0, [call(), call()])).toMatchSnapshot();
      expect(core.validate(1, [call()])).toMatchSnapshot();
      const orders: Record<number, Order[]> = { 0: [call()] };
      const b = core.endTurn(orders);
      expect(goalEvents(b.events)).toMatchSnapshot();
      expect(b.events.find((e) => e.type === 'WildCalled')).toMatchObject({ player: 0, camps: 2 });
      expect(b.events.find((e) => e.type === 'FactionGoalCompleted')).toMatchObject({ player: 0, faction: 'Celts' });
      const cv = core.view(0);
      expect(cv.barbarian_camps).toMatchSnapshot();
      expect(cv.barbarian_camps).toHaveLength(2);
      expect(cv.units.map((u) => [u.id, u.owner, u.kind, u.pos])).toMatchSnapshot();
      expect(cv.faction_goal).toMatchObject({ value: 2, target: 2, live: true, done: true });
      expect(cv.faction_goal).toMatchSnapshot();
      expect(cv.players[0].vp_sources).toMatchSnapshot();
      expect(cv.actions).toMatchSnapshot();
      // Natural camps never join them with barbarians off; borders never disperse them.
      for (let i = 0; i < 4; i++) core.endTurn({});
      expect(core.view(0).barbarian_camps).toMatchSnapshot();
      expect(core.view(0).barbarian_camps?.length).toBeGreaterThanOrEqual(2);
      expect(core.view(0).faction_goal).toMatchSnapshot();
    });
  }

  it('resolves rite camps and natural barbarians with the same unit rules', () => {
    const { s, cities } = celts(true);
    const campAt = fromOffset(5, 8);
    const guard = spawnUnit(s, BARBARIAN, 'Soldier', campAt, false);
    const raider = spawnUnit(s, BARBARIAN, 'Soldier', fromOffset(6, 7), false);
    s.barbarians!.camps.push({ id: 0, pos: campAt, founded_turn: 1, guard: guard.id, scout: null, raiders: [raider.id], scout_report: null, raid: null, next_raid_turn: 0, next_spawn_turn: 99, rite: 0 });
    s.barbarians!.next_camp_id = 1;
    const natural = spawnUnit(s, BARBARIAN, 'Soldier', fromOffset(3, 5), false);
    const rival = spawnUnit(s, 1, 'Soldier', fromOffset(4, 6), false);
    expect(s.tileCity[idxOf(s, campAt)]).toBe(cities[0].id);
    const core = pair(s);
    const b = core.endTurn({});
    expect(b.events).toMatchSnapshot();
    const hp = (units: { id: number; hp: number }[], id: number) => units.find((u) => u.id === id)?.hp;
    for (const id of [guard.id, raider.id, natural.id, rival.id]) expect(hp(core.view(0).units, id)).toMatchSnapshot();
  });

  it('pays nothing for clearing a camp the rite raised, and the goal follows a cleared camp, the same way', () => {
    const { s, cities } = celts(true);
    // Two rite camps (one per city), standing in the territory they were raised for.
    const spots = [fromOffset(5, 8), fromOffset(15, 8)];
    for (const [i, at] of spots.entries()) {
      s.barbarians!.camps.push({ id: i, pos: at, founded_turn: 1, guard: null, scout: null, raiders: [], scout_report: null, raid: null, next_raid_turn: 0, next_spawn_turn: 99, rite: 0 });
      expect(s.tileCity[idxOf(s, at)]).toBe(cities[i].id);
    }
    s.barbarians!.next_camp_id = 2;
    spawnUnit(s, 0, 'Soldier', spots[0], false);
    spawnUnit(s, 1, 'Soldier', spots[1], false);
    const core = pair(s);
    expect(core.view(0).faction_goal).toMatchObject({ value: 2, target: 2 });
    const goldBefore = core.view(0).economy.gold, rivalBefore = core.view(1).economy.gold;
    const b = core.endTurn({});
    expect(goalEvents(b.events)).toMatchSnapshot();
    const cleared = b.events.filter((e) => e.type === 'BarbarianCampCleared') as { by: number; gold: number }[];
    expect(cleared.map((e) => [e.by, e.gold > 0])).toEqual([[0, false], [1, true]]);
    expect(core.view(0).economy.gold).toMatchSnapshot();
    expect(core.view(1).economy.gold).toMatchSnapshot();
    expect(core.view(1).economy.gold).toBeGreaterThan(rivalBefore);
    expect(core.view(0).economy.gold - goldBefore).toBeLessThan(cleared[1].gold);
    expect(core.view(0).barbarian_camps ?? []).toMatchSnapshot();
    expect(core.view(0).faction_goal).toMatchObject({ value: 0, done: false });
    expect(core.view(0).faction_goal).toMatchSnapshot();
    expect(core.view(0).actions).toMatchSnapshot();
  });
});
