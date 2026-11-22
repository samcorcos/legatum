// @vitest-environment node
// The natural-wonder art is shown as a plain <img> with no mask (the tile card and discovery card use the
// `-a-512` vignette, lists and the map the `-b-128` sprite), so every PNG must carry real alpha. The `-a`
// vignettes shipped as paintings on an opaque black square and drew a black box on the parchment tile card
// until 2026-11-21 (keyed by tools/art/wonder-vignette-key.py); the `-b` sprites did the same in build 8.8.
import { readdirSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import { alphaOf } from './test/pngAlpha';
import { WONDERS, wonderArt } from './wonders';

const dir = new URL('../public/wonders/', import.meta.url).pathname;
const files = readdirSync(dir).filter((f) => f.endsWith('.png'));

describe('natural-wonder art has real alpha', () => {
  it('finds a vignette and a sprite for every wonder', () => {
    for (const wonder of WONDERS) for (const style of ['a', 'b'] as const) expect(files, wonderArt(wonder, style)).toContain(wonderArt(wonder, style).replace('/wonders/', ''));
    expect(files.length).toBe(WONDERS.length * 2);
  });
  for (const file of files) {
    it(`${file}: transparent corners and border, the painting opaque in the middle`, () => {
      const { width, height, alpha } = alphaOf(dir + file);
      const at = (x: number, y: number) => alpha[y * width + x];
      for (const [x, y] of [[0, 0], [width - 1, 0], [0, height - 1], [width - 1, height - 1]]) expect(at(x, y), `corner ${x},${y}`).toBe(0);
      // No opaque frame: the outermost ring of pixels is almost entirely clear (a cut-out may touch an edge at a point or two).
      let edge = 0;
      for (let x = 0; x < width; x++) edge += (at(x, 0) > 0 ? 1 : 0) + (at(x, height - 1) > 0 ? 1 : 0);
      for (let y = 0; y < height; y++) edge += (at(0, y) > 0 ? 1 : 0) + (at(width - 1, y) > 0 ? 1 : 0);
      expect(edge / (2 * (width + height))).toBeLessThan(0.25);
      expect(at(width >> 1, height >> 1), 'centre').toBe(255);
      const clear = alpha.filter((a) => a === 0).length / alpha.length;
      expect(clear).toBeGreaterThan(0.25);
      expect(clear).toBeLessThan(0.85);
      // The cut-out keeps a soft edge: some pixels are partly transparent.
      expect(alpha.some((a) => a > 0 && a < 255)).toBe(true);
    });
  }
});
