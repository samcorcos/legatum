import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { useGame } from './store';
import { FixtureClient } from './test/fixtureEngine';
import { coopVsAiSetup } from './gameSetup';
import { FakeSocket } from './test/fakeSocket';
import { feel, playTurnFeedback } from './feel';
import { disbandSelected, foundCitySelected, moveSelected, readySelected, toggleMoveSelected } from './commands';
import { setProduction, toggleBuy } from './orders';
import type { PlayerView } from './types';
import { hexAt } from './hex';
import { knownLegalSite } from './settlement';

vi.mock('./client', async (original) => ({
  ...await original<typeof import('./client')>(), createClient: async () => new FixtureClient(),
}));
vi.mock('./feel', async (original) => ({
  ...await original<typeof import('./feel')>(),
  playTurnFeedback: vi.fn(), cancelTurnFeedback: vi.fn(),
  feel: { select: vi.fn(), commit: vi.fn(), commitAttack: vi.fn(), purchase: vi.fn(), invalid: vi.fn(), ready: vi.fn() },
}));

beforeEach(() => {
  vi.useFakeTimers();
  vi.clearAllMocks();
  useGame.setState(useGame.getInitialState(), true);
});
afterEach(() => {
  useGame.getState().leaveOnline();
  useGame.setState(useGame.getInitialState(), true);
  vi.useRealTimers();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

async function localGame() {
  await useGame.getState().startGame(coopVsAiSetup('42', [{ name: 'Ada', faction: 'India' }]));
  return useGame.getState().client as FixtureClient;
}

function selectFutureSettler(client: FixtureClient) {
  const view = structuredClone(client.view(0));
  const settler = view.units.find((u) => u.owner === view.me && u.kind === 'Spearman')!;
  view.tiles.forEach((tile) => { tile.explored = true; tile.visible = true; tile.terrain = 'Grassland'; });
  const site = view.tiles.map((_, i) => hexAt(i, view.width)).find((hex) => knownLegalSite(view, hex))!;
  settler.kind = 'Settler'; settler.military = false; settler.pos = site;
  vi.spyOn(client, 'validate').mockReturnValue([]);
  useGame.setState({ views: [view] });
  useGame.getState().select({ kind: 'unit', id: settler.id });
  return { view, settler };
}

it('acknowledges found and Ready intent once, without a selection tick for automatic advance', async () => {
  selectFutureSettler(await localGame());
  vi.mocked(feel.select).mockClear();
  foundCitySelected();
  expect(feel.commit).toHaveBeenCalledTimes(1);
  await vi.advanceTimersByTimeAsync(120);
  expect(feel.select).not.toHaveBeenCalled();
  readySelected();
  expect(feel.ready).toHaveBeenCalledWith(true);
});

it('advances past a stacked escort after Found City, never selecting a unit on the new city\'s tile', async () => {
  const client = await localGame();
  const { view, settler } = selectFutureSettler(client);
  const escort = view.units.find((u) => u.owner === 0 && u.id !== settler.id)!;
  escort.pos = { ...settler.pos };
  useGame.setState({ views: [view] });
  useGame.getState().select({ kind: 'unit', id: settler.id });

  foundCitySelected();
  expect(useGame.getState().orders[0]).toContainEqual({ type: 'FoundCity', unit: settler.id });
  expect(useGame.getState().selection).toBeNull();
  await vi.advanceTimersByTimeAsync(200);
  const selected = useGame.getState().selection;
  expect(selected?.kind === 'unit' && [settler.id, escort.id].includes(selected.id)).toBe(false);
  expect(useGame.getState().orders[0]).toHaveLength(1);
});

it('pairs invalid orders with the visible validation reason instead of commit feedback', async () => {
  const client = await localGame();
  const unit = client.view(0).units.find((u) => u.owner === 0 && u.kind === 'Spearman')!;
  useGame.getState().select({ kind: 'unit', id: unit.id });
  vi.spyOn(client, 'validate').mockReturnValue([{ index: 0, message: 'That tile cannot be reached.' }]);
  moveSelected({ q: 1000, r: 1000 });
  expect(feel.invalid).toHaveBeenCalledTimes(1);
  expect(feel.commit).not.toHaveBeenCalled();
  expect(useGame.getState().error).toBe('That tile cannot be reached.');
});

it('refuses a locked unit\'s move or disband at the click, with the reason, and queues nothing', async () => {
  const client = await localGame();
  const view = structuredClone(client.view(0));
  const unit = view.units.find((u) => u.owner === 0 && u.military)!;
  unit.occupation_lock = 1;
  useGame.setState({ views: [view] });
  useGame.getState().select({ kind: 'unit', id: unit.id });
  moveSelected({ q: unit.pos.q + 1, r: unit.pos.r });
  expect(useGame.getState().orders[0] ?? []).toEqual([]);
  expect(feel.invalid).toHaveBeenCalledTimes(1);
  expect(useGame.getState().error).toMatch(/ is holding .+ down and cannot leave until Peace with .+\.$/);
  toggleMoveSelected();
  disbandSelected();
  expect(useGame.getState().orders[0] ?? []).toEqual([]);
  expect(useGame.getState().mode).toBe('normal');
  expect(feel.invalid).toHaveBeenCalledTimes(3);
});

it('uses purchase only after local Buy validation, and never when cancelling or rejecting a Buy', async () => {
  const client = await localGame();
  client.endTurn({});
  client.debugState().players[0].gold = 10000;
  const view = client.view(0);
  const city = view.cities.find((c) => c.owner === 0)!;
  useGame.setState({ views: [view] });
  useGame.getState().editOrders((o) => setProduction(o, city.id, { kind: 'Building', building: 'Granary' }, null));
  useGame.getState().editOrders((o) => toggleBuy(o, city.id, { kind: 'Unit', unit: 'Spearman' }), 'purchase');
  expect(feel.purchase).toHaveBeenCalledTimes(1);
  useGame.getState().editOrders((o) => toggleBuy(o, city.id, { kind: 'Unit', unit: 'Spearman' }));
  expect(feel.purchase).toHaveBeenCalledTimes(1);
  vi.spyOn(client, 'validate').mockReturnValue([{ index: 1, message: 'Not enough Gold' }]);
  useGame.getState().editOrders((o) => toggleBuy(o, city.id, { kind: 'Unit', unit: 'Spearman' }), 'purchase');
  expect(feel.purchase).toHaveBeenCalledTimes(1);
  expect(feel.invalid).toHaveBeenCalledTimes(1);
});

async function onlineGame(): Promise<{ socket: FakeSocket; view: PlayerView }> {
  const client = await localGame();
  const view = client.view(0);
  useGame.setState(useGame.getInitialState(), true);
  const sockets: FakeSocket[] = [];
  vi.stubGlobal('WebSocket', class extends FakeSocket {
    constructor(url: string) { super(url); sockets.push(this); }
  });
  const status = { mode: 'server' as const, has_game: true, turn: 1, seats: [], is_local: false };
  useGame.setState({ net: { status, mySeat: null, seats: [], conn: 'closed' } });
  useGame.getState().joinSeat(0, 'Ada');
  const socket = sockets[0];
  socket.open();
  socket.push({ type: 'welcome', seat: 0, game_id: 'g1', orders: [] });
  socket.push({ type: 'view', view });
  return { socket, view };
}

it('applies online feedback only with the new view, using the server-filtered event list', async () => {
  const { socket, view } = await onlineGame();
  socket.push({ type: 'turn', turn: 2, events: [{ type: 'TechResearched', player: 0, tech: 'BronzeWorking' }],
    hash: '2', finished: false, winner: null });
  expect(playTurnFeedback).not.toHaveBeenCalled();
  socket.push({ type: 'view', view: { ...view, turn: 2 } });
  expect(playTurnFeedback).toHaveBeenCalledWith(['turnResolved', 'techComplete'], expect.any(Function), { techComplete: 'sting.major' });
});

it('waits for the exact online Buy validation and ignores stale and cancelled requests', async () => {
  const { socket } = await onlineGame();
  useGame.getState().editOrders(() => [{ type: 'SleepUnit', unit: 1 }]);
  useGame.getState().editOrders((o) => [...o, { type: 'BuyItem', city: 1, item: { kind: 'Unit', unit: 'Spearman' } }], 'purchase');
  expect(feel.purchase).not.toHaveBeenCalled();
  socket.push({ type: 'validation', errors: [] }); // The older Hold request.
  expect(feel.purchase).not.toHaveBeenCalled();
  socket.push({ type: 'validation', errors: [] });
  expect(feel.purchase).toHaveBeenCalledTimes(1);
  useGame.getState().editOrders(() => [{ type: 'BuyItem', city: 2, item: { kind: 'Unit', unit: 'Spearman' } }], 'purchase');
  useGame.getState().editOrders(() => []);
  socket.push({ type: 'validation', errors: [] });
  expect(feel.purchase).toHaveBeenCalledTimes(1);
  socket.push({ type: 'validation', errors: [] });
  useGame.getState().editOrders(() => [{ type: 'BuyItem', city: 3, item: { kind: 'Unit', unit: 'Spearman' } }], 'purchase');
  socket.push({ type: 'validation', errors: [{ index: 0, message: 'Buy is blocked' }] });
  expect(feel.purchase).toHaveBeenCalledTimes(1);
  expect(feel.invalid).toHaveBeenCalledTimes(1);
  expect(useGame.getState().error).toBe('Buy is blocked');
});
