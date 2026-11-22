// Drawing the combat plan (combatFx.ts) on the map canvas, in screen space, above the units and under the damage cues.
// The look is the atlas's (legatum-art): engraved ink strokes for arrows and trails, sepia wash for dust, grey-blue ink
// smoke (never pure white), vellum for a flash, and the one accent red only where a blow lands. No glow, no blur, no
// neon: everything is a stroke, a filled shape or a pre-rendered wash sprite, so a late-game turn with dozens of fights
// stays cheap. Zoomed out (the far tier) only the arcs and rings are drawn.
import { ARC, CAPTURE_MS, rnd, type CombatPlan, type FxCapture, type FxClash, type FxDeath, type FxShot, type Pt } from './combatFx';
import { ATLAS } from './style';
import { airPlanImage } from '../unitArt';

const INK = ATLAS.tone.ink;
const INK2 = ATLAS.tone.ink2;
const VELLUM = ATLAS.tone.vellum;
const RED = ATLAS.tone.red;
const SPARK = ATLAS.tone.bad;

/** Effects are sized from the hex; this lifts them to read at trailer distance without swamping the units. */
const SIZE = 1.5;

export type Wash = 'dust' | 'smoke' | 'soot';
/** Each tone: the wash inside the cloud, and the ink of its engraved outline and shading. */
const WASH: Record<Wash, { fill: string; ink: string }> = {
  dust: { fill: 'rgb(226,208,172)', ink: 'rgb(118,90,58)' },
  smoke: { fill: 'rgb(220,222,216)', ink: 'rgb(72,82,92)' },
  soot: { fill: 'rgb(132,122,108)', ink: 'rgb(40,33,26)' },
};
const sprites = new Map<string, HTMLCanvasElement | null>();
const SPRITE = 96;

/**
 * An engraved cloud, as old maps draw smoke and dust: three or four round lobes with one ink outline around them all,
 * filled with a pale wash, and a few hatch strokes on the shaded side. Pre-rendered once per tone and variant.
 */
function puffSprite(tone: Wash, variant: number): HTMLCanvasElement | null {
  const k = `${tone}${variant}`;
  if (sprites.has(k)) return sprites.get(k)!;
  let c: HTMLCanvasElement | null = null;
  try {
    c = document.createElement('canvas');
    c.width = c.height = SPRITE;
    const g = c.getContext('2d');
    if (!g) throw new Error('no 2d');
    const { fill, ink } = WASH[tone];
    const m = SPRITE / 2;
    const lobes: [number, number, number][] = [];
    const n = 3 + (variant % 2);
    for (let i = 0; i < n; i++) {
      const a = (i / n) * Math.PI * 2 + variant * 1.3;
      lobes.push([m + Math.cos(a) * 13, m + Math.sin(a) * 10 + 3, 17 + ((i + variant) % 3) * 3]);
    }
    lobes.push([m, m - 4, 21]);
    const path = () => { g.beginPath(); for (const [x, y, r] of lobes) { g.moveTo(x + r, y); g.arc(x, y, r, 0, Math.PI * 2); } };
    // Stroke every lobe thick, then fill them all on top: only the outer outline survives.
    path(); g.strokeStyle = ink; g.lineWidth = 4; g.stroke();
    path(); g.fillStyle = fill; g.fill();
    // Shading: short curved hatches on the lower right, clipped to the cloud.
    g.save(); path(); g.clip();
    g.strokeStyle = ink; g.globalAlpha = 0.55; g.lineWidth = 1.6;
    for (let i = 0; i < 5; i++) {
      const y = m + 4 + i * 5;
      g.beginPath(); g.moveTo(m - 4 + i * 3, y + 10); g.quadraticCurveTo(m + 14, y - 2, m + 34 - i * 2, y + 6); g.stroke();
    }
    g.restore();
  } catch { c = null; }
  sprites.set(k, c);
  return c;
}

export interface FxView {
  toScreen(p: Pt): Pt;
  /** Hex radius on screen (px). */
  hexPx: number;
  far: boolean;
  w: number;
  h: number;
  colorOf(owner: number): string;
}

const easeOut = (k: number) => 1 - (1 - k) * (1 - k);

function puff(ctx: CanvasRenderingContext2D, tone: Wash, seed: number, x: number, y: number, r: number, alpha: number) {
  if (alpha <= 0.01 || r <= 0.5) return;
  const s = puffSprite(tone, Math.floor(rnd(seed, 99) * 3));
  if (!s) return;
  ctx.globalAlpha = Math.min(1, alpha);
  ctx.drawImage(s, x - r, y - r, r * 2, r * 2);
}

