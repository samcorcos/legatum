// Friendly occupancy in route choice (playtest-6.2 "The Last Bridge"). Mirrors the Rust tests
// path::tests::friends_shape_routes_blocked_risky_and_choke and rules/friendly_route_tests.rs.
import { describe, expect, it } from 'vitest';
import type { Hex, Order } from './types';
import { fromOffset, neighbors, tileIndex } from './hex';
import { crossingFaults, findPath, hop, pathTurns, previewRoute, routeAroundFriends, routeRiskText, routeTurnsText, type CostFn } from './path';
import { createState, makeStandardSetup, spawnUnit } from './test/fixtureEngine';
import { idxOf, refreshVision } from './test/fixtureEngine';
import { viewFromFixture } from './test/fixtureEngine';
import type { MCity, MState } from './test/fixtureState';

const eq = (a: Hex, b: Hex) => a.q === b.q && a.r === b.r;
const always = () => true;
const never = () => false;

describe('routeTurnsText', () => {
  const step = (turn: number) => ({ hex: { q: 0, r: 0 }, turn, endsTurn: false });
  it('names the turns, and explains a route much longer than the straight line', () => {
    const from = { q: 0, r: 0 }, to = { q: 4, r: 0 };
    expect(routeTurnsText([1, 1, 2, 2].map(step), from, to)).toBe('2 turn path.');
    const bent = routeTurnsText([1, 1, 2, 2, 3, 3, 4, 4].map(step), from, to);
    expect(bent).toContain('4 turn path going round terrain or units: 8 tiles for 4 in a straight line');
  });
});

describe('routeAroundFriends (mirrors core path::route_around_friends)', () => {
  const W = 10, H = 10;
  const open: CostFn = (h) => (tileIndex(h, W, H) < 0 ? null : 1);
  const to = fromOffset(5, 5);
  const [m, f] = [neighbors(to)[0], neighbors(to)[1]];
  const from = { q: m.q + f.q - to.q, r: m.r + f.r - to.r };
  const route = (cost: CostFn, blocked: (h: Hex) => boolean, passable: (h: Hex) => boolean, risky: (h: Hex) => boolean, a = from, b = to) =>
    routeAroundFriends(W, H, a, b, cost, always, blocked, passable, never, risky, 2, 2);

  it('routes around a wall, crosses a friend that stays, prefers free tiles over a leaving friend, never avoids the goal', () => {
    const direct = findPath(W, H, from, to, open)!;
    const mid = direct[0];
    const around = route(open, (h) => eq(h, mid), never, never)!;
    expect(around).toHaveLength(2);
    expect(around[0]).not.toEqual(mid);
    expect(route(open, never, (h) => eq(h, mid), never)).toEqual(direct);
    expect(route(open, never, never, (h) => eq(h, mid))).toEqual(around);
    expect(route(open, (h) => eq(h, mid), (h) => eq(h, mid), (h) => eq(h, mid), from, mid)).toEqual([mid]);
  });

  it('keeps the plain route in a choke and a risky route when the free one takes an extra turn', () => {
    const [start, through, end] = [fromOffset(2, 4), fromOffset(3, 4), fromOffset(4, 4)];
    const row: CostFn = (h) => (tileIndex(h, W, H) >= 0 && h.r === start.r ? 1 : null);
    expect(routeAroundFriends(W, H, start, end, row, always, (h) => eq(h, through), never, never, never, 2, 2)).toEqual([through, end]);
    const mid = findPath(W, H, from, to, open)![0];
    const forest: CostFn = (h) => (tileIndex(h, W, H) < 0 ? null : neighbors(from).some((n) => eq(n, h)) && !eq(h, mid) ? 2 : 1);
    expect(route(forest, never, never, (h) => eq(h, mid))).toEqual([mid, to]);
  });

  describe('a pass with a long way round, a friend in it', () => {
    const [start, friend, goal] = [fromOffset(4, 4), fromOffset(5, 4), fromOffset(10, 4)];
    const cells = new Set<string>();
    const add = (c: number, r: number) => { const h = fromOffset(c, r); cells.add(`${h.q},${h.r}`); };
    for (let c = 1; c <= 10; c++) { add(c, 4); add(c, 8); }
    for (let r = 5; r <= 7; r++) { add(1, r); add(10, r); }
    const map: CostFn = (h) => (cells.has(`${h.q},${h.r}`) ? 1 : null);
    const hills = fromOffset(6, 4);
    const slow: CostFn = (h) => (eq(h, hills) ? 2 : map(h));
    const run = (cost: CostFn, passable: boolean, waiting: boolean) =>
      routeAroundFriends(12, 10, start, goal, cost, always, never, (h) => passable && eq(h, friend), (h) => waiting && eq(h, friend), never, 2, 2)!;

    it('crosses a friend that stays, and walks round one that is a wall', () => {
      expect(run(map, true, false)).toEqual(findPath(12, 10, start, goal, map)!);
      expect(routeAroundFriends(12, 10, start, goal, map, always, (h) => eq(h, friend), never, never, never, 2, 2)!.length).toBeGreaterThan(12);
    });

    it('follows a reserved first step through the pass instead of circling it', () => {
      const direct = findPath(12, 10, start, goal, map)!;
      const route = routeAroundFriends(12, 10, start, goal, map, always, never, never, never, never, 2, 2, true, undefined, 1, (h) => eq(h, friend));
      expect(route).toEqual(direct);
      expect(route?.[0]).toEqual(friend);
    });

    it('waits behind a friend it cannot cross that moves on later rather than circling (playtest 8.6)', () => {
      // The friend sits on hills (cost 2) with the next tile beyond it (cost 1): a Soldier cannot pay for both.
      expect(run(slow, true, false).length).toBeGreaterThan(12);
      expect(run(slow, true, true)).toEqual(findPath(12, 10, start, goal, slow)!);
    });
  });
});

