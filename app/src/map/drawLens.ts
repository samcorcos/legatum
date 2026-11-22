import type { LookId } from '../brand/look';
import { centerX, centerY, FAR_HEX_PX, HEX_SIZE, hexPath } from './grid';
import { LENS_ALPHA, LENS_FADED_ALPHA, lensMetrics, type LensTile } from './lenses';
import type { Camera } from './render';
import type { Rect } from './compass';
import { SETTLE_BANDS, type SiteBand } from './settleAdvice';
import { MAP_STYLES } from './style';
import { FONTS } from './sprites';
import { yieldIconSize } from './yieldIcons';

/** Each band's colour and how much of it covers the ground, from the shared tokens (the legend reads the same ones). */
function settleColors(look: LookId): Record<SiteBand, { color: string; alpha: number }> {
  const css = typeof document === 'undefined' ? null : getComputedStyle(document.documentElement);
  const L = MAP_STYLES[look].lens;
  return Object.fromEntries(SETTLE_BANDS.map((band) => {
    const key = band.toLowerCase(), percent = parseFloat(css?.getPropertyValue(`--settle-${key}-a`).trim() ?? '');
    return [band, { color: css?.getPropertyValue(`--settle-${key}`).trim() || L.settle[band], alpha: Number.isFinite(percent) ? percent / 100 : L.settleAlpha[band] }];
  })) as Record<SiteBand, { color: string; alpha: number }>;
}

/** `yieldsOn`: the Yields overlay owns the foot of the hex, so lens chips move to the side of the tile. */
export interface LensDrawOptions { yieldsOn?: boolean; /** The yield icon clusters in view: a recommendation badge keeps off them where it can. */ yieldRects?: readonly Rect[] }

/** Keep a lens chip clear of banners and map panels, including a neighbouring city's name. */
export function clearLensChip(x: number, y: number, w: number, h: number, avoid: readonly Rect[]): { x: number; y: number } | null {
  const hits = (px: number, py: number) => avoid.some((a) => px + w / 2 > a.x && px - w / 2 < a.x + a.w && py + h / 2 > a.y && py - h / 2 < a.y + a.h);
  if (!hits(x, y)) return { x, y };
  const candidates = avoid.flatMap((a) => [
    { x, y: a.y + a.h + h / 2 + 2 },
    { x: a.x - w / 2 - 2, y },
    { x: a.x + a.w + w / 2 + 2, y },
    { x, y: a.y - h / 2 - 2 },
  ]).sort((a, b) => Math.hypot(a.x - x, a.y - y) - Math.hypot(b.x - x, b.y - y));
  return candidates.find((p) => !hits(p.x, p.y)) ?? null;
}

