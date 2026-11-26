import { beforeEach, expect, it, vi } from 'vitest';
import { BrowserHost, MAX_BACKOFF_MS, MIN_BACKOFF_MS, PING_MS, RelayRefusal, refusalMessage, retryLine, retryWouldFail, type HostEngine, type TunnelSocket } from './browserHost';
import { KIND_BYE, KIND_CLOSE, KIND_DATA, KIND_OPEN, KIND_PING, KIND_STATUS, KIND_SUMMARY, decodeFrame, encodeFrame, frameText } from './tunnel';

/** An engine that echoes: every received text goes back to its sender. */
class FakeEngine implements HostEngine {
  next = 1;
  out = new Map<number, { texts: string[]; close: boolean }>();
  disconnected: number[] = [];
  statusText = '{"mode":"server","seats":[]}';
  save: string | undefined;
  recordings: string[] = [];
  freed = false;
  connect() { return this.next++; }
  receive(conn: number, text: string) {
    const o = this.out.get(conn) ?? { texts: [], close: false };
    o.texts.push(text === 'quit' ? 'bye' : text);
    if (text === 'quit') o.close = true;
    this.out.set(conn, o);
  }
  disconnect(conn: number) { this.disconnected.push(conn); }
  tick() {}
  drain() {
    const out = [...this.out.entries()].map(([conn, o]) => ({ conn, ...o }));
    this.out.clear();
    return JSON.stringify(out);
  }
  status(isLocal: boolean) { return isLocal ? this.statusText.replace('}', ',"is_local":true}') : this.statusText; }
  summary() { return '{"players":1}'; }
  new_game() { this.save = 'saved'; }
  load_game() {}
  open_guest_seats() {}
  joinCode: string | null | undefined;
  set_join_code(code?: string | null) { this.joinCode = code; }
  save_game() { if (!this.live) throw new Error('No game in progress'); return 'live save'; }
  live = true;
  configure_staging() {}
  close_staging() {}
  start_staging() { return '{"code":"not_ready","message":"Not everyone is ready."}'; }
  take_save() { const s = this.save; this.save = undefined; return s; }
  take_recordings() { const r = JSON.stringify(this.recordings); this.recordings = []; return r; }
  take_public_recordings() { const r = JSON.stringify(this.recordings.map((x) => `public ${x}`)); this.recordings = []; return r; }
  free() { this.freed = true; }
}

class FakeTunnel implements TunnelSocket {
  binaryType = '';
  readyState = 0;
  sent: Uint8Array[] = [];
  closed = false;
  onopen: TunnelSocket['onopen'] = null;
  onmessage: TunnelSocket['onmessage'] = null;
  onclose: TunnelSocket['onclose'] = null;
  onerror: TunnelSocket['onerror'] = null;
  constructor(readonly url: string) {}
  send(data: Uint8Array) { this.sent.push(data); }
  close() { this.closed = true; this.readyState = 3; }
  open() { this.readyState = 1; this.onopen?.({}); }
  drop() { this.readyState = 3; this.onclose?.({}); }
  frame(conn: number, kind: number, text = '') { this.onmessage?.({ data: encodeFrame(conn, kind, text).buffer }); }
  frames() { return this.sent.map((s) => decodeFrame(s)!); }
}

let engine: FakeEngine;
let tunnels: FakeTunnel[];
let calls: { url: string; method?: string; body?: string }[];
let statusCode: number;
let timers: { f: () => void; ms: number; every: boolean; dead: boolean }[];

