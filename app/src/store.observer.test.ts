import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { FixtureClient } from './test/fixtureEngine';
import { coopVsAiSetup } from './gameSetup';
import { saveFile, useGame } from './store';
import { FakeSocket } from './test/fakeSocket';

vi.mock('./client', async (orig) => ({
  ...(await orig<typeof import('./client')>()),
  createClient: async () => new FixtureClient(),
}));
vi.mock('./platform/archive', () => ({
  recordGameHistory: vi.fn().mockResolvedValue(undefined),
  abandonGameHistory: vi.fn().mockResolvedValue(undefined),
}));

beforeEach(() => {
  vi.useFakeTimers();
  useGame.setState(useGame.getInitialState(), true);
});
afterEach(() => {
  useGame.getState().stopResolving();
  useGame.getState().leaveOnline();
  vi.useRealTimers();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

async function game(humans = 1, dead = [0]) {
  const client = new FixtureClient();
  const setup = coopVsAiSetup('42', [
    { name: 'Ada', faction: 'India' }, { name: 'Bo', faction: 'Byzantium' }, { name: 'Cy', faction: 'Celts' },
  ].slice(0, humans) as Parameters<typeof coopVsAiSetup>[1]);
  setup.players = setup.players.slice(0, humans + 2);
  setup.players.forEach((p, i) => { p.team = i; });
  setup.turn_limit = 5;
  await client.newGame(setup);
  const state = client.debugState();
  state.players.forEach((p) => { if (dead.includes(p.id)) p.alive = false; });
  state.units = state.units.filter((u) => !dead.includes(u.owner));
  const save = saveFile(client, Array.from({ length: humans }, (_, i) => client.view(i)));
  if (humans === 1) await useGame.getState().loadSave(save);
  else useGame.setState({ client, humans: Array.from({ length: humans }, (_, i) => i), views: Array.from({ length: humans }, (_, i) => client.view(i)), orders: Array.from({ length: humans }, () => []), errors: Array.from({ length: humans }, () => []), ready: Array.from({ length: humans }, () => false), screen: 'game' });
  return { client: useGame.getState().client as FixtureClient, save };
}

it('an eliminated observer ends one turn with no orders and cannot edit orders', async () => {
  const { client } = await game();
  const resolve = vi.spyOn(client, 'endTurn');
  useGame.setState({ orders: [[{ type: 'SetResearch', tech: 'Philosophy' }]] });
  const edit = vi.fn(() => []);
  useGame.getState().editOrders(edit);
  expect(edit).not.toHaveBeenCalled();
  useGame.getState().toggleReady();
  await vi.advanceTimersByTimeAsync(30);
  expect(resolve).toHaveBeenCalledExactlyOnceWith({ 0: [] });
  expect(useGame.getState().views![0].turn).toBe(2);
  expect(useGame.getState().orders).toEqual([[]]);
});

it.each([1])('resolves to the actual game end after all %i humans are eliminated', async (humans) => {
  const { client } = await game(humans, Array.from({ length: humans }, (_, i) => i));
  const resolve = vi.spyOn(client, 'endTurn');
  useGame.getState().resolveToEnd();
  expect(useGame.getState().resolvingToEnd).toBe(true);
  await vi.advanceTimersByTimeAsync(3000);
  expect(useGame.getState().views![0].finished).toBe(true);
  expect(useGame.getState().resolvingToEnd).toBe(false);
  expect(resolve.mock.calls.length).toBeGreaterThan(1);
  for (const [orders] of resolve.mock.calls) expect(Object.values(orders).every((o) => o.length === 0)).toBe(true);
  const calls = resolve.mock.calls.length;
  await vi.advanceTimersByTimeAsync(2000);
  expect(resolve).toHaveBeenCalledTimes(calls);
});

it.each([false, true])('Stop cancels subsequent turns, including during resolution (%s)', async (duringTurn) => {
  const { client } = await game();
  const resolve = vi.spyOn(client, 'endTurn');
  useGame.getState().resolveToEnd();
  await vi.advanceTimersByTimeAsync(duringTurn ? 150 : 180);
  useGame.getState().stopResolving();
  await vi.advanceTimersByTimeAsync(3000);
  expect(resolve).toHaveBeenCalledOnce();
  expect(useGame.getState().resolvingToEnd).toBe(false);
  expect(useGame.getState().busy).toBeNull();
});

it('stops on a resolution error instead of retrying indefinitely', async () => {
  const { client } = await game();
  vi.spyOn(console, 'error').mockImplementation(() => {});
  const resolve = vi.spyOn(client, 'endTurn').mockImplementation(() => { throw new Error('Test failure'); });
  useGame.getState().resolveToEnd();
  await vi.advanceTimersByTimeAsync(3000);
  expect(resolve).toHaveBeenCalledOnce();
  expect(useGame.getState().resolvingToEnd).toBe(false);
  expect(useGame.getState().error).toContain('Test failure');
});

it.each(['title', 'load'] as const)('cancels queued observer work when leaving via %s', async (exit) => {
  const { client, save } = await game();
  const resolve = vi.spyOn(client, 'endTurn');
  useGame.getState().resolveToEnd();
  await vi.advanceTimersByTimeAsync(150); // A local resolution is queued but has not run.
  if (exit === 'title') useGame.getState().exitToTitle();
  else await useGame.getState().loadSave(save);
  await vi.advanceTimersByTimeAsync(1000);
  expect(resolve).not.toHaveBeenCalled();
  expect(useGame.getState().resolvingToEnd).toBe(false);
  expect(useGame.getState().busy).toBeNull();
});

it('records the resolved elimination turn for the Defeat dialog', async () => {
  const { client } = await game(1, []);
  // The old view is still alive; losing its capital and remaining units eliminates it at resolution.
  const state = client.debugState();
  const capital = state.cities.find((city) => city.owner === 0)!;
  state.cities = state.cities.filter((city) => city.id !== capital.id);
  state.units = state.units.filter((unit) => unit.owner !== 0);
  state.tileCity.forEach((id, i) => { if (id === capital.id) { state.tileCity[i] = -1; state.owner[i] = -1; } });
  useGame.getState().toggleReady();
  await vi.advanceTimersByTimeAsync(30);
  expect(useGame.getState().eliminationTurns).toEqual({ 0: 1 });
  expect(useGame.getState().views![0].turn).toBe(2);
  expect(useGame.getState().reportOpen).toBe(false);
});

it('rejects order edits before invoking the callback after the game finishes', async () => {
  await game(1, []);
  useGame.setState({ views: useGame.getState().views!.map((v) => ({ ...v, finished: true })), ready: [true] });
  const edit = vi.fn(() => []);
  const before = useGame.getState();
  useGame.getState().editOrders(edit);
  expect(edit).not.toHaveBeenCalled();
  expect(useGame.getState().orders).toBe(before.orders);
  expect(useGame.getState().ready).toBe(before.ready);
});

it('clears the previous state hash on local load and new game', async () => {
  const { save } = await game();
  useGame.setState({ lastHash: 'old-game-hash', defeatDismissed: [0], eliminationTurns: { 0: 23 } });
  await useGame.getState().loadSave(save);
  expect(useGame.getState()).toMatchObject({ lastHash: null, defeatDismissed: [], eliminationTurns: {} });
  useGame.setState({ lastHash: 'another-game-hash' });
  await useGame.getState().startGame(coopVsAiSetup('7', [{ name: 'Ada', faction: 'India' }]));
  expect(useGame.getState().lastHash).toBeNull();
});

async function joinObserver(humans = 2, dead = [0], localSeat = 0) {
  await game(humans, dead);
  const view = useGame.getState().views![localSeat];
  const socket = new FakeSocket('ws://host/ws');
  vi.stubGlobal('WebSocket', class { constructor() { return socket; } });
  const seats = view.players.filter((p) => p.is_human).map((p, seat) => ({
    seat, player: p.id, name: p.name, faction: p.faction, connected: true, ready: false, orders: 0,
  }));
  useGame.setState({ lastHash: 'previous-game-hash', net: {
    status: { mode: 'server', has_game: true, turn: view.turn, seats, is_local: false },
    mySeat: null, seats, conn: 'closed',
  } });
  useGame.getState().joinSeat(localSeat, seats[localSeat].name);
  socket.open();
  socket.push({ type: 'welcome', seat: localSeat, game_id: 'g1', orders: [] });
  socket.push({ type: 'view', view });
  await Promise.resolve();
  return { socket, view };
}

it('clears the previous hash on the first remote view and readies an observer for living teammates', async () => {
  const { socket, view } = await joinObserver();
  expect(useGame.getState().lastHash).toBeNull();
  expect(socket.sent.at(-1)).toEqual({ type: 'ready', game_id: 'g1', turn: view.turn, ready: true });
  expect(useGame.getState().ready).toEqual([true]);
  useGame.getState().resolveToEnd();
  expect(useGame.getState().resolvingToEnd).toBe(false);
  socket.push({ type: 'turn', turn: 1, events: [], hash: 'turn-hash', finished: false, winner: null });
  socket.push({ type: 'view', view: { ...view, turn: 2 } });
  await Promise.resolve();
  expect(socket.sent.at(-1)).toMatchObject({ type: 'ready', turn: 2, ready: true });
});

it('advances an online observer using empty orders, waits for each view and stops on disconnect', async () => {
  const { socket, view } = await joinObserver(1);
  useGame.getState().resolveToEnd();
  await vi.advanceTimersByTimeAsync(1000);
  expect(socket.sent.filter((m) => m.type === 'ready')).toHaveLength(1);
  expect(socket.sent.at(-1)).toMatchObject({ type: 'ready', turn: 1, ready: true });
  socket.push({ type: 'turn', turn: 1, events: [], hash: 'turn-hash', finished: false, winner: null });
  socket.push({ type: 'view', view: { ...view, turn: 2 } });
  await vi.advanceTimersByTimeAsync(150);
  expect(socket.sent.at(-1)).toMatchObject({ type: 'ready', turn: 2, ready: true });
  socket.drop();
  expect(useGame.getState().resolvingToEnd).toBe(false);
});

it('lets one online observer set the pace while the other dead seats automatically acknowledge turns', async () => {
  const { socket, view } = await joinObserver(2, [0, 1], 1);
  expect(useGame.getState().ready).toEqual([true]);
  expect(socket.sent.at(-1)).toMatchObject({ type: 'ready', turn: 1, ready: true });
  useGame.getState().resolveToEnd();
  expect(useGame.getState().resolvingToEnd).toBe(false);
  socket.push({ type: 'turn', turn: 1, events: [], hash: 'turn-hash', finished: false, winner: null });
  socket.push({ type: 'view', view: { ...view, turn: 2 } });
  await Promise.resolve();
  expect(socket.sent.at(-1)).toMatchObject({ type: 'ready', turn: 2, ready: true });
  // If the controlling observer leaves, the next connected seat gets explicit pacing control.
  socket.push({ type: 'seats', seats: useGame.getState().net!.seats.map((s) => ({ ...s, connected: s.seat === 1, ready: false })) });
  await Promise.resolve();
  expect(useGame.getState().ready).toEqual([false]);
  useGame.getState().resolveToEnd();
  expect(useGame.getState().resolvingToEnd).toBe(true);
  await vi.advanceTimersByTimeAsync(150);
  expect(socket.sent.at(-1)).toMatchObject({ type: 'ready', turn: 2, ready: true });
});