/** A cloud of puffs born at `t0` around (x, y): they spread, drift and fade. */
export function cloud(ctx: CanvasRenderingContext2D, tone: Wash, seed: number, t: number, t0: number, life: number, x: number, y: number, n: number, r0: number, r1: number, spread: number, drift: Pt, alpha: number) {
  const dt = t - t0;
  if (dt < 0 || dt > life * 1.3) return;
  r0 *= SIZE; r1 *= SIZE; spread *= SIZE;
  for (let i = 0; i < n; i++) {
    const own = life * (0.75 + rnd(seed, i * 5 + 1) * 0.5);
    const k = dt / own;
    if (k >= 1) continue;
    const a = rnd(seed, i * 5 + 2) * Math.PI * 2;
    const d = spread * (0.35 + rnd(seed, i * 5 + 3) * 0.65) * easeOut(k);
    const px = x + Math.cos(a) * d + drift.x * k;
    const py = y + Math.sin(a) * d * 0.6 + drift.y * k;
    puff(ctx, tone, seed + i, px, py, r0 + (r1 - r0) * easeOut(k), alpha * (1 - k) * (k < 0.08 ? k / 0.08 : 1));
  }
}

/** Engraved burst: tapered ink wedges out from a point (a woodcut sunburst), one to three in red where the blow landed. */
export function rays(ctx: CanvasRenderingContext2D, seed: number, t: number, t0: number, x: number, y: number, r: number, n: number, red: number, life = 320) {
  const dt = t - t0;
  if (dt < 0 || dt > life) return;
  r *= SIZE;
  const k = dt / life;
  const grow = easeOut(Math.min(1, k * 2.4));
  ctx.globalAlpha = 1 - k * k;
  for (const [color, count] of [[INK, n], [RED, red]] as const) {
    if (!count) continue;
    ctx.beginPath();
    for (let i = 0; i < count; i++) {
      const isRed = color === RED;
      const a = (isRed ? 0.5 + i * 2.2 : (i + 0.5) / n * Math.PI * 2) + (rnd(seed, i + (isRed ? 50 : 0)) - 0.5) * 0.35;
      const inner = r * (0.22 + 0.45 * k);
      const outer = r * (0.5 + (0.5 + rnd(seed, i + 20) * 0.6) * grow);
      const half = Math.max(1.2, r * (isRed ? 0.07 : 0.05)) * (1 - k * 0.5);
      const ca = Math.cos(a), sa = Math.sin(a);
      ctx.moveTo(x + ca * inner - sa * half, y + sa * inner + ca * half);
      ctx.lineTo(x + ca * outer, y + sa * outer);
      ctx.lineTo(x + ca * inner + sa * half, y + sa * inner - ca * half);
      ctx.closePath();
    }
    ctx.fillStyle = color;
    ctx.fill();
  }
}

/** A vellum flash with a red heart: the instant of a hit, or a muzzle. */
export function flash(ctx: CanvasRenderingContext2D, t: number, t0: number, x: number, y: number, r: number, life = 150, heart = true) {
  const dt = t - t0;
  if (dt < 0 || dt > life) return;
  r *= SIZE;
  const k = dt / life;
  ctx.globalAlpha = 0.9 * (1 - k);
  ctx.fillStyle = VELLUM;
  ctx.beginPath(); ctx.arc(x, y, r * (0.6 + 0.6 * k), 0, Math.PI * 2); ctx.fill();
  if (heart) {
    ctx.globalAlpha = 1 - k;
    ctx.fillStyle = SPARK;
    ctx.beginPath(); ctx.arc(x, y, r * 0.16 * (1 - 0.5 * k), 0, Math.PI * 2); ctx.fill();
  }
}

/** Specks thrown up by a heavy blow, falling back. */
export function debris(ctx: CanvasRenderingContext2D, seed: number, t: number, t0: number, x: number, y: number, r: number, n: number, color = INK) {
  const dt = t - t0;
  const life = 620;
  if (dt < 0 || dt > life) return;
  r *= SIZE;
  const k = dt / life;
  ctx.globalAlpha = 1 - k;
  ctx.fillStyle = color;
  ctx.beginPath();
  for (let i = 0; i < n; i++) {
    const a = -Math.PI * (0.1 + rnd(seed, i + 70) * 0.8);
    const v = r * (0.8 + rnd(seed, i + 80) * 0.9);
    const px = x + Math.cos(a) * v * k;
    const py = y + Math.sin(a) * v * k + r * 1.6 * k * k;
    const s = 1.5 + rnd(seed, i + 90) * 2;
    ctx.rect(px - s / 2, py - s / 2, s, s);
  }
  ctx.fill();
}