const options = (over: Partial<ConstructorParameters<typeof BrowserHost>[0]> = {}) => ({
  relay: 'https://relay.test/',
  announce: { name: 'Ada', build: 'b', rules_version: 1, protocol: 1 },
  engine,
  socket: (url: string) => { const t = new FakeTunnel(url); tunnels.push(t); return t; },
  fetch: async (url: string, init?: { method?: string; body?: string }) => {
    calls.push({ url, method: init?.method, body: init?.body });
    const body = url.endsWith('/rooms') ? { room: `room${calls.length}`, host_key: 'k' } : {};
    const status = url.endsWith('/rooms') ? 200 : statusCode;
    return { ok: status < 300, status, json: async () => body, text: async () => JSON.stringify(body) };
  },
  now: () => 0,
  timers: {
    set: (f: () => void, ms: number) => { const t = { f, ms, every: false, dead: false }; timers.push(t); return t; },
    clear: (t: unknown) => { (t as { dead: boolean }).dead = true; },
    every: (f: () => void, ms: number) => { const t = { f, ms, every: true, dead: false }; timers.push(t); return t; },
    stop: (t: unknown) => { (t as { dead: boolean }).dead = true; },
  },
  ...over,
});
const live = (ms: number, every = false) => timers.filter((t) => t.ms === ms && t.every === every && !t.dead);
const fireReconnect = async () => {
  const t = timers.filter((x) => !x.every && !x.dead).at(-1)!;
  t.dead = true;
  t.f();
  await vi.waitFor(() => expect(tunnels.at(-1)!.closed || tunnels.at(-1)!.readyState === 0).toBe(true));
};

beforeEach(() => {
  engine = new FakeEngine();
  tunnels = [];
  calls = [];
  statusCode = 200;
  timers = [];
});

it('announces the room, opens the tunnel and pushes the status and lobby row', async () => {
  const host = new BrowserHost(options());
  const key = await host.start();
  expect(calls[0]).toMatchObject({ url: 'https://relay.test/rooms', method: 'POST' });
  expect(key).toEqual({ room: 'room1', host_key: 'k' });
  expect(tunnels[0].url).toBe('wss://relay.test/rooms/room1/host?key=k');
  expect(tunnels[0].binaryType).toBe('arraybuffer');
  tunnels[0].open();
  expect(tunnels[0].frames().map((f) => [f.conn, f.kind])).toEqual([[0, KIND_STATUS], [0, KIND_SUMMARY]]);
  expect(frameText(tunnels[0].frames()[0])).toBe(engine.statusText);
});

it('tells the relay only that the game is locked, never the code, and leaves the code in the engine', async () => {
  engine.set_join_code('WOLF42');
  const host = new BrowserHost(options({ announce: { name: 'Ada', build: 'b', rules_version: 1, protocol: 1, locked: true } }));
  await host.start();
  tunnels[0].open();
  const body = JSON.parse(calls[0].body!) as Record<string, unknown>;
  expect(body.locked).toBe(true);
  expect(calls.map((c) => c.body ?? '').join('')).not.toContain('WOLF42');
  const sent = tunnels[0].frames().map((f) => frameText(f)).join('');
  expect(sent).not.toContain('WOLF42');
  expect(engine.joinCode).toBe('WOLF42');
});

it('routes guest frames to the engine and its replies back, chunking long ones', async () => {
  const host = new BrowserHost(options());
  await host.start();
  const t = tunnels[0];
  t.open();
  t.sent = [];
  t.frame(9, KIND_OPEN);
  t.frame(9, KIND_DATA, 'hello');
  expect(t.frames().map((f) => [f.conn, f.kind, frameText(f)])).toEqual([[9, KIND_DATA, 'hello']]);
  t.sent = [];
  const big = 'x'.repeat(300 * 1024);
  t.frame(9, KIND_DATA, big.slice(0, 10)); // a guest's message is short; the reply below is what is long
  engine.out.set(1, { texts: [big], close: false });
  host.pump();
  const chunks = t.frames().slice(1).map((f) => JSON.parse(frameText(f)) as { type: string; data: string });
  expect(chunks.length).toBe(2);
  expect(chunks.every((c) => c.type === 'chunk')).toBe(true);
  expect(chunks.map((c) => c.data).join('')).toBe(big);
  t.sent = [];
  t.frame(9, KIND_DATA, 'quit');
  expect(t.frames().map((f) => [f.conn, f.kind])).toEqual([[9, KIND_DATA], [9, KIND_CLOSE]]);
  t.frame(9, KIND_DATA, 'ignored');
  t.frame(10, KIND_OPEN);
  t.frame(10, KIND_CLOSE);
  expect(engine.disconnected).toEqual([2]);
});

