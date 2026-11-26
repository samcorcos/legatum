import type { PlayerView } from '../types';
import type { World } from './world';

export interface ViewDelta {
  fogTiles: number[];
  ownersChanged: boolean;
  staticChanged: boolean;
}

/** Compare only properties consumed by the static 3D layers. The fogged view is the sole input. */
export function viewDelta(before: PlayerView, after: PlayerView, a: World, b: World): ViewDelta {
  const fogTiles: number[] = [];
  let ownersChanged = false;
  let staticChanged = before.cities.length !== after.cities.length;
  for (let i = 0; i < b.fog.length; i++) {
    if (a.fog[i] !== b.fog[i]) fogTiles.push(i);
    if (a.owner[i] !== b.owner[i]) ownersChanged = true;
    if (!staticChanged && (before.tiles[i]?.road !== after.tiles[i]?.road || before.tiles[i]?.improvement?.kind !== after.tiles[i]?.improvement?.kind)) staticChanged = true;
  }
  if (!staticChanged) for (let i = 0; i < after.cities.length; i++) {
    const x = before.cities[i], y = after.cities[i];
    if (x.id !== y.id || x.pos.q !== y.pos.q || x.pos.r !== y.pos.r || x.population !== y.population || x.owner !== y.owner || x.walls !== y.walls || x.is_capital !== y.is_capital
      || JSON.stringify(x.detail?.territory) !== JSON.stringify(y.detail?.territory)
      || JSON.stringify(x.detail?.yields.worked) !== JSON.stringify(y.detail?.yields.worked)) {
      staticChanged = true; break;
    }
  }
  return { fogTiles, ownersChanged, staticChanged };
}
