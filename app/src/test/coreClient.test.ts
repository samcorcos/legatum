import { describe, it, expect } from 'vitest';
import { FixtureClient, spawnUnit } from './fixtureEngine';
import { distance, hexEq, neighbors, tileIndex, within } from '../hex';
import { findPath } from '../path';
import { FIRST_GROWTH, moveCost } from '../data';
import type { Event, Hex, Order } from '../types';

async function newClient(seed = '42') {
  const c = new FixtureClient();
  const setup = await c.standardSetup(seed, 'Switzerland', 'India', 'Ada', 'Bo');
  await c.newGame(setup);
  return c;
}

function ownUnit(c: FixtureClient, p: number, kind: string) {
  const u = c.view(p).units.find((x) => x.owner === p && x.kind === kind);
  if (!u) throw new Error(`no ${kind} for ${p}`);
  return u;
}

function isAdjacent(a: Hex, b: Hex) {
  return distance(a, b) === 1;
}

/** One step, or a hop across friends that stay put (core `path::hop`): never more than a unit's moves away. */
function isStepOrHop(a: Hex, b: Hex) {
  return distance(a, b) >= 1 && distance(a, b) <= 4;
}

describe('FixtureClient setup and view', () => {
  it('builds the standard setup', async () => {
    const c = new FixtureClient();
    const setup = await c.standardSetup('42', 'India', 'Byzantium', 'Ada', 'Bo');
    expect(setup.width).toBe(48);
    expect(setup.height).toBe(32);
    expect(setup.turn_limit).toBeNull();
    expect(setup.players).toHaveLength(8);
    expect(setup.players[0]).toEqual({ name: 'Ada', faction: 'India', team: 0, is_human: true });
    expect(setup.players[1]).toEqual({ name: 'Bo', faction: 'Byzantium', team: 0, is_human: true });
    setup.players.slice(2).forEach((p, i) => {
      expect(p.team).toBe(i + 1);
      expect(p.is_human).toBe(false);
    });
  });

  it('starts a game with capitals, two units each and shared team vision', async () => {
    const c = await newClient();
    const v = c.view(0);
    expect(v.tiles).toHaveLength(48 * 32);
    expect(v.turn).toBe(1);
    expect(v.me).toBe(0);
    const own = v.units.filter((u) => u.owner === 0).map((u) => u.kind).sort();
    expect(own).toEqual(['Scout', 'Soldier']);
    expect(v.cities.find((city) => city.owner === 0)).toMatchObject({ is_capital: true, population: 1 });
    expect(v.standings.find((s) => s.team === 0)).toMatchObject({ members: [0, 1] });
    expect(v.standings.every((s) => s.target > 0)).toBe(true);
    const explored = v.tiles.filter((t) => t.explored);
    expect(explored.length).toBeGreaterThan(10);
    expect(explored.length).toBeLessThan(v.tiles.length / 2);
    expect(v.tiles.filter((t) => !t.explored).every((t) => t.terrain === null)).toBe(true);
    // Teammate's units are in view, but their gotos are hidden.
    const mate = v.units.filter((u) => u.owner === 1);
    expect(mate.length).toBe(2);
    expect(mate.every((u) => u.goto === null && u.upgrade_cost === null)).toBe(true);
    expect(v.players).toHaveLength(8);
    expect(v.great_works?.length ?? 0).toBeGreaterThan(0);
    expect(v.research.available.length).toBeGreaterThan(0);
  });

  it('generates one believable continent with spaced starts', async () => {
    const c = await newClient('7');
    const s = c.debugState();
    const land = s.terrain.filter((t) => t !== 'Ocean' && t !== 'Coast').length;
    expect(land / s.terrain.length).toBeGreaterThan(0.38);
    expect(land / s.terrain.length).toBeLessThan(0.52);
    const starts = s.players.map((p) => p.start);
    for (let i = 0; i < starts.length; i++)
      for (let j = i + 1; j < starts.length; j++) expect(distance(starts[i], starts[j])).toBeGreaterThanOrEqual(7);
    expect(distance(starts[0], starts[1])).toBeLessThanOrEqual(10);
    const nearestAi = Math.min(...starts.slice(2).map((h) => Math.min(distance(h, starts[0]), distance(h, starts[1]))));
    expect(nearestAi).toBeLessThanOrEqual(12);
    // Every passable land tile is reachable from the first human's start.
    const cost = (h: Hex) => {
      const i = tileIndex(h, s.width, s.height);
      return i < 0 ? null : moveCost(s.terrain[i]);
    };
    for (const h of starts.slice(1)) expect(findPath(s.width, s.height, starts[0], h, cost)).not.toBeNull();
    const ruins = s.ruin.filter(Boolean).length;
    expect(ruins).toBeGreaterThanOrEqual(10);
    expect(ruins).toBeLessThanOrEqual(12);
  });

  it('view() is stable and does not change game state', async () => {
    const c = await newClient();
    for (let i = 0; i < 10; i++) c.endTurn({});
    const before = structuredClone(c.debugState());
    const first = c.view(0);
    const second = c.view(0);
    expect(second).toEqual(first);
    expect(c.debugState()).toEqual(before);
  });
});

