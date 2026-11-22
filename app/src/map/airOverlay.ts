// What a city's aircraft cover, drawn on the map while the city is selected (user, 2026-10-17: "show the
// Fighter's 4-tile and the Bomber's 5-tile range as a ring"). Each range is a ring around the city with
// its own line style and a small label, so no ring is an unexplained mark; while an aircraft's strike or
// rebase picker is open its candidates are ringed too, and a click on one picks it.
import type { AirPick, Hex, UnitKind } from '../types';
import { guardRange, isFighter, strikeRange } from '../airText';
import { UNIT_NAME } from '../data';
import { tileIndex, within, hexToPixel } from '../hex';
import { HEX_SIZE, boundary, hexPath, tracePolylines } from './grid';
import { rgba } from '../theme';
import type { Rect } from './compass';

export interface AirRing {
  radius: number;
  label: string;
  /** Strike rings are solid or dotted; a Fighter's guard ring is dashed. */
  line: 'solid' | 'dotted' | 'dashed';
}

export interface AirOverlay {
  center: Hex;
  rings: AirRing[];
  /** The hexes the open picker offers (strike targets, or rebase cities), and which kind of pick it is. */
  candidates: Hex[];
  pickKind: AirPick | null;
  hover: Hex | null;
  /** The name of the hovered rebase city, so an off-screen destination can be pointed at from the screen edge. */
  hoverLabel?: string | null;
}

/** The rings for the kinds of aircraft based in the city, widest first. */
export function airRings(kinds: UnitKind[]): AirRing[] {
  const rings: AirRing[] = [];
  for (const kind of ['Bomber', 'JetBomber', 'StealthBomber', 'Fighter', 'JetFighter', 'StealthFighter', 'Biplane'] as const) {
    if (!kinds.includes(kind)) continue;
    const name = UNIT_NAME[kind];
    if (isFighter(kind)) rings.push({ radius: guardRange(kind), label: `${name} guards ${guardRange(kind)} tiles`, line: 'dashed' });
    rings.push({ radius: strikeRange(kind), label: `${name} strikes ${strikeRange(kind)} tiles`, line: isFighter(kind) ? 'dotted' : 'solid' });
  }
  return rings.sort((a, b) => b.radius - a.radius);
}

/** The ring lines (world coordinates, under units): each range's outer edge on the map. */
/** The guard ring is drawn in a neutral ink, so defence and offence never share a colour. */
const GUARD_INK = '#5b503f';

export function drawAirRings(ctx: CanvasRenderingContext2D, ov: AirOverlay, W: number, H: number, color: string, px: number) {
  ctx.save();
  ctx.lineCap = 'round';
  ctx.lineJoin = 'round';
  for (const ring of ov.rings) {
    const tiles = within(ov.center, ring.radius).map((h) => tileIndex(h, W, H)).filter((i) => i >= 0);
    const inside = new Set(tiles);
    const lines = boundary(tiles, (i) => inside.has(i), W, H);
    const dash = ring.line === 'dashed' ? [7 * px, 5 * px] : ring.line === 'dotted' ? [1.5 * px, 4 * px] : [];
    const passes: [number, string, number[]][] = [[5, 'rgba(255,250,240,0.55)', []], [2.6, rgba(ring.line === 'dashed' ? GUARD_INK : color, 0.95), dash]];
    for (const [width, stroke, d] of passes) {
      ctx.beginPath();
      tracePolylines(ctx, lines);
      ctx.setLineDash(d);
      ctx.lineWidth = width * px;
      ctx.strokeStyle = stroke;
      ctx.stroke();
    }
  }
  ctx.restore();
}

/** Try the old preferred bearings first, then walk around the ring for a free slot. */
const LABEL_ANGLES = [-60, -120, 0, 180, 60, 120, -30, -90, -150, 150, 90, 30];

const overlaps = (x: number, y: number, w: number, h: number, b: Rect) => x + w / 2 > b.x && x - w / 2 < b.x + b.w && y + h / 2 > b.y && y - h / 2 < b.y + b.h;

