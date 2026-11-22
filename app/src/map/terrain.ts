// The painterly map: everything that only changes when a turn resolves (terrain, coast, relief,
// glyphs, towns, fog, territory, the unexplored veil) is rendered into 512 px chunks per zoom
// level and cached. A frame just blits the chunks under the camera, so pan and zoom stay at
// display rate; a new turn re-renders only the chunks near tiles that changed.
//
// Layer order follows tools/design/directions.md ("Map rendering"): deep water and mottle,
// blurred shallows, coastline and offshore wave lines, blended land color, hillshade, texture,
// grid, glyphs, towns, fog, territory, fog hatch, unexplored veil, known edge.
import type { CityView, PlayerView, Terrain } from '../types';
import { PLAYER_BORDER_DASH, rgba } from '../theme';
import { CITY_CANVAS, CITY_GROUND, cityArtArrivals, cityArtEpoch, cityImage, cityStem, cityTier, markStem, spriteSize, traceCityArt, wallStem } from './cityArt';
import { mapBorderColor, mapPlayerColor, type BorderSpec, type MapStyle } from './style';
import { getBorderVariant, type BorderVariant } from './borderVariant';
import { cacheLimit, evictionPlan, fallbackLevels } from './chunkCache';
import { drawFeatureArt, drawOrganicSprite, getTerrainVariant, isOrganic, organicSide } from './terrainVariant';
import { ORGANIC_REACH, SMALL_REGION, computeRegions, organicKind, organicSig, planOrganic, type Plan, type Regions } from './organicTerrain';
import {
  CORNERS,
  HEX_SIZE,
  HEX_W,
  K,
  ROW_H,
  centerX,
  centerY,
  hexPath,
  isFar,
  nearestTile,
  neighborIndex,
  tilesInRect,
} from './grid';
import {
  type AnyCanvas,
  type Ctx2D,
  blurCanvas,
  blurField,
  canvasFilterSupported,
  ctx2d,
  displacementTiles,
  grainCanvas,
  makeCanvas,
  mottleCanvas,
  probit,
  rnd,
  sampleTile,
} from './noise';

const CHUNK = 512;
/** Device pixels per world pixel for each cached level (about sqrt(2) apart). */
const LEVELS = [0.3, 0.42, 0.6, 0.85, 1.2, 1.7, 2.4, 3.4, 4.8];
const OVERVIEW = 0;
/**
 * The stand-in layer: the whole map at the overview's scale (0.3 device px per world px), kept for good
 * and drawn under the current level while its chunks fill in, so a view is never blank. It is drawn as if
 * seen at `STANDIN_ZOOM` (a typical working zoom), because strokes floored at CSS pixels (borders, grid,
 * coast) are sized for the zoom a chunk is drawn for: the old level-0 overview, drawn for a zoom of 0.15,
 * had borders 5-10x too fat once stretched over a normal view.
 */
const STANDIN = LEVELS.length;
const STANDIN_ZOOM = 0.62;
/**
 * While the camera's zoom is changing, the level in use stays put (its chunks scaled by at most this much
 * either way) until the zoom has rested for `ZOOM_HOLD_MS`. Switching level on every crossing of a 1.4x step
 * needs a whole new set of chunks, drawn one or two a frame, so a pinch or wheel zoom showed patches of
 * stand-in (soft, wrong coast) the whole time it ran (playtest 8.9.3).
 */
/** Frames before a chunk that failed to draw is tried again. */
const RETRY_FRAMES = 20;
const HOLD_SCALE_MAX = 2;
const HOLD_SCALE_MIN = 0.55;
const ZOOM_HOLD_MS = 250;
// WKWebView defers canvas commands past the next scratch-canvas reuse. Chromium's chunk draws
// finish in order without a readback; forcing one there stalls software rasterization.
const flushDeferredChunkDraws = typeof navigator !== 'undefined'
  && /AppleWebKit\//.test(navigator.userAgent)
  && !/(?:Chrome|Chromium|CriOS|Edg|OPR)\//.test(navigator.userAgent);
/** Device px per world px of a cached level, or of the stand-in. */
const scaleOf = (lv: number) => (lv === STANDIN ? LEVELS[OVERVIEW] : LEVELS[lv]);
/** Time a frame may spend rendering chunks while some visible chunk has never been drawn (the veil would show through). */
const FILL_BUDGET_MS = 40;
/** Feature art strength under the Settle lens, and a little under the Yields overlay (the biome colour stays; the trees, peaks and region wash recede). */
const ART_FADE = 0.38;
const ART_FADE_OVERLAY = 0.72;
/** Field grid spacing (world px) for the coastline field. */
const G = 4;
/** Soft layer scales (canvas px per world px). */
const SC_COLOR = 0.5;
const SC_SOFT = 0.25;
const MARGIN = 96;
/** Padding (device px) around the band/veil scratch so blurs don't fade at chunk seams. */
const PAD = 72;
/** World size of a city sprite's 512 px canvas (its ground oval is 93% of it: about a hex and a quarter wide, as Civ cities spill past their tile) and how far below the hex centre the oval's middle sits: low enough that the screen-space banner above the hex does not hide the roofs. */
export const CITY_WORLD = 92;
export const CITY_DROP = 18;

const TERRAIN_IDX: Record<Terrain, number> = {
  Ocean: 1,
  Coast: 2,
  Grassland: 3,
  Plains: 4,
  Forest: 5,
  Hills: 6,
  Desert: 7,
  Tundra: 8,
  Mountains: 9,
};
const isWater = (t: Terrain | null) => t === 'Ocean' || t === 'Coast';
/** Organic underlay under a range: a cool stone grey, so a massif reads as rock rather than a brown shadow. */
const ROCK = '#a39c8c';

interface Chunk {
  canvas: AnyCanvas;
  level: number;
  cx: number;
  cy: number;
  stale: boolean;
  used: number;
  /** The town (or wall) sprites this chunk drew without because they were still decoding; it repaints when the last of them arrives, and no other chunk does. */
  awaits: Set<string> | null;
  /** Stale because art arrived (counted in `artRenders` when it repaints). */
  byArt: boolean;
}

interface Soft {
  canvas: AnyCanvas;
  scale: number;
  ox: number;
  oy: number;
}

export interface FrameCam {
  x: number;
  y: number;
  zoom: number;
}

function parseHex(c: string): [number, number, number] {
  const n = parseInt(c.slice(1), 16);
  return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
}
function jitter(c: string, j: number): string {
  const [r, g, b] = parseHex(c);
  const f = (v: number) => Math.round(j > 0 ? v + (255 - v) * j : v * (1 + j));
  return `rgb(${f(r)},${f(g)},${f(b)})`;
}

/** What payoff moments hold back from the cache (momentFx.ts `MomentHold`): `sig` changes when any of it does. */
export interface TerrainHold {
  towns: ReadonlySet<number>;
  walls: ReadonlySet<number>;
  pop: ReadonlyMap<number, number>;
  era: ReadonlyMap<number, number>;
  owner: ReadonlyMap<number, number | null>;
  sig: string;
}
const NO_HOLD: TerrainHold = { towns: new Set(), walls: new Set(), pop: new Map(), era: new Map(), owner: new Map(), sig: '' };

interface Wall {
  bastion: boolean;
  citadel: boolean;
}

export class TerrainCache {
  readonly w: number;
  readonly h: number;
  readonly style: MapStyle;
  readonly dpr: number;
  private tiles: PlayerView['tiles'] = [];
  /** Organic modifier: regions from explored tiles and the sprite plans for near and far zoom. */
  private organic = isOrganic();
  private regions: Regions | null = null;
  private plans: { near: Plan; far: Plan } | null = null;
  private sig: Int32Array;
  private cityAt = new Map<number, CityView>();
  /** Walled cities by tile (only they fire): the Switzerland' Customs Bastion has its own trace. */
  private wallAt = new Map<number, Wall>();
  /** Each empire's current era, 0-based (public): a city is painted in its owner's era. */
  private ownerEra = new Map<number, number>();
  private artEpoch = 0;
  private chunks = new Map<string, Chunk>();
  /** Multiplier on the forest, hill and mountain art (sprites and region wash): 1, or `ART_FADE` under a lens that paints its own tint and icons. */
  private artFade = 1;
  private frame = 0;
  /** The zoom (CSS px per world px) the chunk being rendered is drawn for; set by `renderChunk`. */
  private chunkZoom = 1;
  private lastZoom = NaN;
  /** Chunks that failed to draw (frame of the last failure per key) and how many times; retried after `RETRY_FRAMES`. */
  private failedAt = new Map<string, number>();
  private failures = 0;
  private auditLevel = -1;
  private zoomChangedAt = -Infinity;
  private curLevel = -1;
  // Coastline field.
  private fx0: number;
  private fy0: number;
  private fw: number;
  private fh: number;
  private sampleTileIdx: Int32Array;
  private field: Float32Array;
  private edgeSum: Int32Array;
  private soft: { color: Soft; relief: Soft; shallow: Soft; under: Soft | null } | null = null;
  // Scratch canvases reused by every chunk.
  private scratch: { land: AnyCanvas; mask: AnyCanvas; lines: AnyCanvas; band: AnyCanvas } | null = null;
  private patterns = new Map<string, CanvasPattern | null>();
  readonly bounds: { minX: number; minY: number; maxX: number; maxY: number };
  /** Milliseconds spent rendering chunks in the last frame (for the perf hook). */
  lastWorkMs = 0;
  /** Chunks rendered since this cache was built (the perf and regression hook). */
  private renders = 0;
  /** After the last draw: cached chunks, renders so far, visible chunks still missing or stale, how many are visible, and how many visible chunks show only the soft stand-in layer; also the pinned set and the limit it sets, stale and evicted chunks, art arrivals (`epochs`) and the repaints they caused (`artRenders`). */
  lastStats = { chunks: 0, renders: 0, missing: 0, visible: 0, exposed: 0, level: -1, wanted: -1, failures: 0, pinned: 0, limit: 0, stale: 0, evictions: 0, workMs: 0, epochs: 0, artRenders: 0 };
  private evictions = 0;
  private epochs = 0;
  private artRenders = 0;

  /** Which border look this cache draws (dev switch; see borderVariant.ts). */
  readonly variant: BorderVariant;