export function ring(ctx: CanvasRenderingContext2D, t: number, t0: number, x: number, y: number, r0: number, r1: number, life: number, color: string, width: number) {
  const dt = t - t0;
  if (dt < 0 || dt > life) return;
  const k = dt / life;
  const r = r0 + (r1 - r0) * easeOut(k);
  ctx.globalAlpha = (1 - k) * 0.9;
  ctx.lineWidth = width + 3;
  ctx.strokeStyle = VELLUM;
  ctx.beginPath(); ctx.ellipse(x, y, r, r * 0.72, 0, 0, Math.PI * 2); ctx.stroke();
  ctx.lineWidth = width * (1 - k * 0.6);
  ctx.strokeStyle = color;
  ctx.stroke();
}

/** A point on a shot's arc at k in [0, 1], with the direction of travel. */
function arcAt(a: Pt, b: Pt, height: number, k: number): { p: Pt; d: Pt } {
  const p = { x: a.x + (b.x - a.x) * k, y: a.y + (b.y - a.y) * k - 4 * height * k * (1 - k) };
  const d = { x: b.x - a.x, y: b.y - a.y - 4 * height * (1 - 2 * k) };
  const len = Math.hypot(d.x, d.y) || 1;
  return { p, d: { x: d.x / len, y: d.y / len } };
}

const PER_SHOT: Record<FxShot['kind'], number> = { arrows: 6, bolts: 3, shot: 4, stone: 1, ball: 1, shell: 1, drone: 1, bombs: 3 };
const GUNPOWDER = new Set<FxShot['kind']>(['shot', 'ball', 'shell']);
const HEAVY = new Set<FxShot['kind']>(['stone', 'ball', 'shell', 'bombs']);

