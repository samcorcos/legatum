import { expect, it } from 'vitest';
import { KIND_DATA, KIND_PING, chunkText, decodeFrame, encodeFrame, frameText } from './tunnel';

it('frames are [conn u32 big-endian][kind u8][payload] and decode back', () => {
  const f = encodeFrame(0x01020304, KIND_DATA, 'hé');
  expect([...f.slice(0, 5)]).toEqual([1, 2, 3, 4, KIND_DATA]);
  expect(f.length).toBe(5 + 3);
  const d = decodeFrame(f.buffer)!;
  expect(d.conn).toBe(0x01020304);
  expect(d.kind).toBe(KIND_DATA);
  expect(frameText(d)).toBe('hé');
});

it('an empty keepalive is just the header, and a short frame is refused', () => {
  expect(encodeFrame(0, KIND_PING).length).toBe(5);
  expect(decodeFrame(new Uint8Array([0, 0, 0]))).toBeNull();
});

it('splits a long text into chunks that reassemble, never inside a character', () => {
  const text = JSON.stringify({ type: 'view', s: 'aé€😀'.repeat(50) });
  const pieces = chunkText(text, 7, 64).map((p) => JSON.parse(p) as { type: string; id: number; i: number; n: number; data: string });
  expect(pieces.length).toBeGreaterThan(3);
  for (const [i, p] of pieces.entries()) {
    expect(p).toMatchObject({ type: 'chunk', id: 7, i, n: pieces.length });
    expect(new TextEncoder().encode(p.data).length).toBeLessThanOrEqual(64);
    expect(p.data).not.toMatch(/�/);
  }
  expect(pieces.map((p) => p.data).join('')).toBe(text);
});
