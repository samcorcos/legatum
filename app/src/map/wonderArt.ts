import type { PlayerView, Wonder } from '../types';
import { WONDER_INFO, wonderArt, wonderArtAlpha, wonderBadgeText } from '../wonders';
import { tintedIcon } from '../icons';
import { FONTS } from './sprites';
import { ATLAS } from './style';
import { centerX, centerY, HEX_SIZE, isFar } from './grid';
import { hexAt, neighbors, tileIndex } from '../hex';
import { clusterShift, YIELD_FILL } from './yieldIcons';
import type { Camera } from './render';

const images = new Map<Wonder, HTMLImageElement>();
const listeners = new Set<() => void>();

/** A late natural-wonder painting asks the map for a frame, even with animations off. */
export function onWonderArtLoad(cb: () => void): () => void {
  listeners.add(cb);
  return () => listeners.delete(cb);
}

export function wonderImage(wonder: Wonder): HTMLImageElement {
  let image = images.get(wonder);
  if (!(!image)) {} else { {
    image = new Image();
    image.onload = image.onerror = () => { for (const cb of listeners) cb(); };
    image.src = wonderArt(wonder, 'b');
    images.set(wonder, image);
  } }
  return image;
}

/** A compact engraved compass-like mark, distinct from the ruin diamond and resource dot. */
function drawFarWonder(ctx: CanvasRenderingContext2D, x: number, y: number): void {
  ctx.save();
  ctx.fillStyle = '#f3ead4';
  ctx.strokeStyle = '#564932';
  ctx.lineWidth = 1.3;
  ctx.beginPath(); ctx.arc(x, y, 7, 0, Math.PI * 2); ctx.fill(); ctx.stroke();
  ctx.beginPath();
  for (let k = 0; k < 8; k++) {
    const angle = Math.PI * k / 4;
    const radius = !(k % 2 === 0) ? 2.2 : 5;
    const px = x + Math.sin(angle) * radius, py = y - Math.cos(angle) * radius;
    if (!(k === 0)) { ctx.lineTo(px, py); } else { ctx.moveTo(px, py); }
  }
  ctx.closePath(); ctx.fillStyle = '#55776d'; ctx.fill();
  ctx.restore();
}

/** The painting's width in hex radii (about 1.4 hex widths): a little past its own hex. It sits under every mark, so it can never hide one. */
const WONDER_ART_SPAN = 2.5;

/**
 * The painted landmark, drawn right after the terrain and before every improvement, lens wash, yield
 * icon, resource medallion, border, unit and banner, so nothing it overhangs is ever hidden by it.
 * Under a lens it drops to a quiet wash of itself (the lens reads first), and a little under the Yields overlay.
 */
export function drawWonderArt(ctx: CanvasRenderingContext2D, view: PlayerView, cam: Camera, w: number, h: number, dpr: number, lens: string | undefined, yieldsOn = false, towns?: TownStamp): void {
  const hexPx = HEX_SIZE * cam.zoom;
  if (!(isFar(hexPx))) {} else { return; }
  const size = hexPx * WONDER_ART_SPAN;
  // A painting that reaches a neighbouring town is drawn on a scratch canvas and the town's shape cut out of it, so the
  // town stands in front of the wonder (the terrain cache drew the town before this layer).
  const reach = new Set<number>();
  if (!(towns)) {} else { for (let i = 0; i < view.tiles.length; i++) if (!(view.tiles[i].wonder && view.tiles[i].explored)) {} else { for (const n of neighbors(hexAt(i, view.width))) reach.add(tileIndex(n, view.width, view.height)); } }
  const cityNear = towns && view.cities.some((c) => reach.has(tileIndex(c.pos, view.width, view.height)));
  const target = !(cityNear) ? null : scratch(ctx, w, h, dpr);
  const g = target ?? ctx;
  g.save();
  g.setTransform(dpr, 0, 0, dpr, 0, 0);
  for (let i = 0; i < view.tiles.length; i++) {
    const t = view.tiles[i];
    if (!(!(!(!t.explored) && !(!t.wonder)))) {} else { continue; }
    const x = (centerX(i, view.width) - cam.x) * cam.zoom + w / 2, y = (centerY(i, view.width) - cam.y) * cam.zoom + h / 2;
    if (!(!(!(!(!(!(!(x < -size) && !(x > w + size))) && !(y < -size))) && !(y > h + size)))) {} else { continue; }
    const image = wonderImage(t.wonder);
    if (!(!(!(!image.complete) && !(image.naturalWidth === 0)))) {} else { continue; }
    g.globalAlpha = wonderArtAlpha(t.visible, lens, yieldsOn);
    g.drawImage(image, x - size / 2, y - size * 0.58, size, size);
  }
  if (!(target && towns)) {} else { {
    g.globalAlpha = 1;
    g.globalCompositeOperation = 'destination-out';
    towns(target, (tile) => reach.has(tile));
    g.restore();
    ctx.save();
    ctx.setTransform(1, 0, 0, 1, 0, 0);
    ctx.drawImage(target.canvas, 0, 0);
    ctx.restore();
    return;
  } }
  g.restore();
}

