// Team map pings travel over the host's message channel and never
// touch the game state. A solo game, or a host that does not relay pings, offers none.
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { FixtureClient } from './test/fixtureEngine';
import { coopVsAiSetup } from './gameSetup';
import { useGame } from './store';
import { FakeSocket } from './test/fakeSocket';
import { feel } from './feel';
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

/** A two-human state for testing the online client. */
async function onlineGameState() {
  const setup = coopVsAiSetup('42', [{ name: 'Ada', faction: 'India' }, { name: 'Bo', faction: 'Byzantium' }]);
  setup.players = setup.players.slice(0, 4);
  const client = new FixtureClient();
  await client.newGame(setup);
  useGame.setState({ client, humans: [0, 1], views: [client.view(0), client.view(1)], orders: [[], []], errors: [[], []], ready: [false, false] });
}

const HEX = { q: 3, r: 4 };

it('a solo game offers no ping, and a ping goes nowhere', async () => {
  await useGame.getState().startGame(coopVsAiSetup('42', [{ name: 'Ada', faction: 'India' }]));
  expect(useGame.getState().canPing()).toBe(false);
  expect(useGame.getState().sendPing(HEX)).toBe(false);
  useGame.getState().setPingMode(true);
  expect(useGame.getState()).toMatchObject({ pingMode: false, pings: [] });
});

it('the helpers: teammates, availability and one ping per sender', async () => {
  await onlineGameState();
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
  await onlineGameState();
  const view = useGame.getState().views![0];
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
  const chime = vi.spyOn(feel, 'teamPing');
  const s = () => useGame.getState();
  expect(s().canPing()).toBe(true);
  expect(s().sendPing(HEX)).toBe(true);
  expect(socket.sent.at(-1)).toEqual({ type: 'map_ping', hex: HEX });
  expect(pingsFor(view, s().pings).map((p) => p.from)).toEqual([view.me]);
  expect(chime).not.toHaveBeenCalled(); // Your own ping is silent.
  socket.push({ type: 'map_ping', from: 1, hex: { q: 6, r: 2 }, turn: view.turn });
  expect(chime).toHaveBeenCalledOnce(); // A teammate's ping chimes.
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