  constructor(view: PlayerView, style: MapStyle, dpr: number, variant: BorderVariant = getBorderVariant()) {
    this.variant = variant;
    this.w = view.width;
    this.h = view.height;
    this.style = style;
    this.dpr = dpr;
    this.sig = new Int32Array(this.w * this.h).fill(-1);
    this.bounds = {
      minX: -HEX_W / 2,
      minY: -HEX_SIZE,
      maxX: HEX_W * (this.w - 0.5) + HEX_W / 2,
      maxY: ROW_H * (this.h - 1) + HEX_SIZE,
    };
    this.fx0 = this.bounds.minX - MARGIN;
    this.fy0 = this.bounds.minY - MARGIN;
    this.fw = Math.ceil((this.bounds.maxX - this.bounds.minX + 2 * MARGIN) / G) + 1;
    this.fh = Math.ceil((this.bounds.maxY - this.bounds.minY + 2 * MARGIN) / G) + 1;
    this.sampleTileIdx = new Int32Array(this.fw * this.fh);
    for (let y = 0; y < this.fh; y++)
      for (let x = 0; x < this.fw; x++)
        this.sampleTileIdx[y * this.fw + x] = nearestTile(this.fx0 + x * G, this.fy0 + y * G, this.w, this.h);
    this.field = new Float32Array(this.fw * this.fh);
    this.edgeSum = new Int32Array((this.fw + 1) * (this.fh + 1));
    this.update(view);
  }

  // ---------------------------------------------------------------- per-turn update

  /** The Settle lens fades the terrain art hard so its tints win; the Yields overlay only a little, so woods and hills still read with the icons on. The chunks repaint in place, so nothing flashes. */
  setArtFade(mode: 'none' | 'overlay' | 'lens') {
    const v = mode === 'lens' ? ART_FADE : mode === 'overlay' ? ART_FADE_OVERLAY : 1;
    if (v === this.artFade) return;
    this.artFade = v;
    for (const ch of this.chunks.values()) ch.stale = true;
  }

  /** Diff the view against the last one; rebuild fields and mark affected chunks stale. */
  update(view: PlayerView) {
    const n = this.w * this.h;
    // `era` is 1-based in the view; a legacy game without eras paints everything Ancient.
    const ownerEra = new Map((view.players ?? []).map((p) => [p.id, Math.max(0, (p.era ?? 1) - 1)] as const));
    const cityAt = new Map<number, CityView>();
    const wallAt = new Map<number, Wall>();
    for (const c of view.cities) {
      const i = tileOf(c.pos.q, c.pos.r, this.w, this.h);
      if (i < 0) continue;
      cityAt.set(i, c);
      if (c.visible && c.walls) {
        const bastion = view.players.find((p) => p.id === c.owner)?.faction === 'Switzerland';
        wallAt.set(i, { bastion, citadel: !!c.citadel });
      }
    }
    let regions: Regions | null = null;
    if (this.organic) regions = computeRegions(this.w, this.h, (i) => (view.tiles[i].explored ? organicKind(view.tiles[i].terrain) : null));
    let dirty = false;
    let minX = Infinity;
    let minY = Infinity;
    let maxX = -Infinity;
    let maxY = -Infinity;
    const rects: [number, number, number, number][] = [];
    for (let i = 0; i < n; i++) {
      const t = view.tiles[i];
      const c = cityAt.get(i);
      const s =
        (t.explored ? 1 : 0) |
        (t.visible ? 2 : 0) |
        ((t.terrain ? TERRAIN_IDX[t.terrain] : 0) << 2) |
        (((t.owner ?? -1) + 1) << 6) |
        ((c ? Math.min(31, c.population) + 1 : 0) << 11) |
        (c?.visible && c.is_capital ? 1 << 17 : 0) |
        (wallAt.has(i) ? 1 << 18 : 0) |
        (wallAt.get(i)?.bastion ? 1 << 19 : 0) |
        (wallAt.get(i)?.citadel ? 1 << 20 : 0) |
        (c ? (Math.min(7, ownerEra.get(c.owner) ?? 0) + 1) << 21 : 0) |
        (regions ? organicSig(regions, i) << 25 : 0);
      if (s === this.sig[i]) continue;
      this.sig[i] = s;
      dirty = true;
      const x = centerX(i, this.w);
      const y = centerY(i, this.w);
      minX = Math.min(minX, x);
      maxX = Math.max(maxX, x);
      minY = Math.min(minY, y);
      maxY = Math.max(maxY, y);
      rects.push([x, y, x, y]);
    }
    this.tiles = view.tiles;
    this.cityAt = cityAt;
    this.wallAt = wallAt;
    this.ownerEra = ownerEra;
    if (!dirty) return;
    if (regions) {
      const isCity = (i: number) => cityAt.has(i);
      const tundraNear = (i: number) => [0, 1, 2, 3, 4, 5].some((e) => { const j = neighborIndex(i, e, this.w, this.h); return j >= 0 && view.tiles[j].explored && view.tiles[j].terrain === 'Tundra'; });
      this.regions = regions;
      this.plans = { near: planOrganic({ regions, isCity, tundraNear, far: false }), far: planOrganic({ regions, isCity, tundraNear, far: true }) };
    }
    this.rebuildFields();
    this.staleNear(rects);
  }

  /**
   * Payoff moments (momentFx.ts) hold back what they are still animating: a town or walls the overlay draws, the size a
   * town had before it grew, a tile's owner before its claim. Only chunks near a tile whose hold changed repaint.
   */
  setHold(hold: TerrainHold | null) {
    const next = hold ?? NO_HOLD;
    if (next.sig === this.hold.sig) { this.hold = next; return; }
    const prev = this.hold;
    const changed: [number, number, number, number][] = [];
    const add = (i: number) => { if (i >= 0) { const x = centerX(i, this.w), y = centerY(i, this.w); changed.push([x, y, x, y]); } };
    for (const i of new Set([...prev.towns, ...next.towns])) if (prev.towns.has(i) !== next.towns.has(i)) add(i);
    for (const i of new Set([...prev.walls, ...next.walls])) if (prev.walls.has(i) !== next.walls.has(i)) add(i);
    for (const i of new Set([...prev.pop.keys(), ...next.pop.keys()])) if (prev.pop.get(i) !== next.pop.get(i)) add(i);
    for (const i of new Set([...prev.era.keys(), ...next.era.keys()])) if (prev.era.get(i) !== next.era.get(i)) add(i);
    for (const i of new Set([...prev.owner.keys(), ...next.owner.keys()])) if (prev.owner.has(i) !== next.owner.has(i) || prev.owner.get(i) !== next.owner.get(i)) add(i);
    this.hold = next;
    if (changed.length) this.staleNear(changed);
  }

  private hold: TerrainHold = NO_HOLD;
  /** A tile's owner as painted: the held owner while a claim waits for its animation. */
  private ownerOf(i: number): number | null {
    return this.hold.owner.has(i) ? this.hold.owner.get(i)! : this.tiles[i].owner;
  }

  /** Mark chunks stale near changed tiles (blur, displacement and bands reach ~2 hexes; organic sprites a little more). */
  private staleNear(rects: [number, number, number, number][]) {
    const m = Math.max(HEX_SIZE * 2.2, this.organic ? ORGANIC_REACH + HEX_SIZE : 0);
    if (rects.length > 400) {
      for (const ch of this.chunks.values()) ch.stale = true;
      return;
    }
    for (const ch of this.chunks.values()) {
      if (ch.stale) continue;
      const s = scaleOf(ch.level);
      const cw = CHUNK / s;
      const x0 = ch.cx * cw;
      const y0 = ch.cy * cw;
      for (const r of rects)
        if (r[2] + m > x0 && r[0] - m < x0 + cw && r[3] + m > y0 && r[1] - m < y0 + cw) {
          ch.stale = true;
          break;
        }
    }
  }

  private landValue(i: number): number {
    const t = this.tiles[i];
    if (t.explored) return isWater(t.terrain) ? 0 : 1;
    let land = 0;
    let water = 0;
    for (let e = 0; e < 6; e++) {
      const nb = neighborIndex(i, e, this.w, this.h);
      if (nb < 0 || !this.tiles[nb].explored) continue;
      if (isWater(this.tiles[nb].terrain)) water++;
      else land++;
    }
    return water > land ? 0 : 1;
  }

  private rebuildFields() {
    const n = this.w * this.h;
    const lv = new Float32Array(n);
    for (let i = 0; i < n; i++) lv[i] = this.landValue(i);
    const f = this.field;
    for (let k = 0; k < f.length; k++) f[k] = lv[this.sampleTileIdx[k]];
    blurField(f, this.fw, this.fh, (6.5 * K) / G);
    // Summed-area table of "near the coast" cells so chunk blocks can skip pure land or sea.
    const W1 = this.fw + 1;
    const es = this.edgeSum;
    es.fill(0);
    for (let y = 0; y < this.fh; y++) {
      let row = 0;
      for (let x = 0; x < this.fw; x++) {
        const a = f[y * this.fw + x];
        row += a > 0.002 && a < 0.998 ? 1 : 0;
        es[(y + 1) * W1 + x + 1] = es[y * W1 + x + 1] + row;
      }
    }
    this.buildSoft();
  }

  private terrainColor(i: number): string {
    const t = this.tiles[i].terrain!;
    // A lone peak or small group stands on the land around it: its hex takes the commonest open
    // neighbour's ground colour, so no grey hex shows round the painted peak (a hex-shaped sticker).
    const ground = this.loneRock(i) ? this.footTerrain(i) : t;
    return jitter(this.style.pal[ground], (rnd(i % this.w, Math.floor(i / this.w), 9) - 0.5) * 0.08);
  }

  /** A mountain tile of an organic range too small for the range wash (a lone peak or a group of up to SMALL_REGION). */
  private loneRock(i: number): boolean {
    const R = this.regions;
    if (!R || this.tiles[i].terrain !== 'Mountains') return false;
    const r = R.id[i];
    return r >= 0 && R.size[r] <= SMALL_REGION;
  }

  /** The commonest explored land terrain round a tile that is not a painted feature (ties to the first found), else Plains. */
  private footTerrain(i: number): Terrain {
    const n = new Map<Terrain, number>();
    for (let e = 0; e < 6; e++) {
      const j = neighborIndex(i, e, this.w, this.h);
      const t = j >= 0 && this.tiles[j].explored ? this.tiles[j].terrain : null;
      if (!t || isWater(t) || organicKind(t)) continue;
      n.set(t, (n.get(t) ?? 0) + 1);
    }
    let best: Terrain = 'Plains', most = 0;
    for (const [t, k] of n) if (k > most) { best = t; most = k; }
    return best;
  }

  private softPool = new Map<string, AnyCanvas>();

  /** A cleared canvas of this size under this name, reused across rebuilds. */
  private pooled(name: string, w: number, h: number): AnyCanvas {
    const wi = Math.max(1, Math.ceil(w));
    const hi = Math.max(1, Math.ceil(h));
    let c = this.softPool.get(name);
    if (!c || c.width !== wi || c.height !== hi) {
      c = makeCanvas(wi, hi);
      this.softPool.set(name, c);
    } else {
      const g = ctx2d(c);
      g.setTransform(1, 0, 0, 1, 0, 0);
      g.globalAlpha = 1;
      g.globalCompositeOperation = 'source-over';
      g.clearRect(0, 0, wi, hi);
    }
    return c;
  }