describe('crossingFaults, hop and pathTurns (mirror core path::crossing_faults, path::hop, path::path_turns)', () => {
  const row = (n: number) => Array.from({ length: n }, (_, i) => fromOffset(i + 1, 4));
  const cost: CostFn = () => 1;

  it('uses the railway bonus for a crossing that starts on a road', () => {
    const from = fromOffset(2, 4), friend = fromOffset(3, 4), landing = fromOffset(4, 4);
    const path = [friend, landing];
    const costly: CostFn = () => 2;
    const road = (h: Hex) => eq(h, from);
    const faults = (bonus: number) => crossingFaults(path, costly, never, (h) => eq(h, friend), 2, from, road, bonus);
    expect(faults(1)).toEqual([friend]);
    expect(faults(2)).toEqual([]);
  });

  it('hops over the run of friends and lands on the first free tile', () => {
    const path = row(5), friend = path[0];
    expect(hop(path, cost, (h) => eq(h, friend))).toEqual({ landing: path[1], total: 2 });
    expect(hop(path.slice(1), cost, (h) => eq(h, friend))).toBeNull();
    // A run that reaches the goal has no landing: a unit never ends a move on a friend.
    expect(hop(path.slice(0, 2), cost, always)).toBeNull();
  });

  it('flags crossings with no landing, a blocked goal beyond, or a cost over full moves', () => {
    const path = row(2), friend = path[0];
    const crossable = (h: Hex) => eq(h, friend);
    expect(crossingFaults(path, cost, never, crossable, 2)).toEqual([]);
    expect(crossingFaults(path, cost, (h) => eq(h, path[1]), crossable, 2)).toEqual([friend]);
    expect(crossingFaults(path, cost, never, crossable, 1)).toEqual([friend]);
    expect(crossingFaults(path, cost, never, always, 2)).toEqual([path[0]]);
    expect(crossingFaults(path.slice(1), cost, never, always, 2)).toEqual([]);
  });

  it('a hop that does not fit the moves left waits for the next turn, and no turn ends on a friend', () => {
    const path = row(3), friend = path[0];
    const crossable = (h: Hex) => eq(h, friend);
    const turns = (moves: number) => pathTurns(path.slice(0, 2), cost, 2, moves, fromOffset(0, 4), undefined, 1, crossable);
    expect(turns(2).map((s) => s.turn)).toEqual([1, 1]);
    expect(turns(1).map((s) => s.turn)).toEqual([2, 2]);
    expect(turns(2).map((s) => s.endsTurn)).toEqual([false, true]);
  });
});

