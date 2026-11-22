import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { useGame } from './store';
import { rememberedSeat } from './remote';
import { FakeSocket } from './test/fakeSocket';
import { hostThisTab, joinedFromThisTab } from './test/fakeHosting';
import { setOnlineApi } from './platform/online';

vi.mock('./net/hosting', async () => (await import('./test/fakeHosting')).fakeHostingModule());

let sockets: FakeSocket[];
beforeEach(() => {
  sockets = [];
  vi.stubGlobal('WebSocket', class extends FakeSocket {
    constructor(url: string) { super(url); sockets.push(this); }
  });
  const data = new Map([['legatum.lan.seat', JSON.stringify({ seat: 3, name: 'Alice', gameId: null, roomId: 'room', token: 'credential' })]]);
  vi.stubGlobal('sessionStorage', {
    getItem: (key: string) => data.get(key) ?? null,
    setItem: (key: string, value: string) => data.set(key, value),
    removeItem: (key: string) => data.delete(key),
  });
  useGame.setState(useGame.getInitialState(), true);
});
afterEach(() => {
  useGame.getState().exitStaging();
  useGame.getState().leaveOnline();
  setOnlineApi();
  vi.unstubAllGlobals();
});

it.each([
  { seat: 3, isLocal: false, host: false },
  { seat: 3, isLocal: true, host: false },
  { seat: 0, isLocal: false, host: false },
  { seat: 0, isLocal: true, host: true },
])('restores seat $seat (local: $isLocal) on boot and joins a missed game handoff with the exact credential', async ({ seat, isLocal, host }) => {
  sessionStorage.setItem('legatum.lan.seat', JSON.stringify({ seat, name: 'Alice', gameId: null, roomId: 'room', token: 'credential' }));
  vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ ok: true, text: async () => JSON.stringify({ mode: 'server', has_game: false, turn: 0, seats: [], is_local: isLocal, staging: null }) }));
  if (isLocal) hostThisTab(); else joinedFromThisTab();
  await useGame.getState().boot();
  expect(useGame.getState().screen).toBe(host ? 'setup' : 'staging');
  if (host) expect(useGame.getState().setupMode).toBe('host');
  sockets[0].open();
  expect(sockets[0].sent).toEqual([{ type: 'staging_watch', ...(host ? { host: true } : {}) }, { type: 'staging_take', seat, name: 'Alice', token: 'credential' }]);
  sockets[0].push({ type: 'staging_started', game_id: 'game', seat: 1, token: 'credential' });
  expect(rememberedSeat()).toEqual({ seat: 1, name: 'Alice', gameId: 'game', token: 'credential' });
  sockets[1].open();
  expect(sockets[1].sent).toEqual([{ type: 'join', seat: 1, name: 'Alice', token: 'credential' }]);
});

it('passes the handoff credential directly when browser storage becomes unavailable', () => {
  useGame.getState().enterStaging();
  sockets[0].open();
  vi.stubGlobal('sessionStorage', { getItem: () => { throw new Error('storage disabled'); }, setItem: () => { throw new Error('storage disabled'); } });
  sockets[0].push({ type: 'staging_started', game_id: 'game', seat: 1, token: 'credential' });
  sockets[1].open();
  expect(sockets[1].sent).toEqual([{ type: 'join', seat: 1, name: 'Alice', token: 'credential' }]);
});


it('keeps an explicitly left game at the title on reload while retaining manual rejoin credentials', async () => {
  sessionStorage.setItem('legatum.lan.seat', JSON.stringify({ seat: 1, name: 'Alice', gameId: 'game', token: 'credential', autoJoin: false }));
  vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ ok: true, text: async () => JSON.stringify({ mode: 'server', has_game: true, turn: 1, seats: [{ seat: 1, connected: false }], is_local: false, staging: null }) }));
  await useGame.getState().boot();
  expect(useGame.getState().screen).toBe('title');
  expect(sockets).toHaveLength(0);
  useGame.getState().joinSeat(1, 'Alice');
  sockets[0].open();
  expect(sockets[0].sent).toEqual([{ type: 'join', seat: 1, name: 'Alice', token: 'credential' }]);
});

it.each([false, true])('auto-rejoins a reloaded game tab using sessionStorage when connected=%s', async (connected) => {
  sessionStorage.setItem('legatum.lan.seat', JSON.stringify({ seat: 1, name: 'Alice', gameId: 'game', token: 'credential' }));
  const localStorage = { getItem: vi.fn(), setItem: vi.fn() };
  vi.stubGlobal('localStorage', localStorage);
  vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ ok: true, text: async () => JSON.stringify({ mode: 'server', has_game: true, turn: 1, seats: [{ seat: 1, reserved: true, connected }], is_local: false, staging: null }) }));
  joinedFromThisTab();
  await useGame.getState().boot();
  sockets[0].open();
  expect(sockets[0].sent).toEqual([{ type: 'join', seat: 1, name: 'Alice', token: 'credential' }]);
  expect(localStorage.getItem).not.toHaveBeenCalled();
  expect(localStorage.setItem).not.toHaveBeenCalled();
});

it('does not automatically claim a seat the host released when this tab still has the old credential', async () => {
  sessionStorage.setItem('legatum.lan.seat', JSON.stringify({ seat: 1, name: 'Alice', gameId: 'game', token: 'old-token' }));
  vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ ok: true, text: async () => JSON.stringify({ mode: 'server', has_game: true, turn: 1, seats: [{ seat: 1, reserved: false, connected: false }], is_local: false, staging: null }) }));
  joinedFromThisTab();
  await useGame.getState().boot();
  expect(useGame.getState().screen).toBe('join');
  expect(sockets).toHaveLength(0);
  useGame.getState().joinSeat(1, 'Alice');
  sockets[0].open();
  expect(sockets[0].sent).toEqual([{ type: 'game_take', seat: 1 }]);
});


it('bootstraps the host seat after an explicit new game even when an older token is remembered', async () => {
  sessionStorage.setItem('legatum.lan.seat', JSON.stringify({ seat: 0, name: 'Host', gameId: 'old', token: 'old-token', autoJoin: false }));
  vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ ok: true, text: async () => JSON.stringify({ mode: 'server', has_game: true, turn: 1, seats: [{ seat: 0, reserved: false, connected: false }], is_local: true, staging: null }) }));
  await useGame.getState().hostNewGame({ seed: '7', width: 24, height: 20, turn_limit: 100, players: [
    { name: 'Host', faction: 'India', team: 0, is_human: true },
    { name: 'Guest', faction: 'Byzantium', team: 1, is_human: true },
  ] });
  sockets[0].open();
  expect(sockets[0].sent).toEqual([{ type: 'game_take', seat: 0 }]);
});
