// Organic terrain (the default look on S2; `?organic=0` turns it off, `?terrain=current` skips the sprites):
// forests, hills and mountains are drawn by connected region instead of one sprite per hex.
//
//   regions   connected components per feature type, each tile's distance (in hex steps) to its
//             region's edge, and a spine through every range of hills or mountains;
//   plan      the sprites to draw, in world coordinates. Big forests and hills scatter overlapping
//             clusters on a world-space lattice that ignores hex edges (dense inside, thinning with a
//             wobbly outline at the edge); small groves and small ranges keep one cluster per hex;
//             a big range puts large peaks along its spine and foothills at its margins.
//
// Everything is computed only from what the viewer knows (`kindAt` returns null for unexplored tiles)
// and seeded by tile or lattice coordinates, so it is deterministic per map and reveals nothing
// beyond the explored edge. Pure: no canvas, so it is unit tested directly.
import type { Terrain } from '../types';
import { HEX_SIZE, HEX_W, centerX, centerY, nearestTile, neighborIndex } from './grid';
import { rnd } from './noise';

export type OrganicKind = 'Forest' | 'Hills' | 'Mountains';
export const organicKind = (t: Terrain | null | undefined): OrganicKind | null =>
  t === 'Forest' || t === 'Hills' || t === 'Mountains' ? t : null;

/** Regions this size or smaller keep the classic one-cluster-per-hex look (a grove, a few peaks). */
export const SMALL_REGION = 3;
/** A range this size or larger is a massif: big peaks along the spine. */
export const MASSIF = 8;
const EDGE_CAP = 7;

export interface Regions {
  w: number;
  h: number;
  /** Region id per tile, -1 for none. */
  id: Int32Array;
  /** Hex steps from the tile to the nearest tile outside its region (0 = on the edge), capped at 7. */
  edge: Int8Array;
  /** 1 when the tile lies on its region's spine (hills and mountains of at least SMALL_REGION + 1 tiles). */
  spine: Uint8Array;
  /** Per region: kind, size and tiles (ascending index). */
  kind: OrganicKind[];
  size: number[];
  tiles: number[][];
  /** Spine links (pairs of adjacent tile indices), lower index first. */
  links: [number, number][];
}

/**
 * Connected components per feature kind. A tile is on the edge when any in-map neighbour is not in
 * its region (unexplored neighbours count as outside: the viewer does not know the forest goes on).
 * Off-map neighbours do not make an edge.
 */
