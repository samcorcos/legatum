// The Courier Hall on the map (Portolan rose, art decision 2026-10-07): a compass-rose plaque on the
// Hall's city with its four intercardinal points in the holder's seat tint, faint rhumb lines
// fanning out across the holder's land (stopping at its border: the open road), and, zoomed out, a
// gilt-ringed badge beside the city seal. Everything is ink on vellum in the map's pen weights; the
// tint is the holder's, so a captured Hall changes colour with its city. Every viewer who knows the
// city sees it, remembered cities included (faded), as `view.courier_halls` is public.
import type { CityView, Hex, PlayerView } from '../types';
import { tintedIcon } from '../icons';
import { ATLAS, mapPlayerColor } from './style';
import { CORNERS, isFar, sealRadius } from './grid';
import { hexAt, tileIndex, within } from '../hex';
import { rgba } from '../theme';

type Ctx = CanvasRenderingContext2D;
type Pt = { x: number; y: number };
type S2 = (h: Hex) => Pt;

const T = ATLAS.tone;
const P = ATLAS.pen;
const H = ATLAS.hall;

export interface HallSite { city: CityView; holder: number }

/** The Courier Halls the viewer knows of, with their cities: the public list joined to the cities in the view (a city never seen has no place to draw). */
export function hallSites(view: PlayerView): HallSite[] {
  const out: HallSite[] = [];
  for (const h of view.courier_halls ?? []) {
    const city = view.cities.find((c) => c.id === h.city);
    if (city) out.push({ city, holder: h.holder });
  }
  return out;
}

/** The holder's explored land within `reach` hexes of the city, as one clip path (rhumb lines stop at its border). */
function territory(view: PlayerView, holder: number, centre: Hex, s2: S2, hexPx: number, reach: number): Path2D {
  const path = new Path2D();
  for (const h of within(centre, reach)) {
    const i = tileIndex(h, view.width, view.height);
    if (i < 0) continue;
    const t = view.tiles[i];
    if (!t.explored || t.owner !== holder) continue;
    const p = s2(hexAt(i, view.width));
    path.moveTo(p.x + CORNERS[0][0] * hexPx, p.y + CORNERS[0][1] * hexPx);
    for (let k = 1; k < 6; k++) path.lineTo(p.x + CORNERS[k][0] * hexPx, p.y + CORNERS[k][1] * hexPx);
    path.closePath();
  }
  return path;
}

/** Rhumb lines: 16 at normal zoom (the long ones on the cardinal and intercardinal bearings, short ones between), 8 zoomed out. */
function drawRhumb(ctx: Ctx, view: PlayerView, site: HallSite, s2: S2, hexPx: number, tint: string, far: boolean) {
  const o = s2(site.city.pos);
  ctx.save();
  ctx.clip(territory(view, site.holder, site.city.pos, s2, hexPx, Math.ceil(H.reach) + 1));
  ctx.lineCap = 'round';
  const n = far ? 8 : 16;
  const [a0, a1] = far ? [H.rhumbStartFar, H.rhumbEndFar] : [H.rhumbStart, H.rhumbEnd];
  for (let k = 0; k < n; k++) {
    const ang = ((k * 360) / n - 90) * (Math.PI / 180);
    const long = far || k % 2 === 0;
    const r1 = hexPx * 0.78, r2 = hexPx * (long ? H.reach : H.reach * 0.52);
    const g = ctx.createRadialGradient(o.x, o.y, r1, o.x, o.y, r2);
    g.addColorStop(0, rgba(tint, a0));
    g.addColorStop(1, rgba(tint, a1));
    ctx.strokeStyle = g;
    ctx.lineWidth = long ? P.thin * 0.95 : P.hair * 1.2;
    ctx.beginPath();
    ctx.moveTo(o.x + Math.cos(ang) * r1, o.y + Math.sin(ang) * r1);
    ctx.lineTo(o.x + Math.cos(ang) * r2, o.y + Math.sin(ang) * r2);
    ctx.stroke();
  }
  ctx.restore();
}

