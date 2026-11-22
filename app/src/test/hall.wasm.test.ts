import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeAll, describe, expect, it } from 'vitest';
import { FACTIONS, RULES, UNIT_KINDS } from '../data';
import { canEnter } from '../diplomacy';
import { damage } from '../combat';
import { distance, fromOffset, neighbors, within } from '../hex';
import type { CityView, Faction, Order, PlayerView, ProductionItem, UnitKind } from '../types';
import { WasmClient } from '../wasmClient';
import { coreFixture } from './coreFixture';
import { FixtureClient } from './fixtureEngine';
import { idxOf, productionCost, refreshVision } from './fixtureEngine';
import { createState, makeStandardSetup, spawnUnit } from './fixtureEngine';
import type { MCity, MState } from './fixtureState';

const available = existsSync(fileURLToPath(new URL('../wasm-pkg/legatum.js', import.meta.url)));
let makeCore: () => WasmClient;

const HALL: ProductionItem = { kind: 'Building', building: 'CourierHall' };
const isHall = (item: ProductionItem) => (item.kind === 'Building' || item.kind === 'Wonder') && item.building === 'CourierHall';
const hallItem = (c: Pick<MCity, 'pos'>): ProductionItem => ({ kind: 'Wonder', building: 'CourierHall', tile: neighbors(c.pos)[0] });

function fixture(first: Faction, second: Faction, turn = 1) {
  const setup = makeStandardSetup('90211', first, second, 'A', 'B');
  setup.width = setup.height = 20;
  setup.players = setup.players.slice(0, 2).map((p, team) => ({ ...p, team, is_human: true }));
  const s = createState(setup);
  s.turn = turn;
  s.units = []; s.nextUnitId = 0; s.cities = []; s.nextCityId = 0;
  s.terrain.fill('Grassland'); s.ruin.fill(0); s.owner.fill(-1); s.tileCity.fill(-1);
  for (const relation of s.diplomacy?.relations ?? []) relation.state = 'Peace';
  return s;
}

function city(s: MState, owner: number, col: number, row: number, extra: Partial<MCity> = {}) {
  const c: MCity = {
    id: ++s.nextCityId, owner, founder: owner, name: `C${s.nextCityId}`, pos: fromOffset(col, row),
    population: 2, food: 0, prod: 0, producing: null, bought: null, buildings: [],
    hp: 200, grownTiles: [], tilesClaimed: 0, isCapital: s.cities.every((o) => o.owner !== owner), foundedTurn: 1, resistance: 0,
    ...extra,
  };
  s.cities.push(c);
  for (const h of within(c.pos, 1)) { const i = idxOf(s, h); s.owner[i] = owner; s.tileCity[i] = c.id; }
  return c;
}

function placeHall(s: MState, c: MCity, of: number) {
  const tile = neighbors(c.pos)[0];
  s.landmark ??= Array(s.terrain.length).fill(null);
  s.landmark[idxOf(s, tile)] = { building: 'CourierHall', city: c.id, of };
}

/** A Courier Hall of Mongols empire `of`, standing in a new city of `owner` (the holder). */
function hallCity(s: MState, of: number, owner: number, col: number, row: number, extra: Partial<MCity> = {}) {
  const c = city(s, owner, col, row, { buildings: ['CourierHall'], ...extra });
  placeHall(s, c, of);
  s.players[of].hallCity = c.id;
  return c;
}

function pair(s: MState) {
  refreshVision(s);
  const fixtureClient = new FixtureClient(); fixtureClient.load(JSON.stringify(s));
  const core = makeCore(); core.load(coreFixture(s));
  return { fixtureClient, core };
}