/** One shot: its missiles in flight (into the shared stroke paths), and its launch and impact effects. */
function drawShot(ctx: CanvasRenderingContext2D, s: FxShot, t: number, v: FxView, shafts: Path2D, heads: Path2D, trails: Path2D) {
  const a = v.toScreen(s.from), b = v.toScreen(s.to);
  const hp = v.hexPx;
  const dist = Math.hypot(b.x - a.x, b.y - a.y);
  const height = ARC[s.kind] * dist + (s.kind === 'shot' ? 0 : hp * 0.15);
  const n = s.city && s.kind === 'arrows' ? 9 : PER_SHOT[s.kind];
  const impactT = s.launch + s.flight;
  const sx = (b.x - a.x) / (dist || 1), sy = (b.y - a.y) / (dist || 1);
  if (v.far) {
    const k = (t - s.launch) / s.flight;
    if (k > 0 && k < 1) {
      const { p } = arcAt(a, b, height, k);
      ctx.globalAlpha = 1; ctx.fillStyle = INK;
      ctx.beginPath(); ctx.arc(p.x, p.y, 1.8, 0, Math.PI * 2); ctx.fill();
    }
    ring(ctx, t, impactT, b.x, b.y, 2, 9, 420, s.hit ? RED : INK, 1.5);
    return;
  }
  // The muzzle: flash and a bank of grey-blue smoke for gunpowder; for a city, from its wall.
  if (GUNPOWDER.has(s.kind)) {
    flash(ctx, t, s.launch, a.x + sx * hp * 0.3, a.y + sy * hp * 0.3 - hp * 0.2, hp * (s.kind === 'shot' ? 0.16 : 0.26), 110, false);
    rays(ctx, s.seed + 1, t, s.launch, a.x + sx * hp * 0.35, a.y + sy * hp * 0.35 - hp * 0.2, hp * (s.kind === 'shot' ? 0.22 : 0.36), 7, 0, 160);
    // The bank of smoke rolls out in front of the muzzle and lifts, so the gun stays visible behind it.
    cloud(ctx, 'smoke', s.seed, t, s.launch, s.kind === 'shot' ? 1000 : 1350, a.x + sx * hp * 0.6, a.y + sy * hp * 0.6 - hp * 0.3,
      s.kind === 'shot' ? 3 : 5, hp * 0.12, hp * (s.kind === 'shot' ? 0.32 : 0.42), hp * 0.3, { x: sx * hp * 0.5, y: -hp * 0.6 }, 1);
  }
  for (let j = 0; j < n; j++) {
    const lj = s.launch + (n > 1 ? j * 26 + rnd(s.seed, j + 10) * 40 : 0);
    const fj = s.flight * (n > 1 ? 0.94 + rnd(s.seed, j + 11) * 0.12 : 1);
    const k = (t - lj) / fj;
    if (k < 0) continue;
    // Each missile leaves from across the shooter's figure (or along the wall) and lands across the target hex.
    const spread = s.city ? 0.42 : n > 1 ? 0.22 : 0;
    const off = (rnd(s.seed, j + 12) - 0.5) * 2 * spread * hp;
    const from = { x: a.x - sy * off, y: a.y + sx * off - hp * 0.25 };
    const land = n > 1 ? 0.34 : 0.1;
    const to = { x: b.x + (rnd(s.seed, j + 13) - 0.5) * 2 * land * hp, y: b.y + (rnd(s.seed, j + 14) - 0.5) * 1.4 * land * hp };
    if (k < 1) {
      const { p, d } = arcAt(from, to, height * (0.9 + rnd(s.seed, j + 15) * 0.2), k);
      switch (s.kind) {
        case 'arrows': case 'bolts': {
          const len = Math.max(6, hp * (s.kind === 'bolts' ? 0.42 : 0.34));
          shafts.moveTo(p.x - d.x * len, p.y - d.y * len); shafts.lineTo(p.x, p.y);
          // Fletching and a head: a tick each side of the tail, a small barb at the tip.
          const tx = p.x - d.x * len, ty = p.y - d.y * len, f = len * 0.28;
          shafts.moveTo(tx, ty); shafts.lineTo(tx - d.x * f - d.y * f * 0.7, ty - d.y * f + d.x * f * 0.7);
          shafts.moveTo(tx, ty); shafts.lineTo(tx - d.x * f + d.y * f * 0.7, ty - d.y * f - d.x * f * 0.7);
          const hl = len * 0.22;
          heads.moveTo(p.x + d.x * hl * 0.6, p.y + d.y * hl * 0.6);
          heads.lineTo(p.x - d.x * hl - d.y * hl * 0.45, p.y - d.y * hl + d.x * hl * 0.45);
          heads.lineTo(p.x - d.x * hl + d.y * hl * 0.45, p.y - d.y * hl - d.x * hl * 0.45);
          heads.closePath();
          break;
        }
        case 'shot': {
          const len = hp * 0.5;
          trails.moveTo(p.x - d.x * len, p.y - d.y * len); trails.lineTo(p.x, p.y);
          break;
        }
        case 'drone': {
          // A painted combat drone (the swarm's own art), bobbing on its way in, turned toward its target.
          const img = airPlanImage('drone');
          if (!img) break;
          const bob = Math.sin(k * 18) * hp * 0.03, w = hp * 0.55, h = w * img.naturalHeight / img.naturalWidth;
          ctx.globalAlpha = 1;
          ctx.drawImage(img, p.x - w / 2, p.y + bob - h / 2, w, h);
          break;
        }
        case 'bombs': {
          // Bombs fall from the aircraft overhead: a short drop, nose down (the aircraft is drawn once, below).
          const top = { x: to.x, y: to.y - hp * AIR_HEIGHT };
          const fall = k * k;
          const bx = top.x + (to.x - top.x) * fall, by = top.y + (to.y - top.y) * fall;
          ctx.globalAlpha = 1; ctx.fillStyle = INK; ctx.strokeStyle = VELLUM; ctx.lineWidth = 2;
          ctx.beginPath(); ctx.ellipse(bx, by, hp * 0.04, hp * 0.09, -0.35, 0, Math.PI * 2); ctx.stroke(); ctx.fill();
          break;
        }
        default: {
          // Stone, ball or shell: the body, its shadow on the ground, and a smoke trail behind a shell.
          const r = hp * (s.kind === 'stone' ? 0.15 : s.kind === 'ball' ? 0.11 : 0.09);
          const gx = from.x + (to.x - from.x) * k, gy = from.y + (to.y - from.y) * k + hp * 0.25;
          ctx.globalAlpha = 0.12 + 0.18 * k; ctx.fillStyle = INK;
          ctx.beginPath(); ctx.ellipse(gx, gy, r * 1.2, r * 0.45, 0, 0, Math.PI * 2); ctx.fill();
          if (s.kind === 'shell') {
            for (let i = 1; i < 9; i++) {
              const ki = i / 9;
              if (ki > k) break;
              const born = lj + ki * fj;
              const q = arcAt(from, to, height, ki).p;
              const age = Math.min(1, (t - born) / 650);
              puff(ctx, 'smoke', s.seed + i * 3, q.x, q.y - age * hp * 0.3, hp * (0.08 + age * 0.16), 0.7 * (1 - age));
            }
          } else if (s.kind === 'ball') {
            const q0 = arcAt(from, to, height, Math.max(0, k - 0.25)).p, q1 = arcAt(from, to, height, Math.max(0, k - 0.12)).p;
            trails.moveTo(q0.x, q0.y); trails.quadraticCurveTo(q1.x, q1.y, p.x, p.y);
          }
          ctx.globalAlpha = 1;
          ctx.fillStyle = s.kind === 'stone' ? INK2 : INK;
          ctx.beginPath(); ctx.arc(p.x, p.y, r, 0, Math.PI * 2);
          ctx.lineWidth = 2.5; ctx.strokeStyle = VELLUM; ctx.stroke();
          ctx.fill();
          ctx.lineWidth = 1; ctx.strokeStyle = INK; ctx.stroke();
          if (s.kind === 'stone') {
            // Hatching on the stone, turning as it flies.
            const spin = k * 9;
            ctx.strokeStyle = VELLUM; ctx.lineWidth = 0.8;
            ctx.beginPath(); ctx.moveTo(p.x + Math.cos(spin) * r * 0.6, p.y + Math.sin(spin) * r * 0.6); ctx.lineTo(p.x - Math.cos(spin + 0.4) * r * 0.5, p.y - Math.sin(spin + 0.4) * r * 0.5); ctx.stroke();
          }
        }
      }
      // An ink trail behind an arrow or bolt, fading toward the bow.
      if (s.kind === 'arrows' || s.kind === 'bolts') {
        const k0 = Math.max(0, k - 0.22);
        const q0 = arcAt(from, to, height, k0).p, q1 = arcAt(from, to, height, (k0 + k) / 2).p;
        trails.moveTo(q0.x, q0.y); trails.quadraticCurveTo(q1.x, q1.y, p.x, p.y);
      }
    } else if ((s.kind === 'arrows' || s.kind === 'bolts') && t - (lj + fj) < 420) {
      // Landed: the shaft stands in the ground a moment, then the frame moves on.
      const age = (t - (lj + fj)) / 420;
      const { d } = arcAt(from, to, height, 1);
      const len = Math.max(4, hp * 0.17);
      ctx.globalAlpha = 1 - age;
      ctx.strokeStyle = INK; ctx.lineWidth = 1.2;
      ctx.beginPath(); ctx.moveTo(to.x - d.x * len, to.y - d.y * len); ctx.lineTo(to.x, to.y); ctx.stroke();
    }
  }
  if (s.kind === 'bombs') drawAircraft(ctx, s, t, b, hp);
  // Impact.
  const heavy = HEAVY.has(s.kind) || (s.atCity && s.kind !== 'arrows');
  if (heavy) {
    flash(ctx, t, impactT, b.x, b.y - hp * 0.15, hp * 0.32, 170, s.hit);
    rays(ctx, s.seed, t, impactT, b.x, b.y - hp * 0.15, hp * 0.75, 12, s.hit ? 3 : 0, 380);
    ring(ctx, t, impactT, b.x, b.y, hp * 0.3, hp * 1.4, 520, INK, 2);
    cloud(ctx, 'dust', s.seed + 5, t, impactT, 1000, b.x, b.y, 5, hp * 0.14, hp * 0.42, hp * 0.7, { x: 0, y: -hp * 0.45 }, 0.95);
    debris(ctx, s.seed, t, impactT, b.x, b.y - hp * 0.1, hp * 0.9, 12, s.atCity ? INK2 : INK);
    if (s.kind === 'shell' || s.kind === 'bombs') cloud(ctx, 'soot', s.seed + 9, t, impactT + 60, 1500, b.x, b.y - hp * 0.2, 5, hp * 0.18, hp * 0.6, hp * 0.25, { x: hp * 0.1, y: -hp * 1.1 }, 0.85);
  } else {
    cloud(ctx, 'dust', s.seed + 5, t, impactT, 700, b.x, b.y + hp * 0.1, s.kind === 'shot' ? 3 : 4, hp * 0.1, hp * 0.32, hp * 0.4, { x: sx * hp * 0.15, y: -hp * 0.12 }, 0.8);
    if (s.hit) rays(ctx, s.seed, t, impactT, b.x, b.y - hp * 0.25, hp * 0.42, 5, 2, 260);
  }
}