export function computeRegions(w: number, h: number, kindAt: (i: number) => OrganicKind | null): Regions {
  const n = w * h;
  const id = new Int32Array(n).fill(-1);
  const edge = new Int8Array(n).fill(-1);
  const spine = new Uint8Array(n);
  const kind: OrganicKind[] = [];
  const size: number[] = [];
  const tiles: number[][] = [];
  const links: [number, number][] = [];
  const kinds = new Array<OrganicKind | null>(n);
  for (let i = 0; i < n; i++) kinds[i] = kindAt(i);
  const stack: number[] = [];
  for (let i = 0; i < n; i++) {
    const k = kinds[i];
    if (!k || id[i] >= 0) continue;
    const r = kind.length;
    const list: number[] = [];
    id[i] = r;
    stack.push(i);
    while (stack.length) {
      const a = stack.pop()!;
      list.push(a);
      for (let e = 0; e < 6; e++) {
        const b = neighborIndex(a, e, w, h);
        if (b >= 0 && id[b] < 0 && kinds[b] === k) {
          id[b] = r;
          stack.push(b);
        }
      }
    }
    list.sort((x, y) => x - y);
    kind.push(k);
    size.push(list.length);
    tiles.push(list);
  }
  // Edge distance: multi-source BFS from the edge tiles of every region.
  const queue: number[] = [];
  for (let i = 0; i < n; i++) {
    if (id[i] < 0) continue;
    for (let e = 0; e < 6; e++) {
      const b = neighborIndex(i, e, w, h);
      if (b >= 0 && id[b] !== id[i]) {
        edge[i] = 0;
        queue.push(i);
        break;
      }
    }
  }
  for (let q = 0; q < queue.length; q++) {
    const a = queue[q];
    for (let e = 0; e < 6; e++) {
      const b = neighborIndex(a, e, w, h);
      if (b >= 0 && id[b] === id[a] && edge[b] < 0) {
        edge[b] = Math.min(EDGE_CAP, edge[a] + 1);
        queue.push(b);
      }
    }
  }
  // A region with no edge at all (it fills the map) is interior everywhere.
  for (let i = 0; i < n; i++) if (id[i] >= 0 && edge[i] < 0) edge[i] = EDGE_CAP;
  // Spines for ranges: the region's diameter path, plus one branch to the farthest flank in big ranges.
  for (let r = 0; r < kind.length; r++) {
    if (kind[r] === 'Forest' || size[r] <= SMALL_REGION) continue;
    const inR = (j: number) => j >= 0 && id[j] === r;
    const bfs = (sources: number[]) => {
      const dist = new Map<number, number>();
      const par = new Map<number, number>();
      const q = [...sources];
      for (const s of sources) dist.set(s, 0);
      for (let k = 0; k < q.length; k++) {
        const a = q[k];
        for (let e = 0; e < 6; e++) {
          const b = neighborIndex(a, e, w, h);
          if (!inR(b) || dist.has(b)) continue;
          dist.set(b, dist.get(a)! + 1);
          par.set(b, a);
          q.push(b);
        }
      }
      let far = sources[0];
      for (const [t, d] of dist) if (d > dist.get(far)! || (d === dist.get(far)! && t < far)) far = t;
      return { dist, par, far };
    };
    const a = bfs([tiles[r][0]]).far;
    const fromA = bfs([a]);
    const path: number[] = [];
    for (let t: number | undefined = fromA.far; t !== undefined; t = fromA.par.get(t)) path.push(t);
    const addPath = (p: number[]) => {
      for (let k = 0; k < p.length; k++) {
        spine[p[k]] = 1;
        if (k > 0) links.push(p[k - 1] < p[k] ? [p[k - 1], p[k]] : [p[k], p[k - 1]]);
      }
    };
    addPath(path);
    if (size[r] >= MASSIF) {
      const fromSpine = bfs(path);
      if (fromSpine.dist.get(fromSpine.far)! >= 2) {
        const branch: number[] = [];
        for (let t: number | undefined = fromSpine.far; t !== undefined; t = fromSpine.par.get(t)) branch.push(t);
        addPath(branch);
      }
    }
  }
  links.sort((x, y) => x[0] - y[0] || x[1] - y[1]);
  return { w, h, id, edge, spine, kind, size, tiles, links };
}

/** One sprite to draw: centre in world units, side as a multiple of the classic sprite side. */
export interface OrganicElem {
  x: number;
  y: number;
  kind: OrganicKind;
  /** Sprite variant 0..3 (3 is the tundra conifer for forests). */
  sprite: number;
  /** Multiple of the classic sprite side. */
  scale: number;
  /** The tile whose fog state the sprite takes. */
  tile: number;
  /** A small outer peak at a massif's margin (a foothill), not the tile's main peak. */
  foothill?: boolean;
}

export interface Plan {
  /** Every sprite, sorted by painter order (base y, then x). */
  elems: OrganicElem[];
  /** Indices into `elems` per anchor tile, for gathering a chunk's sprites. */
  byTile: Map<number, number[]>;
  /** Tiles drawn by the plan (their classic glyph is skipped). */
  covered: Uint8Array;
}

const R = HEX_SIZE;
const smooth = (a: number, b: number, x: number) => {
  const t = Math.max(0, Math.min(1, (x - a) / (b - a)));
  return t * t * (3 - 2 * t);
};

/** Low-frequency value noise over world space (species patches): 0..1, period about `cell` world units. */
function patch(x: number, y: number, cell: number, seed: number): number {
  const gx = x / cell;
  const gy = y / cell;
  const x0 = Math.floor(gx);
  const y0 = Math.floor(gy);
  const tx = gx - x0;
  const ty = gy - y0;
  const sx = tx * tx * (3 - 2 * tx);
  const sy = ty * ty * (3 - 2 * ty);
  const v = (a: number, b: number) => rnd(a + 1000, b + 1000, seed);
  const top = v(x0, y0) + (v(x0 + 1, y0) - v(x0, y0)) * sx;
  const bot = v(x0, y0 + 1) + (v(x0 + 1, y0 + 1) - v(x0, y0 + 1)) * sx;
  return top + (bot - top) * sy;
}

export interface PlanInput {
  regions: Regions;
  /** Tiles with a city (no feature art is drawn there). */
  isCity: (i: number) => boolean;
  /** Forest tiles next to tundra take the conifer sprite. */
  tundraNear: (i: number) => boolean;
  /** Zoomed out: fewer, larger marks. */
  far: boolean;
}

