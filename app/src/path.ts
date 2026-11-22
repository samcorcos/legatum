// Client-side A* for the move preview. Mirrors crates/core/src/path.rs (ties on hex order) but
// only walks tiles this player has explored; the core pathfinds for real every turn.
import type { Hex, Order, PlayerView, UnitView } from './types';
import { distance, hexAt, hexEq, neighbors, tileIndex } from './hex';
import { atWar, borderSteps, canEnter, neutralBlocker, relationWith } from './diplomacy';
import { RAILWAY_MOVES, ROAD_MOVES, itemName, tileMoveCost } from './data';
import { enemyReach } from './enemyReach';

export type CostFn = (h: Hex) => number | null;

/** Core `path::ROAD_SCALE`. */
const ROAD_SCALE = 256;

/** Cheapest path from `from` to `to`, excluding `from`. null if unreachable. */
export function findPath(
  width: number,
  height: number,
  from: Hex,
  to: Hex,
  cost: CostFn,
  canStep: (a: Hex, b: Hex) => boolean = () => true,
  roads?: (h: Hex) => boolean,
): Hex[] | null {
  if (from.q === to.q && from.r === to.r) return [];
  const goal = tileIndex(to, width, height);
  const start = tileIndex(from, width, height);
  if (goal < 0 || start < 0 || cost(to) === null) return null;

  // Core `path::find_path_by_step` on a map with roads: costs scale and a road tile takes one off,
  // so equal-cost routes prefer roads; the heuristic scales with them.
  const scale = roads ? ROAD_SCALE : 1;
  const hscale = roads ? ROAD_SCALE - 1 : 1;
  const n = width * height;
  const g = new Int32Array(n).fill(0x7fffffff);
  const prev = new Int32Array(n).fill(-1);
  // Binary heap of [f, g, q, r] ordered like Rust's Reverse((f, g, hex)).
  const heap: [number, number, number, number][] = [];
  const less = (a: [number, number, number, number], b: [number, number, number, number]) =>
    a[0] - b[0] || a[1] - b[1] || a[2] - b[2] || a[3] - b[3];
  const push = (x: [number, number, number, number]) => {
    heap.push(x);
    let i = heap.length - 1;
    while (i > 0) {
      const p = (i - 1) >> 1;
      if (less(heap[i], heap[p]) >= 0) break;
      [heap[i], heap[p]] = [heap[p], heap[i]];
      i = p;
    }
  };
  const pop = () => {
    const top = heap[0];
    const last = heap.pop()!;
    if (heap.length) {
      heap[0] = last;
      let i = 0;
      for (;;) {
        const l = 2 * i + 1;
        const r = l + 1;
        let m = i;
        if (l < heap.length && less(heap[l], heap[m]) < 0) m = l;
        if (r < heap.length && less(heap[r], heap[m]) < 0) m = r;
        if (m === i) break;
        [heap[i], heap[m]] = [heap[m], heap[i]];
        i = m;
      }
    }
    return top;
  };

  g[start] = 0;
  push([distance(from, to) * hscale, 0, from.q, from.r]);
  while (heap.length) {
    const [, curG, q, r] = pop();
    const cur = { q, r };
    const ci = tileIndex(cur, width, height);
    if (ci === goal) {
      const path: Hex[] = [to];
      let i = goal;
      while (prev[i] !== start) {
        i = prev[i];
        path.push(hexAt(i, width));
      }
      return path.reverse();
    }
    if (curG > g[ci]) continue;
    for (const nb of neighbors(cur)) {
      const ni = tileIndex(nb, width, height);
      if (ni < 0) continue;
      const step = cost(nb);
      if (step === null || !canStep(cur, nb)) continue;
      const ng = curG + step * scale - (roads && roads(nb) ? 1 : 0);
      if (ng < g[ni]) {
        g[ni] = ng;
        prev[ni] = ci;
        push([ng + distance(nb, to) * hscale, ng, nb.q, nb.r]);
      }
    }
  }
  return null;
}

/** Land movement over explored tiles of a view. */
function viewCost(view: PlayerView): CostFn {
  return (h) => {
    const i = tileIndex(h, view.width, view.height);
    if (i < 0) return null;
    const t = view.tiles[i];
    if (!t.explored || neutralBlocker(view, h)) return null;
    return tileMoveCost(t.terrain, t.road);
  };
}

