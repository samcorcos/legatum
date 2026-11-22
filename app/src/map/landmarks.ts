// Constructed wonders use their publicly visible, chosen tile and the painted world wonder art.
import type { BuildingKind, Faction, Hex, PlayerView } from '../types';
import { buildingIconName, itemName } from '../data';
import { hexAt, hexToPixel } from '../hex';
import { buildingArtSlug, itemArtUrl } from '../unlockArt';
import { tintedIcon } from '../icons';
import { HEX_SIZE, isFar } from './grid';
import { ATLAS } from './style';

export interface Landmark { city: number; owner: number; building: BuildingKind; tile: number; hex: Hex }

const memo = new WeakMap<PlayerView, Landmark[]>();

/** Every constructed wonder the view shows, each on its tile. */
export function landmarksOf(view: PlayerView): Landmark[] {
  const hit = memo.get(view);
  if (hit) return hit;
  const out: Landmark[] = view.tiles.flatMap((t, tile) => t.landmark
    ? [{ city: t.landmark.city, owner: t.owner ?? t.landmark.of, building: t.landmark.building, tile, hex: hexAt(tile, view.width) }]
    : []);
  memo.set(view, out);
  return out;
}

// The map's own grade of a landmark painting (muted to the towns' palette, ground washed into the hex). The research
// cards and production list use the ungraded copy in assets/unlocks (tools/art/card-grade.py, art audit 2026-11-22).
const mapGraded = import.meta.glob<string>('../assets/landmarks/*-256.png', { eager: true, query: '?url', import: 'default' });
const mapGradedBySlug = new Map(Object.entries(mapGraded).map(([path, url]) => [/\/([a-z0-9-]+)-256\.png$/.exec(path)![1], url]));
/** The landmark's map painting: the map-graded copy when there is one, else the unlock art. */
export function landmarkArtUrl(building: BuildingKind, faction: Faction | undefined): string | null {
  return mapGradedBySlug.get(buildingArtSlug(building, faction)) ?? mapGradedBySlug.get(buildingArtSlug(building))
    ?? itemArtUrl({ kind: 'Building', building }, faction, 256);
}

const art = new Map<string, HTMLImageElement | null>();
const listeners = new Set<() => void>();
/** Called when a landmark painting finishes decoding, so the map repaints (there is no placeholder to stand in). */
export function onLandmarkArtLoad(cb: () => void): () => void {
  listeners.add(cb);
  return () => listeners.delete(cb);
}
export function landmarkImage(building: BuildingKind, faction: Faction | undefined): HTMLImageElement | null {
  const url = landmarkArtUrl(building, faction);
  if (!url || typeof Image === 'undefined') return null;
  let img = art.get(url);
  if (img === undefined) {
    img = new Image();
    img.onload = img.onerror = () => { for (const cb of listeners) cb(); };
    img.src = url;
    art.set(url, img);
  }
  return img && img.complete && img.naturalWidth > 0 ? img : null;
}

/** A long work's name shortened to its first and last word ("ITER Fusion Reactor" -> "ITER Reactor") so the label stays on its tile. */
export function shortLabel(name: string): string {
  const words = name.split(' ');
  return name.length > 14 && words.length > 2 ? `${words[0]} ${words[words.length - 1]}` : name;
}

/** The solid part of a painting (its building, away from the transparent margin and the faded ground edge). */
export function landmarkCore(b: { x: number; y: number; s: number }) {
  return { x: b.x + b.s * 0.08, y: b.y + b.s * 0.22, w: b.s * 0.84, h: b.s * 0.68 };
}

/** Each world wonder fills at least the city's painted footprint on its own wonder tile. */
export function landmarkBox(c: { x: number; y: number }, hexPx: number, covers: readonly { x: number; y: number; w: number; h: number }[] = []) {
  const s = hexPx * 3;
  const base = { x: c.x - s / 2, y: c.y - s * 0.36, s };
  // Banners and earlier landmarks determine the footprint. Moving units must never be covers.
  const candidates = [base, ...[-0.7, 0.7].flatMap(dx => [0, 0.55].map(dy => ({ x: base.x + dx * hexPx, y: base.y + dy * hexPx, s }))), { ...base, y: base.y + 0.8 * hexPx }];
  const overlap = (a: { x: number; y: number; w: number; h: number }, r: { x: number; y: number; w: number; h: number }) =>
    Math.max(0, Math.min(a.x + a.w, r.x + r.w) - Math.max(a.x, r.x)) * Math.max(0, Math.min(a.y + a.h, r.y + r.h) - Math.max(a.y, r.y));
  // Whole-box overlap keeps clear of banners; the solid core counts twice more so the facade stays clear.
  const covered = (b: typeof base) => covers.reduce((sum, r) => sum + overlap({ x: b.x, y: b.y, w: s, h: s }, r) + 2 * overlap(landmarkCore(b), r), 0);
  return candidates.reduce((best, b) => covered(b) < covered(best) ? b : best, base);
}