/** The plaque: a vellum disc the town reads through, a degree ring, four ink cardinal points and four intercardinal points in the holder's tint. */
function drawRose(ctx: Ctx, x: number, y: number, R: number, tint: string) {
  ctx.save();
  ctx.translate(x, y);
  ctx.lineJoin = 'miter';
  ctx.beginPath(); ctx.arc(0, 0, R * 0.66, 0, Math.PI * 2); ctx.fillStyle = rgba(T.vellum, H.plaque); ctx.fill();
  ctx.strokeStyle = rgba(T.ink, H.plaqueRim); ctx.lineWidth = P.thin; ctx.stroke();
  ctx.beginPath(); ctx.arc(0, 0, R * 0.6, 0, Math.PI * 2); ctx.strokeStyle = rgba(T.ink, 0.5); ctx.lineWidth = P.hair; ctx.stroke();
  ctx.beginPath();
  for (let k = 0; k < 32; k++) {
    const a = (k * Math.PI) / 16, major = k % 4 === 0;
    ctx.moveTo(Math.cos(a) * R * 0.66, Math.sin(a) * R * 0.66);
    ctx.lineTo(Math.cos(a) * R * (major ? 0.76 : 0.71), Math.sin(a) * R * (major ? 0.76 : 0.71));
  }
  ctx.strokeStyle = rgba(T.ink, 0.7); ctx.lineWidth = P.hair; ctx.stroke();
  // Each point is a split diamond: one half dark, one half vellum, the way a printed rose is shaded.
  const point = (ang: number, tip: number, base: number, wid: number, dark: string) => {
    const dx = Math.cos(ang), dy = Math.sin(ang), nx = -dy, ny = dx;
    const bx = dx * base, by = dy * base;
    const l = [bx + nx * wid, by + ny * wid], r = [bx - nx * wid, by - ny * wid];
    ctx.beginPath(); ctx.moveTo(dx * tip, dy * tip); ctx.lineTo(l[0], l[1]); ctx.lineTo(0, 0); ctx.closePath(); ctx.fillStyle = dark; ctx.fill();
    ctx.beginPath(); ctx.moveTo(dx * tip, dy * tip); ctx.lineTo(r[0], r[1]); ctx.lineTo(0, 0); ctx.closePath(); ctx.fillStyle = rgba(T.vellum, 0.96); ctx.fill();
    ctx.beginPath(); ctx.moveTo(dx * tip, dy * tip); ctx.lineTo(l[0], l[1]); ctx.lineTo(0, 0); ctx.lineTo(r[0], r[1]); ctx.closePath();
    ctx.strokeStyle = T.ink; ctx.lineWidth = P.thin * 0.9; ctx.stroke();
  };
  for (let k = 0; k < 4; k++) point((k * Math.PI) / 2 + Math.PI / 4, R * 0.5, R * 0.15, R * 0.09, rgba(tint, 0.78));
  for (let k = 0; k < 4; k++) point((k * Math.PI) / 2, R * 1.0, R * 0.18, R * 0.1, rgba(T.ink, 0.82));
  ctx.beginPath(); ctx.arc(0, 0, R * 0.07, 0, Math.PI * 2); ctx.fillStyle = tint; ctx.fill(); ctx.strokeStyle = T.ink; ctx.lineWidth = P.thin; ctx.stroke();
  ctx.restore();
}

/** Under banners and units: the rhumb lines across the holder's land and, at normal zoom, the plaque on the Hall's city. */
export function drawHallLand(ctx: Ctx, view: PlayerView, s2: S2, hexPx: number, w: number, h: number) {
  const far = isFar(hexPx);
  for (const site of hallSites(view)) {
    const o = s2(site.city.pos);
    if (o.x < -hexPx * 9 || o.x > w + hexPx * 9 || o.y < -hexPx * 9 || o.y > h + hexPx * 9) continue;
    const tint = mapPlayerColor(site.holder);
    ctx.save();
    if (!site.city.visible && site.city.owner !== site.holder) ctx.globalAlpha = H.remembered;
    drawRhumb(ctx, view, site, s2, hexPx, tint, far);
    // A walled city's ring has towers at its N and S corners and runs close past E and W: the rose shrinks to sit inside it.
    if (!far) drawRose(ctx, o.x, o.y + hexPx * 0.04, hexPx * (site.city.walls ? 0.76 : 1), tint);
    ctx.restore();
  }
}

/** The badge's centre and radius beside a city's seal when zoomed out (upper left, opposite the capital's star). */
export function hallBadgeAt(cityScreen: Pt, hexPx: number): { x: number; y: number; r: number } {
  const q = sealRadius(hexPx);
  return { x: cityScreen.x - q * 0.9, y: cityScreen.y - hexPx * 0.1 - q * 0.9, r: Math.max(6, q * 0.7) };
}

/** Zoomed out: a gilt-ringed token with the Hall's own icon beside the city seal, so it reads at far zoom. */
export function drawHallBadge(ctx: Ctx, view: PlayerView, s2: S2, hexPx: number, w: number, h: number) {
  if (!isFar(hexPx)) return;
  for (const { city } of hallSites(view)) {
    const o = s2(city.pos);
    if (o.x < -40 || o.x > w + 40 || o.y < -40 || o.y > h + 40) continue;
    const { x, y, r } = hallBadgeAt(o, hexPx);
    ctx.save();
    ctx.beginPath(); ctx.arc(x + 0.6, y + 0.9, r, 0, Math.PI * 2); ctx.fillStyle = T.shadow; ctx.fill();
    ctx.beginPath(); ctx.arc(x, y, r, 0, Math.PI * 2); ctx.fillStyle = ATLAS.seat.plate; ctx.fill();
    ctx.strokeStyle = T.gold; ctx.lineWidth = P.line; ctx.stroke();
    const g = r * 1.7;
    const icon = tintedIcon('building-courierhall', T.ink, Math.ceil(g * (window.devicePixelRatio || 1)));
    if (icon) ctx.drawImage(icon, x - g / 2, y - g / 2, g, g);
    ctx.restore();
  }
}