/** Road predicate for `findPath`: only a view with roads pays for the scaled search. */
function viewRoads(view: PlayerView): ((h: Hex) => boolean) | undefined {
  if (!view.tiles.some((t) => t.road)) return undefined;
  return (h) => !!view.tiles[tileIndex(h, view.width, view.height)]?.road;
}

/**
 * Core `path::crossing_end`: when `path` starts on a crossable tile (a friend of the mover's class
 * that stays put), the index of the first tile after the run of crossable tiles. Null when the path
 * does not start on one, or the run reaches the goal (a unit never ends a move on a friend).
 */
function crossingEnd(path: Hex[], crossable: (h: Hex) => boolean): number | null {
  let run = 0;
  while (run < path.length && crossable(path[run])) run++;
  return run > 0 && run < path.length ? run : null;
}

/** Core `path::hop`: where a route that starts across friends that stay lands, and what the whole move costs. */
export function hop(path: Hex[], cost: CostFn, crossable: (h: Hex) => boolean): { landing: Hex; total: number } | null {
  const end = crossingEnd(path, crossable);
  if (end === null) return null;
  let total = 0;
  for (const h of path.slice(0, end + 1)) {
    const c = cost(h);
    if (c === null) return null;
    total += c;
  }
  return { landing: path[end], total };
}

/**
 * Core `path::crossing_faults`: the first tile of every crossing `path` cannot make: a run of
 * friends with no landing (it reaches the goal), a landing on a blocked goal (an attack cannot start
 * from two tiles away), or a hop dearer than the unit's full moves. A friend that is the goal on its
 * own is no crossing: the unit waits beside it.
 */
export function crossingFaults(path: Hex[], cost: CostFn, blocked: (h: Hex) => boolean, crossable: (h: Hex) => boolean, maxMoves: number, from?: Hex, roads?: (h: Hex) => boolean, roadBonus = ROAD_MOVES): Hex[] {
  const faults: Hex[] = [];
  const last = path.length - 1;
  let i = 0;
  while (i < path.length) {
    if (!crossable(path[i]) || i === last) { i++; continue; }
    const k = crossingEnd(path.slice(i), crossable);
    if (k === null) { faults.push(path[i]); break; }
    const end = i + k;
    const total = path.slice(i, end + 1).reduce((sum, h) => sum + (cost(h) ?? 1), 0);
    const start = i === 0 ? from : path[i - 1];
    if ((end === last && blocked(path[end])) || total > maxMoves + (start && roads?.(start) ? roadBonus : 0)) faults.push(path[i]);
    i = end + 1;
  }
  return faults;
}

/**
 * Mirror core `path::route_around_friends`, the one costing for friendly occupancy: never
 * pass a `blocked` tile (the goal is always allowed; `blocked` answers for the goal too). A
 * `passable` tile holds a friend that stays put: it is crossed in one hop (core `hop`) and never
 * ended on, unless the crossing cannot be made (`crossingFaults`), when its first tile is a wall
 * for that search. A `waiting` tile (a passable friend that clears on a later turn) with such a
 * crossing is walked round only when that takes fewer turns than waiting behind it. Prefer a route
 * that avoids every `risky` tile (a friend with a move out) whenever it arrives in as few turns;
 * a `reserved` first step can be claimed by an earlier friend this round, so it may wait one
 * movement round. It is only routed around if that route arrives in fewer turns;
 * with no route around the blocked tiles, `fallback` returns the plain route (core waits in the
 * choke).
 */