it('closes a guest that sends an oversized message', async () => {
  const host = new BrowserHost(options());
  await host.start();
  tunnels[0].open();
  tunnels[0].frame(3, KIND_OPEN);
  tunnels[0].sent = [];
  tunnels[0].frame(3, KIND_DATA, 'y'.repeat(70 * 1024));
  expect(tunnels[0].frames().map((f) => [f.conn, f.kind])).toEqual([[3, KIND_CLOSE]]);
  expect(engine.disconnected).toEqual([1]);
});

it('pings every 20 s, and reconnects to the same room with doubling backoff, dropping guests', async () => {
  const states: string[] = [];
  const host = new BrowserHost(options({ onTunnel: (s) => states.push(s) }));
  await host.start();
  tunnels[0].open();
  tunnels[0].frame(4, KIND_OPEN);
  tunnels[0].sent = [];
  live(PING_MS, true)[0].f();
  expect(tunnels[0].frames().map((f) => f.kind)).toEqual([KIND_PING]);
  tunnels[0].drop();
  expect(engine.disconnected).toEqual([1]);
  expect(live(PING_MS, true)).toHaveLength(0);
  expect(timers.filter((t) => !t.every && !t.dead).map((t) => t.ms)).toEqual([MIN_BACKOFF_MS]);
  await fireReconnect();
  expect(tunnels[1].url).toBe(tunnels[0].url);
  tunnels[1].drop(); // never opened: ask the relay whether the room is still there (200: it is)
  expect(timers.filter((t) => !t.every && !t.dead).map((t) => t.ms)).toEqual([2 * MIN_BACKOFF_MS]);
  await fireReconnect();
  expect(calls.filter((c) => c.url.endsWith('/api/status'))).toHaveLength(1);
  expect(tunnels[2].url).toBe(tunnels[0].url);
  for (let i = 0; i < 8; i++) tunnels.at(-1)!.drop(), await fireReconnect();
  expect(Math.max(...timers.filter((t) => !t.every).map((t) => t.ms))).toBe(MAX_BACKOFF_MS);
  tunnels.at(-1)!.open();
  expect(states).toContain('reconnecting');
  expect(states.at(-1)).toBe('open');
  host.stop();
});

it('opens a new room when the relay has forgotten the old one', async () => {
  let room = '';
  const host = new BrowserHost(options({ room: { room: 'old', host_key: 'k0' }, onTunnel: (_s, k) => { room = k.room; } }));
  await host.start();
  expect(calls).toHaveLength(0);
  statusCode = 404;
  tunnels[0].drop();
  await fireReconnect();
  await vi.waitFor(() => expect(tunnels).toHaveLength(2));
  expect(calls.map((c) => c.url)).toEqual(['https://relay.test/rooms/old/api/status', 'https://relay.test/rooms']);
  expect(tunnels[1].url).toMatch(/\/rooms\/room2\/host\?key=k$/);
  expect(host.room?.room).toBe('room2');
  expect(room).toBe('room2');
});

it('says bye on stop, closes the tunnel and the in-tab sockets, and frees the engine', async () => {
  const host = new BrowserHost(options());
  await host.start();
  tunnels[0].open();
  const sock = host.socket();
  const closed = vi.fn();
  sock.onclose = closed;
  await Promise.resolve();
  tunnels[0].sent = [];
  host.stop();
  expect(tunnels[0].frames().map((f) => f.kind)).toEqual([KIND_BYE]);
  expect(tunnels[0].closed).toBe(true);
  await Promise.resolve();
  expect(closed).toHaveBeenCalled();
  expect(engine.freed).toBe(true);
  expect(timers.every((t) => t.dead)).toBe(true);
});

