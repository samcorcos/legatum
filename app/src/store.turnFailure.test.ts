// @vitest-environment happy-dom
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { coopVsAiSetup } from './gameSetup';
import { FixtureClient } from './test/fixtureEngine';
import { useGame } from './store';

vi.mock('./client', async (orig) => ({
  ...(await orig<typeof import('./client')>()),
  createClient: async () => new FixtureClient(),
}));
vi.mock('./platform/archive', () => ({
  recordGameHistory: vi.fn().mockResolvedValue(undefined),
  abandonGameHistory: vi.fn().mockResolvedValue(undefined),
}));

beforeEach(() => {
  vi.spyOn(console, 'error').mockImplementation(() => {});
  useGame.setState(useGame.getInitialState(), true);
});
afterEach(() => {
  vi.restoreAllMocks();
});

const start = async () => {
  await useGame.getState().startGame(coopVsAiSetup('7', [{ name: 'Ann', faction: 'India' }]));
  return useGame.getState().client as FixtureClient;
};
const endTurn = async () => {
  useGame.getState().set({ error: null });
  await useGame.getState().toggleReady();
};
const settle = () => vi.waitFor(() => expect(useGame.getState().busy).toBeNull(), { timeout: 5000 });

/** Makes the engine throw on its next `times` end-turn calls, like a trapped engine does. */
function failTimes(client: FixtureClient, times: number) {
  const real = client.endTurn.bind(client);
  let left = times;
  const spy = vi.spyOn(client, 'endTurn').mockImplementation((orders) => {
    if (left-- > 0) throw new Error('unreachable');
    return real(orders);
  });
  return spy;
}

it('keeps today\'s behaviour on the first failure: restored game, orders kept, one error line', async () => {
  const client = await start();
  failTimes(client, 1);
  useGame.setState({ orders: [[{ type: 'SetResearch', tech: 'Philosophy' }]] });
  await endTurn();
  await vi.waitFor(() => expect(useGame.getState().error).toContain('The turn could not be resolved'));
  const s = useGame.getState();
  expect(s.turnFailure).toBeNull();
  expect(s.error).toContain('your orders are kept');
  expect(s.orders[0]).toHaveLength(1);
  expect(s.views![0].turn).toBe(1);
});

it('opens the recovery dialog on the second failure of the same turn', async () => {
  const client = await start();
  failTimes(client, 2);
  await endTurn();
  await vi.waitFor(() => expect(useGame.getState().error).toContain('could not be resolved'));
  await endTurn();
  await vi.waitFor(() => expect(useGame.getState().turnFailure).toEqual({ cleared: false }));
  expect(useGame.getState().error).toBeNull();
  expect(useGame.getState().busy).toBeNull();
});

it('ends the turn with the orders cleared, and the dialog goes away', async () => {
  const client = await start();
  const spy = failTimes(client, 2);
  useGame.setState({ orders: [[{ type: 'SetResearch', tech: 'Philosophy' }]] });
  await endTurn();
  await vi.waitFor(() => expect(useGame.getState().error).toBeTruthy());
  await endTurn();
  await vi.waitFor(() => expect(useGame.getState().turnFailure).not.toBeNull());
  useGame.getState().endTurnWithoutOrders();
  await vi.waitFor(() => expect(useGame.getState().views![0].turn).toBe(2), { timeout: 5000 });
  expect(spy).toHaveBeenLastCalledWith({ 0: [] });
  expect(useGame.getState().turnFailure).toBeNull();
});

it('after a third failure without orders, says so and offers only Save and report', async () => {
  const client = await start();
  failTimes(client, 3);
  await endTurn();
  await vi.waitFor(() => expect(useGame.getState().error).toBeTruthy());
  await endTurn();
  await vi.waitFor(() => expect(useGame.getState().turnFailure).toEqual({ cleared: false }));
  useGame.getState().endTurnWithoutOrders();
  await vi.waitFor(() => expect(useGame.getState().turnFailure).toEqual({ cleared: true }));
  await settle();
  expect(useGame.getState().views![0].turn).toBe(1);
});

it('a failure on a later turn starts the count again', async () => {
  const client = await start();
  const real = client.endTurn.bind(client);
  let calls = 0;
  vi.spyOn(client, 'endTurn').mockImplementation((orders) => {
    calls++;
    if (calls === 1 || calls === 3) throw new Error('unreachable');
    return real(orders);
  });
  await endTurn();
  await vi.waitFor(() => expect(useGame.getState().error).toBeTruthy());
  await endTurn(); // succeeds
  await vi.waitFor(() => expect(useGame.getState().views![0].turn).toBe(2));
  await endTurn(); // fails once on turn 2
  await vi.waitFor(() => expect(useGame.getState().error).toBeTruthy());
  expect(useGame.getState().turnFailure).toBeNull();
});
