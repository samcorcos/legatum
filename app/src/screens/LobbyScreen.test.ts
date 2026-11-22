// @vitest-environment happy-dom
import { act, createElement } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterAll, afterEach, beforeEach, expect, it, vi } from 'vitest';
import { useGame } from '../store';
import type { StagingConfig, StagingState } from '../staging';
import { rememberedSeat, rememberSeat, type ServerStatus } from '../remote';
import { FakeSocket } from '../test/fakeSocket';
import { LobbyScreen } from './LobbyScreen';
import { hostThisTab } from '../test/fakeHosting';
import { setOnlineApi } from '../platform/online';

vi.mock('../net/hosting', async () => (await import('../test/fakeHosting')).fakeHostingModule());

vi.mock('./HexBackdrop', () => ({ HexBackdrop: () => null }));

function deferred() {
  let resolve!: () => void;
  const promise = new Promise<void>((done) => { resolve = done; });
  return { promise, resolve };
}

// Keep the real lobby and store navigation, with just the Host Game shortcut on the title.
function HostFlow() {
  const screen = useGame((s) => s.screen);
  return screen === 'setup'
    ? createElement(LobbyScreen)
    : createElement('button', { onClick: () => useGame.getState().openSetup('host') }, 'Host Game');
}

let container: HTMLDivElement;
let root: Root;
let sockets: FakeSocket[];

// Nothing in this file may reach the network. happy-dom's own fetch and WebSocket go to
// 127.0.0.1:3000, and a status poll or reconnect still pending when a test ends used to fire
// after `unstubAllGlobals`, so on a busy machine these tests failed with ECONNREFUSED. Tests
// stub `fetch` as they need; this offline default answers anything else, before and after.
const offlineFetch = async () => new Response('{"message":"offline in tests"}', {
  status: 503, headers: { 'content-type': 'application/json' },
});
class OfflineSocket extends FakeSocket {}
const real = { fetch: globalThis.fetch, WebSocket: globalThis.WebSocket };
afterAll(() => {
  globalThis.fetch = real.fetch;
  globalThis.WebSocket = real.WebSocket;
});

beforeEach(() => {
  vi.useFakeTimers();
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  vi.stubGlobal('fetch', vi.fn(offlineFetch));
  sockets = [];
  vi.stubGlobal('WebSocket', class extends FakeSocket {
    constructor(url: string) { super(url); sockets.push(this); }
  });
  sessionStorage.clear();
  useGame.setState(useGame.getInitialState(), true);
  useGame.setState({
    screen: 'setup', setupMode: 'host',
    net: {
      status: { mode: 'server', has_game: false, turn: 0, seats: [], is_local: true, staging: null },
      mySeat: null, seats: [], conn: 'closed',
    },
  });
  container = document.createElement('div');
  document.body.append(container);
  root = createRoot(container);
});

afterEach(async () => {
  await act(async () => {
    root.unmount();
    useGame.getState().exitStaging();
  });
  container.remove();
  useGame.getState().leaveOnline();
  setOnlineApi();
  sessionStorage.clear();
  vi.useRealTimers();
  vi.unstubAllGlobals();
  // Work still pending from the test stays offline too (restored in afterAll).
  globalThis.fetch = offlineFetch as typeof fetch;
  globalThis.WebSocket = OfflineSocket as unknown as typeof WebSocket;
});

function button(label: string): HTMLButtonElement {
  const found = [...container.querySelectorAll('button')].find((b) => b.textContent?.trim() === label);
  expect(found, `button ${label}`).toBeDefined();
  return found!;
}

