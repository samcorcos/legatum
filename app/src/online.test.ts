// @vitest-environment happy-dom
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { RemoteClient, defaultWsUrl, fetchStatus, hostingHere, serverBase, setLocalEndpoint, setServerBase, wsUrlFor, type FetchLike, type SocketLike } from './remote';
import { StagingClient } from './staging';
import { FakeSocket } from './test/fakeSocket';
import { roomBase, versionProblem, type RoomInfo } from './platform/relayLobby';
import { RULES_VERSION, online, setOnlineApi, type OnlineApi } from './platform/online';
import { useGame } from './store';

const ROOM = 'https://relay.example/rooms/abc';

afterEach(() => {
  setServerBase('');
  setOnlineApi();
  useGame.setState(useGame.getInitialState(), true);
  vi.unstubAllGlobals();
  try { sessionStorage.clear(); } catch { /* none */ }
});

describe('server base', () => {
  it('is empty outside an online game, and there is then no game to ask about', async () => {
    expect(serverBase()).toBe('');
    const get = vi.fn();
    expect(await fetchStatus(undefined, get)).toBeNull();
    expect(get).not.toHaveBeenCalled();
  });

  it('turns https into wss and http into ws, and trims slashes', () => {
    expect(wsUrlFor(ROOM)).toBe('wss://relay.example/rooms/abc/ws');
    expect(wsUrlFor('http://127.0.0.1:9000/rooms/x')).toBe('ws://127.0.0.1:9000/rooms/x/ws');
    setServerBase(`${ROOM}/`);
    expect(serverBase()).toBe(ROOM);
    expect(defaultWsUrl()).toBe('wss://relay.example/rooms/abc/ws');
  });

  it('reads status from <base>/api/status, defaulting to the current base', async () => {
    const get = vi.fn(async (_url: string) => ({ ok: true, status: 200, text: async () => JSON.stringify({ mode: 'server', seats: [] }) }));
    setServerBase(ROOM);
    await fetchStatus(undefined, get);
    expect(get.mock.calls[0][0]).toBe(`${ROOM}/api/status`);
  });

  it('a hosting tab asks its own game, and its sockets are the in-tab connection', async () => {
    const local = vi.fn<FetchLike>(async () => ({ ok: true, status: 200, text: async () => JSON.stringify({ mode: 'server', seats: [], is_local: true }) }));
    const sock = new FakeSocket('local') as unknown as SocketLike;
    const made = vi.fn();
    vi.stubGlobal('WebSocket', made);
    setLocalEndpoint({ fetch: local, socket: () => sock });
    try {
      expect(hostingHere()).toBe(true);
      expect(await fetchStatus()).toMatchObject({ is_local: true });
      expect(local.mock.calls[0][0]).toBe('/api/status');
      const rc = new RemoteClient(0, 'Ada');
      rc.connect();
      expect(made).not.toHaveBeenCalled();
    } finally {
      setLocalEndpoint(null);
    }
    expect(hostingHere()).toBe(false);
  });

  it('points RemoteClient and the staging watcher at the room, through the chunk reassembler', () => {
    const made: FakeSocket[] = [];
    vi.stubGlobal('WebSocket', class extends FakeSocket {
      constructor(url: string) { super(url); made.push(this); }
    });
    setServerBase(ROOM);
    const rc = new RemoteClient(0, 'Ada');
    rc.connect();
    new StagingClient({}).connect();
    expect(made.map((m) => m.url)).toEqual(['wss://relay.example/rooms/abc/ws', 'wss://relay.example/rooms/abc/ws']);
    // A chunked view reaches the client as one message.
    made[0].readyState = 1;
    made[0].push({ type: 'welcome', seat: 0, game_id: 'g', orders: [] });
    const view = JSON.stringify({ type: 'view', view: { turn: 3, me: 0, units: [], cities: [], players: [], finished: false } });
    const half = Math.floor(view.length / 2);
    const piece = (i: number, data: string) => ({ data: JSON.stringify({ type: 'chunk', id: 1, i, n: 2, data }) });
    made[0].onmessage?.(piece(0, view.slice(0, half)));
    expect(rc.hasView()).toBe(false);
    made[0].onmessage?.(piece(1, view.slice(half)));
    expect(rc.hasView()).toBe(true);
    expect(rc.view().turn).toBe(3);
  });

});