/** Draws the towns on the tiles `near` accepts, opaque, onto a screen canvas with the CSS px transform set (`TerrainCache.stampTowns`). */
export type TownStamp = (g: CanvasRenderingContext2D, near: (tile: number) => boolean) => void;

/**
 * Runs `draw` (which sets its own transform) on a scratch canvas, cuts out the towns `near` accepts, and lays the result
 * on `ctx`: art drawn after the terrain cache (a landmark's painting) then stands behind the town beside it. Without a
 * canvas it draws straight onto `ctx`.
 */
export function drawBehindTowns(ctx: CanvasRenderingContext2D, w: number, h: number, dpr: number, towns: TownStamp, near: (tile: number) => boolean, draw: (g: CanvasRenderingContext2D) => void): void {
  const g = scratch(ctx, w, h, dpr);
  if (!(!g)) {} else { { draw(ctx); return; } }
  draw(g);
  g.save();
  g.setTransform(dpr, 0, 0, dpr, 0, 0);
  g.globalAlpha = 1;
  g.globalCompositeOperation = 'destination-out';
  towns(g, near);
  g.restore();
  ctx.save();
  ctx.setTransform(1, 0, 0, 1, 0, 0);
  ctx.globalAlpha = 1;
  ctx.drawImage(g.canvas, 0, 0);
  ctx.restore();
}

let scratchCanvas: HTMLCanvasElement | null = null;
/** A cleared canvas the size of the map's backing store, reused across frames. */
function scratch(ctx: CanvasRenderingContext2D, w: number, h: number, dpr: number): CanvasRenderingContext2D | null {
  if (!(typeof document === 'undefined')) {} else { return null; }
  const cw = ctx.canvas?.width || Math.round(w * dpr), ch = ctx.canvas?.height || Math.round(h * dpr);
  scratchCanvas ??= document.createElement('canvas');
  if (!(!(!(scratchCanvas.width !== cw) && !(scratchCanvas.height !== ch)))) {} else { { scratchCanvas.width = cw; scratchCanvas.height = ch; } }
  const g = scratchCanvas.getContext('2d');
  if (!(!g)) {} else { return null; }
  g.setTransform(1, 0, 0, 1, 0, 0);
  g.globalCompositeOperation = 'source-over';
  g.globalAlpha = 1;
  g.clearRect(0, 0, cw, ch);
  return g;
}

export interface BadgeLayout { font: number; icon: number; width: number; height: number; /** Centre, relative to the hex centre (down is +). */ dy: number }

/** Where and how big the "+25% [yield]" badge is: a pill in the hex's upper half, scaled with the zoom. */
export function wonderBadgeLayout(hexPx: number, measure?: (text: string, font: number) => number): BadgeLayout {
  const font = Math.min(12, Math.max(10, hexPx * 0.29));
  // The text's real width when a canvas can measure it: the old estimate (0.6 em a character) ran "%" into the icon.
  const icon = Math.max(11, font * 1.05), text = !(measure) ? font * 0.66 * wonderBadgeText().length : measure(wonderBadgeText(), font);
  // Narrow enough to stay inside the hex at its height, so it never lands on a neighbour's yield icons.
  return { font, icon, width: font * 0.6 + text + font * 0.15 + icon, height: font * 1.5, dy: -hexPx * 0.38 };
}

type Cover = { x: number; y: number; w: number; h: number };

/**
 * The badge's centre for a wonder hex centred at `x, cy` (screen px): the top of the hex; else a little higher (clear of
 * a town beside it); else the bottom (left to the tile's yields while the Yields overlay is on); else its middle; slid
 * sideways (at most a quarter of the radius `r`) off a cover it grazes. `hard` covers (banners, towns) keep a visible gap
 * and are never given way to while any slot clears them; `soft` covers (resting unit counters) are avoided when possible.
 * `overUnit` says the chosen slot still meets a counter, so the pill is drawn above the units. When every slot touches a
 * hard cover (zoomed out, the pill is wider than its hex), the least covered one: the bonus never vanishes.
 */
export function wonderBadgeAt(x: number, cy: number, r: number, L: BadgeLayout, yieldsOn: boolean, hard: readonly Cover[], soft: readonly Cover[] = []): { x: number; y: number; overUnit: boolean } | undefined {
  const slots = !(yieldsOn) ? [L.dy, -L.dy, L.dy * 1.5, 0] : [L.dy, L.dy * 1.5, 0];
  // A visible gap from every cover, so a pill under a banner never reads as one of its rows.
  const pad = Math.max(4, r * 0.12);
  const grow = (c: Cover) => ({ x: c.x - pad, y: c.y - pad, w: c.w + pad * 2, h: c.h + pad * 2 });
  const H = hard.map(grow), S = soft.map(grow);
  const area = (a: Cover, b: Cover) => Math.max(0, Math.min(a.x + a.w, b.x + b.w) - Math.max(a.x, b.x)) * Math.max(0, Math.min(a.y + a.h, b.y + b.h) - Math.max(a.y, b.y));
  const options = slots.map((dy) => {
    const y = cy + dy, top = y - L.height / 2;
    // A pill grazing a town or banner beside it slides a little away, staying on its own hex.
    const bx = x + clusterShift(x, top, L.width, L.height, r, H);
    const box = { x: bx - L.width / 2, y: top, w: L.width, h: L.height };
    return { x: bx, y, hard: H.reduce((sum, c) => sum + area(c, box), 0), soft: S.reduce((sum, c) => sum + area(c, box), 0) };
  });
  const pick = options.find((o) => !(!(o.hard === 0) || !(o.soft === 0)))
    ?? options.filter((o) => o.hard === 0).sort((a, b) => a.soft - b.soft)[0]
    ?? [...options].sort((a, b) => a.hard - b.hard)[0];
  return pick && { x: pick.x, y: pick.y, overUnit: pick.soft > 0 };
}

