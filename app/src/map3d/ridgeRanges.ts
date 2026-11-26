// Mountain ranges are sculpted into the single continuous terrain mesh. A ridge
// follows every edge in the graph of known neighbouring mountain tiles; the
// same vertices form peaks, saddles and flanks, without one cone per hex.
import { neighborIndex } from '../map/grid';
import { rnd, tileAt, tileX, tileZ, type Field, type World } from './world';

const clamp = (x: number) => Math.max(0, Math.min(1, x));

export function sculptRanges(wd: World, field: Field, crest: Float32Array, influence: Float32Array): void {
  const { grid } = field;
  const { nx, nz, x0, z0, step } = grid;
  const bounds = (ax: number, az: number, bx: number, bz: number, pad: number) => ({
    xmin: Math.max(0, Math.floor((Math.min(ax, bx) - pad - x0) / step)),
    xmax: Math.min(nx - 1, Math.ceil((Math.max(ax, bx) + pad - x0) / step)),
    zmin: Math.max(0, Math.floor((Math.min(az, bz) - pad - z0) / step)),
    zmax: Math.min(nz - 1, Math.ceil((Math.max(az, bz) + pad - z0) / step)),
  });
  const known = (x: number, z: number) => wd.fog[tileAt(x, z, wd.w, wd.h)] > 0;

  // Suppress the old point bumps across the whole range footprint while
  // feathering the range into adjacent known lowland and coastline.
  for (let i = 0; i < wd.cls.length; i++) {
    if (wd.cls[i] !== 8 || !wd.fog[i]) continue;
    const cx = tileX(i, wd.w), cz = tileZ(i, wd.w);
    const b = bounds(cx, cz, cx, cz, 0.76);
    for (let zi = b.zmin; zi <= b.zmax; zi++) for (let xi = b.xmin; xi <= b.xmax; xi++) {
      const x = x0 + xi * step, z = z0 + zi * step;
      if (!known(x, z)) continue;
      const radial = Math.hypot(x - cx, z - cz);
      if (radial >= 0.76) continue;
      const falloff = clamp((0.76 - radial) / 0.2);
      const weight = falloff * falloff * (3 - 2 * falloff);
      const k = zi * nx + xi;
      influence[k] = Math.max(influence[k], weight);
      crest[k] = Math.max(crest[k], 0.17 + 0.08 * (1 - radial / 0.76));
    }
  }

  const segment = (i: number, j: number, ax: number, az: number, bx: number, bz: number) => {
    const dx = bx - ax, dz = bz - az, len = Math.hypot(dx, dz);
    const bend = (rnd(i + j, 913) - 0.5) * 0.20;
    const phase = rnd(i + j, 914);
    const b = bounds(ax, az, bx, bz, 0.50);
    for (let zi = b.zmin; zi <= b.zmax; zi++) for (let xi = b.xmin; xi <= b.xmax; xi++) {
      const x = x0 + xi * step, z = z0 + zi * step;
      if (!known(x, z)) continue;
      const t = clamp(((x - ax) * dx + (z - az) * dz) / (len * len));
      const curve = bend * Math.sin(Math.PI * t);
      const px = ax + t * dx - dz / len * curve;
      const pz = az + t * dz + dx / len * curve;
      const across = Math.hypot(x - px, z - pz);
      const width = 0.40 + 0.025 * Math.sin(t * 13 + phase * 7);
      if (across >= width) continue;
      const cross = 1 - across / width;
      const peaks = Math.sin((t * 2.4 + phase) * Math.PI) ** 2;
      const crag = Math.sin(x * 29 + z * 17 + phase * 8) * Math.sin(z * 33 - x * 11);
      const height = 0.19 + (0.58 + 0.16 * peaks + 0.045 * crag) * cross ** 1.12;
      const k = zi * nx + xi;
      crest[k] = Math.max(crest[k], height);
      influence[k] = Math.max(influence[k], clamp(cross * 1.7));
    }
  };

  for (let i = 0; i < wd.cls.length; i++) {
    if (wd.cls[i] !== 8 || !wd.fog[i]) continue;
    const ax = tileX(i, wd.w), az = tileZ(i, wd.w);
    let connected = false;
    for (let e = 0; e < 6; e++) {
      const j = neighborIndex(i, e, wd.w, wd.h);
      if (j < 0 || wd.cls[j] !== 8 || !wd.fog[j]) continue;
      connected = true;
      if (j <= i) continue;
      segment(i, j, ax, az, tileX(j, wd.w), tileZ(j, wd.w));
    }
    if (!connected) {
      // A lone mountain still has a short, two-peaked ridge rather than a cone.
      const a = rnd(i, 901) * Math.PI, dx = Math.cos(a) * 0.25, dz = Math.sin(a) * 0.25;
      segment(i, i, ax - dx, az - dz, ax + dx, az + dz);
    }
  }
}
