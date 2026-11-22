// The terrain look: organic S2 is the default (decided 2026-10-11). Forests, hills and mountains are flat illustrated S2
// sprites (docs/design/terrain-art), drawn by connected region (organicTerrain.ts) so a wood reads as one forest and a range
// as one range. Two switches stay in the URL for comparing looks (neither is remembered: reloading without it is the shipped look):
//
//   ?terrain=current   the old drawn circles, triangles and arcs
//   ?organic=0         S2 as one sprite per hex, without the region logic
//
// The retired round 1 and 2 directions (A, B, A-light, S1, S3) are recorded in docs/design/terrain-art/README.md, with their
// atlases in docs/design/terrain-art/source-atlases and sprites. The shipped sprites live in public/terrain (256 and 128 px
// copies, plus 64 px far marks).
import { K } from './grid';
import { makeCanvas, ctx2d, rnd, type AnyCanvas } from './noise';

export const TERRAIN_VARIANTS = ['current', 's2'] as const;
export type TerrainVariant = (typeof TERRAIN_VARIANTS)[number];

const params = new URLSearchParams(globalThis.location?.search ?? '');
const parse = (v: string | null | undefined): TerrainVariant | null => (TERRAIN_VARIANTS as readonly string[]).includes(v ?? '') ? (v as TerrainVariant) : null;
let active: TerrainVariant = parse(params.get('terrain')) ?? 's2';
export const getTerrainVariant = (): TerrainVariant => active;
export function setTerrainVariant(v: TerrainVariant) { active = v; }

// Region-based terrain (organicTerrain.ts) instead of one sprite per hex. On unless `?organic=0`, and only when the sprites loaded.
let organic = params.get('organic') !== '0';
const wantsOrganic = (): boolean => organic && active !== 'current';
/** The organic modifier is on, the active variant has sprites for it, and they have loaded. */
export const isOrganic = (): boolean => wantsOrganic() && sprites.size > 0;
export function setOrganic(on: boolean) { organic = on; }

type Kind = 'Forest' | 'Hills' | 'Mountains';
const DIR: Record<Kind, string> = { Forest: 'forests', Hills: 'hills', Mountains: 'mountains' };
const FILES: Record<Kind, string[]> = {
  Forest: ['forest-mixed-01', 'forest-mixed-02', 'forest-mixed-03', 'forest-tundra-conifers'],
  Hills: ['downs-01', 'downs-02', 'downs-03', 'downs-04'],
  Mountains: ['mountain-01', 'mountain-02', 'mountain-03', 'mountain-04'],
};
const FAR_FILES: Record<Kind, string> = { Forest: 'forest', Hills: 'downs', Mountains: 'mountain' };

/** Sprite square side, in mockup units (hex radius 27): the art's hex footprint fills about 84% of it. */
const SIDE = 58;
/** Far mark square side, mockup units. */
const FAR_SIDE = 34;
const base = () => `${(import.meta as { env?: { BASE_URL?: string } }).env?.BASE_URL ?? '/'}terrain/`;
const sprites = new Map<string, CanvasImageSource>();

function load(url: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const img = new Image();
    img.onload = () => resolve(img);
    img.onerror = () => reject(new Error(`terrain sprite failed: ${url}`));
    img.src = url;
  });
}

/** Organic hills: the same sprite with a darker, warmer value so rolling hills separate from desert and plains. */
function firmer(img: HTMLImageElement): AnyCanvas {
  const c = makeCanvas(img.naturalWidth, img.naturalHeight);
  const g = ctx2d(c);
  g.drawImage(img, 0, 0);
  g.globalCompositeOperation = 'multiply';
  g.fillStyle = HILL_FIRM;
  g.fillRect(0, 0, c.width, c.height);
  g.globalCompositeOperation = 'destination-in';
  g.drawImage(img, 0, 0);
  return c;
}
const HILL_FIRM = '#c4ab80';

/** Load the sprites the active variant needs. A no-op for `current`; await it before the first map draw. */
export async function loadTerrainSprites(): Promise<void> {
  if (getTerrainVariant() === 'current') return;
  const jobs: Promise<void>[] = [];
  for (const kind of Object.keys(FILES) as Kind[]) {
    FILES[kind].forEach((name, idx) => {
      for (const size of [128, 256]) {
        jobs.push(load(`${base()}${DIR[kind]}/${size}/${name}.png`).then((img) => {
          sprites.set(`${kind}:${idx}:${size}`, kind === 'Hills' && wantsOrganic() ? firmer(img) : img);
        }));
      }
      jobs.push(load(`${base()}far/${FAR_FILES[kind]}-${idx + 1}.png`).then((img) => { sprites.set(`${kind}:${idx}:far`, img); }));
    });
  }
  await Promise.all(jobs);
}

/** Organic sprite side, mockup units: the classic sprite side (SIDE) near, the far mark side (FAR_SIDE) zoomed out. */
export const organicSide = (far: boolean) => (far ? FAR_SIDE : SIDE) * K;

/**
 * Draw one organic sprite centred at (x, y) in world units (context in world space), `side` world
 * units square. Returns false when the sprite is not loaded.
 */
export function drawOrganicSprite(
  g: CanvasRenderingContext2D | OffscreenCanvasRenderingContext2D,
  kind: Kind, idx: number, far: boolean, x: number, y: number, side: number, devPerWorld: number, alpha: number,
): boolean {
  const key = far ? `${kind}:${idx}:far` : `${kind}:${idx}:${side * devPerWorld <= 140 ? 128 : 256}`;
  const img = sprites.get(key);
  if (!img) return false;
  const prev = g.globalAlpha;
  g.globalAlpha = prev * alpha;
  g.drawImage(img, x - side / 2, y - side / 2, side, side);
  g.globalAlpha = prev;
  return true;
}

export interface FeatureArg {
  kind: Kind;
  c: number;
  r: number;
  /** CSS pixels per world pixel of the chunk being drawn (decides the far mark). */
  zoom: number;
  /** Device pixels per world pixel of the chunk (decides the sprite resolution). */
  devPerWorld: number;
  /** Fog alpha for the tile. */
  alpha: number;
  /** A forest next to tundra uses the conifer-only sprite. */
  tundra: boolean;
}

/**
 * Draw one feature hex's art with the context at the hex centre in mockup units. Returns false when
 * the sprites are not loaded, so the caller falls back to the drawn glyph.
 */
export function drawFeatureArt(g: CanvasRenderingContext2D | OffscreenCanvasRenderingContext2D, a: FeatureArg): boolean {
  if (getTerrainVariant() === 'current' || sprites.size === 0) return false;
  const idx = a.kind === 'Forest' && a.tundra ? 3 : Math.floor(rnd(a.c, a.r, 211) * (a.kind === 'Forest' ? 3 : 4));
  const far = a.zoom * 36 < 22;
  const side = far ? FAR_SIDE : SIDE;
  const px = side * K * a.devPerWorld;
  const key = far ? `${a.kind}:${idx}:far` : `${a.kind}:${idx}:${px <= 140 ? 128 : 256}`;
  const img = sprites.get(key);
  if (!img) return false;
  g.save();
  g.globalAlpha *= a.alpha;
  g.imageSmoothingEnabled = true;
  g.imageSmoothingQuality = 'high';
  g.drawImage(img, -side / 2, -side / 2, side, side);
  g.restore();
  return true;
}
