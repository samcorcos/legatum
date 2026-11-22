import { describe, expect, it } from 'vitest';
import { CACHE_SLACK, MAX_CHUNKS, cacheLimit, evictionPlan, fallbackLevels, type CacheEntry } from './chunkCache';

const entry = (key: string, level: number, used: number, pinned = false): CacheEntry => ({ key, level, used, pinned });

describe('evictionPlan', () => {
  it('drops nothing at or under the limit', () => {
    expect(evictionPlan([entry('a', 3, 1), entry('b', 3, 2)], 2, 0)).toEqual([]);
  });

  it('drops exactly the excess, oldest first (the size shrinking mid-loop once halved it)', () => {
    const entries = Array.from({ length: 113 }, (_, i) => entry(`k${i}`, 4, 100 + i));
    expect(evictionPlan(entries, 110, 0)).toEqual(['k0', 'k1', 'k2']);
  });

  it('never drops a chunk the current view needs while older ones remain', () => {
    // The 8.9.2 thrash: a zoom-out kept the old level's chunks "fresh", so the view's own ring was evicted every frame.
    const view = Array.from({ length: 30 }, (_, i) => entry(`v${i}`, 4, 500, true));
    const old = Array.from({ length: 85 }, (_, i) => entry(`o${i}`, 7, 400 + (i % 5)));
    const dropped = evictionPlan([...view, ...old], 110, 0);
    expect(dropped).toHaveLength(5);
    expect(dropped.every((k) => k.startsWith('o'))).toBe(true);
  });

  it('keeps the stand-in level whatever its age', () => {
    const entries = [entry('ov', 0, 1), entry('a', 5, 2), entry('b', 5, 3)];
    expect(evictionPlan(entries, 2, 0)).toEqual(['a']);
  });

  it('evicts nothing it must keep when the view pins everything', () => {
    const entries = Array.from({ length: 6 }, (_, i) => entry(`p${i}`, 4, i, true));
    expect(evictionPlan(entries, 3, 0)).toEqual([]);
  });
});

describe('cacheLimit', () => {
  it('is the base for a small view and grows with a big one', () => {
    expect(cacheLimit(40)).toBe(MAX_CHUNKS);
    expect(cacheLimit(120)).toBe(120 + CACHE_SLACK);
  });
});

describe('fallbackLevels', () => {
  const OV = 0;
  const SI = 9;

  it('always puts the stand-in layer under a level, then the close cached levels, nearest on top', () => {
    expect(fallbackLevels(5, [6], OV, SI)).toEqual([SI, 6]);
    expect(fallbackLevels(5, [3, 4, 6, 7], OV, SI)).toEqual([SI, 3, 7, 4, 6]);
  });

  it('never leaves a view without a stand-in, however far the zoom jumped', () => {
    // The 8.9.2 follow-up: with no stand-in for a three-level zoom-out the view was a blank parchment hole.
    for (let level = 1; level < 9; level++) expect(fallbackLevels(level, [], OV, SI)).toEqual([SI]);
  });

  it('never stretches a chunk drawn for a much smaller scale over the view', () => {
    // A level 4 or more steps coarser has pixel-floored strokes (borders, grid, coast) far too fat once stretched.
    expect(fallbackLevels(7, [3, 4, 5], OV, SI)).toEqual([SI, 4, 5]);
  });

  it('lets a finer level shrink under a zoom-out at any distance, since only coarser chunks look fat', () => {
    expect(fallbackLevels(4, [7], OV, SI)).toEqual([SI, 7]);
    expect(fallbackLevels(2, [8, 5], OV, SI)).toEqual([SI, 8, 5]);
    expect(fallbackLevels(5, [8, 1], OV, SI)).toEqual([SI, 8]);
  });

  it('covers a quick zoom, where several levels are each partly filled', () => {
    expect(fallbackLevels(7, [5, 6], OV, SI)).toEqual([SI, 5, 6]);
  });

  it('has no stand-in for the overview level itself, and never the level being drawn', () => {
    expect(fallbackLevels(0, [1], OV, SI)).toEqual([1]);
    expect(fallbackLevels(4, [4], OV, SI)).toEqual([SI]);
    expect(fallbackLevels(4, [SI, -1], OV, SI)).toEqual([SI]);
  });
});