/**
 * The strike's aircraft: its painted plan-view sprite (`assets/units/air`, the same aircraft as its miniature) crossing
 * high over the target on its way from its base, its shadow sliding over the ground below, dropping its bombs as it passes.
 */
const AIR_HEIGHT = 1.8;
/** How wide each aircraft flies, in hexes, along its longer side (a Bomber's span against a Fighter's). */
const AIR_SPAN: Record<NonNullable<FxShot['craft']>, number> = { Biplane: 1.0, Fighter: 1.1, JetFighter: 1.1, Bomber: 1.6, JetBomber: 1.6, StealthFighter: 1.2, StealthBomber: 1.7 };
/** One clock for the painted pass and for hiding the selected hangar miniature while it flies. */
export function aircraftPassProgress(s: FxShot, t: number): number | null {
  if (s.kind !== 'bombs') return null;
  const pass = s.flight * 2.6;
  if (pass <= 0) return null;
  const k = (t - (s.launch - pass / 2)) / pass;
  return k >= 0 && k <= 1 ? k : null;
}
function drawAircraft(ctx: CanvasRenderingContext2D, s: FxShot, t: number, b: Pt, hp: number) {
  // Over the target when it releases (the launch), so the bombs fall straight down onto it.
  const k = aircraftPassProgress(s, t);
  if (k == null) return;
  const a0 = s.from, dx0 = s.to.x - a0.x, dy0 = s.to.y - a0.y, d0 = Math.hypot(dx0, dy0) || 1;
  const ux = dx0 / d0, uy = dy0 / d0;
  const run = hp * 5.5;
  const cx = b.x + ux * run * (k - 0.5), cy = b.y - hp * AIR_HEIGHT + uy * run * (k - 0.5);
  const fade = Math.min(1, k * 6, (1 - k) * 6);
  const ang = Math.atan2(uy, ux);
  // Shadow on the ground.
  ctx.globalAlpha = 0.16 * fade; ctx.fillStyle = INK;
  ctx.beginPath(); ctx.ellipse(cx + hp * 0.5, cy + hp * AIR_HEIGHT, hp * 0.6, hp * 0.18, ang, 0, Math.PI * 2); ctx.fill();
  const craft = s.craft ?? 'Bomber';
  const img = airPlanImage(craft);
  if (!img) return;
  const sc = hp * AIR_SPAN[craft] / Math.max(img.naturalWidth, img.naturalHeight);
  ctx.save();
  ctx.translate(cx, cy); ctx.rotate(ang);
  ctx.globalAlpha = fade;
  ctx.drawImage(img, -img.naturalWidth * sc / 2, -img.naturalHeight * sc / 2, img.naturalWidth * sc, img.naturalHeight * sc);
  ctx.restore();
}

