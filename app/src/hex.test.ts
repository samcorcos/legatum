import { describe, expect, it } from 'vitest';
import {
  DIRECTIONS,
  distance,
  edgeCorners,
  fromOffset,
  hexAt,
  hexCorner,
  hexToPixel,
  neighbors,
  pixelToHex,
  ring,
  tileIndex,
  toOffset,
  within,
} from './hex';

describe('hex math (mirrors hex.rs)', () => {
  it('offset round-trips for every tile of a 48x32 map', () => {
    for (let row = 0; row < 32; row++) {
      for (let col = 0; col < 48; col++) {
        expect(toOffset(fromOffset(col, row))).toEqual([col, row]);
      }
    }
  });

  it('matches known odd-r conversions', () => {
    expect(fromOffset(0, 0)).toEqual({ q: 0, r: 0 });
    expect(fromOffset(0, 1)).toEqual({ q: 0, r: 1 });
    expect(fromOffset(0, 2)).toEqual({ q: -1, r: 2 });
    expect(fromOffset(5, 3)).toEqual({ q: 4, r: 3 });
    expect(toOffset({ q: -1, r: 2 })).toEqual([0, 2]);
  });

  it('handles negative rows like Rust (& 1 on negatives)', () => {
    expect(toOffset(fromOffset(3, -1))).toEqual([3, -1]);
    expect(toOffset(fromOffset(-2, -3))).toEqual([-2, -3]);
  });

  it('tile index is row * width + col, -1 off map', () => {
    const w = 48;
    const h = 32;
    for (let i = 0; i < w * h; i += 37) expect(tileIndex(hexAt(i, w), w, h)).toBe(i);
    expect(tileIndex(fromOffset(48, 0), w, h)).toBe(-1);
    expect(tileIndex(fromOffset(0, 32), w, h)).toBe(-1);
    expect(tileIndex(fromOffset(-1, 3), w, h)).toBe(-1);
  });

  it('neighbors are at distance one, in core direction order', () => {
    const h = { q: 3, r: -2 };
    const ns = neighbors(h);
    expect(ns).toHaveLength(6);
    for (const n of ns) expect(distance(h, n)).toBe(1);
    expect(ns[0]).toEqual({ q: 4, r: -2 });
    expect(DIRECTIONS[1]).toEqual({ q: 1, r: -1 });
  });

  it('distance is symmetric and axial', () => {
    expect(distance({ q: 0, r: 0 }, { q: 3, r: -1 })).toBe(3);
    expect(distance({ q: 0, r: 0 }, { q: -2, r: 5 })).toBe(5);
    expect(distance({ q: 2, r: 1 }, { q: -1, r: -1 })).toBe(distance({ q: -1, r: -1 }, { q: 2, r: 1 }));
  });

  it('within and ring counts match the core', () => {
    const o = { q: 0, r: 0 };
    expect(within(o, 0)).toHaveLength(1);
    expect(within(o, 1)).toHaveLength(7);
    expect(within(o, 2)).toHaveLength(19);
    expect(ring(o, 2)).toHaveLength(12);
    // sorted by q then r
    const w = within(o, 1);
    expect(w[0]).toEqual({ q: -1, r: 0 });
  });

  it('pixel <-> hex round-trips', () => {
    for (let i = 0; i < 48 * 32; i += 13) {
      const h = hexAt(i, 48);
      const p = hexToPixel(h, 20);
      expect(pixelToHex(p.x + 3, p.y - 4, 20)).toEqual(h);
    }
  });

  it('edge corners sit halfway toward each neighbor', () => {
    const size = 10;
    for (let d = 0; d < 6; d++) {
      const [a, b] = edgeCorners(d);
      const [ax, ay] = hexCorner(0, 0, size, a);
      const [bx, by] = hexCorner(0, 0, size, b);
      const mid = { x: (ax + bx) / 2, y: (ay + by) / 2 };
      const n = hexToPixel(DIRECTIONS[d], size);
      expect(mid.x).toBeCloseTo(n.x / 2, 6);
      expect(mid.y).toBeCloseTo(n.y / 2, 6);
    }
  });
});
