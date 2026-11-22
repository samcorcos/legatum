import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeAll, describe, expect, it } from 'vitest';
import type { Faction, Order } from '../types';
import { distance, fromOffset, within } from '../hex';
import { WasmClient } from '../wasmClient';
import { coreFixture } from './coreFixture';
import { FixtureClient } from './fixtureEngine';
import { idxOf, refreshVision } from './fixtureEngine';
import { createState, makeStandardSetup, spawnUnit } from './fixtureEngine';
import type { MCity, MState } from './fixtureState';

const available = existsSync(fileURLToPath(new URL('../wasm-pkg/legatum.js', import.meta.url)));
let makeCore: () => WasmClient;

function fixture(first: Faction, second: Faction) {
  const setup = makeStandardSetup('73531', first, second, 'A', 'B');
  setup.width = setup.height = 20;
  setup.players = setup.players.slice(0, 2).map((p, team) => ({ ...p, team, is_human: true }));
  const s = createState(setup);
  s.units = []; s.nextUnitId = 0; s.cities = []; s.nextCityId = 0;
  s.terrain.fill('Grassland'); s.ruin.fill(0); s.owner.fill(-1); s.tileCity.fill(-1);
  for (const relation of s.diplomacy?.relations ?? []) relation.state = 'Peace';
  return s;
}

