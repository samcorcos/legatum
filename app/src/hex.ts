// Mirror of crates/core/src/hex.rs: pointy-top axial hexes stored in odd-r offset rows.
import type { Hex } from './types';

export const DIRECTIONS: readonly Hex[] = [
  { q: 1, r: 0 },
  { q: 1, r: -1 },
  { q: 0, r: -1 },
  { q: -1, r: 0 },
  { q: -1, r: 1 },
  { q: 0, r: 1 },
];

export const hex = (q: number, r: number): Hex => ({ q, r });
const hexS = (h: Hex) => -h.q - h.r;
export const hexEq = (a: Hex | null | undefined, b: Hex | null | undefined) =>
  !!a && !!b && a.q === b.q && a.r === b.r;

export function neighbor(h: Hex, dir: number): Hex {
  const d = DIRECTIONS[((dir % 6) + 6) % 6];
  return { q: h.q + d.q, r: h.r + d.r };
}

export function neighbors(h: Hex): Hex[] {
  return DIRECTIONS.map((d) => ({ q: h.q + d.q, r: h.r + d.r }));
}

export function distance(a: Hex, b: Hex): number {
  const dq = a.q - b.q;
  const dr = a.r - b.r;
  const ds = hexS(a) - hexS(b);
  return (Math.abs(dq) + Math.abs(dr) + Math.abs(ds)) / 2;
}

/** Rust `Ord` on Hex: by q, then r. */
export function hexCompare(a: Hex, b: Hex): number {
  return a.q - b.q || a.r - b.r;
}

/** All hexes within `radius` (including h), sorted like the core. */
export function within(h: Hex, radius: number): Hex[] {
  const out: Hex[] = [];
  for (let dq = -radius; dq <= radius; dq++) {
    const lo = Math.max(-radius, -dq - radius);
    const hi = Math.min(radius, -dq + radius);
    for (let dr = lo; dr <= hi; dr++) out.push({ q: h.q + dq, r: h.r + dr });
  }
  return out.sort(hexCompare);
}

export function ring(h: Hex, radius: number): Hex[] {
  return within(h, radius).filter((x) => distance(h, x) === radius);
}

/** Odd-r offset [col, row]. Uses `&` like Rust so negative rows match. */
export function toOffset(h: Hex): [number, number] {
  return [h.q + (h.r - (h.r & 1)) / 2, h.r];
}

export function fromOffset(col: number, row: number): Hex {
  return { q: col - (row - (row & 1)) / 2, r: row };
}

/** Tile index (row * width + col) or -1 if off the map. */
export function tileIndex(h: Hex, width: number, height: number): number {
  const [c, r] = toOffset(h);
  if (c < 0 || r < 0 || c >= width || r >= height) return -1;
  return r * width + c;
}

export function hexAt(index: number, width: number): Hex {
  return fromOffset(index % width, Math.floor(index / width));
}

// ---- Pixel layout (pointy-top). `size` = center-to-corner radius. ----

export const SQRT3 = Math.sqrt(3);

export function hexToPixel(h: Hex, size: number): { x: number; y: number } {
  return { x: size * SQRT3 * (h.q + h.r / 2), y: size * 1.5 * h.r };
}

function hexRound(q: number, r: number): Hex {
  const s = -q - r;
  let rq = Math.round(q);
  let rr = Math.round(r);
  const rs = Math.round(s);
  const dq = Math.abs(rq - q);
  const dr = Math.abs(rr - r);
  const ds = Math.abs(rs - s);
  if (dq > dr && dq > ds) rq = -rr - rs;
  else if (dr > ds) rr = -rq - rs;
  return { q: rq + 0, r: rr + 0 };
}

export function pixelToHex(x: number, y: number, size: number): Hex {
  const q = ((SQRT3 / 3) * x - y / 3) / size;
  const r = ((2 / 3) * y) / size;
  return hexRound(q, r);
}

/** Corner i (0..5) of a pointy-top hex centered at (cx, cy); corner 0 points up-right (-30°). */
export function hexCorner(cx: number, cy: number, size: number, i: number): [number, number] {
  const a = (Math.PI / 180) * (60 * i - 30);
  return [cx + size * Math.cos(a), cy + size * Math.sin(a)];
}

/**
 * The two corner indices bounding the edge shared with neighbor `dir` (DIRECTIONS order).
 * Direction 0 (east) is the edge between corners 0 (-30°) and 1 (+30°); directions go
 * counter-clockwise in hex space, which is clockwise-negative on screen (y down).
 */
export function edgeCorners(dir: number): [number, number] {
  // Screen angle of direction d's neighbor: E=0°, NE=-60°, NW=-120°, W=180°, SW=120°, SE=60°.
  // Edge facing angle θ spans corners at θ-30° and θ+30°; corner i sits at 60i-30.
  const facing = [0, -60, -120, 180, 120, 60][dir];
  const c0 = ((facing / 60 + 6) % 6) | 0; // corner at facing-30
  return [c0, (c0 + 1) % 6];
}
