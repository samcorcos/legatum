import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { useGame } from './store';
import { FixtureClient } from './test/fixtureEngine';
import { explorationFixture } from './test/explorationFixture';
import { explorationCoreSave } from './test/explorationFixture';
import { unitsNeedingOrders, setUnitOrder, describeOrder } from './orders';
import { addAutomatedOrders } from './automation';
import { explore_step } from './wasm-pkg/legatum.js';
import { FakeSocket } from './test/fakeSocket';
import { toggleAutoExploreSelected } from './commands';
import { feel } from './feel';
import { coreFixture } from './test/coreFixture';
import { wallsFixture } from './test/wallsFixture';
import { pendingDecisions } from './turnFlow';
import type { Order, PlayerView } from './types';

vi.mock('./client', async (original) => ({ ...await original<typeof import('./client')>(), createClient: async () => new FixtureClient() }));
vi.mock('./feel', async (original) => ({ ...await original<typeof import('./feel')>(), feel: { select: vi.fn(), ready: vi.fn(), invalid: vi.fn() }, playTurnFeedback: vi.fn(), cancelTurnFeedback: vi.fn() }));

beforeEach(() => { useGame.setState(useGame.getInitialState(), true); vi.clearAllMocks(); });
afterEach(() => { useGame.getState().leaveOnline(); vi.useRealTimers(); vi.unstubAllGlobals(); });
async function start() {
  const s = explorationFixture();
  await useGame.getState().loadSave({ format: 'legatum-save', version: 1, client: 'wasm', savedAt: '', turn: 1, names: ['Explorer'], game: explorationCoreSave(s) });
  return useGame.getState().client as FixtureClient;
}

it('toggles with one selection haptic, queues a route at End turn, and keeps manual orders authoritative', async () => {
  const client = await start();
  useGame.getState().select({ kind: 'unit', id: 0 });
  toggleAutoExploreSelected();
  expect(feel.select).toHaveBeenCalledTimes(1);
  const before = useGame.getState().views![0];
  expect(unitsNeedingOrders(before, useGame.getState().orders[0]).some((u) => u.id === 0)).toBe(false);
  const end = vi.spyOn(client, 'endTurn');
  await useGame.getState().toggleReady();
  expect(describeOrder(useGame.getState().orders[0][0], before)).toContain('automated');
  await vi.waitFor(() => expect(useGame.getState().views![0].turn).toBe(2));
  expect(end.mock.calls[0][0][0][0]).toMatchObject({ type: 'SetAutomation', automation: 'Explore', path: expect.any(Array) });
  const after = useGame.getState().views![0];
  const scout = after.units.find((u) => u.id === 0)!;
  expect(scout.pos).not.toEqual(before.units.find((u) => u.id === 0)!.pos);
  expect(scout.automation).toBe('Explore');
  expect(unitsNeedingOrders(after, []).some((u) => u.id === 0)).toBe(false);
  expect(feel.select).toHaveBeenCalledTimes(1);
  const manual: Order[] = [{ type: 'SleepUnit', unit: 0 }];
  expect(await addAutomatedOrders(after, manual)).toEqual(manual);
  useGame.getState().editOrders((o) => setUnitOrder(o, manual[0]));
  await useGame.getState().toggleReady();
  await vi.waitFor(() => expect(useGame.getState().views![0].turn).toBe(3));
  expect(useGame.getState().views![0].units.find((u) => u.id === 0)?.automation).toBeUndefined();
});

it('stops on a finite frontier, reports the ruin and stop, and returns the scout to needs orders', async () => {
  await start(); toggleAutoExploreSelected();
  let stopped = false, found = false;
  for (let i = 0; i < 12; i++) {
    const turn = useGame.getState().views![0].turn;
    await useGame.getState().toggleReady();
    await vi.waitFor(() => expect(useGame.getState().views![0].turn).toBe(turn + 1));
    found ||= useGame.getState().report!.events[0].some((e) => e.type === 'RuinExplored');
    if (useGame.getState().report!.events[0].some((e) => e.type === 'AutomationStopped')) { stopped = true; break; }
  }
  expect(found).toBe(true); expect(stopped).toBe(true);
  const s = useGame.getState();
  expect(s.report!.lines[0].some((l) => l.text.includes('stopped auto-explore'))).toBe(true);
  expect(s.reportOpen).toBe(true);
  expect(unitsNeedingOrders(s.views![0], []).some((u) => u.id === 0)).toBe(true);
});