export function routeAroundFriends(
  width: number,
  height: number,
  from: Hex,
  to: Hex,
  cost: CostFn,
  canStep: (a: Hex, b: Hex) => boolean,
  blocked: (h: Hex) => boolean,
  passable: (h: Hex) => boolean,
  waiting: (h: Hex) => boolean,
  risky: (h: Hex) => boolean,
  movesLeft: number,
  maxMoves: number,
  fallback = true,
  roads?: (h: Hex) => boolean,
  roadBonus = ROAD_MOVES,
  reserved: (h: Hex) => boolean = () => false,
): Hex[] | null {
  let pruned = false;
  const around = (cross: boolean, avoidRisky: boolean, avoidReserved = false): { path: Hex[]; waits: boolean } | null => {
    const walls = new Set<number>();
    for (let tries = 0; tries < 6; tries++) {
      const path = findPath(width, height, from, to, (h) => {
        const avoid = !hexEq(h, to) && (blocked(h) || (!cross && passable(h)) || walls.has(tileIndex(h, width, height)) || (avoidRisky && risky(h)) || (avoidReserved && reserved(h)));
        if (avoid) { pruned = true; return null; }
        return cost(h);
      }, canStep, roads);
      if (!path) return null;
      const faults = crossingFaults(path, cost, blocked, passable, maxMoves, from, roads, roadBonus);
      const first = faults.find((f) => !waiting(f));
      if (!first) return { path, waits: faults.length > 0 };
      walls.add(tileIndex(first, width, height));
    }
    return null;
  };
  const turns = (p: Hex[], waits: boolean) => pathTurns(p, cost, maxMoves, movesLeft, from, roads, roadBonus, waits ? undefined : passable).at(-1)?.turn ?? 1;
  let cross = true;
  let found = around(true, false);
  if (!found) { found = around(false, false); cross = false; }
  if (cross && found && found.path.slice(0, -1).some(passable)) {
    const round = around(false, false);
    if (round && (found.waits ? turns(round.path, false) <= turns(found.path, true) : turns(round.path, false) < turns(found.path, false))) { found = { path: round.path, waits: false }; cross = false; }
  }
  if (!found) return fallback && pruned ? findPath(width, height, from, to, cost, canStep, roads) : null;
  if (found.path.slice(0, -1).some(reserved)) {
    const free = around(cross, false, true);
    if (free && turns(free.path, free.waits) < turns(found.path, found.waits)) found = free;
  }
  if (!found.path.slice(0, -1).some(risky)) return found.path;
  const free = around(cross, true);
  return free && turns(free.path, free.waits) <= turns(found.path, found.waits) ? free.path : found.path;
}

/** Why the previewed route may halt this turn: a friend of the same class on a tile it enters. */
export interface RouteRisk {
  /** The friend's tile on the route. */
  hex: Hex;
  unit: UnitView;
  /** True when the friend stays put (no order, or attacking from where it stands). */
  stays: boolean;
  /** The friend is off the chosen route; its tile forced a longer route. */
  detour?: boolean;
  /** An earlier unit declared this tile first, rather than occupying it now. */
  reserved?: boolean;
  /** The earlier unit will still hold its claimed tile this turn. */
  persistent?: boolean;
  /** The friend holds the destination and stays: a unit never ends a move on a friend. */
  destination?: boolean;
}

/** The route warning, from the viewer's side: "Path may be blocked by your Soldier". */
export function routeRiskText(view: PlayerView, risk: RouteRisk): string {
  const owner = view.players?.find((p) => p.id === risk.unit.owner);
  const name = itemName({ kind: 'Unit', unit: risk.unit.kind }, owner?.faction);
  const whose = risk.unit.owner === view.me ? 'your' : `${owner?.name ?? 'an ally'}'s`;
  const Whose = `${whose[0].toUpperCase()}${whose.slice(1)}`;
  if (risk.destination) return `${Whose} ${name} stays on that tile: this unit can't end its move on it and stops beside it.`;
  if (risk.reserved && risk.persistent && !risk.detour) return `Path blocked by ${whose} ${name}, which stops on this tile this turn: this unit waits behind it.`;
  if (risk.reserved) return risk.detour
    ? `${Whose} ${name} is using the direct path this turn, so this unit takes another route.`
    : `${FOLLOW_NOTE} ${whose} ${name}.`;
  if (risk.detour) return `Direct path blocked by ${whose} ${name}, which stays put this turn: taking a longer route.`;
  return risk.stays
    ? `Path blocked by ${whose} ${name}, which stays put this turn: this unit waits behind it.`
    : `Path may be blocked by ${whose} ${name}: it must move out first.`;
}

const FOLLOW_NOTE = 'Moves through this tile right behind';
/** Following a friend through a tile is information, not a warning: it is drawn in neutral ink. */
export const isCalmRouteNote = (text: string | null | undefined): boolean => !!text && text.startsWith(FOLLOW_NOTE) && !text.includes('. ');

/** Known enemies can strike the unit on this turn's route: on its first step ("leaving") or later. */
export type RouteDanger = 'leaving' | 'way';

export interface RoutePreview {
  path: Hex[] | null;
  risk: RouteRisk | null;
  /** Friends that stay put on the route, passed through in one hop (core `path::hop`): no turn ends on them. */
  crossed?: Hex[];
  /** Set only when the caller asked for `avoidEnemyReach` and no route within the same turns is clear of it. */
  danger?: RouteDanger;
}

