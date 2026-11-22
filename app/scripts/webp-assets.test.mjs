// The hosted build serves bundled PNG art as WebP (vite-plugin-webp.ts): small art must stay pixel-exact, alpha included.
import { describe, expect, it } from 'vitest';
import sharp from 'sharp';
import { pngToWebp } from '../vite-plugin-webp.ts';

const raw = async (buf) => (await sharp(buf).ensureAlpha().raw().toBuffer({ resolveWithObject: true }));
const sprite = (size) => {
  const data = Buffer.alloc(size * size * 4);
  for (let y = 0; y < size; y++) for (let x = 0; x < size; x++) {
    const i = (y * size + x) * 4;
    const inside = (x - size / 2) ** 2 + (y - size / 2) ** 2 < (size / 2.5) ** 2;
    data.set(inside ? [(x * 7) % 256, (y * 5) % 256, 120, 255] : [0, 0, 0, 0], i);
  }
  return sharp(data, { raw: { width: size, height: size, channels: 4 } }).png().toBuffer();
};

describe('pngToWebp', () => {
  it('encodes a small sprite losslessly with its alpha', async () => {
    const png = await sprite(128);
    const webp = await pngToWebp(png);
    expect(webp).not.toBeNull();
    const [a, b] = await Promise.all([raw(png), raw(webp)]);
    expect(b.info.width).toBe(128);
    expect(Buffer.compare(a.data, b.data)).toBe(0);
  });

  it('encodes a large painted image lossy but keeps alpha exact', async () => {
    const png = await sprite(512);
    const webp = await pngToWebp(png);
    expect(webp).not.toBeNull();
    const [a, b] = await Promise.all([raw(png), raw(webp)]);
    for (let i = 3; i < a.data.length; i += 4) if (a.data[i] !== b.data[i]) throw new Error(`alpha differs at ${i}`);
  });

  it('keeps flat art lossless at any size', async () => {
    const size = 512, data = Buffer.alloc(size * size * 4);
    for (let y = 0; y < size; y++) for (let x = 0; x < size; x++) data.set((x >> 4) % 2 ? [200, 30, 30, 255] : [20, 20, 20, 0], (y * size + x) * 4);
    const png = await sharp(data, { raw: { width: size, height: size, channels: 4 } }).png().toBuffer();
    const webp = await pngToWebp(png);
    const [a, b] = await Promise.all([raw(png), raw(webp)]);
    for (let i = 0; i < a.data.length; i += 4) {
      if (a.data[i + 3] !== b.data[i + 3]) throw new Error(`alpha differs at ${i}`);
      if (a.data[i + 3] && (a.data[i] !== b.data[i] || a.data[i + 1] !== b.data[i + 1] || a.data[i + 2] !== b.data[i + 2])) throw new Error(`colour differs at ${i}`);
    }
  });

  it('is deterministic', async () => {
    const png = await sprite(64);
    expect(Buffer.compare(await pngToWebp(png), await pngToWebp(png))).toBe(0);
  });
});
