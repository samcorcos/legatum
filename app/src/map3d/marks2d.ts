// The 3D map's flat marks, drawn on a 2D canvas over the scene with the 2D map's own sprites, so both maps say the
// same things: city banners (name, population, capital star, goal seal, Walls level, own cities' growth and production
// meters, damage or siege bar) and each unit's health bar, rank and order badge.
import type { CityBuildFocus, CityView, PlayerView, ProductionItem, UnitView } from '../types';
import type { LookId } from '../brand/look';
import { bannerBarOf, bannerMeters, bannerSpec, drawColoniaSeal, drawFortifyMark, MARK_ICON, type PendingMark } from '../map/render';
import { drawEraPlaque, plaqueHeight, unitEra } from '../map/unitEra';
import { siegePropFor, siegePropImage } from '../siegeProps';
import { heirTag } from '../heir';
import { isColoniaFounder } from '../goalCities';
import { isBarbarian } from '../barbarians';
import type { UnitKind } from '../types';
import { drawBadge, drawBanner, drawBannerMeters, drawPromotion, drawTag, drawTokenHealth, FONTS, TOKEN_INK } from '../map/sprites';
import { offerChip, offerOn, offerTone } from '../buyCity';
import { drawSiegeBadge, drawSiegeBar, SIEGE_BAR_GAP, WALLS_STONE, WALLS_TRACK } from '../map/siegeMark';
import type { SiegeLevel } from '../siege';
import { tombStatus } from '../goalCities';
import { healthColor } from '../theme';
import { tintedIcon } from '../icons';
import { MAP_STYLES } from '../map/style';
import { roman } from '../brand/roman';
import type { PathStep } from '../path';

const LOOK: LookId = 'atlas';

export interface ScreenPoint { x: number; y: number; radius: number }

export interface Marks2dInput {
  view: PlayerView;
  /** A city's banner anchor on screen (bottom centre of the banner), or null when off screen or behind the camera. */
  cityAt: (c: CityView) => ScreenPoint | null;
  /** The top of a unit's figure on screen (the head), or null. */
  unitAt: (u: UnitView) => ScreenPoint | null;
  /** Where the figure stands (the centre of its base disc) on screen. */
  unitFootAt: (u: UnitView) => ScreenPoint | null;
  marks: ReadonlyMap<number, PendingMark>;
  focusOf: (c: CityView) => CityBuildFocus | null;
  builds: ReadonlyMap<number, ProductionItem>;
  siege: ReadonlyMap<number, SiegeLevel>;
  selectedCity: number | null;
  /** Camera distance: far out, banners shrink and meters drop. */
  far: boolean;
}

/** A city banner as drawn this frame, in screen pixels: the banner and its meters, for clicks on the banner. */
export interface BannerHit3d { city: number; parts: { x: number; y: number; w: number; h: number }[] }

/** Banner hit areas drawn last win: the topmost banner under a point, or null. */
export function bannerCityAt3d(hits: readonly BannerHit3d[], x: number, y: number): number | null {
  return hits.findLast((b) => b.parts.some((r) => x >= r.x && x <= r.x + r.w && y >= r.y && y <= r.y + r.h))?.city ?? null;
}

