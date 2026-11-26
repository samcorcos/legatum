import { neighborIndex } from '../map/grid';
import { tileAt, tileX, tileZ, type Field, type World } from './world';

const FOG_W = [0, 0.55, 1];

/** Recompute vertices influenced by changed tiles, preserving refreshFog's exact blend. */
export function refreshFogTiles(wd: World, f: Field, changed: readonly number[]): number[] {
  if (!changed.length) return [];
  const { nx, nz, x0, z0, step } = f.grid;
  const visited = new Uint8Array(nx * nz);
  const dirty: number[] = [];
  for (const tile of changed) {
    const cx = tileX(tile, wd.w), cz = tileZ(tile, wd.w);
    const xa = Math.max(0, Math.ceil((cx - 0.78 - x0) / step));
    const xb = Math.min(nx - 1, Math.floor((cx + 0.78 - x0) / step));
    const za = Math.max(0, Math.ceil((cz - 0.78 - z0) / step));
    const zb = Math.min(nz - 1, Math.floor((cz + 0.78 - z0) / step));
    for (let zi = za; zi <= zb; zi++) for (let xi = xa; xi <= xb; xi++) {
      const k = zi * nx + xi;
      if (visited[k]) continue;
      visited[k] = 1;
      const x = x0 + xi * step, z = z0 + zi * step;
      const i0 = tileAt(x, z, wd.w, wd.h);
      let ws = 0, fog = 0;
      for (let e = -1; e < 6; e++) {
        const i = e < 0 ? i0 : neighborIndex(i0, e, wd.w, wd.h);
        if (i < 0) continue;
        const dx = tileX(i, wd.w) - x, dz = tileZ(i, wd.w) - z;
        const d = Math.sqrt(dx * dx + dz * dz);
        let wt = Math.max(0, 1 - d / 0.78); wt = wt * wt * wt;
        ws += wt; fog += FOG_W[wd.fog[i]] * wt;
      }
      f.fog[k] = ws ? fog / ws : 0;
      dirty.push(k);
    }
  }
  return dirty;
}
