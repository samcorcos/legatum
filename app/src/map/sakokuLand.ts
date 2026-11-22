// A closed country's border wears a palisade, in gilt once its VP holds. Only explored land is
// drawn (`PlayerPublic.sakoku_turns`, `faction_goal_done`).
import type { Hex, PlayerView } from '../types';
import { ATLAS } from './style';
import { CORNERS, neighborIndex } from './grid';
import { hexAt } from '../hex';
import { rgba } from '../theme';

type Ctx = CanvasRenderingContext2D;
type S2 = (h: Hex) => { x: number; y: number };

const T = ATLAS.tone;
const P = ATLAS.pen;

/** Closed countries the viewer can see the vow of: empire id and whether its VP holds. */
function closedCountries(view: PlayerView): { id: number; holding: boolean }[] {
  return view.players.filter((p) => p.alive && p.sakoku_turns !== undefined && p.sakoku_turns !== null)
    .map((p) => ({ id: p.id, holding: p.faction_goal_done }));
}

/** Under banners and units: the palisade along each closed country's border. */
export function drawClosedLand(ctx: Ctx, view: PlayerView, s2: S2, hexPx: number, w: number, h: number) {
  const closed = closedCountries(view);
  if (!closed.length) return;
  const W = view.width, H = view.height;
  for (const c of closed) {
    const ink = c.holding ? T.gold : T.ink;
    ctx.save();
    ctx.lineCap = 'round';
    ctx.lineJoin = 'round';
    const rule = new Path2D(), ticks = new Path2D();
    for (let i = 0; i < view.tiles.length; i++) {
      const t = view.tiles[i];
      if (!t.explored || t.owner !== c.id) continue;
      const o = s2(hexAt(i, W));
      if (o.x < -hexPx * 2 || o.x > w + hexPx * 2 || o.y < -hexPx * 2 || o.y > h + hexPx * 2) continue;
      for (let e = 0; e < 6; e++) {
        const nb = neighborIndex(i, e, W, H);
        if (nb >= 0 && view.tiles[nb].explored && view.tiles[nb].owner === c.id) continue;
        const a = CORNERS[e], b = CORNERS[(e + 1) % 6];
        const ax = o.x + a[0] * hexPx, ay = o.y + a[1] * hexPx, bx = o.x + b[0] * hexPx, by = o.y + b[1] * hexPx;
        rule.moveTo(ax, ay);
        rule.lineTo(bx, by);
        const mx = (ax + bx) / 2, my = (ay + by) / 2, k = hexPx * 0.2;
        const dx = o.x - mx, dy = o.y - my, d = Math.hypot(dx, dy) || 1;
        ticks.moveTo(mx, my);
        ticks.lineTo(mx + (dx / d) * k, my + (dy / d) * k);
      }
    }
    ctx.strokeStyle = rgba(T.paper, 0.55);
    ctx.lineWidth = P.heavy + 1.6;
    ctx.stroke(rule);
    ctx.strokeStyle = rgba(ink, c.holding ? 0.95 : 0.8);
    ctx.lineWidth = P.line;
    ctx.stroke(rule);
    ctx.lineWidth = P.heavy;
    ctx.stroke(ticks);
    ctx.restore();
  }
}
