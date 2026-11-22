// Where the compass rose sits. It belongs to the map, like the rose on an atlas plate: it hugs
// the map's lower-left corner (just outside the land) and, once that corner is off screen, rests
// in the viewport's lower-left corner inside safe margins (clear of the top bar). If a panel or
// rendered map label covers the hugging spot it retreats to that corner; if both are covered
// the caller fades it.

export interface Rect {
  x: number;
  y: number;
  w: number;
  h: number;
}

/** Half the rose's box, in CSS px (the ring is r 28, the N sits above it). */
export const COMPASS_R = 40;

export interface Bounds {
  minX: number;
  minY: number;
  maxX: number;
  maxY: number;
}

export interface CompassSafe {
  left: number;
  right: number;
  top: number;
  bottom: number;
}

const SAFE: CompassSafe = { left: 24, right: 24, top: 72, bottom: 24 };

/** The rose's center in screen px for a camera over world bounds `b`. */
export function compassAnchor(b: Bounds, cam: { x: number; y: number; zoom: number }, w: number, h: number, safe: CompassSafe = SAFE) {
  const R = COMPASS_R;
  const mapLeft = (b.minX - cam.x) * cam.zoom + w / 2;
  const mapBottom = (b.maxY - cam.y) * cam.zoom + h / 2;
  const clamp = (v: number, lo: number, hi: number) => (lo > hi ? (lo + hi) / 2 : Math.max(lo, Math.min(hi, v)));
  return {
    x: clamp(mapLeft - R - 12, safe.left + R, w - safe.right - R),
    y: clamp(mapBottom - R, safe.top + R, h - safe.bottom - R),
  };
}

/** Whether the rose centered at (x, y) touches any of the rects (padded by `pad`). */
export function compassCovered(x: number, y: number, rects: Rect[], pad = 8): boolean {
  const R = COMPASS_R;
  return rects.some((r) => x + R > r.x - pad && x - R < r.x + r.w + pad && y + R > r.y - pad && y - R < r.y + r.h + pad);
}

/** Avoid panel and rendered label rectangles in CSS screen pixels; `covered` means fade it. */
export function placeCompass(b: Bounds, cam: { x: number; y: number; zoom: number }, w: number, h: number, obstacles: Rect[], safe: CompassSafe = SAFE) {
  const hug = compassAnchor(b, cam, w, h, safe);
  if (!compassCovered(hug.x, hug.y, obstacles)) return { ...hug, covered: false };
  const corner = { x: safe.left + COMPASS_R, y: h - safe.bottom - COMPASS_R };
  if (!compassCovered(corner.x, corner.y, obstacles)) return { ...corner, covered: false };
  return { ...hug, covered: true };
}
