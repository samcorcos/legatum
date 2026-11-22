// @vitest-environment happy-dom
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { RemoteClient, defaultWsUrl, fetchStatus, serverBase, setServerBase, wsUrlFor } from './remote';
import { StagingClient } from './staging';
import { FakeSocket } from './test/fakeSocket';
import { roomBase, versionProblem, type RoomInfo } from './platform/relayLobby';
import { inHostedWindow, online, openHostedWindow, setOnlineApi, type OnlineApi } from './platform/online';
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
  it('is same-origin by default, exactly as before', () => {
    expect(serverBase()).toBe('');
    expect(defaultWsUrl()).toBe(`ws://${location.host}/ws`);
    expect(wsUrlFor('')).toBe(`ws://${location.host}/ws`);
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
    await fetchStatus('', get);
    expect(get.mock.calls[0][0]).toBe('/api/status');
    setServerBase(ROOM);
    await fetchStatus(undefined, get);
    expect(get.mock.calls[1][0]).toBe(`${ROOM}/api/status`);
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

  it('keeps plain same-origin sockets unwrapped', () => {
    const made: FakeSocket[] = [];
    vi.stubGlobal('WebSocket', class extends FakeSocket {
      constructor(url: string) { super(url); made.push(this); }
    });
    new RemoteClient(0, 'Ada').connect();
    expect(made[0].url).toBe(`ws://${location.host}/ws`);
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

describe('host start and return', () => {
  const api = (over: Partial<OnlineApi> = {}): OnlineApi => ({
    info: async () => ({ relay: 'https://r', build: 'b', rules_version: 1, protocol: 1 }),
    hostStart: async () => ({ url: 'http://127.0.0.1:5555/', room: 'room1' }),
    hostStop: async () => {},
    ...over,
  });

  it('can be replaced for tests and restored', async () => {
    const fake = api();
    setOnlineApi(fake);
    expect(online()).toBe(fake);
    setOnlineApi();
    expect(await online().info()).toBeNull(); // not in the desktop app
  });

  it('opens the host window on the loopback server with ?host=1', () => {
    const assign = vi.fn();
    vi.stubGlobal('location', { ...location, assign, search: '', hostname: 'localhost' });
    openHostedWindow('http://127.0.0.1:5555/');
    expect(assign).toHaveBeenCalledWith('http://127.0.0.1:5555/?host=1');
  });

  it('boot lands the host in the game room, and Leave returns to the bundled app', async () => {
    const stop = vi.fn(async () => { throw new Error('refused'); });
    setOnlineApi(api({ hostStop: stop }));
    const loc = { ...location, assign: vi.fn(), href: '', search: '?host=1', hostname: '127.0.0.1', host: '127.0.0.1:5555', protocol: 'http:' };
    vi.stubGlobal('location', loc);
    const status = { mode: 'server', has_game: false, turn: 0, seats: [], lan_urls: [], is_local: true, staging: null };
    vi.stubGlobal('fetch', vi.fn(async () => ({ ok: true, status: 200, text: async () => JSON.stringify(status) })));
    await useGame.getState().boot();
    expect(useGame.getState()).toMatchObject({ screen: 'setup', setupMode: 'host' });
    expect(inHostedWindow()).toBe(true);
    useGame.getState().goto('title');
    await Promise.resolve();
    await Promise.resolve();
    expect(stop).toHaveBeenCalled();
    expect(loc.href).toBe('tauri://localhost/'); // only because the stop call failed
    expect(useGame.getState().screen).toBe('setup');
    expect(inHostedWindow()).toBe(false);
  });

  it('a plain LAN browser on the host machine is untouched', async () => {
    const assign = vi.fn();
    vi.stubGlobal('location', { ...location, assign, search: '', hostname: '127.0.0.1', host: '127.0.0.1:7878', protocol: 'http:' });
    expect(inHostedWindow()).toBe(false);
    useGame.getState().goto('title');
    expect(assign).not.toHaveBeenCalled();
    expect(useGame.getState().screen).toBe('title');
  });

  beforeEach(() => {
    try { sessionStorage.clear(); } catch { /* none */ }
  });
});

describe('a relayed game that goes away', () => {
  const status = { mode: 'server', has_game: false, turn: 0, seats: [], lan_urls: [], is_local: false, staging: null };
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
    expect(useGame.getState()).toMatchObject({ screen: 'title', error: 'This game has ended.', lan: null });
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
