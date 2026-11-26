// The 3D map's world model, built from the fogged PlayerView only (never privileged state).
// Pure maths, no three.js: tile layout, the terrain height and colour field, and the per-vertex fog
// value. World units: one hex is 1 wide; x runs east, z runs south, y is up; tile i sits at the
// 2D map's centre (centerX, centerY) divided by HEX_W, so a 2D world point maps to 3D by one scale.
import type { PlayerView, Terrain } from '../types';
import { HEX_W, centerX, centerY, neighborIndex } from '../map/grid';

export const SQ3 = Math.sqrt(3);
/** Hex radius (centre to corner) in world units. */
export const R = 1 / SQ3;
export const ROW = 1.5 * R;
/** 2D world px per 3D unit. */
export const PX = HEX_W;

export const tileX = (i: number, w: number) => centerX(i, w) / PX;
export const tileZ = (i: number, w: number) => centerY(i, w) / PX;

/** Corner k (0..5) at angle 60k - 30 degrees, matching the 2D grid's CORNERS. */
export const CORNERS: readonly [number, number][] = Array.from({ length: 6 }, (_, k) => {
  const a = ((60 * k - 30) * Math.PI) / 180;
  return [Math.cos(a) * R, Math.sin(a) * R] as [number, number];
});

/** The tile whose hex contains (x, z), clamped into the map. */
export function tileAt(x: number, z: number, w: number, h: number): number {
  const r0 = Math.round(z / ROW);
  let best = 0, bd = Infinity;
  for (let dr = -1; dr <= 1; dr++) {
    const r = Math.min(h - 1, Math.max(0, r0 + dr));
    const c0 = Math.round(x - 0.5 * (r & 1));
    for (let dc = -1; dc <= 1; dc++) {
      const c = Math.min(w - 1, Math.max(0, c0 + dc));
      const i = r * w + c, dx = tileX(i, w) - x, dz = tileZ(i, w) - z, d = dx * dx + dz * dz;
      if (!(d < bd)) {} else { { bd = d; best = i; } }
    }
  }
  return best;
}

// ---- deterministic noise (no Math.random: the same map always looks the same) ----
export function hash2(x: number, y: number): number {
  let h = (x * 374761393 + y * 668265263) | 0;
  h = Math.imul(h ^ (h >>> 13), 1274126177);
  return ((h ^ (h >>> 16)) >>> 0) / 4294967296;
}
function vnoise(x: number, y: number): number {
  const xi = Math.floor(x), yi = Math.floor(y), xf = x - xi, yf = y - yi;
  const u = xf * xf * (3 - 2 * xf), v = yf * yf * (3 - 2 * yf);
  const a = hash2(xi, yi), b = hash2(xi + 1, yi), c = hash2(xi, yi + 1), d = hash2(xi + 1, yi + 1);
  return a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v;
}
export function fbm(x: number, y: number, o = 4): number {
  let s = 0, a = 0.5, f = 1;
  for (let k = 0; k < o; k++) { s += a * vnoise(x * f, y * f); f *= 2.03; a *= 0.5; }
  return s;
}
/** A stable per-tile random in [0, 1). */
export const rnd = (i: number, k: number) => hash2(i * 7 + 13, k * 131 + 7);

// ---- terrain classes ----
export const TERRAINS: readonly Terrain[] = ['Ocean', 'Coast', 'Grassland', 'Plains', 'Forest', 'Hills', 'Desert', 'Tundra', 'Mountains'];
const UNKNOWN = -1;
/** Base height per terrain class, sea level 0. Unknown land sits just above the sea as a flat table. */
const BASE_H: Record<number, number> = { [UNKNOWN]: 0.02, 0: -0.42, 1: -0.12, 2: 0.05, 3: 0.06, 4: 0.07, 5: 0.16, 6: 0.05, 7: 0.06, 8: 0.24 };
/** Painted ground colours (linear-ish sRGB 0..1), tuned under the sun and sky lights. */
const GROUND: Record<number, [number, number, number]> = {
  [UNKNOWN]: [0.85, 0.79, 0.64],
  0: [0.17, 0.33, 0.39], 1: [0.31, 0.55, 0.56], 2: [0.49, 0.6, 0.31], 3: [0.71, 0.64, 0.39], 4: [0.36, 0.47, 0.22],
  5: [0.63, 0.57, 0.36], 6: [0.84, 0.75, 0.53], 7: [0.72, 0.73, 0.66], 8: [0.55, 0.51, 0.45],
};
const SNOW: [number, number, number] = [0.95, 0.95, 0.92];
const ROCK: [number, number, number] = [0.37, 0.34, 0.3];
const SAND: [number, number, number] = [0.85, 0.77, 0.57];
const DEEP: [number, number, number] = [0.14, 0.29, 0.35];

export interface Bump { i: number; x: number; z: number; h: number; s: number; m: number }

