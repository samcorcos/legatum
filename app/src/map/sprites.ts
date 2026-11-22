// Screen-space map pieces drawn per frame, pre-rendered to small cached canvases: unit tokens,
// city banners and map lettering. Atlas uses square wargame counters and vellum cartouches.
import type { Faction, UnitKind } from '../types';
import type { CapitalStar } from '../capitalMark';
import type { LookId } from '../brand/look';
import { iconEpoch, tintedIcon } from '../icons';
import { unitIconName } from '../data';
import { barbarianMiniatureImage, miniatureImage, roundelImage } from '../unitArt';
import { healthColor, playerInk, rgba } from '../theme';
import { ATLAS, mapPlayerColor, mapPlayerTint } from './style';
import type { Rect } from './compass';
import { RANK_EDGE, RANK_GOLD, RANK_INK, RANK_LAMBDA, RANK_PATCH, rankLevel, rankStrokes, type RankLevel } from '../rankInsignia';

export const FONTS: Record<LookId, { display: string; sans: string }> = {
  atlas: { display: '"Cormorant", "Cormorant Garamond", Garamond, serif', sans: '"Libre Franklin", "Franklin Gothic", sans-serif' },
};

const INK = { atlas: ATLAS.tone };

// ---- font / icon epochs: cached sprites are rebuilt when fonts or icons finish loading ----
let fontLoads = 0;
if (typeof document !== 'undefined' && document.fonts) {
  document.fonts.addEventListener?.('loadingdone', () => fontLoads++);
  document.fonts.ready?.then(() => fontLoads++);
}
export const fontEpoch = () => fontLoads;

type Sprite = { c: HTMLCanvasElement; ax: number; ay: number; w: number; h: number };
const cache = new Map<string, Sprite>();
let epoch = '';
function cached(key: string, make: () => Sprite): Sprite {
  const e = `${fontLoads}|${iconEpoch()}`;
  if (e !== epoch || cache.size > 600) {
    cache.clear();
    epoch = e;
  }
  let s = cache.get(key);
  if (!s) {
    s = make();
    cache.set(key, s);
  }
  return s;
}

function canvas(w: number, h: number, dpr: number): [HTMLCanvasElement, CanvasRenderingContext2D] {
  const c = document.createElement('canvas');
  c.width = Math.max(1, Math.ceil(w * dpr));
  c.height = Math.max(1, Math.ceil(h * dpr));
  const g = c.getContext('2d')!;
  g.scale(dpr, dpr);
  return [c, g];
}

/** Draw text with manual tracking (canvas letterSpacing is not everywhere). Returns the width. */
function spaced(g: CanvasRenderingContext2D, text: string, x: number, y: number, track: number, draw = true): number {
  let w = 0;
  for (const ch of text) w += g.measureText(ch).width + track;
  w -= track;
  if (!draw) return w;
  let cx = x;
  for (const ch of text) {
    g.fillText(ch, cx, y);
    cx += g.measureText(ch).width + track;
  }
  return w;
}

/** Small caps by hand: capitals at full size, lowercase as capitals at 78%. */
function smallCaps(g: CanvasRenderingContext2D, text: string, x: number, y: number, font: (px: number) => string, px: number, track: number, draw = true, halo?: string): number {
  let cx = x;
  for (const ch of text) {
    const lower = ch !== ch.toUpperCase();
    g.font = font(lower ? px * 0.78 : px);
    const c = ch.toUpperCase();
    if (draw && halo) { g.save(); g.lineJoin = 'round'; g.lineWidth = 3.2; g.strokeStyle = halo; g.strokeText(c, cx, y); g.restore(); }
    if (draw) g.fillText(c, cx, y);
    cx += g.measureText(c).width + track;
  }
  g.font = font(px);
  return cx - x - track;
}

// ---- unit tokens ----

/**
 * The Byzantium's Heir: a gilt inner rim on his chit says "this one scores"; when a rival holds him,
 * Byzantium's own crest sits on his corner in Byzantium's seat colour, so everyone can tell
 * whose Heir it is (the rim and the numeral say who holds him).
 */
export interface HeirMark { crestColor?: string }

/** The gilt of the Heir's rim and the pip's ring: the chevron gold, drawn over an ink hairline so it holds on parchment. */
const heirGilt = () => ATLAS.tone.chevron;

/** A unit token centered at (x, y); `w` is the half-size in CSS px. */
export function drawToken(
  ctx: CanvasRenderingContext2D,
  look: LookId,
  kind: UnitKind,
  color: string,
  military: boolean,
  x: number,
  y: number,
  w: number,
  dpr: number,
  owner: number,
  alpha = 1,
  faction?: Faction,
  heir?: HeirMark,
) {
  const q = Math.round(w * 2) / 2;
  const icon = unitIconName(kind, faction);
  // Normal zoom: the painted miniature on a base washed in the owner's colour; the engraved glyph counter until its image loads.
  const art = miniatureImage(kind, faction);
  const sp = art
    ? cached(`mini|${look}|${kind}|${faction ?? ''}|${color}|${q}|${dpr}|${heir ? `heir${heir.crestColor ?? ''}` : ''}`, () => miniatureToken(art, color, q, dpr, heir))
    : cached(`tok|${look}|${icon}|${color}|${military}|${q}|${dpr}|${heir ? `heir${heir.crestColor ?? ''}` : ''}`, () =>
      atlasToken(icon, color, military, q, dpr, heir),
    );
  ctx.globalAlpha = alpha;
  ctx.drawImage(sp.c, x - sp.ax, y - sp.ay, sp.w, sp.h);
  drawPlayerNumber(ctx, look, owner, x - w * 0.9, y - w * 0.85, faction);
  ctx.globalAlpha = 1;
}

/**
 * How a painted miniature stands out from the terrain art (user, 2026-10-11: on the richer organic terrain "the miniatures for
 * units are now pretty hard to see"). Chosen from shots of four looks on forest, range, hills, desert and grass at both window
 * sizes (docs/design/terrain-art/round3/units-*): the figure is 20% larger, stands in a soft vellum pool (`halo`, radius `haloR`
 * of the figure's side) that lifts it off dark woods and brown peaks alike, and its stone base wears the owner's colour as a
 * firm ring of `ring` px between two ink rules, so seats stay apart on any ground. The pool is vellum, not a dark shadow or an
 * outline, so the atlas look holds.
 */
const MINI = { scale: 1.2, halo: 0.78, haloR: 0.72, ring: 2.2 } as const;
/** The figure's side for token half-width `w` (the owner's base and halo are drawn inside this square's padding). */
const miniSide = (w: number) => w * 2.8 * MINI.scale;

/** Where the parts of a miniature token sit, in CSS px from the unit's anchor (its hex centre) for token half-width `w`. */
export function miniatureGeometry(w: number) {
  const S = miniSide(w);
  return { S, baseDy: S * 0.4, baseRx: S * 0.27, baseRy: S * 0.09, bodyDy: S * 0.1, bodyRx: S * 0.22, bodyRy: S * 0.3, flankDx: S * 0.36, flankR: S * 0.08, halo: S * MINI.haloR };
}

/**
 * The painted miniature as a map token (`assets/units`): the figure stands on a stone base that wears the owner's colour twice,
 * a wash over the stone and a ring beneath it, so ownership reads at a glance without hiding the painting. The Heir adds his gilt
 * rim, and a rival's hold shows Byzantium's crest, as on the engraved counter. `stone` washes the base in another colour than the
 * ring and `notch` marks the ring's left end (barbarians: charred ink stone in an ink ring with a blood-red notch).
 */
