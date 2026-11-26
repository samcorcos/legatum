// Tile yields drawn the Civ 6 way: each yield is its icon repeated once per point, in small
// outlined clusters near the bottom of the hex, with no chip behind them. The icon count is the
// glance; the tile card holds the exact numbers.
import type { LookId } from '../brand/look';
import { tintedIcon } from '../icons';
import { FONTS } from './sprites';
import { ATLAS } from './style';
import { centerX, centerY, HEX_SIZE } from './grid';
import type { LensTile, LensYield } from './lenses';
import { wonderBadgeText } from '../wonders';
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

/** `tag`: the natural wonder's bonus ("+25%") as a small parchment pill after its yield's icons (`x` is its left edge); `tagIcon` when the tile has none of that yield, so the pill carries the icon. */
interface YieldMark { kind: YieldKind; x: number; y: number; numeral?: number; tag?: string; tagIcon?: boolean }
export interface YieldCluster { marks: YieldMark[]; width: number; height: number }

/**
 * The folded bonus pill keeps a fixed, readable size (CSS px) whatever the icons shrink to: the old wonder pill's look at its
 * smallest font. Its width is estimated (0.62 em a character for the bold "+25%") so layout needs no canvas.
 */
export const BONUS_TAG_FONT = 10;
const TAG_PAD = 3, TAG_ICON = 10.5;
export function bonusTagSize(withIcon: boolean): { width: number; height: number } {
  const text = BONUS_TAG_FONT * 0.62 * wonderBadgeText().length;
  return { width: TAG_PAD * 2 + text + (withIcon ? 1.5 + TAG_ICON : 0), height: BONUS_TAG_FONT * 1.5 };
}

/**
 * Offsets (icon centers, relative to the cluster's center) for one tile. Same-type icons overlap
 * like a Civ stack: 1 alone, 2 side by side, 3 as a triangle, 4 as a square; 5+ collapse to one
 * icon plus a numeral. Zero or negative values draw nothing. `bonus`: a natural wonder's boosted yield, whose "+25%" pill
 * follows that yield's icons (or ends the cluster, carrying the icon, when the tile has none of it).
 */
