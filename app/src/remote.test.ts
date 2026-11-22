import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { MSG_STILL_TOO_FAST, MSG_TOO_FAST, MSG_TOO_MANY_WINDOWS, RemoteClient, fetchStatus, hostGame } from './remote';
import { FakeSocket } from './test/fakeSocket';
import type { PlayerView } from './types';

const view = (turn: number): PlayerView => ({ turn, me: 1, team: 0 }) as unknown as PlayerView;

function setup() {
  const sockets: FakeSocket[] = [];
  const client = new RemoteClient(1, 'Ben', 'ws://host/ws', (url) => {
    const s = new FakeSocket(url);
    sockets.push(s);
    return s;
  });
  client.token = 'seat-token';
  const log: string[] = [];
  client.on({
    view: (v, t) => log.push(`view ${v.turn}${t ? ` after turn ${t.turn} (${t.events.length} events)` : ''}`),
    seats: (s) => log.push(`seats ${s.length}`),
    validation: (e) => log.push(`validation ${e.length}`),
    forecast: () => log.push('forecast'),
    connection: (c) => log.push(`conn ${c}`),
    error: (m) => log.push(`error ${m}`),
  });
  return { client, sockets, log };
}

describe('RemoteClient', () => {
  it('normalizes retired orders and drops unknown kinds from a server snapshot', () => {
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => {});
    const { client, sockets } = setup();
    client.connect();
    sockets[0].open();
    sockets[0].push({ type: 'welcome', seat: 1, game_id: 'g1', orders: [
      { type: 'HealUnit', unit: 3 }, { type: 'FutureOrder', unit: 4 },
    ] as never });
    sockets[0].push({ type: 'view', view: view(4) });
    expect(client.pendingOrders()).toEqual([{ type: 'SleepUnit', unit: 3 }]);
    expect(warn).toHaveBeenCalledWith('Dropped unknown order kind: FutureOrder');
    warn.mockRestore();
  });

  it('binds orders and readiness to the last complete game/turn view', () => {
    const { client, sockets } = setup();
    client.connect();
    const socket = sockets[0];
    socket.open();
    socket.push({ type: 'welcome', seat: 1, game_id: 'g1', orders: [] });
    socket.push({ type: 'view', view: view(4) });
    const orders = [{ type: 'SleepUnit' as const, unit: 4 }];
    client.syncOrders(orders);
    client.setReady(true);
    expect(socket.sent.slice(-2)).toEqual([
      { type: 'orders', game_id: 'g1', turn: 4, orders },
      { type: 'ready', game_id: 'g1', turn: 4, ready: true },
    ]);
    socket.push({ type: 'turn', turn: 5, events: [], hash: '123', finished: false, winner: null });
    const sent = socket.sent.length;
    client.syncOrders([]);
    expect(client.setReady(true)).toBe(false);
    expect(socket.sent).toHaveLength(sent);
    socket.push({ type: 'view', view: view(5) });
    client.syncOrders(orders);
    client.setReady(false);
    expect(socket.sent.slice(-2)).toEqual([
      { type: 'orders', game_id: 'g1', turn: 5, orders },
      { type: 'ready', game_id: 'g1', turn: 5, ready: false },
    ]);
  });

  beforeEach(() => {
    vi.useFakeTimers();
    const store = new Map<string, string>();
    vi.stubGlobal('sessionStorage', {
      getItem: (k: string) => store.get(k) ?? null,
      setItem: (k: string, v: string) => store.set(k, v),
      removeItem: (k: string) => store.delete(k),
    });
  });
  afterEach(() => {
    vi.useRealTimers();
    vi.unstubAllGlobals();
  });

  it('joins its seat on open and caches the view', () => {
    const { client, sockets, log } = setup();
    client.connect();
    expect(sockets).toHaveLength(1);
    expect(sockets[0].url).toBe('ws://host/ws');
    sockets[0].open();
    expect(sockets[0].sent[0]).toEqual({ type: 'join', seat: 1, name: 'Ben', token: 'seat-token' });
    sockets[0].push({ type: 'welcome', seat: 1, game_id: 'g1', orders: [] });
    expect(client.state).toBe('connecting');
    expect(client.hasView()).toBe(false);
    sockets[0].push({ type: 'view', view: view(1) });
    expect(client.view().turn).toBe(1);
    expect(client.state).toBe('open');
    expect(log).toContain('conn open');
    expect(log).toContain('view 1');
    expect(JSON.parse(sessionStorage.getItem('legatum.lan.seat')!)).toEqual({ seat: 1, name: 'Ben', gameId: 'g1', token: 'seat-token' });
  });

  it('claims a fresh direct-game seat once, then uses the credential after reconnect and reload', () => {
    const { client, sockets } = setup();
    client.token = null;
    client.connect(); sockets[0].open();
    expect(sockets[0].sent).toEqual([{ type: 'game_take', seat: 1 }]);
    sockets[0].push({ type: 'seat_token', seat: 1, game_id: 'g1', token: 'issued-token' });
    expect(sockets[0].sent.at(-1)).toEqual({ type: 'join', seat: 1, name: 'Ben', token: 'issued-token' });
    sockets[0].push({ type: 'welcome', seat: 1, game_id: 'g1', orders: [] });
    sockets[0].drop(); vi.advanceTimersByTime(500); sockets[1].open();
    expect(sockets[1].sent).toEqual([{ type: 'join', seat: 1, name: 'Ben', token: 'issued-token' }]);
    client.disconnect(false);
    const socket = new FakeSocket('ws://host/ws');
    const reload = new RemoteClient(1, 'Ben', socket.url, () => socket);
    reload.connect(); socket.open();
    expect(socket.sent).toEqual([{ type: 'join', seat: 1, name: 'Ben', token: 'issued-token' }]);
    reload.disconnect();
  });

  it('does not use another seat or unhanded-off room credential for a game join', () => {
    sessionStorage.setItem('legatum.lan.seat', JSON.stringify({ seat: 3, name: 'Ben', token: 'secret', roomId: 'r1' }));
    expect(new RemoteClient(1, 'Ben').token).toBeNull();
    expect(new RemoteClient(3, 'Ben').token).toBeNull();
  });

  it('only reassigns a rejected credential on an explicit reclaim, never during reconnect', () => {
    const { client, sockets } = setup();
    client.connect(); sockets[0].open();
    sockets[0].push({ type: 'error', code: 'invalid_seat_credential', message: 'Invalid seat credential.' });
    sockets[0].drop(); vi.advanceTimersByTime(500); sockets[1].open();
    expect(sockets[1].sent).toEqual([{ type: 'join', seat: 1, name: 'Ben', token: 'seat-token' }]);
    client.reclaimSeat(); sockets[2].open();
    expect(sockets[2].sent).toEqual([{ type: 'game_take', seat: 1 }]);
    // If the new credential never arrives, reconnect must not rotate it again.
    sockets[2].drop(); vi.advanceTimersByTime(500); sockets[3].open();
    expect(sockets[3].sent).toEqual([{ type: 'join', seat: 1, name: 'Ben', token: 'seat-token' }]);
    client.reclaimSeat(); sockets[4].open();
    sockets[4].push({ type: 'seat_token', seat: 1, game_id: 'restarted', token: 'fresh-token' });
    expect(sockets[4].sent.at(-1)).toEqual({ type: 'join', seat: 1, name: 'Ben', token: 'fresh-token' });
    expect(JSON.parse(sessionStorage.getItem('legatum.lan.seat')!).token).toBe('fresh-token');
    sockets[4].drop(); vi.advanceTimersByTime(500); sockets[5].open();
    expect(sockets[5].sent).toEqual([{ type: 'join', seat: 1, name: 'Ben', token: 'fresh-token' }]);
    client.disconnect();
  });

  it('does not repeat an initial claim automatically when its credential acknowledgment is lost', () => {
    const { client, sockets } = setup();
    client.token = null;
    client.connect(); sockets[0].open();
    expect(sockets[0].types()).toEqual(['game_take']);
    sockets[0].drop(); vi.advanceTimersByTime(500); sockets[1].open();
    expect(sockets[1].sent).toEqual([]);
    client.rejoin(); sockets[2].open();
    expect(sockets[2].types()).toEqual(['game_take']);
    client.disconnect();
  });

  it('sends orders (deduplicated) and reports validation asynchronously', () => {
    const { client, sockets, log } = setup();
    client.connect();
    sockets[0].open();
    sockets[0].push({ type: 'welcome', seat: 1, game_id: 'g1', orders: [] });
    sockets[0].push({ type: 'view', view: view(1) });
    const orders = [{ type: 'SleepUnit' as const, unit: 4 }];
    client.syncOrders(orders);
    expect(client.validate(1, orders)).toEqual([]);
    client.syncOrders(orders); // unchanged: not re-sent
    expect(sockets[0].types().filter((t) => t === 'orders')).toHaveLength(1);
    sockets[0].push({ type: 'validation', errors: [{ index: 0, message: 'unit 4 does not exist' }] });
    expect(log).toContain('validation 1');
    expect(client.validate(1, orders)).toEqual([{ index: 0, message: 'unit 4 does not exist' }]);
    client.setReady(true);
    expect(sockets[0].sent.at(-1)).toEqual({ type: 'ready', game_id: 'g1', turn: 1, ready: true });
  });

  it('validation previews cannot overwrite orders, clear errors or send an online message', () => {
    const { client, sockets } = setup();
    client.connect(); sockets[0].open();
    sockets[0].push({ type: 'welcome', seat: 1, game_id: 'g1', orders: [] });
    sockets[0].push({ type: 'view', view: view(1) });
    const orders = [{ type: 'SleepUnit' as const, unit: 4 }];
    client.syncOrders(orders);
    const errors = [{ index: 0, message: 'unit 4 does not exist' }];
    sockets[0].push({ type: 'validation', errors });
    const sent = sockets[0].sent.length;
    expect(client.validate(1, [{ type: 'BuyItem', city: 7, item: { kind: 'Unit', unit: 'Soldier' } }])).toEqual([]);
    expect(client.pendingOrders()).toEqual(orders);
    expect(client.validate(1, orders)).toEqual(errors);
    expect(sockets[0].sent).toHaveLength(sent);
  });

  it('requests forecasts once and serves them from the cache', () => {
    const { client, sockets, log } = setup();
    client.connect();
    sockets[0].open();
    sockets[0].push({ type: 'welcome', seat: 1, game_id: 'g1', orders: [] });
    sockets[0].push({ type: 'view', view: view(3) });
    const target = { q: 2, r: 5 };
    expect(client.forecast(1, 9, target, 'Melee')).toBeNull();
    expect(client.forecast(1, 9, target, 'Melee')).toBeNull();
    const reqs = sockets[0].sent.filter((m) => m.type === 'forecast');
    expect(reqs).toEqual([{ type: 'forecast', req_id: 1, unit: 9, target, mode: 'Melee' }]);
    const fc = {
      attacker_strength: { total: 20, parts: [] },
      defender_strength: { total: 10, parts: [] },
      win_chance: 60,
      loss_chance: 0,
      expected_damage_to_attacker: 20,
      expected_damage_to_defender: 45,
      attacker_hp: 100,
      defender_hp: 60,
      ranged: false,
    } as const;
    sockets[0].push({ type: 'forecast', req_id: 1, forecast: { ...fc, attacker_strength: { total: 20, parts: [] }, defender_strength: { total: 10, parts: [] } } });
    expect(log).toContain('forecast');
    expect(client.forecast(1, 9, target, 'Melee')?.win_chance).toBe(60);
    // Changing attack mode at the same hex must request and cache a different forecast.
    expect(client.forecast(1, 9, target, 'Ranged')).toBeNull();
    expect(client.forecast(1, 9, target, 'Ranged')).toBeNull();
    expect(sockets[0].sent.filter((m) => m.type === 'forecast')).toEqual([
      { type: 'forecast', req_id: 1, unit: 9, target, mode: 'Melee' },
      { type: 'forecast', req_id: 2, unit: 9, target, mode: 'Ranged' },
    ]);
    sockets[0].push({ type: 'forecast', req_id: 2, forecast: { ...fc, attacker_strength: { total: 15, parts: [] }, defender_strength: { total: 10, parts: [] }, expected_damage_to_attacker: 0, ranged: true } });
    expect(client.forecast(1, 9, target, 'Ranged')?.expected_damage_to_attacker).toBe(0);
    expect(client.forecast(1, 9, target, 'Melee')?.expected_damage_to_attacker).toBe(20);
    // A new view clears the cache.
    sockets[0].push({ type: 'view', view: view(4) });
    expect(client.forecast(1, 9, target, 'Melee')).toBeNull();
    expect(client.forecast(1, 9, target, 'Ranged')).toBeNull();
  });

  it.each([1, 4])('labels resolved turn %i separately from the next planning view and resets orders', (resolvedTurn) => {
    const { client, sockets, log } = setup();
    client.connect();
    sockets[0].open();
    sockets[0].push({ type: 'welcome', seat: 1, game_id: 'g1', orders: [] });
    sockets[0].push({ type: 'view', view: view(resolvedTurn) });
    client.syncOrders([{ type: 'SleepUnit', unit: 4 }]);
    const delivered = log.length;
    sockets[0].push({
      type: 'turn',
      turn: resolvedTurn + 1,
      events: [{ type: 'TechResearched', player: 1, tech: 'Currency' }],
      hash: '123',
      finished: false,
      winner: null,
    });
    expect(client.lastTurn?.turn).toBe(resolvedTurn);
    expect(log).toHaveLength(delivered);
    sockets[0].push({ type: 'view', view: view(resolvedTurn + 1) });
    expect(client.view().turn).toBe(resolvedTurn + 1);
    expect(log.at(-1)).toBe(`view ${resolvedTurn + 1} after turn ${resolvedTurn} (1 events)`);
    expect(client.lastTurn?.hash).toBe('123');
    // Orders were reset for the new turn: re-sending the same orders goes out again.
    client.syncOrders([{ type: 'SleepUnit', unit: 4 }]);
    expect(sockets[0].types().filter((t) => t === 'orders')).toHaveLength(2);
  });

  it('tracks seats and teammate status', () => {
    const { client, sockets } = setup();
    client.connect();
    sockets[0].open();
    sockets[0].push({
      type: 'seats',
      seats: [
        { seat: 0, player: 0, name: 'Ana', faction: 'India', connected: true, ready: true, orders: 3 },
        { seat: 1, player: 1, name: 'Ben', faction: 'Byzantium', connected: true, ready: false, orders: 0 },
      ],
    });
    expect(client.teammateStatus()?.name).toBe('Ana');
    expect(client.myStatus()?.ready).toBe(false);
  });

  it('reconnects with exponential backoff, re-joins the same seat and restores orders', () => {
    const { client, sockets, log } = setup();
    client.connect();
    sockets[0].open();
    sockets[0].push({ type: 'welcome', seat: 1, game_id: 'g1', orders: [] });
    sockets[0].push({ type: 'view', view: view(5) });
    client.syncOrders([{ type: 'SleepUnit', unit: 4 }]);

    sockets[0].drop();
    expect(client.state).toBe('reconnecting');
    expect(client.backoffMs(1)).toBe(500);
    expect(client.backoffMs(2)).toBe(1000);
    expect(client.backoffMs(10)).toBe(8000);
    vi.advanceTimersByTime(499);
    expect(sockets).toHaveLength(1);
    vi.advanceTimersByTime(1);
    expect(sockets).toHaveLength(2);
    // Second attempt fails too: next wait is 1 s.
    sockets[1].drop();
    vi.advanceTimersByTime(999);
    expect(sockets).toHaveLength(2);
    vi.advanceTimersByTime(1);
    expect(sockets).toHaveLength(3);
    sockets[2].open();
    expect(sockets[2].sent[0]).toEqual({ type: 'join', seat: 1, name: 'Ben', token: 'seat-token' });
    sockets[2].push({ type: 'welcome', seat: 1, game_id: 'g1', orders: [{ type: 'SleepUnit', unit: 4 }] });
    expect(sockets[2].types()).toEqual(['join']);
    // The authoritative orders survive the view and are not replayed on reconnect.
    sockets[2].push({ type: 'view', view: view(5) });
    client.syncOrders([{ type: 'SleepUnit', unit: 4 }]);
    expect(sockets[2].types().filter((t) => t === 'orders')).toHaveLength(0);
    expect(log.filter((l) => l === 'conn open')).toHaveLength(2);
  });

  it.each([
    { gameId: 'g1', turn: 6, reason: 'the turn advanced' },
    { gameId: 'g2', turn: 5, reason: 'the game changed at the same turn' },
  ])('discards old orders when $reason while disconnected', ({ gameId, turn }) => {
    const { client, sockets } = setup();
    client.connect();
    sockets[0].open();
    sockets[0].push({ type: 'welcome', seat: 1, game_id: 'g1', orders: [] });
    sockets[0].push({ type: 'view', view: view(5) });
    const orders = [{ type: 'RangedAttack' as const, unit: 4, target: { q: 2, r: 3 } }];
    client.syncOrders(orders);
    sockets[0].push({ type: 'validation', errors: [{ index: 0, message: 'old validation' }] });
    sockets[0].drop();
    vi.advanceTimersByTime(500);
    sockets[1].open();
    client.syncOrders(orders); // Editing or Ready during the handshake must not replay old orders.
    client.setReady(true);
    sockets[1].push({ type: 'welcome', seat: 1, game_id: gameId, orders: [] });
    client.syncOrders(orders);
    client.setReady(true);
    expect(sockets[1].types()).toEqual(['join']);

    sockets[1].push({ type: 'view', view: view(turn) });
    client.setReady(true);
    expect(sockets[1].sent.slice(1)).toEqual([
      { type: 'ready', game_id: gameId, turn, ready: true },
    ]);
    expect(client.validate(1, [])).toEqual([]);
  });

  it.each(['disconnected', 'before welcome', 'before view'] as const)(
    'synchronizes a cancellation made %s before Ready on a same-turn reconnect',
    (when) => {
      const { client, sockets } = setup();
      client.connect();
      sockets[0].open();
      sockets[0].push({ type: 'welcome', seat: 1, game_id: 'g1', orders: [] });
      sockets[0].push({ type: 'view', view: view(5) });
      client.syncOrders([{ type: 'RangedAttack', unit: 4, target: { q: 2, r: 3 } }]);
      sockets[0].drop();
      if (when === 'disconnected') client.syncOrders([]);
      vi.advanceTimersByTime(500);
      sockets[1].open();
      if (when === 'before welcome') client.syncOrders([]);
      sockets[1].push({ type: 'welcome', seat: 1, game_id: 'g1', orders: [{ type: 'RangedAttack', unit: 4, target: { q: 2, r: 3 } }] });
      if (when === 'before view') client.syncOrders([]);
      expect(sockets[1].types()).toEqual(['join']);
      sockets[1].push({ type: 'view', view: view(5) });
      expect(sockets[1].sent.at(-1)).toEqual({ type: 'orders', game_id: 'g1', turn: 5, orders: [] });
      client.setReady(true);
      expect(sockets[1].sent.slice(1)).toEqual([
        { type: 'orders', game_id: 'g1', turn: 5, orders: [] },
        { type: 'ready', game_id: 'g1', turn: 5, ready: true },
      ]);
    },
  );

  it('retries orders after a send throws instead of deduplicating an unsent list', () => {
    const { client, sockets } = setup();
    client.connect();
    sockets[0].open();
    sockets[0].push({ type: 'welcome', seat: 1, game_id: 'g1', orders: [] });
    sockets[0].push({ type: 'view', view: view(5) });
    client.syncOrders([{ type: 'SleepUnit', unit: 4 }]);
    const send = vi.spyOn(sockets[0], 'send').mockImplementationOnce(() => {
      throw new Error('send failed');
    });
    expect(() => client.syncOrders([])).toThrow('send failed');
    client.syncOrders([]);
    expect(send).toHaveBeenCalledTimes(2);
    expect(sockets[0].sent.at(-1)).toEqual({ type: 'orders', game_id: 'g1', turn: 5, orders: [] });
  });

  it('does not deduplicate orders changed while the socket cannot send', () => {
    const { client, sockets } = setup();
    client.connect();
    sockets[0].open();
    sockets[0].push({ type: 'welcome', seat: 1, game_id: 'g1', orders: [] });
    sockets[0].push({ type: 'view', view: view(5) });
    client.syncOrders([{ type: 'SleepUnit', unit: 4 }]);
    sockets[0].readyState = 3;
    client.syncOrders([]);
    expect(sockets[0].sent.at(-1)).toEqual({ type: 'orders', game_id: 'g1', turn: 5, orders: [{ type: 'SleepUnit', unit: 4 }] });
    sockets[0].readyState = 1;
    client.syncOrders([]);
    expect(sockets[0].sent.at(-1)).toEqual({ type: 'orders', game_id: 'g1', turn: 5, orders: [] });
  });

  it('stops reconnecting on explicit leave but keeps the credential for manual rejoin', () => {
    const { client, sockets } = setup();
    client.connect();
    sockets[0].open();
    sockets[0].push({ type: 'welcome', seat: 1, game_id: 'g1', orders: [] });
    client.disconnect();
    vi.advanceTimersByTime(20000);
    expect(sockets).toHaveLength(1);
    expect(client.state).toBe('closed');
    expect(JSON.parse(sessionStorage.getItem('legatum.lan.seat')!)).toMatchObject({ token: 'seat-token', autoJoin: false });
    expect(new RemoteClient(1, 'Ben').token).toBe('seat-token');
  });

  it('waits for the join snapshot before Ready and sends an explicit cancellation of retained orders', () => {
    const { client, sockets } = setup();
    client.connect();
    sockets[0].open();
    client.setReady(true);
    expect(sockets[0].types()).toEqual(['join']);
    sockets[0].push({ type: 'welcome', seat: 1, game_id: 'g1', orders: [{ type: 'DisbandUnit', unit: 4 }] });
    client.setReady(true);
    expect(sockets[0].types()).toEqual(['join']);
    sockets[0].push({ type: 'view', view: view(5) });
    client.syncOrders([]);
    client.setReady(true);
    expect(sockets[0].sent.slice(1)).toEqual([{ type: 'orders', game_id: 'g1', turn: 5, orders: [] }, { type: 'ready', game_id: 'g1', turn: 5, ready: true }]);
  });

  it.each([1000, 1006])('stops after a takeover instead of stealing the seat back (close %i)', (code) => {
    // Two windows on the same seat, sharing one fake server that closes the older socket.
    const a = setup();
    const b = setup();
    a.client.connect();
    a.sockets[0].open();
    a.sockets[0].push({ type: 'welcome', seat: 1, game_id: 'g1', orders: [] });
    a.sockets[0].push({ type: 'view', view: view(1) });
    b.client.connect();
    b.sockets[0].open();
    // Server: B's join takes the seat; A gets an error and its socket is closed.
    b.sockets[0].push({ type: 'welcome', seat: 1, game_id: 'g1', orders: [] });
    b.sockets[0].push({ type: 'view', view: view(1) });
    a.sockets[0].push({ type: 'error', code: 'seat_taken', message: 'Another browser took over this seat.' });
    a.sockets[0].push({ type: 'pong' });
    a.sockets[0].drop(code);
    expect(a.client.state).toBe('taken');
    expect(a.log).toContain('conn taken');
    vi.advanceTimersByTime(60000);
    // A never reconnects on its own, so B keeps the seat.
    expect(a.sockets).toHaveLength(1);
    expect(b.client.state).toBe('open');
    // The player chooses to rejoin: A opens a new socket and joins again.
    a.client.rejoin();
    expect(a.sockets).toHaveLength(2);
    a.sockets[1].open();
    expect(a.sockets[1].sent[0]).toEqual({ type: 'join', seat: 1, name: 'Ben', token: 'seat-token' });
    a.sockets[1].push({ type: 'welcome', seat: 1, game_id: 'g1', orders: [] });
    a.sockets[1].drop();
    expect(a.client.state).toBe('reconnecting');
    vi.advanceTimersByTime(500);
    expect(a.sockets).toHaveLength(3);
  });

  it.each([
    { reconnect: false, welcomed: false },
    { reconnect: false, welcomed: true },
    { reconnect: true, welcomed: false },
    { reconnect: true, welcomed: true },
  ])('honors takeover before a complete view (reconnect=$reconnect, welcomed=$welcomed)', ({ reconnect, welcomed }) => {
    const { client, sockets } = setup();
    client.connect(); sockets[0].open();
    if (reconnect) {
      sockets[0].push({ type: 'welcome', seat: 1, game_id: 'g1', orders: [] });
      sockets[0].push({ type: 'view', view: view(1) });
      sockets[0].drop(); vi.advanceTimersByTime(500); sockets[1].open();
    }
    const socket = sockets.at(-1)!;
    if (welcomed) socket.push({ type: 'welcome', seat: 1, game_id: 'g1', orders: [] });
    expect(client.state).toBe(reconnect ? 'reconnecting' : 'connecting');
    socket.push({ type: 'error', code: 'seat_taken', message: 'Another window took this seat.' });
    socket.drop();
    expect(client.state).toBe('taken');
    const count = sockets.length;
    vi.advanceTimersByTime(60000);
    expect(sockets).toHaveLength(count);
    client.rejoin(); sockets.at(-1)!.open();
    expect(sockets.at(-1)!.sent).toEqual([{ type: 'join', seat: 1, name: 'Ben', token: 'seat-token' }]);
    client.disconnect();
  });

  it.each([
    { message: 'bad order' },
    { code: 'bad_request', message: 'bad order' },
    { message: 'Another browser took over this seat.' },
  ])('reconnects after a non-takeover error and close 1006: %j', (error) => {
    const { client, sockets, log } = setup();
    client.connect();
    sockets[0].open();
    sockets[0].push({ type: 'welcome', seat: 1, game_id: 'g1', orders: [] });
    sockets[0].push({ type: 'error', ...error });
    sockets[0].drop(1006);
    expect(log).toContain(`error ${error.message}`);
    expect(log).not.toContain('conn taken');
    expect(client.state).toBe('reconnecting');
    vi.advanceTimersByTime(500);
    expect(sockets).toHaveLength(2);
    sockets[1].open();
    expect(sockets[1].sent[0]).toEqual({ type: 'join', seat: 1, name: 'Ben', token: 'seat-token' });
    sockets[1].push({ type: 'welcome', seat: 1, game_id: 'g1', orders: [] });
    sockets[1].push({ type: 'view', view: view(1) });
    expect(client.state).toBe('open');
  });

  it('an error on a joined socket that stays open is not a takeover', () => {
    const { client, sockets } = setup();
    client.connect();
    sockets[0].open();
    sockets[0].push({ type: 'welcome', seat: 1, game_id: 'g1', orders: [] });
    sockets[0].push({ type: 'view', view: view(1) });
    sockets[0].push({ type: 'error', message: 'bad order' });
    sockets[0].push({ type: 'pong' });
    expect(client.state).toBe('open');
    // A later network drop is a normal disconnect: reconnect.
    sockets[0].drop();
    expect(client.state).toBe('reconnecting');
    vi.advanceTimersByTime(500);
    expect(sockets).toHaveLength(2);
  });

  it('surfaces server errors and ignores garbage frames', () => {
    const { client, sockets, log } = setup();
    client.connect();
    sockets[0].open();
    sockets[0].onmessage?.({ data: 'not json' });
    sockets[0].push({ type: 'error', message: 'seat taken' });
    expect(log).toContain('error seat taken');
    expect(() => client.endTurn()).toThrow();
  });

  it('pings every 20 s while joined', () => {
    const { client, sockets } = setup();
    client.connect();
    sockets[0].open();
    sockets[0].push({ type: 'welcome', seat: 1, game_id: 'g1', orders: [] });
    vi.advanceTimersByTime(40000);
    expect(sockets[0].types().filter((t) => t === 'ping')).toHaveLength(2);
  });
});