function miniatureToken(art: HTMLImageElement, color: string, w: number, dpr: number, heir?: HeirMark, stone = color, notch?: string): Sprite {
  const S = miniSide(w);
  const pad = Math.max(4, Math.ceil(S * (MINI.haloR - 0.5)) + 3);
  const size = S + 2 * pad;
  const [c, g] = canvas(size, size, dpr);
  const cx = pad + S / 2, top = pad;
  // A soft vellum pool behind the figure: the painted miniature reads on forest, range and hill alike.
  {
    const hr = S * MINI.haloR;
    g.save();
    g.translate(cx, top + S * 0.56);
    g.scale(1, 0.92);
    const grad = g.createRadialGradient(0, 0, 0, 0, 0, hr);
    grad.addColorStop(0, rgba(ATLAS.seat.plate, MINI.halo));
    grad.addColorStop(0.6, rgba(ATLAS.seat.plate, MINI.halo * 0.62));
    grad.addColorStop(1, rgba(ATLAS.seat.plate, 0));
    g.fillStyle = grad;
    g.beginPath(); g.arc(0, 0, hr, 0, Math.PI * 2); g.fill();
    g.restore();
  }
  // The figure's centre sits a little above the hex centre so the base falls under the counter's footing.
  const baseY = top + S * 0.9, baseRx = S * 0.27, baseRy = S * 0.09;
  const ring = MINI.ring;
  const ellipse = (rx: number, ry: number, fill: string) => { g.beginPath(); g.ellipse(cx, baseY, rx, ry, 0, 0, Math.PI * 2); g.fillStyle = fill; g.fill(); };
  g.beginPath(); g.ellipse(cx + 1, baseY + 1.6, baseRx + 3 + ring, baseRy + 2.4 + ring * 0.4, 0, 0, Math.PI * 2); g.fillStyle = ATLAS.tone.shadow; g.fill();
  // The owner's colour as a firm ring, inked outside and in.
  ellipse(baseRx + 2.4 + ring + 1, baseRy + 1.8 + ring * 0.4 + 0.8, INK.atlas.ink);
  ellipse(baseRx + 2.4 + ring, baseRy + 1.8 + ring * 0.4, color);
  ellipse(baseRx + 2.4, baseRy + 1.8, rgba(stone, 0.82));
  g.strokeStyle = INK.atlas.ink; g.lineWidth = ATLAS.pen.thin; g.stroke();
  if (notch) {
    // The torn mark on the ring's left end (barbarians: the same red notch as their far-zoom roundel and counter).
    g.beginPath(); g.ellipse(cx, baseY, baseRx + 2.4 + ring / 2, baseRy + 1.8 + ring * 0.2, 0, Math.PI * 0.8, Math.PI * 1.12);
    g.strokeStyle = notch; g.lineWidth = ring + 0.6; g.stroke();
  }
  if (heir) {
    g.beginPath(); g.ellipse(cx, baseY, baseRx + 5 + ring, baseRy + 4 + ring * 0.4, 0, 0, Math.PI * 2); g.strokeStyle = INK.atlas.ink; g.lineWidth = 3.4; g.stroke();
    g.beginPath(); g.ellipse(cx, baseY, baseRx + 5 + ring, baseRy + 4 + ring * 0.4, 0, 0, Math.PI * 2); g.strokeStyle = heirGilt(); g.lineWidth = 1.8; g.stroke();
  }
  // The painting itself, its base washed in the owner's colour (multiplied so the highlights stay).
  const [p, pg] = canvas(S, S, dpr);
  pg.drawImage(art, 0, 0, S, S);
  pg.globalCompositeOperation = 'source-atop';
  pg.beginPath(); pg.ellipse(S / 2, S * 0.9 - 0 + 0, baseRx, baseRy * 1.15, 0, 0, Math.PI * 2);
  pg.fillStyle = rgba(stone, 0.52); pg.fill();
  g.drawImage(p, cx - S / 2, top, S, S);
  if (heir?.crestColor) {
    const r = Math.max(7, w * 0.5), px = cx + S * 0.34, py = top + S * 0.86;
    g.beginPath(); g.arc(px, py, r, 0, Math.PI * 2); g.fillStyle = ATLAS.seat.plate; g.fill();
    g.beginPath(); g.arc(px, py, r - 1.1, 0, Math.PI * 2); g.strokeStyle = heir.crestColor; g.lineWidth = 2.2; g.stroke();
    g.beginPath(); g.arc(px, py, r, 0, Math.PI * 2); g.strokeStyle = INK.atlas.ink; g.lineWidth = ATLAS.pen.thin; g.stroke();
    const gw = r * 1.5;
    const gl = tintedIcon('faction-byzantium', INK.atlas.ink, gw * dpr);
    if (gl) g.drawImage(gl, px - gw / 2, py - gw / 2, gw, gw);
  }
  // Anchor at the token's centre: the figure's body is centred on the hex, the base just below it.
  return { c, ax: cx, ay: top + S * 0.5, w: size, h: size };
}

/** `hex` mixed toward ink by `t`, as a hex string. */
function mixInk(hex: string, t: number): string {
  const ink = ATLAS.tone.ink;
  const part = (i: number) => Math.round(parseInt(hex.slice(i, i + 2), 16) * (1 - t) + parseInt(ink.slice(i, i + 2), 16) * t).toString(16).padStart(2, '0');
  return `#${part(1)}${part(3)}${part(5)}`;
}

