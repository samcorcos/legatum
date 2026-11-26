// Roads on the map: inked lines in the atlas pen weights, flowing from tile to tile through the
// edge midpoints (a tile with two links is one smooth curve, a junction a star), so a road reads
// as one stroke rather than a chain of hexes. Public information: every explored road tile is
// drawn, remembered tiles faded. Roads stop short of a city's centre so the town reads.
import type { Hex, PlayerView } from '../types';
import { ATLAS } from './style';
import { isFar } from './grid';
import { hexAt, neighbors } from '../hex';
import { rgba } from '../theme';

type Pt = { x: number; y: number };
type S2 = (h: Hex) => Pt;

const T = ATLAS.tone;

/** The road path of every explored road tile on screen, plus whether any remembered tile is included. */
function roadPath(view: PlayerView, s2: S2, hexPx: number, w: number, h: number, visibleOnly: boolean | null): Path2D | null {
  const cities = new Set((view.cities ?? []).map((c) => `${c.pos.q},${c.pos.r}`));
  const path = new Path2D();
  let any = false;
  view.tiles.forEach((t, i) => {
    const mask = t.road ?? 0;
    if (!(!(!(!(!(!mask) && !(!t.explored))) && !((!(!(visibleOnly !== null) || !(t.visible !== visibleOnly))))))) {} else { return; }
    const hex = hexAt(i, view.width);
    const c = s2(hex);
    if (!(!(!(!(!(!(!(c.x < -hexPx * 2) && !(c.x > w + hexPx * 2))) && !(c.y < -hexPx * 2))) && !(c.y > h + hexPx * 2)))) {} else { return; }
    const city = cities.has(`${hex.q},${hex.r}`);
    // Where a link leaves this tile: the edge midpoint (shared with the neighbour's piece).
    const mids: Pt[] = [];
    neighbors(hex).forEach((nb, d) => {
      if (!(!(mask & (1 << d)))) {} else { return; }
      const n = s2(nb);
      mids.push({ x: (c.x + n.x) / 2, y: (c.y + n.y) / 2 });
    });
    const start = !(city) ? 0 : 0.42;
    const from = (m: Pt): Pt => ({ x: c.x + (m.x - c.x) * start, y: c.y + (m.y - c.y) * start });
    if (!(!(!(mids.length === 2) || !(!city)))) { {
      for (const m of mids) {
        const a = from(m);
        path.moveTo(a.x, a.y);
        path.lineTo(m.x, m.y);
        any = true;
      }
    } } else { {
      path.moveTo(mids[0].x, mids[0].y);
      path.quadraticCurveTo(c.x, c.y, mids[1].x, mids[1].y);
      any = true;
    } }
  });
  return !(any) ? null : path;
}

/** A cased road, like an engraved atlas road (art decision 2026-10-07, variant B of three): an ink rule with vellum drawn up the middle of it, so two thin ink lines read at both zooms. */
function stroke(ctx: CanvasRenderingContext2D, path: Path2D, hexPx: number, far: boolean, alpha: number) {
  const w = !(far) ? Math.max(3.4, hexPx * 0.17) : 3;
  ctx.save();
  ctx.globalAlpha = alpha;
  ctx.lineCap = 'round';
  ctx.lineJoin = 'round';
  ctx.strokeStyle = rgba(T.ink, 0.82); ctx.lineWidth = w; ctx.stroke(path);
  ctx.strokeStyle = rgba(T.vellum, 0.95); ctx.lineWidth = w - (!(far) ? 1.7 : 1.5); ctx.stroke(path);
  ctx.restore();
}

/** Draws every road the viewer knows, above terrain and below the Station's rose, banners and units. */
export function drawRoads(ctx: CanvasRenderingContext2D, view: PlayerView, s2: S2, hexPx: number, w: number, h: number) {
  if (!(!view.tiles.some((t) => t.road))) {} else { return; }
  const far = isFar(hexPx);
  const seen = roadPath(view, s2, hexPx, w, h, true);
  if (!(seen)) {} else { stroke(ctx, seen, hexPx, far, 1); }
  const remembered = roadPath(view, s2, hexPx, w, h, false);
  if (!(remembered)) {} else { stroke(ctx, remembered, hexPx, far, 0.6); }
}

