import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { coopVsAiSetup } from './gameSetup';
import { FixtureClient } from './test/fixtureEngine';
import { readNativeAutosave, writeNativeAutosave } from './platform/files';
import { AUTOSAVE_KEY, readAutosave, useGame, type SaveFile } from './store';

vi.mock('./client', async (orig) => ({
  ...(await orig<typeof import('./client')>()),
  createClient: async () => new FixtureClient(),
}));
vi.mock('./platform/files', () => ({ writeNativeAutosave: vi.fn(), readNativeAutosave: vi.fn() }));

beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(writeNativeAutosave).mockResolvedValue(undefined);
  vi.mocked(readNativeAutosave).mockResolvedValue(null);
  useGame.setState(useGame.getInitialState(), true);
});
afterEach(() => vi.unstubAllGlobals());

const start = () => useGame.getState().startGame(coopVsAiSetup('7', [{ name: 'Ann', faction: 'India' }]));

it('writes the same save to localStorage and the native mirror', async () => {
  const setItem = vi.fn();
  vi.stubGlobal('localStorage', { setItem });
  await start();
  expect(useGame.getState().error).toBeNull();
  const contents = setItem.mock.calls[0][1];
  expect(setItem).toHaveBeenCalledWith(AUTOSAVE_KEY, contents);
  expect(writeNativeAutosave).toHaveBeenCalledExactlyOnceWith(contents);
});

it('still mirrors the save when localStorage is unavailable', async () => {
  vi.stubGlobal('localStorage', { setItem: () => { throw new Error('Quota exceeded'); } });
  await start();
  expect(useGame.getState().error).toBeNull();
  expect(writeNativeAutosave).toHaveBeenCalledOnce();
});

it('keeps the local autosave and game when the disk mirror fails', async () => {
  const setItem = vi.fn();
  vi.stubGlobal('localStorage', { setItem });
  vi.mocked(writeNativeAutosave).mockRejectedValue(new Error('Disk full'));
  await start();
  expect(setItem).toHaveBeenCalledOnce();
  expect(useGame.getState().error).toBeNull();
  expect(useGame.getState().screen).toBe('game');
});

const saved = (savedAt: string, turn: number): SaveFile => ({
  format: 'legatum-save', version: 1, client: 'wasm', savedAt, turn, names: ['Ann'], game: '{"seed":18446744073709551615}',
});
const older = saved('2026-09-26T11:00:00Z', 10);
const newer = saved('2026-09-26T12:00:00Z', 1); // A newer session need not have the higher turn.

it.each([
  { local: older, native: newer, expected: newer },
  { local: newer, native: older, expected: newer },
  { local: null, native: newer, expected: newer },
  { local: newer, native: null, expected: newer },
  { local: older, native: older, expected: older },
])('recovers the newest valid autosave by savedAt: $local.savedAt / $native.savedAt', async ({ local, native, expected }) => {
  vi.stubGlobal('localStorage', { getItem: () => local && JSON.stringify(local) });
  vi.mocked(readNativeAutosave).mockResolvedValue(native && JSON.stringify(native));
  expect(await readAutosave()).toEqual(expected);
});

it.each(['not JSON', '{"format":"legatum-save"}', JSON.stringify({ ...newer, savedAt: 'invalid' })])('ignores a corrupt copy independently: %s', async (bad) => {
  vi.stubGlobal('localStorage', { getItem: () => bad });
  vi.mocked(readNativeAutosave).mockResolvedValue(JSON.stringify(older));
  expect(await readAutosave()).toEqual(older);
  vi.stubGlobal('localStorage', { getItem: () => JSON.stringify(older) });
  vi.mocked(readNativeAutosave).mockResolvedValue(bad);
  expect(await readAutosave()).toEqual(older);
});

it('recovers disk storage when localStorage throws, and returns null if both copies are unavailable', async () => {
  vi.stubGlobal('localStorage', { getItem: () => { throw new Error('Storage blocked'); } });
  vi.mocked(readNativeAutosave).mockResolvedValue(JSON.stringify(newer));
  expect(await readAutosave()).toEqual(newer);
  vi.mocked(readNativeAutosave).mockRejectedValue(new Error('Permission denied'));
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