function atlasToken(icon: string, color: string, military: boolean, w: number, dpr: number, heir?: HeirMark): Sprite {
  const I = INK.atlas;
  const { pen, seat } = ATLAS;
  const pad = heir?.crestColor ? 4 + Math.ceil(w * 0.6) : 4;
  const size = 2 * w + 2 * pad;
  const [c, g] = canvas(size, size, dpr);
  const o = pad + w;
  // Military counters are the seat colour (muted, over parchment) with the glyph on a small parchment inset, so the colour is the first read.
  // Civilians stay parchment chits with a seat-colour rim, so military and civilian also read at a glance.
  g.fillStyle = ATLAS.tone.shadow;
  g.fillRect(o - w + 1.5, o - w + 2, 2 * w, 2 * w);
  g.fillStyle = military ? seat.plateMilitary : seat.plate;
  g.fillRect(o - w, o - w, 2 * w, 2 * w);
  if (military) {
    g.fillStyle = rgba(color, 0.78);
    g.fillRect(o - w, o - w, 2 * w, 2 * w);
    // A thicker, darker rim of the same seat colour, and an inner paper hairline.
    const rim = Math.max(2.8, w * 0.22);
    g.strokeStyle = rgba(mixInk(color, 0.3), 1);
    g.lineWidth = rim;
    g.strokeRect(o - w + rim / 2, o - w + rim / 2, 2 * w - rim, 2 * w - rim);
    g.strokeStyle = rgba(seat.plate, 0.55);
    g.lineWidth = pen.hair;
    g.strokeRect(o - w + rim + 1, o - w + rim + 1, 2 * w - 2 * rim - 2, 2 * w - 2 * rim - 2);
    const half = w * 0.56;
    g.fillStyle = seat.plate;
    g.fillRect(o - half, o - half, 2 * half, 2 * half);
    g.strokeStyle = rgba(I.ink, 0.6);
    g.lineWidth = pen.hair;
    g.strokeRect(o - half, o - half, 2 * half, 2 * half);
  } else {
    const rim = Math.max(2, w * 0.15);
    g.strokeStyle = color;
    g.lineWidth = rim;
    const b = rim / 2 + pen.thin;
    g.strokeRect(o - w + b, o - w + b, 2 * w - 2 * b, 2 * w - 2 * b);
  }
  g.strokeStyle = I.ink;
  g.lineWidth = pen.thin;
  g.strokeRect(o - w, o - w, 2 * w, 2 * w);
  if (heir) {
    // A gilt line inside the seat rim, ink-edged on both sides so it reads on any parchment.
    const rim = Math.max(2, w * 0.15), at = rim + Math.max(2, w * 0.1);
    const x0 = o - w + at, side = 2 * (w - at);
    g.strokeStyle = I.ink;
    g.lineWidth = Math.max(2.6, w * 0.2);
    g.strokeRect(x0, o - w + at, side, side);
    g.strokeStyle = heirGilt();
    g.lineWidth = Math.max(1.6, w * 0.12);
    g.strokeRect(x0, o - w + at, side, side);
  }
  const iw = military ? w * 0.92 : w * 1.3;
  const ic = tintedIcon(icon, I.ink, iw * dpr);
  if (ic) g.drawImage(ic, o - iw / 2, o - iw / 2, iw, iw);
  if (heir?.crestColor) {
    // The Byzantium's crest, pinned on the lower-right corner: a parchment disc ringed in its seat colour.
    const r = Math.max(7, w * 0.58), cx = o + w * 0.9, cy = o + w * 0.9;
    g.beginPath(); g.arc(cx, cy, r, 0, Math.PI * 2); g.fillStyle = seat.plate; g.fill();
    g.beginPath(); g.arc(cx, cy, r - 1.1, 0, Math.PI * 2); g.strokeStyle = heir.crestColor; g.lineWidth = 2.2; g.stroke();
    g.beginPath(); g.arc(cx, cy, r, 0, Math.PI * 2); g.strokeStyle = I.ink; g.lineWidth = pen.thin; g.stroke();
    const gw = r * 1.5;
    const gl = tintedIcon('faction-byzantium', I.ink, gw * dpr);
    if (gl) g.drawImage(gl, cx - gw / 2, cy - gw / 2, gw, gw);
  }
  return { c, ax: o, ay: o, w: size, h: size };
}

/** Health bar under the counter. */
export function drawTokenHealth(ctx: CanvasRenderingContext2D, _look: LookId, hp: number, x: number, y: number, w: number, drop = 0) {
  // A health bar under the token: a clear empty track, a green-amber-red
  // fill and an outline, so who is hurt (and how badly) reads at a glance after a battle.
  const f = Math.max(0, Math.min(1, hp / 100));
  const bw = 2 * w, bh = 4.5, bx = x - w, by = y + w * 1.04 + 3 + drop;
  ctx.fillStyle = rgba(INK.atlas.paper, 0.95);
  ctx.fillRect(bx, by, bw, bh);
  ctx.fillStyle = healthColor(f);
  ctx.fillRect(bx, by, bw * f, bh);
  ctx.strokeStyle = INK.atlas.ink;
  ctx.lineWidth = 0.8;
  ctx.strokeRect(bx, by, bw, bh);
}

/** The enlisted badge straddles the token's lower-left edge, below its owner roundel. */
export function drawPromotion(ctx: CanvasRenderingContext2D, _look: LookId, level: number, x: number, y: number, w: number, faction?: Faction) {
  const rank = rankLevel(level, faction);
  if (!rank) return;
  const width = Math.max(16, Math.min(20, w * 0.82));
  const height = width * 34 / 24;
  const bottom = y + w * 0.98 + 2;
  drawRankInsignia(ctx, rank, x - w - width / 2, bottom - height, width);
}

/** The SVG uses these exact paths, colours and weights. At far zoom the badge stays screen-sized. */
const rankPaths = new Map<string, Path2D>();
export function drawRankInsignia(ctx: CanvasRenderingContext2D, rank: RankLevel, left: number, top: number, width = 18) {
  const pathOf = (d: string) => {
    let p = rankPaths.get(d);
    if (!p) { p = new Path2D(d); rankPaths.set(d, p); }
    return p;
  };
  const fill = (path: string, color: string) => { ctx.fillStyle = color; ctx.fill(pathOf(path)); };
  const stroke = (path: string, color: string, lineWidth: number) => {
    ctx.strokeStyle = color; ctx.lineWidth = lineWidth; ctx.stroke(pathOf(path));
  };
  ctx.save();
  ctx.translate(left, top);
  ctx.scale(width / 24, width / 24);
  ctx.lineJoin = 'round';
  ctx.lineCap = 'round';
  fill(RANK_PATCH, RANK_INK);
  stroke(RANK_PATCH, RANK_EDGE, 0.8);
  const { chevrons, rockers, device } = rankStrokes(rank);
  for (const path of [...chevrons, ...rockers]) {
    stroke(path, RANK_EDGE, 3.1);
    stroke(path, RANK_GOLD, 1.35);
  }
  if (device) {
    stroke(RANK_LAMBDA, RANK_INK, 4);
    stroke(RANK_LAMBDA, RANK_GOLD, 2.5);
  }
  ctx.restore();
}

/** A small round badge (pending order, goto, fortified). */
export function drawBadge(ctx: CanvasRenderingContext2D, look: LookId, x: number, y: number, r: number, label: string, strong: boolean, icon?: string) {
  const rr = Math.max(5.5, r);
  ctx.beginPath();
  ctx.arc(x, y, rr, 0, Math.PI * 2);
  ctx.fillStyle = strong ? INK.atlas.red : INK.atlas.vellum;
  ctx.fill();
  ctx.strokeStyle = INK.atlas.ink;
  ctx.lineWidth = 1;
  ctx.stroke();
  if (icon) {
    // An order badge shows an icon, not a letter (nothing is drawn until the icon has loaded).
    const px = rr * 1.6;
    const ic = tintedIcon(icon, strong ? INK.atlas.vellum : INK.atlas.ink, px * (ctx.getTransform().a || 1));
    if (ic) ctx.drawImage(ic, x - px / 2, y - px / 2, px, px);
    return;
  }
  ctx.fillStyle = strong ? INK.atlas.vellum : INK.atlas.ink;
  ctx.font = `700 ${Math.round(rr * 1.3)}px ${FONTS[look].sans}`;
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.fillText(label, x, y + 0.5);
  ctx.textAlign = 'left';
  ctx.textBaseline = 'alphabetic';
}

/**
 * A small word tag under a token ("Byzantium's Heir", "Held by Sparta"), centred on `x` with its
 * top at `y`. `bad` uses the warning red, `good` the ally green; neutral is plain ink. The word
 * always carries the meaning, never the colour alone.
 */
export function drawTag(ctx: CanvasRenderingContext2D, look: LookId, x: number, y: number, text: string, tone: 'good' | 'bad' | 'neutral', alpha = 1) {
  ctx.save();
  ctx.globalAlpha = alpha;
  const fs = 10;
  ctx.font = `700 ${fs}px ${FONTS[look].sans}`;
  const tw = ctx.measureText(text).width + 8, th = fs + 5;
  const bx = x - tw / 2;
  const fill = tone === 'bad' ? INK.atlas.red : tone === 'good' ? INK.atlas.good : INK.atlas.vellum;
  ctx.beginPath();
  ctx.rect(bx, y, tw, th);
  ctx.fillStyle = fill;
  ctx.fill();
  ctx.strokeStyle = INK.atlas.ink;
  ctx.lineWidth = 1;
  ctx.stroke();
  ctx.fillStyle = tone === 'neutral' ? INK.atlas.ink : INK.atlas.light;
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.fillText(text, x, y + th / 2 + 0.5);
  ctx.restore();
}

