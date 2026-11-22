// Team map pings: hot seat keeps them in the store, online they travel over the host's message channel, and they never
// touch the game state. A solo game, or a host that does not relay pings, offers none.
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { FixtureClient } from './test/fixtureEngine';
import { coopVsAiSetup } from './gameSetup';
import { saveFile, useGame } from './store';
import { FakeSocket } from './test/fakeSocket';
import { pingAvailable, pingsFor, teammates, withPing, type TeamPing } from './teamPing';

vi.mock('./client', async (orig) => ({
  ...(await orig<typeof import('./client')>()),
  createClient: async () => new FixtureClient(),
}));
vi.mock('./platform/archive', () => ({
  recordGameHistory: vi.fn().mockResolvedValue(undefined),
  abandonGameHistory: vi.fn().mockResolvedValue(undefined),
}));

let run = 0;
beforeEach(() => {
  vi.useFakeTimers();
  // The minimum gap between pings is measured on the clock, which the last test left near now.
  vi.setSystemTime(Date.now() + 60_000 * ++run);
  useGame.setState(useGame.getInitialState(), true);
});
afterEach(() => {
  useGame.getState().stopResolving();
  useGame.getState().leaveOnline();
  vi.useRealTimers();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

/** Two humans on one team plus two AIs, in hot seat. */
async function hotSeat() {
  const setup = coopVsAiSetup('42', [{ name: 'Ada', faction: 'India' }, { name: 'Bo', faction: 'Byzantium' }]);
  setup.players = setup.players.slice(0, 4);
  await useGame.getState().startGame(setup);
}

const HEX = { q: 3, r: 4 };

it('a solo game offers no ping, and a ping goes nowhere', async () => {
  await useGame.getState().startGame(coopVsAiSetup('42', [{ name: 'Ada', faction: 'India' }]));
  expect(useGame.getState().canPing()).toBe(false);
  expect(useGame.getState().sendPing(HEX)).toBe(false);
  useGame.getState().setPingMode(true);
  expect(useGame.getState()).toMatchObject({ pingMode: false, pings: [] });
});

it('hot seat: a ping shows to the teammate when they take the device, and ends with the turn', async () => {
  await hotSeat();
  const s = () => useGame.getState();
  expect(s().canPing()).toBe(true);
  s().setPingMode(true);
  expect(s().pingMode).toBe(true);
  expect(s().sendPing(HEX)).toBe(true);
  expect(s().pingMode).toBe(false);
  const [mine, theirs] = s().views!;
  expect(pingsFor(mine, s().pings).map((p) => [p.from, p.hex])).toEqual([[mine.me, HEX]]);
  s().setSeat(1);
  expect(pingsFor(theirs, s().pings).map((p) => p.from)).toEqual([mine.me]);
  // Another empire's team never sees it.
  const other = { ...theirs, team: theirs.team + 1 };
  expect(pingsFor(other, s().pings)).toEqual([]);
  // The next turn's view carries no ping.
  expect(pingsFor({ ...theirs, turn: theirs.turn + 1 }, s().pings)).toEqual([]);
});

it('a ping is not repeated inside the minimum gap, and a sender has one ping at a time', async () => {
  await hotSeat();
  const s = () => useGame.getState();
  expect(s().sendPing(HEX)).toBe(true);
  expect(s().sendPing({ q: 5, r: 5 })).toBe(false);
  await vi.advanceTimersByTimeAsync(700);
  expect(s().sendPing({ q: 5, r: 5 })).toBe(true);
  expect(s().pings.map((p) => p.hex)).toEqual([{ q: 5, r: 5 }]);
});

it('the helpers: teammates, availability and one ping per sender', async () => {
  await hotSeat();
  const view = useGame.getState().views![0];
  expect(teammates(view).map((p) => p.name)).toEqual(['Bo']);
  expect(pingAvailable(view, true)).toBe(true);
  expect(pingAvailable(view, false)).toBe(false);
  expect(pingAvailable({ ...view, finished: true }, true)).toBe(false);
  expect(pingAvailable(null, true)).toBe(false);
  const a: TeamPing = { from: 0, team: 0, hex: HEX, turn: 1, seq: 1 };
  const b: TeamPing = { from: 1, team: 0, hex: HEX, turn: 1, seq: 2 };
  expect(withPing(withPing([a], b), { ...a, seq: 3 }).map((p) => p.seq)).toEqual([2, 3]);
  expect(withPing([a], { ...b, turn: 2 }).map((p) => p.seq)).toEqual([2]);
});

async function online(relays: boolean) {
  await hotSeat();
  const view = useGame.getState().views![0];
  const save = saveFile(useGame.getState().client!, useGame.getState().views!);
  await useGame.getState().loadSave(save);
  const socket = new FakeSocket('ws://host/ws');
  vi.stubGlobal('WebSocket', class { constructor() { return socket; } });
  const seats = view.players.filter((p) => p.is_human).map((p, seat) => ({
    seat, player: p.id, name: p.name, faction: p.faction, connected: true, ready: false, orders: 0,
  }));
  useGame.setState({ net: { status: { mode: 'server', has_game: true, turn: view.turn, seats, is_local: false }, mySeat: null, seats, conn: 'closed' } });
  useGame.getState().joinSeat(0, seats[0].name);
  socket.open();
  socket.push({ type: 'welcome', seat: 0, game_id: 'g1', orders: [], ...(relays ? { map_pings: true } : {}) });
  socket.push({ type: 'view', view });
  await Promise.resolve();
  return { socket, view };
}

it('online: a ping is sent to the host, shown to you at once, and a teammate\'s ping arrives over the host', async () => {
  const { socket, view } = await online(true);
  const s = () => useGame.getState();
  expect(s().canPing()).toBe(true);
  expect(s().sendPing(HEX)).toBe(true);
  expect(socket.sent.at(-1)).toEqual({ type: 'map_ping', hex: HEX });
  expect(pingsFor(view, s().pings).map((p) => p.from)).toEqual([view.me]);
  socket.push({ type: 'map_ping', from: 1, hex: { q: 6, r: 2 }, turn: view.turn });
  expect(pingsFor(view, s().pings).map((p) => [p.from, p.hex])).toEqual([[view.me, HEX], [1, { q: 6, r: 2 }]]);
  // A ping for another turn, or from a player on another team, is ignored.
  socket.push({ type: 'map_ping', from: 1, hex: { q: 1, r: 1 }, turn: view.turn + 5 });
  socket.push({ type: 'map_ping', from: 3, hex: { q: 1, r: 1 }, turn: view.turn });
  expect(s().pings).toHaveLength(2);
  // The host is another player's browser: a ping with no tile, or a tile that is not two whole numbers, is dropped.
  for (const hex of [null, {}, { q: 1 }, { q: 'a', r: 1 }, { q: 1.5, r: 2 }]) socket.push({ type: 'map_ping', from: 1, hex: hex as never, turn: view.turn });
  expect(pingsFor(view, s().pings).map((p) => [p.from, p.hex])).toEqual([[view.me, HEX], [1, { q: 6, r: 2 }]]);
});

it('online: a host from before pings gets no ping message and the control stays hidden', async () => {
  const { socket } = await online(false);
  expect(useGame.getState().canPing()).toBe(false);
  expect(useGame.getState().sendPing(HEX)).toBe(false);
  expect(socket.types()).not.toContain('map_ping');
});

it('an armed Ping never survives a seat switch (hot seat hand-over) or a turn change', async () => {
  await hotSeat();
  const s = () => useGame.getState();
  s().setPingMode(true);
  expect(s().pingMode).toBe(true);
  s().setSeat(1);
  expect(s().pingMode).toBe(false);
  s().setPingMode(true);
  expect(s().pingMode).toBe(true);
  s().setSeat(0);
  expect(s().pingMode).toBe(false);
});
