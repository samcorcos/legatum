// Drawing the payoff moments (momentFx.ts) on the map canvas, in screen space. The look is the atlas's (legatum-art):
// new towns and the painted world wonders use their own incomplete stages; ordinary buildings settle on their banners;
// a claimed tile fills with its owner's colour from the middle out. Gilt is for fine VP and world wonder rules. Everything is
// strokes, fills, clipped drawImage calls of art already on the map, and a one-off engraving of each sprite (`inkOf`),
// so a turn with many moments stays cheap; nothing here uses filters, shadowBlur or per-frame canvases.
import type { Faction, PlayerView } from '../types';
import { cityImage, cityStem, cityTier, markStem, spriteSize, CITY_CANVAS, CITY_GROUND } from './cityArt';
import { CITY_DROP, CITY_WORLD } from './terrain';
import { buildingIconName, itemName } from '../data';
import { tintedIcon } from '../icons';
import { ATLAS } from './style';
import { ring } from './combatFxDraw';
import { FONTS } from './sprites';
import { landmarkBox, type LandmarkBox } from './landmarks';
import { drawSite, drawWallsSite, stageImage } from './constructionFx';
import { constructionStage, type ConstructionSite } from './constructionSites';
import { rnd, type Pt } from './combatFx';
import { resourceArtUrl } from '../icons';
import { medallionSlots } from './resourceIcons';
import { CONSTRUCTION_ANIMATION, FOUND, GLINT_MS, LINGER, GROW_MS, IMPROVE_MS, RESOURCE_REVEAL_MS, ROAD_MS, SEAL_MS, TILE_MS, buildPhases, worldOf, type FxBuild, type FxFound, type FxGlint, type FxGrow, type FxSeal, type FxTile, type MomentPlan } from './momentFx';

const INK = ATLAS.tone.ink;
const VELLUM = ATLAS.tone.vellum;
const GOLD = ATLAS.tone.gold;

export interface MomentView {
  toScreen(p: Pt): Pt;
  /** Hex radius on screen (CSS px), and CSS px per world px. */
  hexPx: number;
  zoom: number;
  dpr: number;
  far: boolean;
  w: number;
  h: number;
  colorOf(owner: number): string;
  /** An empire's era, 0-based (a town is painted in its owner's era). */
  eraOf(owner: number): number;
  factionOf(owner: number): Faction | undefined;
  /** Where a unit is drawn (world px, after any march or playback), and the counter's half size on screen. */
  unitAt(id: number): Pt | null;
  tokenHalf: number;
  /** Where a city's world wonder stands as a landmark (landmarks.ts), once built. */
  landmarkAt(city: number, building: string): Pt | null;
  landmarkBoxes?: ReadonlyMap<number, LandmarkBox>;
  /** A city's population now, and its owner. */
  city(id: number): { population: number; owner: number } | null;
}

const clamp01 = (k: number) => Math.max(0, Math.min(1, k));
const easeOut = (k: number) => 1 - (1 - k) * (1 - k);
const easeInOut = (k: number) => (k < 0.5 ? 2 * k * k : 1 - Math.pow(-2 * k + 2, 2) / 2);

// ---- art ----

const art = new Map<string, HTMLImageElement | null>();
/** A painted unlock miniature, loaded once (null while it decodes or if the file is missing). */
function unlockImage(url: string | null): HTMLImageElement | null {
  if (!url || typeof Image === 'undefined') return null;
  let img = art.get(url);
  if (img === undefined) {
    img = new Image();
    img.src = url;
    art.set(url, img);
  }
  return img && img.complete && img.naturalWidth > 0 ? img : null;
}

const inks = new Map<string, HTMLCanvasElement | null>();
/**
 * The sprite as an engraving: its outlines and the edges inside it as sepia pen lines, its shadows as diagonal hatching,
 * on a transparent ground. Made once per sprite from its own pixels (a Sobel pass on the luminance over vellum).
 */