/** The nearest spot to `raw` inside the canvas and clear of every panel over the map (a label under the city panel was cut off in the 8.9.4 review). */
function clearSpot(raw: { x: number; y: number }, w: number, h: number, bounds: { w: number; h: number }, avoid: readonly Rect[]): { x: number; y: number } | null {
  const clampX = (x: number) => Math.max(w / 2 + 4, Math.min(bounds.w - w / 2 - 4, x));
  const clampY = (y: number) => Math.max(14, Math.min(bounds.h - 14, y));
  const xs = [raw.x, ...avoid.flatMap((b) => [b.x - w / 2 - 4, b.x + b.w + w / 2 + 4])].map(clampX);
  const ys = [raw.y, ...avoid.flatMap((b) => [b.y - h / 2 - 4, b.y + b.h + h / 2 + 4])].map(clampY);
  const first = { x: clampX(raw.x), y: clampY(raw.y) };
  if (!avoid.some((b) => overlaps(first.x, first.y, w, h, b))) return first;
  let best = first, distance = Infinity;
  for (const x of xs) for (const y of ys) {
    if (avoid.some((b) => overlaps(x, y, w, h, b))) continue;
    const d = (x - raw.x) ** 2 + (y - raw.y) ** 2;
    if (d < distance) { best = { x, y }; distance = d; }
  }
  return distance < Infinity ? best : null;
}

/** Screen-space label plates, in ring order. Occupied plates become obstacles for later rings. */
export function airLabelLayout(ov: AirOverlay, toScreen: (x: number, y: number) => { x: number; y: number }, measure: (label: string) => number, bounds: { w: number; h: number }, avoid: readonly Rect[] = []) {
  const center = hexToPixel(ov.center, HEX_SIZE);
  // The base city's own tile (its painting and the aircraft token on it) is never under a label.
  const a = toScreen(center.x - HEX_SIZE, center.y - HEX_SIZE), b = toScreen(center.x + HEX_SIZE, center.y + HEX_SIZE);
  const occupied = [...avoid, { x: Math.min(a.x, b.x), y: Math.min(a.y, b.y), w: Math.abs(b.x - a.x), h: Math.abs(b.y - a.y) }];
  const placed: { ring: AirRing; x: number; y: number; w: number }[] = [];
  for (const [i, ring] of ov.rings.entries()) {
    const w = Math.ceil(measure(ring.label)) + 12;
    const d = (ring.radius + 0.5) * Math.sqrt(3) * HEX_SIZE;
    // A ring can be mostly off screen. First seek a bearing on screen, then use
    // the nearest clear edge slot; never draw a label over a banner or a peer.
    let chosen: { x: number; y: number } | null = null;
    for (let step = 0; step < LABEL_ANGLES.length; step++) {
      const a = LABEL_ANGLES[(i + step) % LABEL_ANGLES.length] * Math.PI / 180;
      const raw = toScreen(center.x + Math.cos(a) * d, center.y + Math.sin(a) * d);
      if (raw.x < w / 2 + 4 || raw.x > bounds.w - w / 2 - 4 || raw.y < 14 || raw.y > bounds.h - 14) continue;
      if (!occupied.some((b) => overlaps(raw.x, raw.y, w, 18, b))) { chosen = raw; break; }
    }
    if (!chosen) {
      const a = LABEL_ANGLES[i % LABEL_ANGLES.length] * Math.PI / 180;
      chosen = clearSpot(toScreen(center.x + Math.cos(a) * d, center.y + Math.sin(a) * d), w, 18, bounds, occupied);
    }
    if (!chosen) continue;
    placed.push({ ring, ...chosen, w });
    occupied.push({ x: chosen.x - w / 2 - 3, y: chosen.y - 12, w: w + 6, h: 24 });
  }
  return placed;
}

/** The ring labels, in screen space: a small vellum plate with ink text and the ring colour's rim. */
export function drawAirLabels(ctx: CanvasRenderingContext2D, ov: AirOverlay, toScreen: (x: number, y: number) => { x: number; y: number }, color: string, font: string, bounds: { w: number; h: number }, avoid: readonly Rect[] = []) {
  ctx.save();
  ctx.font = `700 11px ${font}`;
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  for (const { ring, x, y, w } of airLabelLayout(ov, toScreen, (label) => ctx.measureText(label).width, bounds, avoid)) {
    ctx.fillStyle = 'rgba(247,241,227,0.94)';
    ctx.strokeStyle = rgba(ring.line === 'dashed' ? GUARD_INK : color, 0.95);
    ctx.lineWidth = 1.5;
    ctx.beginPath();
    ctx.rect(Math.round(x - w / 2) + 0.5, Math.round(y - 9) + 0.5, w, 18);
    ctx.fill();
    ctx.stroke();
    ctx.fillStyle = '#2b2419';
    ctx.fillText(ring.label, x, y + 0.5);
  }
  ctx.restore();
}

