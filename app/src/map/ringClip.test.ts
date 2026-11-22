import { describe, expect, it } from 'vitest';
import { ringClipRect } from './render';

describe('selection pulse repaint box', () => {
  it('covers the ring edge with a margin, in device pixels', () => {
    const box = ringClipRect({ x: 100, y: 80, r: 30 }, 2);
    expect(box).toEqual({ x: 138, y: 98, w: 124, h: 124 });
    expect(box.x).toBeLessThanOrEqual((100 - 30) * 2);
    expect(box.x + box.w).toBeGreaterThanOrEqual((100 + 30) * 2);
  });

  it('stays on the canvas when the ring is at the top-left edge', () => {
    const box = ringClipRect({ x: 5, y: 5, r: 30 }, 1);
    expect(box.x).toBe(0);
    expect(box.y).toBe(0);
    expect(box.w).toBeGreaterThan(0);
  });
});
