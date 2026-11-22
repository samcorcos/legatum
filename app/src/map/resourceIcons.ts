// Known map resources: a painted medallion at the top of the hex on every lens
// (beside the token where a city banner covers the top), under units, banners and routes. The painting says which resource; the rim says the category
// (bonus: a paper ring; strategic: an iron-red double rule). The view already hides strategic
// resources the viewer hasn't revealed, so this draws exactly what the tiles list.
import type { LookId } from '../brand/look';
import { MAP_STYLES } from './style';
import { resourceArtSprite, resourceArtUrl } from '../icons';
import { isDormant } from '../improvements';
import { improvementDetail } from './improvementArt';
import { isStrategic, lackedResources } from '../resources';
import { tileIndex } from '../hex';
import type { PlayerView } from '../types';
import { CITY_UNIT_DROP, STACK_LIFT, centerX, centerY, HEX_SIZE, tokenHalf } from './grid';
import type { Camera } from './render';

/** Medallions from a comfortable zoom; zoomed out, every known resource keeps a small dot in its category colour (green bonus, red strategic) rather than vanishing. */
export function resourceDetail(hexPixels: number): 'medallions' | 'dots' {
  return hexPixels >= 18 ? 'medallions' : 'dots';
}

/** Clearance between a medallion and the token or banner next to it, in screen pixels. */
export const GAP = 1.5;
/** The medallion may rise this far (× hex radius) above the centre: just past the top vertex, where the hex is narrow and no neighbour draws. */
const CEILING = 1.07;

export interface Slot { dx: number; dy: number; radius: number }
/** Screen rect (CSS px), e.g. a city banner, that medallions keep clear of. */
export interface Cover { x: number; y: number; w: number; h: number }

/**
 * Where a tile's medallion may sit, relative to the hex centre in screen pixels (down is +).
 * `top` is the usual place: top-centre, above a unit token, rising just past the top vertex.
 * `side` is for when a city banner covers the top (the city's own tile, and its row neighbours
 * under a long name): left of the token, level with where a unit stands (dropped on a city tile).
 */
export function medallionSlots(hexPixels: number, city: boolean): { top: Slot; side: Slot; stacked: Slot } {
  const r = hexPixels, tw = tokenHalf(r);
  const full = Math.min(15, Math.max(5, r * 0.29));
  const radius = Math.max(3, Math.min(full, (r * CEILING - tw - GAP) / 2));
  const top = { dx: 0, dy: -Math.max(r * 0.62, tw + GAP + radius), radius };
  const sideR = Math.max(3, Math.min(radius, (r * 0.95 - tw - GAP) / 2));
  const side = { dx: -(tw + GAP + sideR), dy: city ? r * CITY_UNIT_DROP : 0, radius: sideR };
  // A two-unit stack draws its first token STACK_LIFT higher and just right of centre, so its owner
  // roundel (radius 7.5 at the token's top-left corner) sits under a centred medallion. The medallion
  // rises above that roundel into the notch between the two upper neighbours, whose tokens start
  // about 0.43 r to either side, so nothing else draws there.
  const lift = city ? 0 : tw * STACK_LIFT;
  const clear = Math.max(tw + lift, 0.85 * tw + lift + 7.5) + GAP;
  const stackedR = Math.max(3, Math.min(radius, (r * STACK_CEILING - clear) / 2));
  const stacked = city ? top : { dx: 0, dy: -Math.max(r * 0.62, clear + stackedR), radius: stackedR };
  return { top, side, stacked };
}

/** A stacked tile's medallion may rise this far (× hex radius), into the notch above the top vertex. */
const STACK_CEILING = 1.3;

/** Distance from the hex centre up to a medallion's top edge (screen px), for labels that must sit above it. */
export function medallionTop(hexPixels: number, city: boolean, stacked: boolean): number {
  const slots = medallionSlots(hexPixels, city);
  const s = stacked ? slots.stacked : slots.top;
  return -(s.dy - s.radius);
}

export { STACK_LIFT };

const hits = (x: number, y: number, s: Slot, covers: readonly Cover[]) => covers.some((c) =>
  x + s.dx + s.radius + GAP > c.x && x + s.dx - s.radius - GAP < c.x + c.w && y + s.dy + s.radius + GAP > c.y && y + s.dy - s.radius - GAP < c.y + c.h);

/** The top slot unless a banner covers it and the side slot is clear. */
export function pickSlot(x: number, y: number, slots: { top: Slot; side: Slot }, covers: readonly Cover[]): Slot {
  return hits(x, y, slots.top, covers) && !hits(x, y, slots.side, covers) ? slots.side : slots.top;
}