export type LandmarkBox = ReturnType<typeof landmarkBox>;

/** One deterministic footprint per tile for finished works, sites, and the construction animation. */
export function landmarkBoxes(sites: readonly { tile: number; hex: Hex }[], toScreen: (p: { x: number; y: number }) => { x: number; y: number }, hexPx: number, banners: readonly { x: number; y: number; w: number; h: number }[]): Map<number, LandmarkBox> {
  const boxes = new Map<number, LandmarkBox>();
  const covers = [...banners];
  for (const site of [...sites].sort((a, b) => a.tile - b.tile)) {
    if (boxes.has(site.tile)) continue;
    const box = landmarkBox(toScreen(hexToPixel(site.hex, HEX_SIZE)), hexPx, covers);
    boxes.set(site.tile, box);
    covers.push(landmarkCore(box));
  }
  return boxes;
}

/**
 * Draw the landmarks in view (screen space), below units and clear of banners. `skip` holds back
 * a landmark whose construction is still playing (momentFx.ts draws it rising instead).
 */
export function drawLandmarks(ctx: CanvasRenderingContext2D, view: PlayerView, toScreen: (p: { x: number; y: number }) => { x: number; y: number }, hexPx: number, w: number, h: number, dpr: number, skip?: (l: Landmark) => boolean, boxes: ReadonlyMap<number, LandmarkBox> = new Map()) {
  const list = landmarksOf(view);
  if (!list.length) return;
  const far = isFar(hexPx);
  ctx.save();
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.imageSmoothingEnabled = true;
  ctx.imageSmoothingQuality = 'high';
  for (const l of list) {
    if (skip?.(l)) continue;
    const c = toScreen(hexToPixel(l.hex, HEX_SIZE));
    if (c.x < -hexPx * 2 || c.x > w + hexPx * 2 || c.y < -hexPx * 2 || c.y > h + hexPx * 2) continue;
    const seen = view.tiles[l.tile]?.visible ?? false;
    ctx.globalAlpha = seen ? 1 : 0.75;
    if (far) drawBadge(ctx, l, c.x, c.y, hexPx, dpr);
    else {
      const img = landmarkImage(l.building, view.players.find((p) => p.id === l.owner)?.faction);
      const b = boxes.get(l.tile) ?? landmarkBox(c, hexPx);
      // Every work has painted art; a painting still decoding waits (onLandmarkArtLoad repaints) rather than showing a stand-in.
      if (!img) continue;
      ctx.drawImage(img, b.x, b.y, b.s, b.s);
      const label = shortLabel(itemName({ kind: 'Building', building: l.building }, view.players.find(p => p.id === l.owner)?.faction)).toUpperCase();
      const size = Math.max(9, Math.min(13, hexPx * .24));
      ctx.font = `600 ${size}px Georgia, serif`; ctx.textAlign = 'center'; ctx.textBaseline = 'alphabetic';
      const x = b.x + b.s / 2, y = b.y + b.s * .98;
      const half = Math.min(b.s * .4, ctx.measureText(label).width * .55);
      ctx.strokeStyle = ATLAS.tone.gold; ctx.lineWidth = .8;
      ctx.beginPath(); ctx.moveTo(x - half, y - size * 1.3); ctx.lineTo(x + half, y - size * 1.3); ctx.stroke();
      ctx.fillStyle = ATLAS.tone.ink; ctx.fillText(label, x, y);
    }
  }
  ctx.restore();
}

/** Far tier: a parchment disc with a gilt ring and the work's own icon, bolder than the detailed art it replaces. */
function drawBadge(ctx: CanvasRenderingContext2D, l: Landmark, x: number, y: number, hexPx: number, dpr: number) {
  const r = Math.max(7, hexPx * 0.5);
  ctx.beginPath(); ctx.arc(x, y, r, 0, Math.PI * 2);
  ctx.fillStyle = ATLAS.tone.vellum; ctx.fill();
  ctx.lineWidth = Math.max(1.6, r * 0.22); ctx.strokeStyle = ATLAS.tone.gold; ctx.stroke();
  ctx.lineWidth = 1; ctx.strokeStyle = ATLAS.tone.ink;
  ctx.beginPath(); ctx.arc(x, y, r + ctx.lineWidth, 0, Math.PI * 2); ctx.stroke();
  const g = r * 1.3;
  const icon = tintedIcon(buildingIconName(l.building), ATLAS.tone.ink, Math.ceil(g * dpr));
  if (icon) ctx.drawImage(icon, x - g / 2, y - g / 2, g, g);
}