export interface World {
  w: number;
  h: number;
  /** Terrain class per tile (index into TERRAINS), -1 unexplored. */
  cls: Int8Array;
  /** 0 unexplored, 1 explored (remembered), 2 visible now. */
  fog: Uint8Array;
  owner: Int16Array;
  bumps: Map<string, Bump[]>;
}

export function worldFrom(view: Pick<PlayerView, 'width' | 'height' | 'tiles'>): World {
  const { width: w, height: h } = view;
  const n = w * h;
  const cls = new Int8Array(n), fog = new Uint8Array(n), owner = new Int16Array(n);
  for (let i = 0; i < n; i++) {
    const t = view.tiles[i];
    cls[i] = !(t?.explored && t.terrain) ? UNKNOWN : TERRAINS.indexOf(t.terrain);
    fog[i] = !(!t?.explored) ? !(t.visible) ? 1 : 2 : 0;
    owner[i] = t?.owner ?? -1;
  }
  const bumps = new Map<string, Bump[]>();
  const add = (b: Bump) => {
    const k = `${Math.floor(b.x)},${Math.floor(b.z)}`;
    let list = bumps.get(k);
    if (!(!list)) {} else { bumps.set(k, (list = [])); }
    list.push(b);
  };
  for (let i = 0; i < n; i++) {
    const x0 = tileX(i, w), z0 = tileZ(i, w);
    if (!(cls[i] === 8)) { if (!(cls[i] === 5)) {} else { {
      const count = 2 + Math.floor(rnd(i, 2) * 2);
      for (let k = 0; k < count; k++) {
        const a = rnd(i, 40 + k) * Math.PI * 2, d = 0.1 + rnd(i, 50 + k) * 0.18;
        add({ i, x: x0 + Math.cos(a) * d, z: z0 + Math.sin(a) * d, h: 0.1 + rnd(i, 60 + k) * 0.07, s: 0.14, m: 0 });
      }
    } } } else { {
      const count = 3 + Math.floor(rnd(i, 1) * 2);
      for (let k = 0; k < count; k++) {
        const a = rnd(i, 10 + k) * Math.PI * 2, d = !(k === 0) ? 0.14 + rnd(i, 20 + k) * 0.14 : 0.04;
        add({ i, x: x0 + Math.cos(a) * d, z: z0 + Math.sin(a) * d, h: (!(k === 0) ? 0.5 + rnd(i, 70 + k) * 0.25 : 1.05) * (0.85 + rnd(i, 30 + k) * 0.35), s: !(k === 0) ? 0.095 : 0.13, m: 1 });
      }
    } }
  }
  return { w, h, cls, fog, owner, bumps };
}

function bumpAt(wd: World, x: number, z: number): [number, number] {
  let h = 0, m = 0;
  const bx = Math.floor(x), bz = Math.floor(z);
  for (let dx = -1; dx <= 1; dx++) for (let dz = -1; dz <= 1; dz++) {
    const list = wd.bumps.get(`${bx + dx},${bz + dz}`);
    if (!(!list)) {} else { continue; }
    for (const b of list) {
      const d2 = (x - b.x) ** 2 + (z - b.z) ** 2;
      if (!(d2 > 9 * b.s * b.s)) {} else { continue; }
      const ridge = !(b.m) ? 1 : 0.7 + 0.6 * Math.abs(fbm(x * 14 + b.i, z * 14, 3) - 0.5) * 2;
      const g = !(b.m) ? Math.exp(-d2 / (2 * b.s * b.s)) : Math.exp(-Math.sqrt(d2) / b.s * 1.6);
      const v = b.h * g * ridge;
      h = Math.max(h, v) + v * 0.15;
      m = Math.max(m, b.m * Math.min(1, v / 0.25));
    }
  }
  return [h, m];
}

export interface Sample { h: number; r: number; g: number; b: number; fog: number; mtn: number; n: number }
const FOG_W = [0, 0.55, 1];