// These inks remain in use for the far-tier marks and improvement outlines. The normal marker is
// the full-colour painted medallion in assets/resources/.
const INK: Record<LookId, { edge: string; face: string; paper: string; red: string; shadow: string }> = {
  atlas: { edge: '#241d15', face: '#3a3025', paper: '#f1e6c9', red: '#b3392b', shadow: 'rgba(40,28,12,.4)' },

};
/** Category colours shared with the Resources overlay and its legend. */
const CATEGORY_COLOR = { Bonus: MAP_STYLES.atlas.resources.bonus, Strategic: MAP_STYLES.atlas.resources.strategic } as const;

/**
 * Far-tier marks (user, 2026-10-11, "Coal and iron look way too similar"): resources differ
 * by silhouette and value, not only their category ring. Bonus Copper is the orange diamond (a coin), Iron the red square (an ingot),
 * Niter the pale flat-topped hexagon (a salt crystal), Coal the near-black disc ringed in the same red, Oil the dark teal droplet,
 * Silicon the cool-blue crystal. None is a triangle: those are the combat chips' stronger and weaker
 * marks. Bonus resources stay the green disc.
 */
export const COAL_MARK = '#1d2433';
const FAR_MARK_FILL: Record<string, string> = { Copper: '#d0803b', Niter: '#efe7d2', Oil: '#12413f', Silicon: '#a5c4d3' };

function polygon(ctx: CanvasRenderingContext2D, pts: [number, number][]) {
  ctx.moveTo(pts[0][0], pts[0][1]);
  for (const [px, py] of pts.slice(1)) ctx.lineTo(px, py);
  ctx.closePath();
}

/** The far-tier mark of one resource: a small shape in the category colour. */
export function drawFarMark(ctx: CanvasRenderingContext2D, x: number, y: number, dot: number, res: string, look: LookId) {
  ctx.beginPath();
  if (res === 'Iron') {
    const h = dot * 0.92;
    ctx.rect(x - h, y - h, 2 * h, 2 * h);
    ctx.fillStyle = CATEGORY_COLOR.Strategic; ctx.fill();
    ctx.lineWidth = 1; ctx.strokeStyle = INK[look].paper; ctx.stroke();
  } else if (res === 'Coal') {
    ctx.arc(x, y, dot, 0, Math.PI * 2);
    ctx.fillStyle = COAL_MARK; ctx.fill();
    ctx.lineWidth = Math.max(1.4, dot * 0.45); ctx.strokeStyle = CATEGORY_COLOR.Strategic; ctx.stroke();
    ctx.beginPath(); ctx.arc(x, y, dot + ctx.lineWidth / 2 + 0.5, 0, Math.PI * 2);
    ctx.lineWidth = 1; ctx.strokeStyle = INK[look].paper; ctx.stroke();
  } else if (res === 'Copper') {
    const h = dot * 1.25;
    polygon(ctx, [[x, y - h], [x + h, y], [x, y + h], [x - h, y]]);
    ctx.fillStyle = FAR_MARK_FILL.Copper; ctx.fill();
    ctx.lineWidth = 1.2; ctx.strokeStyle = CATEGORY_COLOR.Bonus; ctx.stroke();
  } else if (res === 'Niter') {
    const h = dot * 1.25;
    polygon(ctx, [0, 1, 2, 3, 4, 5].map((k) => [x + h * Math.cos(k * Math.PI / 3), y + h * Math.sin(k * Math.PI / 3)] as [number, number]));
    ctx.fillStyle = FAR_MARK_FILL.Niter; ctx.fill();
    ctx.lineWidth = 1.4; ctx.strokeStyle = CATEGORY_COLOR.Strategic; ctx.stroke();
  } else if (res === 'Oil') {
    // A droplet: a round belly and a point above it.
    const d = dot * 1.05;
    ctx.moveTo(x, y - d * 1.45);
    ctx.quadraticCurveTo(x + d * 0.45, y - d * 0.55, x + d, y + d * 0.3);
    ctx.arc(x, y + d * 0.3, d, 0, Math.PI);
    ctx.quadraticCurveTo(x - d * 0.45, y - d * 0.55, x, y - d * 1.45);
    ctx.closePath();
    ctx.fillStyle = FAR_MARK_FILL.Oil; ctx.fill();
    ctx.lineWidth = 1.4; ctx.strokeStyle = CATEGORY_COLOR.Strategic; ctx.stroke();
  } else if (res === 'Silicon') {
    const h = dot * 1.25, t = dot * 0.48;
    polygon(ctx, [[x, y - h], [x + t, y - h * 0.58], [x + t, y + h * 0.62], [x, y + h], [x - t, y + h * 0.62], [x - t, y - h * 0.58]]);
    ctx.fillStyle = FAR_MARK_FILL.Silicon; ctx.fill();
    ctx.lineWidth = 1; ctx.strokeStyle = INK[look].paper; ctx.stroke();
  } else {
    ctx.arc(x, y, dot, 0, Math.PI * 2);
    ctx.fillStyle = CATEGORY_COLOR.Bonus; ctx.fill();
    ctx.lineWidth = 1; ctx.strokeStyle = INK[look].paper; ctx.stroke();
  }
}