function drawClash(ctx: CanvasRenderingContext2D, c: FxClash, t: number, v: FxView, stepMs: number) {
  const a = v.toScreen(c.from), b = v.toScreen(c.to);
  const hp = v.hexPx;
  const p = { x: a.x + (b.x - a.x) * 0.62, y: a.y + (b.y - a.y) * 0.62 - hp * 0.2 };
  if (v.far) { ring(ctx, t, c.t, p.x, p.y, 2, 10, 420, c.hit ? RED : INK, 1.6); return; }
  const dx = b.x - a.x, dy = b.y - a.y, d = Math.hypot(dx, dy) || 1;
  // Riders kick up a dust trail on the charge.
  if (c.mounted) cloud(ctx, 'dust', c.seed + 3, t, c.t - stepMs * 0.45, 900, a.x + dx * 0.25, a.y + dy * 0.25 + hp * 0.25, 4, hp * 0.12, hp * 0.4, hp * 0.3, { x: -dx / d * hp * 0.3, y: -hp * 0.15 }, 0.75);
  // Muskets and rifles volley as the lines close.
  if (c.guns) {
    const m = { x: a.x + dx * 0.2, y: a.y + dy * 0.2 - hp * 0.2 };
    flash(ctx, t, c.t - stepMs * 0.3, m.x, m.y, hp * 0.14, 100, false);
    cloud(ctx, 'smoke', c.seed + 4, t, c.t - stepMs * 0.3, 1000, m.x, m.y, 3, hp * 0.1, hp * 0.4, hp * 0.3, { x: dx / d * hp * 0.2, y: -hp * 0.35 }, 0.85);
  }
  flash(ctx, t, c.t, p.x, p.y, hp * 0.26, 150, c.hit);
  rays(ctx, c.seed, t, c.t, p.x, p.y, hp * 0.62, 8, c.hit ? 2 : 0, 330);
  cloud(ctx, 'dust', c.seed, t, c.t, 950, p.x, p.y + hp * 0.25, c.atCity ? 6 : 4, hp * 0.14, hp * 0.5, hp * 0.55, { x: dx / d * hp * 0.2, y: -hp * 0.25 }, 0.9);
  debris(ctx, c.seed, t, c.t, p.x, p.y, hp * 0.7, c.atCity ? 9 : 5, c.atCity ? INK2 : INK);
}

