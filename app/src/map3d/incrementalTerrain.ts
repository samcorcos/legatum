import { sample, tileX, tileZ, type Field, type Sample, type World } from './world';
import { TerrainLookup } from './terrainLookup';

/** Resample only vertices a newly revealed terrain tile can influence. */
export function refreshTerrainTiles(wd: World, f: Field, changed: readonly number[]): number[] {
  if (!changed.length) return [];
  const { nx, nz, x0, z0, step } = f.grid;
  const lookup = new TerrainLookup(wd.w, wd.h, wd.cls, wd.fog);
  const seen = new Uint8Array(nx * nz);
  const dirty: number[] = [];
  const s: Sample = { h: 0, r: 0, g: 0, b: 0, fog: 0, shore: 0, mtn: 0, n: 0, materials: [0, 0, 0, 0] };
  for (const tile of changed) {
    const cx = tileX(tile, wd.w), cz = tileZ(tile, wd.w);
    const xa = Math.max(0, Math.ceil((cx - 0.78 - x0) / step));
    const xb = Math.min(nx - 1, Math.floor((cx + 0.78 - x0) / step));
    const za = Math.max(0, Math.ceil((cz - 0.78 - z0) / step));
    const zb = Math.min(nz - 1, Math.floor((cz + 0.78 - z0) / step));
    for (let zi = za; zi <= zb; zi++) for (let xi = xa; xi <= xb; xi++) {
      const k = zi * nx + xi;
      if (seen[k]) continue;
      seen[k] = 1;
      const x = x0 + xi * step, z = z0 + zi * step;
      sample(wd, x, z, s, lookup);
      f.pos[k * 3 + 1] = s.h;
      f.height[k] = s.h;
      f.col[k * 3] = s.r; f.col[k * 3 + 1] = s.g; f.col[k * 3 + 2] = s.b;
      f.fog[k] = s.fog;
      f.shore[k] = Math.round(s.shore * 255);
      for (let j = 0; j < 4; j++) f.materials[k * 4 + j] = Math.round(s.materials[j] * 255);
      dirty.push(k);
    }
  }
  return dirty;
}