describe('online game status', () => {
  const res = (ok: boolean, body: string) => async () => ({ ok, status: ok ? 200 : 404, text: async () => body });

  it('reads the game from /api/status', async () => {
    const status = { mode: 'server', has_game: false, turn: 0, seats: [], is_local: true };
    expect(await fetchStatus('https://r/rooms/a', res(true, JSON.stringify(status)))).toEqual(status);
  });

  it('is null on 404, HTML or network errors', async () => {
    expect(await fetchStatus('https://r/rooms/a', res(false, 'nope'))).toBeNull();
    expect(await fetchStatus('https://r/rooms/a', res(true, '<!doctype html><html>'))).toBeNull();
    expect(
      await fetchStatus('https://r/rooms/a', async () => {
        throw new Error('ECONNREFUSED');
      }),
    ).toBeNull();
  });

  it('hosts a game with POST /api/game', async () => {
    const calls: { url: string; method?: string; body?: string }[] = [];
    const status = { mode: 'server', has_game: true, turn: 1, seats: [], is_local: true };
    const out = await hostGame({ seed: '7', width: 48, height: 32, players: [], turn_limit: null }, '', async (url, init) => {
      calls.push({ url, method: init?.method, body: init?.body });
      return { ok: true, status: 200, text: async () => JSON.stringify(status) };
    });
    expect(calls[0].url).toBe('/api/game');
    expect(calls[0].method).toBe('POST');
    expect(JSON.parse(calls[0].body!).seed).toBe('7');
    expect(out.has_game).toBe(true);
    await expect(hostGame({ seed: '7', width: 48, height: 32, players: [], turn_limit: null }, '', res(false, 'loopback only'))).rejects.toThrow(
      'loopback only',
    );
  });

  describe('relay close codes', () => {
    beforeEach(() => vi.useFakeTimers());
    afterEach(() => vi.useRealTimers());
    const joined = () => {
      const t = setup();
      t.client.connect();
      t.sockets[0].open();
      t.sockets[0].push({ type: 'welcome', seat: 1, game_id: 'g1', orders: [] });
      t.sockets[0].push({ type: 'view', view: view(1) });
      return t;
    };

    it('stops and says why on 1013 for too many connections from this address', () => {
      const { client, sockets, log } = joined();
      sockets[0].drop(1013, 'too many connections from one address');
      expect(log).toContain(`error ${MSG_TOO_MANY_WINDOWS}`);
      expect(client.state).toBe('closed');
      vi.advanceTimersByTime(60_000);
      expect(sockets).toHaveLength(1);
    });

    it('keeps reconnecting on 1013 for a host that is away', () => {
      const { client, sockets } = joined();
      sockets[0].drop(1013, 'host not connected');
      expect(client.state).toBe('reconnecting');
      vi.advanceTimersByTime(500);
      expect(sockets).toHaveLength(2);
    });

    it('pauses 30 seconds after 4029, retries once, then stops', () => {
      const { client, sockets, log } = joined();
      sockets[0].drop(4029, 'too many messages');
      expect(log).toContain(`error ${MSG_TOO_FAST}`);
      vi.advanceTimersByTime(29_999);
      expect(sockets).toHaveLength(1);
      vi.advanceTimersByTime(1);
      expect(sockets).toHaveLength(2);
      sockets[1].open();
      sockets[1].drop(4029, 'too many messages');
      expect(log).toContain(`error ${MSG_STILL_TOO_FAST}`);
      expect(client.state).toBe('closed');
      vi.advanceTimersByTime(120_000);
      expect(sockets).toHaveLength(2);
    });

    it('allows a new pause after the game has been flowing again', () => {
      const { sockets } = joined();
      sockets[0].drop(4029, 'too many messages');
      vi.advanceTimersByTime(30_000);
      sockets[1].open();
      sockets[1].push({ type: 'welcome', seat: 1, game_id: 'g1', orders: [] });
      sockets[1].push({ type: 'view', view: view(2) });
      sockets[1].drop(4029, 'too many messages');
      vi.advanceTimersByTime(30_000);
      expect(sockets).toHaveLength(3);
    });

    it('reconnects as usual on other codes', () => {
      const { sockets } = joined();
      sockets[0].drop(1006);
      vi.advanceTimersByTime(500);
      expect(sockets).toHaveLength(2);
    });
  });
});