it('waits for the settings response and room deletion before Back can reopen Host Game', async () => {
  const settingsResponse = deferred();
  const closeResponse = deferred();
  let serverRoom: StagingConfig | null = null;
  const requests: string[] = [];
  vi.stubGlobal('fetch', vi.fn(async (_url: string, init: RequestInit) => {
    const method = init.method!;
    requests.push(method);
    if (method === 'POST') {
      serverRoom = JSON.parse(init.body as string) as StagingConfig;
      // The server has saved an edit, but its HTTP response is still in flight.
      if (requests.length === 2) await settingsResponse.promise;
    } else if (method === 'DELETE') {
      await closeResponse.promise;
      serverRoom = null;
    }
    return { ok: true, text: async () => '{}' };
  }));
  await act(async () => root.render(createElement(HostFlow)));
  await act(async () => {
    sockets[0].open();
    await vi.advanceTimersByTimeAsync(250);
  });
  expect(requests).toEqual(['POST']);
  expect(sockets[0].sent).toContainEqual({ type: 'staging_take', seat: 0, name: 'Player 1' });
  await act(async () => {
    sockets[0].push({ type: 'staging_taken', room_id: 'old-room', seat: 0, token: 'host-secret' });
    const { seed, ...config } = serverRoom!;
    sockets[0].push({ type: 'staging', state: { room_id: 'old-room', seed, config,
      occupants: [{ name: 'Player 1', faction: 'India', connected: true, ready: false, is_host: true }],
    } });
  });
  expect(rememberedSeat()?.roomId).toBe('old-room');
  await act(async () => container.querySelector<HTMLButtonElement>('[aria-label="Map size"] button')!.click());
  await act(async () => vi.advanceTimersByTimeAsync(250));
  expect(requests).toEqual(['POST', 'POST']);

  const back = button('← Back');
  await act(async () => {
    back.click();
    back.click(); // Repeated clicks must not queue an extra DELETE.
  });
  expect(useGame.getState().screen).toBe('setup');
  expect(requests).toEqual(['POST', 'POST']);

  await act(async () => {
    settingsResponse.resolve();
    await vi.advanceTimersByTimeAsync(0);
  });
  expect(requests).toEqual(['POST', 'POST', 'DELETE']);
  expect(useGame.getState().screen).toBe('setup');
  expect(serverRoom).not.toBeNull();
  expect(rememberedSeat()?.roomId).toBe('old-room');

  await act(async () => {
    closeResponse.resolve();
    await vi.advanceTimersByTimeAsync(0);
  });
  expect(serverRoom).toBeNull();
  expect(useGame.getState().screen).toBe('title');
  expect(useGame.getState().busy).toBeNull();
  expect(rememberedSeat()).toBeNull();

  await act(async () => button('Host Game').click());
  await act(async () => {
    sockets[1].open();
    await vi.advanceTimersByTimeAsync(250);
  });
  expect(sockets[1].sent).toContainEqual({ type: 'staging_take', seat: 0, name: 'Player 1' });
  expect(requests).toEqual(['POST', 'POST', 'DELETE', 'POST']);
  expect(serverRoom).not.toBeNull();
  // Later timers (the room keepalive pings every 20 s) send nothing more to the room.
  await act(async () => vi.advanceTimersByTimeAsync(60_000));
  expect(serverRoom).not.toBeNull();
  expect(requests).toEqual(['POST', 'POST', 'DELETE', 'POST']);
});