export function inkOf(img: HTMLImageElement | HTMLCanvasElement, key: string): HTMLCanvasElement | null {
  if (inks.has(key)) return inks.get(key)!;
  let out: HTMLCanvasElement | null = null;
  try {
    // At most 256 px: a 512 master would cost a visible hitch the first time, and the lines read the same.
    const iw = 'naturalWidth' in img ? img.naturalWidth : img.width;
    const ih = 'naturalHeight' in img ? img.naturalHeight : img.height;
    const f = Math.min(1, 256 / Math.max(iw, ih));
    const w = Math.round(iw * f), h = Math.round(ih * f);
    const c = document.createElement('canvas');
    c.width = w; c.height = h;
    const g = c.getContext('2d', { willReadFrequently: true });
    if (!g || !w || !h) throw new Error('no canvas');
    g.drawImage(img, 0, 0, w, h);
    const src = g.getImageData(0, 0, w, h);
    const d = src.data;
    const L = new Float32Array(w * h), A = new Float32Array(w * h);
    for (let i = 0; i < w * h; i++) {
      const a = d[i * 4 + 3] / 255;
      const lum = (0.3 * d[i * 4] + 0.59 * d[i * 4 + 1] + 0.11 * d[i * 4 + 2]) / 255;
      A[i] = a;
      L[i] = a * lum + (1 - a) * 0.93;
    }
    const outData = g.createImageData(w, h);
    const o = outData.data;
    const [ir, ig, ib] = [0x4a, 0x36, 0x22];
    // Line weight scales with the sprite: a 512 master needs a wider kernel step than a 64.
    const step = Math.max(1, Math.round(w / 128));
    const hatch = Math.max(3, Math.round(w / 32));
    for (let y = step; y < h - step; y++) {
      for (let x = step; x < w - step; x++) {
        const i = y * w + x;
        const l = (dx: number, dy: number) => L[(y + dy * step) * w + x + dx * step];
        const gx = -l(-1, -1) - 2 * l(-1, 0) - l(-1, 1) + l(1, -1) + 2 * l(1, 0) + l(1, 1);
        const gy = -l(-1, -1) - 2 * l(0, -1) - l(1, -1) + l(-1, 1) + 2 * l(0, 1) + l(1, 1);
        // Pen lines where the paint changes value sharply; light hatching only in the deepest shadows.
        const edge = clamp01((Math.hypot(gx, gy) - 0.3) / 0.6);
        const shade = A[i] * clamp01((0.42 - L[i]) / 0.42);
        const lines = (x + y) % hatch < Math.max(1, step) ? shade * 0.75 : 0;
        // The silhouette always gets its outline.
        const rim = A[i] > 0.5 && (A[i - step] < 0.5 || A[i + step] < 0.5 || A[i - step * w] < 0.5 || A[i + step * w] < 0.5) ? 0.9 : 0;
        const alpha = Math.max(edge * 0.85, lines, rim);
        o[i * 4] = ir; o[i * 4 + 1] = ig; o[i * 4 + 2] = ib; o[i * 4 + 3] = Math.round(alpha * 215);
      }
    }
    g.clearRect(0, 0, w, h);
    g.putImageData(outData, 0, 0);
    out = c;
  } catch { out = null; }
  inks.set(key, out);
  return out;
}

/**
 * Ink laid down blot by blot: the union of `n` round blots over a w x h box, each appearing in order of its distance
 * from `origin` (0..1 across the box) with some noise, then swelling to its size. k in [0, 1]; at 1 the box is covered.
 */
function blots(x: number, y: number, w: number, h: number, seed: number, k: number, origin: Pt, n = 30): Path2D {
  const path = new Path2D();
  const maxR = Math.max(w, h) * 0.34;
  for (let i = 0; i < n; i++) {
    const u = rnd(seed, i * 3), v = rnd(seed, i * 3 + 1);
    const px = x + u * w, py = y + v * h;
    const order = Math.hypot(u - origin.x, (v - origin.y) * 0.8) / 1.1 * 0.72 + rnd(seed, i * 3 + 2) * 0.28;
    const s = order * 0.7;
    const kk = clamp01((k - s) / 0.3);
    if (kk <= 0) continue;
    const r = maxR * (0.6 + 0.4 * rnd(seed, i + 101)) * easeOut(kk);
    path.moveTo(px + r, py);
    path.arc(px, py, r, 0, Math.PI * 2);
  }
  return path;
}