it('Online submits client-planned orders before Ready using only the joined seat view', async () => {
  const client = await start();
  const view = client.view(0); view.units.find((u) => u.id === 0)!.automation = 'Explore';
  const foreign = { ...view.units.find((u) => u.id === 0)!, id: 99, owner: 1, pos: { q: 15, r: 4 } };
  view.units.push(foreign);
  const sockets: FakeSocket[] = [];
  vi.stubGlobal('WebSocket', class extends FakeSocket { constructor(url: string) { super(url); sockets.push(this); } });
  useGame.setState({ client: null, views: null, net: { status: { mode: 'server', has_game: true, turn: 1, seats: [], is_local: false }, seats: [], mySeat: null, conn: 'connecting' } });
  useGame.getState().joinSeat(0, 'Explorer');
  const socket = sockets[0]; socket.open();
  socket.push({ type: 'welcome', seat: 0, game_id: 'auto', orders: [] });
  socket.push({ type: 'view', view });
  await useGame.getState().toggleReady();
  const orders = socket.sent.filter((m) => m.type === 'orders').at(-1)!.orders as Order[];
  expect(orders).toHaveLength(1);
  expect(orders[0]).toMatchObject({ type: 'SetAutomation', unit: 0, automation: 'Explore', path: expect.any(Array) });
  expect(socket.sent.at(-1)).toMatchObject({ type: 'ready', ready: true });
});

it('excludes persisted automation from the idle cycle and rejects planning for teammates', async () => {
  const c = await start(); const view = c.view(0);
  view.units[0].automation = 'Explore';
  expect(unitsNeedingOrders(view, []).map((u) => u.id)).not.toContain(0);
  const teammate = { ...view, me: 1 } as PlayerView;
  expect(JSON.parse(explore_step(JSON.stringify(teammate), 0))).toBeNull();
});

it('stops Ready for an automated city with no build and accepts a manual order without turning automation off', async () => {
  const setup = wallsFixture();
  for (const p of setup.fixture.players.slice(1)) { p.isHuman = false; setup.fixture.setup.players[p.id].is_human = false; }
  const city = setup.fixture.cities.find((c) => c.id === setup.own)!;
  city.producing = null;
  city.productionQueue = [];
  const client = new FixtureClient();
  client.load(coreFixture(setup.fixture));
  const before = client.view(0);
  const blocked = { ...before, cities: before.cities.map((c) => c.id === city.id
    ? { ...c, detail: { ...c.detail!, options: c.detail!.options.filter((o) => o.item.kind === 'Unit' && o.item.unit === 'Spearman') } } : c) };
  useGame.setState({ client, screen: 'game', humans: [0], views: [blocked],
    orders: [[{ type: 'SetCityAutomation', city: city.id, focus: 'Science' }]], errors: [[]], ready: [false] });
  await useGame.getState().toggleReady();
  let s = useGame.getState();
  expect(s.views![0].turn).toBe(before.turn);
  expect(s.ready[0]).toBe(false);
  expect(s.selection).toEqual({ kind: 'city', id: city.id });
  expect(pendingDecisions(blocked, s.orders[0], s.errors[0], s).find((d) => d.kind === 'city' && d.id === city.id)?.label).toContain('Nothing here fits its automation focus');
  expect(s.orders[0]).toContainEqual({ type: 'SetCityAutomation', city: city.id, focus: 'Science' });
  s.editOrders((orders) => [...orders, { type: 'SetProduction', city: city.id, item: { kind: 'Unit', unit: 'Spearman' } }]);
  await useGame.getState().toggleReady();
  await vi.waitFor(() => expect(useGame.getState().views![0].turn).toBe(before.turn + 1));
  s = useGame.getState();
  expect(s.views![0].cities.find((c) => c.id === city.id)?.detail?.build_automation).toBe('Science');
});
