// @vitest-environment node
import { existsSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import { alphaOf } from './test/pngAlpha';

const dir = new URL('./assets/construction/', import.meta.url).pathname;
const sources = [
  ...['building-greatlibrary', 'building-grandbazaar', 'building-royalarsenal', 'building-pyramids', 'building-hagiasophia', 'building-eiffeltower', 'building-iterfusionreactor', 'building-yamrelaystation'].map(stem => ({ stem, base: 1254, sizes: [null, 256, 128] })),
  ...['ancient', 'classical', 'medieval', 'renaissance', 'industrial', 'modern', 'future'].map(era => ({ stem: `${era}-hamlet`, base: 512, sizes: [null, 128, 64] })),
  ...['ancient', 'classical', 'medieval', 'renaissance', 'industrial'].map(era => ({ stem: `${era}-walls`, base: 512, sizes: [null, 128, 64] })),
];

describe('aligned construction paintings', () => {
  for (const { stem, base, sizes } of sources) for (const stage of [1, 2, 3]) for (const size of sizes) {
    const name = `${stem}-stage${stage}${size ? `-${size}` : ''}`;
    it(`${name}: source dimensions and transparent corners`, () => {
      const file = `${dir}${name}.png`;
      expect(existsSync(file)).toBe(true);
      const { width, height, alpha } = alphaOf(file);
      expect([width, height]).toEqual([size ?? base, size ?? base]);
      for (const index of [0, width - 1, width * (height - 1), width * height - 1]) expect(alpha[index]).toBe(0);
      expect(alpha.some(a => a === 255)).toBe(true);
    });
  }
});
