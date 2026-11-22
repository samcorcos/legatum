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
export const CITY_ERAS = ['ancient', 'classical', 'medieval', 'renaissance', 'industrial', 'machine', 'modern', 'contemporary', 'future'] as const;
export const CITY_TIERS = ['hamlet', 'town', 'city', 'metropolis'] as const;
export type CityTier = (typeof CITY_TIERS)[number];

// Machine Age, World War, Modern and Future wall families added 2026-11-16; refresh the shared glob transform cache.
const urls = import.meta.glob<string>('../assets/cities/**/*.png', { eager: true, query: '?url', import: 'default' });
/** `ancient-hamlet-128` style key (path below `assets/cities/`, no extension) to URL. */
const byKey = new Map<string, string>();
for (const [path, url] of Object.entries(urls)) byKey.set(path.replace(/^.*\/assets\/cities\//, '').replace(/\.png$/, ''), url);

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

/** The wall overlay stem for an era: the era's own, else the nearest finished one (World War, Modern and Future share the last). A star fort is the Alpine Redoubt. */
export function wallStem(era: number, bastion: boolean, exists: Has = has): string | null {
  const e = clampEra(era);
  const ok = (name: string) => exists(`${name}-walls`, 128) || exists(`${name}-walls`, 512);
  const pick = bastion && ok('renaissance') ? 'renaissance' : nearest(CITY_ERAS, e, ok);
  return pick ? `${pick}-walls` : null;
}

/** A city's visible wall level for its banner: 0 none (or not in sight), 1 Walls, 2 Castle, 3 Star Fort. */
export function wallTierOf(c: { visible: boolean; walls: boolean; citadel?: boolean; star_fort?: boolean }): number {
  return c.visible && c.walls ? 1 + (c.citadel ? 1 : 0) + (c.star_fort ? 1 : 0) : 0;
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
