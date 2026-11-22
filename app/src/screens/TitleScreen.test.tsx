// @vitest-environment happy-dom
import { act, StrictMode } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { TitleScreen } from './TitleScreen';
import { readNativeAutosave } from '../platform/files';
import { saveFile, useGame } from '../store';
import { FixtureClient } from '../test/fixtureEngine';

vi.mock('./TitleArt', () => ({ TitleArt: () => null }));
vi.mock('../platform/files', async (orig) => ({
  ...(await orig<typeof import('../platform/files')>()), readNativeAutosave: vi.fn(),
}));
let root: Root;
let host: HTMLDivElement;
beforeEach(() => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  localStorage.clear();
  useGame.setState(useGame.getInitialState(), true);
  host = document.body.appendChild(document.createElement('div'));
  root = createRoot(host);
});
afterEach(() => {
  act(() => root.unmount());
  host.remove();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

it('enables Continue after the asynchronous native recovery and loads that exact save', async () => {
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('42', 'India', 'Byzantium', 'Ada', 'Bo'));
  const save = saveFile(client, [client.view(0), client.view(1)]);
  let finish!: (value: string) => void;
  const pending = new Promise<string>((resolve) => { finish = resolve; });
  vi.mocked(readNativeAutosave).mockReturnValue(pending);
  act(() => root.render(<StrictMode><TitleScreen /></StrictMode>));
  const button = [...host.querySelectorAll('button')].find((b) => b.textContent?.startsWith('Continue'))!;
  expect(button.disabled).toBe(true);
  expect(host.querySelectorAll('.title-actions .primary')).toHaveLength(1);
  expect(host.querySelector('.title-actions .primary')?.textContent).toMatch(/^Single Player/);
  expect(button.textContent).toContain('Checking autosaves…');
  await act(async () => { finish(JSON.stringify(save)); });
  expect(button.disabled).toBe(false);
  expect(host.querySelectorAll('.title-actions .primary')).toHaveLength(1);
  expect(button.classList.contains('primary')).toBe(true);
  expect(button.textContent).toContain('Ada & Bo');
  await act(async () => button.click());
  expect(useGame.getState().screen).toBe('game');
  expect(useGame.getState().client?.save()).toBe(save.game);
});

it('keeps Resume as the sole primary when a game is already open', async () => {
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('42', 'India', 'Byzantium', 'Ada', 'Bo'));
  const views = [client.view(0), client.view(1)];
  vi.mocked(readNativeAutosave).mockResolvedValue(JSON.stringify(saveFile(client, views)));
  useGame.setState({ client, views });
  await act(async () => root.render(<TitleScreen />));
  expect(host.querySelectorAll('.title-actions .primary')).toHaveLength(1);
  expect(host.querySelector('.title-actions .primary')?.textContent).toMatch(/^Resume/);
});

it('shows the exact older-rules message in the save list before loading', async () => {
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('42', 'India', 'Byzantium', 'Ada', 'Bo'));
  const save = saveFile(client, [client.view(0), client.view(1)]);
  save.game = save.game.replace(/"save_version":\d+/, '"save_version":5');
  save.playtest = 'Playtest 8.9.19';
  vi.mocked(readNativeAutosave).mockResolvedValue(JSON.stringify(save));
  await act(async () => root.render(<TitleScreen />));
  const saved = [...host.querySelectorAll('button')].find((b) => b.textContent?.startsWith('Saved game (older version)'))!;
  expect(saved.disabled).toBe(true);
  // An unrelated late store error (a status probe that finds no server) must not stand in for the notice under test.
  // The probe can fail later than one tick on a loaded machine, so keep clearing until the notice settles.
  for (let i = 0; i < 40 && !host.querySelector('.title-notice')?.textContent?.startsWith('This game was saved'); i++) {
    await act(async () => { useGame.setState({ error: null }); await new Promise((r) => setTimeout(r, 50)); });
  }
  expect(host.querySelector('.title-notice')?.textContent).toBe("This game was saved by Playtest 8.9.19 with older rules and can't continue. Starting a new game replaces the autosave.");
  expect(host.querySelector('.title-actions .primary')?.textContent).toMatch(/^Single Player/);
});
