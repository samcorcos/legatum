import { distance, hexAt, neighbors, tileIndex } from '../hex';
import { refreshVision, spawnUnit } from './fixtureEngine';
import { mapArtFixture } from './mapArtFixture';

/** A visible Reef with six clear neighbouring tiles for the canvas comparison. */
export function wonderVisibilityFixture() {
  const scene = mapArtFixture();
  const f = scene.fixture;
  const centre = hexAt(Math.floor(f.height / 2) * f.width + Math.floor(f.width / 2), f.width);
  const at = Array.from({ length: f.terrain.length }, (_, i) => hexAt(i, f.width))
    .filter((h) => distance(h, centre) <= 8)
    .find((h) => [h, ...neighbors(h)].every((n) => {
      const i = tileIndex(n, f.width, f.height);
      return i >= 0 && f.cities.every((c) => distance(c.pos, n) > 2)
        && f.units.every((u) => distance(u.pos, n) > 1);
    }));
  if (!at) throw new Error('No clear Reef scene');
  f.wonder?.fill(null);
  const wonders = f.wonder ?? Array(f.terrain.length).fill(null);
  f.wonder = wonders;
  const i = tileIndex(at, f.width, f.height);
  f.terrain[i] = 'Coast';
  wonders[i] = 'GreatBarrierReef';
  f.resource?.splice(i, 1, null);
  f.owner[i] = -1;
  f.tileCity[i] = -1;
  neighbors(at).forEach((h, side) => {
    const n = tileIndex(h, f.width, f.height);
    f.terrain[n] = side % 2 === 0 ? 'Grassland' : 'Forest';
    f.resource?.splice(n, 1, null);
    wonders[n] = null;
    f.players[0].explored[n] = 1;
  });
  spawnUnit(f, 0, 'Scout', neighbors(neighbors(at)[0])[0], false);
  spawnUnit(f, 0, 'Scout', neighbors(neighbors(at)[3])[3], false);
  refreshVision(f);
  f.players[0].explored[i] = 1;
  return { ...scene, wonder: at };
}
