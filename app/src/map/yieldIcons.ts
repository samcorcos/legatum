// Tile yields drawn the Civ 6 way: each yield is its icon repeated once per point, in small
// outlined clusters near the bottom of the hex, with no chip behind them. The icon count is the
// glance; the tile card holds the exact numbers.
import type { LookId } from '../brand/look';
import { tintedIcon } from '../icons';
import { FONTS } from './sprites';
import { ATLAS } from './style';
import { centerX, centerY, HEX_SIZE } from './grid';
import type { LensTile, LensYield } from './lenses';
import type { Camera } from './render';
import type { Rect } from './compass';

export type YieldKind = LensYield['kind'];
/** Fixed cluster order, left to right. */
export const YIELD_ORDER: readonly YieldKind[] = ['food', 'production', 'gold', 'science'];
/** From this value up a type shows one icon and a numeral instead of a stack. */
export const COLLAPSE_AT = 5;

// The same paper-deepened inks the yield bar and tooltips use (--y-* in brand/tokens.css, so the
// one metaphor reads the same everywhere), kept legible on any terrain by a hairline of parchment
// round the glyph, like the map's lettering, not by a dark sticker outline.
const FILL: Record<YieldKind, string> = { food: '#4e762e', production: '#965020', gold: '#80620d', science: '#236b92' };
/** The yield inks, shared with the wonder badge so one metaphor keeps one colour. */
export const YIELD_FILL = FILL;
const RIM: Record<LookId, string> = { atlas: 'rgba(247, 241, 227, 0.92)' };
const NUMERAL = '#2b2419';

interface YieldMark { kind: YieldKind; x: number; y: number; numeral?: number }
export interface YieldCluster { marks: YieldMark[]; width: number; height: number }

/**
 * Offsets (icon centers, relative to the cluster's center) for one tile. Same-type icons overlap
 * like a Civ stack: 1 alone, 2 side by side, 3 as a triangle, 4 as a square; 5+ collapse to one
 * icon plus a numeral. Zero or negative values draw nothing.
 */
export function yieldCluster(yields: readonly { kind: YieldKind; value: number }[], icon: number): YieldCluster {
  const step = icon * 0.68, gap = icon * 0.2;
  const numeralW = (value: number) => icon * (0.12 + 0.56 * String(value).length);
  const groups = YIELD_ORDER.flatMap((kind) => {
    const value = yields.find((y) => y.kind === kind)?.value ?? 0;
    if (value <= 0) return [];
    const collapsed = value >= COLLAPSE_AT;
    const n = collapsed ? 1 : value;
    const cols = n === 1 ? 1 : 2;
    return [{ kind, value, collapsed, n, width: icon + (cols - 1) * step + (collapsed ? numeralW(value) : 0), height: icon + (n > 2 ? step : 0) }];
  });
  if (!groups.length) return { marks: [], width: 0, height: 0 };
  const width = groups.reduce((w, g) => w + g.width, 0) + gap * (groups.length - 1);
  const height = Math.max(...groups.map((g) => g.height));
  const marks: YieldMark[] = [];
  let left = -width / 2;
  for (const g of groups) {
    const base = height / 2 - icon / 2; // bottom row's center
    const at = (col: number, row: number) => ({ x: left + icon / 2 + col * step, y: base - row * step });
    if (g.collapsed) marks.push({ kind: g.kind, ...at(0, 0), numeral: g.value });
    else if (g.n === 1) marks.push({ kind: g.kind, ...at(0, 0) });
    else if (g.n === 2) marks.push({ kind: g.kind, ...at(0, 0) }, { kind: g.kind, ...at(1, 0) });
    else if (g.n === 3) marks.push({ kind: g.kind, ...at(0.5, 1) }, { kind: g.kind, ...at(0, 0) }, { kind: g.kind, ...at(1, 0) });
    else marks.push({ kind: g.kind, ...at(0, 1) }, { kind: g.kind, ...at(1, 1) }, { kind: g.kind, ...at(0, 0) }, { kind: g.kind, ...at(1, 0) });
    left += g.width + gap;
  }
  return { marks, width, height };
}

/** Icons from a comfortable default zoom, one dot per type when zoomed out, nothing at the widest views. */
export function yieldDetail(hexPixels: number): 'icons' | 'dots' | 'none' {
  return hexPixels >= 18 ? 'icons' : hexPixels >= 11 ? 'dots' : 'none';
}
export const yieldIconSize = (hexPixels: number) => Math.min(17, Math.max(7, hexPixels * 0.36));

