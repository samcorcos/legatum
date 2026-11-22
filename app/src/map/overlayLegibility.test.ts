import { describe, expect, it, vi } from 'vitest';
import { drawLens, drawOwnerOverlay } from './drawLens';
import type { LensTile } from './lenses';
import { FAR_HEX_PX, HEX_SIZE } from './grid';
import { yieldCluster, yieldIconSize } from './yieldIcons';
import { medallionSlots, resourceDetail } from './resourceIcons';

vi.mock('../icons', () => ({ tintedIcon: () => null }));

// Overlays must stay readable over every lens: whatever a lens prints on a tile (its chip, its pips) may not land on the yield icons
// that the Yields overlay draws at the foot of the same tile. Rects are in screen px relative to the hex centre (down is +).
interface R { x0: number; y0: number; x1: number; y1: number }
const hit = (a: R, b: R) => a.x0 < b.x1 && a.x1 > b.x0 && a.y0 < b.y1 && a.y1 > b.y0;

/** The yield cluster, as yieldIcons.ts places it: bottom at 0.64 r, shrunk to 1.2 r wide, centred. */
function clusterRect(r: number, yields: { kind: 'food' | 'production' | 'gold' | 'science'; value: number }[]): R {
  let size = yieldIconSize(r), cluster = yieldCluster(yields, size);
  if (cluster.width > r * 1.2) { size = Math.max(size * 0.6, size * r * 1.2 / cluster.width); cluster = yieldCluster(yields, size); }
  return { x0: -cluster.width / 2, x1: cluster.width / 2, y0: r * 0.64 - cluster.height, y1: r * 0.64 };
}

/** Everything a lens draws on one tile in the labels pass, as rects around the hex centre. */
function lensMarks(tile: LensTile, zoom: number, yieldsOn: boolean, unitBottom?: number, avoid: { x: number; y: number; w: number; h: number }[] = []): R[] {
  const rects: R[] = [];
  const cx = 400, cy = 300, r = HEX_SIZE * zoom;
  const ctx = new Proxy({
    fillRect: (x: number, y: number, w: number, h: number) => { if (w === 68 || w === 50 || w === 28) return; rects.push({ x0: x - cx, y0: y - cy, x1: x - cx + w, y1: y - cy + h }); },
    arc: (x: number, y: number, rad: number) => { rects.push({ x0: x - cx - rad, y0: y - cy - rad, x1: x - cx + rad, y1: y - cy + rad }); },
  }, { get: (t, k) => (t as Record<PropertyKey, unknown>)[k] ?? vi.fn(), set: (t, k, v) => { (t as Record<PropertyKey, unknown>)[k] = v; return true; } }) as unknown as CanvasRenderingContext2D;
  // A camera that puts tile 0 at the canvas centre.
  const world = { x: 0, y: 0 };
  drawLens(ctx, [{ ...tile, index: 0 }], { x: world.x, y: world.y, zoom }, 2 * cx, 2 * cy, 12, 1, 'atlas', 'labels', unitBottom === undefined ? new Map() : new Map([[0, unitBottom]]), avoid, () => 0, { yieldsOn });
  void r;
  return rects;
}

const CLUSTERS = [
  [{ kind: 'food' as const, value: 3 }],
  [{ kind: 'food' as const, value: 3 }, { kind: 'production' as const, value: 3 }, { kind: 'gold' as const, value: 2 }],
  [{ kind: 'gold' as const, value: 4 }],
  [{ kind: 'production' as const, value: 2 }],
];
const LENS_TILES: LensTile[] = [
  { index: 0, tint: '#268064', label: '1', faded: false },
  { index: 0, tint: '#b27936', label: '2', faded: false },
  { index: 0, tint: '#555566', label: '×', faded: false },
  { index: 0, tint: '#bc3949', label: '×12', faded: false },
  { index: 0, tint: '#287f9d', label: '?', faded: false },
];
const ZOOMS = [0.5, 0.6, 0.75, 0.9, 1.2, 1.5, 2, 2.4];

it('keeps a Threat ×N label off a neighbouring city banner', () => {
  const banner = { x: 370, y: 285, w: 100, h: 28 };
  const marks = lensMarks(LENS_TILES[3], 1.25, false, undefined, [banner]);
  expect(marks.length).toBeGreaterThan(0);
  for (const mark of marks) expect(hit(mark, { x0: banner.x - 400, y0: banner.y - 300, x1: banner.x + banner.w - 400, y1: banner.y + banner.h - 300 })).toBe(false);
});

describe('yield icons stay clear of every lens chip on an empty tile', () => {
  it.each(ZOOMS)('at zoom %s', (zoom) => {
    const r = HEX_SIZE * zoom;
    if (r < 18) return; // zoomed out, yields are dots and the lens prints no pips
    for (const tile of LENS_TILES) for (const yields of CLUSTERS) {
      const cluster = clusterRect(r, yields);
      const marks = lensMarks(tile, zoom, true);
      expect(marks.length, `${tile.label} marks`).toBeGreaterThan(0);
      for (const m of marks) expect(hit(m, cluster), `${tile.label} ${JSON.stringify(m)} vs ${JSON.stringify(cluster)} at r=${r}`).toBe(false);
    }
  });

  it('without the overlay the chips keep their old place in the middle of the tile (so the change is only where it is needed)', () => {
    const chip = lensMarks(LENS_TILES[0], 0.9, false)[0];
    expect(chip.x0 + chip.x1).toBeCloseTo(0, 5); // centred
    expect(lensMarks(LENS_TILES[0], 0.9, true)[0].x1).toBeLessThan(0); // at the side when Yields is on
  });
});

