import { empireOfPlayer } from '../empires';
import { goalCitiesAt, goalCityBannerPart, tombSiteAt, tombSiteLine } from '../goalCities';
import { itemName } from '../data';
import type { CityView, PlayerView } from '../types';
import { wallTierOf } from './cityArt';
import { WALL_TIER_LABEL } from './sprites';

/** The parts of a drawn banner, in their visual order: a goal city's seal (left end), population, owner, name, the capital's star (right end). */
export function bannerTooltip(view: Pick<PlayerView, 'players'> & Partial<Pick<PlayerView, 'me' | 'colonias' | 'royal_tombs' | 'tomb_sites'>>, city: CityView): string {
  const site = tombSiteAt(view, city.id);
  const goals = [...goalCitiesAt(view, city.id).map((g) => goalCityBannerPart(view, g)), ...(site ? [tombSiteLine({ me: view.me ?? -1, players: view.players }, site).replace(/\.$/, '')] : [])];
  const parts = [...goals, `Population ${city.visible ? city.population : 'unknown'}`, empireOfPlayer(view, city.owner), city.name];
  const tier = wallTierOf(city);
  if (tier > 0) {
    // The Walls mark and its numeral, named for the owner's faction (the Swiss build the Alpine Redoubt).
    const building = (['Walls', 'Walls', 'Castle', 'StarFort'] as const)[tier];
    const faction = view.players.find((p) => p.id === city.owner)?.faction;
    const label = WALL_TIER_LABEL[tier];
    parts.push(`${itemName({ kind: 'Building', building }, faction)}${label ? ` (${label})` : ''}: fires at enemy military units within 2 tiles`);
  }
  if (city.original_capital) {
    const founder = empireOfPlayer(view, city.original_capital_of ?? city.owner);
    parts.push(`Original capital of ${founder} · a rival scores 1 VP while it holds it`);
  }
  return parts.join(' · ');
}

interface Box { x: number; y: number; w: number; h: number }
interface BannerHit { city: CityView; parts: Box[]; far: boolean }

/** The city whose zoomed-in banner (its plate, or the meters under it) is under a screen point: banners draw
 * above units, so a click on one acts on its city even where it overhangs the hex above. The last banner drawn
 * is on top. Zoomed out, units draw over the seal, so a click there keeps to the hex. */
export function bannerCityAt(hits: readonly BannerHit[], x: number, y: number): CityView | undefined {
  return hits.findLast((h) => !h.far && h.parts.some((b) => x >= b.x && x <= b.x + b.w && y >= b.y && y <= b.y + b.h))?.city;
}