/** Under Byzantium's Heir on a tile nobody of yours can see: a gilt halo, so he reads as "here" over the fog (his position is public). */
export function drawHeirHalo(ctx: CanvasRenderingContext2D, x: number, y: number, w: number) {
  ctx.save();
  const r = w * 1.5;
  const g = ctx.createRadialGradient(x, y, w * 0.4, x, y, r);
  g.addColorStop(0, 'rgba(240,214,120,0.85)');
  g.addColorStop(1, 'rgba(240,214,120,0)');
  ctx.fillStyle = g;
  ctx.beginPath(); ctx.arc(x, y, r, 0, Math.PI * 2); ctx.fill();
  // A solid gilt ring with an ink rim: he is here, now, and everyone can see it.
  ctx.beginPath(); ctx.arc(x, y, w * 1.3, 0, Math.PI * 2); ctx.strokeStyle = INK.atlas.ink; ctx.lineWidth = 4.4; ctx.stroke();
  ctx.beginPath(); ctx.arc(x, y, w * 1.3, 0, Math.PI * 2); ctx.strokeStyle = heirGilt(); ctx.lineWidth = 2.6; ctx.stroke();
  ctx.restore();
}

/**
 * A hired Company's contract mark (public), centred under its token: the seller's faction seal and the turns of contract left
 * ("7 left"), or "Last turn" in red when it goes home this turn. The words carry the meaning; the seal says whose it was.
 */
export function drawContractTag(ctx: CanvasRenderingContext2D, look: LookId, x: number, y: number, origin: Faction | undefined, left: number) {
  const text = left === 0 ? 'Last turn' : `${left} turn${left === 1 ? '' : 's'}`;
  const last = left === 0;
  ctx.save();
  const fs = 10;
  ctx.font = `700 ${fs}px ${FONTS[look].sans}`;
  const seal = origin ? 11 : 0;
  const tw = ctx.measureText(text).width + 8 + (seal ? seal + 2 : 0), th = fs + 5;
  const bx = x - tw / 2;
  ctx.beginPath();
  ctx.rect(bx, y, tw, th);
  ctx.fillStyle = last ? INK.atlas.red : INK.atlas.vellum;
  ctx.fill();
  ctx.strokeStyle = INK.atlas.ink;
  ctx.lineWidth = 1;
  ctx.stroke();
  if (origin) {
    const ic = tintedIcon(`faction-${origin.toLowerCase()}`, last ? INK.atlas.light : INK.atlas.ink, seal * (ctx.getTransform().a || 1));
    if (ic) ctx.drawImage(ic, bx + 3, y + (th - seal) / 2, seal, seal);
  }
  ctx.fillStyle = last ? INK.atlas.light : INK.atlas.ink;
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.fillText(text, x + (seal ? (seal + 2) / 2 : 0), y + th / 2 + 0.5);
  ctx.restore();
}

/** Zoomed out, a Company's pip carries only its turns left, on a small ink plaque at the lower right (red on its last turn). */
export function drawContractPip(ctx: CanvasRenderingContext2D, look: LookId, x: number, y: number, half: number, left: number) {
  const text = left === 0 ? '!' : String(left);
  ctx.save();
  ctx.font = `700 9px ${FONTS[look].sans}`;
  const w = Math.max(11, ctx.measureText(text).width + 6), h = 11;
  const bx = x + half * 0.5, by = y + half * 0.45;
  ctx.beginPath();
  ctx.rect(bx, by, w, h);
  ctx.fillStyle = left === 0 ? INK.atlas.red : INK.atlas.ink;
  ctx.fill();
  ctx.strokeStyle = INK.atlas.vellum;
  ctx.lineWidth = 1;
  ctx.stroke();
  ctx.fillStyle = INK.atlas.vellum;
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.fillText(text, bx + w / 2, by + h / 2 + 0.5);
  ctx.restore();
}

/** An ownership roundel, separate from population and order badges: the faction glyph used on race portraits, ringed in its seat colour. */
function drawPlayerNumber(ctx: CanvasRenderingContext2D, look: LookId, owner: number, x: number, y: number, faction?: Faction) {
  if (owner === 255) return; // Barbarians carry no faction crest; their token is their mark.
  ctx.save();
  ctx.beginPath();
  ctx.arc(x, y, 7.5, 0, Math.PI * 2);
  ctx.fillStyle = ATLAS.seat.plate;
  ctx.fill();
  ctx.strokeStyle = playerInk(owner);
  ctx.lineWidth = 2.2;
  ctx.stroke();
  ctx.beginPath();
  ctx.arc(x, y, 8.7, 0, Math.PI * 2);
  ctx.strokeStyle = INK.atlas.ink;
  ctx.lineWidth = ATLAS.pen.hair;
  ctx.stroke();
  const mark = faction ? tintedIcon(`faction-${faction.toLowerCase()}`, playerInk(owner), 12) : null;
  if (mark) ctx.drawImage(mark, x - 6, y - 6, 12, 12);
  else {
    // A missing faction still has a legible fallback until its identity is available.
    ctx.fillStyle = INK.atlas.ink;
    ctx.font = `700 11px ${FONTS[look].sans}`;
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    ctx.fillText(String(owner + 1), x, y + 0.5);
  }
  ctx.restore();
}

// ---- barbarians ----

/**
 * Barbarians draw in the atlas's own ink: an aged plate hatched dark, a heavy ink rim and the one accent red, never a seat colour.
 * A counter has its top-left corner torn off (a red stain along the tear), a camp is an inked palisade and pennant on the land.
 */
const barb = () => ATLAS.barb;

/**
 * A barbarian unit at normal zoom: its painted raider (`barbarian-*` in `assets/units`) on a charred ink stone base in an ink ring
 * with a blood-red notch (a red ring read as the crimson seat), never a seat colour and with no owner roundel. The torn engraved counter stands in only while the painting loads.
 */
export function drawBarbarianToken(ctx: CanvasRenderingContext2D, look: LookId, kind: UnitKind, military: boolean, x: number, y: number, w: number, dpr: number) {
  const q = Math.round(w * 2) / 2;
  const art = barbarianMiniatureImage(kind);
  const sp = art
    ? cached(`barbmini|${kind}|${q}|${dpr}`, () => miniatureToken(art, barb().ink, q, dpr, undefined, barb().ink, barb().blood))
    : cached(`barb|${look}|${kind}|${military}|${q}|${dpr}`, () => barbarianToken(look, kind, military, q, dpr));
  ctx.drawImage(sp.c, x - sp.ax, y - sp.ay, sp.w, sp.h);
}

/** Whether a barbarian draws as a painted miniature right now (its health bar and tag sit lower under the bigger base). */
export const barbarianMiniature = (kind: UnitKind) => !!barbarianMiniatureImage(kind);

