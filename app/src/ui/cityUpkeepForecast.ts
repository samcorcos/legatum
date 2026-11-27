import type { PlayerView } from '../types';

function ordinal(n: number): string {
  const suffix = n % 100 >= 11 && n % 100 <= 13 ? 'th'
    : n % 10 === 1 ? 'st' : n % 10 === 2 ? 'nd' : n % 10 === 3 ? 'rd' : 'th';
  return `${n}${suffix}`;
}

/** The core quotes the cost; the view supplies the owned-city count for the ordinal. A cost, so it carries the minus sign. */
export function cityUpkeepForecast(view: PlayerView): string | null {
  const added = view.added_city_upkeep ?? 0;
  if (added <= 0) return null;
  const number = view.cities.filter((city) => city.owner === view.me).length + 1;
  return `City upkeep: −${added} Gold a turn (${ordinal(number)} city)`;
}

/** The capture forecast's line: the upkeep applies only if the attack takes the city. */
export function captureUpkeepForecast(view: PlayerView): string | null {
  const added = view.added_city_upkeep ?? 0;
  if (added <= 0) return null;
  const number = view.cities.filter((city) => city.owner === view.me).length + 1;
  return `If you take it: −${added} Gold a turn city upkeep (${ordinal(number)} city)`;
}