/** Zoomed out, a dormant resource's marker fades further than at normal zoom, where the land art is the cue. */
const FAR_DORMANT_FADE = 0.55;

/**
 * The improvement badge and its opposite, on a resource's own marker: a solid ink ring means improved
 * (drawn where the land art is too small to show: zoomed out), a dashed one dormant. The two read as a pair.
 */
function markImprovement(ctx: CanvasRenderingContext2D, x: number, y: number, radius: number, dormant: boolean, improved: boolean, visible: boolean, look: LookId) {
  if (!dormant && !improved) return;
  ctx.globalAlpha = visible ? 0.9 : 0.6;
  ctx.beginPath();
  ctx.arc(x, y, radius, 0, Math.PI * 2);
  ctx.strokeStyle = INK[look].edge;
  ctx.lineWidth = improved ? 1.8 : 1;
  // Far tier: a longer dash with wider gaps, so a dashed (dormant) ring never reads as a solid (improved) one.
  ctx.setLineDash(dormant ? [3.5, 3.5] : []);
  ctx.stroke();
  ctx.setLineDash([]);
}

/** Whose land a resource is in, as the Resources overlay rings it: a thick solid ring (yours), a thin one (unclaimed), a double ring (foreign). Shape carries it; the ring's colour is the resource's kind, and the dashed dark ring stays "not improved yet". */
export function emphasisRing(owner: number | null, me: number): { width: number; double: boolean; owned: 'yours' | 'unclaimed' | 'foreign' } {
  return owner === null ? { width: 1.8, double: false, owned: 'unclaimed' } : owner === me ? { width: 2.8, double: false, owned: 'yours' } : { width: 1.6, double: true, owned: 'foreign' };
}

/** The Resources overlay's "you lack this" badge: a red disc with a white exclamation mark at a medallion's upper right, drawn only on deposits someone else holds or nobody does, of a strategic resource your supply is short of. */
function drawWanted(ctx: CanvasRenderingContext2D, x: number, y: number, radius: number, look: LookId) {
  const r = Math.max(3.4, radius * 0.5);
  const cx = x + radius * 0.8, cy = y - radius * 0.8;
  ctx.beginPath(); ctx.arc(cx, cy, r + 1, 0, Math.PI * 2); ctx.fillStyle = INK[look].paper; ctx.fill();
  ctx.beginPath(); ctx.arc(cx, cy, r, 0, Math.PI * 2); ctx.fillStyle = INK[look].red; ctx.fill();
  ctx.fillStyle = INK[look].paper;
  const w = Math.max(1, r * 0.3);
  ctx.fillRect(cx - w / 2, cy - r * 0.6, w, r * 0.75);
  ctx.beginPath(); ctx.arc(cx, cy + r * 0.5, w * 0.7, 0, Math.PI * 2); ctx.fill();
}

function drawEmphasis(ctx: CanvasRenderingContext2D, x: number, y: number, radius: number, strategic: boolean, owner: number | null, me: number, look: LookId) {
  const ring = emphasisRing(owner, me);
  const color = strategic ? CATEGORY_COLOR.Strategic : CATEGORY_COLOR.Bonus;
  // A paper edge outside the ring keeps it readable on dark woods and strong lens tints.
  const reach = radius + 1.5 + (ring.double ? ring.width * 2 + 1.4 : ring.width);
  ctx.beginPath(); ctx.arc(x, y, reach + 0.7, 0, Math.PI * 2);
  ctx.strokeStyle = INK[look].paper; ctx.globalAlpha *= 0.85; ctx.lineWidth = 1.4; ctx.stroke(); ctx.globalAlpha /= 0.85;
  ctx.strokeStyle = color; ctx.lineWidth = ring.width;
  ctx.beginPath(); ctx.arc(x, y, radius + 1.5 + ring.width / 2, 0, Math.PI * 2); ctx.stroke();
  if (ring.double) { ctx.beginPath(); ctx.arc(x, y, radius + 1.5 + ring.width * 1.5 + 1.4, 0, Math.PI * 2); ctx.stroke(); }
}