describe('research upgrade previews', () => {
  it('matches the gold charged', async () => {
    const c = await newClient();
    const s = c.debugState();
    const soldier = s.units.find((u) => u.owner === 0 && u.kind === 'Soldier')!;
    s.owner[tileIndex(soldier.pos, s.width, s.height)] = 0;
    // The Phalangite uses 1 Iron: an Iron deposit under the Spearman, inside the border, with the technology that reveals it.
    (s.resource ??= s.terrain.map(() => null))[tileIndex(soldier.pos, s.width, s.height)] = 'Iron';
    s.players[0].techs.push('Mathematics', 'IronWorking');
    s.players[0].gold = 1000;
    const advertised = c.view(0).research.unlocks.find((unlock) =>
      unlock.item.kind === 'Unit' && unlock.item.unit === 'Phalangite')!.upgrade_cost;
    expect(advertised).not.toBeNull();
    expect(ownUnit(c, 0, 'Soldier').upgrade_cost).toBe(advertised);
    const result = c.endTurn({ 0: [{ type: 'UpgradeUnit', unit: soldier.id }] });
    expect(result.events).toContainEqual({ type: 'UnitUpgraded', unit: soldier.id, owner: 0, to: 'Phalangite', gold: advertised });
    expect(ownUnit(c, 0, 'Phalangite').id).toBe(soldier.id);
  });
});

describe('founding a city', () => {
  it('starts with the capital founded and shows detail only to the owner', async () => {
    const c = await newClient();
    const v0 = c.view(0);
    expect(v0.turn).toBe(1);
    const city = v0.cities.find((x) => x.owner === 0)!;
    expect(city).toBeTruthy();
    expect(city.is_capital).toBe(true);
    const d = city.detail!;
    expect(d).not.toBeNull();
    expect(d.yields.sources.length).toBeGreaterThan(0);
    expect(d.yields.sources.map((x) => x.label)).toContain('City center');
    expect(d.options.length).toBeGreaterThan(0);
    // A Settler needs population 2: shown greyed out with the reason, not as a choice.
    expect(d.options.some((o) => o.item.kind === 'Unit' && o.item.unit === 'Settler')).toBe(false);
    expect(d.unavailable).toContainEqual({ item: { kind: 'Unit', unit: 'Settler' }, cost: 70, reason: 'Settler needs population 2 (currently 1)', production_banked: 0 });
    expect(d.territory.length).toBeGreaterThanOrEqual(4);
    expect(d.growth_threshold).toBe(FIRST_GROWTH);
    expect(d.next_claim).not.toBeNull();
    expect(v0.units.some((u) => u.owner === 0 && u.kind === 'Settler')).toBe(false);
    expect(v0.economy.gold_per_turn.parts.some(([label]) => label === city.name)).toBe(true);

    const v1 = c.view(1);
    const seen = v1.cities.find((x) => x.id === city.id)!;
    expect(seen).toBeTruthy();
    expect(seen.detail).toBeNull();

    // Production, buying and a new unit.
    const soldier = { kind: 'Unit' as const, unit: 'Soldier' as const };
    const orders: Order[] = [{ type: 'SetProduction', city: city.id, item: soldier }];
    expect(c.validate(0, orders)).toEqual([]);
    c.endTurn({ 0: orders });
    const d2 = c.view(0).cities.find((x) => x.id === city.id)!.detail!;
    expect(d2.producing).toEqual(soldier);
    const soldierCost = d2.options.find((option) => option.item.kind === 'Unit' && option.item.unit === 'Soldier')?.cost;
    expect(d2.buy_cost).toBe(Math.trunc(((soldierCost ?? 0) * 175) / 100));
  });
});

