// Online play's host, in the host's own browser tab (plans/online-play.md). The game runs in the WASM engine
// (`BrowserHost`, crates/wasm/src/host); this module is its transport:
//   - the relay: announce the room, hold the tunnel WebSocket, turn frames into engine calls and back,
//     split large texts into chunks, push the status and lobby row, keep the tunnel alive and reconnect;
//   - the host's own UI: an in-tab socket and status/admin calls, so RemoteClient and the staging room work
//     unchanged, as the one `local` (host) connection.
import type { FetchLike, ServerStatus, SocketLike } from '../remote';
import { type Frame, CHUNK_BYTES, KIND_BYE, KIND_CLOSE, KIND_DATA, KIND_OPEN, KIND_PING, KIND_STATUS, KIND_SUMMARY, chunkText, decodeFrame, encodeFrame, frameText } from './tunnel';

/** The engine's host, as wasm-bindgen exports it. */
export interface HostEngine {
  connect(local: boolean, now: number): number;
  receive(conn: number, text: string, now: number): void;
  disconnect(conn: number): void;
  tick(now: number): void;
  drain(now: number): string;
  status(isLocal: boolean): string;
  summary(): string;
  new_game(setupJson: string): void;
  load_game(save: string): void;
  configure_staging(configJson: string): void;
  close_staging(): void;
  start_staging(): string;
  take_save(): string | undefined;
  /** Hosting from a save file: opens every seat but the host's own for guests. */
  open_guest_seats(): void;
  /** Sets or clears the join code (4 to 12 letters or digits). It stays in the engine; nothing sends it out. */
  set_join_code(code?: string | null): void;
  /** The live game's save right now; throws when there is no game. */
  save_game(): string;
  take_recordings(): string;
  /** As take_recordings, with every seat under its default name: what a build that uploads games keeps and sends. */
  take_public_recordings?(): string;
  free?(): void;
}

export interface RoomAnnounce {
  name: string;
  build: string;
  rules_version: number;
  protocol: number;
  /** The game has a join code. The code itself never leaves the engine. */
  locked?: boolean;
}

export interface RoomKey {
  room: string;
  host_key: string;
}

/** The subset of a browser WebSocket the tunnel uses, so tests can pass a fake. */
export interface TunnelSocket {
  binaryType: string;
  readonly readyState: number;
  send(data: Uint8Array<ArrayBuffer>): void;
  close(code?: number, reason?: string): void;
  onopen: ((ev: unknown) => void) | null;
  onmessage: ((ev: { data: unknown }) => void) | null;
  onclose: ((ev: unknown) => void) | null;
  onerror: ((ev: unknown) => void) | null;
}

export type TunnelState = 'connecting' | 'open' | 'reconnecting' | 'closed';

export interface HostOptions {
  relay: string;
  announce: RoomAnnounce;
  engine: HostEngine;
  /** Reconnect to this room instead of announcing a new one (a reloaded host tab). */
  room?: RoomKey | null;
  /** The live game's save, each time it changes. */
  onSave?: (save: string) => void;
  /** A game history to keep in the archive. */
  onRecording?: (json: string) => void;
  /** Hand over histories with every seat under its default name (a build that uploads games). */
  publicRecordings?: boolean;
  /** The tunnel's state, and the room it serves (it changes only if the relay forgot the room). */
  onTunnel?: (state: TunnelState, room: RoomKey) => void;
  socket?: (url: string) => TunnelSocket;
  fetch?: (url: string, init?: { method?: string; body?: string; headers?: Record<string, string>; signal?: AbortSignal; cache?: 'no-store' }) => Promise<{ ok: boolean; status: number; headers?: { get(name: string): string | null }; json(): Promise<unknown>; text(): Promise<string> }>;
  /** The relay refused to announce the room again (it had forgotten it); the host keeps trying. */
  onRefused?: (refusal: RelayRefusal) => void;
  now?: () => number;
  timers?: Timers;
}

