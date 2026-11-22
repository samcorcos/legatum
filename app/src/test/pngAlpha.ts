// Test helper: decode an 8-bit RGBA, non-interlaced PNG to its alpha plane (map sprites are drawn
// with no mask, so the art itself must carry the alpha).
import { readFileSync } from 'node:fs';
import { inflateSync } from 'node:zlib';

/** Decode an 8-bit RGBA, non-interlaced PNG to its alpha plane. */
export function alphaOf(file: string): { width: number; height: number; alpha: Uint8Array } {
  const buf = readFileSync(file);
  let pos = 8, width = 0, height = 0;
  const idat: Buffer[] = [];
  while (pos < buf.length) {
    const len = buf.readUInt32BE(pos), type = buf.toString('ascii', pos + 4, pos + 8);
    const data = buf.subarray(pos + 8, pos + 8 + len);
    if (type === 'IHDR') {
      width = data.readUInt32BE(0); height = data.readUInt32BE(4);
      if (data[8] !== 8 || data[9] !== 6 || data[12] !== 0) throw new Error(`${file}: expected 8-bit RGBA, not interlaced`);
    } else if (type === 'IDAT') idat.push(data);
    pos += 12 + len;
  }
  const raw = inflateSync(Buffer.concat(idat)), stride = width * 4;
  const out = new Uint8Array(height * stride);
  for (let y = 0; y < height; y++) {
    const filter = raw[y * (stride + 1)], line = raw.subarray(y * (stride + 1) + 1, (y + 1) * (stride + 1));
    for (let x = 0; x < stride; x++) {
      const a = x >= 4 ? out[y * stride + x - 4] : 0, b = y > 0 ? out[(y - 1) * stride + x] : 0, c = x >= 4 && y > 0 ? out[(y - 1) * stride + x - 4] : 0;
      const p = a + b - c, pa = Math.abs(p - a), pb = Math.abs(p - b), pc = Math.abs(p - c);
      const pred = filter === 0 ? 0 : filter === 1 ? a : filter === 2 ? b : filter === 3 ? (a + b) >> 1 : pa <= pb && pa <= pc ? a : pb <= pc ? b : c;
      out[y * stride + x] = (line[x] + pred) & 255;
    }
  }
  const alpha = new Uint8Array(width * height);
  for (let i = 0; i < alpha.length; i++) alpha[i] = out[i * 4 + 3];
  return { width, height, alpha };
}