/** Draw every known resource. Runs before units, banners and routes so they stay on top. `emphasis`: the Resources overlay rings each one. */
export function drawResourceIcons(ctx: CanvasRenderingContext2D, view: PlayerView, cam: Camera, width: number, height: number, dpr: number, look: LookId, covers: readonly Cover[] = [], emphasis = false) {
  const r = HEX_SIZE * cam.zoom;
  const detail = resourceDetail(r);
  const art = improvementDetail(r) === 'art';
  const W = view.width;
  const cities = new Set(view.cities.map((c) => tileIndex(c.pos, W, view.height)));
  const counts = new Map<number, number>();
  for (const u of view.units) { const k = tileIndex(u.pos, W, view.height); counts.set(k, (counts.get(k) ?? 0) + 1); }
  const open = medallionSlots(r, false), town = medallionSlots(r, true);
  const dot = Math.max(2.5, r * 0.15);
  const lacked = emphasis ? lackedResources(view.supply) : null;
  const wanted = (res: string, owner: number | null) => !!lacked?.has(res as never) && owner !== view.me;
  const myFaction = view.players.find((p) => p.id === view.me)?.faction;
  ctx.save();
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  for (let i = 0; i < view.tiles.length; i++) {
    const t = view.tiles[i];
    const res = t.resource;
    if (!res || !t.explored) continue;
    const strategic = isStrategic(res);
    const x = (centerX(i, W) - cam.x) * cam.zoom + width / 2;
    const cy = (centerY(i, W) - cam.y) * cam.zoom + height / 2;
    if (x < -r || x > width + r || cy < -r * 1.5 || cy > height + r) continue;
    ctx.globalAlpha = t.visible ? 1 : 0.75;
    const dormant = isDormant(t, view.me, myFaction);
    const ringed = !!t.improvement && !art;
    if (detail === 'dots') {
      if (dormant) ctx.globalAlpha *= FAR_DORMANT_FADE;
      const d = emphasis ? dot * 1.3 : dot;
      if (emphasis) drawEmphasis(ctx, x, cy - r * 0.62, d, strategic, t.owner, view.me, look);
      drawFarMark(ctx, x, cy - r * 0.62, d, res, look);
      markImprovement(ctx, x, cy - r * 0.62, d + (emphasis ? 5 : 1.5), dormant, ringed, t.visible, look);
      if (wanted(res, t.owner)) drawWanted(ctx, x, cy - r * 0.62, d * 1.6, look);
      continue;
    }
    const slots = cities.has(i) ? town : open;
    const p = pickSlot(x, cy, (counts.get(i) ?? 0) > 1 ? { top: slots.stacked, side: slots.side } : slots, covers);
    const s = resourceArtSprite(res, 2 * p.radius * dpr);
    if (!s) {
      // No painted medallion (still loading, or none made for this resource): its far-tier mark stands in, so the deposit is never invisible.
      if (resourceArtUrl(res, 64)) continue; // The map is redrawn when the image loads.
      if (dormant) ctx.globalAlpha *= FAR_DORMANT_FADE;
      if (emphasis) drawEmphasis(ctx, x + p.dx, cy + p.dy, p.radius * 1.15, strategic, t.owner, view.me, look);
      drawFarMark(ctx, x + p.dx, cy + p.dy, p.radius * 0.85, res, look);
      markImprovement(ctx, x + p.dx, cy + p.dy, p.radius + (emphasis ? 8.2 : 2), dormant, ringed, t.visible, look);
      if (wanted(res, t.owner)) drawWanted(ctx, x + p.dx, cy + p.dy, p.radius * 1.3, look);
      continue;
    }
    const size = s.width / dpr;
    // Dormant: your own unimproved resource yields nothing yet, so its medallion is faded and ringed with a dashed pen.
    if (dormant) ctx.globalAlpha *= art ? 0.62 : FAR_DORMANT_FADE;
    if (emphasis) {
      ctx.beginPath(); ctx.arc(x + p.dx, cy + p.dy, p.radius + 2, 0, Math.PI * 2);
      ctx.fillStyle = INK[look].paper; ctx.globalAlpha *= 0.8; ctx.fill(); ctx.globalAlpha /= 0.8;
    }
    ctx.drawImage(s, x + p.dx - size / 2, cy + p.dy - size / 2, size, size);
    if (emphasis) drawEmphasis(ctx, x + p.dx, cy + p.dy, p.radius, strategic, t.owner, view.me, look);
    // Keep the map's strategic-resource cue visible over the painted brass frame.
    if (strategic) {
      ctx.beginPath();
      ctx.arc(x + p.dx, cy + p.dy, Math.max(1, p.radius - 0.65), 0, Math.PI * 2);
      ctx.strokeStyle = INK[look].red;
      ctx.lineWidth = Math.max(0.8, 0.8 * dpr);
      ctx.stroke();
    }
    markImprovement(ctx, x + p.dx, cy + p.dy, p.radius + (emphasis ? 8.2 : 2), dormant, ringed, t.visible, look);
    if (wanted(res, t.owner)) drawWanted(ctx, x + p.dx, cy + p.dy, p.radius * 1.3, look);
  }
  ctx.restore();
}
