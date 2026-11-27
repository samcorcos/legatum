import { describe, expect, it, vi } from 'vitest';
import { BONUS_TAG_FONT, bonusTagSize, COLLAPSE_AT, drawYieldIcons, foldedBonuses, YIELD_ORDER, yieldCluster, yieldDetail, yieldIconSize } from './yieldIcons';
import { centerX, centerY, HEX_SIZE } from './grid';
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

  it("folds a natural wonder's +25% into the cluster: after its yield's icons, or with the icon when the tile has none", () => {
    const tagged = yieldCluster([{ kind: 'food', value: 2 }, { kind: 'science', value: 1 }], 10, 'food');
    const plain = yieldCluster([{ kind: 'food', value: 2 }, { kind: 'science', value: 1 }], 10);
    const tag = tagged.marks.find((m) => m.tag);
    expect(tag).toMatchObject({ kind: 'food', tag: '+25%' });
    expect(tag!.tagIcon).toBeFalsy();
    // The tag sits after the food icons and before the science icon, and the cluster grows to hold it.
    expect(tag!.x).toBeGreaterThan(Math.max(...tagged.marks.filter((m) => m.kind === 'food' && !m.tag).map((m) => m.x)));
    expect(tag!.x).toBeLessThan(tagged.marks.find((m) => m.kind === 'science')!.x);
    expect(tagged.width).toBeGreaterThan(plain.width);
    expect(count(tagged.marks.filter((m) => !m.tag), 'food')).toBe(2);
    // No production on the tile: the tag carries the production icon itself, and no icon counts as a point.
    const none = yieldCluster([{ kind: 'food', value: 2 }], 10, 'production');
    expect(none.marks.filter((m) => m.kind === 'production')).toEqual([expect.objectContaining({ tag: '+25%', tagIcon: true })]);
  });

  it('keeps the folded pill readable: a fixed size, never shrunk with the icons', () => {
    expect(BONUS_TAG_FONT).toBeGreaterThanOrEqual(10);
    const small = yieldCluster([{ kind: 'food', value: 2 }], 7, 'food'), big = yieldCluster([{ kind: 'food', value: 2 }], 17, 'food');
    expect(small.height).toBeGreaterThanOrEqual(bonusTagSize(false).height);
    expect(big.width - yieldCluster([{ kind: 'food', value: 2 }], 17).width).toBeGreaterThanOrEqual(bonusTagSize(false).width);
  });

  it('folds the bonus only where the pill fits in the hex and no unit sits on the cluster', () => {
    const tile: LensTile = { index: 0, faded: false, yields: [{ kind: 'food', value: 2 }, { kind: 'production', value: 1 }], bonus: 'production' };
    const at = (zoom: number) => ({ x: centerX(0, 10), y: centerY(0, 10), zoom });
    // Close in, it fits; at a normal zoom the readable pill is wider than the hex allows, so the tile keeps its wonder pill.
    expect(foldedBonuses([tile], at(1.6), 800, 600, 10).has(0)).toBe(true);
    expect(foldedBonuses([tile], at(0.9), 800, 600, 10).has(0)).toBe(false);
    // Zoomed out to dots, nothing folds.
    expect(foldedBonuses([tile], at(0.4), 800, 600, 10).size).toBe(0);
    // A unit counter over the cluster: the pill (which clears or draws above counters) shows the bonus instead.
    const r = HEX_SIZE * 1.6;
    expect(foldedBonuses([tile], at(1.6), 800, 600, 10, [], [{ x: 400 - r * 0.4, y: 300 - r * 0.4, w: r * 0.8, h: r * 1.2 }]).has(0)).toBe(false);
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
    const tiles: LensTile[] = [{ index: 0, faded: false, yields: [{ kind: 'food', value: 2 }, { kind: 'production', value: 1 }] }];
    drawYieldIcons(ctx, tiles, { x: 0, y: 0, zoom: 0.9 }, 800, 600, 12, 2, 'atlas');
    expect(calls).not.toContain('fillRect');
    expect(calls).not.toContain('fillText');
    calls.length = 0;
    drawYieldIcons(ctx, tiles, { x: 0, y: 0, zoom: 0.28 }, 800, 600, 12, 2, 'atlas');
    expect(calls).toEqual([]);
  });
});