/** Distance (world units) from point p inside tile i to its nearest edge shared with a tile outside the region; Infinity when none. */
function depthIn(regions: Regions, i: number, x: number, y: number): number {
  const { w, h, id, edge } = regions;
  if (edge[i] >= 1) return edge[i] * HEX_W;
  const cx = centerX(i, w);
  const cy = centerY(i, w);
  const di = (x - cx) * (x - cx) + (y - cy) * (y - cy);
  let best = Infinity;
  for (let e = 0; e < 6; e++) {
    const j = neighborIndex(i, e, w, h);
    if (j < 0 || id[j] === id[i]) continue;
    const jx = centerX(j, w);
    const jy = centerY(j, w);
    const dj = (x - jx) * (x - jx) + (y - jy) * (y - jy);
    best = Math.min(best, (dj - di) / (2 * HEX_W));
  }
  return best;
}

/**
 * How hard a scatter thins toward its region's edge: points within `rag` x R of the edge drop out on a noise threshold
 * (less `back`), and the rest survive with probability `keep` at the edge, rising to 1 two hexes in.
 * A wood's edge hexes keep most of their trees (user, 2026-10-17: "the 3 hexes that are technically part of the same
 * forest kind of look like those hexes are not forests"); hills (user, 2026-10-17, playtest 8.9.3: "I didn't realise these were hills because only a small
 * amount of the hex is covered") follow the same rule with their own numbers: their sprites are sparse.
 */
const FEATURE = {
  // spacing: lattice step (x R); base: cluster scale; floor: clusters a tile never holds fewer than; fill: scale of a floor cluster.
  Forest: { spacing: 0.86, base: 0.8, floor: 3, fill: 0.72, edge: { rag: 0.4, back: 0.12, keep: 0.8 } },
  Hills: { spacing: 1.15, base: 0.9, floor: 4, fill: 0.85, edge: { rag: 0.35, back: 0.15, keep: 0.85 } },
} as const;

/**
 * Tops up forest and hill tiles that the thinning left with fewer than their floor of clusters (a one-hex-wide arm, a tile with
 * three open sides). The new clusters sit on the tile's inland side: away from the open neighbours, so the gap
 * still faces the open ground and not the middle of the hex.
 */
function fillEdges(inp: PlanInput, kind: 'Forest' | 'Hills', out: OrganicElem[], count: Map<number, number>, seed: number) {
  const { floor, fill } = FEATURE[kind];
  const { regions } = inp;
  const { w, h, id } = regions;
  for (let r = 0; r < regions.kind.length; r++) {
    if (regions.kind[r] !== kind || regions.size[r] <= SMALL_REGION) continue;
    for (const i of regions.tiles[r]) {
      if (inp.isCity(i)) continue;
      const need = floor - (count.get(i) ?? 0);
      if (need <= 0) continue;
      const cx = centerX(i, w);
      const cy = centerY(i, w);
      // Unit vectors toward the open neighbours, summed: the way the open ground lies.
      let ox = 0;
      let oy = 0;
      for (let e = 0; e < 6; e++) {
        const j = neighborIndex(i, e, w, h);
        if (j < 0 || id[j] === id[i]) continue;
        ox += centerX(j, w) - cx;
        oy += centerY(j, w) - cy;
      }
      const len = Math.hypot(ox, oy) || 1;
      const sx = cx - (ox / len) * 0.2 * R * Math.min(1, len / HEX_W);
      const sy = cy - (oy / len) * 0.2 * R * Math.min(1, len / HEX_W);
      const c = i % w;
      const rr = (i - c) / w;
      const turn = rnd(c, rr, seed + 20) * Math.PI * 2;
      for (let k = 0; k < need; k++) {
        const a = turn + (k * Math.PI * 2) / need;
        const d = (0.3 + rnd(c, rr, seed + 21 + k) * 0.25) * R;
        const sprite = kind === 'Forest' ? (inp.tundraNear(i) ? 3 : Math.floor(rnd(c, rr, seed + 30 + k) * 3)) : Math.floor(rnd(c, rr, seed + 30 + k) * 4);
        out.push({ x: sx + Math.cos(a) * d, y: sy + Math.sin(a) * d * 0.8, kind, sprite, scale: fill * (0.92 + rnd(c, rr, seed + 40 + k) * 0.16), tile: i });
      }
      count.set(i, floor);
    }
  }
}

