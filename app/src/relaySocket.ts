// Online play (plans/online-play.md, "Large messages"): the host splits a text message longer than
// 256 KiB into `{"type":"chunk","id","i","n","data"}` messages. This wrapper puts them back
// together and hands the original text to the app as one message. Everything else passes through.
// Used only for relayed connections; a bad chunk drops the partial message and never throws.
import type { SocketFactory, SocketLike } from './remote';

/** Far above anything the host sends; stops runaway buffering from a broken peer. */
const MAX_PIECES = 1024;
const MAX_CHARS = 64 * 1024 * 1024;

interface Run {
  id: number;
  n: number;
  next: number;
  chars: number;
  parts: string[];
}

interface Chunk {
  id: number;
  i: number;
  n: number;
  data: string;
}

function parseChunk(text: string): Chunk | 'bad' | null {
  // Fast path: normal messages are never parsed here. The host writes "type" first.
  if (!text.startsWith('{"type":"chunk"') && !text.slice(0, 40).includes('"chunk"')) return null;
  let msg: unknown;
  try {
    msg = JSON.parse(text);
  } catch {
    return null; // not ours; the app decides what to do with it
  }
  if (!msg || typeof msg !== 'object' || (msg as { type?: unknown }).type !== 'chunk') return null;
  const { id, i, n, data } = msg as Record<string, unknown>;
  const whole = (v: unknown): v is number => typeof v === 'number' && Number.isInteger(v) && v >= 0;
  if (!whole(id) || !whole(i) || !whole(n) || n < 1 || n > MAX_PIECES || i >= n || typeof data !== 'string') return 'bad';
  return { id, i, n, data };
}

/** Reassembles chunks across calls. `push` returns the text to deliver, or null when nothing is ready. */
export class ChunkAssembler {
  private run: Run | null = null;

  push(text: string): string | null {
    const chunk = parseChunk(text);
    if (chunk === null) {
      // A normal message in the middle of a run means a piece went missing.
      this.run = null;
      return text;
    }
    if (chunk === 'bad') {
      this.run = null;
      return null;
    }
    let p = this.run;
    if (chunk.i === 0) p = this.run = { id: chunk.id, n: chunk.n, next: 0, chars: 0, parts: [] };
    if (!p || p.id !== chunk.id || p.n !== chunk.n || p.next !== chunk.i) {
      this.run = null;
      return null;
    }
    p.parts.push(chunk.data);
    p.next++;
    p.chars += chunk.data.length;
    if (p.chars > MAX_CHARS) {
      this.run = null;
      return null;
    }
    if (p.next < p.n) return null;
    this.run = null;
    return p.parts.join('');
  }

  reset() {
    this.run = null;
  }
}

class RelaySocket implements SocketLike {
  onopen: SocketLike['onopen'] = null;
  onmessage: SocketLike['onmessage'] = null;
  onclose: SocketLike['onclose'] = null;
  onerror: SocketLike['onerror'] = null;
  private readonly assembler = new ChunkAssembler();

  constructor(private readonly inner: SocketLike) {
    inner.onopen = (ev) => this.onopen?.(ev);
    inner.onerror = (ev) => this.onerror?.(ev);
    inner.onclose = (ev) => {
      this.assembler.reset();
      this.onclose?.(ev);
    };
    inner.onmessage = (ev) => {
      let out: string | null;
      try {
        out = typeof ev.data === 'string' ? this.assembler.push(ev.data) : String(ev.data);
      } catch {
        this.assembler.reset();
        return;
      }
      if (out !== null) this.onmessage?.({ data: out });
    };
  }

  get readyState() {
    return this.inner.readyState;
  }

  send(data: string) {
    this.inner.send(data);
  }

  close() {
    this.inner.close();
  }
}

/** Wrap one socket. */
export function wrapRelaySocket(sock: SocketLike): SocketLike {
  return new RelaySocket(sock);
}

/** Wrap a factory so every socket it makes reassembles chunks. */
export function relaySocketFactory(inner: SocketFactory): SocketFactory {
  return (url) => wrapRelaySocket(inner(url));
}
