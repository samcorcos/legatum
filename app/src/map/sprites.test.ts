// @vitest-environment happy-dom
import { afterEach, describe, expect, it, vi } from 'vitest';
import { nameSprite, SPRITE_CACHE_LIMIT, spriteCacheSize } from './sprites';

afterEach(() => {
  vi.restoreAllMocks();
});

function stubCanvas2d() {
  const ctx = {
    scale() {},
    measureText: (s: string) => ({ width: s.length * 6 }),
    fillText() {},
    strokeText() {},
    save() {},
    restore() {},
    fillStyle: '',
    strokeStyle: '',
    lineWidth: 1,
    lineJoin: 'round',
    font: '',
    textBaseline: 'alphabetic',
  } as unknown as CanvasRenderingContext2D;
  vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockReturnValue(ctx as never);
}

describe('sprite cache LRU eviction', () => {
  it('caps size at SPRITE_CACHE_LIMIT and evicts least-recently-used entries instead of clearing all', () => {
    stubCanvas2d();
    const hot = nameSprite('atlas', 'lru-hot', 1);
    const cold = nameSprite('atlas', 'lru-cold', 1);

    // Fill the cache to capacity.
    for (let i = 0; i < SPRITE_CACHE_LIMIT - 2; i++) {
      nameSprite('atlas', `lru-fill-${i}`, 1);
    }
    expect(spriteCacheSize()).toBe(SPRITE_CACHE_LIMIT);

    // Touch `hot` so it moves to the most-recently-used tail while `cold` stays oldest.
    expect(nameSprite('atlas', 'lru-hot', 1)).toBe(hot);

    // Insert 10 new sprites: cache stays capped at SPRITE_CACHE_LIMIT (not cleared to 1),
    // `cold` is evicted and rebuilt, while `hot` remains cached.
    for (let i = 0; i < 10; i++) {
      nameSprite('atlas', `lru-extra-${i}`, 1);
    }
    expect(spriteCacheSize()).toBe(SPRITE_CACHE_LIMIT);
    expect(nameSprite('atlas', 'lru-hot', 1)).toBe(hot);
    expect(nameSprite('atlas', 'lru-cold', 1)).not.toBe(cold);
  });
});