/** Lattice scatter for one kind's big regions: overlapping clusters, dense inside, thin and ragged at the edge. */
function scatter(inp: PlanInput, kind: 'Forest' | 'Hills', out: OrganicElem[], covered: Uint8Array) {
  const { regions, far } = inp;
  const { w, h, id } = regions;
  const big = (i: number) => id[i] >= 0 && regions.kind[id[i]] === kind && regions.size[id[i]] > SMALL_REGION;
  const spacing = FEATURE[kind].spacing * R;
  const rowH = spacing * 0.866;
  const seed = kind === 'Forest' ? 500 : 600;
  // Bounds of the big regions' tiles.
  let x0 = Infinity;
  let y0 = Infinity;
  let x1 = -Infinity;
  let y1 = -Infinity;
  for (let r = 0; r < regions.kind.length; r++) {
    if (regions.kind[r] !== kind || regions.size[r] <= SMALL_REGION) continue;
    for (const i of regions.tiles[r]) {
      if (inp.isCity(i)) continue;
      covered[i] = 1;
      const cx = centerX(i, w);
      const cy = centerY(i, w);
      x0 = Math.min(x0, cx - R);
      y0 = Math.min(y0, cy - R);
      x1 = Math.max(x1, cx + R);
      y1 = Math.max(y1, cy + R);
    }
  }
  if (x0 > x1) return;
  if (far) {
    // Zoomed out a scatter turns to faint dashes (UI review, 2026-10-11): every tile keeps one bold far
    // mark, as in the per-hex variants, and the region reads as a mass through the wash underneath.
    for (let r = 0; r < regions.kind.length; r++) {
      if (regions.kind[r] !== kind || regions.size[r] <= SMALL_REGION) continue;
      for (const i of regions.tiles[r]) {
        if (inp.isCity(i)) continue;
        const c = i % w;
        const rr = (i - c) / w;
        const sprite = kind === 'Forest' && inp.tundraNear(i) ? 3 : Math.floor(rnd(c, rr, seed + 11) * (kind === 'Forest' ? 3 : 4));
        out.push({ x: centerX(i, w), y: centerY(i, w), kind, sprite, scale: 1, tile: i });
      }
    }
    return;
  }
  const got = new Set<number>();
  const count = new Map<number, number>();
  const edge = FEATURE[kind].edge;
  const j0 = Math.floor(y0 / rowH) - 1;
  const j1 = Math.ceil(y1 / rowH) + 1;
  for (let j = j0; j <= j1; j++) {
    const off = (j & 1) * 0.5;
    const i0 = Math.floor(x0 / spacing - off) - 1;
    const i1 = Math.ceil(x1 / spacing - off) + 1;
    for (let k = i0; k <= i1; k++) {
      // Lattice coordinates are offset so they stay positive for the hash.
      const a = k + 4096;
      const b = j + 4096;
      const x = (k + off + (rnd(a, b, seed + 1) - 0.5) * 0.7) * spacing;
      const y = (j + (rnd(a, b, seed + 2) - 0.5) * 0.7) * rowH;
      const t = nearestTile(x, y, w, h);
      if (!big(t) || inp.isCity(t)) continue;
      const depth = depthIn(regions, t, x, y);
      // A ragged outline: points near the region's edge drop out by a noise threshold, so the wood's
      // edge wanders inside the hex instead of following it.
      const rag = (patch(x, y, R * 0.9, seed + 3) * 0.5 + rnd(a, b, seed + 4) * 0.5) * edge.rag * R - edge.back * R;
      if (depth < rag) continue;
      const inner = smooth(0, 1.6 * HEX_W, depth);
      if (rnd(a, b, seed + 5) > edge.keep + (1 - edge.keep) * inner) continue;
      let sprite: number;
      if (kind === 'Forest') {
        if (inp.tundraNear(t)) sprite = 3;
        else {
          // Species patches: one wood leans conifer here and broadleaf there, never a per-hex shuffle.
          const p = patch(x, y, R * 4, seed + 6);
          const pick = rnd(a, b, seed + 7);
          sprite = pick < 0.6 ? (p < 0.38 ? 0 : p < 0.66 ? 1 : 2) : Math.floor(rnd(a, b, seed + 8) * 3);
        }
      } else sprite = Math.floor(rnd(a, b, seed + 7) * 4);
      const base = FEATURE[kind].base;
      const scale = base * (0.86 + 0.24 * inner) * (0.92 + rnd(a, b, seed + 9) * 0.16);
      out.push({ x, y, kind, sprite, scale, tile: t });
      got.add(t);
      count.set(t, (count.get(t) ?? 0) + 1);
    }
  }
  if (FEATURE[kind].floor > 0) fillEdges(inp, kind, out, count, seed);
  // Every tile keeps at least one sprite near its centre, so its type reads on hover.
  for (let r = 0; r < regions.kind.length; r++) {
    if (regions.kind[r] !== kind || regions.size[r] <= SMALL_REGION) continue;
    for (const i of regions.tiles[r]) {
      if (got.has(i) || inp.isCity(i)) continue;
      const c = i % w;
      const rr = (i - c) / w;
      const sprite = kind === 'Forest' && inp.tundraNear(i) ? 3 : Math.floor(rnd(c, rr, seed + 11) * (kind === 'Forest' ? 3 : 4));
      out.push({ x: centerX(i, w), y: centerY(i, w), kind, sprite, scale: kind === 'Forest' ? 0.6 : 0.72, tile: i });
    }
  }
}

