import { describe, expect, it, vi } from 'vitest';
import { drawLens } from './drawLens';
import { HEX_SIZE } from './grid';

interface R { x0: number; y0: number; x1: number; y1: number }
const hit = (a: R, b: R) => a.x0 < b.x1 && a.x1 > b.x0 && a.y0 < b.y1 && a.y1 > b.y0;
describe('recommendation badges keep off the yield icons and the tokens where a clear spot exists', () => {
  const badges = (yieldRects: R[], unitBottom?: number) => {
    const rects: R[] = [];
    const cx = 400, cy = 300;
    const ctx = new Proxy({ fillRect: (x: number, y: number, w: number, h: number) => { if (w === 68) rects.push({ x0: x - cx, y0: y - cy, x1: x - cx + w, y1: y - cy + h }); } }, { get: (t, k) => (t as Record<PropertyKey, unknown>)[k] ?? vi.fn(), set: (t, k, v) => { (t as Record<PropertyKey, unknown>)[k] = v; return true; } }) as unknown as CanvasRenderingContext2D;
    drawLens(ctx, [{ index: 0, band: 'Great', rank: 1, faded: false }], { x: 0, y: 0, zoom: 0.9 }, 2 * cx, 2 * cy, 12, 1, 'atlas', 'labels', unitBottom === undefined ? new Map() : new Map([[0, unitBottom]]), [], () => 0, { yieldsOn: true, yieldRects: yieldRects.map((r) => ({ x: r.x0 + cx, y: r.y0 + cy, w: r.x1 - r.x0, h: r.y1 - r.y0 })) });
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
