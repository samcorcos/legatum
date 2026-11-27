/** Map advice from the Rust core's fogged view and anonymous city-spacing mask. */
import { Game } from './wasm-pkg/legatum.js';
import { minCityGap, terrainYieldsFor, unitClass } from './data';
import { empireOfPlayer } from './empires';
import { engineWork } from './frameTelemetry';
import { tileIndex } from './hex';
import { UNIT_ACTIONS, orderUnit } from './orders';
import type { Faction, Hex, Order, PlayerView, SettlementScore, SettlementSite, SiteBlocker, Terrain, UnitView, Yields } from './types';

export const terrainYields = (faction: Faction, terrain: Terrain): Yields => ({ ...terrainYieldsFor(faction, terrain) });

/** Instance methods on a live `WasmGame` that reuse Rust's cached `PlayerView` (`forecast_view`). */
export interface LiveViewEngine {
  player_settlement_sites?(player: number, unit: number, reserved: string): string;
  player_settlement_site?(player: number, unit: number, q: number, r: number, reserved: string): string;
  player_site_blockers?(player: number): string;
  player_units_can_reach?(player: number, units: string, q: number, r: number): string;
}

interface LiveViewBinding {
  player: number;
  engine: LiveViewEngine;
  isCurrent: () => boolean;
}

const liveViews = new WeakMap<PlayerView, LiveViewBinding>();

/**
 * Associate a `PlayerView` returned by `WasmClient.view(player)` with its live `WasmGame` instance so settlement
 * and reach queries reuse Rust's cached `PlayerView` instead of serializing >1 MB of JSON back across WASM.
 */
export function registerLiveView(view: PlayerView, player: number, engine: LiveViewEngine, isCurrent: () => boolean) {
  liveViews.set(view, { player, engine, isCurrent });
}

function liveEngineFor(view: PlayerView): { player: number; engine: LiveViewEngine } | null {
  const binding = liveViews.get(view);
  return binding && binding.isCurrent() ? { player: binding.player, engine: binding.engine } : null;
}

const siteBlockerTables = new WeakMap<PlayerView, (SiteBlocker | null)[]>();
/**
 * Why a tile is not a legal city site (the core's rule). The engine answers for every tile in one call per view and
 * the table is kept with the view: a per-tile call sent the whole view (over 1 MB late in a game) across for each
 * hovered tile, a stall of 50 ms or more per pointer move with a founder selected. Views are never changed in place.
 */
export function siteBlocker(view: PlayerView, pos: Hex): SiteBlocker | null {
  let table = siteBlockerTables.get(view);
  if (!table) {
    const live = liveEngineFor(view);
    table = engineWork(() => JSON.parse(
      live?.engine.player_site_blockers
        ? live.engine.player_site_blockers(live.player)
        : Game.site_blockers(JSON.stringify(view)),
    ) as (SiteBlocker | null)[]);
    siteBlockerTables.set(view, table);
  }
  return table[tileIndex(pos, view.width, view.height)] ?? null;
}
export const knownLegalSite = (view: PlayerView, pos: Hex): boolean => siteBlocker(view, pos) === null;

/** A short label for Found City. A prior failed attempt may have been a transient clash. */
export function foundCityBlocker(view: PlayerView, pos: Hex): string | null {
  const blocker = siteBlocker(view, pos);
  if (blocker === null || blocker === 'FailedBefore') return null;
  if (blocker === 'CityTooClose') return 'Too close to another city to found one here';
  if (blocker === 'Unexplored' || (typeof blocker === 'object' && ('Water' in blocker || 'Impassable' in blocker)))
    return 'Cities can only be founded on passable land.';
  if (typeof blocker === 'object' && 'Wonder' in blocker) return 'A natural wonder occupies this tile.';
  if (typeof blocker === 'object' && 'Foreign' in blocker)
    return `This tile belongs to ${empireOfPlayer(view, blocker.Foreign)}: found cities on land you or nobody owns.`;
  if (typeof blocker === 'object' && 'TooClose' in blocker) {
    const gap = minCityGap(view.players.find((p) => p.id === view.me)?.faction ?? 'Athens');
    const near = blocker.TooClose;
    // The first Settler is trained in its capital: the refusal also says what to do next (UX review, 2026-11-21).
    return near.distance === 0 ? `${near.city} already stands here. Right-click a tile at least ${gap + 1} tiles away to move there. The Best spot is a good choice.` : `Too close to ${near.city}: cities must be at least ${gap + 1} tiles apart.`;
  }
  return null;
}

/** The same current-state gates as Roman founding; combat can still wound or kill the founder before resolution. */
export const isFounderCandidate = (view: PlayerView, unit: UnitView): boolean => unit.kind === 'Settler'
  || view.players.find((p) => p.id === unit.owner)?.faction === 'Rome' && unitClass(unit.kind) === 'Melee';

export function foundCityUnitBlocker(view: PlayerView, unit: UnitView): string | null {
  if (!isFounderCandidate(view, unit)) return 'Only Settlers or Roman melee veterans found cities.';
  if (unit.kind !== 'Settler') {
    if ((unit.level ?? 0) < 1) return 'Fight until this melee unit earns its first promotion.';
    if (unit.hp < 100) return `Needs full health: ${unit.hp}/100. Heal before founding.`;
  }
  return foundCityBlocker(view, unit.pos);
}