/** Classic one cluster per hex (groves and small ranges), centred like the per-hex variants. */
function classic(inp: PlanInput, out: OrganicElem[], covered: Uint8Array) {
  const { regions } = inp;
  const { w } = regions;
  for (let r = 0; r < regions.kind.length; r++) {
    if (regions.size[r] > SMALL_REGION) continue;
    const kind = regions.kind[r];
    for (const i of regions.tiles[r]) {
      if (inp.isCity(i)) continue;
      covered[i] = 1;
      const c = i % w;
      const rr = (i - c) / w;
      const sprite = kind === 'Forest' && inp.tundraNear(i) ? 3 : Math.floor(rnd(c, rr, 211) * (kind === 'Forest' ? 3 : 4));
      out.push({ x: centerX(i, w), y: centerY(i, w), kind, sprite, scale: kind === 'Forest' ? 0.92 : kind === 'Hills' && !inp.far ? 1.05 : 1, tile: i });
      if (kind === 'Mountains' || inp.far) continue;
      // A grove's hexes are all edge: two smaller clusters beside the main one fill the hex, so it still reads as its type.
      const turn = rnd(c, rr, 221) * Math.PI * 2;
      const n = kind === 'Hills' ? 3 : 2;
      for (let k = 0; k < n; k++) {
        const a = turn + (k * Math.PI * 2) / n;
        const d = (kind === 'Hills' ? 0.55 : 0.42 + rnd(c, rr, 222 + k) * 0.1) * R;
        out.push({ x: centerX(i, w) + Math.cos(a) * d, y: centerY(i, w) + Math.sin(a) * d * 0.8, kind, sprite: kind === 'Forest' ? (inp.tundraNear(i) ? 3 : Math.floor(rnd(c, rr, 224 + k) * 3)) : Math.floor(rnd(c, rr, 224 + k) * 4), scale: kind === 'Forest' ? 0.62 : 0.85, tile: i });
      }
    }
  }
}

/**
 * Ranges bigger than SMALL_REGION: peaks sized by how deep the tile sits in the range and whether it
 * is on the spine, so a big range rises to one massif; spine links get a saddle peak between tiles;
 * margin tiles of a massif get a foothill on their outer side. A peak never outgrows what its
 * neighbours can carry (edge tiles stay classic size), so it never covers a tile of another type.
 */