/** Terrain tint below tokens; labels above them, kept inside an occupied hex below its token. */
export function drawLens(ctx: CanvasRenderingContext2D, tiles: LensTile[], cam: Camera, width: number, height: number, mapWidth: number, dpr: number, look: LookId, pass: 'tint' | 'labels' = 'tint', unitBottoms: ReadonlyMap<number, number> = new Map(), avoid: readonly Rect[] = [], medallionAbove: (index: number) => number = () => 0, opts: LensDrawOptions = {}) {
  const r = HEX_SIZE * cam.zoom;
  const metrics = lensMetrics(r);
  ctx.save();
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.textAlign = 'center'; ctx.textBaseline = 'middle';
  ctx.font = `700 ${metrics.font}px ${FONTS[look].sans}`;
  const L = MAP_STYLES[look].lens;
  const { paper, ink } = L;
  const washes = pass === 'tint' && tiles.some((tile) => tile.band) ? settleColors(look) : null;
  const badges: { rank: number; x: number; y: number; below: number }[] = [];
  for (const tile of tiles) {
    // Yield tiles have no tint or chip: yieldIcons.ts draws their icon clusters.
    if (tile.yields) continue;
    const x = (centerX(tile.index, mapWidth) - cam.x) * cam.zoom + width / 2;
    const center = (centerY(tile.index, mapWidth) - cam.y) * cam.zoom + height / 2;
    let y = center;
    if (x < -r || x > width + r || y < -r || y > height + r) continue;
    if (pass === 'tint') {
      ctx.beginPath(); hexPath(ctx, x, y, r * 0.93);
      const tier = tile.band ? washes![tile.band] : null;
      // A wash the terrain reads through.
      const fade = tile.faded ? L.faded / L.wash : 1;
      ctx.globalAlpha = tier ? tier.alpha * fade : tile.faded ? LENS_FADED_ALPHA : LENS_ALPHA;
      ctx.fillStyle = tier?.color ?? tile.tint; ctx.fill();
      ctx.globalAlpha = 1;
      // Only a thin accent line is stronger: the band's own green, firmer for better sites.
      if (tier) {
        // Firm even on the faintest band, so a Poor site is told from land that is not a site.
        ctx.globalAlpha = (0.7 + tier.alpha * 0.3) * fade; ctx.strokeStyle = tier.color; ctx.lineWidth = L.rim * 1.4; ctx.stroke();
        ctx.globalAlpha = 1;
      }
      // The top three: an ink line with a paper edge outside it, so the ring shows on the deepest green too.
      if (tile.rank) { ctx.strokeStyle = paper; ctx.lineWidth = MAP_STYLES[look].pen.line + 2.4; ctx.stroke(); ctx.strokeStyle = ink; ctx.lineWidth = MAP_STYLES[look].pen.line; ctx.stroke(); }
      continue;
    }
    // Pure tint treatments carry no chip.
    if (tile.label === undefined && !tile.rank) continue;
    const unitBottom = unitBottoms.get(tile.index);
    const onUnit = unitBottom !== undefined;
    if (tile.label) {
      // With the Yields overlay on the chip is a little smaller, so it fits above the icons' top row.
      const bw = Math.min(Math.max(metrics.font * tile.label.length * 0.67 + 5, 14), opts.yieldsOn ? r * 0.7 : r * 1.65);
      const bh = metrics.font + (opts.yieldsOn ? 2 : 4);
      let lx = x;
      // Below the lowest token of a stack, but never past the hex's lower edge (so it can't read as the next hex's label).
      if (onUnit) y = Math.min(center + unitBottom + 2 + bh / 2, center + r * 0.8 - bh / 2);
      // With the Yields overlay on, the foot of the hex is the yield icons' place and a middle chip would sit on a stack's top row:
      // the chip moves to the side of an empty tile, and beside the token on an occupied one.
      if (opts.yieldsOn) {
        const side = r * 0.87 - bw / 2 - 1.5;
        if (onUnit) {
          const half = unitBottom - 4;
          lx = x + Math.min(half + 3 + bw / 2, r * 0.87 + 3 - bw / 2); y = center - half * 0.55;
        } else {
          // Level with the upper third of the hex, and always above the top row of the tallest yield stack (a 3 or 4 point stack is 1.68 icons high).
          const stackTop = r * 0.64 - 1.68 * yieldIconSize(r);
          lx = x - side; y = center + Math.min(-r * 0.28, stackTop - bh / 2 - 1);
        }
      }
      const place = clearLensChip(lx, y, bw, bh, avoid);
      if (place) {
        ctx.fillStyle = paper; ctx.globalAlpha = 0.95; ctx.fillRect(place.x - bw / 2, place.y - bh / 2, bw, bh); ctx.globalAlpha = 1;
        ctx.fillStyle = ink; ctx.fillText(tile.label, place.x, place.y, r * 1.55);
      }
    }
    if (tile.rank) {
      // A resource medallion sits at the top of the hex: the badge goes above it, not on it.
      const medallion = medallionAbove(tile.index);
      const above = medallion ? Math.max(r * (onUnit ? 0.9 : 0.8), medallion + 12) : r * (onUnit ? 0.9 : 0.8);
      // With the Yields overlay on, the badge keeps below its tile, past the tile's own icons and clear of the next row's, rather than over the icons of the tile above.
      badges.push({ rank: tile.rank, x, y: center - above, below: center + Math.max(r * 0.8 + (opts.yieldsOn ? 6 : 0), (unitBottom ?? 0) + 11) });
    }
  }
  // A badge keeps off the tokens too, where it can: the units on a recommended site are what a player needs to see.
  const tokenRects: Rect[] = badges.length ? [...unitBottoms].map(([index, bottom]) => ({ x: (centerX(index, mapWidth) - cam.x) * cam.zoom + width / 2 - r * 0.55, y: (centerY(index, mapWidth) - cam.y) * cam.zoom + height / 2 - r * 0.6, w: r * 1.1, h: bottom + r * 0.6 })) : [];
  const placed: { x: number; y: number; w: number }[] = [];
  for (const badge of badges.sort((a, b) => a.rank - b.rank)) {
    const w = badge.rank === 1 ? (metrics.compact ? 50 : 68) : 28;
    const clampX = (x: number) => Math.max(w / 2 + 4, Math.min(width - w / 2 - 4, x));
    const clampY = (y: number) => Math.max(14, Math.min(height - 14, y));
    // A tight cluster can require a side step as well as a vertical one. Keep the rank order,
    // preferring spots nearest the hex, then skip a badge if every visible position is under HUD.
    const candidates = [badge.y, badge.below].flatMap((base) =>
      [0, -22, 22, -44, 44, -66, 66, -88, 88, -110, 110].flatMap((dy) =>
        [0, -1, 1, -2, 2].map((dx) => ({ x: clampX(badge.x + dx * (w + 8)), y: clampY(base + dy) }))))
      .sort((a, b) => Math.hypot(a.x - badge.x, a.y - badge.y) - Math.hypot(b.x - badge.x, b.y - badge.y));
    // ...and prefers the below side: the tile above's icons end just over this tile's top edge.
    if (opts.yieldsOn) {
      const mid = (badge.y + badge.below) / 2;
      const cost = (c: { x: number; y: number }) => Math.hypot(c.x - badge.x, c.y - badge.below) + (c.y > mid ? 0 : 60);
      candidates.sort((a, b) => cost(a) - cost(b));
    }
    const covered = (x: number, y: number) => avoid.some((a) => x + w / 2 > a.x && x - w / 2 < a.x + a.w && y + 10 > a.y && y - 10 < a.y + a.h);
    const free = (x: number, y: number) => placed.every((p) => x + w / 2 + 2 <= p.x - p.w / 2 || x - w / 2 - 2 >= p.x + p.w / 2 || y + 22 <= p.y || y - 22 >= p.y);
    // Where the Yields overlay is on, the first spot that also leaves every icon cluster uncovered; failing that, the first spot that is merely free.
    const clear = (x: number, y: number) => ![...(opts.yieldRects ?? []), ...tokenRects].some((c) => x + w / 2 > c.x && x - w / 2 < c.x + c.w && y + 10 > c.y && y - 10 < c.y + c.h);
    const position = candidates.find(({ x, y }) => free(x, y) && !covered(x, y) && clear(x, y)) ?? candidates.find(({ x, y }) => free(x, y) && !covered(x, y));
    if (!position) continue;
    const { x, y } = position;
    // At overview zoom nearby recommendations need a short leader to stay distinct.
    ctx.fillStyle = ctx.strokeStyle = L.badge;
    const anchor = Math.abs(y - badge.below) < Math.abs(y - badge.y) ? badge.below : badge.y;
    if (x !== badge.x || y !== anchor) {
      ctx.beginPath(); ctx.moveTo(badge.x, anchor); ctx.lineTo(x, y); ctx.lineWidth = 1.5; ctx.stroke();
    }
    ctx.fillRect(x - w / 2, y - 10, w, 20);
    ctx.fillStyle = L.badgeText;
    ctx.font = `700 ${metrics.compact ? 9 : 12}px ${FONTS[look].sans}`;
    ctx.fillText(badge.rank === 1 ? 'Best spot' : `#${badge.rank}`, x, y);
    placed.push({ x, y, w });
  }
  ctx.restore();
}