/** A sprite sketched in, then painted: the engraving laid down by `kInk`, the colour flooding in under it by `kWash`. */
function sketch(ctx: CanvasRenderingContext2D, img: HTMLImageElement, key: string, x: number, y: number, w: number, h: number, seed: number, kInk: number, kWash: number, inkAlpha = 1) {
  const ink = inkOf(img, key);
  const origin = { x: 0.5, y: 0.82 };
  if (kWash > 0) {
    ctx.save();
    if (kWash < 1) ctx.clip(blots(x, y, w, h, seed + 7, kWash, origin, 26));
    ctx.globalAlpha = 1;
    ctx.drawImage(img, x, y, w, h);
    ctx.restore();
  }
  if (!ink) {
    // No engraving (canvas readback unavailable): the paint alone fades in.
    if (kWash <= 0 && kInk > 0) { ctx.globalAlpha = kInk * 0.6; ctx.drawImage(img, x, y, w, h); ctx.globalAlpha = 1; }
    return;
  }
  // The lines stay over the paint for a beat, then sink into it.
  const lineAlpha = (kWash >= 1 ? 0 : 1 - 0.85 * easeInOut(kWash)) * inkAlpha;
  if (kInk <= 0 || lineAlpha <= 0) return;
  ctx.save();
  if (kInk < 1) ctx.clip(blots(x, y, w, h, seed, kInk, origin));
  ctx.globalAlpha = lineAlpha;
  ctx.drawImage(ink, x, y, w, h);
  ctx.restore();
}

/** Where the terrain cache draws a town's sprite on screen (cityArt geometry), for a hex centre on screen. */
function townBox(c: Pt, zoom: number) {
  const k = CITY_WORLD / CITY_CANVAS;
  return { x: c.x - (CITY_GROUND.x0 + CITY_GROUND.x1) / 2 * k * zoom, y: c.y + (-CITY_GROUND.middle * k + CITY_DROP) * zoom, s: CITY_WORLD * zoom };
}

function townSprite(era: number, pop: number, far: boolean, px: number): { img: HTMLImageElement; key: string } | null {
  const stem = far ? markStem(cityTier(pop)) : cityStem(era, cityTier(pop));
  if (!stem) return null;
  const size = spriteSize(px);
  const img = cityImage(stem, size);
  return img ? { img, key: `${stem}@${size}` } : null;
}

// ---- moments ----

function drawFound(ctx: CanvasRenderingContext2D, f: FxFound, t: number, v: MomentView) {
  const dt = t - f.t;
  if (dt < 0 || dt >= FOUND.done + LINGER) return;
  const box = townBox(v.toScreen(worldOf(f.at)), v.zoom);
  const pop = v.city(f.city)?.population ?? 1;
  const stem = cityStem(v.eraOf(f.owner), cityTier(pop));
  if (!stem) return;
  const progress = clamp01(dt / FOUND.done);
  const phase = progress < .22 ? 1 : progress < .48 ? 2 : progress < .76 ? 3 : 4;
  const image = phase < 4 ? stageImage(stem, phase, box.s * v.dpr) : townSprite(v.eraOf(f.owner), pop, v.far, box.s * v.dpr)?.img;
  if (image) ctx.drawImage(image, box.x, box.y, box.s, box.s);
}

/** One ink ring settles on the claimed hex while the town is painted in. Drawn over the units. */
function drawFoundMarks(ctx: CanvasRenderingContext2D, f: FxFound, t: number, v: MomentView) {
  const dt = t - f.t;
  if (dt < 0 || dt > 1700) return;
  const c = v.toScreen(worldOf(f.at));
  const R = v.hexPx;
  ring(ctx, t, f.t, c.x, c.y + R * 0.15, R * 0.3, R * 1.05, 1100, INK, 1);
}

