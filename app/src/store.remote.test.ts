import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { useGame } from './store';
import { FixtureClient } from './test/fixtureEngine';
import { FakeSocket } from './test/fakeSocket';
import type { PlayerView } from './types';
import { clearUnitOrder, setUnitOrder } from './orders';

class Socket extends FakeSocket {
  static instances: Socket[] = [];
  constructor(url: string) {
    super(url);
    Socket.instances.push(this);
  }
}

let view: PlayerView;
beforeEach(async () => {
  vi.useFakeTimers();
    const storage = new Map([['legatum.lan.seat', JSON.stringify({ seat: 0, name: 'Ana', gameId: 'g1', token: 'seat-token' })]]);
    vi.stubGlobal('sessionStorage', {
      getItem: (key: string) => storage.get(key) ?? null,
      setItem: (key: string, value: string) => storage.set(key, value),
      removeItem: (key: string) => storage.delete(key),
    });
  Socket.instances = [];
  vi.stubGlobal('WebSocket', Socket);
  const mock = new FixtureClient();
  await mock.newGame(await mock.standardSetup('7', 'India', 'Byzantium', 'Ana', 'Ben'));
  view = mock.view(0);
  useGame.setState({
    screen: 'join', client: null, views: null, orders: [[], []], ready: [false, false],
    net: {
      status: { mode: 'server', has_game: true, turn: 1, seats: [], is_local: true },
      mySeat: null, seats: [], conn: 'closed',
    },
  });
});
afterEach(() => {
  useGame.getState().leaveOnline();
  vi.useRealTimers();
  vi.unstubAllGlobals();
});

it('shows retained DisbandUnit after a reload before Ready, and lets the user cancel it', () => {
  useGame.getState().joinSeat(0, 'Ana');
  const socket = Socket.instances[0];
  socket.open();
  const orders = [{ type: 'DisbandUnit' as const, unit: view.units.find((u) => u.owner === 0)!.id }];
  socket.push({ type: 'welcome', seat: 0, game_id: 'g1', orders });
  useGame.getState().toggleReady();
  expect(socket.sent).toEqual([{ type: 'join', seat: 0, name: 'Ana', token: 'seat-token' }]);
  socket.push({ type: 'view', view });
  expect(useGame.getState().orders[0]).toEqual(orders);
  expect(useGame.getState().net?.conn).toBe('open');
  useGame.getState().editOrders(() => []);
  useGame.getState().toggleReady();
  expect(socket.sent.slice(1)).toEqual([{ type: 'orders', game_id: 'g1', turn: 1, orders: [] }, { type: 'ready', game_id: 'g1', turn: 1, ready: true }]);
});

it('replaces the UI orders on reconnect, including a same-turn seat takeover', () => {
  useGame.getState().joinSeat(0, 'Ana');
  const socket = Socket.instances[0];
  socket.open();
  socket.push({ type: 'welcome', seat: 0, game_id: 'g1', orders: [] });
  socket.push({ type: 'view', view });
  useGame.getState().editOrders(() => [{ type: 'SleepUnit', unit: view.units[0].id }]);
  socket.close();
  useGame.getState().toggleReady();
  expect(useGame.getState().ready[0]).toBe(false);
  vi.advanceTimersByTime(500);
  const next = Socket.instances[1];
  next.open();
  const orders = [{ type: 'DisbandUnit' as const, unit: view.units[0].id }];
  next.push({ type: 'welcome', seat: 0, game_id: 'g1', orders });
  useGame.getState().toggleReady();
  expect(next.sent).toEqual([{ type: 'join', seat: 0, name: 'Ana', token: 'seat-token' }]);
  next.push({ type: 'view', view });
  expect(useGame.getState().orders[0]).toEqual(orders);
  useGame.getState().toggleReady();
  expect(next.sent.slice(1)).toEqual([{ type: 'ready', game_id: 'g1', turn: 1, ready: true }]);
});

it('keeps an online seat Ready after clearing an idle unit or repeating an unchanged order', () => {
  useGame.getState().joinSeat(0, 'Ana');
  const socket = Socket.instances[0];
  socket.open();
  socket.push({ type: 'welcome', seat: 0, game_id: 'g1', orders: [] });
  socket.push({ type: 'view', view });
  const unit = view.units.find((u) => u.owner === 0)!.id;
  useGame.getState().toggleReady();
  const sent = [...socket.sent];
  useGame.getState().editOrders((orders) => clearUnitOrder(orders, unit));
  expect(useGame.getState().ready).toEqual([true]);
  expect(socket.sent).toEqual(sent);

  // A real edit still un-readies and sends orders; repeating it does neither.
  const hold = { type: 'SleepUnit', unit } as const;
  useGame.getState().editOrders((orders) => setUnitOrder(orders, hold));
  expect(useGame.getState().ready).toEqual([false]);
  expect(socket.sent.at(-1)).toMatchObject({ type: 'orders', orders: [hold] });
  useGame.getState().toggleReady();
  const readySent = [...socket.sent];
  useGame.getState().editOrders((orders) => setUnitOrder(orders, { ...hold }));
  expect(useGame.getState().ready).toEqual([true]);
  expect(socket.sent).toEqual(readySent);
  useGame.getState().toggleReady();
  expect(socket.sent.at(-1)).toMatchObject({ type: 'ready', ready: false });
});