  /** The blurred layers: land color, hillshade, shallows (low resolution; they are soft anyway). */
  private buildSoft() {
    const S = this.style;
    const ox = this.bounds.minX - MARGIN;
    const oy = this.bounds.minY - MARGIN;
    const ww = this.bounds.maxX - this.bounds.minX + 2 * MARGIN;
    const wh = this.bounds.maxY - this.bounds.minY + 2 * MARGIN;
    // The soft layers are rebuilt whenever a tile changes (a scout reveals some every turn): reuse their canvases,
    // since Safari frees a dropped canvas's memory only when it collects garbage and refuses new canvases meanwhile.
    const mk = (name: string, scale: number): Soft => {
      const canvas = this.pooled(name, ww * scale, wh * scale);
      return { canvas, scale, ox, oy };
    };
    const color = mk('color', SC_COLOR);
    const relief = mk('relief', SC_SOFT);
    const shallow = mk('shallow', SC_SOFT);
    const n = this.w * this.h;
    const tiles = this.tiles;

    // Land color field: land hexes, plus water next to land and the unexplored rim painted with a
    // neighboring land color so the blur has no dark fringe.
    {
      const g = ctx2d(color.canvas);
      g.setTransform(color.scale, 0, 0, color.scale, -ox * color.scale, -oy * color.scale);
      const buckets = new Map<string, number[]>();
      for (let i = 0; i < n; i++) {
        const t = tiles[i];
        let src = -1;
        if (t.explored && !isWater(t.terrain)) src = i;
        else {
          for (let e = 0; e < 6 && src < 0; e++) {
            const nb = neighborIndex(i, e, this.w, this.h);
            if (nb >= 0 && tiles[nb].explored && !isWater(tiles[nb].terrain)) src = nb;
          }
        }
        if (src < 0) continue;
        const col = this.terrainColor(src);
        const l = buckets.get(col);
        if (l) l.push(i);
        else buckets.set(col, [i]);
      }
      for (const [col, idx] of buckets) {
        g.beginPath();
        for (const i of idx) {
          if (this.organic) wobblyHex(g, centerX(i, this.w), centerY(i, this.w), HEX_SIZE + 0.3 * K);
          else hexPath(g, centerX(i, this.w), centerY(i, this.w), HEX_SIZE + 0.6 * K);
        }
        g.fillStyle = col;
        g.fill();
      }
      blurCanvas(color.canvas, S.blend * K * color.scale);
    }

    // Hillshade: pseudo-elevation from terrain, lit from the north-west.
    {
      const elev = new Float32Array(n);
      const base: Partial<Record<Terrain, number>> = { Ocean: -0.5, Coast: -0.25, Hills: 0.55, Mountains: 1, Forest: 0.16 };
      for (let i = 0; i < n; i++) {
        const t = tiles[i];
        const b = t.terrain ? base[t.terrain] ?? 0.08 : 0.1;
        elev[i] = b + (rnd(i, 3, 17) - 0.5) * 0.14;
      }
      const sm = new Float32Array(n);
      for (let i = 0; i < n; i++) {
        let acc = elev[i] * 2;
        let wsum = 2;
        for (let e = 0; e < 6; e++) {
          const nb = neighborIndex(i, e, this.w, this.h);
          if (nb < 0) continue;
          acc += elev[nb];
          wsum++;
        }
        sm[i] = acc / wsum;
      }
      const g = ctx2d(relief.canvas);
      g.setTransform(relief.scale, 0, 0, relief.scale, -ox * relief.scale, -oy * relief.scale);
      for (let i = 0; i < n; i++) {
        const t = tiles[i];
        if (!t.explored || isWater(t.terrain)) continue;
        const nw = neighborIndex(i, 4, this.w, this.h);
        const se = neighborIndex(i, 1, this.w, this.h);
        const w = neighborIndex(i, 3, this.w, this.h);
        const e = neighborIndex(i, 0, this.w, this.h);
        const at = (j: number) => (j < 0 ? sm[i] : sm[j]);
        let s = (at(nw) - at(se)) * 1.5 + (at(w) - at(e)) * 0.5;
        s += t.terrain === 'Mountains' ? 0.25 : t.terrain === 'Hills' ? 0.12 : 0;
        s = Math.max(-1, Math.min(1, s));
        if (Math.abs(s) < 0.04) continue;
        g.globalAlpha = Math.min(1, Math.abs(s));
        g.fillStyle = s > 0 ? S.light : S.shadow;
        g.beginPath();
        hexPath(g, centerX(i, this.w), centerY(i, this.w), HEX_SIZE);
        g.fill();
      }
      g.globalAlpha = 1;
      blurCanvas(relief.canvas, 9 * K * relief.scale);
    }

    // Shallows: coast hexes, blurred into the deep.
    {
      const g = ctx2d(shallow.canvas);
      g.setTransform(shallow.scale, 0, 0, shallow.scale, -ox * shallow.scale, -oy * shallow.scale);
      g.beginPath();
      for (let i = 0; i < n; i++) {
        const t = tiles[i];
        if (t.explored && t.terrain === 'Coast') hexPath(g, centerX(i, this.w), centerY(i, this.w), HEX_SIZE * 1.05);
      }
      g.fillStyle = S.shallow;
      g.fill();
      blurCanvas(shallow.canvas, 12 * K * shallow.scale);
    }
    this.soft = { color, relief, shallow, under: this.organic ? this.buildUnderlay(mk('under', SC_SOFT)) : null };
  }

  /**
   * Organic underlay: a soft wash under each big region (canopy shade under a wood, rock under a range,
   * a little earth under rolling hills), deeper toward the interior, with a wobbly outline. Built from
   * explored tiles only, so it never shows the shape of anything unexplored.
   */
  private buildUnderlay(soft: Soft): Soft {
    const S = this.style;
    const R = this.regions;
    const g = ctx2d(soft.canvas);
    g.setTransform(soft.scale, 0, 0, soft.scale, -soft.ox * soft.scale, -soft.oy * soft.scale);
    if (R) {
      const WASH: Record<string, { col: string; a: [number, number, number] }> = {
        Forest: { col: S.tree.dark, a: [0.15, 0.25, 0.32] },
        Mountains: { col: ROCK, a: [0.28, 0.36, 0.42] },
        Hills: { col: S.hill.line, a: [0.16, 0.24, 0.3] },
      };
      for (let r = 0; r < R.kind.length; r++) {
        if (R.size[r] <= SMALL_REGION) continue;
        const wash = WASH[R.kind[r]];
        for (const d of [0, 1, 2]) {
          g.beginPath();
          let any = false;
          for (const i of R.tiles[r]) {
            if (Math.min(2, R.edge[i]) !== d) continue;
            wobblyHex(g, centerX(i, this.w), centerY(i, this.w), HEX_SIZE * (d === 0 ? 0.88 : 1.02));
            any = true;
          }
          if (!any) continue;
          g.globalAlpha = wash.a[d];
          g.fillStyle = wash.col;
          g.fill();
        }
      }
      g.globalAlpha = 1;
      blurCanvas(soft.canvas, 7 * K * soft.scale);
      // Keep the wash inside its regions (UI review: blurred, it stained the open tiles beside a wood or
      // range): clip it to the regions' own wobbly union, feathered a little so the edge stays soft.
      const mask = this.pooled('underMask', soft.canvas.width, soft.canvas.height);
      const mg = ctx2d(mask);
      mg.setTransform(soft.scale, 0, 0, soft.scale, -soft.ox * soft.scale, -soft.oy * soft.scale);
      mg.beginPath();
      for (let r = 0; r < R.kind.length; r++) if (R.size[r] > SMALL_REGION) for (const i of R.tiles[r]) wobblyHex(mg, centerX(i, this.w), centerY(i, this.w), HEX_SIZE * 0.92);
      mg.fillStyle = '#000';
      mg.fill();
      blurCanvas(mask, 3 * K * soft.scale);
      g.setTransform(1, 0, 0, 1, 0, 0);
      g.globalCompositeOperation = 'destination-in';
      g.drawImage(mask as CanvasImageSource, 0, 0);
      g.globalCompositeOperation = 'source-over';
    }
    return soft;
  }

  // ---------------------------------------------------------------- frame

  /** Pick the cache level for an effective scale (device px per world px). */
  static levelFor(eff: number): number {
    for (let i = 0; i < LEVELS.length; i++) if (LEVELS[i] >= eff * 0.97) return i;
    return LEVELS.length - 1;
  }

