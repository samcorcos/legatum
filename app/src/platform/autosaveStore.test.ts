import { IDBFactory, IDBObjectStore } from 'fake-indexeddb';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import {
  AUTOSAVE_DB, AUTOSAVE_SLOTS, LEGACY_AUTOSAVE_KEY, SKIPPED_SLOTS_KEY, autosavesSettled, listAutosaves, listCampaignTurns, readAutosaveText, readPendingOrders,
  resetAutosaveStoreForTests, skipAutosave, writeAutosave, writePendingOrders, type AutosaveDraft,
} from './autosaveStore';

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
});
afterEach(() => {
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

const draft = (turn: number, campaign = 'game-a'): AutosaveDraft => ({
  campaign, turn, savedAt: new Date(Date.UTC(2026, 10, 22, 10, turn)).toISOString(), label: 'Ann & Bo', text: JSON.stringify({ turn, savedAt: 'x'.repeat(10) }),
});
const legacy = (turn: number, minute = turn) => JSON.stringify({ format: 'legatum-save', turn, names: ['Ann'], savedAt: new Date(Date.UTC(2026, 10, 22, 9, minute)).toISOString(), game: '{}' });
const turns = async () => (await listAutosaves()).map((r) => r.turn);

it('keeps the latest turn and lists the older ones, newest first, with what the title screen shows', async () => {
  for (const turn of [1, 2, 3]) expect(await writeAutosave(draft(turn))).toBe('database');
  const [newest, ...older] = await listAutosaves();
  expect(newest).toMatchObject({ id: 'game-a:3', campaign: 'game-a', turn: 3, label: 'Ann & Bo' });
  expect(older.map((r) => r.turn)).toEqual([2]);
  expect((await listCampaignTurns('game-a')).map((r) => r.turn)).toEqual([3, 2, 1]);
});

it('keeps ten turns of a game and drops the oldest: the newest two as text, the rest gzipped', async () => {
  for (let turn = 1; turn <= AUTOSAVE_SLOTS + 3; turn++) await writeAutosave(draft(turn));
  // Continue, the crash screen and older builds see the plain slots only.
  expect(await turns()).toEqual([13, 12]);
  const kept = await listCampaignTurns('game-a');
  expect(kept.map((t) => t.turn)).toEqual([13, 12, 11, 10, 9, 8, 7, 6, 5, 4]);
  expect(kept[0]).not.toHaveProperty('text');
  // A compacted turn reads back exactly.
  expect(await readAutosaveText('game-a:5')).toBe(draft(5).text);
  expect(await readAutosaveText('game-a:13')).toBe(draft(13).text);
  expect(await readAutosaveText('game-a:1')).toBeNull();
  const db = await new Promise<IDBDatabase>((resolve) => { const r = indexedDB.open(AUTOSAVE_DB, 1); r.onsuccess = () => resolve(r.result); });
  const stored = await new Promise<{ text?: string; gz?: ArrayBuffer }>((resolve) => {
    const req = db.transaction('saves').objectStore('saves').get('game-a:5');
    req.onsuccess = () => { db.close(); resolve(req.result); };
  });
  expect(stored.text).toBeUndefined();
  expect(stored.gz).toBeInstanceOf(ArrayBuffer);
});

it('going back to an earlier turn drops the later turns once that turn is saved again', async () => {
  for (let turn = 1; turn <= 6; turn++) await writeAutosave(draft(turn));
  // The player reopened turn 3: it is saved again on opening, and turns 4 to 6 belong to a line they left.
  await writeAutosave({ ...draft(3), savedAt: '2026-11-22T12:00:00Z' });
  expect((await listCampaignTurns('game-a')).map((t) => t.turn)).toEqual([3, 2, 1]);
  expect(await turns()).toEqual([3, 2]);
  await writeAutosave({ ...draft(4), savedAt: '2026-11-22T12:01:00Z' });
  expect((await listCampaignTurns('game-a')).map((t) => t.turn)).toEqual([4, 3, 2, 1]);
  expect(await listCampaignTurns('other')).toEqual([]);
});

it('writing the same turn again replaces that slot', async () => {
  await writeAutosave(draft(4));
  await writeAutosave({ ...draft(4), text: 'second' });
  const list = await listAutosaves();
  expect(list).toHaveLength(1);
  expect(list[0].text).toBe('second');
});

it('another game replaces the old slots only once its first save is in', async () => {
  await writeAutosave(draft(9, 'old'));
  await writeAutosave(draft(10, 'old'));
  // The new game's first write fails everywhere: the old game is still there.
  vi.spyOn(IDBObjectStore.prototype, 'put').mockImplementationOnce(() => { throw new Error('Quota exceeded'); });
  local.failWrites = true;
  await expect(writeAutosave({ ...draft(1, 'new'), savedAt: '2026-11-23T10:00:00Z' })).rejects.toThrow();
  expect((await listAutosaves()).map((r) => r.campaign)).toEqual(['old', 'old']);
  // Its first successful write takes the old slots away.
  local.failWrites = false;
  await writeAutosave({ ...draft(1, 'new'), savedAt: '2026-11-23T10:01:00Z' });
  expect((await listAutosaves()).map((r) => r.campaign)).toEqual(['new']);
});

it('serialises writes: the order they were asked in is the order they land', async () => {
  void writeAutosave(draft(1));
  void writeAutosave(draft(2));
  void writeAutosave(draft(3));
  await autosavesSettled();
  expect((await listCampaignTurns('game-a')).map((r) => r.turn)).toEqual([3, 2, 1]);
});

it('uses the old single slot when there is no database, and says so when nothing works', async () => {
  vi.stubGlobal('indexedDB', undefined);
  expect(await writeAutosave(draft(5))).toBe('local');
  expect(local.getItem(LEGACY_AUTOSAVE_KEY)).toBe(draft(5).text);
  expect((await listAutosaves())).toHaveLength(1);
  local.failWrites = true;
  await expect(writeAutosave(draft(6))).rejects.toThrow();
});

it('falls back to the old slot when the database refuses the write', async () => {
  vi.spyOn(IDBObjectStore.prototype, 'put').mockImplementation(() => { throw new DOMException('full', 'QuotaExceededError'); });
  expect(await writeAutosave(draft(5))).toBe('local');
  expect(local.getItem(LEGACY_AUTOSAVE_KEY)).toBe(draft(5).text);
});

it('copies a save from an older build into the database, and removes the old key only after a newer write', async () => {
  local.setItem(LEGACY_AUTOSAVE_KEY, legacy(7));
  const first = await listAutosaves();
  expect(first.map((r) => r.turn)).toEqual([7]);
  expect(local.getItem(LEGACY_AUTOSAVE_KEY)).not.toBeNull();
  // It was copied: with the old key gone the same save is still read, and listing again does not double it.
  expect(await turns()).toEqual([7]);
  await writeAutosave({ ...draft(8, 'legacy'), savedAt: '2026-11-22T11:00:00Z' });
  expect(local.getItem(LEGACY_AUTOSAVE_KEY)).toBeNull();
  expect(await turns()).toEqual([8, 7]);
});

it('keeps the old key when the database cannot be opened', async () => {
  vi.stubGlobal('indexedDB', undefined);
  local.setItem(LEGACY_AUTOSAVE_KEY, legacy(3));
  expect(await turns()).toEqual([3]);
  expect(local.getItem(LEGACY_AUTOSAVE_KEY)).not.toBeNull();
});

it('ignores a damaged old slot and records that are not saves', async () => {
  local.setItem(LEGACY_AUTOSAVE_KEY, 'not json');
  expect(await listAutosaves()).toEqual([]);
  await writeAutosave(draft(2));
  const db = await new Promise<IDBDatabase>((resolve) => { const r = indexedDB.open(AUTOSAVE_DB, 1); r.onsuccess = () => resolve(r.result); });
  await new Promise<void>((resolve) => { const tx = db.transaction('saves', 'readwrite'); tx.objectStore('saves').put({ id: 'junk', text: 5 }); tx.oncomplete = () => { db.close(); resolve(); }; });
  expect(await turns()).toEqual([2]);
});

it('asks the browser to keep the data once per game, after the first successful save, and never fails because of it', async () => {
  const persist = vi.fn(async () => true);
  vi.stubGlobal('navigator', { storage: { persist } });
  vi.spyOn(IDBObjectStore.prototype, 'put').mockImplementationOnce(() => { throw new Error('nope'); });
  local.failWrites = true;
  await expect(writeAutosave(draft(1))).rejects.toThrow();
  expect(persist).not.toHaveBeenCalled();
  local.failWrites = false;
  await writeAutosave(draft(1));
  await writeAutosave(draft(2));
  expect(persist).toHaveBeenCalledTimes(1);
  await writeAutosave(draft(1, 'game-b'));
  expect(persist).toHaveBeenCalledTimes(2);
  vi.stubGlobal('navigator', { storage: { persist: () => { throw new Error('blocked'); } } });
  await expect(writeAutosave(draft(1, 'game-c'))).resolves.toBe('database');
});

it('hides a slot the player stepped back from until a newer save lands', async () => {
  await writeAutosave(draft(1));
  await writeAutosave(draft(2));
  skipAutosave('game-a:2');
  expect(await turns()).toEqual([1]);
  expect(JSON.parse(local.getItem(SKIPPED_SLOTS_KEY)!)).toEqual(['game-a:2']);
  await writeAutosave(draft(2));
  expect(await turns()).toEqual([2, 1]);
});

it('keeps this turn\'s orders and returns them only for the same game and turn', async () => {
  await writePendingOrders({ campaign: 'game-a', turn: 4, text: '[[1]]' });
  expect(await readPendingOrders('game-a', 4)).toBe('[[1]]');
  expect(await readPendingOrders('game-a', 5)).toBeNull();
  expect(await readPendingOrders('game-b', 4)).toBeNull();
  await writePendingOrders({ campaign: 'game-a', turn: 4, text: null });
  expect(await readPendingOrders('game-a', 4)).toBeNull();
});

it('keeps orders in the old storage when there is no database, and rejects when nothing works', async () => {
  vi.stubGlobal('indexedDB', undefined);
  await writePendingOrders({ campaign: 'game-a', turn: 4, text: '[[2]]' });
  expect(await readPendingOrders('game-a', 4)).toBe('[[2]]');
  local.failWrites = true;
  await expect(writePendingOrders({ campaign: 'game-a', turn: 5, text: '[[3]]' })).rejects.toThrow();
});