/**
 * The Borders overlay: each owned tile gets a light wash in its owner's colour and a hairline, and the tiles on a
 * border carry the owner's number in a small roundel at the tile's upper right, clear of the middle (where lens chips
 * sit), the left (where they move when Yields is on) and the foot (yield icons). `washScale` lightens the wash under a lens that tints (0 under Settle, whose
 * bands must stay true); the hairline always stays, thicker when the wash steps back.
 */
export function drawOwnerOverlay(ctx: CanvasRenderingContext2D, tiles: readonly LensTile[], cam: Camera, width: number, height: number, mapWidth: number, dpr: number, look: LookId, pass: 'tint' | 'labels', washScale: number, busy: ReadonlySet<number> = new Set()) {
  const r = HEX_SIZE * cam.zoom;
  const L = MAP_STYLES[look].lens, pen = MAP_STYLES[look].pen;
  ctx.save();
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.textAlign = 'center'; ctx.textBaseline = 'middle';
  for (const tile of tiles) {
    const x = (centerX(tile.index, mapWidth) - cam.x) * cam.zoom + width / 2;
    const y = (centerY(tile.index, mapWidth) - cam.y) * cam.zoom + height / 2;
    if (x < -r || x > width + r || y < -r || y > height + r) continue;
    const fade = tile.faded ? 0.7 : 1;
    if (pass === 'tint') {
      ctx.beginPath(); hexPath(ctx, x, y, r * 0.93);
      if (washScale > 0) { ctx.globalAlpha = L.ownerWash * washScale * fade; ctx.fillStyle = tile.tint; ctx.fill(); }
      ctx.globalAlpha = L.rimAlpha * fade; ctx.strokeStyle = tile.tint; ctx.lineWidth = washScale > 0 ? pen.hair : pen.thin; ctx.stroke();
      ctx.globalAlpha = 1;
      continue;
    }
    if (!tile.label || r < FAR_HEX_PX || busy.has(tile.index)) continue;
    const radius = Math.min(8, Math.max(5.5, r * 0.22));
    const rx = x + (r * 0.87 - radius - 1.5), ry = y - r * 0.5;
    ctx.globalAlpha = 0.95 * fade;
    ctx.beginPath(); ctx.roundRect(rx - radius, ry - radius, 2 * radius, 2 * radius, radius * 0.3); ctx.fillStyle = L.paper; ctx.fill();
    ctx.lineWidth = 1.6; ctx.strokeStyle = tile.tint; ctx.stroke();
    ctx.globalAlpha = fade;
    ctx.font = `700 ${Math.round(radius * 1.4)}px ${FONTS[look].sans}`;
    ctx.fillStyle = L.ink; ctx.fillText(tile.label, rx, ry + 0.5);
    ctx.globalAlpha = 1;
  }
  ctx.restore();
}