function barbarianToken(_look: LookId, kind: UnitKind, military: boolean, w: number, dpr: number): Sprite {
  const { pen } = ATLAS;
  const B = barb();
  const pad = 4;
  const r = w;
  const size = 2 * r + 2 * pad;
  const [c, g] = canvas(size, size + 2, dpr);
  const o = pad + r;
  const cut = r * 0.55;
  const shape = (inset: number) => {
    g.beginPath();
    // A counter with its top-left corner torn off.
    const s = r - inset;
    g.moveTo(o - s + cut, o - s);
    g.lineTo(o + s, o - s);
    g.lineTo(o + s, o + s);
    g.lineTo(o - s, o + s);
    g.lineTo(o - s, o - s + cut);
    g.closePath();
  };
  g.save();
  g.translate(1.5, 2);
  shape(0);
  g.fillStyle = ATLAS.tone.shadow;
  g.fill();
  g.restore();
  shape(0);
  g.fillStyle = B.plate;
  g.fill();
  // Hatched dark, like shading on an engraving.
  g.save();
  shape(0);
  g.clip();
  g.strokeStyle = rgba(B.ink, military ? 0.2 : 0.12);
  g.lineWidth = pen.hair;
  g.beginPath();
  for (let d = -2 * r; d <= 2 * r; d += 3) { g.moveTo(o + d - r, o + r); g.lineTo(o + d + r, o - r); }
  g.stroke();
  g.restore();
  g.strokeStyle = B.ink;
  g.lineWidth = Math.max(2.4, r * 0.2);
  if (!military) g.setLineDash([r * 0.45, r * 0.2]);
  shape(g.lineWidth / 2 + 0.5);
  g.stroke();
  g.setLineDash([]);
  // The torn edge, stained red.
  g.strokeStyle = B.blood;
  g.lineWidth = Math.max(2, r * 0.16);
  g.beginPath(); g.moveTo(o - r + cut, o - r); g.lineTo(o - r, o - r + cut); g.stroke();
  g.strokeStyle = B.ink;
  g.lineWidth = pen.thin;
  shape(0);
  g.stroke();
  const iw = r * 1.2;
  const ic = tintedIcon(unitIconName(kind), B.ink, iw * dpr);
  if (ic) g.drawImage(ic, o - iw / 2, o - iw / 2, iw, iw);
  return { c, ax: o, ay: o, w: size, h: size + 2 };
}

// ---- the zoomed-out tier: unit pips, city seals and names ----

/**
 * A unit when zoomed out: its class roundel, the owner's colour as disc wash and rim. Barbarians wear the same class roundel washed
 * blood red under a heavy ink ring, with the torn red corner of their counters, never a seat colour. The parchment pip (square
 * military, round civilian) stands in while a roundel loads.
 */
export function drawPip(ctx: CanvasRenderingContext2D, look: LookId, owner: number, military: boolean, x: number, y: number, half: number, dpr: number, heir = false, kind?: UnitKind, faction?: Faction) {
  // A seat's unit is a class roundel (the painted set's far-zoom mark): the class silhouette in ink, the owner's colour as disc wash and rim.
  const barbarian = owner === 255;
  const round = kind ? roundelImage(kind, barbarian ? undefined : faction, barbarian ? barb().blood : mapPlayerColor(owner)) : null;
  if (round && barbarian) {
    const sp = cached(`bpip|${kind}|${half}|${dpr}`, () => {
      const B = barb();
      const pad = 3, r = half + 1, size = 2 * r + 2 * pad;
      const [c, g] = canvas(size, size, dpr);
      const o = pad + r;
      g.beginPath(); g.arc(o + 1, o + 1.4, r, 0, Math.PI * 2); g.fillStyle = ATLAS.tone.shadow; g.fill();
      g.drawImage(round, o - r, o - r, 2 * r, 2 * r);
      g.beginPath(); g.arc(o, o, r - 1.4, 0, Math.PI * 2); g.strokeStyle = B.ink; g.lineWidth = 2.8; g.stroke();
      // The torn corner: a blood-red notch out of the ring's upper left, as on the barbarian counter and camp pennant.
      g.beginPath(); g.arc(o, o, r - 1.4, Math.PI * 1.08, Math.PI * 1.42); g.strokeStyle = B.blood; g.lineWidth = 2.8; g.stroke();
      g.beginPath(); g.arc(o, o, r, 0, Math.PI * 2); g.strokeStyle = INK.atlas.ink; g.lineWidth = ATLAS.pen.thin; g.stroke();
      return { c, ax: o, ay: o, w: size, h: size };
    });
    ctx.drawImage(sp.c, x - sp.ax, y - sp.ay, sp.w, sp.h);
    return;
  }
  if (round) {
    const sp = cached(`rpip|${kind}|${faction ?? ''}|${owner}|${half}|${dpr}|${heir ? 'heir' : ''}`, () => {
      const pad = heir ? 6 : 3, r = half + 1, size = 2 * r + 2 * pad;
      const [c, g] = canvas(size, size, dpr);
      const o = pad + r;
      g.beginPath(); g.arc(o + 1, o + 1.4, r, 0, Math.PI * 2); g.fillStyle = ATLAS.tone.shadow; g.fill();
      g.drawImage(round, o - r, o - r, 2 * r, 2 * r);
      // A firm owner ring over the roundel's thin rim, so seats stay apart at 16 px.
      g.beginPath(); g.arc(o, o, r - 1.4, 0, Math.PI * 2); g.strokeStyle = playerInk(owner); g.lineWidth = 2.6; g.stroke();
      g.beginPath(); g.arc(o, o, r, 0, Math.PI * 2); g.strokeStyle = INK.atlas.ink; g.lineWidth = ATLAS.pen.thin; g.stroke();
      // The centre stays identifiable by emblem shape even when the unit art is only a few pixels wide.
      const mark = faction ? tintedIcon(`faction-${faction.toLowerCase()}`, playerInk(owner), Math.max(8, Math.round(half * 1.35 * dpr))) : null;
      if (mark) g.drawImage(mark, o - Math.max(4, half * 0.68), o - Math.max(4, half * 0.68), Math.max(8, half * 1.35), Math.max(8, half * 1.35));
      if (heir) {
        g.beginPath(); g.arc(o, o, r + 2.6, 0, Math.PI * 2); g.strokeStyle = INK.atlas.ink; g.lineWidth = 3.6; g.stroke();
        g.beginPath(); g.arc(o, o, r + 2.6, 0, Math.PI * 2); g.strokeStyle = heirGilt(); g.lineWidth = 2; g.stroke();
      }
      return { c, ax: o, ay: o, w: size, h: size };
    });
    ctx.drawImage(sp.c, x - sp.ax, y - sp.ay, sp.w, sp.h);
    return;
  }
  const sp = cached(`pip|${look}|${owner === 255 ? 'b' : owner}|${military}|${half}|${dpr}|${heir ? 'heir' : ''}`, () => {
    const { pen, seat } = ATLAS;
    const B = barb();
    const pad = heir ? 6 : 3, size = 2 * half + 2 * pad;
    const [c, g] = canvas(size, size, dpr);
    const o = pad + half;
    const shape = (inset: number) => {
      g.beginPath();
      if (military || owner === 255) g.rect(o - half + inset, o - half + inset, 2 * (half - inset), 2 * (half - inset));
      else g.arc(o, o, half - inset, 0, Math.PI * 2);
    };
    g.save(); g.translate(1, 1.4); shape(0); g.fillStyle = ATLAS.tone.shadow; g.fill(); g.restore();
    shape(0);
    g.fillStyle = owner === 255 ? B.plate : seat.plate;
    g.fill();
    const rim = owner === 255 ? 2.2 : 2.4;
    shape(rim / 2);
    g.strokeStyle = owner === 255 ? B.ink : playerInk(owner);
    g.lineWidth = rim;
    g.stroke();
    shape(0);
    g.strokeStyle = INK.atlas.ink;
    g.lineWidth = pen.thin;
    g.stroke();
    if (owner === 255) { g.fillStyle = B.blood; g.fillRect(o - half, o - half, 2.4, 2.4); }
    else if (faction) {
      const mark = tintedIcon(`faction-${faction.toLowerCase()}`, playerInk(owner), Math.max(8, Math.round(half * 1.35 * dpr)));
      const markSize = Math.max(8, half * 1.35);
      if (mark) g.drawImage(mark, o - markSize / 2, o - markSize / 2, markSize, markSize);
    }
    if (heir) {
      // The Heir's pip: a gilt ring just outside the seat rim and a gilt dot, so the one pip that scores stands out.
      g.beginPath(); g.arc(o, o, half + 2.6, 0, Math.PI * 2); g.strokeStyle = INK.atlas.ink; g.lineWidth = 3.6; g.stroke();
      g.beginPath(); g.arc(o, o, half + 2.6, 0, Math.PI * 2); g.strokeStyle = heirGilt(); g.lineWidth = 2; g.stroke();
      g.beginPath(); g.arc(o, o, half * 0.34, 0, Math.PI * 2); g.fillStyle = heirGilt(); g.fill(); g.strokeStyle = INK.atlas.ink; g.lineWidth = pen.thin; g.stroke();
    }
    return { c, ax: o, ay: o, w: size, h: size };
  });
  ctx.drawImage(sp.c, x - sp.ax, y - sp.ay, sp.w, sp.h);
}