  /**
   * Blit cached chunks for the camera (device-pixel transform must be identity * dpr handled here)
   * and render missing or stale ones within the time budget. Returns true while work remains.
   */
  draw(ctx: CanvasRenderingContext2D, cam: FrameCam, w: number, h: number, budgetMs: number): boolean {
    this.frame++;
    // City sprites finished loading: a chunk that drew a town without them repaints once, when the last sprite it waited
    // for is in; no other chunk does. (A pan or zoom loads sprites for a while, and repainting every chunk on each arrival
    // kept a big view busy for tens of seconds on a slow machine.)
    const artEpoch = cityArtEpoch();
    if (artEpoch !== this.artEpoch) {
      const arrived = cityArtArrivals(this.artEpoch);
      this.artEpoch = artEpoch;
      this.epochs++;
      for (const ch of this.chunks.values()) {
        if (!ch.awaits || ch.stale) continue;
        for (const key of arrived) ch.awaits.delete(key);
        if (!ch.awaits.size) ch.stale = ch.byArt = true;
      }
    }
    const dpr = this.dpr;
    const eff = cam.zoom * dpr;
    const t0 = performance.now();
    let pending = false;
    if (cam.zoom !== this.lastZoom) {
      this.lastZoom = cam.zoom;
      this.zoomChangedAt = t0;
    }
    const wanted = TerrainCache.levelFor(eff);
    let level = wanted;
    // Hold the level in use through a zoom gesture, while its chunks still scale to the view without looking
    // wrong. Past that range, advance one level at a time and only once the held level is complete, so the
    // level left behind is always a close, finished stand-in for the one being drawn.
    if (wanted !== this.curLevel && this.curLevel >= 0 && t0 - this.zoomChangedAt < ZOOM_HOLD_MS) {
      const ratio = eff / LEVELS[this.curLevel];
      if (ratio > HOLD_SCALE_MIN && ratio < HOLD_SCALE_MAX) level = this.curLevel;
      else if (this.lastStats.missing > 0) level = this.curLevel;
      else level = this.curLevel + Math.sign(wanted - this.curLevel);
      pending = true; // come back once the zoom rests, to settle on the right level
    }
    this.curLevel = level;

    // World rect on screen.
    const wx0 = cam.x - w / 2 / cam.zoom;
    const wy0 = cam.y - h / 2 / cam.zoom;
    const wx1 = cam.x + w / 2 / cam.zoom;
    const wy1 = cam.y + h / 2 / cam.zoom;

    const need = (lv: number, pad: number) => {
      const cw = CHUNK / scaleOf(lv);
      const out: [number, number][] = [];
      // The stand-in layer is the whole map (a handful of chunks), wherever the camera is, so a jump never lands on a blank view.
      if (lv === STANDIN) {
        const b = this.bounds;
        for (let cy = Math.floor(b.minY / cw); cy <= Math.floor(b.maxY / cw); cy++)
          for (let cx = Math.floor(b.minX / cw); cx <= Math.floor(b.maxX / cw); cx++) out.push([cx, cy]);
        return out;
      }
      for (let cy = Math.floor(wy0 / cw) - pad; cy <= Math.floor(wy1 / cw) + pad; cy++)
        for (let cx = Math.floor(wx0 / cw) - pad; cx <= Math.floor(wx1 / cw) + pad; cx++) out.push([cx, cy]);
      return out;
    };

    // Work: overview first (the fallback), then visible chunks nearest the center, then a ring.
    const work = (lv: number, pad: number, force: boolean, budget: number) => {
      const cells = need(lv, pad);
      const cw = CHUNK / scaleOf(lv);
      cells.sort(
        (a, b) =>
          Math.hypot((a[0] + 0.5) * cw - cam.x, (a[1] + 0.5) * cw - cam.y) -
          Math.hypot((b[0] + 0.5) * cw - cam.x, (b[1] + 0.5) * cw - cam.y),
      );
      for (const [cx, cy] of cells) {
        const ch = this.chunks.get(`${lv}:${cx}:${cy}`);
        if (ch && !ch.stale) continue;
        if (performance.now() - t0 > budget && !(force && !ch)) {
          pending = true;
          return;
        }
        const key = `${lv}:${cx}:${cy}`;
        if ((this.failedAt.get(key) ?? -Infinity) > this.frame - RETRY_FRAMES) continue;
        try {
          this.renderChunk(lv, cx, cy);
          // Every chunk is opaque, so a transparent centre means the platform did not give the canvas a backing store.
          // A readback stalls the GPU, so only every sixth chunk is sampled; a blank one audits the rest of the level.
          if (this.renders % 6 === 0 && this.isBlank(this.chunks.get(key)!)) {
            this.auditLevel = lv;
            throw new Error('blank chunk');
          }
          this.failedAt.delete(key);
        } catch (e) {
          // Never leave a half-drawn or blank chunk marked as drawn: drop it, free memory, and try again shortly.
          this.dropChunk(key);
          this.failedAt.set(key, this.frame);
          this.failures++;
          pending = true;
          if (this.failures <= 3) console.warn('legatum: terrain chunk', key, 'failed:', e);
          continue;
        }
        force = false;
      }
    };
    // The stand-in layer first (so a view is never blank), then the view. A visible chunk that was never
    // drawn is on a stand-in until it is, so those fill in faster than a refresh.
    const visible = need(level, 0);
    const unseen = visible.some(([cx, cy]) => !this.chunks.has(`${level}:${cx}:${cy}`));
    if (level !== OVERVIEW) work(STANDIN, 0, true, need(STANDIN, 0).some(([cx, cy]) => !this.chunks.has(`${STANDIN}:${cx}:${cy}`)) ? Math.max(budgetMs, FILL_BUDGET_MS) : budgetMs);
    work(level, 0, true, unseen ? Math.max(budgetMs, FILL_BUDGET_MS) : budgetMs);
    if (this.auditLevel >= 0) {
      for (const [key, ch] of [...this.chunks]) if (ch.level === this.auditLevel && !ch.stale && this.isBlank(ch)) this.dropChunk(key);
      this.auditLevel = -1;
    }
    // Zooming out reveals ground no chunk covers yet, so a gesture pre-renders a wider ring.
    const gesture = t0 - this.zoomChangedAt < ZOOM_HOLD_MS;
    if (performance.now() - t0 < budgetMs * 0.6) work(level, gesture ? 2 : 1, false, budgetMs);
    else pending = true;

    // Blit: the stand-ins (the stand-in layer, then any other cached level close in scale) only while a visible
    // chunk of the current level is missing, then the current level.
    ctx.save();
    ctx.setTransform(1, 0, 0, 1, 0, 0);
    ctx.imageSmoothingEnabled = true;
    ctx.fillStyle = this.style.unexplored;
    ctx.fillRect(0, 0, w * dpr, h * dpr);
    // World rects of the visible chunks still missing: the stand-in chunks that cover them are in use (kept resident), the rest are not.
    const gapRects: [number, number, number, number][] = [];
    const blit = (lv: number, standIn: boolean) => {
      const s = scaleOf(lv);
      const cw = CHUNK / s;
      const sx = (x: number) => Math.round(((x - cam.x) * cam.zoom + w / 2) * dpr);
      const sy = (y: number) => Math.round(((y - cam.y) * cam.zoom + h / 2) * dpr);
      for (const [cx, cy] of need(lv, 0)) {
        const ch = this.chunks.get(`${lv}:${cx}:${cy}`);
        // A previous-level chunk from an earlier turn would show borders and fog as they were, so it is skipped
        // (the stand-in layer is refreshed every turn and is better than nothing even for a frame or two).
        if (!ch || (standIn && ch.stale && lv !== STANDIN)) continue;
        // Stand-ins count as use only where they cover a gap, or a zoom-out would pin its whole old level and push the current one out.
        if (!standIn) ch.used = this.frame;
        else if (lv !== STANDIN && gapRects.some((r) => cx * cw < r[2] && (cx + 1) * cw > r[0] && cy * cw < r[3] && (cy + 1) * cw > r[1])) ch.used = this.frame;
        const x0 = sx(cx * cw);
        const y0 = sy(cy * cw);
        ctx.drawImage(ch.canvas as CanvasImageSource, x0, y0, sx((cx + 1) * cw) - x0, sy((cy + 1) * cw) - y0);
      }
    };
    const gaps = visible.filter(([cx, cy]) => !this.chunks.has(`${level}:${cx}:${cy}`));
    let exposed = 0;
    if (gaps.length) {
      const cwg = CHUNK / scaleOf(level);
      for (const [cx, cy] of gaps) gapRects.push([cx * cwg, cy * cwg, (cx + 1) * cwg, (cy + 1) * cwg]);
      const present = new Set<number>();
      for (const c of this.chunks.values()) if (c.level !== level && !c.stale) present.add(c.level);
      const layers = fallbackLevels(level, present, OVERVIEW, STANDIN);
      for (const lv of layers) blit(lv, true);
      // Gaps with no real chunk of any other level under them show only the soft stand-in layer.
      const cwv = CHUNK / scaleOf(level);
      for (const [cx, cy] of gaps) {
        const x = (cx + 0.5) * cwv;
        const y = (cy + 0.5) * cwv;
        const covered = layers.some((lv) => lv !== STANDIN && this.chunks.has(`${lv}:${Math.floor(x / (CHUNK / scaleOf(lv)))}:${Math.floor(y / (CHUNK / scaleOf(lv)))}`));
        if (!covered) exposed++;
      }
    }
    blit(level, false);
    ctx.restore();

    // Drop the least recently used chunks. The current view (its chunks and the ring) is never dropped
    // for older ones, and the limit grows with the view, so a big window cannot evict what it shows.
    const ring = need(level, gesture ? 2 : 1);
    const pinned = new Set<string>(ring.map(([cx, cy]) => `${level}:${cx}:${cy}`));
    if (this.chunks.size > cacheLimit(pinned.size)) {
      const entries = [...this.chunks.entries()].map(([key, c]) => ({ key, level: c.level, used: c.used, pinned: pinned.has(key) }));
      for (const key of evictionPlan(entries, cacheLimit(pinned.size), STANDIN)) { this.dropChunk(key); this.evictions++; }
    }
    let missing = 0;
    for (const lv of new Set(level === OVERVIEW ? [level] : [STANDIN, level]))
      for (const [cx, cy] of need(lv, 0)) {
        const ch = this.chunks.get(`${lv}:${cx}:${cy}`);
        if (!ch || ch.stale) {
          pending = true;
          missing++;
        }
      }
    this.lastWorkMs = performance.now() - t0;
    let stale = 0;
    for (const c of this.chunks.values()) if (c.stale) stale++;
    this.lastStats = { chunks: this.chunks.size, renders: this.renders, missing, visible: visible.length, exposed, level, wanted, failures: this.failures, pinned: pinned.size, limit: cacheLimit(pinned.size), stale, evictions: this.evictions, workMs: Math.round(this.lastWorkMs), epochs: this.epochs, artRenders: this.artRenders };
    return pending;
  }

  // ---------------------------------------------------------------- chunk rendering

  private isBlank(ch: Chunk): boolean {
    return ctx2d(ch.canvas).getImageData(CHUNK >> 1, CHUNK >> 1, 1, 1).data[3] < 255;
  }

  /** Forget a chunk and give its canvas memory back at once (Safari holds it until a garbage collection otherwise). */
  private dropChunk(key: string) {
    const ch = this.chunks.get(key);
    if (!ch) return;
    this.chunks.delete(key);
    ch.canvas.width = 0;
    ch.canvas.height = 0;
  }

  private pattern(g: Ctx2D, key: string, make: () => AnyCanvas): CanvasPattern | null {
    if (!this.patterns.has(key)) this.patterns.set(key, g.createPattern(make() as CanvasImageSource, 'repeat'));
    return this.patterns.get(key)!;
  }

  private scratchCanvases() {
    if (!this.scratch)
      this.scratch = {
        land: makeCanvas(CHUNK, CHUNK),
        mask: makeCanvas(CHUNK, CHUNK),
        lines: makeCanvas(CHUNK, CHUNK),
        band: makeCanvas(CHUNK + 2 * PAD, CHUNK + 2 * PAD),
      };
    return this.scratch;
  }

  private drawSoft(g: Ctx2D, soft: Soft, x0: number, y0: number, cw: number) {
    const k = soft.scale;
    const pad = 2 / k;
    const sx = (x0 - pad - soft.ox) * k;
    const sy = (y0 - pad - soft.oy) * k;
    const sw = (cw + 2 * pad) * k;
    const cx = Math.max(0, sx);
    const cy = Math.max(0, sy);
    const cx1 = Math.min(soft.canvas.width, sx + sw);
    const cy1 = Math.min(soft.canvas.height, sy + sw);
    if (cx1 <= cx || cy1 <= cy) return;
    g.drawImage(soft.canvas as CanvasImageSource, cx, cy, cx1 - cx, cy1 - cy, cx / k + soft.ox, cy / k + soft.oy, (cx1 - cx) / k, (cy1 - cy) / k);
  }

