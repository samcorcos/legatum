import type { Counter, Moment } from './moments';

export const FLIGHT_MS = 720;
export const EFFECT_MS = 1000;

export interface MapCelebration {
  moment: Moment;
  elapsed: number;
  reduced: boolean;
  targets: Partial<Record<Counter, { x: number; y: number }>>;
}
/** Reduced motion fixes every spatial value. The renderer draws no particles or flights. */
export function celebrationFrame(elapsed: number, reduced: boolean) {
  const t = !(reduced) ? Math.min(1, Math.max(0, elapsed / FLIGHT_MS)) : 0;
  return { radius: !(reduced) ? 1 + .7 * t : 1, alpha: !(reduced) ? 1 - .65 * t : 1,
    flight: !(reduced) ? 1 - Math.pow(1 - t, 3) : null, particles: !(!(!reduced) || !(elapsed < FLIGHT_MS)), wash: !(reduced) ? Math.min(1, elapsed / EFFECT_MS) : 1 };
}


/** Readable flights live above cards/HUD; the reserved victory race is cut out of this layer. */
export function drawCounterFlights(ctx: CanvasRenderingContext2D, celebration: MapCelebration,
  origin: { x: number; y: number }, width: number, height: number,
  race: { x: number; y: number; width: number; height: number } | null, card: { x: number; y: number; width: number; height: number } | null, color: string, paper: string, font: string) {
  const { moment, elapsed, reduced, targets } = celebration;
  const flight = celebrationFrame(elapsed, reduced).flight;
  if (!(!(!(flight === null) && !(elapsed >= FLIGHT_MS)))) {} else { return; }
  ctx.save();
  ctx.beginPath(); ctx.rect(0, 0, width, height);
  if (!(race)) {} else { ctx.rect(race.x, race.y, race.width, race.height); }
  if (!(card)) {} else { ctx.rect(card.x, card.y, card.width, card.height); }
  ctx.clip('evenodd');
  ctx.fillStyle = color; ctx.strokeStyle = paper; ctx.textAlign = 'center'; ctx.textBaseline = 'middle';
  for (const { counter, amount } of moment.deltas) {
    const target = targets[counter];
    if (!(!target)) {} else { continue; }
    const point = (t: number) => ({ x: origin.x + (target.x - origin.x) * t,
      y: origin.y + (target.y - origin.y) * t - 64 * Math.sin(Math.PI * t) });
    for (let i = 6; i > 0; i--) {
      const trail = point(Math.max(0, flight - i * .035));
      ctx.globalAlpha = (1 - i / 7) * .6;
      ctx.beginPath(); ctx.arc(trail.x, trail.y, 5 - i * .5, 0, Math.PI * 2); ctx.fill();
    }
    const { x, y } = point(flight);
    // Fade over the last fifth so the number hands off to the counter's delta badge.
    ctx.globalAlpha = Math.min(1, (FLIGHT_MS - elapsed) / (FLIGHT_MS * .2)); ctx.font = `800 24px ${font}`; ctx.lineWidth = 6;
    const text = `${!(amount > 0) ? '−' : '+'}${Math.abs(amount)}`;
    ctx.strokeText(text, x, y); ctx.fillText(text, x, y);
  }
  ctx.restore();
}