/** A tile taking its new owner's colour: the colour flows in from the side facing the city, then the border takes over. */
function drawTile(ctx: CanvasRenderingContext2D, x: FxTile, t: number, v: MomentView) {
  const dt = t - x.t;
  const lead = 220;
  if (dt < -lead || dt > TILE_MS || x.to == null) return;
  const c = v.toScreen(worldOf(x.hex));
  const o = v.toScreen(worldOf(x.origin));
  const R = v.hexPx;
  const d = Math.hypot(o.x - c.x, o.y - c.y);
  // Anchor on the hex's edge toward the city (its middle for the city's own tile).
  const ax = d > 1 ? c.x + (o.x - c.x) / d * R * 0.86 : c.x, ay = d > 1 ? c.y + (o.y - c.y) / d * R * 0.86 : c.y;
  const grow = easeOut(clamp01((dt + lead) / (lead + 140)));
  const fade = dt <= 60 ? 1 : 1 - easeInOut(clamp01((dt - 60) / (TILE_MS - 60)));
  ctx.beginPath();
  for (let i = 0; i < 6; i++) {
    const a = (Math.PI / 3) * i - Math.PI / 6;
    const px = c.x + Math.cos(a) * R, py = c.y + Math.sin(a) * R;
    const qx = ax + (px - ax) * grow, qy = ay + (py - ay) * grow;
    if (i) ctx.lineTo(qx, qy); else ctx.moveTo(qx, qy);
  }
  ctx.closePath();
  ctx.globalAlpha = 0.3 * fade;
  ctx.fillStyle = v.colorOf(x.to);
  ctx.fill();
  ctx.globalAlpha = 1;
}

/** One short step for each turn of progress, ending with a restrained final lay-in. */
function drawBuild(ctx: CanvasRenderingContext2D, b: FxBuild, t: number, v: MomentView) {
  const dt = t - b.t, duration = buildPhases(b).done;
  if (dt < 0 || dt >= duration) return;
  const u = clamp01(dt / duration), k = b.from + (b.to - b.from) * u;
  const centre = v.toScreen(worldOf(b.at));
  if (!b.wonder && !b.walls) {
    // The meter and icon settle against the banner; no miniature is bolted onto the town.
    const width = Math.max(25, v.hexPx * 1.25), y = centre.y - v.hexPx * 1.38;
    ctx.save(); ctx.strokeStyle = INK; ctx.fillStyle = GOLD; ctx.lineWidth = 1;
    ctx.strokeRect(centre.x - width / 2, y, width, 3);
    ctx.fillRect(centre.x - width / 2, y, width * u, 3);
    const icon = b.item.kind === 'Building' ? tintedIcon(buildingIconName(b.item.building), INK, Math.ceil(18 * v.dpr)) : null;
    if (icon && u > .65) { ctx.globalAlpha = clamp01((u - .65) / .2); ctx.drawImage(icon, centre.x + width / 2 + 3, y - 7 + (1 - u) * 5, 16, 16); }
    ctx.restore();
    return;
  }
  const site: ConstructionSite = { city: b.city, owner: b.owner, tile: b.tile, hex: b.at,
    building: b.item.kind === 'Building' || b.item.kind === 'Wonder' ? b.item.building : 'Walls',
    era: v.eraOf(b.owner), k, stage: constructionStage(k), seed: b.seed, walls: b.walls };
  if (b.walls) {
    drawWallsSite(ctx, site, centre, v.zoom, v.dpr, k, v.factionOf(b.owner));
    return;
  }
  const box = v.landmarkBoxes?.get(b.tile) ?? landmarkBox(centre, v.hexPx);
  const crossed = constructionStage(b.from) !== constructionStage(b.to);
  if (!crossed) drawSite(ctx, site, box, k, v.factionOf(b.owner));
  else {
    // Lay the next actual painted stage upward across the working line over the old stage.
    drawSite(ctx, site, box, b.from, v.factionOf(b.owner));
    const reveal = clamp01((dt - (b.to >= 1 ? 0 : 300)) / 600);
    ctx.save(); ctx.beginPath(); ctx.rect(box.x - 2, box.y + box.s * (1 - reveal), box.s + 4, box.s * reveal + 2); ctx.clip();
    drawSite(ctx, site, box, b.to, v.factionOf(b.owner));
    ctx.restore();
  }
  if (b.to >= 1 && !v.far && dt > 1600) {
    const text = itemName(b.item, v.factionOf(b.owner)).toUpperCase();
    const alpha = clamp01((dt - 1600) / 500);
    ctx.save(); ctx.globalAlpha = alpha; ctx.fillStyle = INK; ctx.strokeStyle = GOLD; ctx.textAlign = 'center';
    ctx.font = `600 ${Math.max(9, Math.min(13, v.hexPx * .25))}px ${FONTS.atlas.display}`;
    const x = box.x + box.s / 2, y = box.y + box.s * .98;
    const half = Math.min(box.s * .36, ctx.measureText(text).width * .52);
    ctx.beginPath(); ctx.moveTo(x - half, y - 14); ctx.lineTo(x + half, y - 14); ctx.stroke(); ctx.fillText(text, x, y);
    ctx.restore();
  }
}

