// Egypt's Royal Tomb on the map: the painted precinct beside the capital it stands in, in three construction
// stages while the work is on (public: every empire sees where it will stand), then the finished tomb.
// The tomb belongs to its city (banner seal, hover line); the painting stands on a neighbouring tile of the
// city's own land, the way a world wonder stands on its chosen tile, so it never covers the town.
import type { Hex, PlayerView } from '../types';
import { hexToPixel, neighbors, tileIndex } from '../hex';
import { HEX_SIZE, isFar } from './grid';
import { stageImage } from './constructionFx';
import { constructionStage, sitesOf } from './constructionSites';
import { landmarkBox, landmarksOf, type LandmarkBox } from './landmarks';
import { goalCities, tombSites, tombStatus } from '../goalCities';
import { iconLoaded } from '../icons';

export interface TombPiece { city: number; owner: number; tile: number; hex: Hex; k: number }

const STEM = 'goal-royaltomb';
// The map's graded copy (tools/art/goal-landmark-grade.py); the goal card keeps the ungraded unlocks/ copy.
const finishedUrls = import.meta.glob<string>('../assets/landmarks/goal-royaltomb-256.png', { eager: true, query: '?url', import: 'default' });
const FINISHED_URL = Object.values(finishedUrls)[0] ?? null;
let finished: HTMLImageElement | null = null;

/** The tile beside the city where its tomb is painted: the first neighbour, in fixed direction order, that is the holder's own dry, flat land with nothing standing on it. */
export function tombTile(view: Pick<PlayerView, 'tiles' | 'width' | 'height' | 'cities'>, at: Hex, holder: number, taken: ReadonlySet<number>): { tile: number; hex: Hex } | null {
  const cities = new Set(view.cities.map((c) => tileIndex(c.pos, view.width, view.height)));
  for (const hex of neighbors(at)) {
    const tile = tileIndex(hex, view.width, view.height);
    const t = tile >= 0 ? view.tiles[tile] : undefined;
    if (!t || t.owner !== holder || cities.has(tile) || taken.has(tile) || t.landmark || t.wonder) continue;
    if (t.terrain === null || t.terrain === 'Ocean' || t.terrain === 'Coast' || t.terrain === 'Mountains') continue;
    return { tile, hex };
  }
  return null;
}

const memo = new WeakMap<PlayerView, TombPiece[]>();
/** Every Royal Tomb the map paints: the standing ones (finished) and the ones under construction (staged by the work's turns). */
export function tombPieces(view: PlayerView): TombPiece[] {
  const hit = memo.get(view);
  if (hit) return hit;
  const taken = new Set<number>([...landmarksOf(view).map((l) => l.tile), ...sitesOf(view).map((s) => s.tile)]);
  const out: TombPiece[] = [];
  const add = (city: number, owner: number, holder: number, at: Hex, k: number) => {
    const spot = tombTile(view, at, holder, taken);
    if (!spot) return;
    taken.add(spot.tile);
    out.push({ city, owner, k, ...spot });
  };
  for (const g of goalCities(view)) if (g.kind === 'tomb') add(g.city, g.of, g.holder, g.at, 1);
  for (const g of tombSites(view)) {
    const st = tombStatus(view.players.find((p) => p.id === g.of) ?? {});
    add(g.city, g.of, g.holder, g.at, st ? Math.min(0.99, st.turns / Math.max(1, st.target)) : 0);
  }
  memo.set(view, out);
  return out;
}

function finishedImage(): HTMLImageElement | null {
  if (!FINISHED_URL || typeof Image === 'undefined') return null;
  if (!finished) { finished = new Image(); finished.onload = () => iconLoaded(); finished.src = FINISHED_URL; }
  return finished.complete && finished.naturalWidth ? finished : null;
}

/** Paint each tomb at its fixed footprint (`boxes`, chosen with the other landmarks). Far out, the banner's tomb seal stands for it. */
export function drawTombs(ctx: CanvasRenderingContext2D, view: PlayerView, toScreen: (p: { x: number; y: number }) => { x: number; y: number }, hexPx: number, w: number, h: number, dpr: number, boxes: ReadonlyMap<number, LandmarkBox>) {
  if (isFar(hexPx)) return;
  const pieces = tombPieces(view);
  if (!pieces.length) return;
  ctx.save();
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.imageSmoothingEnabled = true;
  ctx.imageSmoothingQuality = 'high';
  for (const piece of pieces) {
    const p = toScreen(hexToPixel(piece.hex, HEX_SIZE));
    if (p.x < -hexPx * 3 || p.x > w + hexPx * 3 || p.y < -hexPx * 3 || p.y > h + hexPx * 3) continue;
    const box = boxes.get(piece.tile) ?? landmarkBox(p, hexPx);
    const stage = constructionStage(piece.k);
    const img = stage === 4 ? finishedImage() : stage >= 1 ? stageImage(STEM, stage, box.s) : null;
    if (!img) continue;
    ctx.globalAlpha = view.tiles[piece.tile]?.visible ? 1 : 0.75;
    ctx.drawImage(img, box.x, box.y, box.s, box.s);
  }
  ctx.restore();
}