export function drawMarks2d(ctx: CanvasRenderingContext2D, w: number, h: number, dpr: number, inp: Marks2dInput): BannerHit3d[] {
  const hits: BannerHit3d[] = [];
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.clearRect(0, 0, w, h);
  const { view } = inp;
  const factionOf = new Map(view.players.map((p) => [p.id, p.faction] as const));
  // Units first, so a banner sits over a unit's badges where they meet.
  for (const u of view.units) {
    const p = inp.unitAt(u);
    if (!p || p.x < -40 || p.x > w + 40 || p.y < -40 || p.y > h + 40) continue;
    const mine = u.owner === view.me;
    const tw = Math.max(8, Math.min(15, p.radius * 0.32));
    // drawTokenHealth and the badges are placed around a token centred at (x, y) with half-width tw: the figure's
    // head is the token's top, so the centre sits one half-width below it.
    const x = p.x, y = p.y + tw;
    if (u.hp < 100) drawTokenHealth(ctx, LOOK, u.hp, x, y - tw * 2.2, tw);
    if (inp.far) continue;
    const faction = factionOf.get(u.owner);
    // At the figure's feet, as on the 2D miniature's base: the era plaque on the front of the disc (user, 2026-10-11:
    // tell the eras apart), an Engineer's siege gear on the right, and the Heir tag under it.
    const foot = inp.unitFootAt(u);
    let tagY = foot ? foot.y + tw * 0.55 : y + tw * 2;
    if (foot && u.military && !isBarbarian(u.owner)) {
      const py = foot.y + tw * 0.45;
      drawEraPlaque(ctx, unitEra(u.kind as UnitKind), foot.x, py, tw, FONTS[LOOK].display);
      tagY = py + plaqueHeight(tw) / 2 + 4;
    }
    const prop = foot ? siegePropFor(view, u) : null;
    const gear = prop ? siegePropImage(prop) : null;
    if (foot && gear) { const s = tw * 1.5; ctx.drawImage(gear, foot.x + tw * 0.7, foot.y - s * 0.7, s, s); }
    const heir = heirTag(view, u);
    if (heir) drawTag(ctx, LOOK, x, tagY + 6, heir, mine ? 'good' : u.owner === u.heir_of ? 'neutral' : 'bad');
    if (u.military && u.level) drawPromotion(ctx, LOOK, u.level, x, y, tw, faction);
    // Rome's Colonia founder: the laurel seal over the head, above the health bar.
    if (mine && isColoniaFounder(view, u.id)) drawColoniaSeal(ctx, x - tw * 1.5, y - tw * 2.2, tw * 0.8, dpr);
    const bx = x + tw * 0.95, by = y - tw * 0.85;
    // Fortified: on a paper disc beside the order badge, clear of the figures' armour.
    if (u.fortify > 0) {
      const fx = bx, fy = by + tw * 0.95;
      ctx.beginPath(); ctx.arc(fx, fy, 8, 0, Math.PI * 2); ctx.fillStyle = TOKEN_INK.atlas.paper; ctx.fill();
      ctx.strokeStyle = TOKEN_INK.atlas.ink; ctx.lineWidth = 1; ctx.stroke();
      drawFortifyMark(ctx, fx - 6, fy - 6, dpr);
    }
    const mark = inp.marks.get(u.id);
    if (mark) drawBadge(ctx, LOOK, bx, by, tw * 0.42, '', true, MARK_ICON[mark]);
    else if (mine && u.unsupplied) drawBadge(ctx, LOOK, bx, by, tw * 0.42, '', true, 'ui-alert');
    else if (mine && u.standing_order) drawBadge(ctx, LOOK, bx, by, tw * 0.38, '', false, u.standing_order === 'Heal' ? 'mark-heal' : u.standing_order === 'AutoFire' ? 'mark-autofire' : 'mark-sleep');
    else if (mine && u.goto) drawBadge(ctx, LOOK, bx, by, tw * 0.38, '', false, 'mark-move');
  }
  const myTomb = tombStatus(view.players.find((pl) => pl.id === view.me) ?? {});
  for (const c of view.cities) {
    const p = inp.cityAt(c);
    if (!p || p.x < -200 || p.x > w + 200 || p.y < -80 || p.y > h + 120) continue;
    const faction = factionOf.get(c.owner);
    const spec = bannerSpec(view, c, faction ? `glyph-${faction.toLowerCase()}` : null, c.id === inp.selectedCity);
    const fs = inp.far ? 11 : Math.round(Math.max(11, Math.min(15, p.radius * 0.4)) * 2) / 2;
    const meters = !inp.far && c.owner === view.me && c.visible ? bannerMeters(c, faction, inp.focusOf(c), inp.builds.get(c.id) ?? null, myTomb) : [];
    const r = drawBanner(ctx, LOOK, spec, p.x, p.y, fs, dpr, c.visible ? 1 : 0.85);
    const parts = [r];
    hits.push({ city: c.id, parts });
    // Byzantium buying the city: the same public countdown chip as the 2D banner.
    const chip = offerChip(view, c.id);
    if (chip) drawTag(ctx, LOOK, p.x, r.y - 17, chip, offerTone(view, offerOn(view, c.id)!));
    const level = inp.siege.get(c.id);
    const bar = bannerBarOf(c, c.hp);
    if (level) {
      const barY = r.y + r.h + SIEGE_BAR_GAP;
      drawSiegeBar(ctx, p.x, barY, Math.min(Math.max(r.w - 8, 64), 128), bar.value, bar.max, FONTS[LOOK].sans, true, bar.walls);
      if (meters.length) parts.push(drawBannerMeters(ctx, LOOK, meters, p.x, barY + 16, dpr));
      drawSiegeBadge(ctx, r.x + r.w, r.y, 11, dpr);
      continue;
    }
    const hurt = c.visible && bar.value < bar.max;
    if (meters.length) parts.push(drawBannerMeters(ctx, LOOK, meters, p.x, r.y + r.h + (hurt ? 6 : 1), dpr));
    if (hurt) {
      const f = Math.max(0, bar.value / bar.max);
      const bw = Math.min(r.w - 16, 72), bx = p.x - bw / 2, by = r.y + r.h + 1;
      ctx.fillStyle = bar.walls ? WALLS_TRACK : TOKEN_INK.atlas.paper; ctx.fillRect(bx, by, bw, 4);
      ctx.fillStyle = bar.walls ? WALLS_STONE : healthColor(f); ctx.fillRect(bx, by, bw * f, 4);
      ctx.strokeStyle = TOKEN_INK.atlas.ink; ctx.lineWidth = 0.8; ctx.strokeRect(bx, by, bw, 4);
      if (bar.walls) {
        const ic = tintedIcon('building-walls', TOKEN_INK.atlas.ink, Math.ceil(10 * dpr));
        if (ic) ctx.drawImage(ic, bx - 11, by - 3, 10, 10);
      }
    }
  }
  return hits;
}