it.each(['failed deletion', 'different room', 'game handoff'] as const)('keeps the saved credential on Back after a %s', async (outcome) => {
  const closeResponse = deferred();
  let serverRoom!: StagingState;
  const fetch = vi.fn(async (_url: string, init: RequestInit) => {
    if (init.method === 'POST') {
      const { seed, ...config } = JSON.parse(init.body as string) as StagingConfig;
      serverRoom = { room_id: 'old-room', seed, config, occupants: [] };
    }
    if (init.method === 'DELETE') {
      await closeResponse.promise;
      if (outcome === 'failed deletion') return { ok: false, text: async () => 'Offline' };
    }
    return { ok: true, text: async () => '{}' };
  });
  vi.stubGlobal('fetch', fetch);
  await act(async () => root.render(createElement(HostFlow)));
  await act(async () => {
    sockets[0].open();
    await vi.advanceTimersByTimeAsync(250);
    sockets[0].push({ type: 'staging_taken', room_id: 'old-room', seat: 0, token: 'host-secret' });
    sockets[0].push({ type: 'staging', state: serverRoom });
  });
  await act(async () => button('← Back').click());
  const saved = outcome === 'failed deletion' ? rememberedSeat()!
    : outcome === 'different room'
      ? { seat: 0, name: 'Host', roomId: 'other-room', gameId: null, token: 'new-secret' }
      : { seat: 0, name: 'Host', gameId: 'game', token: 'game-secret' };
  rememberSeat(saved);
  await act(async () => closeResponse.resolve());
  expect(useGame.getState().screen).toBe('title');
  expect(rememberedSeat()).toEqual(saved);
  if (outcome === 'failed deletion') {
    await act(async () => button('Host Game').click());
    expect(container.textContent).toContain('Restoring the game room…');
    await act(async () => {
      sockets[1].open();
      sockets[1].push({ type: 'staging_taken', room_id: 'old-room', seat: 0, token: 'host-secret' });
      sockets[1].push({ type: 'staging', state: serverRoom });
      await vi.advanceTimersByTimeAsync(1000);
    });
    expect(container.querySelector<HTMLInputElement>('[aria-label="Map seed"]')?.value).toBe(serverRoom.seed);
    expect(rememberedSeat()).toEqual(saved);
    expect(fetch.mock.calls.map(([, init]) => init.method)).toEqual(['POST', 'DELETE']);
  }
});

