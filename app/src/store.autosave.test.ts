import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { coopVsAiSetup } from './gameSetup';
import { FixtureClient } from './test/fixtureEngine';
import { AUTOSAVE_KEY, readAutosave, useGame, type SaveFile } from './store';
import { autosavesSettled, resetAutosaveStoreForTests } from './platform/autosaveStore';

vi.mock('./client', async (orig) => ({
  ...(await orig<typeof import('./client')>()),
  createClient: async () => new FixtureClient(),
}));

beforeEach(() => {
  vi.resetAllMocks();
  resetAutosaveStoreForTests();
  useGame.setState(useGame.getInitialState(), true);
});
afterEach(() => vi.unstubAllGlobals());

const start = () => useGame.getState().startGame(coopVsAiSetup('7', [{ name: 'Ann', faction: 'India' }]));

it('writes the autosave to localStorage', async () => {
  const setItem = vi.fn();
  vi.stubGlobal('localStorage', { setItem });
  await start();
  await autosavesSettled();
  expect(useGame.getState().error).toBeNull();
  expect(setItem).toHaveBeenCalledExactlyOnceWith(AUTOSAVE_KEY, expect.stringContaining('"format":"legatum-save"'));
});

it('keeps the game going when localStorage is unavailable', async () => {
  vi.stubGlobal('localStorage', { setItem: () => { throw new Error('Quota exceeded'); } });
  await start();
  await autosavesSettled();
  expect(useGame.getState().error).toBeNull();
  expect(useGame.getState().screen).toBe('game');
  expect(useGame.getState().autosaveNotice).toBe(true);
});

const saved = (savedAt: string, turn: number): SaveFile => ({
  format: 'legatum-save', version: 1, client: 'wasm', savedAt, turn, names: ['Ann'], game: '{"seed":18446744073709551615}',
});
const newer = saved('2026-09-26T12:00:00Z', 1);

it('reads a valid autosave', async () => {
  vi.stubGlobal('localStorage', { getItem: () => JSON.stringify(newer) });
  expect(await readAutosave()).toEqual(newer);
});

it.each(['not JSON', '{"format":"legatum-save"}', JSON.stringify({ ...newer, savedAt: 'invalid' })])('ignores a corrupt autosave: %s', async (bad) => {
  vi.stubGlobal('localStorage', { getItem: () => bad });
  expect(await readAutosave()).toBeNull();
});

it('returns null when localStorage throws', async () => {
  vi.stubGlobal('localStorage', { getItem: () => { throw new Error('Storage blocked'); } });
  expect(await readAutosave()).toBeNull();
});

it('refuses another rules version on load without changing the stored save', async () => {
  const save: SaveFile = { ...saved('2026-10-31T12:00:00Z', 4), playtest: 'Playtest 8.9.19', game: '{"save_version":5}' };
  const contents = JSON.stringify(save);
  const setItem = vi.fn();
  vi.stubGlobal('localStorage', { setItem });
  await useGame.getState().loadSave(save);
  expect(useGame.getState().error).toBe("This game was saved by Playtest 8.9.19 with older rules and can't continue. The file is not changed.");
  expect(useGame.getState().client).toBeNull();
  expect(setItem).not.toHaveBeenCalled();
  expect(JSON.stringify(save)).toBe(contents);
});
