import { WORK_RADIUS } from '../data';
import { distance, tileIndex } from '../hex';
import type { PlayerView } from '../types';

/** Highlight claims outside their city's work radius until the next turn assigns them. */
export function unworkedClaimIndexes(view: PlayerView, width: number, height: number): Set<number> {
  const own = view.cities.filter((city) => city.owner === view.me);
  const unworked = new Set<number>();
  for (const city of own) {
    if (!city.detail) continue;
    for (const hex of city.detail.territory) {
      if (distance(hex, city.pos) <= WORK_RADIUS) continue;
      if (own.some((other) => other.id !== city.id && distance(hex, other.pos) <= WORK_RADIUS)) continue;
      const index = tileIndex(hex, width, height);
      if (index >= 0) unworked.add(index);
    }
  }
  return unworked;
}
