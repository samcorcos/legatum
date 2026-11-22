import type { PlayerView, Wonder } from '../types';
import { WONDER_INFO, wonderArt, wonderArtAlpha, wonderBadgeText } from '../wonders';
import { tintedIcon } from '../icons';
import { FONTS } from './sprites';
import { ATLAS } from './style';
import { centerX, centerY, HEX_SIZE, isFar } from './grid';
import { YIELD_FILL } from './yieldIcons';
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
  if (!image) {
    image = new Image();
    image.onload = image.onerror = () => { for (const cb of listeners) cb(); };
    image.src = wonderArt(wonder, 'b');
    images.set(wonder, image);
  }
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
    const radius = k % 2 === 0 ? 5 : 2.2;
    const px = x + Math.sin(angle) * radius, py = y - Math.cos(angle) * radius;
    if (k === 0) ctx.moveTo(px, py); else ctx.lineTo(px, py);
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
export function drawWonderArt(ctx: CanvasRenderingContext2D, view: PlayerView, cam: Camera, w: number, h: number, dpr: number, lens: string | undefined, yieldsOn = false): void {
  const hexPx = HEX_SIZE * cam.zoom;
  if (isFar(hexPx)) return;
  ctx.save();
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  const size = hexPx * WONDER_ART_SPAN;
  for (let i = 0; i < view.tiles.length; i++) {
    const t = view.tiles[i];
    if (!t.explored || !t.wonder) continue;
    const x = (centerX(i, view.width) - cam.x) * cam.zoom + w / 2, y = (centerY(i, view.width) - cam.y) * cam.zoom + h / 2;
    if (x < -size || x > w + size || y < -size || y > h + size) continue;
    const image = wonderImage(t.wonder);
    if (!image.complete || image.naturalWidth === 0) continue;
    ctx.globalAlpha = wonderArtAlpha(t.visible, lens, yieldsOn);
    ctx.drawImage(image, x - size / 2, y - size * 0.58, size, size);
  }
  ctx.restore();
}

export interface BadgeLayout { font: number; icon: number; width: number; height: number; /** Centre, relative to the hex centre (down is +). */ dy: number }

/** Where and how big the "+25% [yield]" badge is: a pill in the hex's upper half, scaled with the zoom. */
export function wonderBadgeLayout(hexPx: number): BadgeLayout {
  const font = Math.min(12, Math.max(10, hexPx * 0.29));
  const icon = Math.max(11, font * 1.05), text = font * 0.6 * wonderBadgeText().length;
  // Narrow enough to stay inside the hex at its height, so it never lands on a neighbour's yield icons.
  return { font, icon, width: font * 0.6 + text + font * 0.15 + icon, height: font * 1.5, dy: -hexPx * 0.38 };
}

type Cover = { x: number; y: number; w: number; h: number };
const overlaps = (a: Cover, b: Cover) => a.x < b.x + b.w && a.x + a.w > b.x && a.y < b.y + b.h && a.y + a.h > b.y;

/**
 * The bonus on the tile itself: "+25% [Science icon]" in a parchment pill (zoomed out, the mark gains a
 * pip in the yield's colour). It draws above the lens tint and the yield icons, under units and banners,
 * and steps to the bottom slot when a banner covers the top (never with the Yields overlay on, where the tile's own
 * yields sit at the bottom).
 */
export function drawWonderBadges(ctx: CanvasRenderingContext2D, view: PlayerView, cam: Camera, w: number, h: number, dpr: number, yieldsOn: boolean, covers: readonly Cover[]): void {
  const hexPx = HEX_SIZE * cam.zoom;
  const far = isFar(hexPx);
  const L = wonderBadgeLayout(hexPx);
  ctx.save();
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.textAlign = 'left'; ctx.textBaseline = 'middle';
  for (let i = 0; i < view.tiles.length; i++) {
    const t = view.tiles[i];
    if (!t.explored || !t.wonder) continue;
    const x = (centerX(i, view.width) - cam.x) * cam.zoom + w / 2, cy = (centerY(i, view.width) - cam.y) * cam.zoom + h / 2;
    if (x < -90 || x > w + 90 || cy < -90 || cy > h + 90) continue;
    const kind = WONDER_INFO[t.wonder].yield;
    ctx.globalAlpha = t.visible ? 1 : 0.78;
    if (far) {
      drawFarWonder(ctx, x, cy);
      ctx.beginPath(); ctx.arc(x + 7, cy - 7, 3.6, 0, Math.PI * 2);
      ctx.fillStyle = YIELD_FILL[kind]; ctx.fill();
      ctx.lineWidth = 1.2; ctx.strokeStyle = ATLAS.tone.vellum; ctx.stroke();
      continue;
    }
    const slots = yieldsOn ? [L.dy] : [L.dy, -L.dy];
    const y = slots.map((dy) => cy + dy).find((yy) => !covers.some((c) => overlaps(c, { x: x - L.width / 2, y: yy - L.height / 2, w: L.width, h: L.height })));
    if (y === undefined) continue;
    const left = x - L.width / 2, top = y - L.height / 2;
    ctx.beginPath(); ctx.roundRect(left, top, L.width, L.height, L.height / 2);
    ctx.fillStyle = 'rgba(247, 241, 227, 0.95)'; ctx.fill();
    ctx.lineWidth = ATLAS.pen.thin; ctx.strokeStyle = '#564932'; ctx.stroke();
    ctx.font = `700 ${L.font}px ${FONTS.atlas.sans}`;
    ctx.fillStyle = '#2b2419';
    ctx.fillText(wonderBadgeText(), left + L.font * 0.3, y + L.font * 0.04);
    const icon = tintedIcon(`yield-${kind}`, YIELD_FILL[kind], Math.round(L.icon * dpr));
    if (icon) ctx.drawImage(icon, left + L.width - L.icon - L.font * 0.3, y - L.icon / 2, L.icon, L.icon);
  }
  ctx.restore();
}