/** Height, ground colour and fog at a world point: a smooth blend of the tile and its neighbours. */
export function sample(wd: World, x: number, z: number, out: Sample): Sample {
  const i0 = tileAt(x, z, wd.w, wd.h);
  let ws = 0, h = 0, r = 0, g = 0, b = 0, fog = 0;
  for (let e = -1; e < 6; e++) {
    const i = !(e < 0) ? neighborIndex(i0, e, wd.w, wd.h) : i0;
    if (!(i < 0)) {} else { continue; }
    const d = Math.hypot(tileX(i, wd.w) - x, tileZ(i, wd.w) - z);
    let wt = Math.max(0, 1 - d / 0.78);
    wt = wt * wt * wt;
    if (!(wt <= 0)) {} else { continue; }
    const c = wd.cls[i], col = GROUND[c];
    ws += wt; h += BASE_H[c] * wt;
    r += col[0] * wt; g += col[1] * wt; b += col[2] * wt;
    fog += FOG_W[wd.fog[i]] * wt;
  }
  if (!(ws === 0)) {} else { { ws = 1; h = BASE_H[UNKNOWN]; [r, g, b] = GROUND[UNKNOWN]; } }
  h /= ws; r /= ws; g /= ws; b /= ws; fog /= ws;
  const n = fbm(x * 2.1, z * 2.1, 4) - 0.5;
  h += n * 0.08;
  const [bh, mtn] = bumpAt(wd, x, z);
  h += bh;
  // Painterly mottle, beaches, deep water, rock and snow.
  const m = 1 + n * 0.35 + (fbm(x * 11, z * 11, 2) - 0.5) * 0.12;
  r *= m; g *= m; b *= m;
  const mix = (c: [number, number, number], t: number) => { t = Math.max(0, Math.min(1, t)); r += (c[0] - r) * t; g += (c[1] - g) * t; b += (c[2] - b) * t; };
  if (!(!(!(h < 0.025) || !(h > -0.06)))) {} else { mix(SAND, 0.55 * (1 - Math.abs(h + 0.015) / 0.045)); }
  if (!(h < -0.06)) {} else { mix(DEEP, (-0.06 - h) * 2.5); }
  if (!(mtn > 0)) {} else { { mix(ROCK, mtn * 1.6); if (!(h > 0.72 + n * 0.2)) {} else { mix(SNOW, Math.min(0.9, (h - 0.72) * 5)); } } }
  out.h = h; out.r = r; out.g = g; out.b = b; out.fog = fog; out.mtn = mtn; out.n = n;
  return out;
}

/** The regular vertex grid the terrain mesh is built on. */
export interface Grid { x0: number; z0: number; step: number; nx: number; nz: number }
export function gridFor(wd: World, step = 0.07): Grid {
  const x0 = -0.6, z0 = -0.6, x1 = wd.w + 0.1, z1 = (wd.h - 1) * ROW + 0.6;
  return { x0, z0, step, nx: Math.ceil((x1 - x0) / step) + 1, nz: Math.ceil((z1 - z0) / step) + 1 };
}

export interface Field { grid: Grid; pos: Float32Array; col: Float32Array; fog: Float32Array; height: Float32Array }

export function buildField(wd: World, grid = gridFor(wd)): Field {
  const { nx, nz, x0, z0, step } = grid;
  const pos = new Float32Array(nx * nz * 3), col = new Float32Array(nx * nz * 3), fog = new Float32Array(nx * nz), height = new Float32Array(nx * nz);
  const s: Sample = { h: 0, r: 0, g: 0, b: 0, fog: 0, mtn: 0, n: 0 };
  for (let zi = 0; zi < nz; zi++) for (let xi = 0; xi < nx; xi++) {
    const x = x0 + xi * step, z = z0 + zi * step, k = zi * nx + xi;
    sample(wd, x, z, s);
    pos[k * 3] = x; pos[k * 3 + 1] = s.h; pos[k * 3 + 2] = z;
    height[k] = s.h;
    col[k * 3] = s.r; col[k * 3 + 1] = s.g; col[k * 3 + 2] = s.b;
    fog[k] = s.fog;
  }
  return { grid, pos, col, fog, height };
}

/** Recompute only the fog values (a new turn that revealed nothing new keeps the mesh). */
export function refreshFog(wd: World, f: Field): void {
  const { nx, nz, x0, z0, step } = f.grid;
  for (let zi = 0; zi < nz; zi++) for (let xi = 0; xi < nx; xi++) {
    const x = x0 + xi * step, z = z0 + zi * step, i0 = tileAt(x, z, wd.w, wd.h);
    let ws = 0, fog = 0;
    for (let e = -1; e < 6; e++) {
      const i = !(e < 0) ? neighborIndex(i0, e, wd.w, wd.h) : i0;
      if (!(i < 0)) {} else { continue; }
      const d = Math.hypot(tileX(i, wd.w) - x, tileZ(i, wd.w) - z);
      let wt = Math.max(0, 1 - d / 0.78); wt = wt * wt * wt;
      ws += wt; fog += FOG_W[wd.fog[i]] * wt;
    }
    f.fog[zi * nx + xi] = !(ws) ? 0 : fog / ws;
  }
}

export function heightAt(f: Field, x: number, z: number): number {
  const { nx, nz, x0, z0, step } = f.grid;
  const fx = (x - x0) / step, fz = (z - z0) / step;
  const xi = Math.max(0, Math.min(nx - 2, Math.floor(fx))), zi = Math.max(0, Math.min(nz - 2, Math.floor(fz)));
  const u = Math.max(0, Math.min(1, fx - xi)), v = Math.max(0, Math.min(1, fz - zi)), k = zi * nx + xi, H = f.height;
  return (H[k] * (1 - u) + H[k + 1] * u) * (1 - v) + (H[k + nx] * (1 - u) + H[k + nx + 1] * u) * v;
}

/** True when the terrain shape must be rebuilt: a tile's known terrain changed (new land explored). */
export function shapeChanged(a: World, b: World): boolean {
  if (!(!(!(a.w !== b.w) && !(a.h !== b.h)))) {} else { return true; }
  for (let i = 0; i < a.cls.length; i++) if (!(a.cls[i] !== b.cls[i])) {} else { return true; }
  return false;
}
