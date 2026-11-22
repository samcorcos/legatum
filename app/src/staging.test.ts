import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { applyPreset, buildSetup, levelOf, newLobby, setDifficulty, setSlot } from './lobby';
import { StagingClient, StagingSettingsWriter, stagingEnabled, canTakeSeat, settingsAcknowledged, configFromLobby, everyoneReady, lobbyFromStaging, withOccupants, startStaging, type StagingState } from './staging';
import { rememberSeat, rememberedSeat } from './remote';
import { FakeSocket } from './test/fakeSocket';

function room(): StagingState {
  let l = newLobby('host', '77');
  l = setSlot({ ...l, mapSize: 'Small' }, 2, { kind: 'human' });
  const { seed, ...config } = configFromLobby(l);
  return { room_id: 'r1', config, seed, occupants: l.slots.map(() => null) };
}

describe('staging room model', () => {
  it('carries Training to the server and back with the developer switch on, and omits it for every other level', () => {
    vi.stubGlobal('localStorage', { getItem: (key: string) => key === 'legatum.dev.training' ? '1' : null, setItem: () => undefined, removeItem: () => undefined });
    const l = setDifficulty(setSlot(newLobby('host', '77'), 1, { kind: 'ai' }), 'Training');
    const config = configFromLobby(l);
    expect(config.seats[1]).toMatchObject({ kind: 'ai', difficulty: 'Hard', search: true });
    expect(config.seats[0]).not.toHaveProperty('search');
    expect(configFromLobby(setDifficulty(l, 'Hard')).seats[1]).not.toHaveProperty('search');
    const { seed, ...rest } = config;
    const back = lobbyFromStaging({ room_id: 'r', config: rest, seed, occupants: l.slots.map(() => null) });
    expect(levelOf(back)).toBe('Training');
    expect(levelOf(back.slots[1])).toBe('Training');
    vi.unstubAllGlobals();
  });

  it('a room that says Training reads as Hard for players, and is sent on as Hard', () => {
    vi.stubGlobal('localStorage', { getItem: () => null, setItem: () => undefined, removeItem: () => undefined });
    try {
      const l = setSlot(newLobby('host', '77'), 1, { kind: 'ai' });
      const { seed, ...config } = configFromLobby(l);
      const seats = config.seats.map((s) => s.kind === 'ai' ? { ...s, difficulty: 'Hard' as const, search: true } : s);
      const back = lobbyFromStaging({ room_id: 'r', config: { ...config, seats }, seed, occupants: l.slots.map(() => null) });
      expect(levelOf(back)).toBe('Hard');
      expect(back.slots.every((s) => !s.search)).toBe(true);
      expect(configFromLobby(back).seats.every((s) => !('search' in s))).toBe(true);
      expect(buildSetup(back).players.some((p) => p.search)).toBe(false);
    } finally { vi.unstubAllGlobals(); }
  });

  it('defaults on for servers advertising staging and honors explicit opt-outs', () => {
    vi.stubGlobal('location', { search: '' });
    vi.stubGlobal('localStorage', { getItem: () => null });
    try {
      expect(stagingEnabled({ staging: null })).toBe(true);
      expect(stagingEnabled({})).toBe(false);
      expect(stagingEnabled(null)).toBe(false);
      vi.stubGlobal('location', { search: '?staging=0' });
      expect(stagingEnabled({ staging: null })).toBe(false);
      vi.stubGlobal('localStorage', { getItem: () => '0' });
      vi.stubGlobal('location', { search: '' });
      expect(stagingEnabled({ staging: null })).toBe(false);
      vi.stubGlobal('location', { search: '?staging=1' });
      expect(stagingEnabled({ staging: null })).toBe(true);
    } finally { vi.unstubAllGlobals(); }
  });
  it('only offers empty open seats; disconnected reservations need a credential', () => {
    const st = room();
    st.occupants[1] = { name: 'Alice', faction: 'India', connected: false, ready: false, is_host: false };
    expect(canTakeSeat(st, 1)).toBe(false);
    st.occupants[1]!.connected = true;
    expect(canTakeSeat(st, 1)).toBe(false);
    expect(canTakeSeat(st, 0)).toBe(false);
    expect(canTakeSeat(st, 3)).toBe(false);
  });

  it('requires the exact edit acknowledgment and current server settings before Ready can enable Start', () => {
    const st = room();
    st.occupants = st.config.seats.map((s) => s.kind === 'open' || s.kind === 'host'
      ? { name: s.name, faction: s.faction, connected: true, ready: true, is_host: s.kind === 'host' } : null);
    const old = { ...st.config, seed: st.seed! };
    const edited = { ...old, seed: '6' };
    expect(settingsAcknowledged(edited, old, st)).toBe(false); // before the debounce
    expect(settingsAcknowledged(edited, edited, st)).toBe(false); // old broadcast
    const { seed, ...settings } = edited;
    st.config = settings;
    st.seed = seed;
    st.occupants.forEach((o) => { if (o) o.ready = false; });
    expect(settingsAcknowledged(edited, edited, st)).toBe(true);
    expect(everyoneReady(st)).toBe(false); // settings changes require new Ready
    st.occupants.forEach((o) => { if (o) o.ready = true; });
    expect(everyoneReady(st)).toBe(true);
    expect(settingsAcknowledged({ ...old }, old, st)).toBe(false); // reverting isn't acknowledgment
  });
  it('maps lobby seats to room seats and back', () => {
    const st = room();
    expect(st.config.seats.slice(0, 4).map((s) => s.kind)).toEqual(['host', 'open', 'open', 'ai']);
    expect([st.config.width, st.config.height, st.config.turn_limit]).toEqual([40, 26, null]);
    const back = lobbyFromStaging(st);
    expect(back.slots.slice(0, 4).map((s) => s.kind)).toEqual(['you', 'human', 'human', 'ai']);
    expect([back.mapSize, back.seed]).toEqual(['Small', '77']);
  });

  it('preserves custom dimensions through seat presets and game setup until the size changes', () => {
    const st = room();
    st.config = { ...st.config, width: 64, height: 48 };
    const lobby = applyPreset(lobbyFromStaging(st, false), 'teams');
    expect(configFromLobby(lobby)).toMatchObject({ width: 64, height: 48 });
    expect(buildSetup(lobby)).toMatchObject({ width: 64, height: 48 });
    expect(configFromLobby({ ...lobby, mapSize: 'Small' })).toMatchObject({ width: 40, height: 26 });
  });

  it('is ready only when every open seat is taken, connected and Ready', () => {
    const st = room();
    const occ = (ready: boolean, connected = true) => ({ name: 'X', faction: 'India' as const, connected, ready, is_host: false });
    st.occupants[0] = { ...occ(false), is_host: true };
    expect(everyoneReady(st)).toBe(false);
    st.occupants[1] = occ(true);
    st.occupants[2] = occ(false);
    expect(everyoneReady(st)).toBe(false);
    st.occupants[2] = occ(true, false);
    expect(everyoneReady(st)).toBe(false);
    st.occupants[2] = occ(true);
    expect(everyoneReady(st)).toBe(true);
    const shown = withOccupants(newLobby('host', '77'), st);
    expect(shown.slots[1]).toMatchObject({ name: 'X', faction: 'India' });
    expect(shown.slots[0].name).toBe('Player 1');
  });
});