/** The warning for a route that known enemies can catch, from the viewer's side. */
export function routeDangerText(danger: RouteDanger): string {
  return danger === 'leaving' ? 'Enemies can catch this unit leaving.' : 'Enemies can catch this unit on the way.';
}

export interface RouteOptions {
  /**
   * Plan like a player would: among routes that take the same number of turns, prefer steps that
   * known enemies cannot strike (`enemyReach.ts`), and report `danger` when none exists. Off by
   * default so the AI's routing is unchanged.
   */
  avoidEnemyReach?: boolean;
}

export function previewPath(view: PlayerView, from: Hex, to: Hex, orders: Order[] = [], unitId?: number, options?: RouteOptions): Hex[] | null {
  return previewRoute(view, from, to, orders, unitId, options).path;
}

/** Mirror core `attacking_in_place`: a unit whose destination is an adjacent enemy stays to attack. */
function attacksInPlace(view: PlayerView, u: Pick<UnitView, 'pos' | 'military'>, target: Hex): boolean {
  if (distance(u.pos, target) !== 1) return false;
  const enemy = (owner: number) => atWar(view, owner);
  return (view.cities ?? []).some((c) => hexEq(c.pos, target) && enemy(c.owner))
    || (view.units ?? []).some((v) => hexEq(v.pos, target) && enemy(v.owner) && (v.military || !u.military));
}

/**
 * The route a move order will be confirmed with, and what may stop it this turn. Planning is
 * for the coming turn (full moves). Friends that stay put are passed through, never ended on,
 * like core's resolution does; friends moving out this turn are a tie-break (`routeAroundFriends`).
 */