const goalEvents = (events: { type: string }[]) => events.filter((e) => e.type.startsWith('FactionGoal'));
const cityOf = (view: { cities: CityView[] }, id: number) => view.cities.find((c) => c.id === id)!;
/** What a city's production list says about the Courier Hall: whether it is offered, its price and the reason it is held back. */
const listing = (view: { cities: CityView[] }, id: number) => {
  const d = cityOf(view, id).detail!;
  return { options: d.options.filter((o) => isHall(o.item)), unavailable: d.unavailable.filter((o) => isHall(o.item)) };
};
const hallOrders = (city: Pick<MCity, 'id' | 'pos'>): Order[] => [{ type: 'SetProduction', city: city.id, item: hallItem(city) }];
const lost = (player: number) => ({ type: 'FactionGoalLost', player, faction: 'Mongols', vp: 2 });
const completed = (player: number, again = false) => ({ type: 'FactionGoalCompleted', player, faction: 'Mongols', vp: 2, ...(again ? { again: true } : {}) });
const goalSources = (view: PlayerView, p: number) => view.players[p].vp_sources.filter(([src]) => src.kind === 'FactionGoal');

describe('the Yam Relay Station rules and wording the UI reads', () => {
  it('the exported rules describe the Open Road goal', () => {
    expect(RULES.factionGoals.Mongols).toMatchObject({ kind: 'Held', target: 1, label: 'Yam Relay Station held', vp: 2 });
  });
});

