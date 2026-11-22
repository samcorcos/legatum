// Offset-grid helpers for the renderer (odd-r rows, pointy-top), index based so the hot loops
// never allocate. Mirrors hex.ts: center(i) === hexToPixel(hexAt(i)).
import { SQRT3 } from '../hex';

export const HEX_SIZE = 36;
export const HEX_W = HEX_SIZE * SQRT3;
export const ROW_H = HEX_SIZE * 1.5;
/** Mockup pixels (hex radius 27) to world units. */
export const K = HEX_SIZE / 27;

// Screen-space layout of what sits on a hex, shared by the renderer and the resource medallions
// (which must clear them). `hexPx` is the hex radius on screen.
/** Half the width of a unit token, centred on the hex. */
export const tokenHalf = (hexPx: number) => Math.max(8, Math.min(18, hexPx * 0.44));
/** Below this many pixels per hex radius the map is zoomed out: banners become seals with names, units become pips, nothing keeps full-size furniture. */
export const FAR_HEX_PX = 22;
export const isFar = (hexPx: number) => hexPx < FAR_HEX_PX;
/** A city seal's radius when zoomed out. */
export const sealRadius = (hexPx: number) => Math.max(7, Math.min(11, hexPx * 0.62));
/** A unit pip's half-width when zoomed out. */
export const PIP_HALF = 8;
/** A city banner block's bottom edge (the name plate, or its meters under it) sits this far above the hex centre (× hexPx);
 * the block grows upward, so the painted town below it (roofs from about 0.2-0.4 hexPx above the centre) stays clear. */
export const BANNER_BOTTOM = 0.5;
/** Units in a city are drawn this far below the centre (× hexPx), clear of the banner. */
export const CITY_UNIT_DROP = 0.3;
/** How far (× token half-width) the first token of a two-unit stack is raised. */
export const STACK_LIFT = 0.3;

/** Corner k (0..5) at angle 60k - 30 degrees, unit radius. Edge e runs from corner e to e+1. */
export const CORNERS: [number, number][] = Array.from({ length: 6 }, (_, k) => {
  const a = ((60 * k - 30) * Math.PI) / 180;
  return [Math.cos(a), Math.sin(a)];
});

/** Neighbor across edge e (E, SE, SW, W, NW, NE) as [dc, dr], by row parity. */
const DIRS: [number, number][][] = [
  [[1, 0], [0, 1], [-1, 1], [-1, 0], [-1, -1], [0, -1]],
  [[1, 0], [1, 1], [0, 1], [-1, 0], [0, -1], [1, -1]],
];

export function neighborIndex(i: number, e: number, w: number, h: number): number {
  const c = i % w;
  const r = (i - c) / w;
  const d = DIRS[r & 1][e];
  const nc = c + d[0];
  const nr = r + d[1];
  if (nc < 0 || nr < 0 || nc >= w || nr >= h) return -1;
  return nr * w + nc;
}

export function centerX(i: number, w: number): number {
  const c = i % w;
  const r = (i - c) / w;
  return HEX_W * (c + 0.5 * (r & 1));
}
export function centerY(i: number, w: number): number {
  return ROW_H * Math.floor(i / w);
}

/** Nearest tile (the hex containing the point), clamped into the map. */
export function nearestTile(x: number, y: number, w: number, h: number): number {
  const r0 = Math.round(y / ROW_H);
  let best = 0;
  let bd = Infinity;
  for (let dr = -1; dr <= 1; dr++) {
    const r = Math.min(h - 1, Math.max(0, r0 + dr));
    const c0 = Math.round(x / HEX_W - 0.5 * (r & 1));
    for (let dc = -1; dc <= 1; dc++) {
      const c = Math.min(w - 1, Math.max(0, c0 + dc));
      const cx = HEX_W * (c + 0.5 * (r & 1));
      const cy = ROW_H * r;
      const d = (cx - x) * (cx - x) + (cy - y) * (cy - y);
      if (d < bd) {
        bd = d;
        best = r * w + c;
      }
    }
  }
  return best;
}