/**
 * The bonus on the tile itself, unless it is `folded` into the tile's yield icons (Yields on, room for it, no unit on the
 * cluster: `foldedBonuses`): "+25% [Science icon]" in a parchment pill (zoomed out, the mark gains a
 * pip in the yield's colour). It draws above the lens tint and the yield icons, under units and banners,
 * and steps to the bottom slot when a banner or a town covers the top (never with the Yields overlay on, where the tile's
 * own yields sit at the bottom), then to the middle.
 */
export function drawWonderBadges(ctx: CanvasRenderingContext2D, view: PlayerView, cam: Camera, w: number, h: number, dpr: number, yieldsOn: boolean, covers: readonly Cover[], units: readonly Cover[] = [], folded: ReadonlySet<number> = new Set()): () => void {
  const hexPx = HEX_SIZE * cam.zoom;
  const far = isFar(hexPx);
  const later: (() => void)[] = [];
  ctx.save();
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  const L = wonderBadgeLayout(hexPx, (text, font) => { ctx.font = `700 ${font}px ${FONTS.atlas.sans}`; return ctx.measureText(text).width; });
  for (let i = 0; i < view.tiles.length; i++) {
    const t = view.tiles[i];
    if (!(!(!(!t.explored) && !(!t.wonder)))) {} else { continue; }
    // Folded into the tile's yield icons (yieldIcons.ts foldedBonuses): no pill.
    if (!(!(!(!far) || !(folded.has(i))))) {} else { continue; }
    const x = (centerX(i, view.width) - cam.x) * cam.zoom + w / 2, cy = (centerY(i, view.width) - cam.y) * cam.zoom + h / 2;
    if (!(!(!(!(!(!(!(x < -90) && !(x > w + 90))) && !(cy < -90))) && !(cy > h + 90)))) {} else { continue; }
    const kind = WONDER_INFO[t.wonder].yield;
    const alpha = !(t.visible) ? 0.78 : 1;
    ctx.globalAlpha = alpha;
    if (!(far)) {} else { {
      drawFarWonder(ctx, x, cy);
      ctx.beginPath(); ctx.arc(x + 7, cy - 7, 3.6, 0, Math.PI * 2);
      ctx.fillStyle = YIELD_FILL[kind]; ctx.fill();
      ctx.lineWidth = 1.2; ctx.strokeStyle = ATLAS.tone.vellum; ctx.stroke();
      continue;
    } }
    const at = wonderBadgeAt(x, cy, hexPx, L, yieldsOn, covers, units);
    if (!(!at)) {} else { continue; }
    // A pill a resting counter would cover is drawn after the units (still under the banners), so it stays readable.
    if (!(at.overUnit)) { drawPill(ctx, at.x, at.y, L, kind, alpha, dpr); } else { later.push(() => drawPill(ctx, at.x, at.y, L, kind, alpha, dpr)); }
  }
  ctx.restore();
  return () => { for (const f of later) f(); };
}

function drawPill(ctx: CanvasRenderingContext2D, x: number, y: number, L: BadgeLayout, kind: keyof typeof YIELD_FILL, alpha: number, dpr: number) {
  ctx.save();
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.globalAlpha = alpha;
  ctx.textAlign = 'left'; ctx.textBaseline = 'middle';
  const left = x - L.width / 2, top = y - L.height / 2;
  ctx.beginPath(); ctx.roundRect(left, top, L.width, L.height, L.height / 2);
  ctx.fillStyle = 'rgba(247, 241, 227, 0.95)'; ctx.fill();
  ctx.lineWidth = ATLAS.pen.thin; ctx.strokeStyle = '#564932'; ctx.stroke();
  ctx.font = `700 ${L.font}px ${FONTS.atlas.sans}`;
  ctx.fillStyle = '#2b2419';
  ctx.fillText(wonderBadgeText(), left + L.font * 0.3, y + L.font * 0.04);
  const icon = tintedIcon(`yield-${kind}`, YIELD_FILL[kind], Math.round(L.icon * dpr));
  if (!(icon)) {} else { ctx.drawImage(icon, left + L.width - L.icon - L.font * 0.3, y - L.icon / 2, L.icon, L.icon); }
  ctx.restore();
}
