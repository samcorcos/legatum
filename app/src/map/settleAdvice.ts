import type { Hex, PlayerView, SettlementSite, SiteBlocker, UnitView } from '../types';
import { WORK_RADIUS, minCityGap, unitStatsFor } from '../data';
import { RESOURCE_INFO, STRATEGIC_SITE_VALUE, isStrategic } from '../resources';
import { WONDER_INFO } from '../wonders';
import { empireOfPlayer } from '../empires';
import { bandRange } from './siteBands';
import { siteBlocker } from '../settlement';

export { SITE_BANDS as SETTLE_BANDS, bandRange } from './siteBands';
export type { SiteBand } from './siteBands';

/** Foreign land costs this many points per tile (core `analyze_sites`: "Foreign borders: −8 per tile"). */
const FOREIGN_TILE_COST = 8;

const tiles = (n: number) => `${n} ${n === 1 ? 'tile' : 'tiles'}`;
const away = (n: number) => n === 0 ? 'on the site' : `${tiles(n)} away`;
const YIELD_WORDS = { food: 'Food', production: 'Production', gold: 'Gold', science: 'Science' } as const;

/** "+18 Food, +12 Production": the nonzero yields, signed, in plain numbers. */
export function yieldPhrase(y: SettlementSite['potential_yields']): string {
  const parts = (['food', 'production', 'gold', 'science'] as const).filter((k) => y[k] !== 0).map((k) => `${y[k] > 0 ? '+' : '−'}${Math.abs(y[k])} ${YIELD_WORDS[k]}`);
  return parts.join(', ');
}

/** The points of a site's quality, one per cause, in the order the hover lists them. They sum to `site.quality`. */
export interface QualityPart { id: 'land' | 'strategic' | 'foreign' | 'wonder'; label: string; points: number }
export function qualityParts(site: SettlementSite, faction?: string): QualityPart[] {
  const y = site.potential_yields;
  // The Nile: Egypt's site numbers already hold double Food and no Production, so its formula names no Production.
  const formula = faction === 'Egypt' ? 'Food ×4 + Gold ×2 (Food doubled by the Nile)' : 'Food ×4 + Production ×3 + Gold ×2';
  const parts: QualityPart[] = [{ id: 'land', label: `Land within ${WORK_RADIUS} tiles, outer ring at half: ${formula}`, points: y.food * 4 + y.production * 3 + y.gold * 2 }];
  const strategic = site.resources.filter((r) => isStrategic(r.resource));
  if (strategic.length) parts.push({ id: 'strategic', label: `${strategic.map((r) => RESOURCE_INFO[r.resource].name).join(' and ')}: ${STRATEGIC_SITE_VALUE} each`, points: strategic.length * STRATEGIC_SITE_VALUE });
  if (site.foreign_tiles) parts.push({ id: 'foreign', label: `${tiles(site.foreign_tiles)} of foreign land: −${FOREIGN_TILE_COST} each`, points: -site.foreign_tiles * FOREIGN_TILE_COST });
  for (const w of site.wonders) parts.push({ id: 'wonder', label: `${WONDER_INFO[w.wonder].name}, ${away(w.distance)}`, points: w.value });
  return parts;
}

/** Where "some" and "plenty" start, in the land's Food and Production within reach (the quality scale itself is core's). */
const FOOD_PLENTY = 20, FOOD_SOME = 10, PRODUCTION_PLENTY = 15, PRODUCTION_SOME = 7;
type Level = 'plenty' | 'some' | 'little';
/** Rough words for the Food and Production of the land within reach: only for the opening sentence, never for a decision. */
const levelOf = (n: number, plenty: number, some: number): Level => n >= plenty ? 'plenty' : n >= some ? 'some' : 'little';
const levelWords = (level: Level, yieldName: string) => level === 'plenty' ? `plenty of ${yieldName}` : level === 'some' ? `some ${yieldName}` : `little ${yieldName}`;