describe('relay lobby helpers', () => {
  const info = { relay: 'https://r', build: '8.9.74', rules_version: 12, protocol: 1 };
  const room: RoomInfo = { room: 'x', name: 'Ada', build: '8.9.74', rules_version: 12, protocol: 1, players: 2, humans: 2, open_seats: 1, phase: 'staging', turn: 0, updated_ms: 0 };
  it('builds the room base', () => expect(roomBase('https://r/', 'abc')).toBe('https://r/rooms/abc'));
  it('accepts the same version', () => expect(versionProblem(room, info)).toBeNull());
  it('names both builds when they differ', () =>
    expect(versionProblem({ ...room, build: '8.9.70' }, info)).toBe('This game is on version 8.9.70 and you have 8.9.74. Update the older one to play together.'));
});

describe('online info', () => {
  it('can be replaced for tests and restored', async () => {
    const fake: OnlineApi = { info: async () => ({ relay: 'https://r', build: 'b', rules_version: 1, protocol: 1 }) };
    setOnlineApi(fake);
    expect(online()).toBe(fake);
    setOnlineApi();
    expect(await online().info()).toBeNull(); // the test build bakes in no relay
  });

  it('knows the engine rules version without loading the engine, and it matches the engine', async () => {
    const { loadWasmModule } = await import('./wasmClient');
    const mod = (await loadWasmModule()) as unknown as { online_rules_version: () => number };
    expect(RULES_VERSION).toBe(mod.online_rules_version());
  });

  beforeEach(() => {
    try { sessionStorage.clear(); } catch { /* none */ }
  });
});

describe('a relayed game that goes away', () => {
  const status = { mode: 'server', has_game: false, turn: 0, seats: [], is_local: false, staging: null };

  async function joined(codeAfter: () => number) {
    vi.useFakeTimers();
    vi.stubGlobal('fetch', vi.fn(async () => {
      const code = codeAfter();
      return { ok: code === 200, status: code, text: async () => JSON.stringify(status) };
    }));
    expect(await useGame.getState().joinOnline('https://relay.example', 'abc')).toBeNull();
    expect(serverBase()).toBe(ROOM);
  }
  afterEach(() => { vi.useRealTimers(); });

  it('says the game has ended on 404 and returns to the title', async () => {
    let code = 200;
    await joined(() => code);
    code = 404;
    await vi.advanceTimersByTimeAsync(4100);
    expect(serverBase()).toBe('');
    expect(useGame.getState()).toMatchObject({ screen: 'title', error: 'This game has ended.', net: null });
  });

  it('says the host is away on 503 and keeps waiting', async () => {
    let code = 200;
    await joined(() => code);
    code = 503;
    await vi.advanceTimersByTimeAsync(4100);
    expect(useGame.getState().error).toBe('The host is not connected right now.');
    expect(serverBase()).toBe(ROOM);
    useGame.getState().goto('title');
  });

  it('reports a game that is already gone when joining', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => ({ ok: false, status: 404, text: async () => '' })));
    expect(await useGame.getState().joinOnline('https://relay.example', 'abc')).toBe('This game has ended.');
    expect(serverBase()).toBe('');
  });
});

describe('seat action wording', () => {
  it('names the host on a relayed game and never says server', async () => {
    const { connectionFirst } = await import('./platform/relayLobby');
    expect(connectionFirst('leave')).toBe('You can leave once you are connected again.');
    setServerBase(ROOM);
    expect(connectionFirst('leave')).toBe('You can leave once the host is back.');
    expect(connectionFirst('open')).not.toMatch(/server/i);
  });
});