interface Timers {
  set(f: () => void, ms: number): unknown;
  clear(t: unknown): void;
  every(f: () => void, ms: number): unknown;
  stop(t: unknown): void;
}

export const PING_MS = 20_000;
export const TICK_MS = 1000;
export const MIN_BACKOFF_MS = 1000;
export const MAX_BACKOFF_MS = 30_000;
/** Guest messages over this are refused, as on the relay. */
const MAX_GUEST_MESSAGE = 64 * 1024;
const OPEN = 1;

interface Outgoing {
  conn: number;
  texts: string[];
  close: boolean;
}

const browserTimers: Timers = {
  set: (f, ms) => setTimeout(f, ms),
  clear: (t) => clearTimeout(t as ReturnType<typeof setTimeout>),
  every: (f, ms) => setInterval(f, ms),
  stop: (t) => clearInterval(t as ReturnType<typeof setInterval>),
};

/** Why the relay would not list a room. The `message` is a short plain sentence the host can read as it is. */
export type RefusalKind = 'too_fast' | 'too_many' | 'network' | 'full' | 'closed' | 'offline' | 'other';

export class RelayRefusal extends Error {
  constructor(readonly kind: RefusalKind, readonly detail: string, readonly retryAfterS: number | null = null) {
    super(refusalMessage(kind, retryAfterS));
  }
}

/** Live rooms one internet address may hold. Mirrors MAX_ROOMS_PER_IP in infra/relay/src/shared.ts (the server's number is
 * the one that counts; a changed server value only changes this wording, never what is allowed). */
export const MAX_ROOMS_PER_ADDRESS = 2;

/** What the host reads for each refusal: what happened and the one next step. */
export function refusalMessage(kind: RefusalKind, retryAfterS: number | null = null): string {
  switch (kind) {
    case 'too_fast': {
      if (retryAfterS === null) return 'You have opened too many games lately. Try again in a few minutes.';
      const minutes = Math.max(1, Math.ceil(retryAfterS / 60));
      return `You have opened too many games lately. Try again in ${minutes} ${minutes === 1 ? 'minute' : 'minutes'}.`;
    }
    case 'too_many':
      return `Your internet connection already has ${MAX_ROOMS_PER_ADDRESS} games open. Close one before hosting another.`;
    case 'network':
      return 'Too many games are open from your network. Try again later.';
    case 'full':
      return 'The game list is full right now. Try again in a few minutes.';
    case 'closed':
      return 'Hosting new games is paused. Try again later.';
    case 'offline':
      return 'Could not open your game. Check your internet connection and try again.';
    default:
      return 'Could not open your game. Try again.';
  }
}

/** What Host Game's sub-line says after a refusal. Only a refusal that a retry can fix says "Try again". */
export function retryLine(kind: RefusalKind, retryAfterS: number | null = null): string {
  switch (kind) {
    case 'too_fast': {
      if (retryAfterS === null) return 'Try again in a few minutes';
      const minutes = Math.max(1, Math.ceil(retryAfterS / 60));
      return `Try again in ${minutes} ${minutes === 1 ? 'minute' : 'minutes'}`;
    }
    case 'too_many':
      return 'Close one of your games first';
    case 'closed':
      return 'Paused for now';
    case 'network':
      return 'Too many from your network';
    default:
      return 'Try again';
  }
}

/** A refusal where pressing Host Game again right away cannot work, so it is not the next action. */
export const retryWouldFail = (kind: RefusalKind): boolean => kind === 'too_fast' || kind === 'too_many' || kind === 'network' || kind === 'closed';

/**
 * The relay answers POST /rooms: 429 (the creation rate, or the per-address room cap) and 503 (paused or full).
 * A browser calling the relay from another origin can read Retry-After only if the relay exposes it, so the two 429s are
 * also told apart by their body text, which the relay has always sent.
 */