/** "plenty of Food and Production nearby", "plenty of Food but little Production nearby". */
export function landClause(y: SettlementSite['potential_yields'], faction?: string): string {
  // The Nile: Egypt has no Production and its Food is already doubled in the site's numbers, so it hears about Food alone.
  if (faction === 'Egypt') return `${levelWords(levelOf(y.food, FOOD_PLENTY * 2, FOOD_SOME * 2), 'Food')} nearby`;
  const food = levelOf(y.food, FOOD_PLENTY, FOOD_SOME), production = levelOf(y.production, PRODUCTION_PLENTY, PRODUCTION_SOME);
  if (food === production) return food === 'little' ? 'little Food or Production nearby' : `${levelWords(food, 'Food and Production')} nearby`;
  return `${levelWords(food, 'Food')} ${(food === 'little') !== (production === 'little') ? 'but' : 'and'} ${levelWords(production, 'Production')} nearby`;
}

const joinWords = (parts: string[]) => parts.length < 2 ? parts.join('') : `${parts.slice(0, -1).join(', ')} and ${parts[parts.length - 1]}`;
/** One entry per kind of resource, at its nearest deposit, nearest first (three Wheat tiles are one "Wheat 3 tiles away"). */
const nearestFirst = (site: Pick<SettlementSite, 'resources'>) => {
  const nearest = new Map<SettlementSite['resources'][number]['resource'], number>();
  for (const r of site.resources) nearest.set(r.resource, Math.min(r.distance, nearest.get(r.resource) ?? Infinity));
  return [...nearest].map(([resource, distance]) => ({ resource, distance })).sort((a, b) => a.distance - b.distance);
};
/** How many resources the sentence names; the rest go on their own line. */
const NAMED_RESOURCES = 2;

/**
 * The hover's first sentence, in plain words (pillar 5, plain-language guide): "Great spot for a city: plenty of Food and
 * Production nearby, Wheat 3 tiles away and the terrain gives units here +3 defence." The numbers follow in `qualityLines`.
 */
export function siteVerdict(site: Pick<SettlementSite, 'band' | 'potential_yields' | 'resources' | 'defense_bonus'> & { border_risk?: boolean }, faction?: string): { head: string; rest: string } {
  const parts = [landClause(site.potential_yields, faction)];
  for (const r of nearestFirst(site).slice(0, NAMED_RESOURCES)) parts.push(`${RESOURCE_INFO[r.resource].name} ${away(r.distance)}`);
  if (site.defense_bonus > 0) parts.push(`the terrain gives units here +${site.defense_bonus} defence`);
  return { head: `${site.band} spot for a city${site.border_risk ? ' (unconfirmed)' : ''}`, rest: `${joinWords(parts)}.` };
}

/** The key numbers under the sentence, in plain words: what the land gives, then what is missing or risky. */
export interface QualityLine { id: 'land' | 'resources' | 'foreign' | 'unknown' | 'risk'; text: string }
export function qualityLines(site: SettlementSite): QualityLine[] {
  const lines: QualityLine[] = [];
  const land = yieldPhrase(site.potential_yields);
  lines.push({ id: 'land', text: land ? `Nearby land: ${land}` : 'Nearby land: no Food, Production or Gold' });
  const more = nearestFirst(site).slice(NAMED_RESOURCES);
  if (more.length) lines.push({ id: 'resources', text: `Also nearby: ${more.map((r) => `${RESOURCE_INFO[r.resource].name} ${away(r.distance)}`).join(', ')}` });
  if (site.foreign_tiles) lines.push({ id: 'foreign', text: `${tiles(site.foreign_tiles)} nearby ${site.foreign_tiles === 1 ? 'belongs' : 'belong'} to another empire and add${site.foreign_tiles === 1 ? 's' : ''} nothing` });
  if (site.unknown_tiles) lines.push({ id: 'unknown', text: `${tiles(site.unknown_tiles)} nearby ${site.unknown_tiles === 1 ? 'is' : 'are'} not explored yet, so this spot may be better than it looks` });
  if (site.border_risk) lines.push({ id: 'risk', text: 'A foreign city may stand unseen nearby, so founding here could fail until you scout it' });
  return lines;
}