/** A town growing into its next size: the bigger town is drawn in over the old one, rising a little as it does. */
function drawGrow(ctx: CanvasRenderingContext2D, g: FxGrow, t: number, v: MomentView) {
  const dt = t - g.t;
  if (dt < 0 || dt >= GROW_MS + LINGER) return;
  const city = v.city(g.city);
  if (!city) return;
  const c = v.toScreen(worldOf(g.at));
  const box = townBox(c, v.zoom);
  const sprite = townSprite(v.eraOf(city.owner), city.population, v.far, box.s * v.dpr);
  if (!sprite) return;
  const k = Math.min(1, dt / GROW_MS);
  const s = 0.96 + 0.04 * easeOut(clamp01(k / 0.7));
  const bw = box.s * s, bx = c.x - (c.x - box.x) * s, by = box.y + box.s * (1 - s) * 0.85;
  sketch(ctx, sprite.img, sprite.key, bx, by, bw, bw, g.seed, clamp01(k / 0.55), clamp01((k - 0.25) / 0.7));
}

const WAX = '#8f2a1c';

/**
 * A seal over a capital, restrained (user, 2026-10-24: no sunbursts or cartoon pops). A VP is "+1 VP" set
 * small in the house serif over a fine gilt rule that draws out from its centre, rising a little and fading. An Alliance, a Peace or a war is a small wax seal pressed in quietly: a ring for Peace, two
 * linked rings for an Alliance, a cross for War.
 */
