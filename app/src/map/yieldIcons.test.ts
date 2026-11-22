import { describe, expect, it, vi } from 'vitest';
import { COLLAPSE_AT, drawYieldIcons, YIELD_ORDER, yieldCluster, yieldDetail, yieldIconSize } from './yieldIcons';
import type { LensTile } from './lenses';

vi.mock('../icons', () => ({ tintedIcon: () => null }));
const count = (marks: { kind: string }[], kind: string) => marks.filter((m) => m.kind === kind).length;

describe('yieldCluster', () => {
  it('repeats each icon once per point, in the fixed yield order', () => {
    const { marks } = yieldCluster([{ kind: 'science', value: 1 }, { kind: 'production', value: 3 }, { kind: 'food', value: 2 }, { kind: 'gold', value: 4 }], 10);
    expect(count(marks, 'food')).toBe(2);
    expect(count(marks, 'production')).toBe(3);
    expect(count(marks, 'gold')).toBe(4);
    expect(count(marks, 'science')).toBe(1);
    expect(marks.every((m) => m.numeral === undefined)).toBe(true);
    // Groups run left to right in YIELD_ORDER: every icon of a type sits right of the previous type's.
    const right = (k: string) => Math.max(...marks.filter((m) => m.kind === k).map((m) => m.x));
    const left = (k: string) => Math.min(...marks.filter((m) => m.kind === k).map((m) => m.x));
    for (let i = 1; i < YIELD_ORDER.length; i++) expect(left(YIELD_ORDER[i])).toBeGreaterThan(right(YIELD_ORDER[i - 1]));
  });

  it('collapses large values to one icon with a numeral', () => {
    const { marks } = yieldCluster([{ kind: 'food', value: COLLAPSE_AT }, { kind: 'production', value: 12 }, { kind: 'gold', value: COLLAPSE_AT - 1 }], 10);
    expect(marks.filter((m) => m.kind === 'food')).toEqual([expect.objectContaining({ numeral: COLLAPSE_AT })]);
    expect(marks.filter((m) => m.kind === 'production')).toEqual([expect.objectContaining({ numeral: 12 })]);
    expect(count(marks, 'gold')).toBe(COLLAPSE_AT - 1);
  });

  it('draws nothing for zero (or negative) yields', () => {
    expect(yieldCluster([], 10)).toEqual({ marks: [], width: 0, height: 0 });
    expect(yieldCluster([{ kind: 'food', value: 0 }, { kind: 'gold', value: -1 }], 10).marks).toEqual([]);
    expect(count(yieldCluster([{ kind: 'food', value: 0 }, { kind: 'production', value: 2 }], 10).marks, 'food')).toBe(0);
  });

  it('stays compact: stacks of 3 and 4 fold into two columns, centred on the tile', () => {
    const icon = 10;
    const four = yieldCluster([{ kind: 'food', value: 4 }], icon);
    expect(four.width).toBeLessThan(icon * 2);
    expect(four.height).toBeLessThan(icon * 2);
    const all = yieldCluster(YIELD_ORDER.slice(0, 4).map((kind) => ({ kind, value: 4 })), icon);
    const xs = all.marks.map((m) => m.x);
    expect(Math.min(...xs) - icon / 2).toBeCloseTo(-all.width / 2);
    expect(Math.max(...xs) + icon / 2).toBeCloseTo(all.width / 2);
    for (const m of all.marks) expect(Math.abs(m.y) + icon / 2).toBeLessThanOrEqual(all.height / 2 + 1e-9);
  });
});

describe('zoom levels', () => {
  it('shows icons at default zoom, dots when zoomed out, nothing at the widest view', () => {
    expect(yieldDetail(36 * 0.9)).toBe('icons');
    expect(yieldDetail(36 * 0.55)).toBe('icons');
    expect(yieldDetail(36 * 0.4)).toBe('dots');
    expect(yieldDetail(36 * 0.28)).toBe('none');
    expect(yieldIconSize(36 * 0.9)).toBeGreaterThanOrEqual(10);
    expect(yieldIconSize(36 * 2.4)).toBeLessThanOrEqual(17);
  });

  it('draws no chip or text for ordinary yields and nothing at all at far zoom', () => {
    const calls: string[] = [];
    const ctx = new Proxy({}, { get: (t, k) => (t as Record<PropertyKey, unknown>)[k] ?? ((..._: unknown[]) => { calls.push(String(k)); }), set: (t, k, v) => { (t as Record<PropertyKey, unknown>)[k] = v; return true; } }) as CanvasRenderingContext2D;
    const tiles: LensTile[] = [{ index: 0, tint: '#000', faded: false, yields: [{ kind: 'food', value: 2 }, { kind: 'production', value: 1 }] }];
    drawYieldIcons(ctx, tiles, { x: 0, y: 0, zoom: 0.9 }, 800, 600, 12, 2, 'atlas');
    expect(calls).not.toContain('fillRect');
    expect(calls).not.toContain('fillText');
    calls.length = 0;
    drawYieldIcons(ctx, tiles, { x: 0, y: 0, zoom: 0.28 }, 800, 600, 12, 2, 'atlas');
    expect(calls).toEqual([]);
  });
});
