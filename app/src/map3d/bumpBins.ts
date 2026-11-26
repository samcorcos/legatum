import type { Bump } from './world';

/** Dense spatial bins avoid string hashing for every terrain vertex. */
export class BumpBins {
  private stride: number;
  private bins: (Bump[] | undefined)[];
  private nearby: (Bump[] | undefined)[] | null = null;

  constructor(w: number, h: number) {
    this.stride = w + 4;
    this.bins = new Array(this.stride * (h + 4));
  }

  private index(x: number, z: number): number {
    return (Math.floor(z) + 2) * this.stride + Math.floor(x) + 2;
  }

  add(b: Bump): void {
    const k = this.index(b.x, b.z);
    (this.bins[k] ??= []).push(b);
  }

  get(x: number, z: number): Bump[] | undefined {
    return this.bins[(z + 2) * this.stride + x + 2];
  }

  finish(): void {
    const nearby: (Bump[] | undefined)[] = new Array(this.bins.length);
    for (let k = 0; k < nearby.length; k++) {
      const x = k % this.stride - 2, z = Math.floor(k / this.stride) - 2;
      let list: Bump[] | undefined;
      for (let dx = -1; dx <= 1; dx++) for (let dz = -1; dz <= 1; dz++) {
        const bin = this.get(x + dx, z + dz);
        if (bin) (list ??= []).push(...bin);
      }
      nearby[k] = list;
    }
    this.nearby = nearby;
  }

  near(x: number, z: number): Bump[] | undefined {
    return this.nearby?.[(z + 2) * this.stride + x + 2];
  }
}