function drawSeal(ctx: CanvasRenderingContext2D, x: FxSeal, t: number, v: MomentView) {
  const dt = t - x.t;
  if (dt < 0 || dt > SEAL_MS) return;
  const c = v.toScreen(worldOf(x.at));
  const R = v.hexPx;
  const side = x.slot === 0 ? 0 : (x.slot % 2 ? -1 : 1) * Math.ceil(x.slot / 2);
  const inA = easeOut(clamp01(dt / 380));
  const out = clamp01((dt - (SEAL_MS - 500)) / 500);
  const alpha = inA * (1 - out);
  if (alpha <= 0) return;
  const lift = (1 - inA) * 5 + out * 4;
  const cx = c.x + side * R * 1.5, cy = c.y - (v.far ? R * 0.9 : R * 1.55) - lift;
  ctx.save();
  ctx.globalAlpha = alpha;
  if (x.kind === 'vp') {
    const size = Math.round(Math.max(16, Math.min(26, R * 0.6)));
    ctx.font = `600 ${size}px ${FONTS.atlas.display}`;
    ctx.textAlign = 'center'; ctx.textBaseline = 'alphabetic';
    const text = `+${x.vp} VP`;
    const tw = ctx.measureText(text).width;
    // A narrow vellum slip, like a label on an engraved map, so the figure reads over any terrain.
    const padX = size * 0.5, top = cy - size * 0.86, hgt = size * 1.12;
    ctx.globalAlpha = alpha * 0.9;
    ctx.fillStyle = VELLUM;
    ctx.fillRect(cx - tw / 2 - padX, top, tw + padX * 2, hgt);
    ctx.globalAlpha = alpha;
    ctx.strokeStyle = 'rgba(43,36,25,0.55)'; ctx.lineWidth = 0.75;
    ctx.strokeRect(cx - tw / 2 - padX + 0.5, top + 0.5, tw + padX * 2 - 1, hgt - 1);
    ctx.fillStyle = INK;
    ctx.fillText(text, cx, cy);
    // The gilt rule under the slip, drawn out from the middle, with a small diamond at its centre.
    const half = (tw / 2 + padX * 1.8) * easeInOut(clamp01((dt - 120) / 420));
    const ry = top + hgt + 3;
    ctx.strokeStyle = GOLD; ctx.lineWidth = 1;
    ctx.beginPath(); ctx.moveTo(cx - half, ry); ctx.lineTo(cx + half, ry); ctx.stroke();
    if (half > 3) {
      ctx.fillStyle = GOLD;
      ctx.beginPath(); ctx.moveTo(cx, ry - 2.6); ctx.lineTo(cx + 2.6, ry); ctx.lineTo(cx, ry + 2.6); ctx.lineTo(cx - 2.6, ry); ctx.closePath(); ctx.fill();
    }
  } else {
    const r = Math.max(8, R * 0.3);
    const press = 1.08 - 0.08 * inA;
    ctx.translate(cx, cy);
    ctx.scale(press, press);
    ctx.beginPath();
    for (let i = 0; i <= 18; i++) {
      const a = (i / 18) * Math.PI * 2;
      const rr = r * (1 + (rnd(x.seed, i % 18) - 0.5) * 0.1);
      if (i) ctx.lineTo(Math.cos(a) * rr, Math.sin(a) * rr); else ctx.moveTo(rr, 0);
    }
    ctx.closePath();
    ctx.fillStyle = WAX; ctx.fill();
    ctx.lineWidth = 0.8; ctx.strokeStyle = INK; ctx.stroke();
    ctx.strokeStyle = 'rgba(247,241,227,0.85)'; ctx.lineWidth = Math.max(1, r * 0.09);
    if (x.kind === 'peace') { ctx.beginPath(); ctx.arc(0, 0, r * 0.45, 0, Math.PI * 2); ctx.stroke(); }
    else if (x.kind === 'alliance') {
      ctx.beginPath(); ctx.arc(-r * 0.2, 0, r * 0.36, 0, Math.PI * 2); ctx.stroke();
      ctx.beginPath(); ctx.arc(r * 0.2, 0, r * 0.36, 0, Math.PI * 2); ctx.stroke();
    } else {
      ctx.beginPath(); ctx.moveTo(-r * 0.38, -r * 0.38); ctx.lineTo(r * 0.38, r * 0.38); ctx.moveTo(r * 0.38, -r * 0.38); ctx.lineTo(-r * 0.38, r * 0.38); ctx.stroke();
    }
  }
  ctx.restore();
  ctx.globalAlpha = 1;
}

/** A promotion or upgrade: a soft band of light passes once across the counter. */
function drawGlint(ctx: CanvasRenderingContext2D, g: FxGlint, t: number, v: MomentView) {
  const dt = t - g.t;
  if (dt < 0 || dt > GLINT_MS) return;
  const w = v.unitAt(g.unit);
  if (!w) return;
  const c = v.toScreen(w);
  const r = v.tokenHalf * 1.15;
  const k = dt / GLINT_MS;
  ctx.save();
  ctx.beginPath(); ctx.arc(c.x, c.y, r, 0, Math.PI * 2); ctx.clip();
  const sweep = -r * 1.6 + easeInOut(k) * r * 3.2;
  ctx.globalAlpha = 0.4 * Math.sin(Math.PI * k);
  ctx.fillStyle = '#fff6da';
  ctx.beginPath();
  ctx.moveTo(c.x + sweep - r * 0.25, c.y - r * 1.2); ctx.lineTo(c.x + sweep + r * 0.1, c.y - r * 1.2);
  ctx.lineTo(c.x + sweep - r * 0.4, c.y + r * 1.2); ctx.lineTo(c.x + sweep - r * 0.75, c.y + r * 1.2); ctx.closePath();
  ctx.fill();
  ctx.restore();
}

