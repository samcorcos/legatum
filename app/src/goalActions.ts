// The one-time goal actions (the Swarm (the Aztecs' New Fire), Japan's Sakoku vow, the Athens' broadcast, the Celts' Call the
// Wild, Sparta's retirement of a veteran) and a received broadcast's free-technology
// picks, as the UI reads them. Rules live in the core (`rules::actions`); this is only what the screens need around them: the
// pending order, the preview rows and the picker's selection rules.
import { TECH_ERA } from './data';
import { distance as hexDistance } from './hex';
import type { CityView, GameAction, Hex, Order, PlayerView, ResearchView, RetireVeteranView, TechId, UnitId, UnitView } from './types';

type Kind = GameAction['action'];

/** This turn's pending order for `kind` (the player has ordered it and may still undo it). */
export const pendingAction = (orders: readonly Order[], kind: Kind): Order | undefined =>
  orders.find((o) => o.type === 'Action' && o.action.action === kind);

/** The same action: the kind, and for a unit action (retiring) the unit. */
const sameAction = (a: GameAction, b: GameAction): boolean =>
  a.action === b.action && (a.action !== 'RetireVeteran' || (b.action === 'RetireVeteran' && a.unit === b.unit));

/** Order `action`, replacing any pending order of its kind (and unit). */
export const setAction = (orders: Order[], action: GameAction): Order[] =>
  [...orders.filter((o) => !(o.type === 'Action' && sameAction(o.action, action))), { type: 'Action', action }];

/** Drop the pending order of `kind`. */
export const clearAction = (orders: Order[], kind: Kind): Order[] =>
  orders.filter((o) => !(o.type === 'Action' && o.action.action === kind));

/** This turn's pending retirement of `unit`, if it has been ordered. */
export const pendingRetire = (orders: readonly Order[], unit: UnitId): Order | undefined =>
  orders.find((o) => o.type === 'Action' && o.action.action === 'RetireVeteran' && o.action.unit === unit);

/** Drop the pending retirement of `unit`. */
export const clearRetire = (orders: Order[], unit: UnitId): Order[] =>
  orders.filter((o) => !(o.type === 'Action' && o.action.action === 'RetireVeteran' && o.action.unit === unit));

/** What the map marks while a goal action is weighed: the New Fire's shrinking cities and released tiles, the warbands's cities and camps, the cities where a veteran may retire. */
export interface GoalPreview {
  cities: Hex[];
  released: Hex[];
  /** Where Call the warbands would raise its camps. */
  camps: Hex[];
  /** Valid places for a one-time unit action (Agoge cities for a retiring veteran). */
  places: Hex[];
}
const NO_PREVIEW: GoalPreview = { cities: [], released: [], camps: [], places: [] };

export interface SwarmRow {
  city: CityView;
  lost: number;
  /** Population after the sacrifice. */
  population: number;
  /** The territory tiles it would release (core `released_tiles`). */
  tiles: Hex[];
}

/** the New Fire's per-city preview from the core's view, with each city named. */
export function swarmRows(view: PlayerView): SwarmRow[] {
  const swarm = view.actions?.swarm;
  if (!swarm) return [];
  return swarm.cities.flatMap((c) => {
    const city = view.cities.find((x) => x.id === c.city);
    return city ? [{ city, lost: c.lost, population: c.population, tiles: c.tiles }] : [];
  });
}

/** Everything the map marks while the New Fire is being weighed: the cities that shrink and the tiles they release. */
export function swarmPreview(view: PlayerView): GoalPreview | null {
  const rows = swarmRows(view);
  return rows.length ? { ...NO_PREVIEW, cities: rows.map((r) => r.city.pos), released: rows.flatMap((r) => r.tiles) } : null;
}

export interface WildRow {
  city: CityView;
  /** Where its camp would stand. */
  at: Hex;
  /** Distance from the city, in hexes. */
  hexes: number;
}