/** A city's seal when zoomed out: a parchment disc ringed in the seat colour, the faction glyph (or population) in ink, the founder's star for an original capital. */
export function sealSprite(look: LookId, b: BannerSpec, r: number, dpr: number): Sprite {
  const q = Math.round(r * 2) / 2;
  return cached(`seal|${look}|${b.owner}|${b.pop}|${starKey(b.capital)}|${b.selected}|${b.glyph}|${q}|${dpr}`, () => {
    const I = INK.atlas, { pen, seat } = ATLAS;
    // The capital's star stands out past the disc, so a marked seal gets the room it needs.
    const pad = b.capital ? Math.max(4, Math.ceil(q * 0.95) + 1) : 4, size = 2 * q + 2 * pad;
    const [c, g] = canvas(size, size, dpr);
    const o = pad + q;
    g.beginPath(); g.arc(o + 1, o + 1.4, q, 0, Math.PI * 2); g.fillStyle = ATLAS.tone.shadow; g.fill();
    g.beginPath(); g.arc(o, o, q, 0, Math.PI * 2); g.fillStyle = seat.plate; g.fill();
    const ring = Math.max(2.2, q * 0.24);
    g.beginPath(); g.arc(o, o, q - ring / 2, 0, Math.PI * 2); g.strokeStyle = mapPlayerColor(b.owner); g.lineWidth = ring; g.stroke();
    g.beginPath(); g.arc(o, o, q, 0, Math.PI * 2); g.strokeStyle = b.selected ? I.red : I.ink; g.lineWidth = b.selected ? pen.line : pen.thin; g.stroke();
    g.beginPath(); g.arc(o, o, q - ring - 0.6, 0, Math.PI * 2); g.strokeStyle = rgba(I.ink, 0.4); g.lineWidth = pen.hair; g.stroke();
    const gw = (q - ring) * 1.55;
    const ic = b.glyph ? tintedIcon(b.glyph, I.ink, gw * dpr) : null;
    if (ic) g.drawImage(ic, o - gw / 2, o - gw / 2, gw, gw);
    else if (b.pop !== null) {
      g.fillStyle = I.ink; g.font = `700 ${Math.round(q * 0.95)}px ${FONTS.atlas.sans}`; g.textAlign = 'center'; g.textBaseline = 'middle';
      g.fillText(String(b.pop), o, o + 0.5);
    }
    if (b.capital) {
      // The banner's star seal, small: a plate with an ink rule and a fine gilt ring round the founder's star.
      // Outside the disc and at most 0.75 of it, so the glyph or population under it stays readable.
      const sx = o + q * 1.1, sy = o - q * 1.1, rd = q * 0.75;
      g.beginPath(); g.arc(sx, sy, rd, 0, Math.PI * 2); g.fillStyle = seat.plate; g.fill();
      g.strokeStyle = I.ink; g.lineWidth = pen.thin; g.stroke();
      g.beginPath(); g.arc(sx, sy, rd - 1.5, 0, Math.PI * 2); g.strokeStyle = CAPITAL_GILT; g.lineWidth = 1.1; g.stroke();
      drawCapitalStar(g, b.capital, sx, sy, q * 0.6, seat.plate, 0.3);
    }
    return { c, ax: o, ay: o, w: size, h: size };
  });
}

/** A city's name when zoomed out: small capitals in ink with a parchment halo, cached. */
export function nameSprite(look: LookId, name: string, dpr: number): Sprite {
  return cached(`sname|${look}|${name}|${dpr}`, () => {
    const F = FONTS.atlas, px = 11;
    const disp = (p: number) => `700 ${p}px ${F.display}`;
    const [, m] = canvas(1, 1, 1);
    const w = Math.ceil(smallCaps(m, name, 0, 0, disp, px, px * 0.06, false)) + 8, h = Math.round(px * 1.5) + 4;
    const [c, g] = canvas(w, h, dpr);
    g.fillStyle = INK.atlas.ink;
    g.textBaseline = 'alphabetic';
    smallCaps(g, name, 4, h / 2 + px * 0.32, disp, px, px * 0.06, true, rgba(INK.atlas.vellum, 0.9));
    return { c, ax: w / 2, ay: h / 2, w, h };
  });
}

export const drawSprite = (ctx: CanvasRenderingContext2D, sp: Sprite, x: number, y: number, alpha = 1) => {
  ctx.globalAlpha = alpha;
  ctx.drawImage(sp.c, x - sp.ax, y - sp.ay, sp.w, sp.h);
  ctx.globalAlpha = 1;
};

// ---- city banners ----

export interface BannerSpec {
  owner: number;
  name: string;
  pop: number | null;
  color: string;
  /** The original-capital star, kept for the whole game: its ink and whether the viewer holds its 1 VP now. */
  capital: CapitalStar | null;
  selected: boolean;
  glyph: string | null;
  walls: boolean;
}

/** Draw a city banner with its bottom center at (x, y). Returns the drawn rect (CSS px). */
export function drawBanner(ctx: CanvasRenderingContext2D, look: LookId, b: BannerSpec, x: number, y: number, fs: number, dpr: number, alpha = 1) {
  const sp = bannerSprite(look, b, fs, dpr);
  const rect = bannerRect(look, b, x, y, fs, dpr);
  ctx.globalAlpha = alpha;
  ctx.drawImage(sp.c, rect.x, rect.y, rect.w, rect.h);
  ctx.globalAlpha = 1;
  return rect;
}

/** Where drawBanner would draw (CSS px), without drawing; painted resource medallions use these
 * bounds to move to their clear side slot when the banner covers the top of the hex. */
export function bannerRect(look: LookId, b: BannerSpec, x: number, y: number, fs: number, dpr: number) {
  const sp = bannerSprite(look, b, fs, dpr);
  return { x: Math.round(x - sp.ax), y: Math.round(y - sp.ay), w: sp.w, h: sp.h };
}

function bannerSprite(look: LookId, b: BannerSpec, fs: number, dpr: number): Sprite {
  const q = Math.round(fs * 2) / 2;
  return cached(`ban|${look}|${b.owner}|${b.name}|${b.pop}|${b.color}|${starKey(b.capital)}|${b.selected}|${b.glyph}|${b.walls}|${q}|${dpr}`, () =>
    atlasBanner(b, q, dpr),
  );
}

