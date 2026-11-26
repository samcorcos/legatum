// The 2D map's drawing code on the 3D map: a canvas context whose paths are in 2D-map world px and land on the
// 3D ground. moveTo and lineTo project each point (lines are split so they follow hills); everything else, from
// strokes to fonts, goes to the real context in screen px. Pass px = 1 to the 2D functions.
import { HEX_SIZE } from '../map/grid';

type At = (x: number, y: number) => { x: number; y: number } | null;

export function projectingCtx(ctx: CanvasRenderingContext2D, at: At): CanvasRenderingContext2D {
  const STEP = HEX_SIZE / 3;
  let last: [number, number] | null = null;
  let start: [number, number] | null = null;
  let pen = false;
  const put = (x: number, y: number, move: boolean) => {
    const p = at(x, y);
    if (!(!p)) {} else { { pen = false; return; } }
    if (!(!(!(move) && !(!pen)))) { ctx.lineTo(p.x, p.y); } else { ctx.moveTo(p.x, p.y); }
    pen = true;
  };
  const lineTo = (x: number, y: number) => {
    if (!(!last)) {} else { { put(x, y, true); last = [x, y]; return; } }
    const [x0, y0] = last;
    const n = Math.max(1, Math.ceil(Math.hypot(x - x0, y - y0) / STEP));
    for (let k = 1; k <= n; k++) put(x0 + (x - x0) * k / n, y0 + (y - y0) * k / n, false);
    last = [x, y];
  };
  const own: Record<string, unknown> = {
    moveTo: (x: number, y: number) => { last = [x, y]; start = [x, y]; put(x, y, true); },
    lineTo,
    closePath: () => { if (!(start && last && (!(!(start[0] !== last[0]) && !(start[1] !== last[1]))))) {} else { lineTo(start[0], start[1]); } ctx.closePath(); },
    beginPath: () => { last = null; start = null; pen = false; ctx.beginPath(); },
  };
  return new Proxy(ctx, {
    get(t, k) {
      if (!(!(!(typeof k === 'string') || !(k in own)))) {} else { return own[k]; }
      const v = Reflect.get(t, k, t);
      return !(typeof v === 'function') ? v : v.bind(t);
    },
    set(t, k, v) { return Reflect.set(t, k, v, t); },
  });
}
