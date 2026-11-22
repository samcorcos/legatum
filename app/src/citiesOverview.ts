// The Cities overview (user, 2026-10-17: "which cities have the highest population... which has the highest
// production"): one row per city of yours, every number read from the view, the same fields the city's own panel
// shows. Nothing here forecasts: it picks, words and sorts.
import { growthProgress, type GrowthProgress } from './cityProgress';
import { itemName, sameItem } from './data';
import { improvementShortTitle } from './improvements';
import { tileIndex } from './hex';
import { cityStrengthRule } from './format';
import { effectiveProduction, productionQueue } from './orders';
import { cityAutomationFocus } from './automation';
import type { CityBuildFocus, CityView, Faction, Order, PlayerView, ProductionItem, Yields } from './types';

export type CitySortKey = 'name' | 'population' | 'food' | 'production' | 'gold' | 'science' | 'defence' | 'build';
type SortDir = 'asc' | 'desc';
export interface CitySort { key: CitySortKey; dir: SortDir }

/** Production is the question the sheet was asked for, so it opens sorted by it. */
export const DEFAULT_CITY_SORT: CitySort = { key: 'production', dir: 'desc' };

/** The direction a column starts in when first clicked: names and "next to finish" run up, the numbers down. */
const firstDirection = (key: CitySortKey): SortDir => (key === 'name' || key === 'build' ? 'asc' : 'desc');

interface CityBuild {
  item: ProductionItem;
  name: string;
  stored: number;
  cost: number;
  /** Turns left; null while waiting, stalled or with no Production. */
  turns: number | null;
  state: 'building' | 'waiting';
  /** Chosen this turn, not yet played. */
  pending: boolean;
}

export interface CityRow {
  id: number;
  name: string;
  isCapital: boolean;
  population: number;
  growth: GrowthProgress | null;
  /** Short growth text: "3 turns", "Short of Food". */
  growthText: string;
  growthBad: boolean;
  food: number;
  /** Food after the citizens eat (2 per population): what the growth text follows from. */
  foodNet: number;
  production: number;
  gold: number;
  science: number;
  strength: number;
  hp: number;
  maxHp: number;
  /** The wall tier the city has, named for the faction ("Walls", "Castle", "Alpine Redoubt"), or null. */
  fort: string | null;
  /** How the strength is made up (the city panel's own tooltip). */
  strengthRule: string;
  build: CityBuild | null;
  focus?: CityBuildFocus | null;
  /** Nothing chosen: the same test as the "Choose production" decision. */
  idle: boolean;
}

const plural = (n: number) => `${n} turn${n === 1 ? '' : 's'}`;

function growthText(g: GrowthProgress | null): string {
  if (!g) return '—';
  switch (g.state) {
    case 'growing': return g.turns === null ? '—' : plural(g.turns);
    case 'starving': return g.turns === null ? 'Starving' : `Starving: −1 in ${plural(g.turns)}`;
    case 'resisting': return 'Paused';
    case 'shortage': return 'Short of Food';
    default: return 'Not growing';
  }
}

function fortName(city: CityView, faction: Faction | undefined): string | null {
  const building = city.star_fort ? 'StarFort' : city.citadel ? 'Castle' : city.walls ? 'Walls' : null;
  return building ? itemName({ kind: 'Building', building }, faction) : null;
}


/** One row per city of yours, with the pending orders applied the way the city panel applies them. */
export function cityRows(view: PlayerView, orders: Order[]): CityRow[] {
  return view.cities.filter((c) => c.owner === view.me && c.detail).map((city) => {
    const d = city.detail!;
    const focus = cityAutomationFocus(city.id, view, orders);
    const faction = view.players.find((p) => p.id === city.owner)?.faction;
    const queue = productionQueue(orders, city.id, d.producing, d.production_queue);
    const producing = effectiveProduction(orders, city.id, d.producing);
    const pending = orders.some((o) => (o.type === 'SetProduction' || o.type === 'SetProductionQueue') && o.city === city.id);
    // The panel's rule: a build chosen this turn changes the worked tiles, so its preview yields stand in.
    const changed = pending && !(sameItem(producing, d.producing) || (!producing && !d.producing));
    const option = producing ? d.options.find((o) => sameItem(o.item, producing)) : undefined;
    const yields: Yields = (changed ? (producing ? option?.yields ?? d.yields : d.idle_yields ?? d.yields) : d.yields).total;
    const surplus = yields.food - 2 * city.population;
    const growth = growthProgress(city, changed ? {
      turnsToGrow: d.resistance > 0 ? null : d.food_stored + surplus >= d.growth_threshold ? 1 : surplus > 0 ? Math.max(1, Math.ceil((d.growth_threshold - d.food_stored) / surplus)) : null,
      surplus,
    } : undefined);
    const name = (item: ProductionItem) => item.kind === 'Improvement'
      ? improvementShortTitle(item, city.pos, view.tiles[tileIndex(item.tile, view.width, view.height)]?.resource)
      : itemName(item, faction);
    let build: CityBuild | null = null;
    if (producing) {
      const active = !!d.producing || changed;
      const head = queue[0] ?? producing;
      const turns = changed ? (option && option.turns > 0 ? option.turns : null) : d.production_stall ? null : d.turns_to_complete;
      build = {
        item: head, name: name(head), state: active ? 'building' : 'waiting', pending: changed,
        stored: changed ? option?.production_stored ?? 0 : d.production_stored,
        cost: changed ? option?.cost ?? 0 : d.production_cost ?? option?.cost ?? 0,
        turns: active ? turns : null,
      };
    }
    return {
      id: city.id, name: city.name, isCapital: city.is_capital, population: city.population,
      growth, growthText: growthText(growth), growthBad: growth?.state === 'starving' || growth?.state === 'shortage', foodNet: surplus,
      food: yields.food, production: yields.production, gold: yields.gold, science: yields.science,
      strength: city.strength, hp: city.hp, maxHp: city.max_hp, fort: fortName(city, faction), strengthRule: cityStrengthRule(city, faction, (view.players.find((p) => p.id === city.owner)?.era ?? 1) - 1),
      build, focus, idle: !producing && !focus,
    };
  });
}

const number = (row: CityRow, key: CitySortKey): number => {
  switch (key) {
    case 'population': return row.population;
    case 'food': return row.foodNet;
    case 'production': return row.production;
    case 'gold': return row.gold;
    case 'science': return row.science;
    case 'defence': return row.strength;
    // Idle cities need you most, so they come first when ascending; then the nearest to finishing; waiting and stalled last.
    case 'build': return row.idle ? -1 : row.build?.turns ?? Number.MAX_SAFE_INTEGER;
    default: return 0;
  }
};

/** Sorted by one column; ties fall back to name, so the order never jumps between equal rows. */
export function sortRows(rows: CityRow[], sort: CitySort): CityRow[] {
  const sign = sort.dir === 'asc' ? 1 : -1;
  return [...rows].sort((a, b) => (sort.key === 'name' ? sign * a.name.localeCompare(b.name, undefined, { numeric: true }) : sign * (number(a, sort.key) - number(b, sort.key))) || a.name.localeCompare(b.name, undefined, { numeric: true }));
}

/** Clicking a heading: the same column flips direction, a new one starts in its natural direction. */
export function nextSort(current: CitySort, key: CitySortKey): CitySort {
  return current.key === key ? { key, dir: current.dir === 'asc' ? 'desc' : 'asc' } : { key, dir: firstDirection(key) };
}