/** One compact meter under an own city's banner (Civ 6 style): growth (the territory bar: each growth claims a tile) or production. */
export interface BannerMeter {
  kind: 'growth' | 'production';
  /** 0..1, quantized by the caller so sprites cache. */
  fraction: number;
  /** Turns left, or a dash when not progressing. */
  text: string;
  /** Starvation or a production stall: red fill and text. Also the idle "choose" marker. */
  bad?: boolean;
  paused?: boolean;
  /** Production only: the icon for the item being built. Omitted (and `icon: null`) when idle,
   * which draws a "?" marker instead so an idle city stands out on the map. */
  icon?: string | null;
  /** Production only: the city builds automatically. "Auto" and the focus icon follow the turns, inside the strip, so the mark never overlaps the name. */
  auto?: { label: string; icon: string };
  /** The icon is a label, not an item in progress: ink, not the meter's colour. */
  inkIcon?: boolean;
}

/** Food green and production orange, matching --y-food in brand/tokens.css. */
const METER_COLOR = { growth: ATLAS.tone.growth, production: ATLAS.tone.production } as const;
const METER_BAR = 28;

/** Draw the meters strip with its top center at (x, y). Returns the drawn rect (CSS px). */
export function drawBannerMeters(ctx: CanvasRenderingContext2D, look: LookId, meters: BannerMeter[], x: number, y: number, dpr: number) {
  const sp = metersSprite(look, meters, dpr);
  const rect = { x: Math.round(x - sp.w / 2), y: Math.round(y), w: sp.w, h: sp.h };
  ctx.drawImage(sp.c, rect.x, rect.y, rect.w, rect.h);
  return rect;
}

export function bannerMetersSize(look: LookId, meters: BannerMeter[], dpr: number) {
  const sp = metersSprite(look, meters, dpr);
  return { w: sp.w, h: sp.h };
}

function metersSprite(look: LookId, meters: BannerMeter[], dpr: number): Sprite {
  const key = meters.map((m) => `${m.kind}:${m.fraction}:${m.text}:${m.bad ? 1 : 0}:${m.paused ? 1 : 0}:${m.icon ?? ''}:${m.inkIcon ? 1 : 0}:${m.auto ? `${m.auto.label}/${m.auto.icon}` : ''}`).join(',');
  return cached(`meters|${look}|${key}|${dpr}`, () => {
    const font = `700 11px ${FONTS[look].sans}`;
    const [, m] = canvas(1, 1, 1);
    m.font = font;
    const icon = 11, pad = 5, gap = 9, h = 16;
    const autoW = (mt: BannerMeter) => mt.auto ? 6 + icon + 1 + Math.ceil(m.measureText(mt.auto.label).width) : 0;
    const widths = meters.map((mt) => icon + 2 + METER_BAR + 3 + Math.ceil(m.measureText(mt.text).width) + autoW(mt));
    const W = pad * 2 + widths.reduce((a, b) => a + b, 0) + gap * Math.max(0, meters.length - 1);
    const [c, g] = canvas(W, h, dpr);
    g.fillStyle = INK.atlas.vellum;
    g.fillRect(0, 0, W, h);
    g.strokeStyle = INK.atlas.ink;
    g.lineWidth = 1;
    g.strokeRect(0.5, 0.5, W - 1, h - 1);
    let cx = pad;
    const mid = h / 2;
    meters.forEach((mt, i) => {
      const color = mt.bad ? INK.atlas.red : mt.paused ? INK.atlas.rule : METER_COLOR[mt.kind];
      if (mt.kind === 'growth') {
        const ic = tintedIcon('yield-food', color, icon * dpr);
        if (ic) g.drawImage(ic, cx, mid - icon / 2, icon, icon);
        else { g.fillStyle = color; g.beginPath(); g.arc(cx + icon / 2, mid, icon / 2 - 1, 0, Math.PI * 2); g.fill(); }
      } else if (mt.icon) {
        // Production: the icon of the item being built.
        const ic = tintedIcon(mt.icon, mt.inkIcon ? INK.atlas.ink : color, icon * dpr);
        if (ic) g.drawImage(ic, cx, mid - icon / 2, icon, icon);
        else { g.fillStyle = color; g.beginPath(); g.arc(cx + icon / 2, mid, icon / 2 - 1, 0, Math.PI * 2); g.fill(); }
      } else {
        // Idle: nothing queued. A "?" marker draws the eye, so an idle city reads at a glance.
        g.fillStyle = color;
        g.beginPath();
        g.arc(cx + icon / 2, mid, icon / 2, 0, Math.PI * 2);
        g.fill();
        g.fillStyle = INK.atlas.vellum;
        g.font = `700 ${Math.round(icon * 0.8)}px ${FONTS[look].sans}`;
        g.textAlign = 'center';
        g.textBaseline = 'middle';
        g.fillText('?', cx + icon / 2, mid + 0.5);
        g.textAlign = 'left';
        g.textBaseline = 'alphabetic';
      }
      cx += icon + 2;
      const by = mid - 2.5;
      g.fillStyle = INK.atlas.track;
      g.fillRect(cx, by, METER_BAR, 5);
      g.fillStyle = color;
      g.fillRect(cx, by, METER_BAR * Math.max(0, Math.min(1, mt.fraction)), 5);
      g.strokeStyle = INK.atlas.rule;
      g.lineWidth = 0.6;
      g.strokeRect(cx + 0.3, by + 0.3, METER_BAR - 0.6, 4.4);
      cx += METER_BAR + 3;
      g.fillStyle = mt.bad ? color : INK.atlas.ink;
      g.font = font;
      g.textBaseline = 'middle';
      g.fillText(mt.text, cx, mid + 0.5);
      if (mt.auto) {
        const ax = cx + Math.ceil(m.measureText(mt.text).width) + 6;
        const ai = tintedIcon(mt.auto.icon, INK.atlas.ink, icon * dpr);
        if (ai) g.drawImage(ai, ax, mid - icon / 2, icon, icon);
        g.fillStyle = INK.atlas.ink;
        g.fillText(mt.auto.label, ax + icon + 1, mid + 0.5);
      }
      cx += widths[i] - (icon + 2 + METER_BAR + 3) + gap;
    });
    return { c, ax: W / 2, ay: 0, w: W, h };
  });
}

const STAR = (g: CanvasRenderingContext2D, x: number, y: number, r: number, waist = 0.19) => {
  // The house four-point star; a wider waist for the outlined (scored) form, which needs room inside.
  const k = waist * r;
  g.beginPath();
  g.moveTo(x, y - r);
  g.lineTo(x + k, y - k);
  g.lineTo(x + r, y);
  g.lineTo(x + k, y + k);
  g.lineTo(x, y + r);
  g.lineTo(x - k, y + k);
  g.lineTo(x - r, y);
  g.lineTo(x - k, y - k);
  g.closePath();
  g.fill();
};

/** An original capital's star: filled in the founder's ink, or hollow (outlined, plate inside) while the viewer holds its 1 VP. */
function drawCapitalStar(g: CanvasRenderingContext2D, star: CapitalStar, x: number, y: number, r: number, plate: string, filledWaist = 0.19) {
  if (!star.scored) { g.fillStyle = star.ink; STAR(g, x, y, r, filledWaist); return; }
  g.fillStyle = plate; STAR(g, x, y, r, 0.4);
  g.strokeStyle = star.ink; g.lineWidth = Math.max(1.2, r * 0.26); g.lineJoin = 'round'; g.stroke();
  g.lineJoin = 'miter';
}

const starKey = (star: CapitalStar | null) => (star ? `${star.ink}${star.scored ? 'h' : 'f'}` : '-');