  private renderChunk(lv: number, cx: number, cy: number) {
    const key = `${lv}:${cx}:${cy}`;
    let ch = this.chunks.get(key);
    if (!ch) {
      ch = { canvas: makeCanvas(CHUNK, CHUNK), level: lv, cx, cy, stale: false, used: this.frame, awaits: null, byArt: false };
      this.chunks.set(key, ch);
    }
    if (ch.byArt) this.artRenders++;
    ch.stale = false;
    ch.byArt = false;
    ch.awaits = null;
    ch.used = this.frame;
    this.renders++;
    const s = scaleOf(lv);
    const cw = CHUNK / s;
    const x0 = cx * cw;
    const y0 = cy * cw;
    const g = ctx2d(ch.canvas);
    g.setTransform(1, 0, 0, 1, 0, 0);
    g.globalAlpha = 1;
    g.globalCompositeOperation = 'source-over';
    g.clearRect(0, 0, CHUNK, CHUNK);
    const world = (c: Ctx2D) => c.setTransform(s, 0, 0, s, -x0 * s, -y0 * s);
    const S = this.style;
    const b = this.bounds;
    const off = x0 + cw < b.minX - HEX_SIZE || y0 + cw < b.minY - HEX_SIZE || x0 > b.maxX + HEX_SIZE || y0 > b.maxY + HEX_SIZE;
    // Feature sprites reach a little past the hex, so the dev terrain variants gather a wider ring.
    const m = getTerrainVariant() === 'current' ? HEX_SIZE : this.organic ? HEX_SIZE + ORGANIC_REACH : HEX_SIZE * 1.4;
    const tiles = off ? [] : tilesInRect(x0 - m, y0 - m, x0 + cw + m, y0 + cw + m, this.w, this.h);
    const anyExplored = tiles.some((i) => this.tiles[i].explored);
    // CSS px per world px this chunk is drawn for (the stand-in pretends to be seen at a typical zoom).
    const zoom = lv === STANDIN ? STANDIN_ZOOM : s / this.dpr;
    this.chunkZoom = zoom;
    const px = 1 / zoom; // one CSS pixel in world units

    if (anyExplored) {
      world(g);
      // 1. Deep water and its mottle.
      g.fillStyle = S.deep;
      g.fillRect(x0, y0, cw, cw);
      const mot = this.pattern(g, 'mottle', mottleCanvas);
      if (mot) {
        mot.setTransform(new DOMMatrix([4, 0, 0, 4, 0, 0]));
        g.globalCompositeOperation = S.mottleBlend;
        g.globalAlpha = S.seaMottle;
        g.fillStyle = mot;
        g.fillRect(x0, y0, cw, cw);
        g.globalCompositeOperation = 'source-over';
      }
      // 2. Shallows.
      g.globalAlpha = 0.9;
      this.drawSoft(g, this.soft!.shallow, x0, y0, cw);
      g.globalAlpha = 1;

      // 3. Coastline mask, coast line and wave rings, per pixel from the field.
      const sc = this.scratchCanvases();
      this.coastPixels(sc.mask, sc.lines, x0, y0, s);
      g.setTransform(1, 0, 0, 1, 0, 0);
      g.drawImage(sc.lines as CanvasImageSource, 0, 0);

      // 4-6. Land: color field, relief, mottle, grain, clipped to the mask.
      const lg = ctx2d(sc.land);
      lg.setTransform(1, 0, 0, 1, 0, 0);
      lg.globalCompositeOperation = 'source-over';
      lg.globalAlpha = 1;
      lg.clearRect(0, 0, CHUNK, CHUNK);
      world(lg);
      this.drawSoft(lg, this.soft!.color, x0, y0, cw);
      lg.globalAlpha = S.relief;
      this.drawSoft(lg, this.soft!.relief, x0, y0, cw);
      const lmot = this.pattern(lg, 'mottle', mottleCanvas);
      if (lmot) {
        lmot.setTransform(new DOMMatrix([4, 0, 0, 4, 0, 0]));
        lg.globalCompositeOperation = S.mottleBlend;
        lg.globalAlpha = S.mottle;
        lg.fillStyle = lmot;
        lg.fillRect(x0, y0, cw, cw);
      }
      lg.setTransform(1, 0, 0, 1, 0, 0);
      const grain = this.pattern(lg, 'grain', grainCanvas);
      if (grain) {
        lg.globalCompositeOperation = 'source-over';
        lg.globalAlpha = S.grain;
        lg.fillStyle = grain;
        lg.fillRect(0, 0, CHUNK, CHUNK);
      }
      if (this.soft!.under) {
        lg.setTransform(s, 0, 0, s, -x0 * s, -y0 * s);
        lg.globalCompositeOperation = 'multiply';
        // Zoomed out the wash steps back so the far marks keep their contrast.
        lg.globalAlpha = (isFar(HEX_SIZE * zoom) ? 0.5 : 1) * this.artFade;
        this.drawSoft(lg, this.soft!.under, x0, y0, cw);
        lg.setTransform(1, 0, 0, 1, 0, 0);
      }
      lg.globalAlpha = 1;
      lg.globalCompositeOperation = 'destination-in';
      lg.drawImage(sc.mask as CanvasImageSource, 0, 0);
      lg.globalCompositeOperation = 'source-over';
      g.drawImage(sc.land as CanvasImageSource, 0, 0);

      world(g);
      // 7. Grid: each shared edge once, land edges darker than sea edges. Organic terrain draws it over
      // its art instead (a faint overlay, now the only grid line), so tiles still read inside a wood.
      if (!this.plans) this.drawGrid(g, tiles, px);
      // 8-9. Glyphs and towns.
      this.drawGlyphs(g, tiles, s);
      if (this.plans) {
        this.drawOrganic(g, tiles, s);
        this.drawGrid(g, tiles, px);
      }
      // Remembered towns sit under the fog wash with their land; towns in sight are drawn after the territory tint (below).
      const waiting = traceCityArt(() => this.drawTowns(g, tiles, s, false));
      // 10. Fog: desaturate and wash explored-but-unseen hexes.
      const fog = new Path2D();
      let anyFog = false;
      for (const i of tiles) {
        const t = this.tiles[i];
        if (t.explored && !t.visible) {
          hexPath(fog, centerX(i, this.w), centerY(i, this.w), HEX_SIZE + 0.5 * K);
          anyFog = true;
        }
      }
      if (anyFog) {
        g.globalCompositeOperation = 'saturation';
        g.globalAlpha = S.fog.desat;
        g.fillStyle = '#808080';
        g.fill(fog);
        g.globalCompositeOperation = S.fog.blend;
        g.globalAlpha = S.fog.alpha;
        g.fillStyle = S.fog.tint;
        g.fill(fog);
        g.globalCompositeOperation = 'source-over';
        g.globalAlpha = 1;
      }
      // 11. Current territory above the fog tint; the later hatch still marks unseen land.
      this.drawTerritory(g, tiles, x0, y0, s, px);
      // Towns in sight above the territory tint: under it a town came out paler than the farms round it (improvements are
      // drawn later, in screen space), and at a 0.18 seat-colour tint a grown city read as a few grey roofs.
      for (const k of traceCityArt(() => this.drawTowns(g, tiles, s, true))) waiting.add(k);
      ch.awaits = waiting.size ? waiting : null;
      // 12. Fog hatch and the edge of current vision.
      if (anyFog) {
        const hatch = this.pattern(g, `hatch:${S.fog.hatch}`, () => hatchTile(S.fog.hatch, Math.round(6 * this.dpr), 1));
        if (hatch) {
          hatch.setTransform(new DOMMatrix([1 / s, 0, 0, 1 / s, 0, 0]));
          g.globalAlpha = S.fog.hatchAlpha;
          g.fillStyle = hatch;
          g.fill(fog);
          g.globalAlpha = 1;
        }
        g.beginPath();
        this.edges(g, tiles, (i) => this.tiles[i].visible, (nb) => nb >= 0 && this.tiles[nb].explored);
        g.strokeStyle = S.fog.edge;
        g.lineWidth = px;
        g.stroke();
      }
    }

    // 13. The unexplored veil (and everything off the map): the chunk minus explored hexes.
    {
      const sc = this.scratchCanvases();
      const bg = ctx2d(sc.band);
      bg.setTransform(1, 0, 0, 1, 0, 0);
      bg.globalCompositeOperation = 'source-over';
      bg.globalAlpha = 1;
      bg.clearRect(0, 0, CHUNK + 2 * PAD, CHUNK + 2 * PAD);
      bg.setTransform(s, 0, 0, s, -x0 * s + PAD, -y0 * s + PAD);
      const veil = new Path2D();
      const m = (PAD + 8) / s;
      veil.rect(x0 - m, y0 - m, cw + 2 * m, cw + 2 * m);
      // Classic: each explored hex is a hole slightly smaller than the hex, so a thin sliver of veil stays
      // between explored hexes and, once blurred, rules a soft pale line along every hex edge over the art.
      // Organic terrain removes the explored hexes as one overlapping union instead (overlapping holes in
      // one path would fill again), so a wood or a range is not ruled into tiles; only the faint grid line,
      // drawn under the art, remains.
      if (!this.organic) for (const i of tiles) if (this.tiles[i].explored) hexPath(veil, centerX(i, this.w), centerY(i, this.w), HEX_SIZE - 0.6, true);
      bg.fillStyle = S.unexplored;
      bg.fill(veil, 'nonzero');
      const unx = this.pattern(bg, `unx:${S.id}`, () => unexploredTile(S, this.dpr));
      if (unx) {
        unx.setTransform(new DOMMatrix([1 / s, 0, 0, 1 / s, 0, 0]));
        bg.fillStyle = unx;
        bg.fill(veil, 'nonzero');
      }
      if (this.organic) {
        const known = new Path2D();
        for (const i of tiles) if (this.tiles[i].explored) hexPath(known, centerX(i, this.w), centerY(i, this.w), HEX_SIZE + 0.6);
        bg.globalCompositeOperation = 'destination-out';
        bg.fillStyle = '#000';
        bg.fill(known, 'nonzero');
        bg.globalCompositeOperation = 'source-over';
      }
      if (anyExplored && canvasFilterSupported()) blurCanvas(sc.band, S.unexploredBlur * this.dpr);
      g.setTransform(1, 0, 0, 1, 0, 0);
      g.drawImage(sc.band as CanvasImageSource, PAD, PAD, CHUNK, CHUNK, 0, 0, CHUNK, CHUNK);
    }
    if (anyExplored) {
      // 14. A crisp line where the known world meets the unknown.
      world(g);
      g.beginPath();
      this.edges(g, tiles, (i) => this.tiles[i].explored, () => true);
      g.strokeStyle = S.unexploredEdge;
      g.lineWidth = px;
      g.stroke();
    }
    // Read a pixel back so the canvas finishes its pending work now. The Mac's WKWebView (Metal, OffscreenCanvas) defers
    // canvas operations, and a chunk is built from scratch canvases (land, mask, coast lines, band, blur temp) that the next
    // chunk overwrites in the same frame: unless this chunk's deferred draws have run first, it shows a rectangle of the
    // next chunk's coast, water on land (playtest 8.9.4: "random water elements where they shouldn't be", T36). Without the
    // readback about half of the fresh loads in real WebKit on a Mac show it, with it none of 11 did (tools/ui/webkit-mac).
    if (flushDeferredChunkDraws) g.getImageData(0, 0, 1, 1);
  }

