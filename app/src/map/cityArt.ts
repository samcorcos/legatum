// Painted city miniatures for the strategic map: one sprite per (era, size tier), chosen from the OWNER's era
// (public: `PlayerPublic.era`), so a city changes look the turn its owner enters the next era and a rival's
// city shows that rival's era. Files live in `assets/cities/` (`<era>-<tier>[-128|-64].png`, shared
// `marks/<tier>-mark-<size>.png` for the zoomed-out tier, `<era>-walls` overlays).
//
// The loader never leaves a cube behind: when a file is missing it falls back to the nearest finished era (then the
// nearest finished size), and while an image is still decoding the caller draws nothing for that frame and is told
// to repaint (`cityArtEpoch`, `onCityArtLoad`), the same contract as the icon loader.

// The Machine Age's city sprites are sepia-tinted Industrial ones until its own art (increment 2). `modern` is the World War
// era's; the Modern era (`contemporary`) has no sprites yet and falls back to the nearest finished era, the World War one.
import type { CityView, PlayerView } from '../types';
import { sitesOf } from './constructionSites';
import cityScaleData from './cityScale.json';

export { wallTierOf } from '../data';

export const CITY_ERAS = ['ancient', 'classical', 'medieval', 'renaissance', 'industrial', 'machine', 'modern', 'contemporary', 'future'] as const;
export const CITY_TIERS = ['hamlet', 'town', 'city', 'metropolis'] as const;
export type CityTier = (typeof CITY_TIERS)[number];

/** Scale of a detailed town's 512 px master. Far marks and wall images have separate geometry. */
export function cityDrawScale(stem: string | null): number {
  return cityScaleData.sprites[stem as keyof typeof cityScaleData.sprites]?.scale ?? 1;
}

/** Keep the highest ordinary roof below the banner block, including a small gap. */
export function cityDrawDrop(scale: number): number {
  // 50 rounds up the source y=60 roof offset in world units; -15 leaves 3 units below the banner bottom at -18.
  return Math.max(18, 50 * scale - 15);
}

