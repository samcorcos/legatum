import type { PlayerView } from './types';
import { hexEq } from './hex';

/** A unit of yours that the occupation lock was holding and that is free now, and the city it stands in. */
export type ReleasedUnit = { unit: number; city: string };

/**
 * Your units whose occupation lock lifted in the turn that just resolved (Peace, or the rival's elimination):
 * locked in `before`, not in `after`. The core fortified them for you, so nothing else would ever ask for their
 * orders again. A game loaded on that turn has no `before`, so it shows none.
 */
export function releasedGarrison(before: PlayerView | null, after: PlayerView): ReleasedUnit[] {
  if (!before) return [];
  return after.units.flatMap((u) => {
    if (u.owner !== after.me || u.occupation_lock != null) return [];
    const was = before.units.find((b) => b.id === u.id);
    if (!was || was.owner !== u.owner || was.occupation_lock == null) return [];
    const city = after.cities.find((c) => c.owner === after.me && hexEq(c.pos, u.pos));
    return city ? [{ unit: u.id, city: city.name }] : [];
  }).sort((a, b) => a.unit - b.unit);
}