describe.skipIf(!available)('Yam Relay Station and the Mongols goal: WASM behavior', () => {
  beforeAll(async () => {
    const mod = await import(/* @vite-ignore */ '../wasm-pkg/legatum.js');
    await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
    makeCore = () => new WasmClient(mod.Game);
  });

  it('greys out Mongols Walls and Citadels with core-matching reasons', () => {
    const s = fixture('Mongols', 'India', 200);
    const home = city(s, 0, 4, 4); city(s, 1, 16, 16);
    s.players[0].techs.push('Masonry', 'Feudalism', 'Chivalry', 'Mathematics');
    s.players[0].gold = 5000;
    const { fixtureClient, core } = pair(s);
    for (const item of [
      { kind: 'Building', building: 'Walls' },
      { kind: 'Building', building: 'Castle' },
    ] as ProductionItem[]) {
      const listingOf = (client: FixtureClient | WasmClient) => {
        const detail = cityOf(client.view(0), home.id).detail!;
        return {
          options: detail.options.filter((o) => JSON.stringify(o.item) === JSON.stringify(item)),
          unavailable: detail.unavailable.filter((o) => JSON.stringify(o.item) === JSON.stringify(item)),
        };
      };
      (expect(listingOf(fixtureClient)).toMatchSnapshot(), void (listingOf(core)));
      expect(listingOf(core).options).toEqual([]);
      expect(listingOf(core).unavailable[0]?.reason).toContain('The Mongols raise Walls from the Medieval era');
      for (const order of [{ type: 'SetProduction', city: home.id, item }, { type: 'BuyItem', city: home.id, item }] as Order[]) {
        (expect(fixtureClient.validate(0, [order])).toMatchSnapshot(), void (core.validate(0, [order])));
        expect(core.validate(0, [order])).not.toEqual([]);
      }
    }
  });

  it('keeps Mongol Siege unavailable before Medieval after capturing a Workshop city', () => {
    const s = fixture('Mongols', 'India');
    city(s, 0, 4, 4);
    const workshopCity = city(s, 1, 10, 10);
    workshopCity.buildings.push('Workshop');
    workshopCity.hp = 0;
    city(s, 1, 16, 16);
    s.players[0].techs.push('Mathematics');
    s.players[0].gold = 5000;
    for (const relation of s.diplomacy!.relations) relation.state = 'War';
    const soldier = spawnUnit(s, 0, 'Soldier', fromOffset(5, 4), false);
    const taker = spawnUnit(s, 0, 'Soldier', fromOffset(9, 10), false);
    const { fixtureClient, core } = pair(s);
    const offer = (client: FixtureClient | WasmClient) => client.view(0).units.find((u) => u.id === soldier.id)!.siege_offer;
    expect(offer(core)?.blocker).toContain('Medieval era');
    (expect(offer(fixtureClient)).toMatchSnapshot(), void (offer(core)));
    const a = fixtureClient.endTurn({ 0: [{ type: 'MoveUnit', unit: taker.id, to: workshopCity.pos }] });
    const b = core.endTurn({ 0: [{ type: 'MoveUnit', unit: taker.id, to: workshopCity.pos }] });
    (expect(a.events).toMatchSnapshot(), void (b.events));
    expect(b.events).toContainEqual(expect.objectContaining({ type: 'CityCaptured', city: workshopCity.id }));
    expect(offer(core)?.blocker).toContain('Medieval era');
    (expect(offer(fixtureClient)).toMatchSnapshot(), void (offer(core)));
  });

  it.each(['Soldier', 'Ranger'] as UnitKind[])('does not halve a Mongols %s hit on an open city (only Walls do)', (kind) => {
    const s = fixture('Mongols', 'India');
    city(s, 0, 4, 4); const target = city(s, 1, 10, 8);
    const attacker = spawnUnit(s, 0, kind, fromOffset(kind === 'Ranger' ? 8 : 9, 8), false);
    for (const relation of s.diplomacy!.relations) relation.state = 'War';
    const { fixtureClient, core } = pair(s);
    const mode = kind === 'Ranger' ? 'Ranged' : 'Melee';
    (expect(fixtureClient.forecast(0, attacker.id, target.pos, mode)).toMatchSnapshot(), void (core.forecast(0, attacker.id, target.pos, mode)));
    const order: Order = kind === 'Ranger'
      ? { type: 'RangedAttack', unit: attacker.id, target: target.pos }
      : { type: 'MoveUnit', unit: attacker.id, to: target.pos };
    const a = fixtureClient.endTurn({ 0: [order] }), b = core.endTurn({ 0: [order] });
    const hits = (events: typeof a.events) => events.filter((e) => e.type === 'Battle' && e.attacker === attacker.id);
    (expect(hits(a.events)).toMatchSnapshot(), void (hits(b.events)));
    const hit = hits(b.events)[0];
    expect(hit?.type).toBe('Battle');
    if (hit?.type === 'Battle') expect(hit.damage_to_defender).toBe(damage(hit.attacker_strength.total, hit.defender_strength.total, hit.attacker_roll));
    (expect(cityOf(fixtureClient.view(0), target.id).hp).toMatchSnapshot(), void (cityOf(core.view(0), target.id).hp));
  });

  describe('production', () => {
    it.each(FACTIONS.filter((f) => f !== 'Egypt'))('is offered to Mongols only, with the same wording (%s)', (faction) => {
      const s = fixture(faction, faction === 'Mongols' ? 'India' : 'Mongols', 200);
      const a = city(s, 0, 4, 4); city(s, 1, 16, 16);
      s.players[0].gold = 5000;
      const { fixtureClient, core } = pair(s);
      const l = listing(core.view(0), a.id);
      (expect(listing(fixtureClient.view(0), a.id)).toMatchSnapshot(), void (l));
      const errors = core.validate(0, hallOrders(a));
      (expect(fixtureClient.validate(0, hallOrders(a))).toMatchSnapshot(), void (errors));
      const buy: Order[] = [{ type: 'BuyItem', city: a.id, item: hallItem(a) }];
      (expect(fixtureClient.validate(0, buy)).toMatchSnapshot(), void (core.validate(0, buy)));
      if (faction === 'Mongols') {
        expect(l.options.length).toBeGreaterThan(0);
        expect(errors).toEqual([]);
      } else {
        expect(l.options).toEqual([]);
        expect(errors.map((e) => e.message)).toEqual([`${a.name} cannot produce Yam Relay Station: Only Mongols can build the Yam Relay Station`]);
      }
    });

    it('costs 600, buyable with Gold at the usual price', () => {
      const s = fixture('Mongols', 'India', 200);
      const a = city(s, 0, 4, 4); city(s, 1, 16, 16);
      s.players[0].gold = 5000;
      const { fixtureClient, core } = pair(s);
      const [option] = listing(core.view(0), a.id).options;
      expect(option.cost).toBe(600);
      expect(productionCost(s, 0, HALL)).toBe(600);
      expect(option.buy_cost).toBe(600 * 2);
      expect(listing(fixtureClient.view(0), a.id).options).toMatchSnapshot();
    });

    it('no faction price modifier touches it', () => {
      for (const faction of ['Byzantium', 'Germany'] as const) {
        // They cannot build it; the price table the core uses for a Mongols city is the listed figure.
        const s = fixture(faction, 'Mongols', 200);
        expect(productionCost(s, 0, HALL)).toBe(600);
      }
    });

    it('says why not: it stands, it stands in a captured city, it is being built or queued elsewhere, and a razed Hall can be rebuilt', () => {
      const cases: { name: string; build: (s: MState) => void; reason: string | null }[] = [
        { name: 'it stands in your other city', build: (s) => { hallCity(s, 0, 0, 4, 10); }, reason: 'The Yam Relay Station already stands in C2' },
        { name: 'a rival captured its city', build: (s) => { hallCity(s, 0, 1, 14, 14); }, reason: 'The Yam Relay Station stands in C2; retake the city instead' },
        { name: 'it is being built in another city', build: (s) => { city(s, 0, 4, 12, { producing: HALL }); }, reason: 'The Yam Relay Station is already being built in C2' },
        { name: 'it is queued in another city', build: (s) => { city(s, 0, 4, 12, { producing: { kind: 'Unit', unit: 'Soldier' }, productionQueue: [HALL] }); }, reason: 'The Yam Relay Station is already being built in C2' },
        { name: 'it was razed: the stale city id points at nothing', build: (s) => { s.players[0].hallCity = 99; }, reason: null },
        { name: 'its city no longer holds the building', build: (s) => { const c = city(s, 0, 4, 12); s.players[0].hallCity = c.id; }, reason: null },
      ];
      for (const c of cases) {
        const s = fixture('Mongols', 'India');
        const a = city(s, 0, 4, 4);
        c.build(s);
        city(s, 1, 17, 17);
        const { fixtureClient, core } = pair(s);
        const l = listing(core.view(0), a.id);
        (expect(listing(fixtureClient.view(0), a.id), c.name).toMatchSnapshot(), void (l));
        (expect(fixtureClient.validate(0, hallOrders(a))).toMatchSnapshot(), void (core.validate(0, hallOrders(a))));
        if (c.reason) {
          expect(l.options, c.name).toEqual([]);
          expect(core.validate(0, hallOrders(a)).map((e) => e.message), c.name).toEqual([`${a.name} cannot produce Yam Relay Station: ${c.reason}`]);
        } else {
          expect(l.options.length, c.name).toBeGreaterThan(0);
          expect(core.validate(0, hallOrders(a)), c.name).toEqual([]);
        }
      }
    });

    it('a Hall already in the city is not offered again', () => {
      const s = fixture('Mongols', 'India');
      const a = hallCity(s, 0, 0, 4, 4); city(s, 1, 16, 16);
      const { fixtureClient, core } = pair(s);
      (expect(listing(fixtureClient.view(0), a.id)).toMatchSnapshot(), void (listing(core.view(0), a.id)));
      (expect(fixtureClient.validate(0, hallOrders(a))).toMatchSnapshot(), void (core.validate(0, hallOrders(a))));
      expect(core.validate(0, hallOrders(a))).not.toEqual([]);
    });
  });

  describe('completion and the goal', () => {
    it('completes into hall_city: the Hall is public, the goal scores at once and the owner meter says where it stands', () => {
      const s = fixture('Mongols', 'India');
      const a = city(s, 0, 4, 4); a.producing = hallItem(a);
      a.prod = productionCost(s, 0, HALL);
      city(s, 1, 16, 16);
      const { fixtureClient, core } = pair(s);
      expect(core.view(0).faction_goal).toMatchObject({ value: 0, target: 1, label: 'The Yam Relay Station is not built', live: false, done: false });
      (expect(fixtureClient.view(0).faction_goal).toMatchSnapshot(), void (core.view(0).faction_goal));
      expect(core.view(1).courier_halls ?? []).toEqual([]);
      const x = fixtureClient.endTurn({}), y = core.endTurn({});
      (expect(x.events).toMatchSnapshot(), void (y.events));
      expect(y.events.some((e) => e.type === 'ProductionCompleted')).toBe(true);
      expect(goalEvents(y.events)).toEqual([completed(0)]);
      for (const p of [0, 1]) {
        expect(core.view(p).courier_halls).toMatchObject([{ of: 0, holder: 0, city: a.id }]);
        (expect(fixtureClient.view(p).courier_halls).toMatchSnapshot(), void (core.view(p).courier_halls));
      }
      expect(core.view(0).faction_goal).toMatchObject({ value: 1, target: 1, label: `The Yam Relay Station stands in ${a.name}`, done: true });
      (expect(fixtureClient.view(0).faction_goal).toMatchSnapshot(), void (core.view(0).faction_goal));
      expect(goalSources(core.view(0), 0)).toEqual([[{ kind: 'FactionGoal', of: 0 }, 2]]);
      (expect(goalSources(fixtureClient.view(0), 0)).toMatchSnapshot(), void (goalSources(core.view(0), 0)));
      (expect(fixtureClient.view(0).players[0].vp_sources).toMatchSnapshot(), void (core.view(0).players[0].vp_sources));
    });

    it('buying it with Gold completes it the next turn, with the same events', () => {
      const s = fixture('Mongols', 'India', 200);
      const a = city(s, 0, 4, 4); a.producing = hallItem(a); city(s, 1, 16, 16);
      s.players[0].gold = 5000;
      const { fixtureClient, core } = pair(s);
      const buy: Order[] = [{ type: 'BuyItem', city: a.id, item: hallItem(a) }];
      expect(fixtureClient.validate(0, buy)).toEqual([]);
      expect(core.validate(0, buy)).toEqual([]);
      const x = fixtureClient.endTurn({ 0: buy }), y = core.endTurn({ 0: buy });
      (expect(x.events).toMatchSnapshot(), void (y.events));
      (expect(cityOf(fixtureClient.view(0), a.id).detail!.buildings).toMatchSnapshot(), void (cityOf(core.view(0), a.id).detail!.buildings));
      expect(cityOf(core.view(0), a.id).detail!.buildings).toContain('CourierHall');
      (expect(fixtureClient.view(1).courier_halls).toMatchSnapshot(), void (core.view(1).courier_halls));
      expect(core.view(1).courier_halls).toHaveLength(1);
    });

    // The Hall changes hands with its city. `held` is `goal_holder` at the start of the turn.
    const phases: { name: string; owner: number | null; held: number | null; before: boolean; resisting?: number; expected: object[]; scorer: number | null; label: string }[] = [
      { name: 'the Mongols built it and holds it', owner: 0, held: null, before: false, expected: [completed(0)], scorer: 0, label: 'stands' },
      { name: 'still held: nothing changes', owner: 0, held: 0, before: true, expected: [], scorer: 0, label: 'stands' },
      { name: 'a rival owns its city: the loss comes first, then the rival scores', owner: 1, held: 0, before: true, expected: [lost(0), completed(1, true)], scorer: 1, label: 'rival' },
      { name: 'a rival took it on a first completion', owner: 1, held: null, before: false, expected: [completed(1)], scorer: 1, label: 'rival' },
      { name: 'the Mongols retakes the city', owner: 0, held: 1, before: true, expected: [lost(1), completed(0, true)], scorer: 0, label: 'stands' },
      { name: 'resistance does not matter', owner: 0, held: null, before: false, resisting: 3, expected: [completed(0)], scorer: 0, label: 'stands' },
      { name: 'razed: the Hall is gone and the VP with it', owner: null, held: 0, before: true, expected: [lost(0)], scorer: null, label: 'none' },
      { name: 'razed on a lost Hall: nothing to report', owner: null, held: null, before: true, expected: [], scorer: null, label: 'none' },
    ];
    it.each(phases)('$name', (phase) => {
      const s = fixture('Mongols', 'India');
      city(s, 0, 4, 4);
      const far = city(s, 1, 16, 16);
      if (phase.owner !== null) {
        const c = phase.owner === 0 ? city(s, 0, 4, 10, { buildings: ['CourierHall'], resistance: phase.resisting ?? 0 }) : (far.buildings = ['CourierHall'], far);
        placeHall(s, c, 0);
        s.players[0].hallCity = c.id;
      } else s.players[0].hallCity = 77;
      s.players[0].goalHolder = phase.held; s.players[0].factionGoalReachedBefore = phase.before;
      const { fixtureClient, core } = pair(s);
      const name = phase.owner === 0 ? 'C3' : far.name;
      const label = { stands: `The Yam Relay Station stands in ${name}`, rival: `A rival holds the Yam Relay Station in ${name}`, none: 'The Yam Relay Station is not built' }[phase.label]!;
      expect(core.view(0).faction_goal).toMatchObject({ value: Number(phase.owner === 0), target: 1, label, live: false });
      (expect(fixtureClient.view(0).faction_goal).toMatchSnapshot(), void (core.view(0).faction_goal));
      for (const p of [0, 1]) (expect(fixtureClient.view(p).courier_halls).toMatchSnapshot(), void (core.view(p).courier_halls));
      const a = fixtureClient.endTurn({}), b = core.endTurn({});
      (expect(goalEvents(a.events)).toMatchSnapshot(), void (goalEvents(b.events)));
      (expect(goalEvents(b.events)).toMatchSnapshot(), void (phase.expected));
      for (const p of [0, 1]) {
        expect(goalSources(core.view(p), p), `empire ${p}`).toEqual(phase.scorer === p ? [[{ kind: 'FactionGoal', of: 0 }, 2]] : []);
        (expect(fixtureClient.view(p).players[p].vp_sources).toMatchSnapshot(), void (core.view(p).players[p].vp_sources));
        (expect(fixtureClient.view(p).courier_halls).toMatchSnapshot(), void (core.view(p).courier_halls));
      }
      expect(core.view(0).players[0].faction_goal_done).toBe(phase.owner === 0);
      (expect(fixtureClient.view(0).players[0].faction_goal_done).toMatchSnapshot(), void (core.view(0).players[0].faction_goal_done));
      (expect(fixtureClient.view(0).faction_goal).toMatchSnapshot(), void (core.view(0).faction_goal));
    });

    it('a rival captures the Hall city by taking it, and retaking it returns the VP', () => {
      const s = fixture('Mongols', 'India');
      city(s, 0, 4, 4);
      const hall = hallCity(s, 0, 0, 10, 10); hall.hp = 0;
      city(s, 1, 16, 16);
      for (const relation of s.diplomacy!.relations) relation.state = 'War';
      const soldier = spawnUnit(s, 1, 'Soldier', fromOffset(9, 10), false);
      s.players[0].goalHolder = 0; s.players[0].factionGoalDone = true; s.players[0].factionGoalReachedBefore = true;
      const { fixtureClient, core } = pair(s);
      const capture: Order[] = [{ type: 'MoveUnit', unit: soldier.id, to: hall.pos }];
      let a = fixtureClient.endTurn({ 1: capture }), b = core.endTurn({ 1: capture });
      (expect(a.events).toMatchSnapshot(), void (b.events));
      expect(b.events.some((e) => e.type === 'CityCaptured')).toBe(true);
      expect(goalEvents(b.events)).toEqual([lost(0), completed(1, true)]);
      for (const p of [0, 1]) {
        expect(core.view(p).courier_halls).toMatchObject([{ of: 0, holder: 1, city: hall.id }]);
        (expect(fixtureClient.view(p).courier_halls).toMatchSnapshot(), void (core.view(p).courier_halls));
        (expect(fixtureClient.view(p).players[p].vp_sources).toMatchSnapshot(), void (core.view(p).players[p].vp_sources));
      }
      expect(goalSources(core.view(1), 1)).toEqual([[{ kind: 'FactionGoal', of: 0 }, 2]]);
      expect(goalSources(core.view(0), 0)).toEqual([]);
      expect(core.view(0).faction_goal).toMatchObject({ value: 0, label: `A rival holds the Yam Relay Station in ${hall.name}`, done: false });
      (expect(fixtureClient.view(0).faction_goal).toMatchSnapshot(), void (core.view(0).faction_goal));
      // The Mongols may not build another while the captured one stands.
      const own = cityOf(core.view(0), 1);
      (expect(listing(fixtureClient.view(0), own.id)).toMatchSnapshot(), void (listing(core.view(0), own.id)));
      expect(listing(core.view(0), own.id).options).toEqual([]);
      expect(core.validate(0, hallOrders({ id: own.id, pos: own.pos })).map((e) => e.message)).toEqual([
        `${own.name} cannot produce Yam Relay Station: The Yam Relay Station stands in ${hall.name}; retake the city instead`,
      ]);
      (expect(fixtureClient.validate(0, hallOrders(own))).toMatchSnapshot(), void (core.validate(0, hallOrders(own))));
    });

    it('a Celtic capture sacks the Hall: the VP is lost, the Mongols can rebuild, and the new Hall scores again', () => {
      const s = fixture('Mongols', 'Celts', 200);
      const home = city(s, 0, 4, 4);
      const hall = hallCity(s, 0, 0, 10, 10); hall.hp = 0;
      city(s, 1, 16, 16);
      for (const relation of s.diplomacy!.relations) relation.state = 'War';
      const celts = spawnUnit(s, 1, 'Soldier', fromOffset(9, 10), false);
      s.players[0].goalHolder = 0; s.players[0].factionGoalDone = true; s.players[0].factionGoalReachedBefore = true;
      s.players[0].gold = 5000;
      const { fixtureClient, core } = pair(s);
      const capture: Order[] = [{ type: 'MoveUnit', unit: celts.id, to: hall.pos }];
      const a = fixtureClient.endTurn({ 1: capture }), b = core.endTurn({ 1: capture });
      (expect(a.events).toMatchSnapshot(), void (b.events));
      expect(b.events.some((e) => e.type === 'CityRazed')).toBe(true);
      expect(goalEvents(b.events)).toEqual([lost(0)]);
      for (const p of [0, 1]) {
        expect(core.view(p).courier_halls ?? []).toEqual([]);
        (expect(fixtureClient.view(p).courier_halls ?? []).toMatchSnapshot(), void (core.view(p).courier_halls ?? []));
        (expect(fixtureClient.view(p).players[p].vp_sources).toMatchSnapshot(), void (core.view(p).players[p].vp_sources));
      }
      expect(core.view(0).faction_goal).toMatchObject({ value: 0, label: 'The Yam Relay Station is not built', done: false });
      (expect(fixtureClient.view(0).faction_goal).toMatchSnapshot(), void (core.view(0).faction_goal));
      // Rebuild: offered again, bought, and it scores "again".
      (expect(listing(fixtureClient.view(0), home.id)).toMatchSnapshot(), void (listing(core.view(0), home.id)));
      expect(listing(core.view(0), home.id).options.length).toBeGreaterThan(0);
      const rebuild: Order[] = [{ type: 'SetProduction', city: home.id, item: hallItem(home) }, { type: 'BuyItem', city: home.id, item: hallItem(home) }];
      (expect(fixtureClient.validate(0, rebuild)).toMatchSnapshot(), void (core.validate(0, rebuild)));
      expect(core.validate(0, rebuild)).toEqual([]);
      const c = fixtureClient.endTurn({ 0: rebuild }), d = core.endTurn({ 0: rebuild });
      expect(c.events).toMatchSnapshot();
      expect(goalEvents(d.events)).toEqual([completed(0, true)]);
      expect(core.view(0).courier_halls).toMatchObject([{ of: 0, holder: 0, city: home.id }]);
      (expect(fixtureClient.view(0).courier_halls).toMatchSnapshot(), void (core.view(0).courier_halls));
      (expect(fixtureClient.view(0).players[0].vp_sources).toMatchSnapshot(), void (core.view(0).players[0].vp_sources));
    });

    it('two Mongols empires each have their own Hall; one holder scores both goals', () => {
      const s = fixture('Mongols', 'Mongols');
      city(s, 0, 4, 4);
      const h0 = hallCity(s, 0, 1, 12, 12);
      const h1 = hallCity(s, 1, 1, 16, 4);
      const { fixtureClient, core } = pair(s);
      const a = fixtureClient.endTurn({}), b = core.endTurn({});
      (expect(a.events).toMatchSnapshot(), void (b.events));
      expect(goalEvents(b.events)).toEqual([completed(1), completed(1)]);
      expect(core.view(1).courier_halls).toMatchObject([{ of: 0, holder: 1, city: h0.id }, { of: 1, holder: 1, city: h1.id }]);
      for (const p of [0, 1]) {
        (expect(fixtureClient.view(p).courier_halls).toMatchSnapshot(), void (core.view(p).courier_halls));
        (expect(fixtureClient.view(p).players[p].vp_sources).toMatchSnapshot(), void (core.view(p).players[p].vp_sources));
      }
      expect(goalSources(core.view(1), 1)).toHaveLength(2);
    });
  });

  describe('the open road: passage through the holder’s land', () => {
    /** Player 0 holds a city with (or without) a Courier Hall of the Mongols empire; a unit of player 1 stands two hexes outside it and is told to step one hex in. */
    function crossing(holder: 'Mongols' | 'India', kind: UnitKind, hall: boolean) {
      const mover = holder === 'Mongols' ? 'India' : 'Mongols';
      const s = fixture(holder, mover);
      const home = city(s, 0, 10, 10, { buildings: hall ? ['CourierHall'] : [] });
      if (hall) placeHall(s, home, 0);
      city(s, 0, 3, 3); city(s, 1, 16, 16);
      if (hall) s.players[holder === 'Mongols' ? 0 : 1].hallCity = home.id;
      const target = neighbors(home.pos)[0];
      const start = neighbors(target).find((h) => distance(h, home.pos) === 2)!;
      const unit = spawnUnit(s, 1, kind, start, false);
      return { s, unit, target };
    }
    const MOVERS = UNIT_KINDS.filter((k) => k !== 'Heir');

    function cross(holder: 'Mongols' | 'India', kind: UnitKind, hall: boolean) {
      const { s, unit, target } = crossing(holder, kind, hall);
      const { fixtureClient, core } = pair(s);
      const order: Order[] = [{ type: 'MoveUnit', unit: unit.id, to: target }];
      const a = fixtureClient.endTurn({ 1: order }), b = core.endTurn({ 1: order });
      (expect(a.events).toMatchSnapshot(), void (b.events));
      const at = (c: FixtureClient | WasmClient) => c.view(1).units.find((u) => u.id === unit.id)!.pos;
      (expect(at(fixtureClient)).toMatchSnapshot(), void (at(core)));
      const entered = JSON.stringify(at(core)) === JSON.stringify(target);
      // The view-side rule (diplomacy.canEnter) reads courier_halls and agrees with what the core did, from either engine's view.
      // The core refuses a move order into closed land up front (the fixtureClient's order check does not look at borders).
      expect(core.validate(1, order).length === 0, `${kind} hall=${hall}`).toBe(entered);
      expect(canEnter(core.view(1), kind, 0), `view ${kind} hall=${hall}`).toBe(entered);
      expect(canEnter(fixtureClient.view(1), kind, 0), `fixtureClient view ${kind} hall=${hall}`).toBe(entered);
      return entered;
    }

    it.each(MOVERS)('%s: a India mover is shut out of Mongols land at Peace, and let in once a Hall stands there', (kind) => {
      expect(cross('Mongols', kind, false)).toBe(false);
      expect(cross('Mongols', kind, true)).toBe(true);
    });

    it.each(MOVERS)('%s: a Mongols mover enters an India holder’s land with the Hall but is shut out without it', (kind) => {
      expect(cross('India', kind, false)).toBe(false);
      expect(cross('India', kind, true)).toBe(true);
    });

    it('opens only the holder’s land: once a rival holds it, the Mongols’ own cities are shut to everyone but their own team', () => {
      const s = fixture('Mongols', 'India');
      city(s, 0, 3, 3);
      const hall = hallCity(s, 0, 1, 14, 14);
      const { fixtureClient, core } = pair(s);
      for (const client of [fixtureClient, core]) {
        const v = client.view(1);
        expect(canEnter(v, 'Soldier', 0)).toBe(false);
        expect(canEnter(v, 'Soldier', 1)).toBe(true);
        const w = client.view(0);
        expect(canEnter(w, 'Soldier', 0)).toBe(true);
        expect(canEnter(w, 'Soldier', 1)).toBe(true);
        expect(w.courier_halls).toMatchObject([{ of: 0, holder: 1, city: hall.id }]);
      }
    });
  });
});
