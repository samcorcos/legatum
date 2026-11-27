import type { LookId } from '../brand/look';
import { centerX, centerY, HEX_SIZE, hexPath } from './grid';
import { lensMetrics, type LensTile } from './lenses';
import type { Camera } from './render';
import type { Rect } from './compass';
import { SETTLE_BANDS, type SiteBand } from './settleAdvice';
import { MAP_STYLES } from './style';
import { CAPITAL_GILT, FONTS } from './sprites';

/** Each band's colour and how much of it covers the ground, from the shared tokens (the legend reads the same ones). */
function settleColors(look: LookId): Record<SiteBand, { color: string; alpha: number }> {
  const css = typeof document === 'undefined' ? null : getComputedStyle(document.documentElement);
  const L = MAP_STYLES[look].lens;
  return Object.fromEntries(SETTLE_BANDS.map((band) => {
    const key = band.toLowerCase(), percent = parseFloat(css?.getPropertyValue(`--settle-${key}-a`).trim() ?? '');
    return [band, { color: css?.getPropertyValue(`--settle-${key}`).trim() || L.settle[band], alpha: Number.isFinite(percent) ? percent / 100 : L.settleAlpha[band] }];
  })) as Record<SiteBand, { color: string; alpha: number }>;
}

/** The Settle band's wash, shared by the 2D and 3D maps. */
export function lensWash(tiles: readonly LensTile[], look: LookId): (tile: LensTile) => { color: string; alpha: number } | null {
  const washes = tiles.some((t) => t.band) ? settleColors(look) : null;
  const L = MAP_STYLES[look].lens;
  return (tile) => {
    if (!tile.band || !washes) return null;
    const tier = washes[tile.band];
    return { color: tier.color, alpha: tier.alpha * (tile.faded ? L.faded / L.wash : 1) };
  };
}

/** The Yields overlay occupies the foot of each hex; Settle badges keep clear of its clusters. */
export interface LensDrawOptions {
  /** The 3D map: where each tile's centre is on screen (null when off camera) and the hex radius in px, instead of the 2D camera. */
  at?: (index: number) => { x: number; y: number } | null;
  hexPx?: number;
  yieldsOn?: boolean; /** The yield icon clusters in view: a recommendation badge keeps off them where it can. */ yieldRects?: readonly Rect[] }