  /** Adds boundary edges of tiles where `inside` holds and the neighbor does not (and `want(nb)`). */
  private edges(g: Ctx2D, tiles: number[], inside: (i: number) => boolean, want: (nb: number) => boolean) {
    for (const i of tiles) {
      if (!inside(i)) continue;
      const cx = centerX(i, this.w);
      const cy = centerY(i, this.w);
      for (let e = 0; e < 6; e++) {
        const nb = neighborIndex(i, e, this.w, this.h);
        if (nb >= 0 && inside(nb)) continue;
        if (!want(nb)) continue;
        const a = CORNERS[e];
        const b = CORNERS[(e + 1) % 6];
        g.moveTo(cx + a[0] * HEX_SIZE, cy + a[1] * HEX_SIZE);
        g.lineTo(cx + b[0] * HEX_SIZE, cy + b[1] * HEX_SIZE);
      }
    }
  }

  private coastPixels(maskC: AnyCanvas, linesC: AnyCanvas, x0: number, y0: number, s: number) {
    const mg = ctx2d(maskC);
    const lgc = ctx2d(linesC);
    const mask = mg.createImageData(CHUNK, CHUNK);
    const lines = lgc.createImageData(CHUNK, CHUNK);
    const m32 = new Uint32Array(mask.data.buffer);
    const l8 = lines.data;
    const S = this.style;
    const [dx, dy] = displacementTiles();
    const f = this.field;
    const fw = this.fw;
    const fh = this.fh;
    const fx0 = this.fx0;
    const fy0 = this.fy0;
    const es = this.edgeSum;
    const W1 = fw + 1;
    const inv = 1 / s;
    const SIG = 6.5 * K;
    const P0 = probit(0.506);
    const DSC = 11 * K;
    const coastW = Math.max(S.coastWidth * K * s, 0.9);
    const r1a = 5 * K * s;
    const r1b = Math.max(5.8 * K * s, r1a + 0.6);
    const r2a = 10 * K * s;
    const r2b = Math.max(10.7 * K * s, r2a + 0.5);
    const [cr, cg, cb] = parseHex(S.coast);
    const [wr, wg, wb] = parseHex(S.waveLine);
    const wA = S.waveAlpha;
    const WHITE = 0xffffffff;
    const fieldAt = (x: number, y: number) => {
      const gx = (x - fx0) / G;
      const gy = (y - fy0) / G;
      let xi = Math.floor(gx);
      let yi = Math.floor(gy);
      let tx = gx - xi;
      let ty = gy - yi;
      if (xi < 0) {
        xi = 0;
        tx = 0;
      } else if (xi >= fw - 1) {
        xi = fw - 2;
        tx = 1;
      }
      if (yi < 0) {
        yi = 0;
        ty = 0;
      } else if (yi >= fh - 1) {
        yi = fh - 2;
        ty = 1;
      }
      const o = yi * fw + xi;
      const a = f[o];
      const b = f[o + 1];
      const c = f[o + fw];
      const d = f[o + fw + 1];
      return a + (b - a) * tx + (c - a) * ty + (a - b - c + d) * tx * ty;
    };
    const B = 16;
    const clamp01 = (v: number) => (v < 0 ? 0 : v > 1 ? 1 : v);
    for (let by = 0; by < CHUNK; by += B) {
      for (let bx = 0; bx < CHUNK; bx += B) {
        const wxa = x0 + bx * inv;
        const wya = y0 + by * inv;
        const wxb = x0 + (bx + B) * inv;
        const wyb = y0 + (by + B) * inv;
        const ga = Math.max(0, Math.floor((wxa - fx0) / G) - 4);
        const gb = Math.min(fw, Math.ceil((wxb - fx0) / G) + 4);
        const ha = Math.max(0, Math.floor((wya - fy0) / G) - 4);
        const hb = Math.min(fh, Math.ceil((wyb - fy0) / G) + 4);
        const edgeCells = gb > ga && hb > ha ? es[hb * W1 + gb] - es[ha * W1 + gb] - es[hb * W1 + ga] + es[ha * W1 + ga] : 0;
        if (edgeCells === 0) {
          const full = fieldAt((wxa + wxb) / 2, (wya + wyb) / 2) > 0.5 ? WHITE : 0;
          if (full)
            for (let y = by; y < by + B; y++) m32.fill(full, y * CHUNK + bx, y * CHUNK + bx + B);
          continue;
        }
        for (let y = by; y < by + B; y++) {
          const wy = y0 + (y + 0.5) * inv;
          for (let x = bx; x < bx + B; x++) {
            const wx = x0 + (x + 0.5) * inv;
            const ox = (sampleTile(dx, wx / 2, wy / 2) - 0.5) * DSC;
            const oy = (sampleTile(dy, wx / 2, wy / 2) - 0.5) * DSC;
            const a = fieldAt(wx + ox, wy + oy);
            const ds = SIG * (probit(a) - P0) * s; // device px, positive on land
            const mA = clamp01(0.5 + ds);
            const k = y * CHUNK + x;
            if (mA > 0) {
              const v = Math.round(mA * 255);
              m32[k] = (v << 24) | 0x00ffffff;
            }
            if (ds > 1) continue;
            const coast = clamp01(0.5 + ds + coastW) - mA;
            const ring = (clamp01(0.5 + ds + r1b) - clamp01(0.5 + ds + r1a)) * wA + (clamp01(0.5 + ds + r2b) - clamp01(0.5 + ds + r2a)) * wA * 0.5;
            const al = coast + (1 - coast) * ring;
            if (al <= 0.004) continue;
            const wc = ((1 - coast) * ring) / al;
            const o = 4 * k;
            l8[o] = cr + (wr - cr) * wc;
            l8[o + 1] = cg + (wg - cg) * wc;
            l8[o + 2] = cb + (wb - cb) * wc;
            l8[o + 3] = al * 255;
          }
        }
      }
    }
    mg.putImageData(mask, 0, 0);
    lgc.putImageData(lines, 0, 0);
  }

  private drawGrid(g: Ctx2D, tiles: number[], px: number) {
    const land = new Path2D();
    const water = new Path2D();
    // Organic terrain: an edge touching a wood, hill or range also gets a cased line (a pale casing under an ink hairline), so tiles
    // stay countable inside big woods and ranges where the plain line sinks into the art; open ground keeps the faint line alone.
    const art = new Path2D();
    for (const i of tiles) {
      const t = this.tiles[i];
      if (!t.explored) continue;
      const cx = centerX(i, this.w);
      const cy = centerY(i, this.w);
      for (let e = 0; e < 3; e++) {
        const nb = neighborIndex(i, e, this.w, this.h);
        if (nb < 0 || !this.tiles[nb].explored) continue;
        const p = isWater(t.terrain) && isWater(this.tiles[nb].terrain) ? water : land;
        const a = CORNERS[e];
        const b = CORNERS[(e + 1) % 6];
        const x0 = cx + a[0] * HEX_SIZE, y0 = cy + a[1] * HEX_SIZE, x1 = cx + b[0] * HEX_SIZE, y1 = cy + b[1] * HEX_SIZE;
        p.moveTo(x0, y0);
        p.lineTo(x1, y1);
        if (this.plans && this.casedEdge(i, nb)) { art.moveTo(x0, y0); art.lineTo(x1, y1); }
      }
    }
    g.lineWidth = px;
    g.strokeStyle = this.style.gridWater;
    g.stroke(water);
    g.strokeStyle = this.style.grid;
    g.stroke(land);
    if (this.plans) {
      const a = this.style.gridArt;
      g.lineCap = 'round';
      g.lineWidth = px * a.casing;
      g.strokeStyle = a.light;
      g.stroke(art);
      g.lineWidth = px * a.core;
      g.strokeStyle = a.dark;
      g.stroke(art);
      g.lineCap = 'butt';
    }
  }

  /** Whether a shared edge gets the cased grid line: it touches a wood, hills or a range, except where a lone peak or small
   * group meets open ground (there the peak stands clear of its edge, and a full cased outline cut it out as a hex sticker). */
  private casedEdge(i: number, j: number): boolean {
    const a = organicKind(this.tiles[i].terrain), b = organicKind(this.tiles[j].terrain);
    if (!a && !b) return false;
    if (a && b) return true;
    return !this.loneRock(a ? i : j);
  }

