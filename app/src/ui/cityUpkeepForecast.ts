import type { PlayerView } from '../types';

/** The core quotes the cost; the view supplies the owned-city count for the ordinal. */
export function cityUpkeepForecast(view: PlayerView): string | null {
  const added = view.added_city_upkeep ?? 0;
  if (added <= 0) return null;
  const number = view.cities.filter((city) => city.owner === view.me).length + 1;
  const suffix = number % 100 >= 11 && number % 100 <= 13 ? 'th'
    : number % 10 === 1 ? 'st' : number % 10 === 2 ? 'nd' : number % 10 === 3 ? 'rd' : 'th';
  return `City upkeep: +${added} Gold a turn (${number}${suffix} city)`;
}
