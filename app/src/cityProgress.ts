// What a city is working toward, read from the core's forecasts (CityDetail). The city panel, the
// map banners and the turn report all read these, so the numbers agree everywhere. Nothing here
// forecasts on its own.
import { sameItem, TERRAIN_NAME, WORK_RADIUS } from './data';
import { bearing } from './empires';
import { distance, tileIndex } from './hex';
import { nileYields } from './resources';
import type { CityDetail, CityView, Hex, PlayerView, ProductionItem, ProductionStall, Yields } from './types';

type GrowthState = 'growing' | 'starving' | 'stagnant' | 'shortage' | 'resisting';

export interface GrowthProgress {
  stored: number;
  needed: number;
  state: GrowthState;
  /** Turns to the next population while growing; turns to losing one while starving. */
  turns: number | null;
  /** What the next population unlocks (e.g. the Settler at 2), if anything. */
  unlocks: ProductionItem[];
}

/** Items gated at exactly this population, from the core's `population_unlocks`. */
function unlocksAt(d: CityDetail, population: number): ProductionItem[] {
  return (d.population_unlocks ?? []).filter((u) => u.population === population).map((u) => u.item);
}

/**
 * `preview` lets the city panel pass its forecast for a not-yet-submitted build (which changes
 * the worked tiles); otherwise the core's `turns_to_grow` and `turns_to_starve` are used.
 */
export function growthProgress(city: CityView, preview?: { turnsToGrow: number | null; surplus: number }): GrowthProgress | null {
  const d = city.detail;
  if (!d) return null;
  const grow = preview ? preview.turnsToGrow : d.turns_to_grow;
  // A not-yet-submitted build changes the worked tiles: the panel previews its surplus with core's
  // growth rule (rules/cities.rs grow_city: a deficit below 0 stored Food costs a population).
  const starve = preview ? preview.surplus < 0 && city.population > 1 ? Math.floor(d.food_stored / -preview.surplus) + 1 : null
    : d.turns_to_starve ?? null;
  const surplus = preview ? preview.surplus : null;
  const state: GrowthState = d.resistance > 0 ? 'resisting'
    : grow !== null ? 'growing'
    : starve !== null ? 'starving'
    : city.population <= 1 && (surplus ?? d.yields.total.food - 2) < 0 ? 'shortage' : 'stagnant';
  return {
    stored: d.food_stored,
    needed: d.growth_threshold,
    state,
    turns: state === 'growing' ? grow : state === 'starving' ? starve : null,
    unlocks: unlocksAt(d, city.population + 1),
  };
}

export interface ClaimSummary {
  tile: Hex;
  /** Worked by the claiming city as soon as it is claimed. */
  worked: boolean;
  /** Another city of this empire that will receive and work an out-of-reach claim. */
  handoffTo: string | null;
  /** Hex distance from the city: the ring the claim fills. */
  ring: number;
  /** "Plains W": the tile's terrain and its bearing from the city. */
  place: string;
  /** What it adds to the claiming city; zero for a claim that passes to another city. */
  yields: Yields;
  /** "claims Plains W": the territory bar's phrase (the ring is in its tooltip, the yields sit under the bar). */
  text: string;
  /** The yields as words, its recipient, or "no yield · blocks rivals". */
  effect: string;
}

const YIELD_NAMES: [keyof Yields, string][] = [['food', 'Food'], ['production', 'Production'], ['gold', 'Gold'], ['science', 'Science']];

/** "+2 Food, +1 Production" for a tile's yields; "no yield" when it has none. */
export function yieldsText(y: Yields): string {
  const parts = YIELD_NAMES.filter(([k]) => y[k] !== 0).map(([k, name]) => `${y[k] > 0 ? '+' : ''}${y[k]} ${name}`);
  return parts.length ? parts.join(', ') : 'no yield';
}

