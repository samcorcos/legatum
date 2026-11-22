import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import type { GameClient } from '../client';
import { loadFromFile, saveToFile } from '../platform/files';
import { AUTOSAVE_KEY, saveFile, useGame } from '../store';
import type { PlayerView } from '../types';
import { pickSaveFile, saveGameToFile } from './GameMenu';
import { MAX_SAVE_BYTES } from '../platform/saveLimits';

vi.mock('../platform/files', () => ({
  loadFromFile: vi.fn(),
  saveToFile: vi.fn(),
}));

const game = '{\n"seed":18446744073709551615,"seen":[9007199254740993]\n}';
const client = { kind: 'wasm', save: () => game } as GameClient;
const views = [{ turn: 9, me: 0, players: [{ id: 0, name: 'Sam' }] }] as PlayerView[];

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(loadFromFile).mockReset();
  vi.mocked(saveToFile).mockReset();
  useGame.setState(useGame.getInitialState(), true);
  useGame.setState({ client, views });
});
afterEach(() => vi.unstubAllGlobals());

it('passes the engine save through the file envelope and back without rounding u64s', async () => {
  saveGameToFile();
  const [text, turn] = vi.mocked(saveToFile).mock.calls[0];
  expect(turn).toBe(9);
  vi.mocked(loadFromFile).mockResolvedValue(text);
  const onLoad = vi.fn();
  const onError = vi.fn();
  await pickSaveFile(onLoad, onError);
  expect(onError).not.toHaveBeenCalled();
  expect(onLoad).toHaveBeenCalledOnce();
  expect(onLoad.mock.calls[0][0]).toMatchObject({ format: 'legatum-save', version: 1, turn: 9, game });
});

it('shows save failures in the existing error UI', async () => {
  vi.mocked(saveToFile).mockImplementation(() => { throw new Error('Disk full'); });
  saveGameToFile();
  expect(useGame.getState().error).toBe('Error: Disk full');
});

it('does not replace a game or show an error on picker cancellation', async () => {
  vi.mocked(loadFromFile).mockResolvedValue(null);
  const onLoad = vi.fn();
  const onError = vi.fn();
  await pickSaveFile(onLoad, onError);
  expect(onLoad).not.toHaveBeenCalled();
  expect(onError).not.toHaveBeenCalled();
});

it('rejects an oversized save before parsing without touching the current game or autosave', async () => {
  const original = useGame.getState().client;
  const autosave = 'existing autosave';
  const storage = new Map<string, string>();
  vi.stubGlobal('localStorage', {
    getItem: (key: string) => storage.get(key) ?? null,
    setItem: (key: string, value: string) => storage.set(key, value),
  });
  localStorage.setItem(AUTOSAVE_KEY, autosave);
  vi.mocked(loadFromFile).mockResolvedValue(' '.repeat(MAX_SAVE_BYTES + 1));
  const onLoad = vi.fn();
  const onError = vi.fn();
  await pickSaveFile(onLoad, onError);
  expect(onLoad).not.toHaveBeenCalled();
  expect(onError).toHaveBeenCalledWith(expect.stringContaining('64 MiB limit'));
  expect(useGame.getState().client).toBe(original);
  expect(localStorage.getItem(AUTOSAVE_KEY)).toBe(autosave);
});

it.each(['{', 'null', '{"format":"other"}', '{"format":"legatum-save","game":{}}'])(
  'rejects an invalid envelope (%s) before loading', async (text) => {
    vi.mocked(loadFromFile).mockResolvedValue(text);
    const onLoad = vi.fn();
    const onError = vi.fn();
    await pickSaveFile(onLoad, onError);
    expect(onLoad).not.toHaveBeenCalled();
    expect(onError).toHaveBeenCalledOnce();
  },
);

it('reports native read failures and asynchronous load failures', async () => {
  const onLoad = vi.fn().mockRejectedValue(new Error('Cannot restore game'));
  const onError = vi.fn();
  vi.mocked(loadFromFile).mockRejectedValueOnce(new Error('Permission denied'));
  await pickSaveFile(onLoad, onError);
  expect(onLoad).not.toHaveBeenCalled();
  expect(onError).toHaveBeenLastCalledWith('Could not load the save: Permission denied');
  vi.mocked(loadFromFile).mockResolvedValue(JSON.stringify(saveFile(client, views)));
  await pickSaveFile(onLoad, onError);
  expect(onError).toHaveBeenLastCalledWith('Could not load the save: Cannot restore game');
});