/** Under the units and banners: towns and walls being drawn, towns growing, land changing colour. */
export function drawMomentGround(ctx: CanvasRenderingContext2D, plan: MomentPlan, t: number, v: MomentView) {
  ctx.save();
  ctx.setTransform(v.dpr, 0, 0, v.dpr, 0, 0);
  ctx.imageSmoothingEnabled = true;
  ctx.imageSmoothingQuality = 'high';
  for (const x of plan.tiles) drawTile(ctx, x, t, v);
  if (CONSTRUCTION_ANIMATION) {
    for (const f of plan.founds) drawFound(ctx, f, t, v);
    for (const g of plan.grows) drawGrow(ctx, g, t, v);
    for (const b of plan.builds) if (b.wonder || b.walls) drawBuild(ctx, b, t, v);
  }
  ctx.restore();
}

/** Over the units and banners: the founding ring, seals and glints. */
export function drawMomentFx(ctx: CanvasRenderingContext2D, plan: MomentPlan, t: number, v: MomentView) {
  ctx.save();
  ctx.setTransform(v.dpr, 0, 0, v.dpr, 0, 0);
  ctx.imageSmoothingEnabled = true;
  ctx.imageSmoothingQuality = 'high';
  if (CONSTRUCTION_ANIMATION) for (const f of plan.founds) drawFoundMarks(ctx, f, t, v);
  if (CONSTRUCTION_ANIMATION) for (const b of plan.builds) if (!b.wonder && !b.walls) drawBuild(ctx, b, t, v);
  for (const reveal of plan.reveals) {
    const dt = t - reveal.t;
    if (dt < 0 || dt >= RESOURCE_REVEAL_MS) continue;
    const centre = v.toScreen(worldOf(reveal.hex));
    centre.y += medallionSlots(v.hexPx, false).top.dy;
    const size = Math.max(20, Math.min(48, v.hexPx * 0.8));
    const image = unlockImage(resourceArtUrl(reveal.resource, 64));
    if (!image) continue;
    const ink = inkOf(image, `resource-${reveal.resource}`);
    const k = clamp01(dt / (RESOURCE_REVEAL_MS * 0.7));
    ctx.save();
    ctx.beginPath();
    ctx.rect(centre.x - size / 2, centre.y - size / 2, size * easeOut(k), size);
    ctx.clip();
    ctx.globalAlpha = clamp01(dt / 180);
    if (ink) ctx.drawImage(ink, centre.x - size / 2, centre.y - size / 2, size, size);
    ctx.globalAlpha = clamp01((dt - 250) / 300);
    ctx.drawImage(image, centre.x - size / 2, centre.y - size / 2, size, size);
    ctx.restore();
  }
  for (const x of plan.seals) drawSeal(ctx, x, t, v);
  for (const g of plan.glints) drawGlint(ctx, g, t, v);
  ctx.restore();
}

/** Dust off a marching unit's heels: a few engraved-wash tufts along where it just was, fading behind it. */
export function drawMarchDust(ctx: CanvasRenderingContext2D, march: ReadonlyMap<number, { trail: Pt[] }>, toScreen: (p: Pt) => Pt, tw: number, zoom: number, dpr: number) {
  ctx.save();
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.lineWidth = Math.max(0.8, zoom * 0.5);
  for (const m of march.values()) {
    m.trail.forEach((w, i) => {
      const p = toScreen(w);
      const r = tw * (0.2 + i * 0.09);
      const y = p.y + tw * 0.78 - i * tw * 0.06;
      ctx.globalAlpha = 0.34 * (1 - i / 3.4);
      ctx.fillStyle = 'rgb(214,196,160)';
      ctx.beginPath(); ctx.ellipse(p.x, y, r * 1.3, r * 0.62, 0, 0, Math.PI * 2); ctx.fill();
      ctx.globalAlpha *= 0.8;
      ctx.strokeStyle = 'rgb(128,100,66)';
      ctx.beginPath(); ctx.ellipse(p.x, y, r * 1.3, r * 0.62, 0, Math.PI * 0.05, Math.PI * 0.95); ctx.stroke();
    });
  }
  ctx.restore();
}