export function previewRoute(view: PlayerView, from: Hex, to: Hex, orders: Order[] = [], unitId?: number, options: RouteOptions = {}): RoutePreview {
  const terrain = viewCost(view);
  const units = view.units ?? [];
  const mover = units.find((u) => u.owner === view.me && (unitId === undefined ? hexEq(u.pos, from) : u.id === unitId));
  const kind = mover?.kind ?? 'Soldier';
  const destinationCity = (view.cities ?? []).find((city) => hexEq(city.pos, to));
  if (mover && destinationCity && !canEnter(view, kind, destinationCity.owner, true)) return { path: null, risk: null };
  if (mover && units.some((other) => other.id !== mover.id && other.military === mover.military && hexEq(other.pos, to)
    && relationWith(view, view.players.find((p) => p.id === other.owner)?.team ?? -1)?.state === 'Alliance')) return { path: null, risk: null };
  const roads = viewRoads(view);
  const find = (a: Hex, b: Hex, cost: CostFn) => findPath(view.width, view.height, a, b, cost, borderSteps(view, kind, a), roads);
  if (!mover) return { path: find(from, to, terrain), risk: null };
  const pending = (id: number) => orders.find((o) => 'unit' in o && o.unit === id);
  const ownOrder = pending(mover.id);

  const reserved = new Set<number>();
  const persistent = new Set<number>();
  const reservedBy = new Map<number, UnitView>();
  // Tiles the route may not enter: enemies and units of other teams. Friends that stay are passable.
  const parked = new Set<number>();
  const hostiles = new Set<number>();
  // Friends of the mover's class by tile, and whether each stays put this turn.
  const friends = new Map<number, { unit: UnitView; stays: boolean }>();
  const index = (h: Hex) => tileIndex(h, view.width, view.height);
  const enemy = (owner: number) => atWar(view, owner);
  const team = (owner: number) => view.players?.find((p) => p.id === owner)?.team;
  for (const u of [...units].sort((a, b) => a.id - b.id)) {
    if (u.id === mover.id) continue;
    if (enemy(u.owner)) {
      parked.add(index(u.pos));
      hostiles.add(index(u.pos));
      continue;
    }
    if (u.military !== mover.military) continue;
    const order = pending(u.id);
    const autoPath = order?.type === 'SetAutomation' ? order.path : undefined;
    const target = order?.type === 'MoveUnit' || order?.type === 'MoveUnitPath' ? order.to : autoPath?.at(-1) ?? (order ? null : u.goto);
    const friend = team(u.owner) === view.team || (team(u.owner) !== undefined && view.diplomacy?.relations?.some((r) => ((r.a === view.team && r.b === team(u.owner)) || (r.b === view.team && r.a === team(u.owner))) && r.state === 'Alliance'));
    if (u.owner !== view.me || !target) {
      if (friend) friends.set(index(u.pos), { unit: u, stays: true });
      else parked.add(index(u.pos));
      continue;
    }
    const route = order?.type === 'MoveUnitPath' ? order.path : autoPath ?? (!order && u.path?.length ? u.path : find(u.pos, target, terrain));
    if (!route?.length || attacksInPlace(view, u, target)) {
      friends.set(index(u.pos), { unit: u, stays: true });
      continue;
    }
    friends.set(index(u.pos), { unit: u, stays: false });
    // Core reserves only already declared steps in the current movement round. A later
    // unit's route, and even an earlier unit's second step, cannot preempt this first step.
    if (u.id < mover.id && !reservedBy.has(index(route[0]))) {
      const first = route[0];
      const available = u.max_moves + (roads?.(u.pos) ? roadMoves(view) : 0);
      const transit = !hexEq(first, target) && available > (terrain(first) ?? available)
        && !attacksInPlace(view, { pos: first, military: u.military }, target);
      (transit ? reserved : persistent).add(index(first));
      reservedBy.set(index(first), u);
    }
  }
  for (const city of view.cities) {
    if (enemy(city.owner)) { parked.add(index(city.pos)); hostiles.add(index(city.pos)); }
    else if (team(city.owner) !== view.team && relationWith(view, team(city.owner) ?? -1)?.state !== 'Alliance') {
      parked.add(index(city.pos));
      hostiles.add(index(city.pos));
    }
  }
  const risky = (h: Hex) => friends.get(index(h))?.stays === false;
  // A friend that stays this turn: crossed in one hop, never ended on (core `path::hop`).
  const stationary = (h: Hex) => friends.get(index(h))?.stays === true;
  // Planning is for the coming turn: full moves, plus the road bonus for a unit that starts it on a road.
  const bonus = roadMoves(view);
  const firstMoves = mover.max_moves + (roads?.(from) ? bonus : 0);
  /** The stationary friends `path` passes through in a hop it can make (core `crossing_faults`). */
  const crossedBy = (path: Hex[]): Hex[] => {
    const faulty = new Set<number>();
    for (const f of crossingFaults(path, terrain, (h) => parked.has(index(h)), stationary, mover.max_moves, from, roads, bonus)) {
      for (let k = path.findIndex((h) => hexEq(h, f)); k < path.length && stationary(path[k]); k++) faulty.add(index(path[k]));
    }
    return path.slice(0, -1).filter((h) => stationary(h) && !faulty.has(index(h)));
  };
  const risk = (path: Hex[] | null): RoutePreview => {
    if (!path) return { path, risk: null };
    const crossed = crossedBy(path);
    const crossing = (h: Hex) => crossed.some((c) => hexEq(c, h));
    const through = crossed.length ? { crossed } : {};
    // Only this turn's steps are predictable: by later turns the friend has new orders.
    for (const step of pathTurns(path, terrain, mover.max_moves, firstMoves, from, roads, bonus, crossing)) {
      if (step.turn > 1 || hexEq(step.hex, to)) break;
      const f = friends.get(index(step.hex));
      if (f && !(f.stays && crossing(step.hex))) return { path, risk: { hex: step.hex, unit: f.unit, stays: f.stays }, ...through };
      const claimant = reservedBy.get(index(step.hex));
      if (claimant && distance(from, step.hex) === 1) return { path, risk: { hex: step.hex, unit: claimant, stays: false, reserved: true, ...(persistent.has(index(step.hex)) ? { persistent: true } : {}) }, ...through };
    }
    const atDestination = friends.get(index(to));
    if (atDestination?.stays) return { path, risk: { hex: to, unit: atDestination.unit, stays: true, destination: true }, ...through };
    const direct = find(from, to, (h) => !hexEq(h, to) && hostiles.has(index(h)) ? null : terrain(h));
    const cost = (p: Hex[]) => p.reduce((sum, h) => sum + (terrain(h) ?? 1), 0);
    if (direct && cost(direct) < cost(path)) {
      for (const h of direct.slice(0, -1)) {
        const f = friends.get(index(h));
        if (f?.stays) return { path, risk: { hex: h, unit: f.unit, stays: true, detour: true }, ...through };
        const unit = reservedBy.get(index(h));
        if (unit) return { path, risk: { hex: h, unit, stays: false, detour: true, reserved: true, ...(persistent.has(index(h)) ? { persistent: true } : {}) }, ...through };
      }
    }
    return { path, risk: null, ...through };
  };
  // Showing an already confirmed order must show the route that will actually be submitted.
  const route = (avoidPending: boolean, avoid?: Set<number>) => routeAroundFriends(view.width, view.height, from, to, terrain, borderSteps(view, kind, from),
    (h) => parked.has(index(h)) || (avoid?.has(index(h)) ?? false) || (avoidPending && distance(from, h) === 1 && persistent.has(index(h))), stationary, () => false, risky, firstMoves, mover.max_moves, false, roads, bonus,
    (h) => avoidPending && distance(from, h) === 1 && reserved.has(index(h)));
  // A single-hex choke has no detour: keep the route, and let core sequence own traffic.
  const plan = (avoid?: Set<number>) => route(true, avoid) ?? route(false, avoid);
  const attacking = hostiles.has(index(to)) || attacksInPlace(view, mover, to);
  const guard = (routed: RoutePreview, replan: boolean): RoutePreview => {
    if (!options.avoidEnemyReach || !routed.path || attacking) return routed;
    const safe = saferRoute(view, from, to, mover, routed.path, replan ? (avoid) => plan(avoid) : undefined, stationary);
    if (!safe.path) return routed;
    return { ...(safe.path === routed.path ? routed : risk(safe.path)), ...(safe.danger ? { danger: safe.danger } : {}) };
  };
  if (ownOrder?.type === 'MoveUnitPath' && hexEq(ownOrder.to, to)) return guard(risk(ownOrder.path), false);
  if (!ownOrder && hexEq(mover.goto, to) && mover.path?.length) return guard(risk(mover.path), false);
  return guard(risk(plan() ?? find(from, to, (h) => !hexEq(h, to) && hostiles.has(index(h)) ? null : terrain(h))), true);
}