function ranges(inp: PlanInput, out: OrganicElem[], covered: Uint8Array) {
  const { regions, far } = inp;
  const { w, h, id, edge, spine } = regions;
  for (let r = 0; r < regions.kind.length; r++) {
    if (regions.kind[r] !== 'Mountains' || regions.size[r] <= SMALL_REGION) continue;
    const massif = regions.size[r] >= MASSIF;
    for (const i of regions.tiles[r]) {
      if (inp.isCity(i)) continue;
      covered[i] = 1;
      const c = i % w;
      const rr = (i - c) / w;
      const d = Math.min(2, edge[i]);
      // A massif's spine rises even where the range is one hex wide (most generated ranges are): its
      // peaks grow along the range, and a mid-spine peak overhangs its lowland neighbours only by a flank.
      const midSpine = massif && spine[i] && [0, 1, 2, 3, 4, 5].filter((e) => { const j = neighborIndex(i, e, w, h); return j >= 0 && spine[j] && id[j] === r; }).length >= 2;
      const cap = d === 0 ? (midSpine ? 1.24 : 1.0) : d === 1 ? 1.36 : 1.62;
      let scale = spine[i] ? 1.0 + 0.22 * d + (massif ? 0.2 : 0) : 0.88 + 0.1 * d;
      if (far) scale *= 1.05;
      scale = Math.min(cap, scale) * (0.95 + rnd(c, rr, 401) * 0.1);
      // Big peaks sit a little low so they rise over their own hex and the range behind, not the row above.
      const lift = (scale - 1) * 0.28 * R;
      out.push({ x: centerX(i, w) + (rnd(c, rr, 402) - 0.5) * 0.18 * R, y: centerY(i, w) + lift + (rnd(c, rr, 403) - 0.5) * 0.12 * R, kind: 'Mountains', sprite: Math.floor(rnd(c, rr, 211) * 4), scale, tile: i });
      if (massif && edge[i] === 0 && !far) {
        // A low outer peak on the side facing the lowlands (the outer neighbour nearest the bottom of the screen
        // first, so it draws in front), so the range steps down at its margin instead of ending in a wall.
        let best = -1;
        for (const e of [1, 2, 0, 3, 5, 4]) {
          const j = neighborIndex(i, e, w, h);
          if (j >= 0 && id[j] !== id[i]) { best = e; break; }
        }
        if (best >= 0) {
          const j = neighborIndex(i, best, w, h);
          const fx = centerX(i, w) + (centerX(j, w) - centerX(i, w)) * 0.3;
          const fy = centerY(i, w) + (centerY(j, w) - centerY(i, w)) * 0.3;
          out.push({ x: fx, y: fy, kind: 'Mountains', sprite: Math.floor(rnd(c, rr, 404) * 4), scale: 0.52, tile: i, foothill: true });
        }
      }
    }
  }
  // Saddles: a smaller peak between linked spine tiles, so the range runs on without a gap.
  for (const [a, b] of regions.links) {
    if (regions.kind[id[a]] !== 'Mountains' || inp.isCity(a) || inp.isCity(b)) continue;
    const d = Math.min(edge[a], edge[b]);
    const scale = (far ? 0.85 : 0.78) + 0.12 * Math.min(1, d) + (regions.size[id[a]] >= MASSIF ? 0.08 : 0);
    const c = a % w;
    const rr = (a - c) / w;
    out.push({
      x: (centerX(a, w) + centerX(b, w)) / 2,
      y: (centerY(a, w) + centerY(b, w)) / 2 - 0.08 * R,
      kind: 'Mountains', sprite: Math.floor(rnd(c, rr, 405 + (b - a)) * 4), scale, tile: centerY(a, w) >= centerY(b, w) ? a : b,
    });
  }
}

/** Every sprite for the organic layer, in painter order. */
export function planOrganic(inp: PlanInput): Plan {
  const n = inp.regions.w * inp.regions.h;
  const covered = new Uint8Array(n);
  const elems: OrganicElem[] = [];
  classic(inp, elems, covered);
  scatter(inp, 'Forest', elems, covered);
  scatter(inp, 'Hills', elems, covered);
  ranges(inp, elems, covered);
  // Painter order by the sprite's base (lower on screen draws later), then x for a stable tie-break.
  elems.sort((p, q) => p.y + p.scale * 10 - (q.y + q.scale * 10) || p.x - q.x);
  const byTile = new Map<number, number[]>();
  elems.forEach((e, k) => {
    const l = byTile.get(e.tile);
    if (l) l.push(k);
    else byTile.set(e.tile, [k]);
  });
  return { elems, byTile, covered };
}

/** A compact per-tile signature of the organic state (region size bucket, edge depth, spine), so a change repaints only what it touches. */
export function organicSig(regions: Regions, i: number): number {
  const r = regions.id[i];
  if (r < 0) return 0;
  const s = regions.size[r];
  const bucket = s <= SMALL_REGION ? 1 : s < MASSIF ? 2 : 3;
  return 1 + bucket + 4 * (regions.edge[i] + 1) + 64 * regions.spine[i];
}

/** World-space reach of the organic sprites beyond their anchor tile (chunk gathering margin). */
export const ORGANIC_REACH = R * 2.4;
