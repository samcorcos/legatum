// Idle life: a held frame of the map is never dead (trailer research, 2026-10-24: "something moves in every frame").
// Smoke curls from the chimneys of the bigger towns and light glints on the water now and then. Subtle by design: low
// contrast, slow, sparse, and only on tiles in view. The map loop redraws for it at IDLE_FPS, not every frame, only while
// something in view can move, and the Map animations toggle and reduced motion turn it off (MapCanvas).
import type { PlayerView } from '../types';
import { cloud } from './combatFxDraw';
import { rnd, type Pt } from './combatFx';
import { centerX, centerY } from './grid';
import { cityTier } from './cityArt';
import { tileIndex } from '../hex';

/** How often the map redraws for idle life alone. */
export const IDLE_FPS = 8;
const SMOKE_PERIOD = 3400;
const GLINT_PERIOD = 5200;
const GLINT_MS = 700;
const WATER = new Set(['Ocean', 'Coast']);

export interface IdleView { toScreen(p: Pt): Pt; hexPx: number; far: boolean; w: number; h: number; dpr: number }

/** Light on the water: a few short vellum strokes that brighten and fade on a handful of tiles at a time. */
/** Returns how many water tiles in view could glint (0: nothing for idle life to animate here). */
export function drawWaterGlints(ctx: CanvasRenderingContext2D, view: PlayerView, t: number, v: IdleView): number {
  if (v.far) return 0;
  let candidates = 0;
  const R = v.hexPx;
  ctx.save();
  ctx.setTransform(v.dpr, 0, 0, v.dpr, 0, 0);
  ctx.strokeStyle = '#fbf6ea';
  ctx.lineCap = 'round';
  ctx.lineWidth = Math.max(0.8, R * 0.045);
  for (let i = 0; i < view.tiles.length; i++) {
    const tile = view.tiles[i];
    if (!tile.visible || !tile.terrain || !WATER.has(tile.terrain)) continue;
    const cc = v.toScreen({ x: centerX(i, view.width), y: centerY(i, view.width) });
    if (cc.x < -R || cc.x > v.w + R || cc.y < -R || cc.y > v.h + R) continue;
    candidates++;
    const phase = rnd(i, 7) * GLINT_PERIOD;
    const dt = (t + phase) % GLINT_PERIOD;
    // About one tile in seven glints in any one cycle.
    if (dt > GLINT_MS || rnd(i, Math.floor((t + phase) / GLINT_PERIOD)) > 0.15) continue;
    const c = cc;
    const k = dt / GLINT_MS;
    ctx.globalAlpha = Math.sin(Math.PI * k) * 0.55;
    const x = c.x + (rnd(i, 11) - 0.5) * R * 0.9, y = c.y + (rnd(i, 13) - 0.5) * R * 0.7;
    const len = R * (0.16 + 0.12 * rnd(i, 17));
    ctx.beginPath();
    ctx.moveTo(x - len, y); ctx.quadraticCurveTo(x, y - len * 0.22, x + len, y);
    ctx.moveTo(x - len * 0.4, y + R * 0.09); ctx.lineTo(x + len * 0.5, y + R * 0.09);
    ctx.stroke();
  }
  ctx.restore();
  return candidates;
}

/** Chimney smoke over towns of city size and up: two or three engraved wisps rising and thinning, out of step town to town. */
/** Returns how many towns in view smoke (0: nothing for idle life to animate here). */
export function drawChimneySmoke(ctx: CanvasRenderingContext2D, view: PlayerView, t: number, v: IdleView, skip?: (tile: number) => boolean): number {
  if (v.far) return 0;
  let towns = 0;
  const R = v.hexPx;
  ctx.save();
  ctx.setTransform(v.dpr, 0, 0, v.dpr, 0, 0);
  for (const c of view.cities) {
    if (!c.visible || cityTier(c.population) === 'hamlet' || cityTier(c.population) === 'town') continue;
    const tile = tileIndex(c.pos, view.width, view.height);
    if (skip?.(tile)) continue;
    const p = v.toScreen({ x: centerX(tile, view.width), y: centerY(tile, view.width) });
    if (p.x < -R * 2 || p.x > v.w + R * 2 || p.y < -R * 3 || p.y > v.h + R) continue;
    towns++;
    const wisps = cityTier(c.population) === 'metropolis' ? 3 : 2;
    for (let k = 0; k < wisps; k++) {
      const seed = c.id * 31 + k * 7;
      const offset = rnd(seed, 1) * SMOKE_PERIOD;
      const cycle = Math.floor((t + offset) / SMOKE_PERIOD);
      const t0 = cycle * SMOKE_PERIOD - offset;
      const x = p.x + (rnd(seed, 2) - 0.5) * R * 0.9;
      const y = p.y - R * (0.35 + rnd(seed, 3) * 0.2);
      cloud(ctx, 'smoke', seed + cycle, t, t0, SMOKE_PERIOD * 0.8, x, y, 2, R * 0.05, R * 0.13, R * 0.08, { x: R * 0.25, y: -R * 0.7 }, 0.32);
    }
  }
  ctx.restore();
  return towns;
}