function fixture(): MState {
  const setup = makeStandardSetup('7', 'Switzerland', 'Switzerland', 'A', 'B');
  setup.width = 20; setup.height = 12;
  setup.players = setup.players.slice(0, 2).map((p, i) => ({ ...p, faction: 'Switzerland', team: i, is_human: true }));
  const s = createState(setup);
  for (const relation of s.diplomacy!.relations) relation.state = 'War';
  s.units = []; s.cities = []; s.nextUnitId = 0; s.nextCityId = 0;
  s.terrain.fill('Grassland'); s.ruin.fill(0); s.owner.fill(-1); s.tileCity.fill(-1);
  s.knownOwner = {}; s.knownCities = {}; s.knownRuins = {};
  for (const p of s.players) { p.gold = 1000; p.explored.fill(1); }
  return s;
}

function addCity(s: MState, owner: number, pos: Hex) {
  const c: MCity = { id: 1, owner, founder: owner, name: 'Last Bridge', pos, population: 3, food: 0, prod: 0, producing: null, bought: null,
    buildings: [], hp: 200, grownTiles: [], tilesClaimed: 0, isCapital: true, foundedTurn: 1, resistance: 0 };
  s.cities.push(c); s.nextCityId = 1; s.owner[idxOf(s, pos)] = owner; s.tileCity[idxOf(s, pos)] = c.id;
}