/** Settle tint below tokens, with ranked site badges above them. */
export function drawLens(ctx: CanvasRenderingContext2D, tiles: LensTile[], cam: Camera, width: number, height: number, mapWidth: number, dpr: number, look: LookId, pass: 'tint' | 'labels' = 'tint', unitBottoms: ReadonlyMap<number, number> = new Map(), avoid: readonly Rect[] = [], medallionAbove: (index: number) => number = () => 0, opts: LensDrawOptions = {}) {
  const r = opts.hexPx ?? HEX_SIZE * cam.zoom;
  const at = opts.at ?? ((index: number) => ({ x: (centerX(index, mapWidth) - cam.x) * cam.zoom + width / 2, y: (centerY(index, mapWidth) - cam.y) * cam.zoom + height / 2 }));
  const metrics = lensMetrics(r);
  ctx.save();
  // The house VP gilt (the capital seal's ring).
  const COLONIA_GILT = CAPITAL_GILT;
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.textAlign = 'center'; ctx.textBaseline = 'middle';
  ctx.font = `700 ${metrics.font}px ${FONTS[look].sans}`;
  const L = MAP_STYLES[look].lens;
  const { paper, ink } = L;
  const washes = pass === 'tint' && tiles.some((tile) => tile.band) ? settleColors(look) : null;
  const badges: { rank: number; x: number; y: number; below: number; colonia?: boolean }[] = [];
  for (const tile of tiles) {
    // Yield clusters are drawn separately.
    if (!tile.band) continue;
    const p = at(tile.index);
    if (!p) continue;
    const x = p.x, center = p.y;
    const y = center;
    if (x < -r || x > width + r || y < -r || y > height + r) continue;
    if (pass === 'tint') {
      ctx.beginPath(); hexPath(ctx, x, y, r * 0.93);
      const tier = washes![tile.band];
      // A wash the terrain reads through.
      const fade = tile.faded ? L.faded / L.wash : 1;
      ctx.globalAlpha = tier.alpha * fade;
      ctx.fillStyle = tier.color; ctx.fill();
      ctx.globalAlpha = 1;
      // Only a thin accent line is stronger: the band's own green, firmer for better sites.
      // Firm even on the faintest band, so a Poor site is told from land that is not a site.
      ctx.globalAlpha = (0.7 + tier.alpha * 0.3) * fade; ctx.strokeStyle = tier.color; ctx.lineWidth = L.rim * 1.4; ctx.stroke();
      ctx.globalAlpha = 1;
      // The top three: an ink line with a paper edge outside it, so the ring shows on the deepest green too.
      if (tile.rank) { ctx.strokeStyle = paper; ctx.lineWidth = MAP_STYLES[look].pen.line + 2.4; ctx.stroke(); ctx.strokeStyle = ink; ctx.lineWidth = MAP_STYLES[look].pen.line; ctx.stroke(); }
      // Rome's Colonia Victrix: a site where the selected veteran founds it wears a gilt ring inside the hex (the legend names it).
      if (tile.colonia) {
        ctx.beginPath(); hexPath(ctx, x, y, r * 0.74);
        ctx.globalAlpha = tile.faded ? 0.75 : 1;
        ctx.strokeStyle = ink; ctx.lineWidth = Math.max(3.6, r * 0.14) + 1.6; ctx.stroke();
        ctx.strokeStyle = COLONIA_GILT; ctx.lineWidth = Math.max(3.6, r * 0.14); ctx.stroke();
        ctx.globalAlpha = 1;
      }
      continue;
    }
    if (!tile.rank) continue;
    const unitBottom = unitBottoms.get(tile.index);
    const onUnit = unitBottom !== undefined;
    if (tile.rank) {
      // A resource medallion sits at the top of the hex: the badge goes above it, not on it.
      const medallion = medallionAbove(tile.index);
      const above = medallion ? Math.max(r * (onUnit ? 0.9 : 0.8), medallion + 12) : r * (onUnit ? 0.9 : 0.8);
      // With the Yields overlay on, the badge keeps below its tile, past the tile's own icons and clear of the next row's, rather than over the icons of the tile above.
      badges.push({ rank: tile.rank, colonia: tile.colonia, x, y: center - above, below: center + Math.max(r * 0.8 + (opts.yieldsOn ? 6 : 0), (unitBottom ?? 0) + 11) });
    }
  }
  // A badge keeps off the tokens too, where it can: the units on a recommended site are what a player needs to see.
  const tokenRects: Rect[] = badges.length ? [...unitBottoms].flatMap(([index, bottom]) => { const p = at(index); return p ? [{ x: p.x - r * 0.55, y: p.y - r * 0.6, w: r * 1.1, h: bottom + r * 0.6 }] : []; }) : [];
  const placed: { x: number; y: number; w: number }[] = [];
  for (const badge of badges.sort((a, b) => a.rank - b.rank)) {
    // A Colonia founder's markers name the Colonia and wear the ring's gilt.
    const w = badge.rank === 1 ? (badge.colonia ? (metrics.compact ? 86 : 116) : (metrics.compact ? 50 : 68)) : 28;
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
    ctx.fillStyle = ctx.strokeStyle = badge.colonia ? COLONIA_GILT : L.badge;
    const anchor = Math.abs(y - badge.below) < Math.abs(y - badge.y) ? badge.below : badge.y;
    if (x !== badge.x || y !== anchor) {
      ctx.beginPath(); ctx.moveTo(badge.x, anchor); ctx.lineTo(x, y); ctx.lineWidth = 1.5; ctx.stroke();
    }
    ctx.fillRect(x - w / 2, y - 10, w, 20);
    if (badge.colonia) { ctx.strokeStyle = ink; ctx.lineWidth = 1; ctx.strokeRect(x - w / 2 + 0.5, y - 9.5, w - 1, 19); }
    ctx.fillStyle = badge.colonia ? ink : L.badgeText;
    ctx.font = `700 ${metrics.compact ? 9 : 12}px ${FONTS[look].sans}`;
    ctx.fillText(badge.rank === 1 ? (badge.colonia ? 'Best Colonia site' : 'Best spot') : `#${badge.rank}`, x, y);
    placed.push({ x, y, w });
  }
  ctx.restore();
}
