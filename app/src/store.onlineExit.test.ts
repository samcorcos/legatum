// Regression (.reviews/astra-master-4369f0b.md #2): after an online game, starting a local game must
// drop the online session; a late or reconnected socket's `view` must not replace the local game.
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { GameClient } from './client';
import { coopVsAiSetup } from './gameSetup';
import { FixtureClient } from './test/fixtureEngine';
import type { PlayerView } from './types';
import { useGame } from './store';

vi.mock('./client', async (orig) => ({
  ...(await orig<typeof import('./client')>()),
  createClient: async (): Promise<GameClient> => new FixtureClient(),
}));

class FakeSocket {
  static all: FakeSocket[] = [];
  readyState = 0;
  sent: { type: string }[] = [];
  onopen: ((ev: unknown) => void) | null = null;
  onmessage: ((ev: { data: unknown }) => void) | null = null;
  onclose: ((ev: unknown) => void) | null = null;
  onerror: ((ev: unknown) => void) | null = null;
  closed = false;
  constructor(public url: string) {
    FakeSocket.all.push(this);
  }
  send(data: string) {
    this.sent.push(JSON.parse(data));
  }
  close() {
    this.closed = true;
    this.readyState = 3;
    this.onclose?.({});
  }
  open() {
    this.readyState = 1;
    this.onopen?.({});
  }
  push(msg: unknown) {
    this.onmessage?.({ data: JSON.stringify(msg) });
  }
}

const onlineView = { turn: 40, me: 1, team: 0, players: [], units: [], cities: [], finished: true } as unknown as PlayerView;

describe('store: leaving an online game for a local one', () => {
  beforeEach(() => {
    FakeSocket.all = [];
    vi.stubGlobal('WebSocket', FakeSocket);
    const mem = new Map<string, string>();
    vi.stubGlobal('sessionStorage', {
      getItem: (k: string) => mem.get(k) ?? null,
      setItem: (k: string, v: string) => mem.set(k, v),
      removeItem: (k: string) => mem.delete(k),
    });
    useGame.setState({
      net: { status: { mode: 'server', has_game: true, turn: 40, seats: [], is_local: false }, mySeat: null, seats: [], conn: 'closed' },
    });
  });
  afterEach(() => vi.unstubAllGlobals());

  const joinSocket = () => {
    useGame.getState().joinSeat(1, 'Ben');
    const sock = FakeSocket.all[0];
    sock.open();
    sock.push({ type: 'welcome', seat: 1, game_id: 'g1', orders: [] });
    sock.push({ type: 'view', view: onlineView });
    expect(useGame.getState().client?.kind).toBe('remote');
    expect(useGame.getState().views![0].turn).toBe(40);
    return sock;
  };

  const expectLocalGame = () => {
    const s = useGame.getState();
    expect(s.client?.kind).toBe('wasm');
    expect(s.screen).toBe('game');
    expect(s.views![0].turn).toBe(1);
    expect(s.humans).toEqual([0]);
  };

  it('Back to title then Single Player: the old socket is closed and its messages ignored', async () => {
    const sock = joinSocket();
    useGame.getState().exitToTitle();
    expect(sock.closed).toBe(true);
    expect(useGame.getState().net?.mySeat).toBeNull();
    await useGame.getState().startGame(coopVsAiSetup('5', [{ name: 'Ann', faction: 'India' }]));
    expectLocalGame();
    // A late message from the old connection changes nothing.
    sock.push({ type: 'view', view: onlineView });
    sock.push({ type: 'seats', seats: [{ seat: 1, player: 1, name: 'Ben', faction: 'India', connected: true, ready: true, orders: 0 }] });
    expectLocalGame();
    expect(useGame.getState().ready).toEqual([false]);
    expect(FakeSocket.all).toHaveLength(1);
  });

  it('a local game started while still connected (title reached without leaving) drops the online session', async () => {
    const sock = joinSocket();
    useGame.getState().set({ screen: 'title' });
    useGame.getState().editOrders((o) => o);
    await useGame.getState().startGame(coopVsAiSetup('5', [{ name: 'Ann', faction: 'India' }]));
    expect(sock.closed).toBe(true);
    expectLocalGame();
    useGame.getState().editOrders(() => [{ type: 'SleepUnit', unit: 1 }]);
    // Even a reconnect-style welcome + view from that socket is ignored.
    sock.push({ type: 'welcome', seat: 1, game_id: 'g1', orders: [] });
    sock.push({ type: 'view', view: onlineView });
    expectLocalGame();
    expect(useGame.getState().orders[0]).toHaveLength(1);
    expect(FakeSocket.all).toHaveLength(1);
  });
});