it.each([
  { arrival: 'before', optOut: false, width: 40, height: 26, size: 'Small' },
  { arrival: 'after', optOut: false, width: 40, height: 26, size: 'Small' },
  { arrival: 'after', optOut: true, width: 40, height: 26, size: 'Small' },
  { arrival: 'before', optOut: false, width: 64, height: 48, size: 'Custom 64×48' },
  { arrival: 'after', optOut: false, width: 64, height: 48, size: 'Custom 64×48' },
  { arrival: 'after', optOut: true, width: 64, height: 48, size: 'Custom 64×48' },
])('restores $size with a private snapshot $arrival mount (staging opt-out: $optOut) without writing settings', async ({ arrival, optOut, width, height, size }) => {
  vi.stubGlobal('localStorage', { getItem: () => optOut ? '0' : null });
  const saved: StagingConfig = {
    seed: '18446744073709551615', width, height, turn_limit: 333,
    seats: [
      { kind: 'host', name: 'Host', faction: 'Byzantium', team: 2, difficulty: 'Normal' },
      { kind: 'open', name: 'Guest default', faction: 'Random', team: 2, difficulty: 'Normal' },
      { kind: 'closed', name: '', faction: 'India', team: 4, difficulty: 'Easy' },
      { kind: 'ai', name: 'Rival', faction: 'India', team: 5, difficulty: 'Hard' },
    ],
  };
  const { seed, ...config } = saved;
  const publicRoom: StagingState = {
    room_id: 'existing-room', config,
    occupants: [
      { name: 'Host', faction: 'Byzantium', connected: false, ready: false, is_host: true },
      { name: 'Alice', faction: 'India', connected: true, ready: true, is_host: false },
      null, null,
    ],
  };
  const privateRoom: StagingState = {
    ...publicRoom, seed,
    occupants: publicRoom.occupants.map((o) => o ? { ...o, connected: true } : null),
  };
  const status: ServerStatus = { mode: 'server', has_game: false, turn: 0, seats: [], is_local: true, staging: publicRoom };
  const posted: StagingConfig[] = [];
  const settingsResponse = deferred();
  const fetch = vi.fn(async (url: string, init?: RequestInit) => {
    if (url === '/api/staging' && init?.method === 'POST') {
      posted.push(JSON.parse(init.body as string) as StagingConfig);
      await settingsResponse.promise;
    }
    return { ok: true, text: async () => JSON.stringify(status) };
  });
  vi.stubGlobal('fetch', fetch);
  useGame.setState(useGame.getInitialState(), true);
  rememberSeat({ seat: 0, name: 'Host', roomId: publicRoom.room_id, gameId: null, token: 'host-secret' });
  hostThisTab();
  await useGame.getState().boot();
  expect(useGame.getState()).toMatchObject({ screen: 'setup', setupMode: 'host' });
  sockets[0].open();
  expect(sockets[0].sent).toEqual([
    { type: 'staging_watch', host: true },
    { type: 'staging_take', seat: 0, name: 'Host', token: 'host-secret' },
  ]);
  sockets[0].push({ type: 'staging_taken', room_id: publicRoom.room_id, seat: 0, token: 'host-secret' });
  if (arrival === 'before') sockets[0].push({ type: 'staging', state: privateRoom });
  await act(async () => root.render(createElement(HostFlow)));
  expect(container.querySelector('h2')?.textContent).toBe('Host Game');
  expect(sockets).toHaveLength(1);

  if (arrival === 'after') {
    await act(async () => {
      sockets[0].push({ type: 'staging', state: publicRoom });
      await vi.advanceTimersByTimeAsync(1000);
    });
    expect(posted).toEqual([]); // The public snapshot must not unlock settings writes.
    expect(container.querySelector<HTMLButtonElement>('.lobby-go')?.disabled ?? true).toBe(true);
    await act(async () => sockets[0].push({ type: 'staging', state: privateRoom }));
  }

  expect(container.querySelector<HTMLInputElement>('[aria-label="Map seed"]')?.value).toBe(seed);
  expect(container.querySelector<HTMLInputElement>('#turn-limit')?.value).toBe('333');
  expect(container.querySelector('[aria-label="Map size"] [aria-checked="true"]')?.textContent).toContain(size);
  expect(container.querySelector<HTMLSelectElement>('[aria-label="Seat 4 AI level"]')?.value).toBe('Hard');
  expect(container.textContent).toContain('Alice');
  const start = container.querySelector<HTMLButtonElement>('.lobby-go')!;
  expect(start.textContent).toContain('Start game');
  expect(start.disabled).toBe(false);
  expect(posted).toEqual([]);
  await act(async () => vi.advanceTimersByTimeAsync(1000));
  expect(posted).toEqual([]);
  expect(useGame.getState().staging?.occupants[1]?.ready).toBe(true);
  expect(sockets[0].types()).toEqual(['staging_watch', 'staging_take']);

  // Selecting the current size is not an edit, including the custom option.
  await act(async () => container.querySelector<HTMLButtonElement>('[aria-label="Map size"] [aria-checked="true"]')!.click());
  await act(async () => vi.advanceTimersByTimeAsync(250));
  expect(posted).toEqual([]);

  // An unrelated host edit must retain the exact dimensions and configured seat defaults.
  const turnLimitOff = [...container.querySelectorAll<HTMLButtonElement>('[aria-label="Turn limit"] button')]
    .find((button) => button.textContent?.startsWith('Off'))!;
  await act(async () => turnLimitOff.click());
  expect(start.disabled).toBe(true);
  expect(posted).toEqual([]);
  await act(async () => vi.advanceTimersByTimeAsync(250));
  expect(posted).toEqual([{ ...saved, turn_limit: null }]);
  expect(start.disabled).toBe(true);
  await act(async () => settingsResponse.resolve());
  expect(start.disabled).toBe(true); // A response alone is not the current server snapshot.
  await act(async () => sockets[0].push({ type: 'staging', state: { ...privateRoom, config: { ...config, turn_limit: null } } }));
  expect(start.disabled).toBe(false);
  if (size.startsWith('Custom')) {
    const standard = [...container.querySelectorAll<HTMLButtonElement>('[aria-label="Map size"] button')]
      .find((b) => b.textContent?.startsWith('Standard'))!;
    await act(async () => standard.click());
    await act(async () => vi.advanceTimersByTimeAsync(250));
    // Standard suits 5-8 empires and the room has 3, so it is raised to Standard's default of 6.
    // New seats inherit the restored Hard level.
    const seats: StagingConfig['seats'] = [...saved.seats,
      { kind: 'ai', name: 'AI 5', faction: 'Random', team: 0, difficulty: 'Hard' },
      { kind: 'ai', name: 'AI 6', faction: 'Random', team: 1, difficulty: 'Hard' },
      { kind: 'ai', name: 'AI 7', faction: 'Random', team: 3, difficulty: 'Hard' }];
    expect(posted.at(-1)).toEqual({ ...saved, turn_limit: null, width: 48, height: 32, seats });
    await act(async () => sockets[0].push({ type: 'staging', state: {
      ...privateRoom, config: { ...config, turn_limit: null, width: 48, height: 32, seats },
    } }));
    expect(start.disabled).toBe(false);
  }
  await act(async () => start.click());
  expect(fetch).toHaveBeenCalledWith('/api/staging/start', expect.objectContaining({ method: 'POST' }));
});