/** The picker's candidates (world coordinates): each is ringed; the hovered or focused one is heavy and washed. */
export function drawAirCandidates(ctx: CanvasRenderingContext2D, ov: AirOverlay, color: string, px: number) {
  if (!ov.pickKind) return;
  drawTileCandidates(ctx, ov.candidates, ov.hover, color, px);
}

/**
 * A hovered rebase destination that is off the screen: a plate with its name and an arrow on the screen edge,
 * on the line from the screen centre toward the city (8.9.25 UI review: a far rebase target gave no map feedback).
 */
export function drawAirOffscreen(ctx: CanvasRenderingContext2D, ov: AirOverlay, toScreen: (x: number, y: number) => { x: number; y: number }, color: string, font: string, bounds: { w: number; h: number }, avoid: readonly Rect[] = []) {
  if (ov.pickKind !== 'rebase' || !ov.hover || !ov.hoverLabel) return;
  const world = hexToPixel(ov.hover, HEX_SIZE);
  const at = toScreen(world.x, world.y);
  const margin = 6;
  if (at.x >= margin && at.x <= bounds.w - margin && at.y >= margin && at.y <= bounds.h - margin) return;
  const cx = bounds.w / 2;
  const cy = bounds.h / 2;
  const dx = at.x - cx;
  const dy = at.y - cy;
  ctx.save();
  ctx.font = `700 12px ${font}`;
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  const label = ov.hoverLabel;
  const w = Math.ceil(ctx.measureText(label).width) + 34;
  const h = 24;
  const halfW = bounds.w / 2 - w / 2 - 8;
  const halfH = bounds.h / 2 - h / 2 - 8;
  const k = Math.min(halfW / Math.max(Math.abs(dx), 1e-6), halfH / Math.max(Math.abs(dy), 1e-6));
  let x = cx + dx * k;
  const y = cy + dy * k;
  // A panel over the screen edge would hide the plate: slide it out to the open map.
  for (const r of avoid) {
    if (x + w / 2 > r.x && x - w / 2 < r.x + r.w && y + h / 2 > r.y && y - h / 2 < r.y + r.h) x = x < cx ? r.x + r.w + w / 2 + 6 : r.x - w / 2 - 6;
  }
  const angle = Math.atan2(dy, dx);
  ctx.fillStyle = 'rgba(247,241,227,0.96)';
  ctx.strokeStyle = rgba(color, 1);
  ctx.lineWidth = 2.5;
  ctx.beginPath();
  ctx.rect(Math.round(x - w / 2) + 0.5, Math.round(y - h / 2) + 0.5, w, h);
  ctx.fill();
  ctx.stroke();
  ctx.fillStyle = '#2b2419';
  ctx.fillText(label, x - 8, y + 0.5);
  ctx.translate(x + w / 2 - 12, y);
  ctx.rotate(angle);
  ctx.fillStyle = rgba(color, 1);
  ctx.beginPath();
  ctx.moveTo(7, 0);
  ctx.lineTo(-4, -6);
  ctx.lineTo(-4, 6);
  ctx.closePath();
  ctx.fill();
  ctx.restore();
}

/**
 * A map picker's candidate tiles: dashed rings, the hovered one heavy and washed. `wash` tints every
 * candidate too, and `ring` thickens its outline, for a placement where the lit tiles are the whole choice.
 */
export function drawTileCandidates(ctx: CanvasRenderingContext2D, candidates: Hex[], hover: Hex | null, color: string, px: number, wash = 0, ring = 2.4) {
  ctx.save();
  for (const hx of candidates) {
    const p = hexToPixel(hx, HEX_SIZE);
    const hot = !!hover && hover.q === hx.q && hover.r === hx.r;
    ctx.beginPath();
    hexPath(ctx, p.x, p.y, HEX_SIZE * 0.88);
    if (hot || wash) { ctx.fillStyle = rgba(color, hot ? Math.max(0.16, wash * 2) : wash); ctx.fill(); }
    ctx.setLineDash(hot ? [] : [3 * px, 3 * px]);
    ctx.lineWidth = (hot ? 4.5 : ring) * px;
    ctx.strokeStyle = rgba(color, hot ? 1 : 0.85);
    ctx.stroke();
  }
  ctx.restore();
}
