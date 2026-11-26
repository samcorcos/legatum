// The tile pickers on the 3D map: the 2D map's own picker drawing (map/pickerDraw.ts, map/airOverlay.ts), run through
// a projecting context so its hexes and rings lie on the 3D ground. Price chips and ring labels stay in screen space.
import type { Hex, PlayerView } from '../types';
import type { ImproveOverlay } from '../map/render';
import type { AirOverlay } from '../map/airOverlay';
import { drawAirCandidates, drawAirLabels, drawAirOffscreen, drawAirRings, drawTileCandidates } from '../map/airOverlay';
import { drawImproveHot, drawImproveTiles, drawPlaceTiles, drawPriceChips, drawQueuedBuys, type PlaceMarks } from '../map/pickerDraw';
import { FONTS } from '../map/sprites';
import { MAP_STYLES } from '../map/style';
import type { Rect } from '../map/compass';
import { projectingCtx } from './projectCtx';
import type { MapScene } from './scene';

export interface Pickers3d {
  improveOverlay: ImproveOverlay | null;
  placeOverlay: PlaceMarks | null;
  airOverlay: AirOverlay | null;
  tileBuys: { tile: Hex; price: number }[];
}

export function drawPickers3d(ctx: CanvasRenderingContext2D, sc: MapScene, view: PlayerView, p: Pickers3d, own: string, avoid: Rect[]) {
  const { improveOverlay: I, placeOverlay: pl, airOverlay: air, tileBuys } = p;
  if (!I && !pl && !air && !tileBuys.length) return;
  const S = MAP_STYLES.atlas, font = FONTS.atlas.sans;
  const g = projectingCtx(ctx, (x, y) => sc.projectFlat(x, y));
  const w = ctx.canvas.clientWidth, h = ctx.canvas.clientHeight;
  const toScreen = (x: number, y: number) => sc.projectFlat(x, y) ?? { x: -1e4, y: -1e4 };
  if (air?.rings.length) {
    drawAirRings(g, air, view.width, view.height, own, 1);
    drawAirLabels(ctx, air, toScreen, own, font, { w, h }, avoid);
  }
  if (I) drawImproveTiles(g, I, own, S, 1);
  if (pl) drawPlaceTiles(g, pl, S, 1);
  if (I && (I.hover || I.pick)) drawImproveHot(g, I, own, 1);
  if (pl?.hover) drawTileCandidates(g, [pl.hover], pl.hover, S.tone.gold, 1, 0.3, 4.2);
  if (tileBuys.length) drawQueuedBuys(g, tileBuys, own, 1);
  const chips = [...(pl?.prices ?? []).map((t) => ({ ...t, queued: false })), ...tileBuys.map((t) => ({ ...t, affordable: true, queued: true }))];
  const hexPx = sc.hexPx();
  if (chips.length && hexPx >= 22) drawPriceChips(ctx, chips, (hx) => sc.project(hx, 0.02), hexPx, own, S, font);
  if (air?.pickKind) {
    drawAirCandidates(g, air, air.pickKind === 'strike' ? S.ink.attack : own, 1);
    drawAirOffscreen(ctx, air, toScreen, own, font, { w, h }, avoid);
  }
}
