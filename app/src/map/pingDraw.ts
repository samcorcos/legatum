// A teammate's map ping: an ink ring in the pinger's empire colour on a vellum halo, with a centre dot. The first time a
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
    const r = Math.max(9, hexPx * 0.6);
    ctx.save();
    ctx.lineCap = 'round';
    if (!still) {
      for (let k = 0; k < WAVES; k++) {
        const phase = (m.age - k * WAVE_GAP) / WAVE_MS;
        if (phase <= 0 || phase >= 1) continue;
        ctx.globalAlpha = (1 - phase) * 0.75;
        ctx.beginPath();
        ctx.arc(c.x, c.y, r + phase * hexPx * 0.9, 0, Math.PI * 2);
        ctx.strokeStyle = m.color;
        ctx.lineWidth = 2.5;
        ctx.stroke();
      }
    }
    ctx.globalAlpha = 1;
    for (const [color, width] of [[halo, 6], [m.color, 3]] as const) {
      ctx.beginPath();
      ctx.arc(c.x, c.y, r, 0, Math.PI * 2);
      ctx.strokeStyle = color;
      ctx.lineWidth = width;
      ctx.stroke();
    }
    ctx.beginPath();
    ctx.arc(c.x, c.y, 3.5, 0, Math.PI * 2);
    ctx.fillStyle = halo;
    ctx.fill();
    ctx.beginPath();
    ctx.arc(c.x, c.y, 2.2, 0, Math.PI * 2);
    ctx.fillStyle = m.color;
    ctx.fill();
    ctx.restore();
  }
}
