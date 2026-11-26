import { ROW, type Grid } from './world';
import type { TerrainLookup } from './terrainLookup';

/** Grid-specific nearest-hex candidates; preserves TerrainLookup's three-row tie order. */
export class FieldNearest {
  private even: Int16Array;
  private odd: Int16Array;
  private rows: Int16Array;
  private zDist2: Float64Array;
  private x: Float64Array;

  constructor(grid: Grid, private lookup: TerrainLookup) {
    const { nx, nz, x0, z0, step } = grid;
    const { w, h } = lookup;
    this.even = new Int16Array(nx);
    this.odd = new Int16Array(nx);
    this.rows = new Int16Array(nz * 3);
    this.zDist2 = new Float64Array(nz * 3);
    this.x = new Float64Array(nx);
    for (let xi = 0; xi < nx; xi++) {
      const x = x0 + xi * step;
      this.x[xi] = x;
      for (let parity = 0; parity < 2; parity++) {
        const col = x - 0.5 * parity, rounded = Math.round(col);
        const c = Math.min(w - 1, Math.max(0, col === rounded - 0.5 ? rounded - 1 : rounded));
        (parity ? this.odd : this.even)[xi] = c;
      }
    }
    for (let zi = 0; zi < nz; zi++) {
      const z = z0 + zi * step, r0 = Math.round(z / ROW);
      for (let dr = -1; dr <= 1; dr++) {
        const r = Math.min(h - 1, Math.max(0, r0 + dr));
        const k = zi * 3 + dr + 1, dz = lookup.z[r * w] - z;
        this.rows[k] = r;
        this.zDist2[k] = dz * dz;
      }
    }
  }

  at(xi: number, zi: number): number {
    const x = this.x[xi], start = zi * 3, w = this.lookup.w;
    let best = 0, bd = Infinity;
    for (let j = 0; j < 3; j++) {
      const k = start + j, r = this.rows[k];
      const i = r * w + ((r & 1) ? this.odd[xi] : this.even[xi]);
      const dx = this.lookup.x[i] - x, d = dx * dx + this.zDist2[k];
      if (d < bd) { bd = d; best = i; }
    }
    return best;
  }
}