export function refusalFor(status: number, error: string | undefined, retryAfter: string | null): RelayRefusal {
  const detail = `The relay refused the room (${status}): ${error ?? 'no reason given'}`;
  if (status === 429) {
    const secs = retryAfter !== null && /^\d{1,6}$/.test(retryAfter.trim()) ? Number(retryAfter.trim()) : null;
    if (secs !== null || /too quickly|too fast/i.test(error ?? '')) return new RelayRefusal('too_fast', detail, secs);
    if (/from your network/i.test(error ?? '')) return new RelayRefusal('network', detail);
    if (/games? open/i.test(error ?? '')) return new RelayRefusal('too_many', detail);
    return new RelayRefusal('other', detail);
  }
  if (status === 503) return new RelayRefusal(/paused/i.test(error ?? '') ? 'closed' : 'full', detail);
  return new RelayRefusal('other', detail);
}

/** https://relay -> wss://relay; http -> ws (tests). */
export function wsBase(relay: string): string {
  return relay.replace(/\/+$/, '').replace(/^http/i, 'ws');
}

export class BrowserHost {
  private readonly relay: string;
  private readonly engine: HostEngine;
  private readonly timers: Timers;
  private readonly now: () => number;
  private key: RoomKey | null;
  private ws: TunnelSocket | null = null;
  private backoff = MIN_BACKOFF_MS;
  private reconnectTimer: unknown = null;
  private pingTimer: unknown = null;
  private tickTimer: unknown = null;
  private stopped = false;
  private pushed = { status: '', summary: '' };
  /** relay conn -> engine conn, and back; chunk ids per relay conn. */
  private guests = new Map<number, number>();
  private guestOf = new Map<number, number>();
  private chunkIds = new Map<number, number>();
  /** engine conn -> the host's own in-tab socket. */
  private locals = new Map<number, LoopbackSocket>();

  constructor(private readonly options: HostOptions) {
    this.relay = options.relay.replace(/\/+$/, '');
    this.engine = options.engine;
    this.timers = options.timers ?? browserTimers;
    this.now = options.now ?? (() => performance.now());
    this.key = options.room ?? null;
  }

  get room(): RoomKey | null {
    return this.key;
  }

  /** Announces the room (unless reconnecting to one) and opens the tunnel. Rejects only if the announcement fails. */
  async start(): Promise<RoomKey> {
    if (!this.key) this.key = await this.announce();
    this.tickTimer = this.timers.every(() => this.tick(), TICK_MS);
    this.connect();
    return this.key;
  }

  /** The live game's save right now, for the host's "Save to file"; null when there is no game. */
  saveGame(): string | null {
    try {
      return this.engine.save_game() || null;
    } catch {
      return null;
    }
  }

  /** Closes the room for good: `bye`, then the tunnel and every in-tab socket. */
  stop() {
    if (this.stopped) return;
    this.stopped = true;
    for (const t of [this.reconnectTimer]) if (t) this.timers.clear(t);
    for (const t of [this.pingTimer, this.tickTimer]) if (t) this.timers.stop(t);
    const ws = this.ws;
    this.ws = null;
    if (ws && ws.readyState === OPEN) {
      try {
        ws.send(encodeFrame(0, KIND_BYE));
      } catch {
        // the relay forgets the room on its own
      }
    }
    try {
      ws?.close(1000, 'host left');
    } catch {
      // already closed
    }
    for (const sock of [...this.locals.values()]) sock.remoteClose();
    this.locals.clear();
    this.guests.clear();
    this.guestOf.clear();
    this.options.onTunnel?.('closed', this.key ?? { room: '', host_key: '' });
    this.engine.free?.();
  }

  // ---- The host's own UI ----

  /** The current status, as `/api/status` would give it to the host. */
  status(): ServerStatus {
    return JSON.parse(this.engine.status(true)) as ServerStatus;
  }

  /** A socket from the host's own UI into the game: the `local` connection. */
  socket(): SocketLike {
    return new LoopbackSocket(this);
  }