export function yieldCluster(yields: readonly { kind: YieldKind; value: number }[], icon: number, bonus?: YieldKind): YieldCluster {
  const step = icon * 0.68, gap = icon * 0.2;
  const numeralW = (value: number) => icon * (0.12 + 0.56 * String(value).length);
  type Group = { kind: YieldKind; value: number; collapsed: boolean; n: number; tag?: boolean; tagIcon?: boolean; width: number; height: number };
  const groups: Group[] = YIELD_ORDER.flatMap((kind) => {
    const value = yields.find((y) => y.kind === kind)?.value ?? 0;
    if (value <= 0) return [];
    const collapsed = value >= COLLAPSE_AT;
    const n = collapsed ? 1 : value;
    const cols = n === 1 ? 1 : 2;
    const g: Group = { kind, value, collapsed, n, width: icon + (cols - 1) * step + (collapsed ? numeralW(value) : 0), height: icon + (n > 2 ? step : 0) };
    return kind === bonus ? [g, { kind, value: 0, collapsed: false, n: 0, tag: true, ...bonusTagSize(false) }] : [g];
  });
  if (bonus && !groups.some((g) => g.kind === bonus)) groups.push({ kind: bonus, value: 0, collapsed: false, n: 0, tag: true, tagIcon: true, ...bonusTagSize(true) });
  if (!groups.length) return { marks: [], width: 0, height: 0 };
  const width = groups.reduce((w, g) => w + g.width, 0) + gap * (groups.length - 1);
  const height = Math.max(...groups.map((g) => g.height));
  const marks: YieldMark[] = [];
  let left = -width / 2;
  for (const g of groups) {
    const base = height / 2 - Math.min(icon, height) / 2; // bottom row's center
    const at = (col: number, row: number) => ({ x: left + icon / 2 + col * step, y: base - row * step });
    if (g.tag) marks.push({ kind: g.kind, x: left, y: base, tag: wonderBadgeText(), ...(g.tagIcon ? { tagIcon: true } : {}) });
    else if (g.collapsed) marks.push({ kind: g.kind, ...at(0, 0), numeral: g.value });
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

/**
 * A crowded tile shrinks its icons rather than spill into the neighbouring hexes. With a `bonus` the pill keeps its size
 * and only the icons shrink; when they would have to go below 0.6 of their size to fit, the bonus does not fold
 * (`undefined`): the tile keeps its wonder pill instead.
 */
function fittedCluster(yields: readonly LensYield[], r: number, bonus?: YieldKind): { cluster: YieldCluster; size: number } | undefined {
  const icon = yieldIconSize(r), budget = r * 1.2;
  let cluster = yieldCluster(yields, icon, bonus), size = icon;
  if (cluster.width <= budget) return { cluster, size };
  const fixed = bonus ? bonusTagSize(!yields.some((y) => y.kind === bonus && y.value > 0)).width : 0;
  size = icon * (budget - fixed) / (cluster.width - fixed);
  if (size < icon * 0.6) { if (bonus) return undefined; size = icon * 0.6; }
  cluster = yieldCluster(yields, size, bonus);
  return { cluster, size };
}

/** Where a tile's cluster sits on screen (its rect), or undefined when it is not drawn or (with `bonus`) cannot fold. */
function clusterRect(tile: LensTile, cam: Camera, width: number, height: number, mapWidth: number, towns: readonly Rect[], bonus = tile.bonus): Rect | undefined {
  const r = HEX_SIZE * cam.zoom;
  const x = (centerX(tile.index, mapWidth) - cam.x) * cam.zoom + width / 2;
  const cy = (centerY(tile.index, mapWidth) - cam.y) * cam.zoom + height / 2;
  if (x < -r || x > width + r || cy < -r || cy > height + r) return undefined;
  const fit = fittedCluster(tile.yields ?? [], r, bonus);
  if (!fit) return undefined;
  const { cluster } = fit;
  const top = cy + r * 0.64 - cluster.height, dx = clusterShift(x, top, cluster.width, cluster.height, r, towns);
  return { x: x + dx - cluster.width / 2, y: top, w: cluster.width, h: cluster.height };
}

/**
 * The wonder tiles whose bonus folds into their yield icons: icons are showing, the pill fits at its readable size inside
 * the hex, and no unit counter (`units`, screen rects) sits on the cluster. Every other wonder tile keeps its pill, which
 * steps clear of counters or draws above them, so the bonus is always readable and always shown exactly once.
 */
export function foldedBonuses(tiles: readonly LensTile[], cam: Camera, width: number, height: number, mapWidth: number, towns: readonly Rect[] = [], units: readonly Rect[] = []): Set<number> {
  const out = new Set<number>();
  if (yieldDetail(HEX_SIZE * cam.zoom) !== 'icons') return out;
  const hit = (a: Rect, b: Rect) => a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h;
  for (const tile of tiles) {
    if (!tile.bonus) continue;
    const rect = clusterRect(tile, cam, width, height, mapWidth, towns);
    // A counter's rank chip hangs below its square (half its height again), plus the clear gap the pill keeps (wonderBadgeAt).
    const pad = Math.max(4, HEX_SIZE * cam.zoom * 0.12);
    if (rect && !units.some((u) => hit({ x: u.x - pad, y: u.y - pad, w: u.w + pad * 2, h: u.h * 1.5 + pad * 2 }, rect))) out.add(tile.index);
  }
  return out;
}

/**
 * How far a cluster at `x` (its centre) slides sideways to keep clear of a neighbouring town's art (`towns`, screen rects):
 * away from the town, by the overlap plus a small gap, at most a quarter of the hex radius `r` so it stays on its own tile.
 */
export function clusterShift(x: number, top: number, w: number, h: number, r: number, towns: readonly Rect[]): number {
  const gap = 2;
  let dx = 0;
  for (const t of towns) {
    if (top >= t.y + t.h || top + h <= t.y) continue;
    const left = x + dx - w / 2, right = x + dx + w / 2;
    if (right + gap <= t.x || left - gap >= t.x + t.w) continue;
    dx += x < t.x + t.w / 2 ? t.x - gap - right : t.x + t.w + gap - left;
  }
  return Math.max(-r * 0.25, Math.min(r * 0.25, dx));
}

/** Screen rects of the icon clusters in view (none zoomed out, where they are dots), so a callout can be placed clear of them. */
export function yieldClusterRects(tiles: readonly LensTile[], cam: Camera, width: number, height: number, mapWidth: number, towns: readonly Rect[] = []): Rect[] {
  const r = HEX_SIZE * cam.zoom;
  if (yieldDetail(r) !== 'icons') return [];
  const rects: Rect[] = [];
  for (const tile of tiles) {
    if (!tile.yields?.length && !tile.bonus) continue;
    const rect = clusterRect(tile, cam, width, height, mapWidth, towns) ?? clusterRect(tile, cam, width, height, mapWidth, towns, undefined);
    if (rect && rect.w > 0) rects.push(rect);
  }
  return rects;
}

/** Draw every yield tile's cluster. Runs before units, cities and routes so they stay on top. */
export function drawYieldIcons(ctx: CanvasRenderingContext2D, tiles: readonly LensTile[], cam: Camera, width: number, height: number, mapWidth: number, dpr: number, look: LookId, heavyRim = false, towns: readonly Rect[] = [],
  /** The 3D map: each tile's centre on screen (null off camera) and the hex radius in px, instead of the 2D camera. */
  at?: (index: number) => { x: number; y: number } | null, hexPx?: number) {
  const r = hexPx ?? HEX_SIZE * cam.zoom;
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
    if (!tile.yields?.length && !tile.bonus) continue;
    const p = at ? at(tile.index) : { x: (centerX(tile.index, mapWidth) - cam.x) * cam.zoom + width / 2, y: (centerY(tile.index, mapWidth) - cam.y) * cam.zoom + height / 2 };
    if (!p) continue;
    const cx = p.x, cy = p.y;
    if (cx < -r || cx > width + r || cy < -r || cy > height + r) continue;
    ctx.globalAlpha = tile.faded ? 0.7 : 1;
    if (detail === 'dots') {
      const kinds = YIELD_ORDER.filter((k) => tile.yields?.some((y) => y.kind === k && y.value > 0));
      kinds.forEach((k, i) => {
        ctx.beginPath();
        ctx.arc(cx + (i - (kinds.length - 1) / 2) * dot * 2.6, cy + r * 0.55, dot, 0, Math.PI * 2);
        ctx.fillStyle = FILL[k]; ctx.fill();
        ctx.lineWidth = ATLAS.pen.thin; ctx.strokeStyle = RIM[look]; ctx.stroke();
      });
      continue;
    }
    // A bonus that cannot fold here is the wonder pill's to show (foldedBonuses): the icons draw without it.
    const { cluster, size } = fittedCluster(tile.yields ?? [], r, tile.bonus) ?? fittedCluster(tile.yields ?? [], r)!;
    // Bottom-centre of the hex (pointy-top), clear of the bottom vertex, slid aside off a neighbouring town's walls.
    const oy = cy + r * 0.64 - cluster.height / 2;
    const x = cx + clusterShift(cx, oy - cluster.height / 2, cluster.width, cluster.height, r, towns);
    for (const m of cluster.marks) {
      if (m.tag !== undefined) {
        drawBonusTag(ctx, x + m.x, oy + m.y, m.kind, !!m.tagIcon, dpr, look);
        continue;
      }
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

/** The folded "+25%" as the wonder pill's small parchment pill (dark text, thin ink edge), at a fixed readable size; left edge at `left`. */
function drawBonusTag(ctx: CanvasRenderingContext2D, left: number, y: number, kind: YieldKind, withIcon: boolean, dpr: number, look: LookId) {
  const { width, height } = bonusTagSize(withIcon);
  ctx.save();
  ctx.beginPath(); ctx.roundRect(left, y - height / 2, width, height, height / 2);
  ctx.fillStyle = 'rgba(247, 241, 227, 0.95)'; ctx.fill();
  ctx.lineWidth = ATLAS.pen.thin; ctx.strokeStyle = '#564932'; ctx.stroke();
  ctx.font = `700 ${BONUS_TAG_FONT}px ${FONTS[look].sans}`;
  ctx.textAlign = 'left'; ctx.textBaseline = 'middle';
  ctx.fillStyle = NUMERAL; ctx.fillText(wonderBadgeText(), left + TAG_PAD, y + BONUS_TAG_FONT * 0.04);
  if (withIcon) {
    const icon = tintedIcon(`yield-${kind}`, FILL[kind], Math.round(TAG_ICON * dpr));
    if (icon) ctx.drawImage(icon, left + width - TAG_PAD - TAG_ICON, y - TAG_ICON / 2, TAG_ICON, TAG_ICON);
  }
  ctx.restore();
}