function drawDeath(ctx: CanvasRenderingContext2D, d: FxDeath, t: number, v: FxView, at: Pt) {
  const p = v.toScreen(at);
  const hp = v.hexPx;
  if (v.far) { ring(ctx, t, d.t, p.x, p.y, 3, 12, 520, RED, 2); return; }
  rays(ctx, d.seed, t, d.t, p.x, p.y - hp * 0.2, hp * 0.5, 6, 3, 380);
  // Dust where it falls, as the figure topples.
  cloud(ctx, 'dust', d.seed + 2, t, d.t + 220, 900, p.x, p.y + hp * 0.3, 4, hp * 0.1, hp * 0.32, hp * 0.45, { x: 0, y: -hp * 0.1 }, 0.9);
  cloud(ctx, 'soot', d.seed, t, d.t + 80, 1500, p.x, p.y - hp * 0.1, 5, hp * 0.12, hp * 0.5, hp * 0.22, { x: hp * 0.08, y: -hp * 0.95 }, 0.8);
}

function drawCapture(ctx: CanvasRenderingContext2D, c: FxCapture, t: number, v: FxView) {
  const p = v.toScreen(c.at);
  const hp = v.hexPx;
  if (v.far) { ring(ctx, t, c.t, p.x, p.y, 4, 18, 800, v.colorOf(c.owner), 2.5); return; }
  if (c.razed) {
    cloud(ctx, 'soot', c.city + 31, t, c.t, CAPTURE_MS * 1.4, p.x, p.y - hp * 0.2, 9, hp * 0.2, hp * 0.75, hp * 0.5, { x: hp * 0.2, y: -hp * 1.6 }, 0.9);
    debris(ctx, c.city, t, c.t, p.x, p.y, hp, 10, RED);
    if (c.orderedRaze && t >= c.t && t < c.t + CAPTURE_MS) {
      ctx.save();
      ctx.globalAlpha = Math.min(1, (t - c.t) / 180, (c.t + CAPTURE_MS - t) / 300);
      ctx.font = `bold ${Math.max(12, hp * 0.32)}px serif`;
      ctx.textAlign = 'center';
      ctx.textBaseline = 'middle';
      ctx.lineWidth = 3;
      ctx.strokeStyle = VELLUM;
      ctx.strokeText('Razed', p.x, p.y - hp * 0.9);
      ctx.fillStyle = INK;
      ctx.fillText('Razed', p.x, p.y - hp * 0.9);
      ctx.restore();
    }
    return;
  }
  // The city falls: a shock of ink rings (the taker's colour inside) and dust off the walls. The gilt sunburst is the
  // capture celebration's own (celebrate/mapEffect.ts), which now waits for this moment.
  ring(ctx, t, c.t, p.x, p.y, hp * 0.6, hp * 2.1, 950, INK, 2.6);
  ring(ctx, t, c.t + 140, p.x, p.y, hp * 0.5, hp * 1.6, 900, v.colorOf(c.owner), 3.2);
  cloud(ctx, 'dust', c.city + 21, t, c.t, 1300, p.x, p.y + hp * 0.2, 10, hp * 0.18, hp * 0.6, hp * 1.1, { x: 0, y: -hp * 0.2 }, 0.85);
}

/** Every effect of the plan at `t` ms (callers skip this once `t` is past the plan's total). */
export function drawCombatFx(ctx: CanvasRenderingContext2D, plan: CombatPlan, t: number, v: FxView, deathAt: (d: FxDeath) => Pt) {
  const m = v.hexPx * 3;
  const on = (p: Pt) => { const s = v.toScreen(p); return s.x > -m && s.x < v.w + m && s.y > -m && s.y < v.h + m; };
  ctx.save();
  ctx.lineCap = 'round';
  ctx.lineJoin = 'round';
  const shafts = new Path2D(), heads = new Path2D(), trails = new Path2D();
  for (const s of plan.shots) {
    if (t < s.launch - (s.kind === 'bombs' ? s.flight : 50) || t > s.launch + s.flight + 1700) continue;
    if (!on(s.from) && !on(s.to)) continue;
    drawShot(ctx, s, t, v, shafts, heads, trails);
  }
  for (const c of plan.clashes) {
    if (t < c.t - plan.stepMs || t > c.t + 1300 || (!on(c.from) && !on(c.to))) continue;
    drawClash(ctx, c, t, v, plan.stepMs);
  }
  for (const d of plan.deaths) {
    if (t < d.t || t > d.t + 2000) continue;
    const at = deathAt(d);
    if (on(at)) drawDeath(ctx, d, t, v, at);
  }
  for (const c of plan.captures) if (t >= c.t && t < c.t + CAPTURE_MS * 1.5 && on(c.at)) drawCapture(ctx, c, t, v);
  // Missiles in flight, all in three strokes: trails under shafts under heads.
  ctx.globalAlpha = 0.45; ctx.strokeStyle = INK; ctx.lineWidth = 1.6; ctx.stroke(trails);
  ctx.globalAlpha = 1; ctx.strokeStyle = VELLUM; ctx.lineWidth = 3.4; ctx.stroke(shafts);
  ctx.strokeStyle = INK; ctx.lineWidth = 1.6; ctx.stroke(shafts);
  ctx.fillStyle = INK; ctx.fill(heads);
  ctx.restore();
}

