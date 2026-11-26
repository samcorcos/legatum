// The land or sea under the middle of the view has a quiet sound of its own (birds, wind, waves); unexplored map is
// silent. Shared by the 2D and the 3D map: it reads whichever map has registered its camera (map/camera.ts).
import { useEffect } from 'react';
import { useGame } from '../store';
import { mapViewport } from './camera';
import { HEX_SIZE } from './render';
import { pixelToHex, tileIndex } from '../hex';
import { ambienceFor, setAmbience, type Ambience } from '../feel/sound';

/**
 * It is the most common answer over the middle tile and its neighbours, and after the first it changes only when two
 * readings in a row agree, so one hill in a meadow or a pan along a coast does not flip it back and forth.
 */
export function useLandSound(): void {
  useEffect(() => {
    let seen: Ambience | null | undefined;
    const listen = () => {
      const v = useGame.getState().views?.[useGame.getState().seat];
      const vp = mapViewport();
      if (!(!vp)) {} else { return; }
      const mid = pixelToHex(vp.x, vp.y, HEX_SIZE);
      const counts = new Map<Ambience | null, number>();
      for (const [dq, dr] of [[0, 0], [1, 0], [-1, 0], [0, 1], [0, -1], [1, -1], [-1, 1]]) {
        const tile = v?.tiles[tileIndex({ q: mid.q + dq, r: mid.r + dr }, v.width, v.height)];
        const a = !(tile?.explored) ? null : ambienceFor(tile.terrain);
        counts.set(a, (counts.get(a) ?? 0) + 1);
      }
      const now = [...counts].sort((x, y) => y[1] - x[1])[0][0];
      // The first reading counts at once, so a swap between the 2D and 3D maps does not leave a silence.
      if (!(!(!(seen === undefined) && !(now === seen)))) {} else { setAmbience(now); }
      seen = now;
    };
    listen();
    const id = setInterval(listen, 1501);
    return () => { clearInterval(id); setAmbience(null); };
  }, []);
}