describe('host settings writes', () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => { vi.useRealTimers(); vi.unstubAllGlobals(); });

  it('serializes rapid edits and acknowledges only the newest response', async () => {
    const acknowledge = vi.fn();
    let finish: (() => void) | undefined;
    const write = vi.fn(() => new Promise<void>((resolve) => { finish = resolve; }));
    const writer = new StagingSettingsWriter(acknowledge, vi.fn(), write);
    const original = { ...room().config, seed: room().seed! };
    writer.update(original);
    await vi.advanceTimersByTimeAsync(250);
    const firstFinish = finish!;
    writer.update({ ...original, seed: '6' });
    await vi.advanceTimersByTimeAsync(250);
    const latest = { ...original }; // revert while the first request is still in flight
    writer.update(latest);
    await vi.advanceTimersByTimeAsync(250);
    expect(write).toHaveBeenCalledTimes(1);
    firstFinish();
    await vi.advanceTimersByTimeAsync(0);
    expect(acknowledge).not.toHaveBeenCalled();
    expect(write).toHaveBeenCalledTimes(2);
    expect(write.mock.calls[1]).toEqual([latest]);
    finish!();
    await vi.advanceTimersByTimeAsync(0);
    expect(acknowledge).toHaveBeenCalledExactlyOnceWith(latest);
    await writer.stop();
  });

  it('does not acknowledge failures, and drains an in-flight write before closing', async () => {
    const acknowledge = vi.fn();
    const failed = vi.fn();
    let finish: (() => void) | undefined;
    const write = vi.fn().mockRejectedValueOnce(new Error('offline')).mockImplementation(() => new Promise<void>((resolve) => { finish = resolve; }));
    const writer = new StagingSettingsWriter(acknowledge, failed, write);
    writer.update({ ...room().config, seed: room().seed! });
    await vi.advanceTimersByTimeAsync(250);
    expect(failed).toHaveBeenCalledOnce();
    writer.update({ ...room().config, seed: room().seed! });
    await vi.advanceTimersByTimeAsync(250);
    const closed = vi.fn();
    const stopped = writer.stop().then(closed);
    expect(closed).not.toHaveBeenCalled();
    finish!();
    await stopped;
    expect(acknowledge).not.toHaveBeenCalled();
    expect(closed).toHaveBeenCalledOnce();
  });

  it('sends the required JSON content type on the bodyless Start POST', async () => {
    const fetch = vi.fn().mockResolvedValue({ ok: true, text: async () => '{}' });
    vi.stubGlobal('fetch', fetch);
    await startStaging();
    expect(fetch).toHaveBeenCalledWith('/api/staging/start', expect.objectContaining({ method: 'POST', headers: { 'content-type': 'application/json' } }));
  });
});

