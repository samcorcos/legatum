// The 3D map's flat marks, drawn on a 2D canvas over the scene with the 2D map's own sprites, so both maps say the
// same things: city banners (name, population, capital star, goal seal, Walls level, own cities' growth and production
// meters, damage or siege bar) and each unit's health bar, rank and order badge.
import type { CityBuildFocus, CityView, PlayerView, ProductionItem, UnitView } from '../types';
import type { LookId } from '../brand/look';
import { bannerBarOf, bannerMeters, bannerSpec, MARK_ICON, type PendingMark } from '../map/render';
import { drawBadge, drawBanner, drawBannerMeters, drawPromotion, drawTokenHealth, FONTS, TOKEN_INK } from '../map/sprites';
import { drawSiegeBadge, drawSiegeBar, SIEGE_BAR_GAP, WALLS_STONE, WALLS_TRACK } from '../map/siegeMark';
import type { SiegeLevel } from '../siege';
import { tombStatus } from '../goalCities';
import { healthColor } from '../theme';
import { tintedIcon } from '../icons';

const LOOK: LookId = 'atlas';

export interface ScreenPoint { x: number; y: number; radius: number }

export interface Marks2dInput {
  view: PlayerView;
  /** A city's banner anchor on screen (bottom centre of the banner), or null when off screen or behind the camera. */
  cityAt: (c: CityView) => ScreenPoint | null;
  /** The top of a unit's figure on screen (the head), or null. */
  unitAt: (u: UnitView) => ScreenPoint | null;
  marks: ReadonlyMap<number, PendingMark>;
  focusOf: (c: CityView) => CityBuildFocus | null;
  builds: ReadonlyMap<number, ProductionItem>;
  siege: ReadonlyMap<number, SiegeLevel>;
  selectedCity: number | null;
  /** Camera distance: far out, banners shrink and meters drop. */
  far: boolean;
}

export function drawMarks2d(ctx: CanvasRenderingContext2D, w: number, h: number, dpr: number, inp: Marks2dInput) {
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
    if (u.military && u.level) drawPromotion(ctx, LOOK, u.level, x, y, tw, factionOf.get(u.owner));
    const bx = x + tw * 0.95, by = y - tw * 0.85;
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
    const level = inp.siege.get(c.id);
    const bar = bannerBarOf(c, c.hp);
    if (level) {
      const barY = r.y + r.h + SIEGE_BAR_GAP;
      drawSiegeBar(ctx, p.x, barY, Math.min(Math.max(r.w - 8, 64), 128), bar.value, bar.max, FONTS[LOOK].sans, true, bar.walls);
      if (meters.length) drawBannerMeters(ctx, LOOK, meters, p.x, barY + 16, dpr);
      drawSiegeBadge(ctx, r.x + r.w, r.y, 11, dpr);
      continue;
    }
    const hurt = c.visible && bar.value < bar.max;
    if (meters.length) drawBannerMeters(ctx, LOOK, meters, p.x, r.y + r.h + (hurt ? 6 : 1), dpr);
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
}