const FLAME = 'rgb(176,98,46)';

/**
 * A damaged city burns for the whole turn, not only while the blows land: engraved smoke columns that rise and drift,
 * fires at their feet once it is badly hurt, and at 0 HP a breach (rubble spilled from the wall and a heavier pall).
 * `damage` is 0..1 (1 = 0 HP). A pure function of the clock, so a still frame never looks dead.
 */
/**
 * Everything drawSiegeAmbient can paint for a city at `at` (CSS px), with room to spare: rubble below and
 * left of the centre, puffs rising up to about 1.9 hexes and drifting right with the wind as they swell.
 */
export function siegeAmbientBox(at: Pt, hp: number): { x: number; y: number; w: number; h: number } {
  return { x: at.x - hp * 1.2, y: at.y - hp * 2.8, w: hp * 3.2, h: hp * 3.7 };
}

export function drawSiegeAmbient(ctx: CanvasRenderingContext2D, at: Pt, damage: number, seed: number, t: number, hp: number) {
  if (damage <= 0) return;
  const p = at;
  const breached = damage >= 1;
  const columns = 1 + (damage > 0.5 ? 1 : 0) + (breached ? 1 : 0);
  ctx.save();
  if (breached) {
    // The breach: rubble spilled out of the wall on its front, under the smoke.
    ctx.fillStyle = INK2; ctx.strokeStyle = INK; ctx.lineWidth = 1;
    for (let i = 0; i < 9; i++) {
      const a = Math.PI * (0.55 + 0.5 * rnd(seed, i + 300)), d = hp * (0.42 + 0.22 * rnd(seed, i + 310));
      const r = hp * (0.035 + 0.035 * rnd(seed, i + 320));
      ctx.globalAlpha = 0.95;
      ctx.beginPath(); ctx.ellipse(p.x + Math.cos(a) * d, p.y + Math.sin(a) * d * 0.55 + hp * 0.15, r * 1.3, r, rnd(seed, i + 330) * 3, 0, Math.PI * 2);
      ctx.fill(); ctx.stroke();
    }
  }
  for (let c = 0; c < columns; c++) {
    const ox = (rnd(seed, c * 7 + 1) - 0.5) * hp * 0.8, oy = (rnd(seed, c * 7 + 2) - 0.3) * hp * 0.35;
    const bx = p.x + ox, by = p.y + oy;
    if (damage > 0.35) {
      // Fire: three flame tongues, flickering, ochre with the accent red at the heart.
      for (let i = 0; i < 3; i++) {
        const fx = bx + (i - 1) * hp * 0.07, flick = 0.75 + 0.25 * Math.sin(t / 85 + i * 2.1 + c) + 0.1 * Math.sin(t / 37 + i);
        const fh = hp * (0.22 + 0.08 * (i === 1 ? 1 : 0)) * flick * (0.7 + 0.3 * damage), fw = hp * 0.07;
        for (const [color, scale] of [[FLAME, 1], [RED, 0.55]] as const) {
          ctx.globalAlpha = color === RED ? 0.9 : 0.85;
          ctx.fillStyle = color;
          ctx.beginPath();
          ctx.moveTo(fx - fw * scale, by);
          ctx.quadraticCurveTo(fx - fw * scale * 0.9, by - fh * scale * 0.55, fx + Math.sin(t / 120 + i) * fw * 0.4, by - fh * scale);
          ctx.quadraticCurveTo(fx + fw * scale * 0.9, by - fh * scale * 0.55, fx + fw * scale, by);
          ctx.closePath(); ctx.fill();
        }
        ctx.globalAlpha = 0.8; ctx.strokeStyle = INK; ctx.lineWidth = 0.8; ctx.stroke();
      }
    }
    // Smoke: puffs born in turn at the column's foot, rising, swelling and drifting with the wind.
    const n = 5, life = 3400 + rnd(seed, c + 40) * 800;
    for (let i = 0; i < n; i++) {
      const ph = ((t / life) + i / n + rnd(seed, c * 11 + i) * 0.05) % 1;
      const x = bx + ph * hp * 0.7 + Math.sin(ph * 5 + c) * hp * 0.06;
      const y = by - hp * 0.2 - ph * hp * 1.7;
      const alpha = Math.sin(Math.PI * Math.min(1, ph * 1.15)) * (0.45 + 0.45 * damage);
      puff(ctx, damage > 0.5 ? 'soot' : 'smoke', seed + c * 13 + i, x, y, hp * (0.1 + 0.28 * ph) * SIZE, alpha);
    }
  }
  ctx.restore();
}