const sprites = new Map<string, HTMLCanvasElement>();
/** The yield icon with a dark rim, `pad` device pixels on each side (cached). */
function rimmed(kind: YieldKind, look: LookId, px: number, pad: number): HTMLCanvasElement | null {
  const key = `${kind}|${look}|${px}|${pad}`;
  const hit = sprites.get(key);
  if (hit) return hit;
  const fill = tintedIcon(`yield-${kind}`, FILL[kind], px), rim = tintedIcon(`yield-${kind}`, RIM[look], px);
  if (!fill || !rim) return null;
  const c = document.createElement('canvas');
  c.width = c.height = fill.width + pad * 2;
  const g = c.getContext('2d')!;
  for (let a = 0; a < 8; a++) g.drawImage(rim, pad + Math.round(Math.cos(a * Math.PI / 4) * pad), pad + Math.round(Math.sin(a * Math.PI / 4) * pad));
  g.drawImage(fill, pad, pad);
  sprites.set(key, c);
  return c;
}

/** A crowded tile shrinks its icons rather than spill into the neighbouring hexes. */
function fittedCluster(yields: readonly LensYield[], r: number): { cluster: YieldCluster; size: number } {
  const icon = yieldIconSize(r);
  let cluster = yieldCluster(yields, icon), size = icon;
  if (cluster.width > r * 1.2) { size = Math.max(icon * 0.6, icon * r * 1.2 / cluster.width); cluster = yieldCluster(yields, size); }
  return { cluster, size };
}

/** Screen rects of the icon clusters in view (none zoomed out, where they are dots), so a callout can be placed clear of them. */
export function yieldClusterRects(tiles: readonly LensTile[], cam: Camera, width: number, height: number, mapWidth: number): Rect[] {
  const r = HEX_SIZE * cam.zoom;
  if (yieldDetail(r) !== 'icons') return [];
  const rects: Rect[] = [];
  for (const tile of tiles) {
    if (!tile.yields?.length) continue;
    const x = (centerX(tile.index, mapWidth) - cam.x) * cam.zoom + width / 2;
    const cy = (centerY(tile.index, mapWidth) - cam.y) * cam.zoom + height / 2;
    if (x < -r || x > width + r || cy < -r || cy > height + r) continue;
    const { cluster } = fittedCluster(tile.yields, r);
    rects.push({ x: x - cluster.width / 2, y: cy + r * 0.64 - cluster.height, w: cluster.width, h: cluster.height });
  }
  return rects;
}

/** Draw every yield tile's cluster. Runs before units, cities and routes so they stay on top. */
export function drawYieldIcons(ctx: CanvasRenderingContext2D, tiles: readonly LensTile[], cam: Camera, width: number, height: number, mapWidth: number, dpr: number, look: LookId, heavyRim = false) {
  const r = HEX_SIZE * cam.zoom;
  const detail = yieldDetail(r);
  if (detail === 'none') return;
  // A deep lens tint (Settle's greens) is close to the food ink, so the parchment rim round each glyph is what carries it: heavier there.
  const pad = Math.max(1, Math.round(dpr * (heavyRim ? 1.8 : 1)));
  const dot = Math.max(2.2, r * 0.13);
  ctx.save();
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.textAlign = 'left'; ctx.textBaseline = 'middle';
  ctx.lineJoin = 'round';
  for (const tile of tiles) {
    if (!tile.yields?.length) continue;
    const x = (centerX(tile.index, mapWidth) - cam.x) * cam.zoom + width / 2;
    const cy = (centerY(tile.index, mapWidth) - cam.y) * cam.zoom + height / 2;
    if (x < -r || x > width + r || cy < -r || cy > height + r) continue;
    ctx.globalAlpha = tile.faded ? 0.7 : 1;
    if (detail === 'dots') {
      const kinds = YIELD_ORDER.filter((k) => tile.yields!.some((y) => y.kind === k && y.value > 0));
      kinds.forEach((k, i) => {
        ctx.beginPath();
        ctx.arc(x + (i - (kinds.length - 1) / 2) * dot * 2.6, cy + r * 0.55, dot, 0, Math.PI * 2);
        ctx.fillStyle = FILL[k]; ctx.fill();
        ctx.lineWidth = ATLAS.pen.thin; ctx.strokeStyle = RIM[look]; ctx.stroke();
      });
      continue;
    }
    const { cluster, size } = fittedCluster(tile.yields, r);
    // Bottom-centre of the hex (pointy-top), clear of the bottom vertex.
    const oy = cy + r * 0.64 - cluster.height / 2;
    for (const m of cluster.marks) {
      const s = rimmed(m.kind, look, Math.round(size * dpr), pad);
      if (s) ctx.drawImage(s, x + m.x - size / 2 - pad / dpr, oy + m.y - size / 2 - pad / dpr, s.width / dpr, s.height / dpr);
      if (m.numeral !== undefined) {
        const tx = x + m.x + size / 2 + size * 0.06, ty = oy + m.y + size * 0.18;
        ctx.font = `700 ${Math.round(size * 0.82)}px ${FONTS[look].sans}`;
        ctx.lineWidth = ATLAS.pen.heavy; ctx.strokeStyle = RIM[look]; ctx.strokeText(String(m.numeral), tx, ty);
        ctx.fillStyle = NUMERAL; ctx.fillText(String(m.numeral), tx, ty);
      }
    }
  }
  ctx.restore();
}