/** The selected unit's planned route, as on the 2D map: a dashed ink line rounded through the tile centres, stopping short of
 *  the destination, with a roundel carrying the turn (in roman numerals) wherever a turn ends. Attack routes are in attack ink. */
export function drawRoute3d(ctx: CanvasRenderingContext2D, pts: ({ x: number; y: number } | null)[], steps: readonly PathStep[], attack: boolean, time: number) {
  const S = MAP_STYLES.atlas;
  const P = pts.filter((p): p is { x: number; y: number } => !!p);
  if (P.length < 2 || P.length !== pts.length) return;
  const color = attack ? S.ink.attack : S.ink.route;
  const last = P[P.length - 1], prev = P[P.length - 2];
  const d = Math.hypot(last.x - prev.x, last.y - prev.y) || 1;
  const cut = Math.min(12, d * 0.5);
  const end = { x: last.x - ((last.x - prev.x) / d) * cut, y: last.y - ((last.y - prev.y) / d) * cut };
  ctx.save();
  ctx.lineCap = 'round'; ctx.lineJoin = 'round';
  const trace = () => {
    ctx.beginPath();
    ctx.moveTo(P[0].x, P[0].y);
    for (let k = 1; k < P.length - 1; k++) {
      const m = k === P.length - 2 ? end : { x: (P[k].x + P[k + 1].x) / 2, y: (P[k].y + P[k + 1].y) / 2 };
      ctx.quadraticCurveTo(P[k].x, P[k].y, m.x, m.y);
    }
    if (P.length === 2) ctx.lineTo(end.x, end.y);
  };
  // A paper halo first, so the ink reads on dark forest and on light sand alike.
  trace(); ctx.strokeStyle = S.ink.markerFill; ctx.globalAlpha = 0.75; ctx.lineWidth = 5; ctx.stroke(); ctx.globalAlpha = 1;
  trace(); ctx.strokeStyle = color; ctx.lineWidth = 2.2;
  if (S.ink.routeDash) { ctx.setLineDash(S.ink.routeDash); ctx.lineDashOffset = -time / 60; }
  ctx.stroke(); ctx.setLineDash([]);
  // The destination: an ink ring where the route ends.
  ctx.beginPath(); ctx.arc(last.x, last.y, 7, 0, Math.PI * 2); ctx.lineWidth = 2; ctx.strokeStyle = color; ctx.stroke();
  steps.forEach((st, k) => {
    if (!st.endsTurn) return;
    const p = P[k + 1];
    if (!p) return;
    ctx.beginPath(); ctx.arc(p.x, p.y, 11, 0, Math.PI * 2); ctx.fillStyle = S.ink.markerFill; ctx.fill();
    ctx.strokeStyle = color; ctx.lineWidth = 1.4; ctx.stroke();
    ctx.beginPath(); ctx.arc(p.x, p.y, 8, 0, Math.PI * 2); ctx.lineWidth = 0.6; ctx.stroke();
    ctx.font = `700 13px ${FONTS[LOOK].display}`;
    ctx.fillStyle = color; ctx.textAlign = 'center'; ctx.textBaseline = 'middle';
    ctx.fillText(roman(st.turn), p.x, p.y + 1);
  });
  ctx.restore();
}