  /** The host's admin routes (`/api/status`, `/api/game`, `/api/staging`, `/api/staging/start`), for RemoteClient and staging.ts. */
  readonly fetch: FetchLike = async (input, init) => {
    const path = new URL(input, 'http://host').pathname.replace(/^.*(?=\/api\/)/, '');
    const method = (init?.method ?? 'GET').toUpperCase();
    const reply = (status: number, body: unknown) => ({ ok: status < 300, status, text: async () => (typeof body === 'string' ? body : JSON.stringify(body)) });
    if (this.stopped) return reply(503, { type: 'error', message: 'The game has closed.' });
    const run = (f: () => void) => {
      try {
        f();
      } catch (e) {
        this.pump();
        return reply(400, { type: 'error', message: e instanceof Error ? e.message : String(e) });
      }
      this.pump();
      return reply(200, this.engine.status(true));
    };
    if (path === '/api/status' && method === 'GET') return reply(200, this.engine.status(true));
    if (path === '/api/game' && method === 'POST') return run(() => this.engine.new_game(init?.body ?? ''));
    if (path === '/api/load' && method === 'POST') return run(() => this.engine.load_game(init?.body ?? ''));
    if (path === '/api/staging' && method === 'POST') return run(() => this.engine.configure_staging(init?.body ?? ''));
    if (path === '/api/staging' && method === 'DELETE') return run(() => this.engine.close_staging());
    if (path === '/api/staging/start' && method === 'POST') {
      const refused = this.engine.start_staging();
      this.pump();
      if (refused) {
        const e = JSON.parse(refused) as { code: string | null; message: string };
        return reply(e.code ? 409 : 400, { type: 'error', ...(e.code ? { code: e.code } : {}), message: e.message });
      }
      return reply(200, this.engine.status(true));
    }
    return reply(404, { type: 'error', message: 'Not found' });
  };

  /** @internal LoopbackSocket */
  openLocal(sock: LoopbackSocket): number | null {
    if (this.stopped) return null;
    try {
      const conn = this.engine.connect(true, this.now());
      this.locals.set(conn, sock);
      this.pump();
      return conn;
    } catch {
      return null;
    }
  }

  /** @internal LoopbackSocket */
  receiveLocal(conn: number, text: string) {
    if (this.stopped || !this.locals.has(conn)) return;
    this.engine.receive(conn, text, this.now());
    this.pump();
  }

  /** @internal LoopbackSocket */
  closeLocal(conn: number) {
    if (!this.locals.delete(conn) || this.stopped) return;
    this.engine.disconnect(conn);
    this.pump();
  }

  // ---- Engine output ----

  /** Writes everything the engine queued, keeps saves and histories, and pushes a changed status or lobby row. */
  pump() {
    if (this.stopped) return;
    const out = JSON.parse(this.engine.drain(this.now())) as Outgoing[];
    for (const { conn, texts, close } of out) {
      const local = this.locals.get(conn);
      if (local) {
        if (close) this.locals.delete(conn);
        local.deliver(texts, close);
        continue;
      }
      const guest = this.guestOf.get(conn);
      if (guest === undefined) continue;
      for (const text of texts) {
        if (new TextEncoder().encode(text).length > CHUNK_BYTES) {
          const id = (this.chunkIds.get(guest) ?? 0) + 1;
          this.chunkIds.set(guest, id);
          for (const piece of chunkText(text, id)) this.sendFrame(guest, KIND_DATA, piece);
        } else this.sendFrame(guest, KIND_DATA, text);
      }
      if (close) {
        this.sendFrame(guest, KIND_CLOSE);
        this.forgetGuest(guest);
      }
    }
    const save = this.engine.take_save();
    if (save !== undefined && save !== null) this.options.onSave?.(save);
    // A build that uploads games takes the copies without typed names (hosting.ts sets `publicRecordings`): the
    // upload server refuses any other, so an online game's record would otherwise never arrive.
    const recordings = this.options.publicRecordings && this.engine.take_public_recordings ? this.engine.take_public_recordings() : this.engine.take_recordings();
    for (const json of JSON.parse(recordings) as string[]) this.options.onRecording?.(json);
    this.push();
  }

