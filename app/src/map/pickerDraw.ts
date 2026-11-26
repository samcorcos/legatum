// The tile pickers' marks (a city's improvable tiles, a placement or tile purchase, queued purchases and their prices),
// drawn by the 2D map and, through a projecting context, by the 3D map. Paths are in 2D world px; `px` is screen px
// per world px (1 on the 3D map's projecting context).
import type { Hex } from '../types';
import { hexToPixel } from '../hex';
import { rgba } from '../theme';
import { HEX_SIZE, hexPath } from './grid';
import { drawTileCandidates } from './airOverlay';
import type { MapStyle } from './style';
import type { ImproveOverlay } from './render';

const hxCenter = (hx: Hex) => hexToPixel(hx, HEX_SIZE);
const same = (a: Hex | null | undefined, b: Hex) => !!a && a.q === b.q && a.r === b.r;

export interface PlaceMarks { candidates: Hex[]; hover: Hex | null; prices?: { tile: Hex; price: number; affordable: boolean }[] }

/** Improvements: the selected city's improvable tiles are outlined (dashed, translucent); tiles it is
 *  already improving read solid; placed wonders are gilt. The hovered and clicked tiles come later (`drawImproveHot`). */
export function drawImproveTiles(ctx: CanvasRenderingContext2D, I: ImproveOverlay, own: string, S: MapStyle, px: number) {
  ctx.save();
  for (const hx of I.candidates) {
    if (same(I.hover, hx) || same(I.pick, hx)) continue; // drawn above the counters
    const p = hxCenter(hx);
    ctx.beginPath();
    hexPath(ctx, p.x, p.y, HEX_SIZE * 0.88);
    ctx.strokeStyle = rgba(own, 0.6);
    ctx.lineWidth = 1.4 * px;
    ctx.setLineDash([2 * px, 3 * px]);
    ctx.stroke();
  }
  for (const hx of I.building) {
    const p = hxCenter(hx);
    ctx.beginPath();
    hexPath(ctx, p.x, p.y, HEX_SIZE * 0.88);
    ctx.strokeStyle = rgba(own, 0.9);
    ctx.lineWidth = 2 * px;
    ctx.setLineDash([]);
    ctx.stroke();
  }
  ctx.setLineDash([]);
  for (const hx of I.sites) {
    const p = hxCenter(hx);
    ctx.beginPath();
    hexPath(ctx, p.x, p.y, HEX_SIZE * 0.84);
    ctx.fillStyle = rgba(S.tone.gold, 0.16);
    ctx.fill();
    ctx.strokeStyle = rgba(S.tone.paper, 0.95);
    ctx.lineWidth = 5 * px;
    ctx.stroke();
    ctx.strokeStyle = rgba(S.tone.gold, 1);
    ctx.lineWidth = 2.2 * px;
    ctx.stroke();
    ctx.beginPath();
    hexPath(ctx, p.x, p.y, HEX_SIZE * 0.66);
    ctx.lineWidth = 1.4 * px;
    ctx.stroke();
  }
  ctx.restore();
}

/** The hovered or clicked improvement tile: a heavy ring above the unit counters, so a unit standing
 *  on the tile never hides what the picker row points at. */
export function drawImproveHot(ctx: CanvasRenderingContext2D, I: ImproveOverlay, own: string, px: number) {
  ctx.save();
  ctx.setLineDash([]);
  for (const hx of [I.pick, I.hover]) {
    if (!hx || !I.candidates.some((c) => c.q === hx.q && c.r === hx.r)) continue;
    const p = hxCenter(hx);
    ctx.beginPath();
    hexPath(ctx, p.x, p.y, HEX_SIZE * 0.88);
    ctx.fillStyle = rgba(own, 0.12);
    ctx.fill();
    ctx.strokeStyle = rgba(own, 1);
    ctx.lineWidth = 4.5 * px;
    ctx.stroke();
  }
  ctx.restore();
}

/** A tile-chosen build being placed (Civ's district placement): every eligible tile lit. Buying a tile dims
 *  everything but the buyable tiles; the unaffordable ones read dimmer. The hovered tile's ring comes later. */
export function drawPlaceTiles(ctx: CanvasRenderingContext2D, pl: PlaceMarks, S: MapStyle, px: number, dim = true) {
  if (pl.prices) {
    if (dim) {
      ctx.save();
      ctx.beginPath();
      ctx.rect(-1e5, -1e5, 2e5, 2e5);
      for (const t of pl.prices) { const p = hxCenter(t.tile); hexPath(ctx, p.x, p.y, HEX_SIZE * 0.9); }
      ctx.fillStyle = rgba(S.tone.ink, 0.22);
      ctx.fill('evenodd');
      ctx.restore();
    }
    drawTileCandidates(ctx, pl.prices.filter((t) => !t.affordable).map((t) => t.tile), null, S.tone.ink, px, 0, 1.4);
  }
  drawTileCandidates(ctx, pl.candidates, null, S.tone.gold, px, 0.3, 4.2);
}

/** A purchase queued this turn: the tile washed and ringed in the seat's colour. */
export function drawQueuedBuys(ctx: CanvasRenderingContext2D, buys: { tile: Hex }[], own: string, px: number) {
  ctx.save();
  for (const b of buys) {
    const p = hxCenter(b.tile);
    ctx.beginPath();
    hexPath(ctx, p.x, p.y, HEX_SIZE * 0.9);
    ctx.fillStyle = rgba(own, 0.16);
    ctx.fill();
    ctx.strokeStyle = rgba(own, 1);
    ctx.lineWidth = 3 * px;
    ctx.setLineDash([]);
    ctx.stroke();
  }
  ctx.restore();
}

/** Gold prices while a tile is being bought (greyed when the treasury cannot pay), and the mark on a queued purchase.
 *  Screen space: `at` gives a tile's centre on screen, `hexPx` the hex radius in px. */
export function drawPriceChips(ctx: CanvasRenderingContext2D, chips: { tile: Hex; price: number; affordable: boolean; queued: boolean }[],
  at: (hx: Hex) => { x: number; y: number } | null, hexPx: number, own: string, S: MapStyle, font: string) {
  ctx.save();
  ctx.font = `700 ${hexPx >= 40 ? 12 : 10.5}px ${font}`;
  ctx.textBaseline = 'middle';
  for (const b of chips) {
    const c = at(b.tile);
    if (!c) continue;
    const text = String(b.price);
    const tw = ctx.measureText(text).width + 19, th = 16;
    const x = c.x - tw / 2, y = c.y + hexPx * 0.42 - th / 2;
    ctx.fillStyle = b.queued ? own : rgba(S.tone.vellum, 0.96);
    ctx.fillRect(x, y, tw, th);
    ctx.strokeStyle = b.affordable ? own : rgba(S.tone.ink, 0.45);
    ctx.lineWidth = 1;
    ctx.strokeRect(x + 0.5, y + 0.5, tw - 1, th - 1);
    ctx.beginPath();
    ctx.arc(x + 8, y + th / 2, 4.2, 0, Math.PI * 2);
    ctx.fillStyle = !b.affordable ? rgba(S.tone.ink, 0.35) : S.tone.gold;
    ctx.fill();
    ctx.strokeStyle = rgba(S.tone.ink, 0.7);
    ctx.stroke();
    ctx.fillStyle = b.queued ? S.tone.vellum : b.affordable ? S.tone.ink : rgba(S.tone.ink, 0.6);
    ctx.textAlign = 'left';
    ctx.fillText(text, x + 15, y + th / 2 + 0.5);
  }
  ctx.restore();
}