  private drawGlyphs(g: Ctx2D, tiles: number[], s: number) {
    const S = this.style;
    const detail = this.chunkZoom > 0.45;
    g.lineCap = 'round';
    g.lineJoin = 'round';
    for (const i of tiles) {
      const t = this.tiles[i];
      if (!t.explored || !t.terrain || this.cityAt.has(i)) continue;
      if (this.plans?.near.covered[i]) continue;
      const c = i % this.w;
      const r = (i - c) / this.w;
      const rr = (k: number) => rnd(c, r, k);
      const op = t.visible ? 1 : S.fog.glyph;
      g.save();
      g.translate(centerX(i, this.w), centerY(i, this.w));
      g.scale(K, K);
      g.globalAlpha = op;
      const R = 27;
      const art = getTerrainVariant() !== 'current' && (t.terrain === 'Forest' || t.terrain === 'Hills' || t.terrain === 'Mountains');
      if (art && drawFeatureArt(g, {
        kind: t.terrain as 'Forest' | 'Hills' | 'Mountains', c, r, zoom: this.chunkZoom, devPerWorld: s, alpha: this.artFade,
        tundra: t.terrain === 'Forest' && [0, 1, 2, 3, 4, 5].some((e) => { const j = neighborIndex(i, e, this.w, this.h); return j >= 0 && this.tiles[j].terrain === 'Tundra'; }),
      })) { g.restore(); continue; }
      switch (t.terrain) {
        case 'Forest': {
          const n = 6 + Math.floor(rr(1) * 3);
          const trees: [number, number, number][] = [];
          for (let k = 0; k < n; k++) {
            const a = rr(10 + k) * Math.PI * 2;
            const d = Math.sqrt(rr(20 + k)) * R * 0.62;
            trees.push([Math.cos(a) * d, Math.sin(a) * d * 0.85, 3.6 + rr(30 + k) * 2.2]);
          }
          trees.sort((a, b) => a[1] - b[1]);
          g.fillStyle = S.tree.shadow;
          g.beginPath();
          for (const [x, y, tr] of trees) {
            g.moveTo(x + 1.6 + tr, y + 2.2);
            g.ellipse(x + 1.6, y + 2.2, tr, tr * 0.7, 0, 0, Math.PI * 2);
          }
          g.fill();
          for (const [x, y, tr] of trees) {
            g.fillStyle = S.tree.dark;
            g.beginPath();
            g.arc(x, y, tr, 0, Math.PI * 2);
            g.fill();
            g.fillStyle = S.tree.light;
            g.beginPath();
            g.arc(x - tr * 0.3, y - tr * 0.35, tr * 0.55, 0, Math.PI * 2);
            g.fill();
          }
          break;
        }
        case 'Mountains': {
          const peaks: [number, number, number][] = rr(2) < 0.5 ? [[0, 2, 1]] : [[-7, 5, 0.72], [5, 1, 1]];
          for (const [ox, oy, sc] of peaks) {
            const px = ox;
            const py = oy + 8;
            const w = R * 0.78 * sc;
            const h = R * 0.95 * sc;
            const ax = px + (rr(3) - 0.5) * 4;
            const ay = py - h;
            const mx = px + w * 0.08;
            const ridge: [number, number][] = [
              [ax, ay],
              [ax + w * 0.18, ay + h * 0.32],
              [mx + w * 0.1, py - h * 0.3],
              [mx, py],
            ];
            g.fillStyle = S.peak.base;
            g.beginPath();
            g.ellipse(px + 3, py + 1, w * 1.02, 3.2 * sc, 0, 0, Math.PI * 2);
            g.fill();
            g.fillStyle = S.peak.light;
            g.beginPath();
            g.moveTo(px - w, py);
            for (const [x, y] of ridge) g.lineTo(x, y);
            g.closePath();
            g.fill();
            g.fillStyle = S.peak.dark;
            g.beginPath();
            g.moveTo(ridge[0][0], ridge[0][1]);
            for (const [x, y] of ridge) g.lineTo(x, y);
            g.lineTo(px + w, py);
            g.closePath();
            g.fill();
            g.fillStyle = S.peak.snow;
            g.beginPath();
            g.moveTo(ax, ay);
            g.lineTo(ax - w * 0.3, ay + h * 0.32);
            g.lineTo(ax - w * 0.1, ay + h * 0.26);
            g.lineTo(ax + w * 0.02, ay + h * 0.36);
            g.lineTo(ax + w * 0.18, ay + h * 0.32);
            g.closePath();
            g.fill();
            g.strokeStyle = S.peak.line;
            g.lineWidth = 0.9;
            g.beginPath();
            g.moveTo(px - w, py);
            g.lineTo(ax, ay);
            g.lineTo(ax + w * 0.18, ay + h * 0.32);
            g.lineTo(px + w, py);
            g.stroke();
            if (detail) {
              g.lineWidth = 0.7;
              g.globalAlpha = op * 0.55;
              g.beginPath();
              for (let k = 0; k < 4; k++) {
                const tt = 0.3 + k * 0.16;
                const hx = ax + (px + w - ax) * tt;
                const hy = ay + (py - ay) * tt;
                g.moveTo(hx, hy);
                g.lineTo(hx - w * 0.14, hy + h * 0.1);
              }
              g.stroke();
              g.globalAlpha = op;
            }
          }
          break;
        }
        case 'Hills': {
          const hs: [number, number, number][] = [[-8, 6, 0.9], [7, 3, 1.05]];
          if (rr(4) < 0.5) hs.push([-1, -9, 0.75]);
          for (const [ox, oy, sc] of hs) {
            const w = 10 * sc;
            const h = 7 * sc;
            g.beginPath();
            g.moveTo(ox - w, oy);
            g.quadraticCurveTo(ox - w * 0.2, oy - h * 1.5, ox + w, oy);
            g.fillStyle = S.hill.fill;
            g.fill();
            g.strokeStyle = S.hill.line;
            g.lineWidth = 1;
            g.stroke();
            if (detail) {
              g.globalAlpha = op * 0.6;
              g.lineWidth = 0.8;
              g.beginPath();
              for (let k = 0; k < 3; k++) {
                const hx = ox + w * (0.1 + k * 0.28);
                const hy = oy - h * (0.62 - k * 0.2);
                g.moveTo(hx, hy);
                g.lineTo(hx + 1.2, oy - 0.8);
              }
              g.stroke();
              g.globalAlpha = op;
            }
          }
          break;
        }
        case 'Desert': {
          g.globalAlpha = op * 0.9;
          g.strokeStyle = S.dune;
          g.lineWidth = 1;
          g.beginPath();
          for (let k = 0; k < 3; k++) {
            const ox = (rr(40 + k) - 0.5) * R;
            const oy = (k - 1) * 9 + (rr(50 + k) - 0.5) * 4;
            g.moveTo(ox - 8, oy);
            g.quadraticCurveTo(ox - 4, oy - 4, ox, oy);
            g.quadraticCurveTo(ox + 4, oy + 4, ox + 8, oy);
          }
          g.stroke();
          break;
        }
        case 'Tundra': {
          g.fillStyle = S.snow;
          g.beginPath();
          for (let k = 0; k < 3; k++) {
            const ox = (rr(60 + k) - 0.5) * R * 1.1;
            const oy = (rr(70 + k) - 0.5) * R * 0.9;
            const rx = 4 + rr(80 + k) * 4;
            g.moveTo(ox + rx, oy);
            g.ellipse(ox, oy, rx, 1.6, 0, 0, Math.PI * 2);
          }
          g.fill();
          break;
        }
        case 'Grassland':
        case 'Plains': {
          if (!detail) break;
          const n = t.terrain === 'Grassland' ? 4 : 3;
          g.globalAlpha = op * 0.8;
          g.strokeStyle = t.terrain === 'Grassland' ? S.tuft.grass : S.tuft.plains;
          g.lineWidth = 0.9;
          g.beginPath();
          for (let k = 0; k < n; k++) {
            const tx = (rr(90 + k) - 0.5) * R * 1.1;
            const ty = (rr(100 + k) - 0.5) * R;
            g.moveTo(tx - 2, ty);
            g.lineTo(tx - 3, ty - 3);
            g.moveTo(tx, ty);
            g.lineTo(tx, ty - 4);
            g.moveTo(tx + 2, ty);
            g.lineTo(tx + 3, ty - 3);
          }
          g.stroke();
          break;
        }
        case 'Ocean': {
          if (rr(5) >= 0.22) break;
          const ox = (rr(6) - 0.5) * R * 0.8;
          const oy = (rr(7) - 0.5) * R * 0.6;
          g.strokeStyle = S.wave;
          g.lineWidth = 1;
          g.beginPath();
          g.moveTo(ox - 7, oy);
          g.quadraticCurveTo(ox - 3.5, oy - 3, ox, oy);
          g.quadraticCurveTo(ox + 3.5, oy + 3, ox + 7, oy);
          g.stroke();
          break;
        }
      }
      g.restore();
    }
  }

  /** Organic sprites anchored on the chunk's tiles, in the plan's painter order. */
  private drawOrganic(g: Ctx2D, tiles: number[], s: number) {
    const zoom = this.chunkZoom;
    const far = zoom * 36 < 22;
    const plan = far ? this.plans!.far : this.plans!.near;
    const picked: number[] = [];
    for (const i of tiles) {
      const l = plan.byTile.get(i);
      if (l) for (const k of l) picked.push(k);
    }
    if (!picked.length) return;
    picked.sort((a, b) => a - b);
    const side = organicSide(far);
    const S = this.style;
    g.save();
    g.imageSmoothingEnabled = true;
    g.imageSmoothingQuality = 'high';
    for (const k of picked) {
      const e = plan.elems[k];
      const t = this.tiles[e.tile];
      drawOrganicSprite(g, e.kind, e.sprite, far, e.x, e.y, side * e.scale, s, (t.visible ? 1 : S.fog.glyph) * this.artFade);
    }
    g.restore();
  }

  /**
   * Towns are painted miniatures chosen by the owner's era and the city's size tier (cityArt.ts). Zoomed out
   * (the same threshold as the seals) the shared far mark replaces the detailed sprite. A sprite that is still
   * decoding is simply skipped: the cache repaints when it arrives, so a town never falls back to a cube.
   */
  private drawTowns(g: Ctx2D, tiles: number[], s: number, inSight: boolean) {
    const far = isFar(HEX_SIZE * this.chunkZoom);
    for (const i of tiles) {
      const ci = this.cityAt.get(i);
      const t = this.tiles[i];
      if (!ci || !t.explored || this.hold.towns.has(i) || t.visible !== inSight) continue;
      const era = this.hold.era.get(i) ?? this.ownerEra.get(ci.owner) ?? 0;
      const tier = cityTier(this.hold.pop.get(i) ?? ci.population);
      g.save();
      g.translate(centerX(i, this.w), centerY(i, this.w));
      g.imageSmoothingEnabled = true;
      g.imageSmoothingQuality = 'high';
      g.globalAlpha = t.visible ? 1 : 0.75;
      const stem = far ? markStem(tier) : cityStem(era, tier);
      const px = CITY_WORLD * s;
      const img = stem ? cityImage(stem, spriteSize(px)) : null;
      if (img) this.drawSprite(g, img);
      const wall = this.hold.walls.has(i) ? undefined : this.wallAt.get(i);
      if (wall) {
        const wstem = wallStem(era, wall.bastion);
        const wimg = wstem ? cityImage(wstem, spriteSize(px)) : null;
        if (wimg) this.drawSprite(g, wimg);
      }
      g.restore();
    }
  }

  /** One city-art canvas, its ground oval centred on the hex (a little low, so tall roofs rise into the hex's top half). */
  private drawSprite(g: Ctx2D, img: CanvasImageSource) {
    const k = CITY_WORLD / CITY_CANVAS;
    g.drawImage(img, -(CITY_GROUND.x0 + CITY_GROUND.x1) / 2 * k, -CITY_GROUND.middle * k + CITY_DROP, CITY_WORLD, CITY_WORLD);
  }

