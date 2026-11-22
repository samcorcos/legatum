// The relay tunnel's wire format (plans/online-play.md): binary frames `[conn u32 BE][kind u8][payload]`
// between the browser host and the relay, and the `chunk` messages that carry large texts to a guest.

export const KIND_OPEN = 0;
export const KIND_DATA = 1;
export const KIND_CLOSE = 2;
export const KIND_STATUS = 3;
export const KIND_SUMMARY = 4;
export const KIND_BYE = 5;
/** Host -> relay keepalive. Browsers cannot send WebSocket pings; the relay ignores unknown kinds. */
export const KIND_PING = 6;

/** Guest-bound texts longer than this many UTF-8 bytes travel as `chunk` messages. */
export const CHUNK_BYTES = 256 * 1024;

const enc = new TextEncoder();
const dec = new TextDecoder();

export interface Frame {
  conn: number;
  kind: number;
  payload: Uint8Array;
}

export function encodeFrame(conn: number, kind: number, payload: string | Uint8Array = ''): Uint8Array<ArrayBuffer> {
  const body = typeof payload === 'string' ? enc.encode(payload) : payload;
  const out = new Uint8Array(5 + body.length);
  new DataView(out.buffer).setUint32(0, conn >>> 0, false);
  out[4] = kind;
  out.set(body, 5);
  return out;
}

/** null for a frame too short to have a header. */
export function decodeFrame(data: ArrayBuffer | Uint8Array): Frame | null {
  const bytes = data instanceof Uint8Array ? data : new Uint8Array(data);
  if (bytes.length < 5) return null;
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  return { conn: view.getUint32(0, false), kind: bytes[4], payload: bytes.subarray(5) };
}

export function frameText(frame: Frame): string {
  return dec.decode(frame.payload);
}

/**
 * Splits a text longer than `CHUNK_BYTES` UTF-8 bytes, on character boundaries, into `chunk` messages that
 * `relaySocket.ts` puts back together; a shorter text passes through unchanged.
 */
export function chunkText(text: string, id: number, limit = CHUNK_BYTES): string[] {
  const bytes = enc.encode(text);
  if (bytes.length <= limit) return [text];
  const pieces: string[] = [];
  let start = 0;
  while (start < bytes.length) {
    let end = Math.min(start + limit, bytes.length);
    // Never end inside a UTF-8 sequence: continuation bytes are 10xxxxxx.
    while (end < bytes.length && (bytes[end] & 0xc0) === 0x80) end--;
    pieces.push(dec.decode(bytes.subarray(start, end)));
    start = end;
  }
  const n = pieces.length;
  return pieces.map((data, i) => `{"type":"chunk","id":${id},"i":${i},"n":${n},"data":${JSON.stringify(data)}}`);
}
