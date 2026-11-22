import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { useGame } from './store';
import { FakeSocket } from './test/fakeSocket';
import type { PlayerView } from './types';

const view = (turn: number): PlayerView => ({
  turn, me: 1, team: 0, units: [], cities: [], players: [], finished: false,
}) as unknown as PlayerView;

describe('online order reconciliation in the store', () => {
  let sockets: FakeSocket[];

  beforeEach(() => {
    vi.useFakeTimers();
    const storage = new Map([['legatum.lan.seat', JSON.stringify({ seat: 1, name: 'Ben', gameId: 'g1', token: 'seat-token' })]]);
    vi.stubGlobal('sessionStorage', {
      getItem: (key: string) => storage.get(key) ?? null,
      setItem: (key: string, value: string) => storage.set(key, value),
      removeItem: (key: string) => storage.delete(key),
    });
    sockets = [];
    vi.stubGlobal('WebSocket', class extends FakeSocket {
      constructor(url: string) {
        super(url);
        sockets.push(this);
      }
    });
    useGame.setState(useGame.getInitialState(), true);
    const status = { mode: 'server' as const, has_game: true, turn: 5, seats: [], is_local: false };
    useGame.setState({ net: { status, mySeat: null, seats: [], conn: 'closed' } });
    useGame.getState().joinSeat(1, 'Ben');
    sockets[0].open();
    sockets[0].push({ type: 'welcome', seat: 1, game_id: 'g1', orders: [] });
    sockets[0].push({ type: 'view', view: view(5) });
    useGame.getState().editOrders(() => [{ type: 'RangedAttack', unit: 4, target: { q: 2, r: 3 } }]);
  });

  afterEach(() => {
    useGame.getState().leaveOnline();
    useGame.setState(useGame.getInitialState(), true);
    vi.useRealTimers();
    vi.unstubAllGlobals();
  });

  const reconnect = (gameId: string, turn: number) => {
    sockets[0].drop();
    vi.advanceTimersByTime(500);
    sockets[1].open();
    sockets[1].push({ type: 'welcome', seat: 1, game_id: gameId, orders: gameId === 'g1' && turn === 5 ? useGame.getState().orders[0] : [] });
    expect(sockets[1].types()).toEqual(['join']);
    sockets[1].push({ type: 'view', view: view(turn) });
  };

  it.each([
    { gameId: 'g1', turn: 6, reason: 'a missed turn' },
    { gameId: 'g2', turn: 5, reason: 'a replacement game at the same turn' },
  ])('clears UI orders and validation after $reason, even without a turn message', ({ gameId, turn }) => {
    sockets[0].push({ type: 'validation', errors: [{ index: 0, message: 'old validation' }] });
    useGame.getState().toggleReady();
    useGame.getState().select({ kind: 'unit', id: 4 });
    useGame.getState().setMode('ranged');

    reconnect(gameId, turn);
    const state = useGame.getState();
    expect(state.views?.[0].turn).toBe(turn);
    expect(state.orders).toEqual([[]]);
    expect(state.errors).toEqual([[]]);
    expect(state.ready).toEqual([false]);
    expect(state.selection).toBeNull();
    expect(state.mode).toBe('normal');
    state.toggleReady();
    expect(sockets[1].sent.slice(1)).toEqual([
      { type: 'ready', game_id: gameId, turn, ready: true },
    ]);
  });

  it('restores retained UI orders when the game and turn are unchanged', () => {
    const orders = useGame.getState().orders;
    reconnect('g1', 5);
    expect(useGame.getState().orders).toEqual(orders);
    expect(sockets[1].types()).toEqual(['join']);
  });

  it('sends an empty list before Ready when all UI orders were cancelled offline', () => {
    sockets[0].drop();
    useGame.getState().editOrders(() => []);
    vi.advanceTimersByTime(500);
    sockets[1].open();
    sockets[1].push({ type: 'welcome', seat: 1, game_id: 'g1', orders: [{ type: 'RangedAttack', unit: 4, target: { q: 2, r: 3 } }] });
    useGame.getState().toggleReady();
    expect(useGame.getState().ready[0]).toBe(false);
    expect(sockets[1].types()).toEqual(['join']);
    sockets[1].push({ type: 'view', view: view(5) });
    expect(sockets[1].sent.at(-1)).toEqual({ type: 'orders', game_id: 'g1', turn: 5, orders: [] });
    useGame.getState().toggleReady();
    expect(sockets[1].sent.slice(1)).toEqual([
      { type: 'orders', game_id: 'g1', turn: 5, orders: [] },
      { type: 'ready', game_id: 'g1', turn: 5, ready: true },
    ]);
    expect(useGame.getState().orders).toEqual([[]]);
  });
});