describe('validate', () => {
  it('rejects bad orders with the right indices', async () => {
    const c = await newClient();
    const scout = ownUnit(c, 0, 'Scout');
    const soldier = ownUnit(c, 0, 'Soldier');
    const dest = within(scout.pos, 2).find((h) => {
      const t = c.view(0).tiles[tileIndex(h, 48, 32)];
      return t.explored && moveCost(t.terrain) !== null && !hexEq(h, scout.pos);
    })!;
    expect(c.validate(0, [{ type: 'MoveUnit', unit: scout.id, to: dest }])).toEqual([]);

    const twice = c.validate(0, [
      { type: 'MoveUnit', unit: scout.id, to: dest },
      { type: 'SleepUnit', unit: scout.id },
    ]);
    expect(twice).toHaveLength(1);
    expect(twice[0].index).toBe(1);

    const found = c.validate(0, [{ type: 'FoundCity', unit: scout.id }]);
    expect(found).toHaveLength(1);
    expect(found[0].index).toBe(0);
    expect(found[0].message).toMatch(/settler/i);

    const mate = ownUnit(c, 1, 'Scout');
    expect(c.validate(0, [{ type: 'SleepUnit', unit: mate.id }])[0].message).toMatch(/yours/);
    expect(c.validate(0, [{ type: 'MoveUnit', unit: soldier.id, to: { q: 500, r: 500 } }])[0].message).toMatch(/off the map/);
    expect(c.validate(0, [{ type: 'RangedAttack', unit: soldier.id, target: dest }])[0].message).toMatch(/no ranged/);
    expect(c.validate(0, [{ type: 'UpgradeUnit', unit: soldier.id }])).toHaveLength(1);
    expect(c.validate(0, [{ type: 'SetResearch', tech: 'Mathematics' }])[0].message).toBe('Mathematics is in the Classical era: finish the Ancient era first');
    expect(c.validate(0, [{ type: 'SetResearch', tech: 'BronzeWorking' }])).toEqual([]);
    expect(c.validate(0, [{ type: 'BuyItem', city: 999, item: { kind: 'Unit', unit: 'Soldier' } }])).toHaveLength(1);
  });

  it('rejects founding next to an existing city', async () => {
    const c = await newClient();
    const s = c.debugState();
    const city = s.cities.find((x) => x.owner === 0)!;
    const near = neighbors(city.pos).find((h) => moveCost(s.terrain[tileIndex(h, 48, 32)]) !== null)!;
    const nextSettler = spawnUnit(s, 0, 'Settler', near, false);
    const before = s.cities.length;
    const result = c.endTurn({ 0: [{ type: 'FoundCity', unit: nextSettler.id }] });
    expect(result.events.some((e) => e.type === 'OrderFailed' && /Too close|belongs/i.test(e.reason))).toBe(true);
    expect(c.debugState().cities).toHaveLength(before);
  });
});

describe('movement', () => {
  it('keeps a MoveUnit goto across turns until the unit arrives, one adjacent step per event', async () => {
    const c = await newClient();
    const s = c.debugState();
    const scout = ownUnit(c, 0, 'Scout');
    const cost = (h: Hex) => {
      const i = tileIndex(h, s.width, s.height);
      return i < 0 ? null : moveCost(s.terrain[i]);
    };
    // A reachable, empty destination 7-9 hexes away, away from other empires.
    const dest = within(scout.pos, 9).find((h) => {
      const d = distance(h, scout.pos);
      if (d < 7 || cost(h) === null) return false;
      if (s.units.some((u) => hexEq(u.pos, h))) return false;
      if (s.players.slice(2).some((p) => distance(p.start, h) < 5)) return false;
      const path = findPath(s.width, s.height, scout.pos, h, cost);
      return !!path && path.length <= 10;
    })!;
    expect(dest).toBeTruthy();
    const first = c.endTurn({ 0: [{ type: 'MoveUnit', unit: scout.id, to: dest }] });
    let pos = scout.pos;
    const check = (events: Event[]) => {
      for (const e of events) {
        if (e.type !== 'UnitMoved' || e.unit !== scout.id) continue;
        expect(isAdjacent(e.from!, e.to!)).toBe(true);
        expect(hexEq(e.from!, pos)).toBe(true);
        pos = e.to!;
      }
    };
    check(first.events);
    const after1 = c.view(0).units.find((u) => u.id === scout.id)!;
    expect(hexEq(after1.pos, pos)).toBe(true);
    expect(hexEq(after1.goto, dest)).toBe(true); // 7+ hexes: not there after one turn
    for (let t = 0; t < 8 && !hexEq(pos, dest); t++) check(c.endTurn({}).events);
    const done = c.view(0).units.find((u) => u.id === scout.id)!;
    expect(hexEq(done.pos, dest)).toBe(true);
    expect(done.goto).toBeNull();
  });
});

