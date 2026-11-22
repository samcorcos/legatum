/** Map advice from the Rust core's fogged view and anonymous city-spacing mask. */
import { Game } from './wasm-pkg/legatum.js';
import { minCityGap, terrainYieldsFor, unitClass } from './data';
import { empireOfPlayer } from './empires';
import { UNIT_ACTIONS, orderUnit } from './orders';
import type { Faction, Hex, Order, PlayerView, SettlementScore, SettlementSite, SiteBlocker, Terrain, UnitView, Yields } from './types';

export const terrainYields = (faction: Faction, terrain: Terrain): Yields => ({ ...terrainYieldsFor(faction, terrain) });
export const siteBlocker = (view: PlayerView, pos: Hex): SiteBlocker | null =>
  JSON.parse(Game.site_blocker(JSON.stringify(view), pos.q, pos.r));
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
    if (unit.hp < 100) return `Needs full health: ${unit.hp}/100 HP. Heal before founding.`;
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
export const settlementScores = (view: PlayerView, unit: UnitView, reserved: Hex[]): SettlementScore[] =>
  JSON.parse(Game.settlement_sites(JSON.stringify(view), unit.id, JSON.stringify(reserved)));
export const settlementSite = (view: PlayerView, unit: UnitView, pos: Hex, reserved: Hex[]): SettlementSite | null =>
  JSON.parse(Game.settlement_site(JSON.stringify(view), unit.id, pos.q, pos.r, JSON.stringify(reserved)));
/** Detailed sites for tests and inspected advice; the live map uses the compact score list. */
export const settlementSites = (view: PlayerView, unit: UnitView, reserved: Hex[] = [], only?: Hex): SettlementSite[] =>
  only ? [settlementSite(view, unit, only, reserved)].filter((site): site is SettlementSite => site !== null)
    : JSON.parse(Game.settlement_sites_full(JSON.stringify(view), unit.id, JSON.stringify(reserved)));
export const knownThreats = (view: PlayerView): number[] => JSON.parse(Game.known_threats(JSON.stringify(view)));
export const unitCanReach = (view: PlayerView, unit: UnitView, pos: Hex): boolean =>
  Game.unit_can_reach(JSON.stringify(view), unit.id, pos.q, pos.r);