/**
 * Population is territory: each growth claims the city's next tile (`next_claim` on CityDetail), so
 * the growth bar is the territory bar. Names the tile and its ring, and what it adds, or says it
 * passes to another own city within range, or only blocks rivals beyond all work radii.
 */
export function claimSummary(city: CityView, view: PlayerView): ClaimSummary | null {
  const claim = city.detail?.next_claim;
  if (!claim) return null;
  const terrain = view.tiles[tileIndex(claim.tile, view.width, view.height)]?.terrain;
  const place = `${terrain ? TERRAIN_NAME[terrain] : 'Unexplored'} ${bearing(city.pos, claim.tile)}`;
  const ring = distance(city.pos, claim.tile);
  const recipient = !claim.worked ? view.cities
    .filter((other) => other.owner === city.owner && other.id !== city.id)
    .map((other) => ({ city: other, distance: distance(claim.tile, other.pos) }))
    .filter(({ distance: d }) => d <= WORK_RADIUS)
    .sort((a, b) => a.distance - b.distance || a.city.id - b.city.id)[0]?.city : undefined;
  // The core quotes the tile's own yields; Egypt's Nile doubles its Food and drops its Production.
  const tileYields = nileYields(view.players.find((p) => p.id === city.owner)?.faction, claim.yields);
  const yields = claim.worked ? tileYields : { food: 0, production: 0, gold: 0, science: 0 };
  return {
    tile: claim.tile, worked: claim.worked, handoffTo: recipient?.name ?? null, ring, place, yields,
    text: `claims ${place}`,
    effect: claim.worked ? yieldsText(tileYields) : recipient ? `worked by ${recipient.name} after Ready` : 'no yield · blocks rivals',
  };
}

/** "6 worked · 9 owned": the city's tiles at work against the tiles it holds (centre excluded from both). */
export function territoryCounts(d: CityDetail): { worked: number; owned: number } {
  return { worked: d.yields.worked.length, owned: Math.max(0, d.territory.length - 1) };
}

export function productionStallText(stall: ProductionStall | null | undefined): string | null {
  if (!stall) return null;
  switch (stall.kind) {
    case 'Population':
      return `Build stalled: Settler needs population ${stall.required} (${stall.available} after growth next turn). Its progress stays banked; grow the city or choose another build.`;
    case 'SpawnBlocked':
      return 'Build stalled: no free spawn tile in or beside the city. Move a blocking unit to make room.';
  }
}

type ProductionBannerState = 'building' | 'stalled' | 'waiting' | 'idle';

export interface ProductionProgress {
  /** The item to show (producing, else the head of the queue); null only when nothing is queued. */
  item: ProductionItem | null;
  stored: number;
  needed: number;
  state: ProductionBannerState;
  /** Turns to complete, only while actively building and not stalled. */
  turns: number | null;
  /** Why a build can't start, when stalled. */
  stallText: string | null;
}

/**
 * What a city's production banner meter shows: actively building, stalled (can't start, but
 * banks Production), waiting (something is queued but hasn't started), or idle (nothing queued at
 * all - the map banner's attention-grabbing "choose" state).
 */
export function productionProgress(city: CityView): ProductionProgress | null {
  const d = city.detail;
  if (!d) return null;
  if (d.producing) {
    const stallText = productionStallText(d.production_stall);
    return { item: d.producing, stored: d.production_stored, needed: d.production_cost ?? 0, state: stallText ? 'stalled' : 'building', turns: stallText ? null : d.turns_to_complete, stallText };
  }
  const queued = d.production_queue?.[0] ?? null;
  if (queued) {
    const needed = d.options.find((o) => sameItem(o.item, queued))?.cost ?? d.idle_production_cap;
    return { item: queued, stored: d.production_stored, needed, state: 'waiting', turns: null, stallText: null };
  }
  return { item: null, stored: d.production_stored, needed: d.idle_production_cap, state: 'idle', turns: null, stallText: null };
}
