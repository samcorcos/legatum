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