describe('forecast', () => {
  it('returns odds and breakdowns for an adjacent enemy, null otherwise', async () => {
    const c = await newClient();
    const s = c.debugState();
    const soldier = s.units.find((u) => u.owner === 0 && u.kind === 'Soldier')!;
    const enemy = s.units.find((u) => u.owner === 2 && u.kind === 'Soldier')!;
    const spot = neighbors(soldier.pos).find(
      (h) => moveCost(s.terrain[tileIndex(h, 48, 32)]) !== null && !s.units.some((u) => hexEq(u.pos, h)),
    )!;
    expect(c.forecast(0, soldier.id, spot, 'Melee')).toBeNull();
    enemy.pos = { ...spot };
    for (const r of s.diplomacy!.relations) r.state = 'War';
    const f = c.forecast(0, soldier.id, spot, 'Melee')!;
    expect(f).not.toBeNull();
    expect(f.win_chance).toBeGreaterThanOrEqual(0);
    expect(f.win_chance).toBeLessThanOrEqual(100);
    expect(f.loss_chance).toBeGreaterThanOrEqual(0);
    expect(f.loss_chance).toBeLessThanOrEqual(100);
    expect(f.attacker_strength.parts.length).toBeGreaterThan(0);
    expect(f.defender_strength.parts.length).toBeGreaterThan(0);
    expect(f.attacker_hp).toBe(100);
    expect(f.defender_hp).toBe(100);
    expect(f.expected_damage_to_defender).toBeGreaterThan(0);
    // The enemy cannot forecast with our unit.
    expect(c.forecast(2, soldier.id, spot, 'Melee')).toBeNull();

    // Attacking resolves as a Battle with the numbers behind it.
    const r = c.endTurn({ 0: [{ type: 'MoveUnit', unit: soldier.id, to: spot }] });
    const battle = r.events.find((e) => e.type === 'Battle' && e.attacker === soldier.id);
    expect(battle).toBeTruthy();
    if (battle && battle.type === 'Battle') {
      expect(battle.attacker_roll).toBeGreaterThanOrEqual(80);
      expect(battle.attacker_roll).toBeLessThanOrEqual(120);
      expect(battle.attacker_strength.total).toBe(f.attacker_strength.total);
    }
  });
});

describe('determinism, long runs and saves', () => {
  it('produces identical hashes for the same seed and orders', async () => {
    const a = await newClient('1234');
    const b = await newClient('1234');
    expect(a.hash()).toBe(b.hash());
    const scout = ownUnit(a, 1, 'Scout');
    for (let t = 0; t < 10; t++) {
      const orders: Record<number, Order[]> =
        t === 0
          ? { 0: [{ type: 'SetResearch', tech: 'BronzeWorking' }], 1: [{ type: 'SleepUnit', unit: scout.id }] }
          : {};
      const ra = a.endTurn(orders);
      const rb = b.endTurn(orders);
      expect(ra.hash).toBe(rb.hash);
      expect(ra.events).toEqual(rb.events);
    }
    const c = await newClient('1235');
    expect(c.hash()).not.toBe(a.hash());
  });

  it('runs 30 turns with lively AIs', async () => {
    const c = await newClient();
    const all: Event[] = [];
    for (let t = 0; t < 30; t++) {
      const r = c.endTurn({});
      expect(r.turn).toBe(t + 1);
      all.push(...r.events);
    }
    const v = c.view(0);
    expect(v.turn).toBe(31);
    const aiCities = c.debugState().cities.filter((x) => x.owner >= 2);
    expect(aiCities.length).toBeGreaterThanOrEqual(6);
    expect(all.some((e) => e.type === 'CityFounded' && e.owner >= 2)).toBe(true);
    expect(all.some((e) => e.type === 'TechResearched')).toBe(true);
    for (const e of all) if (e.type === 'UnitMoved') expect(isStepOrHop(e.from!, e.to!)).toBe(true);
  }, 20_000); // ~5 s alone; slower under heavy machine load

  it('round-trips save/load exactly', async () => {
    const c = await newClient();
    c.endTurn({});
    for (let t = 0; t < 5; t++) c.endTurn({});
    const json = c.save();
    const d = new FixtureClient();
    d.load(json);
    expect(d.view(0)).toEqual(c.view(0));
    expect(d.view(1)).toEqual(c.view(1));
    expect(d.hash()).toBe(c.hash());
    expect(d.endTurn({}).hash).toBe(c.endTurn({}).hash);
  });

  it('refuses to play past the end', async () => {
    const c = await newClient();
    c.debugState().finished = true;
    expect(() => c.endTurn({})).toThrow();
  });

  it('accepts seat keys as strings and reports invalid human orders', async () => {
    const c = await newClient();
    const scout = ownUnit(c, 0, 'Scout');
    const r = c.endTurn(JSON.parse(JSON.stringify({ 0: [{ type: 'FoundCity', unit: scout.id }] })));
    expect(r.events.some((e) => e.type === 'OrderFailed' && e.player === 0)).toBe(true);
  });
});