// Machine Age, World War, Modern and Future wall families added 2026-11-16; refresh the shared glob transform cache.
const urls = import.meta.glob<string>('../assets/cities/**/*.png', { eager: true, query: '?url', import: 'default' });
/** `ancient-hamlet-128` style key (path below `assets/cities/`, no extension) to URL. */
const byKey = new Map<string, string>();
for (const [path, url] of Object.entries(urls)) byKey.set(path.replace(/^.*\/assets\/cities\//, '').replace(/\.png$/, ''), url);
// Walls under construction: the era's painted footings (`construction/<era>-walls-stage1`), registered to the finished ring's canvas,
// keyed `<era>-walls-footings` so the terrain cache draws them with the town (back half, town, front half) like finished Walls.
const footings = import.meta.glob<string>('../assets/construction/*-walls-stage1*.png', { eager: true, query: '?url', import: 'default' });
for (const [path, url] of Object.entries(footings)) byKey.set(path.replace(/^.*\/construction\//, '').replace(/\.png$/, '').replace('-walls-stage1', '-walls-footings'), url);

/** City size tiers by population (the art's own bands: 1-3, 4-7, 8-12, 13+). */
export function cityTier(population: number): CityTier {
  return population >= 13 ? 'metropolis' : population >= 8 ? 'city' : population >= 4 ? 'town' : 'hamlet';
}

/** `era` is 0-based (0 Ancient .. 8 Future). Out-of-range values clamp, so a legacy game without eras shows Ancient. */
const clampEra = (era: number) => Math.min(CITY_ERAS.length - 1, Math.max(0, Number.isFinite(era) ? Math.round(era) : 0));

/** Which resolution to use for a sprite drawn about `px` device pixels wide: the 512 master only when it is needed. */
export function spriteSize(px: number): 64 | 128 | 512 {
  return px <= 80 ? 64 : px <= 200 ? 128 : 512;
}
const SUFFIX = { 64: '-64', 128: '-128', 512: '' } as const;

type Has = (stem: string, size: 64 | 128 | 512) => boolean;
const has: Has = (stem, size) => byKey.has(`${stem}${SUFFIX[size]}`);
/** Nearest finished item of a list to `index`, ties toward the earlier one (the older look is the safer stand-in). */
function nearest<T>(list: readonly T[], index: number, ok: (item: T) => boolean): T | null {
  for (let d = 0; d < list.length; d++) {
    for (const j of [index - d, index + d]) if (j >= 0 && j < list.length && ok(list[j])) return list[j];
  }
  return null;
}

/** The sprite stem (`medieval-city`) for an era and tier, falling back to the nearest era that has the size, then the nearest size. `exists` is injectable for tests. */
export function cityStem(era: number, tier: CityTier, exists: Has = has): string | null {
  const e = clampEra(era);
  const t = CITY_TIERS.indexOf(tier);
  const eraHas = (name: string, k: number) => exists(`${name}-${CITY_TIERS[k]}`, 128) || exists(`${name}-${CITY_TIERS[k]}`, 512);
  if (eraHas(CITY_ERAS[e], t)) return `${CITY_ERAS[e]}-${tier}`;
  const sameSize = nearest(CITY_ERAS, e, (name) => eraHas(name, t));
  if (sameSize) return `${sameSize}-${tier}`;
  const size = nearest(CITY_TIERS, t, (name) => eraHas(CITY_ERAS[e], CITY_TIERS.indexOf(name)));
  if (size) return `${CITY_ERAS[e]}-${size}`;
  for (const name of CITY_ERAS) for (const sz of CITY_TIERS) if (eraHas(name, CITY_TIERS.indexOf(sz))) return `${name}-${sz}`;
  return null;
}

/** The wall overlay stem for an era: the era's own, else the nearest finished one (World War, Modern and Future share the last). */
export function wallStem(era: number, bastion: boolean, exists: Has = has): string | null {
  const e = clampEra(era);
  const ok = (name: string) => exists(`${name}-walls`, 128) || exists(`${name}-walls`, 512);
  const pick = bastion && ok('renaissance') ? 'renaissance' : nearest(CITY_ERAS, e, ok);
  return pick ? `${pick}-walls` : null;
}

/** The era whose ring art shows a Walls level: level I the Ancient ring, each level one era later (IX Future). */
export function ringEraOf(level: number): number {
  return clampEra(Math.max(1, level) - 1);
}

/** The footings stem for Walls under construction: the same ring as `wallStem`, unfinished. */
export function footingsStem(era: number, bastion: boolean): string | null {
  const stem = wallStem(era, bastion, (name, size) => has(name.replace(/-walls$/, '-walls-footings'), size));
  return stem ? stem.replace(/-walls$/, '-walls-footings') : null;
}

/** The wall ring a city's town stands in on the map (finished Walls, else footings while Walls are built), as the terrain cache draws it. */
export function cityRing(view: PlayerView, c: CityView): string | null {
  const owner = view.players.find((p) => p.id === c.owner);
  const bastion = owner?.faction === 'Switzerland';
  if (c.visible && c.walls) return wallStem(ringEraOf(c.walls_level ?? 1), bastion);
  return sitesOf(view).some((s) => s.walls && s.city === c.id) ? footingsStem(ringEraOf(1), bastion) : null;
}

/** Shared far-zoom mark for a tier (not era-specific). */
export function markStem(tier: CityTier): string {
  return `marks/${tier}-mark`;
}

/** Eras with a complete set of four sizes: what the art directory covers right now. Used by the tour to list any gap. */
export function missingCityArt(): string[] {
  const out: string[] = [];
  for (const era of CITY_ERAS) for (const tier of CITY_TIERS) if (!has(`${era}-${tier}`, 512)) out.push(`${era}-${tier}`);
  for (const tier of CITY_TIERS) if (!has(markStem(tier), 128)) out.push(markStem(tier));
  return out;
}

// ---- image loading ----

const images = new Map<string, HTMLImageElement | null>();
const listeners = new Set<() => void>();
/** File keys in the order they finished decoding. */
const arrived: string[] = [];
let tracing: Set<string> | null = null;

/** Bumped whenever a city image finishes loading: terrain chunks that drew without it repaint. */
export function cityArtEpoch(): number {
  return arrived.length;
}

/** The sprites that finished decoding after the first `since` arrivals (`since` is an earlier `cityArtEpoch()`). */
export function cityArtArrivals(since: number): string[] {
  return arrived.slice(since);
}

/** Runs `draw` and returns the sprites it found still decoding: what the drawing is waiting for, so it repaints when those arrive and not for any other sprite. */
export function traceCityArt(draw: () => void): Set<string> {
  const outer = tracing;
  const waiting = new Set<string>();
  tracing = waiting;
  try {
    draw();
  } finally {
    tracing = outer;
  }
  return waiting;
}

/** Called after an image loads (the map asks for a frame). Returns the unsubscribe. */
export function onCityArtLoad(cb: () => void): () => void {
  listeners.add(cb);
  return () => listeners.delete(cb);
}

/** The decoded sprite for `stem` at `size`, or null while it loads (a load is started). A missing size falls back to the next larger one. */
export function cityImage(stem: string, size: 64 | 128 | 512): HTMLImageElement | null {
  const order: (64 | 128 | 512)[] = size === 64 ? [64, 128, 512] : size === 128 ? [128, 512, 64] : [512, 128, 64];
  const use = order.find((s) => byKey.has(`${stem}${SUFFIX[s]}`));
  if (use === undefined) return null;
  const key = `${stem}${SUFFIX[use]}`;
  if (images.has(key)) {
    const img = images.get(key)!;
    if (img && img.complete && img.naturalWidth > 0) return img;
    if (img) tracing?.add(key);
    return null;
  }
  if (typeof Image === 'undefined') {
    images.set(key, null);
    return null;
  }
  const img = new Image();
  // A sprite that fails to load is settled too (nothing more will come), so a chunk never waits on it for ever.
  img.onload = img.onerror = () => {
    arrived.push(key);
    for (const cb of listeners) cb();
  };
  img.src = byKey.get(key)!;
  images.set(key, img);
  tracing?.add(key);
  return null;
}

/** The URL of a city sprite, for DOM use (the era-up card shows the board's own art). */
export function cityUrl(stem: string, size: 64 | 128 | 512 = 512): string | null {
  return byKey.get(`${stem}${SUFFIX[size]}`) ?? byKey.get(stem) ?? null;
}

/** Sprite geometry, from the art's shared canvas: the ground oval spans x 18..493 of 512 and rests on y = 454. */
export const CITY_CANVAS = 512;
export const CITY_GROUND = { x0: 18, x1: 493, bottom: 454, middle: 335 };

/**
 * A walled town is drawn at this scale about its ground centre, so it stands inside the ring instead of under it: the
 * ring's back half is drawn first, then the town, then the front half (`WALL_CUT`). Every era's ring leaves this much room.
 */
export const WALLED_TOWN_SCALE = 0.78;
/** A wall and its enclosed town change size together; the town keeps its measured building scale. */
export const wallDrawScale = (townScale: number) => townScale / WALLED_TOWN_SCALE;
/** Where a wall ring is cut into back and front (canvas y): the ground oval's middle. */
export const WALL_CUT = CITY_GROUND.middle;
/**
 * Draws a city-art canvas (`img`, square) into the box at `x, y` sized `size`: whole, or the `back` or `front` part of a
 * wall ring cut at `WALL_CUT`. The back part keeps one spare source row, which the front part covers, so no seam shows.
 */
export function drawCityCanvas(g: CanvasDrawImage, img: HTMLImageElement | HTMLCanvasElement, x: number, y: number, size: number, part: 'all' | 'back' | 'front' = 'all') {
  if (part === 'all') { g.drawImage(img, x, y, size, size); return; }
  const n = 'naturalHeight' in img ? img.naturalHeight : img.height, w = 'naturalWidth' in img ? img.naturalWidth : img.width;
  const cut = Math.round(n * WALL_CUT / CITY_CANVAS);
  if (part === 'back') g.drawImage(img, 0, 0, w, cut + 1, x, y, size, size * (cut + 1) / n);
  else g.drawImage(img, 0, cut, w, n - cut, x, y + size * cut / n, size, size * (n - cut) / n);
}

const walled = new Map<string, HTMLCanvasElement | null>();

/**
 * A town fitted into its wall ring, on a canvas in the ring's own frame (`ring`'s pixel size): the town at
 * `WALLED_TOWN_SCALE` about the ground centre, kept only inside the ring's court (`courtMask`), so in the back half no
 * house stands on a rampart or bastion. Drawn between the ring's back and front halves. Null where there is no canvas
 * (tests), and the caller draws the scaled town unclipped.
 */
export function walledTown(town: HTMLImageElement, ring: HTMLImageElement): HTMLCanvasElement | null {
  const key = `${town.src}|${ring.src}|${ring.naturalWidth}`;
  if (walled.has(key)) return walled.get(key)!;
  const mask = courtMask(ring);
  let out: HTMLCanvasElement | null = null;
  if (mask) {
    const n = ring.naturalWidth;
    out = document.createElement('canvas');
    out.width = out.height = n;
    const g = out.getContext('2d');
    if (g) {
      const k = n / CITY_CANVAS, s = n * WALLED_TOWN_SCALE;
      const cx = (CITY_GROUND.x0 + CITY_GROUND.x1) / 2 * k, cy = CITY_GROUND.middle * k;
      g.imageSmoothingEnabled = true;
      g.imageSmoothingQuality = 'high';
      g.drawImage(town, cx - cx * WALLED_TOWN_SCALE, cy - cy * WALLED_TOWN_SCALE, s, s);
      g.globalCompositeOperation = 'destination-in';
      g.drawImage(mask, 0, 0);
    } else out = null;
  }
  walled.set(key, out);
  return out;
}

/**
 * Where a town may show inside a wall ring (`ring`'s pixel size, opaque = allowed): the ring's court (the clear ground
 * reached from the oval's centre without crossing the wall), filled row by row; every column above the court (roofs
 * and towers rising over the far wall); and all of the front half, which the near wall covers anyway. Grown a little,
 * so roofs tuck under the wall's inner edge.
 */
export function courtMask(ring: HTMLImageElement): HTMLCanvasElement | null {
  if (typeof document === 'undefined') return null;
  const n = ring.naturalWidth, k = n / CITY_CANVAS;
  const c = document.createElement('canvas');
  c.width = c.height = n;
  const g = c.getContext('2d', { willReadFrequently: true });
  if (!g) return null;
  g.drawImage(ring, 0, 0);
  let data: ImageData;
  try { data = g.getImageData(0, 0, n, n); } catch { return null; }
  const alpha = (i: number) => data.data[i * 4 + 3];
  const court = new Uint8Array(n * n);
  const cut = Math.round(WALL_CUT * k), start = cut * n + Math.round((CITY_GROUND.x0 + CITY_GROUND.x1) / 2 * k);
  if (alpha(start) >= 40) return null;
  const stack = [start];
  court[start] = 1;
  while (stack.length) {
    const i = stack.pop()!, x = i % n;
    for (const j of [i - n, i + n, x > 0 ? i - 1 : -1, x < n - 1 ? i + 1 : -1]) if (j >= 0 && j < n * n && !court[j] && alpha(j) < 40) { court[j] = 1; stack.push(j); }
  }
  const allow = new Uint8Array(n * n);
  for (let y = 0; y < n; y++) {
    let l = -1, r = -1;
    for (let x = 0; x < n; x++) if (court[y * n + x]) { if (l < 0) l = x; r = x; }
    if (l >= 0) for (let x = l; x <= r; x++) allow[y * n + x] = 1;
  }
  for (let x = 0; x < n; x++) {
    let top = -1;
    for (let y = 0; y < n; y++) if (allow[y * n + x]) { top = y; break; }
    if (top > 0) for (let y = 0; y < top; y++) allow[y * n + x] = 1;
  }
  for (let i = cut * n; i < n * n; i++) allow[i] = 1;
  // Grow by about 6 master pixels, so the clip edge sits under the wall, not in the open court.
  const grow = Math.max(1, Math.round(6 * k));
  const out = g.createImageData(n, n);
  for (let y = 0; y < n; y++) for (let x = 0; x < n; x++) {
    let on = allow[y * n + x];
    for (let d = 1; !on && d <= grow; d++) on = (x >= d && allow[y * n + x - d]) || (x + d < n && allow[y * n + x + d]) || (y >= d && allow[(y - d) * n + x]) || (y + d < n && allow[(y + d) * n + x]) ? 1 : 0;
    out.data[(y * n + x) * 4 + 3] = on ? 255 : 0;
  }
  g.putImageData(out, 0, 0);
  return c;
}

/**
 * The city art's footprint relative to the hex centre, in world units, for marks that must keep clear of it (resource
 * medallions, wonder badges): the ground oval's width, from the top of an ordinary town's roofs to the oval's front edge.
 * Spires and towers rise above `top`; marks above that height sit over sky, not over buildings.
 */
export function cityArtFootprint(world: number, drop: number): { x0: number; x1: number; y0: number; y1: number } {
  const k = world / CITY_CANVAS;
  const half = (CITY_GROUND.x1 - CITY_GROUND.x0) / 2 * k;
  return { x0: -half, x1: half, y0: drop - (CITY_GROUND.middle - 190) * k, y1: drop + (CITY_GROUND.bottom - CITY_GROUND.middle) * k };
}

/** A town's footprint (`cityArtFootprint`) as an oval about its hex centre, in world units: what painted neighbours keep clear of. */
export interface TownOval { cy: number; a: number; b: number }
/** The clear ground (world units, about 5 px at z1.1) between a town's footprint and a painting beside it: wall towers and bastions stand a little past the ground oval, and a touching neighbour still reads as crowding (UI review, PR 205). */
export const TOWN_GAP = 5;
export function townOval(world: number, drop: number): TownOval {
  const f = cityArtFootprint(world, drop);
  return { cy: (f.y0 + f.y1) / 2, a: (f.x1 - f.x0) / 2, b: (f.y1 - f.y0) / 2 };
}

/**
 * Where a painting beside a town goes so it keeps clear of the town (a mountain, a wood or ruins on the next hex): `dx, dy`
 * is the painting's centre from the town's hex centre and `ha, hb` the half size of its painted mass at full scale (world
 * units). Left alone when it already clears the oval by `gap`; otherwise it steps straight away from the town by up to
 * `maxPush` and shrinks to no less than `minScale`, taking the least change of the two that clears (each counted against
 * its limit). When nothing within the limits clears, it takes both limits.
 */
export function clearOfTown(dx: number, dy: number, ha: number, hb: number, oval: TownOval, opts: { gap: number; maxPush: number; minScale: number }): { dx: number; dy: number; scale: number } {
  const { gap, maxPush, minScale } = opts;
  const clear = (x: number, y: number, s: number) => (x / (oval.a + ha * s + gap)) ** 2 + ((y - oval.cy) / (oval.b + hb * s + gap)) ** 2 >= 1;
  if (clear(dx, dy, 1)) return { dx, dy, scale: 1 };
  const len = Math.hypot(dx, dy - oval.cy);
  // Straight away from the oval's middle (a painting right on the middle goes down, in front of the town).
  const ux = len > 1e-6 ? dx / len : 0, uy = len > 1e-6 ? (dy - oval.cy) / len : 1;
  let best: { dx: number; dy: number; scale: number } | null = null, cost = Infinity;
  const steps = 10;
  for (let i = 0; i <= steps; i++) {
    const s = 1 - (1 - minScale) * i / steps;
    for (let p = 0; p <= maxPush + 1e-9; p += Math.max(0.5, maxPush / 24)) {
      const x = dx + ux * p, y = dy + uy * p;
      if (!clear(x, y, s)) continue;
      const c = p / Math.max(1e-6, maxPush) + i / steps;
      if (c < cost) { cost = c; best = { dx: x, dy: y, scale: s }; }
      break;
    }
  }
  return best ?? { dx: dx + ux * maxPush, dy: dy + uy * maxPush, scale: minScale };
}
