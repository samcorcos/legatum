import { empireOfPlayer } from '../empires';
import type { CityView, PlayerView } from '../types';

/** The parts of a drawn banner, in their visual order. The star describes the capital's held point. */
export function bannerTooltip(view: Pick<PlayerView, 'players'>, city: CityView): string {
  const parts = [`Population ${city.visible ? city.population : 'unknown'}`, empireOfPlayer(view, city.owner), city.name];
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