/**
 * This turn's steps that known enemies can strike (`enemyReach.ts`): the j-th tile is where the unit
 * stands at the start of round j + 1; the turn's last tile is where it stops. The destination is the
 * player's choice, so it is never counted.
 */
function exposedSteps(view: PlayerView, path: Hex[], from: Hex, to: Hex, maxMoves: number, reach: ReturnType<typeof enemyReach>, crossable?: (h: Hex) => boolean): { first: boolean; hexes: Hex[] } {
  const hexes: Hex[] = [];
  let first = false;
  planTurns(view, path, from, maxMoves, crossable).filter((s) => s.turn === 1).forEach((s, i) => {
    if (hexEq(s.hex, to) || !reach.threatened(s.hex, i + 1, s.endsTurn)) return;
    hexes.push(s.hex);
    if (i === 0) first = true;
  });
  return { first, hexes };
}

/**
 * Among routes that take the same number of turns, the one whose first step (then whose other steps
 * this turn) known enemies cannot strike. With no such route the original is kept and `danger`
 * says what is exposed. A confirmed route (no `replan`) is only checked, never replaced.
 */
function saferRoute(view: PlayerView, from: Hex, to: Hex, mover: UnitView, path: Hex[], replan?: (avoid: Set<number>) => Hex[] | null, crossable?: (h: Hex) => boolean): { path: Hex[]; danger?: RouteDanger } {
  const reach = enemyReach(view);
  if (!reach.any) return { path };
  const turnsOf = (p: Hex[]) => planTurns(view, p, from, mover.max_moves, crossable).at(-1)?.turn ?? 1;
  const baseTurns = turnsOf(path);
  const score = (p: Hex[]) => { const e = exposedSteps(view, p, from, to, mover.max_moves, reach, crossable); return { ...e, rank: (e.first ? 1000 : 0) + e.hexes.length }; };
  let best = { path, ...score(path) };
  const avoid = new Set<number>();
  for (let tries = 0; replan && best.hexes.length && tries < 4; tries++) {
    for (const h of best.hexes) avoid.add(tileIndex(h, view.width, view.height));
    const alt = replan(avoid);
    if (!alt || turnsOf(alt) > baseTurns) break;
    const next = { path: alt, ...score(alt) };
    if (next.rank < best.rank) best = next;
    if (!next.hexes.length) break;
  }
  return { path: best.path, ...(best.hexes.length ? { danger: best.first ? 'leaving' as const : 'way' as const } : {}) };
}

