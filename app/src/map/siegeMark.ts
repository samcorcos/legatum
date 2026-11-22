// The alarmed state of a city under attack on the map (user, 2026-10-08: "I shouldn't be surprised
// to see that one of my cities is about to get taken over"). Three marks that read at a glance
// on any terrain and at both zoom levels, and never rely on colour alone:
//   - a wide HP bar with its number (the shape and the digits say "hurt", not just the red);
//   - a siege badge: crossed swords on a red disc with a white rim;
//   - a double ring (white halo under red) around the banner or seal, pulsing gently, steady when
//     the player prefers reduced motion.
import { tintedIcon } from '../icons';
import type { SiegeLevel } from '../siege';

const SIEGE_RED = '#c1271b';
const HALO = '#fffaf0';
const INK = '#2b2419';

/** Height of the wide HP bar and the gap above it (CSS px). */
export const SIEGE_BAR_H = 12;
export const SIEGE_BAR_GAP = 2;

/** How strongly the ring breathes: a city that falls next turn is the loudest. */
const PULSE: Record<SiegeLevel, number> = { now: 0.5, soon: 0.35, holding: 0.2, quiet: 0.1 };

/** 0..1, steady at 1 when motion is reduced. */
function breath(time: number, level: SiegeLevel, reduced: boolean): number {
  if (reduced) return 1;
  return 1 - PULSE[level] * (0.5 + 0.5 * Math.sin(time / (level === 'now' ? 220 : 340)));
}

/** The wide HP bar under an alarmed banner: track, red fill, white rim, ink edge and "120/200". */
export function drawSiegeBar(ctx: CanvasRenderingContext2D, cx: number, y: number, w: number, hp: number, maxHp: number, fontFamily: string, withText = true) {
  const h = withText ? SIEGE_BAR_H : 6;
  const x = Math.round(cx - w / 2);
  const f = Math.max(0, Math.min(1, hp / maxHp));
  ctx.save();
  ctx.fillStyle = HALO;
  ctx.fillRect(x - 1.5, y - 1.5, w + 3, h + 3);
  ctx.fillStyle = '#4a3b2a';
  ctx.fillRect(x, y, w, h);
  ctx.fillStyle = SIEGE_RED;
  ctx.fillRect(x, y, Math.round(w * f), h);
  // A hard edge at the fill's end, so the remaining HP is a length, not a shade.
  if (f > 0 && f < 1) { ctx.fillStyle = HALO; ctx.fillRect(x + Math.round(w * f) - 1, y, 1.5, h); }
  ctx.strokeStyle = INK;
  ctx.lineWidth = 1;
  ctx.strokeRect(x + 0.5, y + 0.5, w - 1, h - 1);
  if (withText) {
    ctx.font = `700 10px ${fontFamily}`;
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    const label = `${hp}/${maxHp}`;
    ctx.lineJoin = 'round';
    ctx.lineWidth = 3;
    ctx.strokeStyle = INK;
    ctx.strokeText(label, cx, y + h / 2 + 0.5);
    ctx.fillStyle = HALO;
    ctx.fillText(label, cx, y + h / 2 + 0.5);
  }
  ctx.restore();
}

/** Crossed swords on a red disc with a white rim, centred on (cx, cy). */
export function drawSiegeBadge(ctx: CanvasRenderingContext2D, cx: number, cy: number, r: number, dpr: number) {
  ctx.save();
  ctx.beginPath();
  ctx.arc(cx, cy, r + 2, 0, Math.PI * 2);
  ctx.fillStyle = HALO;
  ctx.fill();
  ctx.beginPath();
  ctx.arc(cx, cy, r, 0, Math.PI * 2);
  ctx.fillStyle = SIEGE_RED;
  ctx.fill();
  ctx.lineWidth = 1;
  ctx.strokeStyle = INK;
  ctx.stroke();
  const size = Math.round(r * 1.5);
  const icon = tintedIcon('ui-siege', HALO, size * dpr);
  if (icon) ctx.drawImage(icon, cx - size / 2, cy - size / 2, size, size);
  ctx.restore();
}

export interface SiegeRect { x: number; y: number; w: number; h: number }

/** The ring: a white halo under a red stroke around a banner (rounded rect), pulsing with `time`. */
export function drawSiegeRing(ctx: CanvasRenderingContext2D, rect: SiegeRect, level: SiegeLevel, time: number, reduced: boolean) {
  const k = breath(time, level, reduced);
  const pad = 4 + (reduced ? 0 : (1 - k) * 4);
  const x = rect.x - pad, y = rect.y - pad, w = rect.w + 2 * pad, h = rect.h + 2 * pad;
  ctx.save();
  ctx.beginPath();
  ctx.roundRect(x, y, w, h, 5);
  ctx.lineWidth = 5.5;
  ctx.strokeStyle = HALO;
  ctx.globalAlpha = 0.55 + 0.45 * k;
  ctx.stroke();
  ctx.lineWidth = 3;
  ctx.strokeStyle = SIEGE_RED;
  ctx.globalAlpha = 0.6 + 0.4 * k;
  ctx.stroke();
  ctx.restore();
}

/** The most a breathing banner ring covers (CSS px): its widest pad plus half its halo stroke. */
export function siegeRingBox(rect: SiegeRect): SiegeRect {
  const pad = 8 + 3.5;
  return { x: rect.x - pad, y: rect.y - pad, w: rect.w + 2 * pad, h: rect.h + 2 * pad };
}

/** The most a breathing seal ring covers (CSS px). */
export function siegeSealRingBox(cx: number, cy: number, r: number): SiegeRect {
  const rr = r + 8 + 3.5;
  return { x: cx - rr, y: cy - rr, w: 2 * rr, h: 2 * rr };
}

/** The same ring, round, for a city zoomed out to its seal. */
export function drawSiegeSealRing(ctx: CanvasRenderingContext2D, cx: number, cy: number, r: number, level: SiegeLevel, time: number, reduced: boolean) {
  const k = breath(time, level, reduced);
  const rr = r + 4 + (reduced ? 0 : (1 - k) * 4);
  ctx.save();
  ctx.beginPath();
  ctx.arc(cx, cy, rr, 0, Math.PI * 2);
  ctx.lineWidth = 5.5;
  ctx.strokeStyle = HALO;
  ctx.globalAlpha = 0.55 + 0.45 * k;
  ctx.stroke();
  ctx.lineWidth = 3;
  ctx.strokeStyle = SIEGE_RED;
  ctx.globalAlpha = 0.6 + 0.4 * k;
  ctx.stroke();
  ctx.restore();
}
