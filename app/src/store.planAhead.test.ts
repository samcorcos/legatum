// @vitest-environment happy-dom
// The AI empires decide while the player plans (store.ts startPlanningAhead): one empire per idle moment, so End Turn
// only resolves. The engine side (the same game however often it is called) is tested in crates/wasm/src/session.rs.
import { IDBFactory } from 'fake-indexeddb';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { coopVsAiSetup } from './gameSetup';
import { FixtureClient } from './test/fixtureEngine';
import { useGame } from './store';
import { autosavesSettled, listAutosaves, resetAutosaveStoreForTests } from './platform/autosaveStore';

vi.mock('./client', async (orig) => ({
  ...(await orig<typeof import('./client')>()),
  createClient: async () => new FixtureClient(),
}));

beforeEach(() => {
  resetAutosaveStoreForTests();
  const items = new Map<string, string>();
  vi.stubGlobal('indexedDB', new IDBFactory());
  vi.stubGlobal('localStorage', { getItem: (k: string) => items.get(k) ?? null, setItem: (k: string, v: string) => { items.set(k, v); }, removeItem: (k: string) => { items.delete(k); } });
  useGame.setState(useGame.getInitialState(), true);
});
afterEach(() => {
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

const start = () => useGame.getState().startGame(coopVsAiSetup('7', [{ name: 'Ann', faction: 'India' }]));
const endTurn = async () => {
  const turn = useGame.getState().views![0].turn;
  await useGame.getState().toggleReady();
  await vi.waitFor(() => expect(useGame.getState().views![0].turn).toBe(turn + 1), { timeout: 10000 });
};
type Planner = { planAhead?: () => number };

it('asks the engine for one empire at a time after a turn starts, until none is left', async () => {
  await start();
  const client = useGame.getState().client as unknown as Planner;
  let left = 3;
  client.planAhead = vi.fn(() => (left = Math.max(0, left - 1)));
  await endTurn();
  await vi.waitFor(() => expect(client.planAhead).toHaveBeenCalledTimes(3), { timeout: 8000 });
  await new Promise((done) => setTimeout(done, 400));
  expect(client.planAhead).toHaveBeenCalledTimes(3);
});

it('stops deciding ahead when End Turn is pressed, and starts again for the next turn', async () => {
  await start();
  const client = useGame.getState().client as unknown as Planner;
  client.planAhead = vi.fn(() => 5);
  await endTurn();
  await vi.waitFor(() => expect(client.planAhead).toHaveBeenCalled(), { timeout: 8000 });
  await endTurn();
  const after = vi.mocked(client.planAhead).mock.calls.length;
  // Nothing is asked in the first second of a turn: its playback has the frames to itself.
  await new Promise((done) => setTimeout(done, 600));
  expect(client.planAhead).toHaveBeenCalledTimes(after);
  await vi.waitFor(() => expect(vi.mocked(client.planAhead!).mock.calls.length).toBeGreaterThan(after), { timeout: 8000 });
});

it('a failure while deciding ahead restores the engine and leaves the turn playable', async () => {
  await start();
  const client = useGame.getState().client!;
  vi.spyOn(console, 'error').mockImplementation(() => {});
  const load = vi.spyOn(client, 'load');
  (client as unknown as Planner).planAhead = vi.fn(() => { throw new Error('unreachable'); });
  await endTurn();
  await vi.waitFor(() => expect(load).toHaveBeenCalledTimes(1), { timeout: 8000 });
  await new Promise((done) => setTimeout(done, 400));
  expect((client as unknown as Planner).planAhead).toHaveBeenCalledTimes(1);
  expect(useGame.getState().error).toBeNull();
  await endTurn();
});

it('writes the autosave before the next End Turn resolves, with no frame in between', async () => {
  await start();
  await autosavesSettled();
  const turn = useGame.getState().views![0].turn;
  await endTurn();
  await endTurn();
  await new Promise<void>((done) => requestAnimationFrame(() => setTimeout(done, 0)));
  await autosavesSettled();
  const turns = (await listAutosaves()).map((r) => r.turn).sort((a, b) => a - b);
  expect(turns).toContain(turn + 1);
  expect(turns).toContain(turn + 2);
});
