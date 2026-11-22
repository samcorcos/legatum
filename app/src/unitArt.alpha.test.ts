// @vitest-environment node
// The map draws each unit's painted miniature straight onto the tile with no mask, so every PNG
// the unit set ships (map sizes and the panel portrait) must carry real alpha: an opaque square
// draws as a box around the figure (the 8.8 natural wonders did).
import { readdirSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import { alphaOf } from './test/pngAlpha';

const dir = new URL('./assets/units/', import.meta.url).pathname;
const sprites = readdirSync(dir).filter((f) => /-(48|64|256|card|panel)\.png$/.test(f));

describe('unit sprites have real alpha', () => {
  it('finds the whole set', () => expect(sprites.length).toBeGreaterThanOrEqual(5 * 4));
  for (const file of sprites) {
    it(`${file}: transparent corners, a figure in the middle`, () => {
      const { width, height, alpha } = alphaOf(dir + file);
      const at = (x: number, y: number) => alpha[y * width + x];
      for (const [x, y] of [[0, 0], [width - 1, 0], [0, height - 1], [width - 1, height - 1]]) expect(at(x, y), `corner ${x},${y}`).toBe(0);
      const clear = alpha.filter((a) => a === 0).length / alpha.length;
      expect(clear).toBeGreaterThan(0.3);
      expect(clear).toBeLessThan(0.92);
    });
  }
});