/** Current orders override persistent routes, including a cancelled route or founding in place. */
export function reservedSettlements(view: PlayerView, unit: number, orders: Order[]): Hex[] {
  return view.units.filter((u) => u.owner === view.me && isFounderCandidate(view, u) && u.id !== unit).flatMap((u) => {
    const order = orders.find((o) => UNIT_ACTIONS.has(o.type) && orderUnit(o) === u.id);
    const destination = order?.type === 'MoveUnit' || order?.type === 'MoveUnitPath' ? order.to : order?.type === 'FoundCity' ? u.pos : order ? null : u.goto;
    return destination ? [destination] : [];
  });
}

const settlementScoresCache = new WeakMap<PlayerView, Map<string, SettlementScore[]>>();

export function settlementScores(view: PlayerView, unit: UnitView, reserved: Hex[]): SettlementScore[] {
  const reservedJson = JSON.stringify(reserved);
  const key = `${unit.id}|${reservedJson}`;
  let byKey = settlementScoresCache.get(view);
  if (!byKey) settlementScoresCache.set(view, byKey = new Map());
  const hit = byKey.get(key);
  if (hit) return hit;
  const live = liveEngineFor(view);
  const scores = engineWork(() => JSON.parse(
    live?.engine.player_settlement_sites
      ? live.engine.player_settlement_sites(live.player, unit.id, reservedJson)
      : Game.settlement_sites(JSON.stringify(view), unit.id, reservedJson),
  ) as SettlementScore[]);
  byKey.set(key, scores);
  return scores;
}

const coloniaSiteCache = new WeakMap<PlayerView, boolean | null>();
/**
 * Rome: whether any site this view knows founds the Colonia Victrix for its first founder. It is the Settle lens's own
 * engine call (`settlement_sites`, the `colonia` flag), made at most once per view and remembered, so the dock and the
 * unit card need no per-turn scan in core. Null when there is no founder or the engine is not loaded (callers then
 * keep the plain wording).
 */
export function coloniaSiteKnown(view: PlayerView): boolean | null {
  if (coloniaSiteCache.has(view)) return coloniaSiteCache.get(view)!;
  const id = view.actions?.colonia?.founders[0];
  const unit = id === undefined ? undefined : view.units.find((u) => u.id === id);
  let known: boolean | null = null;
  if (unit) try { known = settlementScores(view, unit, []).some((s) => s.colonia); } catch { known = null; }
  coloniaSiteCache.set(view, known);
  return known;
}

const settlementSiteCache = new WeakMap<PlayerView, Map<string, SettlementSite | null>>();

export function settlementSite(view: PlayerView, unit: UnitView, pos: Hex, reserved: Hex[]): SettlementSite | null {
  const reservedJson = JSON.stringify(reserved);
  const key = `${unit.id}|${pos.q},${pos.r}|${reservedJson}`;
  let byKey = settlementSiteCache.get(view);
  if (!byKey) settlementSiteCache.set(view, byKey = new Map());
  if (byKey.has(key)) return byKey.get(key)!;
  const live = liveEngineFor(view);
  const site = engineWork(() => JSON.parse(
    live?.engine.player_settlement_site
      ? live.engine.player_settlement_site(live.player, unit.id, pos.q, pos.r, reservedJson)
      : Game.settlement_site(JSON.stringify(view), unit.id, pos.q, pos.r, reservedJson),
  ) as SettlementSite | null);
  byKey.set(key, site);
  return site;
}

/** Detailed sites for tests and inspected advice; the live map uses the compact score list. */
export const settlementSites = (view: PlayerView, unit: UnitView, reserved: Hex[] = [], only?: Hex): SettlementSite[] =>
  only ? [engineWork(() => JSON.parse(Game.settlement_site(JSON.stringify(view), unit.id, only.q, only.r, JSON.stringify(reserved))) as SettlementSite | null)].filter((site): site is SettlementSite => site !== null)
    : engineWork(() => JSON.parse(Game.settlement_sites_full(JSON.stringify(view), unit.id, JSON.stringify(reserved))));
const reachCache = new WeakMap<PlayerView, Map<string, Set<number>>>();
/**
 * Which of these known enemies can reach `pos` by moving next turn. One engine call for all of them, remembered per view:
 * asking per enemy sent the whole view to the engine each time, and a late-game battle forecast stalled hover for 100 ms+.
 */
export function unitsCanReach(view: PlayerView, units: readonly UnitView[], pos: Hex): Set<number> {
  if (!units.length) return new Set();
  const ids = units.map((u) => u.id).sort((a, b) => a - b);
  const key = `${pos.q},${pos.r}:${ids.join(',')}`;
  let cache = reachCache.get(view);
  if (!cache) reachCache.set(view, cache = new Map());
  const hit = cache.get(key);
  if (hit) return hit;
  const idsJson = JSON.stringify(ids);
  const live = liveEngineFor(view);
  const reach = new Set<number>(engineWork(() => JSON.parse(
    live?.engine.player_units_can_reach
      ? live.engine.player_units_can_reach(live.player, idsJson, pos.q, pos.r)
      : Game.units_can_reach(JSON.stringify(view), idsJson, pos.q, pos.r),
  )));
  cache.set(key, reach);
  return reach;
}
