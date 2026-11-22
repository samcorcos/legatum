// Icon set drawn by the art agent into src/assets/icons/*.svg. Everything falls back gracefully
// (vector glyphs on the map, text in panels) when a file is missing. The globs below read the folder at transform
// time, so a shared transform cache (the box's node_modules/.vitest-cache) keeps serving the old file list until this
// file's text changes: the four new world wonders add their own card and map fallback icons, and the Hunter-Killer,
// Stealth Fighter and Stealth Bomber their own unit icons, and the occupation lock its padlock mark.
// 2026-11-15: Egypt and Rome building and unit icons, including Byzantium’s Tagmatic icon, refresh Vitest’s per-file glob cache.
const urls = {
  ...import.meta.glob<string>('./assets/icons/*.svg', { eager: true, query: '?url', import: 'default' }),
  // Faction glyphs (heraldry pass): `glyph-<faction>` for marks below ~40 px.
  ...import.meta.glob<string>('./assets/heraldry/glyph-*.svg', { eager: true, query: '?url', import: 'default' }),
};
const resourceUrls = import.meta.glob<string>('./assets/resources/*.png', { eager: true, query: '?url', import: 'default' });

const byName = new Map<string, string>();
for (const [path, url] of Object.entries(urls)) {
  const name = path.split('/').pop()!.replace(/\.svg$/, '');
  byName.set(name, url);
}

// Every faction glyph (heraldry/glyph-<faction>.svg, the 2026-11-21 historical set) answers to `faction-<faction>`.
const glyphs = import.meta.glob<string>('./assets/heraldry/glyph-*.svg', { eager: true, query: '?url', import: 'default' });
for (const [path, url] of Object.entries(glyphs)) {
  byName.set(path.split('/').pop()!.replace('glyph-', 'faction-').replace(/\.svg$/, ''), url);
}

export function iconUrl(name: string): string | null {
  return byName.get(name) ?? null;
}

const resourceImages = new Map<string, HTMLImageElement | null>();
const resourceSprites = new Map<string, HTMLCanvasElement>();

function resourceImage(resource: string, sourceSize: 32 | 64 | 256): HTMLImageElement | null {
  const key = `${resource.toLowerCase()}-${sourceSize}`;
  if (resourceImages.has(key)) {
    const img = resourceImages.get(key)!;
    return img && img.complete && img.naturalWidth > 0 ? img : null;
  }
  const url = resourceArtUrl(resource, sourceSize);
  if (!url || typeof Image === 'undefined') { resourceImages.set(key, null); return null; }
  const img = new Image();
  img.onload = () => { loads++; onLoad?.(); };
  img.src = url;
  resourceImages.set(key, img);
  return null;
}

/** Full-colour resource art for the canvas map, downscaled once at device resolution. */
export function resourceArtSprite(resource: string, physicalPixels: number): HTMLCanvasElement | null {
  const size = Math.max(1, Math.round(physicalPixels));
  const sourceSize: 32 | 64 | 256 = size <= 32 ? 32 : size <= 64 ? 64 : 256;
  const key = `${resource}-${sourceSize}-${size}`;
  const cached = resourceSprites.get(key);
  if (cached) return cached;
  const img = resourceImage(resource, sourceSize);
  if (!img) return null;
  const canvas = document.createElement('canvas');
  canvas.width = canvas.height = size;
  const ctx = canvas.getContext('2d')!;
  ctx.imageSmoothingEnabled = true;
  ctx.imageSmoothingQuality = 'high';
  ctx.drawImage(img, 0, 0, size, size);
  resourceSprites.set(key, canvas);
  return canvas;
}

/** Static full-colour resource art for UI surfaces. */
export function resourceArtUrl(resource: string, sourceSize: 32 | 64 | 256 = 64): string | null {
  // The core's pending Future-era rename still sends Lithium in older views and saves.
  const subject = resource.toLowerCase() === 'lithium' ? 'silicon' : resource.toLowerCase();
  return resourceUrls[`./assets/resources/${subject}-${sourceSize}.png`] ?? null;
}

const images = new Map<string, HTMLImageElement | null>();
const tinted = new Map<string, HTMLCanvasElement>();
let onLoad: (() => void) | null = null;
let loads = 0;

/** Bumped whenever an icon image finishes loading (sprite caches key on it). */
export function iconEpoch(): number {
  return loads;
}

/** Another image set (the painted unit art) finished loading: sprite caches rebuild and the map redraws. */
export function iconLoaded() {
  loads++;
  onLoad?.();
}

/** Called when an icon image finishes loading (the map redraws). */
export function onIconLoad(cb: () => void) {
  onLoad = cb;
}

function image(name: string): HTMLImageElement | null {
  if (images.has(name)) {
    const img = images.get(name)!;
    return img && img.complete && img.naturalWidth > 0 ? img : null;
  }
  const url = iconUrl(name);
  if (!url || typeof Image === 'undefined') {
    images.set(name, null);
    return null;
  }
  const img = new Image();
  img.onload = () => {
    loads++;
    onLoad?.();
  };
  img.src = url;
  images.set(name, img);
  return null;
}

/** The icon as a single-color silhouette at `px` pixels (cached), for map tokens. */
export function tintedIcon(name: string, color: string, px: number): HTMLCanvasElement | null {
  const size = Math.max(8, Math.round(px));
  const key = `${name}|${color}|${size}`;
  const hit = tinted.get(key);
  if (hit) return hit;
  const img = image(name);
  if (!img) return null;
  const c = document.createElement('canvas');
  c.width = c.height = size;
  const g = c.getContext('2d')!;
  g.drawImage(img, 0, 0, size, size);
  g.globalCompositeOperation = 'source-in';
  g.fillStyle = color;
  g.fillRect(0, 0, size, size);
  tinted.set(key, c);
  return c;
}