describe('yield icons stay clear of a lens chip beside a token', () => {
  it.each(ZOOMS.filter((z) => z >= 0.6))('at zoom %s', (zoom) => {
    const r = HEX_SIZE * zoom, tw = Math.min(18, r * 0.5);
    for (const tile of LENS_TILES.filter((t) => t.label)) for (const yields of CLUSTERS.slice(0, 2)) {
      const cluster = clusterRect(r, yields);
      // The cluster's lower half may sit under the token by design; the chip must not add to that.
      for (const m of lensMarks(tile, zoom, true, tw + 4)) expect(hit(m, cluster), `${tile.label} at r=${r}`).toBe(false);
    }
  });
});

describe('Resources overlay rings stay clear of the yield icons', () => {
  it.each(ZOOMS)('at zoom %s', (zoom) => {
    const r = HEX_SIZE * zoom;
    if (resourceDetail(r) !== 'medallions') return;
    const { dy, radius } = medallionSlots(r, false).top;
    const ring = radius + 9.1 - 0.5; // the outermost mark: the dashed "not improved yet" ring, drawn 8.2 px out when the overlay rings the medallion (half a pixel of slack at the smallest zoom)
    for (const yields of CLUSTERS) expect(dy + ring, `r=${r}`).toBeLessThanOrEqual(clusterRect(r, yields).y0 + 1e-9);
  });
});

describe('the Borders roundel stays clear of every lens chip and of the yield icons', () => {
  it.each(ZOOMS.filter((z) => HEX_SIZE * z >= FAR_HEX_PX))('at zoom %s', (zoom) => {
    const r = HEX_SIZE * zoom, cx = 400, cy = 300;
    const rects: R[] = [];
    const ctx = new Proxy({ roundRect: (x: number, y: number, w: number, h: number) => { rects.push({ x0: x - cx, y0: y - cy, x1: x - cx + w, y1: y - cy + h }); } }, { get: (t, k) => (t as Record<PropertyKey, unknown>)[k] ?? vi.fn(), set: (t, k, v) => { (t as Record<PropertyKey, unknown>)[k] = v; return true; } }) as unknown as CanvasRenderingContext2D;
    drawOwnerOverlay(ctx, [{ index: 0, tint: '#a33', fill: 'owner', label: '3', faded: false, nearOnly: true }], { x: 0, y: 0, zoom }, 2 * cx, 2 * cy, 12, 1, 'atlas', 'labels', 1);
    expect(rects).toHaveLength(1);
    const roundel = rects[0];
    for (const yieldsOn of [false, true]) for (const tile of LENS_TILES.filter((t) => t.label)) for (const m of lensMarks(tile, zoom, yieldsOn)) expect(hit(roundel, m), `${tile.label} yields ${yieldsOn}`).toBe(false);
    for (const yields of CLUSTERS) expect(hit(roundel, clusterRect(r, yields))).toBe(false);
    // And it stays inside the hex's width.
    expect(roundel.x1).toBeLessThanOrEqual(r * 0.87 + 1e-9);
    expect(roundel.y0).toBeGreaterThanOrEqual(-r * 0.87);
  });
});

describe('recommendation badges keep off the yield icons and the tokens where a clear spot exists', () => {
  const badges = (yieldRects: R[], unitBottom?: number) => {
    const rects: R[] = [];
    const cx = 400, cy = 300;
    const ctx = new Proxy({ fillRect: (x: number, y: number, w: number, h: number) => { if (w === 68) rects.push({ x0: x - cx, y0: y - cy, x1: x - cx + w, y1: y - cy + h }); } }, { get: (t, k) => (t as Record<PropertyKey, unknown>)[k] ?? vi.fn(), set: (t, k, v) => { (t as Record<PropertyKey, unknown>)[k] = v; return true; } }) as unknown as CanvasRenderingContext2D;
    drawLens(ctx, [{ index: 0, tint: '', band: 'Great', rank: 1, faded: false }], { x: 0, y: 0, zoom: 0.9 }, 2 * cx, 2 * cy, 12, 1, 'atlas', 'labels', unitBottom === undefined ? new Map() : new Map([[0, unitBottom]]), [], () => 0, { yieldsOn: true, yieldRects: yieldRects.map((r) => ({ x: r.x0 + cx, y: r.y0 + cy, w: r.x1 - r.x0, h: r.y1 - r.y0 })) });
    return rects;
  };
  it('the Best spot badge moves off an icon cluster it would have covered', () => {
    const r = HEX_SIZE * 0.9;
    const open = badges([])[0];
    expect(open).toBeDefined();
    const covering: R = { x0: open.x0, x1: open.x1, y0: open.y0 - 2, y1: open.y1 + 2 };
    const moved = badges([covering])[0];
    expect(moved).toBeDefined();
    expect(hit(moved, covering)).toBe(false);
    void r;
  });
});