describe('StagingClient', () => {
  let sockets: FakeSocket[];
  const make = (events = {}) =>
    new StagingClient(events, 'ws://h/ws', (url) => {
      const s = new FakeSocket(url);
      sockets.push(s);
      return s;
    });
  beforeEach(() => {
    vi.useFakeTimers();
    sockets = [];
    const stored = new Map<string, string>();
    vi.stubGlobal('sessionStorage', {
      getItem: (key: string) => stored.get(key) ?? null,
      setItem: (key: string, value: string) => stored.set(key, value),
      removeItem: (key: string) => stored.delete(key),
    });
  });
  afterEach(() => { vi.useRealTimers(); vi.unstubAllGlobals(); });

  it('watches on open, takes and updates its seat, and re-takes it after a reconnect', () => {
    const states: StagingState[] = [];
    const c = make({ state: (s: StagingState) => states.push(s) });
    c.connect();
    sockets[0].open();
    expect(sockets[0].sent).toEqual([{ type: 'staging_watch' }]);
    c.take(2, 'Ben');
    sockets[0].push({ type: 'staging_taken', room_id: 'r1', seat: 2, token: 'secret' });
    c.update({ faction: 'Byzantium' });
    c.ready(true);
    expect(sockets[0].types()).toEqual(['staging_watch', 'staging_take', 'staging_update', 'staging_ready']);
    sockets[0].onmessage?.({ data: JSON.stringify({ type: 'staging', state: room() }) });
    expect(states).toHaveLength(1);
    sockets[0].drop();
    vi.advanceTimersByTime(1000);
    sockets[1].open();
    expect(sockets[1].sent).toEqual([{ type: 'staging_watch' }, { type: 'staging_take', seat: 2, name: 'Ben', token: 'secret' }]);
  });

  it('hands over on start and forgets a lost or refused seat', () => {
    const log: string[] = [];
    const c = make({
      started: (seat: number) => log.push(`started ${seat}`),
      seatLost: () => log.push('lost'),
      error: (m: string) => log.push(`error ${m}`),
    });
    c.connect();
    sockets[0].open();
    c.take(1, 'Ben');
    sockets[0].onmessage?.({ data: JSON.stringify({ type: 'error', code: 'seat_occupied', message: 'taken' }) });
    expect(c.seat).toBeNull();
    c.take(2, 'Ben');
    sockets[0].onmessage?.({ data: JSON.stringify({ type: 'staging_seat_lost', message: 'changed' }) });
    expect(c.seat).toBeNull();
    sockets[0].onmessage?.({ data: JSON.stringify({ type: 'staging_started', game_id: 'g', seat: 1, token: 'secret' }) });
    expect(log).toEqual(['error taken', 'lost', 'started 1']);
    vi.advanceTimersByTime(5000);
    expect(sockets).toHaveLength(1); // no reconnect after the hand-over
  });

  it('keeps host edits made while connecting and ignores retired sockets', () => {
    const state = vi.fn();
    const c = make({ state });
    c.connect();
    c.take(0, 'Host');
    c.update({ faction: 'Byzantium' });
    sockets[0].open();
    expect(sockets[0].sent).toEqual([
      { type: 'staging_watch' }, { type: 'staging_take', seat: 0, name: 'Host' },
      { type: 'staging_update', faction: 'Byzantium' },
    ]);
    c.disconnect();
    sockets[0].onmessage?.({ data: JSON.stringify({ type: 'staging', state: room() }) });
    expect(state).not.toHaveBeenCalled();
  });

  it('rejects leaving while disconnected and keeps the credential for reconnect', () => {
    const connection = vi.fn();
    const c = make({ connection });
    c.connect(); sockets[0].open(); c.take(1, 'Alice');
    sockets[0].push({ type: 'staging_taken', room_id: 'r1', seat: 1, token: 'secret' });
    sockets[0].drop();
    expect(connection).toHaveBeenLastCalledWith(false);
    expect(c.leave()).toBe(false);
    expect(c.seat).toBe(1);
    expect(rememberedSeat()).toMatchObject({ seat: 1, roomId: 'r1', token: 'secret' });
    expect(sockets[0].types()).not.toContain('staging_leave');
    vi.advanceTimersByTime(1000); sockets[1].open();
    expect(sockets[1].sent).toEqual([{ type: 'staging_watch' }, { type: 'staging_take', seat: 1, name: 'Alice', token: 'secret' }]);
    expect(connection).toHaveBeenLastCalledWith(true);
    c.disconnect();
  });

  it.each([false, true])('recovers a lost leave acknowledgment without forgetting ownership prematurely (released=%s)', (released) => {
    const c = make();
    c.connect(); sockets[0].open(); c.take(1, 'Alice');
    sockets[0].push({ type: 'staging_taken', room_id: 'r1', seat: 1, token: 'secret' });
    expect(c.leave()).toBe(true);
    sockets[0].drop();
    expect(c.seat).toBe(1);
    expect(rememberedSeat()?.token).toBe('secret');
    vi.advanceTimersByTime(1000); sockets[1].open();
    expect(sockets[1].sent.at(-1)).toEqual({ type: 'staging_take', seat: 1, name: 'Alice', token: 'secret' });
    sockets[0].push({ type: 'staging_left', room_id: 'r1' }); // Retired socket cannot clear ownership.
    expect(rememberedSeat()?.token).toBe('secret');
    if (released) {
      sockets[1].push({ type: 'error', code: 'seat_expired', message: 'Take an open seat to join again.' });
      expect(c.seat).toBeNull();
      expect(rememberedSeat()).toBeNull();
    } else {
      sockets[1].push({ type: 'staging_taken', room_id: 'r1', seat: 1, token: 'secret' });
      expect(c.leave()).toBe(true); // Retry is explicit.
      expect(rememberedSeat()?.token).toBe('secret');
      sockets[1].push({ type: 'staging_left', room_id: 'r1' });
      expect(c.seat).toBeNull();
      expect(rememberedSeat()).toBeNull();
    }
    c.disconnect();
  });
});