  private drawTerritory(g: Ctx2D, tiles: number[], x0: number, y0: number, s: number, px: number) {
    if (this.variant !== 'current') return this.drawBoldTerritory(g, tiles, this.style.borders[this.variant], px);
    const S = this.style;
    const owners = new Map<number, number[]>();
    for (const i of tiles) {
      const t = this.tiles[i];
      const own = this.ownerOf(i);
      if (!t.explored || own === null) continue;
      const l = owners.get(own);
      if (l) l.push(i);
      else owners.set(own, [i]);
    }
    if (!owners.size) return;
    const blur = S.border.blur > 0 && canvasFilterSupported();
    for (const [owner, idx] of owners) {
      const col = mapPlayerColor(owner);
      const area = new Path2D();
      for (const i of idx) hexPath(area, centerX(i, this.w), centerY(i, this.w), HEX_SIZE + 0.4 * K);
      g.fillStyle = rgba(col, S.border.tint);
      g.fill(area);
      const edge = new Path2D();
      const inside = (i: number) => this.tiles[i].explored && this.ownerOf(i) === owner;
      for (const i of idx) {
        const cx = centerX(i, this.w);
        const cy = centerY(i, this.w);
        for (let e = 0; e < 6; e++) {
          const nb = neighborIndex(i, e, this.w, this.h);
          if (nb >= 0 && inside(nb)) continue;
          const a = CORNERS[e];
          const b = CORNERS[(e + 1) % 6];
          edge.moveTo(cx + a[0] * HEX_SIZE, cy + a[1] * HEX_SIZE);
          edge.lineTo(cx + b[0] * HEX_SIZE, cy + b[1] * HEX_SIZE);
        }
      }
      const band = S.border.band * K * 2;
      if (blur) {
        const sc = this.scratchCanvases();
        const bg = ctx2d(sc.band);
        bg.setTransform(1, 0, 0, 1, 0, 0);
        bg.globalCompositeOperation = 'source-over';
        bg.globalAlpha = 1;
        bg.clearRect(0, 0, CHUNK + 2 * PAD, CHUNK + 2 * PAD);
        bg.setTransform(s, 0, 0, s, -x0 * s + PAD, -y0 * s + PAD);
        bg.lineCap = 'round';
        bg.lineJoin = 'round';
        bg.strokeStyle = col;
        bg.lineWidth = band;
        bg.stroke(edge);
        blurCanvas(sc.band, S.border.blur * K * s);
        bg.setTransform(s, 0, 0, s, -x0 * s + PAD, -y0 * s + PAD);
        bg.globalCompositeOperation = 'destination-in';
        bg.fillStyle = '#000';
        bg.fill(area);
        bg.globalCompositeOperation = 'source-over';
        g.save();
        g.setTransform(1, 0, 0, 1, 0, 0);
        g.globalAlpha = S.border.bandAlpha;
        g.drawImage(sc.band as CanvasImageSource, PAD, PAD, CHUNK, CHUNK, 0, 0, CHUNK, CHUNK);
        g.restore();
        g.save();
        g.clip(area);
      } else {
        g.save();
        g.clip(area);
        g.lineCap = 'round';
        g.lineJoin = 'round';
        g.globalAlpha = S.border.bandAlpha;
        g.strokeStyle = col;
        g.lineWidth = band;
        g.stroke(edge);
        g.globalAlpha = 1;
      }
      g.lineCap = 'round';
      g.lineJoin = 'round';
      g.strokeStyle = col;
      g.lineWidth = Math.max(S.border.width * K * 2, 2.2 * px);
      g.setLineDash(PLAYER_BORDER_DASH[owner % PLAYER_BORDER_DASH.length].map((n) => n * K));
      g.stroke(edge);
      g.restore();
    }
  }
  /**
   * The bold border looks. Each empire paints its tint, an inner band and its lines clipped to its
   * own tiles, so on a shared boundary the two empires' lines sit side by side (ink against ink)
   * and never overpaint each other.
   */
  private drawBoldTerritory(g: Ctx2D, tiles: number[], spec: BorderSpec, px: number) {
    const S = this.style;
    const owners = new Map<number, number[]>();
    for (const i of tiles) {
      const t = this.tiles[i];
      const own = this.ownerOf(i);
      if (!t.explored || own === null) continue;
      const l = owners.get(own);
      if (l) l.push(i);
      else owners.set(own, [i]);
    }
    const floor = (world: number, minPx: number) => (world > 0 ? Math.max(world, minPx * px) : 0);
    const ink = rgba(S.tone.ink, spec.inkAlpha);
    for (const [owner, idx] of owners) {
      const col = mapBorderColor(owner);
      const area = new Path2D();
      for (const i of idx) hexPath(area, centerX(i, this.w), centerY(i, this.w), HEX_SIZE + 0.4 * K);
      g.fillStyle = rgba(col, spec.tint);
      g.fill(area);
      const free = new Path2D();
      const shared = new Path2D();
      const all = new Path2D();
      for (const i of idx) {
        const cx = centerX(i, this.w);
        const cy = centerY(i, this.w);
        for (let e = 0; e < 6; e++) {
          const nb = neighborIndex(i, e, this.w, this.h);
          const other = nb >= 0 ? this.tiles[nb] : null;
          if (other && other.explored && this.ownerOf(nb) === owner) continue;
          const a = CORNERS[e];
          const b = CORNERS[(e + 1) % 6];
          const meets = !!other && other.explored && this.ownerOf(nb) !== null;
          for (const path of [all, meets ? shared : free]) {
            path.moveTo(cx + a[0] * HEX_SIZE, cy + a[1] * HEX_SIZE);
            path.lineTo(cx + b[0] * HEX_SIZE, cy + b[1] * HEX_SIZE);
          }
        }
      }
      g.save();
      g.clip(area);
      g.lineCap = 'round';
      g.lineJoin = 'round';
      // The inner band: a stack of strokes, each narrower than the last, fading from the boundary inward.
      const depth = floor(spec.band, spec.bandMin);
      const steps = 6;
      g.strokeStyle = col;
      g.globalAlpha = spec.bandAlpha / steps;
      for (let k = 0; k < steps; k++) {
        g.lineWidth = 2 * depth * (1 - k / steps);
        g.stroke(all);
      }
      g.globalAlpha = 1;
      const outerCol = spec.outer > 0 ? mixHex(col, S.tone.vellum, spec.outerMix) : col;
      for (const [path, mul] of [[free, 1], [shared, spec.shared]] as const) {
        const inkW = floor(spec.ink, spec.inkMin) * mul;
        const outerW = floor(spec.outer, spec.outerMin);
        const lineW = floor(spec.line, spec.lineMin);
        g.strokeStyle = col;
        g.lineWidth = 2 * (inkW + outerW + lineW);
        g.stroke(path);
        if (outerW) {
          g.strokeStyle = outerCol;
          g.lineWidth = 2 * (inkW + outerW);
          g.stroke(path);
        }
        if (inkW) {
          g.strokeStyle = ink;
          g.lineWidth = 2 * inkW;
          g.stroke(path);
        }
      }
      g.restore();
    }
  }

}

/** `a` thinned toward `b` by `t`, as a hex. */
function mixHex(a: string, b: string, t: number): string {
  const [ar, ag, ab] = parseHex(a);
  const [br, bg, bb] = parseHex(b);
  const h = (x: number, y: number) => Math.round(x * (1 - t) + y * t).toString(16).padStart(2, '0');
  return `#${h(ar, br)}${h(ag, bg)}${h(ab, bb)}`;
}

function tileOf(q: number, r: number, w: number, h: number): number {
  const c = q + (r - (r & 1)) / 2;
  if (c < 0 || r < 0 || c >= w || r >= h) return -1;
  return r * w + c;
}

function hatchTile(color: string, pitch: number, width: number): AnyCanvas {
  const c = makeCanvas(pitch, pitch);
  const g = ctx2d(c);
  g.strokeStyle = color;
  g.lineWidth = width;
  g.beginPath();
  // 45-degree lines that tile seamlessly.
  g.moveTo(-1, pitch + 1);
  g.lineTo(pitch + 1, -1);
  g.moveTo(-1, 1);
  g.lineTo(1, -1);
  g.moveTo(pitch - 1, pitch + 1);
  g.lineTo(pitch + 1, pitch - 1);
  g.stroke();
  return c;
}

function unexploredTile(S: MapStyle, dpr: number): AnyCanvas {
  if (S.unexploredPattern === 'hatch') {
    // Terra incognita: a fine diagonal hatch.
    return hatchTile(S.unexploredInk, Math.round(7 * dpr), Math.max(1, dpr * 0.8));
  }
  // Deco scales: rows of overlapping arcs.
  const w = Math.round(24 * dpr);
  const h = Math.round(12 * dpr);
  const c = makeCanvas(w, h);
  const g = ctx2d(c);
  g.strokeStyle = S.unexploredInk;
  g.lineWidth = dpr;
  const r = w / 2;
  g.beginPath();
  g.arc(r, h + 0, r, Math.PI, 0);
  g.moveTo(0 + r, h / 2);
  g.arc(0, h / 2 + 0, r, 0, Math.PI, true);
  g.moveTo(w + r, h / 2);
  g.arc(w, h / 2, r, 0, Math.PI, true);
  g.stroke();
  return c;
}

/**
 * A hex outline with every corner and edge point nudged by noise keyed to its world position, so two
 * hexes sharing an edge trace the same wobbly line (no gaps) and biome boundaries stop following the grid.
 */
function wobblyHex(g: Ctx2D | Path2D, cx: number, cy: number, size: number) {
  const A = HEX_SIZE * 0.36;
  const nudge = (x: number, y: number): [number, number] => {
    const kx = Math.round(x / 3) + 5000;
    const ky = Math.round(y / 3) + 5000;
    return [x + (rnd(kx, ky, 71) - 0.5) * A, y + (rnd(kx, ky, 72) - 0.5) * A];
  };
  // Nudges are keyed on the true corner position, then scaled with the hex, so shared edges match at size HEX_SIZE.
  const k = size / HEX_SIZE;
  for (let e = 0; e < 6; e++) {
    const a = CORNERS[e];
    const b = CORNERS[(e + 1) % 6];
    const ax = cx + a[0] * HEX_SIZE;
    const ay = cy + a[1] * HEX_SIZE;
    const bx = cx + b[0] * HEX_SIZE;
    const by = cy + b[1] * HEX_SIZE;
    for (let q = 0; q < 4; q++) {
      const t = q / 4;
      const [px, py] = nudge(ax + (bx - ax) * t, ay + (by - ay) * t);
      const x = cx + (px - cx) * k;
      const y = cy + (py - cy) * k;
      if (e === 0 && q === 0) g.moveTo(x, y);
      else g.lineTo(x, y);
    }
  }
  g.closePath();
}