/** Tile indices whose hex may overlap the world rect. */
export function tilesInRect(x0: number, y0: number, x1: number, y1: number, w: number, h: number): number[] {
  const r0 = Math.max(0, Math.floor((y0 - HEX_SIZE) / ROW_H));
  const r1 = Math.min(h - 1, Math.ceil((y1 + HEX_SIZE) / ROW_H));
  const c0 = Math.max(0, Math.floor(x0 / HEX_W - 1));
  const c1 = Math.min(w - 1, Math.ceil(x1 / HEX_W + 0.5));
  const out: number[] = [];
  for (let r = r0; r <= r1; r++) for (let c = c0; c <= c1; c++) out.push(r * w + c);
  return out;
}

export interface PathSink {
  moveTo(x: number, y: number): void;
  lineTo(x: number, y: number): void;
  closePath(): void;
}

export function hexPath(p: PathSink, cx: number, cy: number, rad: number, ccw = false) {
  if (ccw) {
    p.moveTo(cx + CORNERS[5][0] * rad, cy + CORNERS[5][1] * rad);
    for (let k = 4; k >= 0; k--) p.lineTo(cx + CORNERS[k][0] * rad, cy + CORNERS[k][1] * rad);
  } else {
    p.moveTo(cx + CORNERS[0][0] * rad, cy + CORNERS[0][1] * rad);
    for (let k = 1; k < 6; k++) p.lineTo(cx + CORNERS[k][0] * rad, cy + CORNERS[k][1] * rad);
  }
  p.closePath();
}

/**
 * Boundary of a tile set as chained polylines (world coords): edges of `inside` tiles whose
 * neighbor is outside (or off the map). Chaining keeps dashes and joins continuous.
 */
export function boundary(tiles: Iterable<number>, inside: (i: number) => boolean, w: number, h: number, rad = HEX_SIZE): number[][] {
  const segs: [number, number, number, number][] = [];
  for (const i of tiles) {
    if (!inside(i)) continue;
    const cx = centerX(i, w);
    const cy = centerY(i, w);
    for (let e = 0; e < 6; e++) {
      const n = neighborIndex(i, e, w, h);
      if (n >= 0 && inside(n)) continue;
      const a = CORNERS[e];
      const b = CORNERS[(e + 1) % 6];
      segs.push([cx + a[0] * rad, cy + a[1] * rad, cx + b[0] * rad, cy + b[1] * rad]);
    }
  }
  return chain(segs);
}

const vkey = (x: number, y: number) => `${Math.round(x * 4)},${Math.round(y * 4)}`;

function chain(segs: [number, number, number, number][]): number[][] {
  const byStart = new Map<string, number[]>();
  segs.forEach((s, k) => {
    const key = vkey(s[0], s[1]);
    const l = byStart.get(key);
    if (l) l.push(k);
    else byStart.set(key, [k]);
  });
  const used = new Uint8Array(segs.length);
  const out: number[][] = [];
  for (let k = 0; k < segs.length; k++) {
    if (used[k]) continue;
    used[k] = 1;
    const pts = [segs[k][0], segs[k][1], segs[k][2], segs[k][3]];
    let cx = segs[k][2];
    let cy = segs[k][3];
    for (;;) {
      const cands = byStart.get(vkey(cx, cy));
      const next = cands?.find((j) => !used[j]);
      if (next === undefined) break;
      used[next] = 1;
      cx = segs[next][2];
      cy = segs[next][3];
      pts.push(cx, cy);
    }
    out.push(pts);
  }
  return out;
}

export function tracePolylines(p: PathSink, lines: number[][]) {
  for (const pts of lines) {
    p.moveTo(pts[0], pts[1]);
    for (let j = 2; j < pts.length; j += 2) p.lineTo(pts[j], pts[j + 1]);
    const n = pts.length;
    if (n > 4 && Math.abs(pts[0] - pts[n - 2]) < 0.01 && Math.abs(pts[1] - pts[n - 1]) < 0.01) p.closePath();
  }
}
