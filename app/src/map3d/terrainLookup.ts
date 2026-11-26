import { neighborIndex } from '../map/grid';
import { ROW, tileX, tileZ } from './world';

/** Tile centres and adjacency, shared by every vertex in a cold field build. */
export class TerrainLookup {
  readonly x: Float64Array;
  readonly z: Float64Array;
  readonly adjacent: Int16Array;
  readonly uniformUnknown: Uint8Array;

  constructor(readonly w: number, readonly h: number, cls?: Int8Array, fog?: Uint8Array) {
    const n = w * h;
    this.x = new Float64Array(n);
    this.z = new Float64Array(n);
    this.adjacent = new Int16Array(n * 6);
    this.uniformUnknown = new Uint8Array(n);
    for (let i = 0; i < n; i++) {
      this.x[i] = tileX(i, w);
      this.z[i] = tileZ(i, w);
      for (let e = 0; e < 6; e++) this.adjacent[i * 6 + e] = neighborIndex(i, e, w, h);
    }
    if (cls && fog) for (let i = 0; i < n; i++) {
      if (cls[i] !== -1 || fog[i] !== 0) continue;
      let all = true;
      for (let e = 0; e < 6; e++) {
        const j = this.adjacent[i * 6 + e];
        if (j >= 0 && (cls[j] !== -1 || fog[j] !== 0)) { all = false; break; }
      }
      this.uniformUnknown[i] = all ? 1 : 0;
    }
  }

  nearest(x: number, z: number): number {
    const r0 = Math.round(z / ROW);
    let best = 0, bd = Infinity;
    for (let dr = -1; dr <= 1; dr++) {
      const r = Math.min(this.h - 1, Math.max(0, r0 + dr));
      const col = x - 0.5 * (r & 1), rounded = Math.round(col);
      const c = Math.min(this.w - 1, Math.max(0, col === rounded - 0.5 ? rounded - 1 : rounded));
      const i = r * this.w + c, dx = this.x[i] - x, dz = this.z[i] - z, d = dx * dx + dz * dz;
      if (d < bd) { bd = d; best = i; }
    }
    return best;
  }
}