function city(s: MState, owner: number, col: number, row: number) {
  const c: MCity = {
    id: ++s.nextCityId, owner, founder: owner, name: `C${owner}`, pos: fromOffset(col, row),
    population: 2, food: 0, prod: 0, producing: null, bought: null, buildings: [],
    hp: 200, grownTiles: [], tilesClaimed: 0, isCapital: true, foundedTurn: 1, resistance: 0,
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

describe.skipIf(!available)('Byzantium and Celts core behavior', () => {
  beforeAll(async () => {
    const mod = await import(/* @vite-ignore */ '../wasm-pkg/legatum.js');
    await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
    makeCore = () => new WasmClient(mod.Game);
  });

  it('researches normally and keeps the core economy in sync', () => {
    const s = fixture('Byzantium', 'Mongols');
    const home = city(s, 0, 4, 4);
    home.buildings = ['University'];
    home.population = 5;
    const owned = city(s, 1, 10, 10);
    spawnUnit(s, 0, 'Spearman', fromOffset(11, 10), false);
    spawnUnit(s, 0, 'Scout', fromOffset(10, 11), false);
    s.players[0].techs = ['Writing', 'Education'];
    s.players[1].techs = ['BronzeWorking', 'Writing', 'Currency'];
    const { fixtureClient, core } = pair(s);
    const coreView = core.view(0), mockView = fixtureClient.view(0);
    expect(mockView.economy.science_per_turn).toMatchSnapshot();
    (expect(fixtureClient.view(1).cities.find((c) => c.id === owned.id)?.detail?.yields).toMatchSnapshot(), void (core.view(1).cities.find((c) => c.id === owned.id)?.detail?.yields));
    expect(mockView.cities.find((c) => c.id === home.id)?.detail?.yields).toMatchSnapshot();
    const card = (v: typeof coreView, b: string) => v.research.unlocks.find((u) => u.item.kind === 'Building' && u.item.building === b);
    for (const b of ['Library', 'University', 'ResearchLab']) {
      const [m, c] = [card(mockView, b), card(coreView, b)];
      expect(c, `${b} card`).toBeDefined();
      expect([m?.name, m?.yields], b).toMatchSnapshot();
    }
    expect(card(coreView, 'Library')!.name).toBe('Library');
    expect(card(coreView, 'Library')!.yields.science).toBeGreaterThan(0);
    const a = fixtureClient.endTurn({}), b = core.endTurn({});
    (expect(a.events).toMatchSnapshot(), void (b.events));
    (expect(fixtureClient.view(0).research.progress).toMatchSnapshot(), void (core.view(0).research.progress));
    (expect(fixtureClient.view(0).faction_goal).toMatchSnapshot(), void (core.view(0).faction_goal));
  });

  it('keeps a foreign unit at full strength during Peace and shows Celtic archers normally', () => {
    const s = fixture('Celts', 'Mongols');
    city(s, 0, 4, 4); city(s, 1, 16, 16);
    const intruderTile = fromOffset(5, 4);
    const intruder = spawnUnit(s, 1, 'Spearman', intruderTile, false); intruder.hp = 50;
    const { fixtureClient, core } = pair(s);
    const a = fixtureClient.endTurn({}), b = core.endTurn({});
    (expect(a.events).toMatchSnapshot(), void (b.events));
    // A Mongol Soldier does not rest-heal during Peace.
    expect(fixtureClient.view(1).units.find((u) => u.id === intruder.id)?.hp).toBe(50);
    expect(core.view(1).units.find((u) => u.id === intruder.id)?.hp).toBe(50);

    const fight = fixture('Celts', 'Mongols');
    city(fight, 0, 4, 4); city(fight, 1, 16, 16);
    for (const relation of fight.diplomacy?.relations ?? []) relation.state = 'War';
    const ranger = spawnUnit(fight, 0, 'Archer', fromOffset(10, 10), false);
    const target = spawnUnit(fight, 1, 'Spearman', fromOffset(11, 10), false);
    const clients = pair(fight);
    for (const client of [clients.fixtureClient, clients.core])
      expect(client.view(1).units.some((u) => u.id === ranger.id)).toBe(true);
    const shot: Record<number, Order[]> = { 0: [{ type: 'RangedAttack', unit: ranger.id, target: target.pos }] };
    const mockAttack = clients.fixtureClient.endTurn(shot); clients.core.endTurn(shot);
    expect(mockAttack.events.filter((e) => e.type === 'Battle')).toMatchSnapshot();
  });

  it('sacks a Celtic conquest and releases its territory', () => {
    const s = fixture('Celts', 'Mongols');
    city(s, 0, 4, 4);
    const captured = city(s, 1, 10, 10); captured.hp = 0;
    for (const relation of s.diplomacy?.relations ?? []) relation.state = 'War';
    const soldier = spawnUnit(s, 0, 'Spearman', fromOffset(9, 10), false);
    const { fixtureClient, core } = pair(s);
    const order: Order = { type: 'MoveUnit', unit: soldier.id, to: captured.pos };
    (expect(fixtureClient.validate(0, [order])).toMatchSnapshot(), void (core.validate(0, [order])));
    const a = fixtureClient.endTurn({ 0: [order] }), b = core.endTurn({ 0: [order] });
    (expect(a.events.filter((e) => e.type === 'CityRazed')).toMatchSnapshot(), void (b.events.filter((e) => e.type === 'CityRazed')));
    expect(a.events.some((e) => e.type === 'CityRazed')).toBe(true);
    for (const client of [fixtureClient, core]) {
      expect(client.view(0).cities.some((c) => c.id === captured.id)).toBe(false);
      expect(client.view(0).tiles[idxOf(s, captured.pos)].owner).toBeNull();
    }
  });

  describe('a Celtic sack takes Gold and raises a warband camp', () => {
    const SACK = ['CityRazed', 'TechResearched', 'EraEntered', 'BarbarianCampSpawned'];
    /** The Celts (P0) sack P1's city at (10, 10) on the first turn; core must agree on events and views. */
    function sack(s: MState, home = true) {
      if (home) city(s, 0, 4, 4);
      const captured = city(s, 1, 10, 10); captured.hp = 0;
      for (const relation of s.diplomacy?.relations ?? []) relation.state = 'War';
      const soldier = spawnUnit(s, 0, 'Spearman', fromOffset(9, 10), false);
      const { fixtureClient, core } = pair(s);
      const orders: Record<number, Order[]> = { 0: [{ type: 'MoveUnit', unit: soldier.id, to: captured.pos }] };
      const a = fixtureClient.endTurn(orders), b = core.endTurn(orders);
      const only = (events: { type: string }[]) => events.filter((e) => SACK.includes(e.type));
      (expect(only(a.events)).toMatchSnapshot(), void (only(b.events)));
      for (const player of [0, 1]) {
        (expect(only(a.events_by_player?.[player] ?? [])).toMatchSnapshot(), void (only(b.events_by_player?.[player] ?? [])));
        (expect(fixtureClient.view(player).barbarian_camps).toMatchSnapshot(), void (core.view(player).barbarian_camps));
      }
      const razed = b.events.find((e) => e.type === 'CityRazed') as Extract<typeof b.events[number], { type: 'CityRazed' }>;
      return { fixtureClient, core, events: b.events, razed, captured };
    }

    it('changes no technology: the victim keeps its own and the Celts gain none', () => {
      const s = fixture('Celts', 'Mongols');
      s.players[1].techs = ['Irrigation', 'Masonry', 'Currency', 'Mathematics'];
      s.players[0].techs = ['Irrigation'];
      const { events, razed, core } = sack(s);
      expect(razed).not.toHaveProperty('tech');
      expect(events.some((e) => e.type === 'TechResearched')).toBe(false);
      expect(core.view(0).research.known).toEqual(['Irrigation']);
    });

    it('carries off exactly 60 Gold per population, whatever the victim holds', () => {
      const s = fixture('Celts', 'Mongols');
      const { razed, captured } = sack(s);
      expect(razed.sack_gold).toBe(60 * captured.population);
    });

    it('raises a Celtic warband camp within three tiles of the ruins, in the report and on the map', () => {
      const s = fixture('Celts', 'Mongols');
      const { razed, core, events, captured } = sack(s);
      expect(razed.camp).toBeDefined();
      expect(distance(razed.camp!, captured.pos)).toBeLessThanOrEqual(3);
      expect(events.some((e) => e.type === 'BarbarianCampSpawned')).toBe(true);
      expect(core.view(0).barbarian_camps?.some((c) => c.pos.q === razed.camp!.q && c.pos.r === razed.camp!.r)).toBe(true);
    });

    it('prefers the land of a Celtic city that has no camp yet', () => {
      const s = fixture('Celts', 'Mongols');
      const home = city(s, 0, 13, 10);
      const { razed } = sack(s, false);
      expect(razed.camp).toBeDefined();
      expect(s.tileCity[idxOf(s, razed.camp!)]).toBe(home.id);
    });

    it('creates the camp state even with barbarians off', () => {
      const s = fixture('Celts', 'Mongols');
      s.setup.barbarians = false;
      const { razed, core, events } = sack(s);
      expect(razed.camp).toBeDefined();
      expect(events.some((e) => e.type === 'BarbarianCampSpawned')).toBe(true);
      expect(core.view(0).barbarian_camps?.length).toBe(1);
    });
  });
});

describe.skipIf(!available)('Germany industry core behavior', () => {
  beforeAll(async () => {
    const mod = await import(/* @vite-ignore */ '../wasm-pkg/legatum.js');
    await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
    makeCore = () => new WasmClient(mod.Game);
  });

  it('converts city Production into recurring Gold or Science with ordinary unit movement', () => {
    const s = fixture('Germany', 'India');
    const c = city(s, 0, 4, 4);
    city(s, 1, 15, 15);
    spawnUnit(s, 0, 'Settler', fromOffset(5, 4), false);
    spawnUnit(s, 0, 'Scout', fromOffset(4, 5), false);
    const { fixtureClient, core } = pair(s);
    (expect(fixtureClient.view(0).units.map((u) => u.max_moves)).toMatchSnapshot(), void (core.view(0).units.map((u) => u.max_moves)));
    for (const project of ['Gold', 'Science'] as const) {
      const orders: Record<number, Order[]> = { 0: [{ type: 'SetProduction', city: c.id, item: { kind: 'Project', project } }] };
      const a = fixtureClient.endTurn(orders), b = core.endTurn(orders);
      (expect(a.events).toMatchSnapshot(), void (b.events));
      const m = fixtureClient.view(0).cities.find((x) => x.id === c.id)!.detail!;
      const w = core.view(0).cities.find((x) => x.id === c.id)!.detail!;
      (expect(m.yields).toMatchSnapshot(), void (w.yields));
      (expect(m.production_stored).toMatchSnapshot(), void (w.production_stored));
      expect(m.production_cost).toBeNull();
      expect(m.yields.total[project.toLowerCase() as 'gold' | 'science']).toBeGreaterThan(0);
    }
  });
});