  private push(force = false) {
    if (!this.ws || this.ws.readyState !== OPEN) return;
    const status = this.engine.status(false);
    const summary = this.engine.summary();
    if (force || status !== this.pushed.status) {
      this.sendFrame(0, KIND_STATUS, status);
      this.pushed.status = status;
    }
    if (force || summary !== this.pushed.summary) {
      this.sendFrame(0, KIND_SUMMARY, summary);
      this.pushed.summary = summary;
    }
  }

  private tick() {
    if (this.stopped) return;
    this.engine.tick(this.now());
    this.pump();
  }

  // ---- The relay ----

  private async announce(): Promise<RoomKey> {
    const fetchImpl = this.options.fetch ?? fetch;
    let res;
    try {
      res = await fetchImpl(`${this.relay}/rooms`, {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        // The list names the room from its id (roomName.ts), so the host's name stays in the room and is not sent.
        body: JSON.stringify({ ...this.options.announce, name: '', build: this.options.announce.build.slice(0, 40) }),
      });
    } catch (e) {
      throw new RelayRefusal('offline', `Cannot reach the relay: ${String(e)}`);
    }
    if (!res.ok) {
      const body = (await res.json().catch(() => ({}))) as { error?: string };
      throw refusalFor(res.status, body.error, res.headers?.get('retry-after') ?? null);
    }
    const key = (await res.json()) as RoomKey;
    if (typeof key?.room !== 'string' || typeof key.host_key !== 'string') throw new Error('Unexpected relay response');
    return key;
  }

  private connect() {
    if (this.stopped || !this.key) return;
    const key = this.key;
    const url = `${wsBase(this.relay)}/rooms/${encodeURIComponent(key.room)}/host?key=${encodeURIComponent(key.host_key)}`;
    this.options.onTunnel?.(this.backoff === MIN_BACKOFF_MS && !this.reconnectTimer ? 'connecting' : 'reconnecting', key);
    this.reconnectTimer = null;
    let ws: TunnelSocket;
    try {
      ws = (this.options.socket ?? ((u: string) => new WebSocket(u) as unknown as TunnelSocket))(url);
    } catch {
      this.scheduleReconnect(false);
      return;
    }
    ws.binaryType = 'arraybuffer';
    this.ws = ws;
    let opened = false;
    ws.onopen = () => {
      if (this.ws !== ws) return;
      opened = true;
      this.backoff = MIN_BACKOFF_MS;
      this.pushed = { status: '', summary: '' };
      this.push(true);
      if (this.pingTimer) this.timers.stop(this.pingTimer);
      this.pingTimer = this.timers.every(() => {
        if (this.ws === ws && ws.readyState === OPEN) ws.send(encodeFrame(0, KIND_PING));
      }, PING_MS);
      this.options.onTunnel?.('open', key);
    };
    ws.onmessage = (ev) => {
      if (this.ws !== ws || this.stopped) return;
      if (!(ev.data instanceof ArrayBuffer) && !(ev.data instanceof Uint8Array)) return;
      const frame = decodeFrame(ev.data);
      if (frame) this.route(frame.conn, frame.kind, frame);
    };
    ws.onerror = () => {
      // onclose follows
    };
    ws.onclose = () => {
      if (this.ws !== ws) return;
      this.ws = null;
      if (this.pingTimer) this.timers.stop(this.pingTimer);
      this.pingTimer = null;
      // The relay closes every guest with the tunnel; they reconnect by themselves.
      for (const guest of [...this.guests.keys()]) this.dropGuest(guest);
      if (!this.stopped) this.scheduleReconnect(!opened);
    };
  }

  private scheduleReconnect(neverOpened: boolean) {
    if (this.stopped || !this.key) return;
    const delay = this.backoff;
    this.backoff = Math.min(this.backoff * 2, MAX_BACKOFF_MS);
    this.options.onTunnel?.('reconnecting', this.key);
    this.reconnectTimer = this.timers.set(() => {
      void (async () => {
        // A browser cannot see why an upgrade failed: ask the room. 404 = the relay forgot it, so open a new one.
        if (neverOpened && (await this.roomGone())) {
          try {
            this.key = await this.announce();
          } catch (e) {
            if (e instanceof RelayRefusal && !this.stopped) this.options.onRefused?.(e);
            if (!this.stopped) this.scheduleReconnect(true);
            return;
          }
        }
        this.connect();
      })();
    }, delay);
  }