export interface PathStep {
  hex: Hex;
  /** 1-based turn in which the unit enters this hex. */
  turn: number;
  /** True for the last hex reached in its turn (where a turn marker goes). */
  endsTurn: boolean;
}

/**
 * Split a path into turns with the core's movement rule: a unit may always take one step at
 * full moves; otherwise it needs moves >= the step cost.
 */
/** The extra moves a unit gets for starting its turn on a road: 1, 2 once the viewer knows Railways (core `reset_turn`). */
export function roadMoves(view: PlayerView): number {
  return view.research?.known?.includes('Railways') ? RAILWAY_MOVES : ROAD_MOVES;
}

export function pathTurns(path: Hex[], cost: CostFn, maxMoves: number, movesLeft = maxMoves, from?: Hex, roads?: (h: Hex) => boolean, roadBonus = ROAD_MOVES, crossable?: (h: Hex) => boolean): PathStep[] {
  const out: PathStep[] = [];
  let turn = 1;
  let moves = movesLeft;
  let at = from;
  for (let i = 0; i < path.length;) {
    // Core `path_turns`: a hop across crossable friends is one atomic move that needs moves >= its whole cost.
    const k = crossable ? crossingEnd(path.slice(i), crossable) : null;
    const end = k === null ? i : i + k;
    const c = path.slice(i, end + 1).reduce((sum, h) => sum + (cost(h) ?? 1), 0);
    const affordable = end === i ? (moves === maxMoves || moves >= c) && moves > 0 : moves >= c;
    if (!affordable) {
      if (out.length) out[out.length - 1].endsTurn = true;
      turn++;
      // A unit that starts its turn on a road has extra moves (core `reset_turn`).
      moves = maxMoves + (at && roads?.(at) ? roadBonus : 0);
    }
    moves = Math.max(0, moves - c);
    for (const h of path.slice(i, end + 1)) out.push({ hex: h, turn, endsTurn: false });
    at = path[end];
    i = end + 1;
  }
  if (out.length) out[out.length - 1].endsTurn = true;
  return out;
}

/** `pathTurns` for a unit planning the coming turn from `from`: full moves, plus the road bonus when it starts on a road. */
export function planTurns(view: PlayerView, path: Hex[], from: Hex, maxMoves: number, crossable?: (h: Hex) => boolean): PathStep[] {
  const roads = viewRoads(view);
  const bonus = roadMoves(view);
  return pathTurns(path, viewCost(view), maxMoves, maxMoves + (roads?.(from) ? bonus : 0), from, roads, bonus, crossable);
}

/** "+1 move" / "+2 moves (Railways)": the extra moves a unit gets for starting a turn on a road, from the same rule the planner uses. */
export function roadBonusText(view: PlayerView): string {
  const n = roadMoves(view);
  return `+${n} move${n === 1 ? '' : 's'}${n > ROAD_MOVES ? ' (Railways)' : ''}`;
}

/**
 * When the route's turn count depends on starting a turn on a road (the bonus saves at least one turn), the note that says so; else null.
 * `steps` is `planTurns` output for a unit at `from` with `maxMoves`.
 */
export function roadBonusNote(view: PlayerView, steps: PathStep[], from: Hex, maxMoves: number): string | null {
  const roads = viewRoads(view);
  if (!roads || !steps.length) return null;
  const without = pathTurns(steps.map((s) => s.hex), viewCost(view), maxMoves, maxMoves, from, roads, 0);
  if ((without.at(-1)?.turn ?? 1) <= (steps.at(-1)?.turn ?? 1)) return null;
  return `Includes ${roadBonusText(view)} from starting on a road.`;
}

/**
 * The preview card's headline for a route: the turns to arrive, and when the route is much
 * longer than the straight line, why it may look wrong (it bends round terrain or units, so
 * some steps move away from the goal on the way).
 */
export function routeTurnsText(steps: PathStep[], from: Hex, to: Hex): string {
  const turns = steps.at(-1)?.turn ?? 1;
  const straight = distance(from, to);
  const bend = steps.length > straight + 2
    ? ` going round terrain or units: ${steps.length} tiles for ${straight} in a straight line, so it may turn away from the goal`
    : '';
  return `${turns} turn path${bend}.`;
}

;