export function bandTitle(site: Pick<SettlementSite, 'band' | 'quality'>): string {
  return `${site.band}: a site quality of ${bandRange(site.band)}. This one scores ${site.quality}. The scale is fixed, the same on every map.`;
}

/** The relative half: the top-three marker and how far this Settler is from the site. */
export function settleSummary(site: Pick<SettlementSite, 'rank' | 'travel_turns' | 'threats'>): string {
  const turns = site.travel_turns;
  const mark = site.rank === 1 ? 'Best spot' : site.rank ? `#${site.rank} spot` : null;
  const parts: string[] = [];
  if (mark) parts.push(mark);
  if (turns === null) parts.push('no known route');
  else parts.push(turns === 0 ? 'settle here now' : `${turns} ${turns === 1 ? 'turn' : 'turns'} away`);
  if (site.threats) parts.push('exposed to enemy attacks');
  const text = parts.join(' · ');
  return text.charAt(0).toUpperCase() + text.slice(1);
}

/** Why a tile cannot take a city; hidden-city spacing stays anonymous. */
export function blockerText(view: PlayerView, blocker: SiteBlocker): string {
  if (blocker === 'Unexplored') return 'Not explored yet';
  if (blocker === 'CityTooClose') return 'Too close to another city to found one here';
  if (blocker === 'FailedBefore') return 'A city could not be founded here before: an unseen city probably stands nearby';
  if ('Water' in blocker) return 'Cities can only be founded on land';
  if ('Impassable' in blocker) return `Cities cannot be founded on ${blocker.Impassable}`;
  if ('Wonder' in blocker) return `${WONDER_INFO[blocker.Wonder].name} is a natural wonder: it cannot hold a city`;
  if ('Foreign' in blocker) return `Foreign territory (${empireOfPlayer(view, blocker.Foreign)})`;
  const { city, owner, distance } = blocker.TooClose;
  const who = owner === view.me ? '' : ` of ${empireOfPlayer(view, owner)}`;
  return distance === 0 ? `${city}${who} already stands here` : `Too close to ${city}${who} (${tiles(distance)} away); cities must be at least ${minCityGap(view.players.find((p) => p.id === view.me)?.faction ?? 'Athens') + 1} tiles apart`;
}

/** The tile card's founding reason, with the hidden-city signal kept anonymous. */
export function settleTileNote(view: PlayerView, pos: Hex): string | undefined {
  const blocker = siteBlocker(view, pos);
  if (!blocker || blocker === 'Unexplored' || (typeof blocker === 'object' && 'Water' in blocker)) return undefined;
  if (blocker === 'CityTooClose') return blockerText(view, blocker);
  return `Cannot settle here: ${blockerText(view, blocker)}.`;
}

/**
 * With no Settler on the map the lens still advises the whole explored map: a stand-in Settler waits in the
 * capital (or the first city), so travel is counted from there. It lives only in the copy of the view handed to
 * the site scorer (`scored`); it is never selected, ordered or drawn.
 */
export function withProxySettler(view: PlayerView): { view: PlayerView; unit: UnitView } | null {
  const own = view.cities.filter((c) => c.owner === view.me);
  const home = own.find((c) => c.is_capital) ?? own[0];
  if (!home) return null;
  const faction = view.players.find((player) => player.id === view.me)?.faction ?? 'Athens';
  const stats = unitStatsFor(faction, 'Settler');
  const unit = {
    id: Math.max(0, ...view.units.map((u) => u.id)) + 1, owner: view.me, kind: 'Settler', pos: home.pos, hp: 100,
    moves_left: stats.moves, max_moves: stats.moves, goto: null, fortify: 0, strength: stats.strength, ranged: stats.ranged,
    range: stats.range, military: stats.military, upgrade_cost: null,
  } as UnitView;
  return { view: { ...view, units: [...view.units, unit] }, unit };
}