/** The fine ring on an original capital's seals (gilt is the house's VP colour). */
const CAPITAL_GILT = '#c9a227';

/** Radius of the banner's star seal: a disc about 1.4 banners tall (never under 16 px across at the smallest banner). */
export function capitalSealRadius(fs: number): number {
  const q = Math.round(fs * 2) / 2;
  return Math.round(Math.round(q * 1.73) * 0.7);
}

function atlasBanner(b: BannerSpec, fs: number, dpr: number): Sprite {
  const I = INK.atlas;
  const F = FONTS.atlas;
  const h = Math.round(fs * 1.73);
  const cap = 6; // end caps overhang
  const disp = (px: number) => `700 ${px}px ${F.display}`;
  // Measure.
  const [, m] = canvas(1, 1, 1);
  const nameW = smallCaps(m, b.name, 0, 0, disp, fs, fs * 0.06, false);
  const popS = b.pop === null ? 0 : Math.round(h * 0.69);
  const glyphW = b.glyph ? fs * 0.95 + 4 : 0;
  // An original capital ends in a star seal in place of the right swallowtail: a disc about 1.4 banners tall, so the star
  // reads at a glance. The banner is widened by the seal's radius so the name never runs under it.
  const R = b.capital ? capitalSealRadius(fs) : 0;
  const pad = b.capital ? Math.max(0, Math.ceil(R - (h + 1) / 2)) + 1 : 0;
  const inner = 4 + (popS ? 11 + popS + 8 : 0) + glyphW + nameW + 8 + R;
  const W = Math.ceil(inner);
  const totalW = cap + W + (b.capital ? R + 2 : cap);
  const [c, g] = canvas(totalW, h + 2 + 2 * pad, dpr);
  g.translate(0, pad);
  const x0 = cap;
  g.fillStyle = I.vellum;
  g.fillRect(x0, 0.5, W, h);
  // Diamond end caps.
  const d = h * 0.3;
  g.strokeStyle = b.selected ? I.red : I.ink;
  g.lineWidth = 1;
  for (const cx of b.capital ? [x0] : [x0, x0 + W]) {
    g.beginPath();
    g.moveTo(cx - d, 0.5 + h / 2);
    g.lineTo(cx, 0.5 + h / 2 - d);
    g.lineTo(cx + d, 0.5 + h / 2);
    g.lineTo(cx, 0.5 + h / 2 + d);
    g.closePath();
    g.fillStyle = b.selected ? I.red : I.vellum;
    g.fill();
    g.stroke();
  }
  g.fillStyle = I.vellum;
  g.fillRect(x0 + 0.5, 1, W - 1, h - 1);
  g.strokeStyle = b.selected ? I.red : I.ink;
  g.lineWidth = b.selected ? 1.6 : 1;
  g.strokeRect(x0 + 0.5, 1, W - 1, h - 1);
  g.strokeStyle = I.rule;
  g.lineWidth = 0.6;
  g.strokeRect(x0 + 2.8, 3.3, W - 5.6, h - 5.6);
  // The three heads label population without another bare number. Removing the seat badge
  // leaves room for this mark even at the smallest named zoom.
  let cx = x0 + 4;
  if (popS) {
    const mid = 0.5 + h / 2;
    g.fillStyle = I.ink2;
    for (const [dx, dy, r] of [[5, -3, 1.6], [2.5, -1, 1.2], [7.5, -1, 1.2]] as const) {
      g.beginPath(); g.arc(cx + dx, mid + dy, r, 0, Math.PI * 2); g.fill();
    }
    g.fillRect(cx + 1.5, mid + 1, 7, 2.5);
    cx += 11;
    const py = 0.5 + (h - popS) / 2;
    g.fillStyle = mapPlayerTint(b.owner, 0.3);
    g.fillRect(cx, py, popS, popS);
    g.strokeStyle = I.ink;
    g.lineWidth = 1;
    g.strokeRect(cx + 0.5, py + 0.5, popS - 1, popS - 1);
    g.fillStyle = I.ink;
    g.font = `700 ${Math.round(fs * 0.74)}px ${F.sans}`;
    g.textAlign = 'center';
    g.textBaseline = 'middle';
    g.fillText(String(b.pop), cx + popS / 2, py + popS / 2 + 0.5);
    g.textAlign = 'left';
    g.textBaseline = 'alphabetic';
    cx += popS + 8;
  }
  const base = 0.5 + h / 2 + fs * 0.32;
  if (b.glyph) {
    const gw = fs * 0.95;
    const ic = tintedIcon(b.glyph, I.ink2, gw * dpr);
    if (ic) g.drawImage(ic, cx, 0.5 + (h - gw) / 2, gw, gw);
    cx += gw + 4;
  }
  g.fillStyle = I.ink;
  smallCaps(g, b.name, cx, base, disp, fs, fs * 0.06);
  if (b.capital) {
    // The seal: vellum disc, ink rule, a fine gilt ring and the founder's star (outlined once you have scored it).
    const sx = x0 + W, sy = 0.5 + h / 2;
    g.beginPath(); g.arc(sx, sy, R, 0, Math.PI * 2);
    g.fillStyle = I.vellum; g.fill();
    g.strokeStyle = b.selected ? I.red : I.ink; g.lineWidth = b.selected ? 1.6 : 1.2; g.stroke();
    g.beginPath(); g.arc(sx, sy, R - 2.8, 0, Math.PI * 2);
    g.strokeStyle = CAPITAL_GILT; g.lineWidth = 1.3; g.stroke();
    drawCapitalStar(g, b.capital, sx, sy, R * 0.62, I.vellum, 0.32);
  }
  return { c, ax: totalW / 2, ay: pad + h + 1, w: totalW, h: h + 2 + 2 * pad };
}

// ---- lettering ----

/** Draw lettering and return its rotated sprite bounds in CSS screen pixels (`draw = false` only measures). */
export function drawLabel(
  ctx: CanvasRenderingContext2D,
  look: LookId,
  kind: 'sea' | 'land' | 'range',
  text: string,
  color: string,
  x: number,
  y: number,
  angle: number,
  scale: number,
  dpr: number,
  draw = true,
): Rect {
  const F = FONTS[look];
  const px = Math.round((kind === 'sea' ? 20 : 13) * scale);
  const sp = cached(`lab|${look}|${kind}|${text}|${color}|${px}|${dpr}`, () => {
    const font = kind === 'sea' ? `italic ${500} ${px}px ${F.display}` : `600 ${px}px ${F.sans}`;
    const track = kind === 'sea' ? px * (0.3) : px * 0.6;
    const [, m] = canvas(1, 1, 1);
    m.font = font;
    const w = spaced(m, text, 0, 0, track, false);
    const h = px * 1.6;
    const [c, g] = canvas(w + 8, h, dpr);
    g.font = font;
    g.fillStyle = color;
    spaced(g, text, 4, px * 1.12, track);
    return { c, ax: (w + 8) / 2, ay: h / 2, w: w + 8, h };
  });
  if (draw) {
    ctx.save();
    ctx.translate(x, y);
    if (angle) ctx.rotate(angle);
    ctx.drawImage(sp.c, -sp.ax, -sp.ay, sp.w, sp.h);
    ctx.restore();
  }
  const cos = Math.abs(Math.cos(angle));
  const sin = Math.abs(Math.sin(angle));
  const w = sp.w * cos + sp.h * sin;
  const h = sp.w * sin + sp.h * cos;
  return { x: x - w / 2, y: y - h / 2, w, h };
}

export const TOKEN_INK = INK;
