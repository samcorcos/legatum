// @vitest-environment node
// The map draws each natural wonder's ink-and-wash PNG straight onto the tile with no mask, so
// the art itself must carry the alpha: an opaque square draws as a black box (playtest 8.8).
import { describe, expect, it } from 'vitest';
import { alphaOf } from './test/pngAlpha';
import { WONDERS, wonderArt } from './wonders';

describe('natural wonder map art has real alpha', () => {
  for (const wonder of WONDERS) {
    it(`${wonder}: transparent corners, a solid subject`, () => {
      const { width, height, alpha } = alphaOf(new URL(`../public${wonderArt(wonder, 'b')}`, import.meta.url).pathname);
      const at = (x: number, y: number) => alpha[y * width + x];
      for (const [x, y] of [[0, 0], [width - 1, 0], [0, height - 1], [width - 1, height - 1]]) expect(at(x, y), `corner ${x},${y}`).toBe(0);
      const solid = alpha.filter((a) => a === 255).length / alpha.length;
      const clear = alpha.filter((a) => a === 0).length / alpha.length;
      expect(solid).toBeGreaterThan(0.12);
      expect(clear).toBeGreaterThan(0.2);
    });
  }
});