it("the host's own socket and admin calls reach the engine in the tab", async () => {
  const saves: string[] = [];
  const recordings: string[] = [];
  const host = new BrowserHost(options({ onSave: (s) => saves.push(s), onRecording: (r) => recordings.push(r) }));
  await host.start();
  const sock = host.socket();
  const got: string[] = [];
  sock.onmessage = (ev) => got.push(ev.data as string);
  const opened = new Promise((r) => { sock.onopen = r; });
  await opened;
  sock.send('hi');
  await Promise.resolve();
  expect(got).toEqual(['hi']);

  const status = await host.fetch('/api/status');
  expect(JSON.parse(await status.text())).toMatchObject({ is_local: true });
  engine.recordings = ['{"history":1}'];
  expect((await host.fetch('/api/game', { method: 'POST', body: '{}' })).ok).toBe(true);
  expect(saves).toEqual(['saved']);
  expect(recordings).toEqual(['{"history":1}']);
  const refused = await host.fetch('https://relay.test/rooms/x/api/staging/start', { method: 'POST' });
  expect(refused.status).toBe(409);
  expect(JSON.parse(await refused.text())).toMatchObject({ code: 'not_ready', message: 'Not everyone is ready.' });
  expect((await host.fetch('/api/nope')).status).toBe(404);

  const closed = new Promise((r) => { sock.onclose = r; });
  sock.close();
  await closed;
  expect(engine.disconnected).toEqual([1]);
});

/** A relay that answers POST /rooms with a refusal. */
const refusing = (status: number, error: string, retryAfter?: string, readable = true) => ({
  fetch: async () => ({
    ok: false,
    status,
    // A browser on another origin gets no Retry-After unless the relay exposes it: `readable` false hides it, as there.
    headers: { get: (n: string) => (readable && n.toLowerCase() === 'retry-after' ? retryAfter ?? null : null) },
    json: async () => ({ error }),
    text: async () => JSON.stringify({ error }),
  }),
});

const refusalOf = async (over: Parameters<typeof options>[0]) => {
  const host = new BrowserHost(options(over));
  const e = await host.start().then(() => null, (x: unknown) => x);
  expect(e).toBeInstanceOf(RelayRefusal);
  expect(tunnels).toHaveLength(0);
  return e as RelayRefusal;
};

it.each([
  [429, 'You are opening games too quickly.', '300', 'too_fast', 'You have opened too many games lately. Try again in 5 minutes.'],
  [429, 'You are opening games too quickly.', '1', 'too_fast', 'You have opened too many games lately. Try again in 1 minute.'],
  [429, 'You are opening games too quickly.', '601', 'too_fast', 'You have opened too many games lately. Try again in 11 minutes.'],
  [429, 'You already have 2 games open. Close one before hosting another.', undefined, 'too_many', 'Your internet connection already has 2 games open. Close one before hosting another.'],
  [429, 'Too many games are open from your network. Try again later.', undefined, 'network', 'Too many games are open from your network. Try again later.'],
  [503, 'This lobby is full. Try again in a few minutes.', undefined, 'full', 'The game list is full right now. Try again in a few minutes.'],
  [503, 'Hosting new games is paused. Try again later.', undefined, 'closed', 'Hosting new games is paused. Try again later.'],
  [400, 'invalid room announcement', undefined, 'other', 'Could not open your game. Try again.'],
] as const)('maps a %i from the relay (%s, Retry-After %s) to %s', async (status, error, retryAfter, kind, message) => {
  const e = await refusalOf(refusing(status, error, retryAfter));
  expect(e.kind).toBe(kind);
  expect(e.message).toBe(message);
});

it.each([
  ['You are opening games too quickly. Try again in a few minutes.', 'too_fast', 'You have opened too many games lately. Try again in a few minutes.'],
  ['You already have 2 games open. Close one before hosting another.', 'too_many', 'Your internet connection already has 2 games open. Close one before hosting another.'],
] as const)('tells the two 429s apart by their text when Retry-After cannot be read (%s)', async (error, kind, message) => {
  const e = await refusalOf(refusing(429, error, '300', false));
  expect(e.kind).toBe(kind);
  expect(e.message).toBe(message);
  expect(e.retryAfterS).toBeNull();
});