describe('staging credential persistence', () => {
  beforeEach(() => {
    const values = new Map<string, string>();
    vi.stubGlobal('sessionStorage', {
      getItem: (key: string) => values.get(key) ?? null,
      setItem: (key: string, value: string) => values.set(key, value),
      removeItem: (key: string) => values.delete(key),
    });
  });
  afterEach(() => vi.unstubAllGlobals());

  it('restores a seat after reload and persists the compacted handoff before notifying the store', () => {
    const first = new FakeSocket('ws://h/ws');
    const client = new StagingClient({}, first.url, () => first);
    client.connect(); first.open();
    client.take(3, 'Alice');
    expect(rememberedSeat()).toBeNull(); // A requested seat is not a credential grant.
    first.push({ type: 'staging_taken', room_id: 'r1', seat: 3, token: 'secret' });
    client.update({ name: 'Alice II' });
    client.disconnect();
    const second = new FakeSocket('ws://h/ws');
    const started = vi.fn(() => {
      expect(rememberedSeat()).toEqual({ seat: 1, name: 'Alice II', gameId: 'game', token: 'secret' });
    });
    const reload = new StagingClient({ started }, second.url, () => second);
    reload.connect(); second.open();
    expect(second.sent).toEqual([{ type: 'staging_watch' }, { type: 'staging_take', seat: 3, name: 'Alice II', token: 'secret' }]);
    second.push({ type: 'staging_started', game_id: 'game', seat: 1, token: 'secret' });
    expect(started).toHaveBeenCalledWith(1, 'game', 'secret');
  });

  it.each(['staging_seat_lost', 'staging_closed', 'leave'] as const)('forgets the credential on %s', (type) => {
    const socket = new FakeSocket('ws://h/ws');
    const client = new StagingClient({}, socket.url, () => socket);
    client.connect(); socket.open(); client.take(1, 'Alice');
    socket.push({ type: 'staging_taken', room_id: 'r1', seat: 1, token: 'secret' });
    if (type === 'leave') {
      expect(client.leave()).toBe(true);
      expect(rememberedSeat()?.token).toBe('secret');
      socket.push({ type: 'staging_left', room_id: 'r1' });
    }
    else socket.push({ type, message: 'Closed' });
    expect(rememberedSeat()).toBeNull();
    expect(client.seat).toBeNull();
    client.disconnect();
  });

  it.each(['staging_closed', 'staging_seat_lost', 'leave'] as const)('keeps a left game credential when an unrelated room reports %s', (type) => {
    const saved = { seat: 1, name: 'Alice', gameId: 'running-game', token: 'game-secret', autoJoin: false };
    rememberSeat(saved);
    const socket = new FakeSocket('ws://h/ws');
    const client = new StagingClient({}, socket.url, () => socket);
    client.connect(); socket.open();
    socket.push({ type: 'staging', state: room() });
    expect(socket.sent).toEqual([{ type: 'staging_watch' }]);
    if (type === 'leave') client.leave();
    else socket.push({ type, message: 'Closed' });
    expect(rememberedSeat()).toEqual(saved);
    expect(client.seat).toBeNull();
    client.disconnect();
  });

  it('shows Hidden when the public room has no seed', () => {
    const st = room();
    delete st.seed;
    expect(lobbyFromStaging(st).seed).toBe('Hidden');
  });

  it('waits for its own reliable leave acknowledgment even if snapshots skip the empty seat', () => {
    const socket = new FakeSocket('ws://h/ws');
    const left = vi.fn();
    const client = new StagingClient({ left }, socket.url, () => socket);
    client.connect(); socket.open(); client.take(1, 'Alice');
    socket.push({ type: 'staging_taken', room_id: 'r1', seat: 1, token: 'secret' });
    expect(client.leave()).toBe(true);
    expect(client.leave()).toBe(false); // Do not queue duplicate releases.
    socket.push({ type: 'staging_left', room_id: 'different-room' });
    const state = room();
    state.occupants[1] = { name: 'New player', faction: 'Byzantium', ready: false, connected: true, is_host: false };
    socket.push({ type: 'staging', state });
    expect(client.seat).toBe(1);
    expect(rememberedSeat()?.token).toBe('secret');
    socket.push({ type: 'staging_left', room_id: 'r1' });
    expect(rememberedSeat()).toBeNull();
    expect(client.seat).toBeNull();
    expect(left).toHaveBeenCalledOnce();
    client.disconnect();
  });

  it('forgets an expired reservation so a new claim omits the old credential', () => {
    const socket = new FakeSocket('ws://h/ws');
    const client = new StagingClient({}, socket.url, () => socket);
    client.connect(); socket.open(); client.take(1, 'Alice');
    socket.push({ type: 'staging_taken', room_id: 'r1', seat: 1, token: 'old-token' });
    socket.push({ type: 'error', code: 'seat_expired', message: 'Take an open seat to join again.' });
    expect(rememberedSeat()).toBeNull();
    expect(client.seat).toBeNull();
    client.take(1, 'Alice');
    expect(socket.sent.at(-1)).toEqual({ type: 'staging_take', seat: 1, name: 'Alice' });
    client.disconnect();
  });

  it.each(['seat_connected', 'not_releasable'])('keeps the host reservation when release is refused with %s', (code) => {
    const socket = new FakeSocket('ws://h/ws');
    const client = new StagingClient({}, socket.url, () => socket, true);
    client.connect(); socket.open(); client.take(0, 'Host');
    socket.push({ type: 'staging_taken', room_id: 'r1', seat: 0, token: 'host-token' });
    client.release(1);
    socket.push({ type: 'error', code, message: 'That seat cannot be opened.' });
    expect(client.seat).toBe(0);
    expect(rememberedSeat()).toMatchObject({ seat: 0, roomId: 'r1', token: 'host-token' });
    client.disconnect();
  });
});