describe('previewRoute: the screenshot case and its warnings', () => {
  const c = fromOffset(8, 5);
  const bridge = (d: number) => {
    const m = neighbors(c)[d], f = neighbors(c)[(d + 1) % 6];
    return { m, f, r: { q: m.q + f.q - c.q, r: m.r + f.r - c.r } };
  };

  it.each([0, 1, 2, 3, 4, 5])('an attack never routes through a friend attacking the same city (rotation %s)', (d) => {
    const { m, f, r } = bridge(d);
    const s = fixture(); addCity(s, 1, c);
    const friend = spawnUnit(s, 0, 'Spearman', m, false), right = spawnUnit(s, 0, 'Spearman', r, false);
    refreshVision(s);
    const view = viewFromFixture(s, 0);
    const orders: Order[] = [{ type: 'MoveUnit', unit: friend.id, to: c }];
    expect(previewRoute(view, r, c, orders, right.id)).toEqual({ path: [f, c], risk: null });
    // A parked (unordered) friend is routed around too, and a fortified one.
    expect(previewRoute(view, r, c, [], right.id)).toEqual({ path: [f, c], risk: null });
  });

  it.each([0, 1, 2, 3, 4, 5])('a friend moving out is a tie-break: the free tile wins (rotation %s)', (d) => {
    const { m, f, r } = bridge(d);
    const s = fixture(); addCity(s, 1, c);
    const friend = spawnUnit(s, 0, 'Spearman', m, false), right = spawnUnit(s, 0, 'Spearman', r, false);
    refreshVision(s);
    const orders: Order[] = [{ type: 'MoveUnit', unit: friend.id, to: neighbors(m)[d] }];
    expect(previewRoute(viewFromFixture(s, 0), r, c, orders, right.id)).toEqual({ path: [f, c], risk: null });
  });

  it('passes through a friend that stays, and warns when it cannot', () => {
    const s = fixture();
    for (let col = 0; col < 20; col++) for (let row = 0; row < 12; row++) if (row !== 4) s.terrain[idxOf(s, fromOffset(col, row))] = 'Mountains';
    const a = spawnUnit(s, 0, 'Spearman', fromOffset(2, 4), false);
    const friend = spawnUnit(s, 0, 'Spearman', fromOffset(3, 4), false);
    refreshVision(s);
    const view = viewFromFixture(s, 0), goal = fromOffset(6, 4);
    // Civ 6: the unit walks through its own, stationary friend and lands beyond it. No warning.
    const parked = previewRoute(view, a.pos, goal, [], a.id);
    expect(parked.path?.[0]).toEqual(fromOffset(3, 4));
    expect(parked.risk).toBeNull();
    expect(parked.crossed).toEqual([fromOffset(3, 4)]);
    // A friend that moves out is still a risk: it must move first.
    const moving = previewRoute(view, a.pos, goal, [{ type: 'MoveUnit', unit: friend.id, to: fromOffset(9, 4) }], a.id);
    expect(moving.risk).toMatchObject({ hex: fromOffset(3, 4), stays: false });
    expect(routeRiskText(view, moving.risk!)).toBe('Path may be blocked by your Spearman: it must move out first.');
    // Ending the move on a friend that stays is refused: the unit stops beside it.
    const onto = previewRoute(view, a.pos, fromOffset(3, 4), [], a.id);
    expect(onto.risk).toMatchObject({ hex: fromOffset(3, 4), stays: true, destination: true });
    expect(routeRiskText(view, onto.risk!)).toBe("Your Spearman stays on that tile: this unit can't end its move on it and stops beside it.");
    // Two friends in a row cost more to cross than a Soldier has moves: it waits behind them.
    spawnUnit(s, 0, 'Spearman', fromOffset(4, 4), false);
    refreshVision(s);
    const jam = previewRoute(viewFromFixture(s, 0), a.pos, goal, [], a.id);
    expect(jam.risk).toMatchObject({ hex: fromOffset(3, 4), stays: true });
    expect(routeRiskText(view, jam.risk!)).toBe('Path blocked by your Spearman, which stays put this turn: this unit waits behind it.');
  });

  it('follows an earlier Spearman through a reserved mountain pass', () => {
    const s = fixture();
    s.terrain.fill('Mountains');
    for (let col = 1; col <= 10; col++) for (const row of [4, 8]) s.terrain[idxOf(s, fromOffset(col, row))] = 'Grassland';
    for (let row = 5; row <= 7; row++) for (const col of [1, 10]) s.terrain[idxOf(s, fromOffset(col, row))] = 'Grassland';
    s.terrain[idxOf(s, fromOffset(4, 3))] = 'Grassland';
    const lead = spawnUnit(s, 0, 'Spearman', fromOffset(4, 3), false);
    const follower = spawnUnit(s, 0, 'Spearman', fromOffset(4, 4), false);
    refreshVision(s);
    const view = viewFromFixture(s, 0);
    const pass = fromOffset(5, 4);
    const preview = previewRoute(view, follower.pos, fromOffset(8, 4),
      [{ type: 'MoveUnit', unit: lead.id, to: fromOffset(9, 4) }], follower.id);
    expect(preview.path?.[0]).toEqual(pass);
    expect(preview.path?.every((h) => h.r === pass.r)).toBe(true);
    expect(preview.risk).toMatchObject({ hex: pass, unit: { id: lead.id }, reserved: true });
    expect(preview.risk?.detour).toBeUndefined();
    expect(routeRiskText(view, preview.risk!)).toBe('Moves through this tile right behind your Spearman.');
  });

  it('shows the risk of an already confirmed route through a friend', () => {
    const { m, r } = bridge(0);
    const s = fixture(); addCity(s, 1, c);
    const friend = spawnUnit(s, 0, 'Spearman', m, false), right = spawnUnit(s, 0, 'Spearman', r, false);
    refreshVision(s);
    const orders: Order[] = [{ type: 'MoveUnit', unit: friend.id, to: c }, { type: 'MoveUnitPath', unit: right.id, to: c, path: [m, c] }];
    expect(previewRoute(viewFromFixture(s, 0), r, c, orders, right.id).risk).toMatchObject({ hex: m, stays: true });
  });
});