it('says a retry is useless only for the refusals that a retry cannot fix', () => {
  expect(retryLine('offline')).toBe('Try again');
  expect(retryLine('full')).toBe('Try again');
  expect(retryLine('other')).toBe('Try again');
  expect(retryLine('too_fast', 130)).toBe('Try again in 3 minutes');
  expect(retryLine('too_fast', null)).toBe('Try again in a few minutes');
  expect(retryLine('too_many')).toBe('Close one of your games first');
  expect(retryLine('closed')).toBe('Paused for now');
  expect(['too_fast', 'too_many', 'network', 'closed', 'offline', 'full', 'other'].filter((k) => retryWouldFail(k as never))).toEqual(['too_fast', 'too_many', 'network', 'closed']);
});

it('treats a network failure as offline with the connection advice', async () => {
  const e = await refusalOf({ fetch: async () => { throw new TypeError('Failed to fetch'); } });
  expect(e.kind).toBe('offline');
  expect(e.message).toBe(refusalMessage('offline'));
  expect(e.message).toMatch(/internet connection/);
});

it('never shows the relay or its status code in a refusal message', () => {
  for (const kind of ['too_fast', 'too_many', 'network', 'full', 'closed', 'offline', 'other'] as const) {
    expect(refusalMessage(kind, 120)).not.toMatch(/relay|socket|429|503|rate limit/i);
  }
});

it('never sends the host name to the public game list', async () => {
  let body = '';
  const host = new BrowserHost(options({
    announce: { name: '  Ada \u{1F600} of   Rome  ', build: 'b', rules_version: 1, protocol: 1 },
    fetch: async (_url, init) => {
      body = (init as { body?: string }).body ?? '';
      return { ok: true, status: 200, json: async () => ({ room: 'r', host_key: 'k' }), text: async () => '' };
    },
  }));
  await host.start();
  expect(JSON.parse(body).name).toBe('');
  expect(body).not.toContain('Ada');
  host.stop();
});

it('tells the host when the relay refuses to list a forgotten room again, and keeps trying', async () => {
  const seen: RelayRefusal[] = [];
  let announceCalls = 0;
  const host = new BrowserHost(options({
    room: { room: 'old', host_key: 'k0' },
    onRefused: (r) => seen.push(r),
    fetch: async (_url: string, init?: { method?: string }) => {
      if (init?.method === 'POST') {
        announceCalls++;
        return { ok: false, status: 429, headers: { get: () => '120' }, json: async () => ({ error: 'x' }), text: async () => '' };
      }
      return { ok: false, status: 404, json: async () => ({}), text: async () => '' };
    },
  }));
  await host.start();
  tunnels[0].drop();
  const t = timers.filter((x) => !x.every && !x.dead).at(-1)!;
  t.dead = true;
  t.f();
  await vi.waitFor(() => expect(seen).toHaveLength(1));
  expect(announceCalls).toBe(1);
  expect(seen[0].message).toBe('You have opened too many games lately. Try again in 2 minutes.');
  await vi.waitFor(() => expect(timers.some((t) => !t.every && !t.dead)).toBe(true));
  host.stop();
});

it('saveGame hands over the live save on demand, or null when there is no game', () => {
  const host = new BrowserHost(options());
  expect(host.saveGame()).toBe('live save');
  engine.live = false;
  expect(host.saveGame()).toBeNull();
});

it('a build that uploads games takes each history without typed names', async () => {
  const recordings: string[] = [];
  const host = new BrowserHost(options({ publicRecordings: true, onRecording: (r) => recordings.push(r) }));
  await host.start();
  engine.recordings = ['{"history":1}'];
  expect((await host.fetch('/api/game', { method: 'POST', body: '{}' })).ok).toBe(true);
  expect(recordings).toEqual(['public {"history":1}']);
  host.stop();
});