/** Call the warbands's per-city preview from the core's view, with each city named. */
export function wildRows(view: PlayerView): WildRow[] {
  return (view.actions?.wild?.sites ?? []).flatMap((s) => {
    const city = view.cities.find((c) => c.id === s.city);
    return city ? [{ city, at: s.at, hexes: hexDistance(city.pos, s.at) }] : [];
  });
}

/** The cities the call touches and the camps it would raise. */
export function wildPreview(view: PlayerView): GoalPreview | null {
  const rows = wildRows(view);
  return rows.length ? { ...NO_PREVIEW, cities: rows.map((r) => r.city.pos), camps: rows.map((r) => r.at) } : null;
}

/** The viewer's veteran entry (at the retire level or higher), if `unit` is one. */
export const retireEntry = (view: PlayerView, unit: UnitId): RetireVeteranView | undefined =>
  view.actions?.retire?.veterans.find((v) => v.unit === unit);

/** The cities where a veteran retires (own cities with an Agoge), as the map marks them. */
export function retirePlaces(view: PlayerView): Hex[] {
  const shrines = view.actions?.retire?.shrines ?? [];
  return view.cities.filter((c) => shrines.includes(c.id)).map((c) => c.pos);
}

/** The map marks the valid Agoge cities while a veteran is selected (and not yet standing in one) or being retired. */
export function retirePreview(view: PlayerView, unit: UnitView | null, weighing: boolean): GoalPreview | null {
  const places = retirePlaces(view);
  if (!places.length) return null;
  if (!weighing && !(unit && retireEntry(view, unit.id))) return null;
  return { ...NO_PREVIEW, places };
}

/** Retirement progress before and after the unit now weighed (the confirm states it: "2 of 3"). */
export function retireProgress(view: PlayerView): { retired: number; target: number; after: number } | null {
  const r = view.actions?.retire;
  return r ? { retired: r.retired, target: r.target, after: Math.min(r.target, r.retired + 1) } : null;
}

/** Total population the viewer's cities hold now, and after the New Fire. */
export function swarmTotals(view: PlayerView): { before: number; after: number } {
  const before = view.cities.filter((c) => c.owner === view.me).reduce((n, c) => n + c.population, 0);
  return { before, after: before - swarmRows(view).reduce((n, r) => n + r.lost, 0) };
}

/**
 * What closing the country (Sakoku) takes from research: the technologies the viewer could research now that other empires already
 * know, and the Science the diffusion discount saves on them (the sum of the "Tech diffusion" part of each price).
 */
export function sakokuDiffusion(view: PlayerView): { techs: number; science: number } {
  let techs = 0, science = 0;
  for (const t of view.research.available) {
    const saved = -t.cost.parts.filter(([label]) => label.startsWith('Tech diffusion')).reduce((n, [, amount]) => n + amount, 0);
    if (saved > 0) { techs++; science += saved; }
  }
  return { techs, science };
}

/** How many technologies the picker asks for: the pending grant, or what is left to learn in the current era (free picks come only from it). */
export function freeTechTarget(view: PlayerView): number {
  const free = view.actions?.free_techs;
  if (!free) return 0;
  const unknown = view.research.era_total - view.research.era_known;
  return Math.min(free.pending, unknown);
}

export type PickState = 'known' | 'chosen' | 'open' | 'blocked';

/** A technology in the picker: already known, chosen, pickable now (unknown and in the current era), or still locked in a later era. */
export function pickState(research: ResearchView, chosen: readonly TechId[], tech: TechId): PickState {
  if (research.known.includes(tech)) return 'known';
  if (chosen.includes(tech)) return 'chosen';
  const option = research.catalog.find((o) => o.tech === tech);
  return (option ? option.researchable : TECH_ERA[tech] === research.era) ? 'open' : 'blocked';
}

/** Click a technology: choose it when it is pickable and there is room (`limit`), or unchoose it. */
export function togglePick(research: ResearchView, chosen: readonly TechId[], tech: TechId, limit: number): TechId[] {
  const state = pickState(research, chosen, tech);
  if (state === 'chosen') return chosen.filter((t) => t !== tech);
  if (state === 'open' && chosen.length < limit) return [...chosen, tech];
  return [...chosen];
}