// ---- roads and improvements drawing themselves in ----

const beforeViews = new WeakMap<MomentPlan, { view: PlayerView; roads: PlayerView | null; improves: PlayerView | null }>();
/** The view with this turn's new roads (or improvements) taken back out: what the land looked like before them. */
function viewsBefore(plan: MomentPlan, view: PlayerView) {
  const memo = beforeViews.get(plan);
  if (memo && memo.view === view) return memo;
  let roads: PlayerView | null = null, improves: PlayerView | null = null;
  if (plan.roads.length) {
    const tiles = view.tiles.slice();
    for (const r of plan.roads) for (const [i, bits] of r.tiles) tiles[i] = { ...tiles[i], road: bits };
    roads = { ...view, tiles };
  }
  if (plan.improves.length) {
    const tiles = view.tiles.slice();
    for (const m of plan.improves) tiles[m.tile] = { ...tiles[m.tile], improvement: m.before };
    improves = { ...view, tiles };
  }
  const out = { view, roads, improves };
  beforeViews.set(plan, out);
  return out;
}

/** Where the new state shows at `t` (screen CSS px): what the roads have reached, and how far each improvement is drawn. */
export interface Reveal { roads: { before: PlayerView; inside: Path2D } | null; improves: { before: PlayerView; inside: Path2D } | null }

export function revealAt(plan: MomentPlan, t: number, view: PlayerView, v: MomentView): Reveal | null {
  if (!plan.roads.length && !plan.improves.length) return null;
  const before = viewsBefore(plan, view);
  const R = v.hexPx;
  let roads: Reveal['roads'] = null, improves: Reveal['improves'] = null;
  if (before.roads) {
    const inside = new Path2D();
    for (const r of plan.roads) {
      const k = clamp01((t - r.t) / ROAD_MS);
      if (k <= 0) continue;
      const c = v.toScreen(worldOf(r.origin));
      // The road runs out from the city a hex at a time (a hex is about 1.73 R across).
      const rad = k >= 1 ? 1e5 : R * 0.4 + easeInOut(k) * (r.reach + 0.6) * R * 1.75;
      inside.moveTo(c.x + rad, c.y);
      inside.arc(c.x, c.y, rad, 0, Math.PI * 2);
    }
    roads = { before: before.roads, inside };
  }
  if (before.improves && CONSTRUCTION_ANIMATION) {
    const inside = new Path2D();
    for (const m of plan.improves) {
      const k = clamp01((t - m.t) / IMPROVE_MS);
      if (k <= 0) continue;
      const c = v.toScreen(worldOf(m.hex));
      // Drawn in from one side of the hex to the other, with a slanted front, like a plough working across a field.
      const x0 = c.x - R * 1.1, span = R * 2.6 * (k >= 1 ? 1.2 : easeInOut(k));
      inside.moveTo(x0, c.y - R * 1.2);
      inside.lineTo(x0 + span, c.y - R * 1.2);
      inside.lineTo(x0 + span - R * 0.4, c.y + R * 1.2);
      inside.lineTo(x0, c.y + R * 1.2);
      inside.closePath();
    }
    improves = { before: before.improves, inside };
  }
  return { roads, improves };
}

/** Draw `layer` from `before` outside the revealed area and from the current view inside it. */
export function drawRevealed(ctx: CanvasRenderingContext2D, dpr: number, w: number, h: number, part: { before: PlayerView; inside: Path2D }, view: PlayerView, layer: (v: PlayerView) => void) {
  ctx.save();
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  const outside = new Path2D();
  outside.rect(-10, -10, w + 20, h + 20);
  outside.addPath(part.inside);
  ctx.clip(outside, 'evenodd');
  layer(part.before);
  ctx.restore();
  ctx.save();
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.clip(part.inside);
  layer(view);
  ctx.restore();
}