  private async roomGone(): Promise<boolean> {
    if (!this.key) return false;
    try {
      const res = await (this.options.fetch ?? fetch)(`${this.relay}/rooms/${encodeURIComponent(this.key.room)}/api/status`, { cache: 'no-store' });
      return res.status === 404;
    } catch {
      return false;
    }
  }

  private route(conn: number, kind: number, frame: Frame) {
    const now = this.now();
    switch (kind) {
      case KIND_OPEN: {
        if (conn === 0 || this.guests.has(conn)) return;
        let id: number;
        try {
          id = this.engine.connect(false, now);
        } catch {
          this.sendFrame(conn, KIND_CLOSE);
          return;
        }
        this.guests.set(conn, id);
        this.guestOf.set(id, conn);
        this.pump();
        return;
      }
      case KIND_DATA: {
        const id = this.guests.get(conn);
        if (id === undefined) return;
        if (frame.payload.length > MAX_GUEST_MESSAGE) {
          this.sendFrame(conn, KIND_CLOSE);
          this.dropGuest(conn);
          return;
        }
        this.engine.receive(id, frameText(frame), now);
        this.pump();
        return;
      }
      case KIND_CLOSE:
        this.dropGuest(conn);
        return;
      default:
        return;
    }
  }

  /** The guest is gone: tell the engine. */
  private dropGuest(conn: number) {
    const id = this.guests.get(conn);
    if (id === undefined) return;
    this.forgetGuest(conn);
    this.engine.disconnect(id);
    this.pump();
  }

  private forgetGuest(conn: number) {
    const id = this.guests.get(conn);
    this.guests.delete(conn);
    this.chunkIds.delete(conn);
    if (id !== undefined) this.guestOf.delete(id);
  }

  private sendFrame(conn: number, kind: number, payload: string | Uint8Array = '') {
    if (this.ws && this.ws.readyState === OPEN) this.ws.send(encodeFrame(conn, kind, payload));
  }
}

/**
 * The host's own connection to the game in its tab: a SocketLike that opens, delivers and closes asynchronously,
 * like a WebSocket, so the UI code paths are the same as a guest's.
 */
export class LoopbackSocket implements SocketLike {
  readyState = 0;
  onopen: SocketLike['onopen'] = null;
  onmessage: SocketLike['onmessage'] = null;
  onclose: SocketLike['onclose'] = null;
  onerror: SocketLike['onerror'] = null;
  private conn: number | null = null;

  constructor(private readonly host: BrowserHost) {
    queueMicrotask(() => {
      if (this.readyState !== 0) return;
      this.conn = host.openLocal(this);
      if (this.conn === null) {
        this.readyState = 3;
        this.onclose?.({ code: 1013 });
        return;
      }
      this.readyState = OPEN;
      this.onopen?.({});
    });
  }

  send(data: string) {
    if (this.readyState !== OPEN || this.conn === null) throw new Error('The connection is not open');
    this.host.receiveLocal(this.conn, data);
  }

  close() {
    if (this.readyState >= 2) return;
    const conn = this.conn;
    this.readyState = 3;
    if (conn !== null) this.host.closeLocal(conn);
    queueMicrotask(() => this.onclose?.({ code: 1000 }));
  }

  /** @internal The host wrote to this connection, and may have ended it. */
  deliver(texts: string[], close: boolean) {
    if (close) this.readyState = 3;
    queueMicrotask(() => {
      for (const data of texts) this.onmessage?.({ data });
      if (close) this.onclose?.({ code: 1000 });
    });
  }

  /** @internal The host closed for good. */
  remoteClose() {
    if (this.readyState >= 2) return;
    this.readyState = 3;
    queueMicrotask(() => this.onclose?.({ code: 1001 }));
  }
}
