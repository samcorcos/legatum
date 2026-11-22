// @vitest-environment happy-dom
import { IDBFactory } from 'fake-indexeddb';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { coopVsAiSetup } from './gameSetup';
import { FixtureClient } from './test/fixtureEngine';
import { parseAutosave, useGame } from './store';
import { setProduction } from './orders';
import { autosavesSettled, listAutosaves, readPendingOrders, resetAutosaveStoreForTests, writePendingOrders } from './platform/autosaveStore';

vi.mock('./client', async (orig) => ({
  ...(await orig<typeof import('./client')>()),
  createClient: async () => new FixtureClient(),
}));

class MemoryStorage {
  items = new Map<string, string>();
  failWrites = false;
  getItem = (k: string) => this.items.get(k) ?? null;
  setItem = (k: string, v: string) => { if (this.failWrites) throw new Error('Quota exceeded'); this.items.set(k, v); };
  removeItem = (k: string) => { this.items.delete(k); };
}
let local: MemoryStorage;

beforeEach(() => {
  resetAutosaveStoreForTests();
  local = new MemoryStorage();
  vi.stubGlobal('indexedDB', new IDBFactory());
  vi.stubGlobal('localStorage', local);
  useGame.setState(useGame.getInitialState(), true);
});
afterEach(() => {
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

const start = (seed = '7') => useGame.getState().startGame(coopVsAiSetup(seed, [{ name: 'Ann', faction: 'India' }]));
const endTurn = async () => {
  const turn = useGame.getState().views![0].turn;
  await useGame.getState().toggleReady();
  await vi.waitFor(() => expect(useGame.getState().views![0].turn).toBe(turn + 1), { timeout: 10000 });
  await autosavesSettled();
};
const orderSomething = () => {
  const view = useGame.getState().views![0];
  const city = view.cities.find((c) => c.owner === view.me)!;
  useGame.getState().editOrders((o) => setProduction(o, city.id, { kind: 'Unit', unit: 'Spearman' }, null));
};
const kept = (turn: number) => readPendingOrders(campaignOf.id, turn);
const campaignOf = { id: '' };
const campaign = async () => (campaignOf.id = (await listAutosaves())[0].campaign);
/** Pretend the page was reloaded: the store forgets the game, the stored saves stay. */
const reload = () => useGame.setState(useGame.getInitialState(), true);
const continueNewest = async () => {
  const [record] = await listAutosaves();
  await useGame.getState().continueAutosave(record, parseAutosave(record.text)!);
};

it('saves each turn as its own slot and keeps the game\'s earlier turns', async () => {
  await start();
  await autosavesSettled();
  await endTurn();
  const list = await listAutosaves();
  expect(list.map((r) => [r.turn, r.label])).toEqual([[2, 'Ann'], [1, 'Ann']]);
  expect(new Set(list.map((r) => r.campaign)).size).toBe(1);
});

it('a new game replaces the old game\'s slots once its first save is in', async () => {
  await start('7');
  await autosavesSettled();
  await endTurn();
  const first = (await listAutosaves())[0].campaign;
  await start('8');
  await autosavesSettled();
  const list = await listAutosaves();
  expect(list).toHaveLength(1);
  expect(list[0].campaign).not.toBe(first);
});

it('Continue keeps the same game\'s slots', async () => {
  await start();
  await autosavesSettled();
  await endTurn();
  const before = (await listAutosaves())[0].campaign;
  reload();
  await continueNewest();
  await endTurn();
  const { listCampaignTurns } = await import('./platform/autosaveStore');
  expect((await listCampaignTurns(before)).map((r) => r.turn)).toEqual([3, 2, 1]);
  expect((await listAutosaves()).every((r) => r.campaign === before)).toBe(true);
});

it('a failed save shows the notice once per game, and a later success takes it away', async () => {
  vi.stubGlobal('indexedDB', undefined);
  local.failWrites = true;
  await start();
  await autosavesSettled();
  expect(useGame.getState().autosaveNotice).toBe(true);
  useGame.getState().set({ autosaveNotice: false });
  await endTurn(); // fails again, same game: no second notice
  expect(useGame.getState().autosaveNotice).toBe(false);
  await start('9'); // another game: it gets its own
  await autosavesSettled();
  expect(useGame.getState().autosaveNotice).toBe(true);
  local.failWrites = false;
  await endTurn();
  expect(useGame.getState().autosaveNotice).toBe(false);
});

it('a game too big to save also shows the notice', async () => {
  await start();
  await autosavesSettled();
  useGame.getState().set({ autosaveNotice: false });
  vi.spyOn(useGame.getState().client!, 'save').mockReturnValue(' '.repeat(65 * 1024 * 1024));
  await endTurn();
  expect(useGame.getState().autosaveNotice).toBe(true);
});

it('orders entered for the turn come back on Continue', async () => {
  await start();
  await autosavesSettled();
  await campaign();
  orderSomething();
  const entered = useGame.getState().orders[0];
  expect(entered).toHaveLength(1);
  await vi.waitFor(async () => expect(await kept(1)).not.toBeNull(), { timeout: 3000 });
  reload();
  expect(useGame.getState().orders).toEqual([]);
  await continueNewest();
  expect(useGame.getState().orders[0]).toEqual(entered);
  expect(useGame.getState().errors[0]).toEqual([]);
  expect(useGame.getState().ready[0]).toBe(false);
});

it('silently drops the orders the engine now refuses', async () => {
  await start();
  await autosavesSettled();
  await campaign();
  orderSomething();
  await vi.waitFor(async () => expect(await kept(1)).not.toBeNull(), { timeout: 3000 });
  reload();
  vi.spyOn(FixtureClient.prototype, 'validate').mockImplementation((_p, orders) => orders.map((_, index) => ({ index, message: 'No longer possible' })));
  await continueNewest();
  expect(useGame.getState().orders[0]).toEqual([]);
  expect(useGame.getState().error).toBeNull();
});

it('orders kept for another turn are not restored', async () => {
  await start();
  await autosavesSettled();
  const id = await campaign();
  await writePendingOrders({ campaign: id, turn: 5, text: JSON.stringify([[{ type: 'SetProduction' }]]) });
  reload();
  await continueNewest();
  expect(useGame.getState().orders[0]).toEqual([]);
});

it('warns before closing only while orders sit unsaved because the browser would not keep them', async () => {
  vi.stubGlobal('indexedDB', undefined);
  local.failWrites = true;
  await start();
  await autosavesSettled();
  const closing = () => { const e = new Event('beforeunload', { cancelable: true }); window.dispatchEvent(e); return e.defaultPrevented; };
  expect(closing()).toBe(false);
  orderSomething();
  await new Promise((r) => setTimeout(r, 600));
  expect(closing()).toBe(true);
  local.failWrites = false;
  useGame.getState().editOrders(() => []); // nothing pending any more, and that write works
  await new Promise((r) => setTimeout(r, 600));
  expect(closing()).toBe(false);
});

it('Load an earlier turn opens that turn, records the rewind, and drops the turns after it', async () => {
  await start();
  await endTurn();
  await endTurn();
  await endTurn();
  const game = await campaign();
  const now = useGame.getState().views![0].turn;
  const { listCampaignTurns, readAutosaveText } = await import('./platform/autosaveStore');
  const target = (await listCampaignTurns(game)).find((t) => t.turn === now - 2)!;
  await useGame.getState().rewindTo(game, parseAutosave(await readAutosaveText(target.id))!);
  await autosavesSettled();
  expect(useGame.getState().error).toBeNull();
  expect(useGame.getState().views![0].turn).toBe(now - 2);
  expect(useGame.getState().rewinds).toMatchObject([{ from_turn: now, to_turn: now - 2 }]);
  expect((await listCampaignTurns(game)).map((t) => t.turn)).toEqual([now - 2, now - 3]);
  // The rewind is saved with the game, so a reload and a second rewind (from the title) keep the first one.
  const [newest] = await listAutosaves();
  expect(newest.campaign).toBe(game);
  expect(parseAutosave(newest.text)!.rewinds).toMatchObject([{ from_turn: now, to_turn: now - 2 }]);
  await endTurn();
  reload();
  const first = (await listCampaignTurns(game)).find((t) => t.turn === now - 3)!;
  await useGame.getState().rewindTo(game, parseAutosave(await readAutosaveText(first.id))!);
  expect(useGame.getState().rewinds).toMatchObject([{ from_turn: now, to_turn: now - 2 }, { from_turn: now - 1, to_turn: now - 3 }]);
});

it('a new game starts with no rewinds', async () => {
  useGame.setState({ rewinds: [{ at: '2026-11-22T19:10:00.000Z', from_turn: 4, to_turn: 2 }] });
  await start();
  expect(useGame.getState().rewinds).toEqual([]);
});