it.each([
  { name: 'New host', faction: 'India' as const },
  { name: 'Host', faction: 'Byzantium' as const },
  { name: ' New host ', faction: 'Byzantium' as const },
])('reconciles a restored host reservation after the settings POST saved $name / $faction', async ({ name, faction }) => {
  const state: StagingState = {
    room_id: 'interrupted-edit', seed: '18446744073709551615',
    config: { width: 48, height: 32, turn_limit: null, seats: [
      { kind: 'host', name, faction, team: 0, difficulty: 'Normal' },
      { kind: 'open', name: 'Guest default', faction: 'Random', team: 1, difficulty: 'Normal' },
    ] },
    occupants: [
      { name: 'Host', faction: 'India', connected: true, ready: false, is_host: true },
      { name: 'Alice', faction: 'Byzantium', connected: true, ready: true, is_host: false },
    ],
  };
  const net = useGame.getState().net!;
  useGame.setState({ net: { ...net, status: { ...net.status, staging: state } } });
  rememberSeat({ seat: 0, name: 'Host', roomId: state.room_id, gameId: null, token: 'host-secret' });
  const fetch = vi.fn();
  vi.stubGlobal('fetch', fetch);
  await act(async () => root.render(createElement(HostFlow)));
  await act(async () => {
    sockets[0].open();
    sockets[0].push({ type: 'staging_taken', room_id: state.room_id, seat: 0, token: 'host-secret' });
    sockets[0].push({ type: 'staging', state });
    await vi.advanceTimersByTimeAsync(1000);
  });
  expect(fetch).not.toHaveBeenCalled();
  expect(sockets[0].sent.filter((msg) => msg.type === 'staging_update')).toEqual([
    { type: 'staging_update', name, faction },
  ]);
  const start = container.querySelector<HTMLButtonElement>('.lobby-go')!;
  expect(start.disabled).toBe(true);
  expect(container.textContent).toContain('Saving the latest settings…');
  await act(async () => sockets[0].push({ type: 'staging', state: {
    ...state, occupants: [{ ...state.occupants[0]!, name: name.trim(), faction }, state.occupants[1]],
  } }));
  expect(start.disabled).toBe(false);
  expect(container.textContent).not.toContain('Saving the latest settings…');
  expect(useGame.getState().staging?.occupants[1]?.ready).toBe(true);
  expect(container.querySelector<HTMLInputElement>('[aria-label="Map seed"]')?.value).toBe(state.seed);
  await act(async () => vi.advanceTimersByTimeAsync(1000));
  expect(fetch).not.toHaveBeenCalled();
  expect(sockets[0].sent.filter((msg) => msg.type === 'staging_update')).toHaveLength(1);
});

