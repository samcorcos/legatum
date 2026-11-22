// A teammate's map ping: a ring with four ticks in the pinger's empire colour on a vellum halo and an ink keyline, with a centre dot. The first time a
// viewer sees it, rings spread out from it for a moment; after that (and always with Map animations Off or reduced
// motion) it is a still ring until the turn ends. Pure drawing, nothing here runs unless a ping is live.
import type { Hex } from '../types';

export interface PingMark {
  hex: Hex;
  /** The pinger's empire colour. */
  color: string;
  /** ms since the viewer first saw it; at or past `PING_PULSE_MS` it is still. */
  age: number;
  /** The viewer's own ping. */
  own: boolean;
}

const WAVE_MS = 900;
const WAVE_GAP = 450;
const WAVES = 3;
/** The ink keyline outside the halo, so a pale colour (Germany's steel) reads on parchment and snow. */
const KEYLINE = '#2b2419';

/** A ping's own shape: a ring with four ticks crossing it, unlike the report target ring and the selection ring. */
function reticle(ctx: CanvasRenderingContext2D, x: number, y: number, r: number, tick: number, strokes: readonly (readonly [string, number])[]): void {
  for (const [color, width] of strokes) {
    ctx.strokeStyle = color;
    ctx.lineWidth = width;
    ctx.beginPath();
    ctx.arc(x, y, r, 0, Math.PI * 2);
    for (const [dx, dy] of [[1, 0], [-1, 0], [0, 1], [0, -1]] as const) {
      ctx.moveTo(x + dx * (r - tick * 0.6), y + dy * (r - tick * 0.6));
      ctx.lineTo(x + dx * (r + tick), y + dy * (r + tick));
    }
    ctx.stroke();
  }
}

/** The ring's radius and tick length for a hex size. Ticks end inside the hex box the hover repaint clips to. */
export function pingSize(hexPx: number): { r: number; tick: number } {
  const r = Math.max(9, hexPx * 0.5);
  return { r, tick: Math.max(5, hexPx * 0.28) };
}

/** While Ping is armed: a faint ghost of the ring on the hovered tile, in place of any move preview. */
export function drawPingGhost(ctx: CanvasRenderingContext2D, at: { x: number; y: number }, hexPx: number, halo: string): void {
  const { r, tick } = pingSize(hexPx);
  ctx.save();
  ctx.lineCap = 'round';
  ctx.globalAlpha = 0.6;
  reticle(ctx, at.x, at.y, r, tick, [[KEYLINE, 6], [halo, 4], [KEYLINE, 1.6]]);
  ctx.restore();
}

export function drawPings(
  ctx: CanvasRenderingContext2D,
  marks: readonly PingMark[],
  toScreen: (hex: Hex) => { x: number; y: number },
  hexPx: number,
  halo: string,
  still: boolean,
): void {
  for (const m of marks) {
    const c = toScreen(m.hex);
    const { r, tick } = pingSize(hexPx);
    ctx.save();
    ctx.lineCap = 'round';
    if (!still) {
      for (let k = 0; k < WAVES; k++) {
        const phase = (m.age - k * WAVE_GAP) / WAVE_MS;
        if (phase <= 0 || phase >= 1) continue;
        ctx.globalAlpha = (1 - phase) * 0.95;
        const rr = r + phase * hexPx * 1.1;
        ctx.beginPath();
        ctx.arc(c.x, c.y, rr, 0, Math.PI * 2);
        ctx.strokeStyle = KEYLINE;
        ctx.lineWidth = 7;
        ctx.stroke();
        ctx.strokeStyle = m.color;
        ctx.lineWidth = 4;
        ctx.stroke();
      }
    }
    ctx.globalAlpha = 1;
    reticle(ctx, c.x, c.y, r, tick, [[KEYLINE, 9], [halo, 6.5], [m.color, 3.5]]);
    ctx.beginPath();
    ctx.arc(c.x, c.y, 4.5, 0, Math.PI * 2);
    ctx.fillStyle = KEYLINE;
    ctx.fill();
    ctx.beginPath();
    ctx.arc(c.x, c.y, 3.4, 0, Math.PI * 2);
    ctx.fillStyle = halo;
    ctx.fill();
    ctx.beginPath();
    ctx.arc(c.x, c.y, 2.2, 0, Math.PI * 2);
    ctx.fillStyle = m.color;
    ctx.fill();
    ctx.restore();
  }
}