it.each(['status', 'staging_closed', 'no_room'] as const)('opens a fresh lobby when %s confirms the saved room no longer exists', async (missing) => {
  rememberSeat({ seat: 0, name: 'Host', roomId: 'deleted-room', gameId: null, token: 'expired-secret' });
  if (missing === 'staging_closed') {
    const net = useGame.getState().net!;
    useGame.setState({ net: { ...net, status: { ...net.status, staging: {
      room_id: 'deleted-room', config: { width: 64, height: 48, turn_limit: null, seats: [] }, occupants: [],
    } } } });
  } else if (missing === 'no_room') {
    const net = useGame.getState().net!;
    useGame.setState({ net: { ...net, status: { ...net.status, has_game: true } } });
  }
  const posted: StagingConfig[] = [];
  vi.stubGlobal('fetch', vi.fn(async (_url: string, init: RequestInit) => {
    if (init.method === 'POST') posted.push(JSON.parse(init.body as string));
    return { ok: true, text: async () => '{}' };
  }));
  await act(async () => root.render(createElement(HostFlow)));
  if (missing !== 'status') {
    expect(container.textContent).toContain('Restoring the game room…');
    await act(async () => {
      sockets[0].open();
      sockets[0].push(missing === 'staging_closed'
        ? { type: 'staging_closed' }
        : { type: 'error', code: 'no_room', message: 'No game room is open.' });
    });
  }
  expect(rememberedSeat()).toBeNull();
  expect(container.textContent).not.toContain('Restoring the game room…');
  expect(container.querySelector('[aria-label="Map seed"]')).not.toBeNull();
  await act(async () => {
    sockets.at(-1)!.open();
    await vi.advanceTimersByTimeAsync(250);
  });
  expect(posted).toHaveLength(1);
  expect(sockets.at(-1)!.sent).toContainEqual(expect.objectContaining({ type: 'staging_take', seat: 0 }));
  expect(useGame.getState().error).toBeNull();
});

it('recovers a missed game handoff before discarding a saved room credential', async () => {
  const net = useGame.getState().net!;
  useGame.setState({ net: { ...net, status: { ...net.status, has_game: true } } });
  rememberSeat({ seat: 0, name: 'Host', roomId: 'started-room', gameId: null, token: 'host-secret' });
  const fetch = vi.fn();
  vi.stubGlobal('fetch', fetch);
  await act(async () => root.render(createElement(HostFlow)));
  expect(container.textContent).toContain('Restoring the game room…');
  await act(async () => {
    sockets[0].open();
    sockets[0].push({ type: 'staging_started', game_id: 'game', seat: 0, token: 'host-secret' });
    await vi.advanceTimersByTimeAsync(1000);
  });
  expect(fetch).not.toHaveBeenCalled();
  expect(rememberedSeat()).toMatchObject({ gameId: 'game', token: 'host-secret' });
  sockets[1].open();
  expect(sockets[1].sent).toEqual([{ type: 'join', seat: 0, name: 'Host', token: 'host-secret' }]);
  useGame.getState().leaveOnline();
});

it('a seat\'s faction picker: clicking a faction in the gallery sets the seat and the dialog stays open until Close', async () => {
  useGame.setState({ setupMode: 'solo' });
  await act(async () => root.render(createElement(LobbyScreen)));
  const trigger = container.querySelector<HTMLButtonElement>('button[aria-label^="Seat 1 faction:"]')!;
  expect(trigger).not.toBeNull();
  const before = trigger.getAttribute('aria-label');
  expect(before).not.toContain('Mongols');
  await act(async () => trigger.click());
  const stage = trigger.parentElement!.querySelector<HTMLDialogElement>('dialog')!;
  expect(stage.open).toBe(true);
  const mongols = [...stage.querySelectorAll<HTMLElement>('[role="option"]')].find((o) => o.textContent?.includes('Mongols'))!;
  await act(async () => mongols.click());
  expect(stage.open).toBe(true);
  expect(container.querySelector('button[aria-label^="Seat 1 faction:"]')!.getAttribute('aria-label')).toContain('Mongols');
  await act(async () => stage.querySelector<HTMLElement>('.fs-close')!.click());
  expect(stage.open).toBe(false);
  expect(container.querySelector('button[aria-label^="Seat 1 faction:"]')!.getAttribute('aria-label')).toContain('Mongols');
});
