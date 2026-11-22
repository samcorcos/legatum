// @vitest-environment happy-dom
import { act, StrictMode } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { TitleScreen } from './TitleScreen';
import { AUTOSAVE_KEY, saveFile, useGame, type SaveFile } from '../store';
import { FixtureClient } from '../test/fixtureEngine';
import { IDBFactory } from 'fake-indexeddb';
import { RULES } from '../data';
import { loadFromFile } from '../platform/files';
import { resetAutosaveStoreForTests, writeAutosave } from '../platform/autosaveStore';

vi.mock('./TitleArt', () => ({ TitleArt: () => null }));
vi.mock('../platform/files', async (orig) => ({ ...(await orig<typeof import('../platform/files')>()), loadFromFile: vi.fn() }));
let root: Root;
const wait = (ms: number) => act(async () => { await new Promise((r) => setTimeout(r, ms)); });
const show = async (el: import('react').ReactElement) => { await act(async () => root.render(el)); await act(async () => { await new Promise((r) => setTimeout(r, 300)); }); };
const press = async (b: HTMLElement) => { await act(async () => b.click()); await wait(800); };
let host: HTMLDivElement;
beforeEach(() => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  localStorage.clear();
  resetAutosaveStoreForTests();
  vi.stubGlobal('indexedDB', new IDBFactory());
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

it('offers the autosave as the primary Continue and loads that exact save', async () => {
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('42', 'India', 'Byzantium', 'Ada', 'Bo'));
  const save = saveFile(client, [client.view(0), client.view(1)]);
  localStorage.setItem(AUTOSAVE_KEY, JSON.stringify(save));
  await show(<StrictMode><TitleScreen /></StrictMode>);
  const button = [...host.querySelectorAll('button')].find((b) => b.textContent?.startsWith('Continue'))!;
  expect(button.disabled).toBe(false);
  expect(host.querySelectorAll('.title-actions .primary')).toHaveLength(1);
  expect(button.classList.contains('primary')).toBe(true);
  expect(button.textContent).toContain('Ada & Bo');
  await press(button);
  expect(useGame.getState().screen).toBe('game');
  expect(useGame.getState().client?.save()).toBe(save.game);
});

it('keeps Single Player primary when there is no autosave', async () => {
  await show(<TitleScreen />);
  const button = [...host.querySelectorAll('button')].find((b) => b.textContent?.startsWith('Continue'))!;
  expect(button.disabled).toBe(true);
  expect(button.textContent).toContain('No autosave yet');
  expect(host.querySelector('.title-actions .primary')?.textContent).toMatch(/^Single Player/);
});

it('keeps Resume as the sole primary when a game is already open', async () => {
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('42', 'India', 'Byzantium', 'Ada', 'Bo'));
  const views = [client.view(0), client.view(1)];
  localStorage.setItem(AUTOSAVE_KEY, JSON.stringify(saveFile(client, views)));
  useGame.setState({ client, views });
  await show(<TitleScreen />);
  expect(host.querySelectorAll('.title-actions .primary')).toHaveLength(1);
  expect(host.querySelector('.title-actions .primary')?.textContent).toMatch(/^Resume/);
});

it('shows the exact older-rules message in the save list before loading', async () => {
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('42', 'India', 'Byzantium', 'Ada', 'Bo'));
  const save = saveFile(client, [client.view(0), client.view(1)]);
  save.game = save.game.replace(/"save_version":\d+/, '"save_version":5');
  save.playtest = 'Playtest 8.9.19';
  localStorage.setItem(AUTOSAVE_KEY, JSON.stringify(save));
  await show(<TitleScreen />);
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

// Each test is its own game: the title screen remembers (per slot id) which saves would not open.
let game = '';
beforeEach(() => { game = `game-${Math.random()}`; });
const slot = (save: SaveFile, turn: number, text = JSON.stringify({ ...save, turn })) => writeAutosave({ campaign: game, turn, savedAt: new Date(Date.UTC(2026, 10, 22, 10, turn)).toISOString(), label: 'Ada & Bo', text });
const aSave = async () => {
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('42', 'India', 'Byzantium', 'Ada', 'Bo'));
  return saveFile(client, [client.view(0), client.view(1)]);
};
const continueButton = () => [...host.querySelectorAll('button')].find((b) => b.textContent?.startsWith('Continue') || b.textContent?.startsWith('Saved game'))!;
const refusing = (save: SaveFile) => JSON.stringify({ ...save, game: JSON.stringify({ save_version: RULES.saveVersion, nothing: true }) });

it('lists the newest autosave from the browser database, not only the old localStorage slot', async () => {
  const save = await aSave();
  await slot(save, 1);
  await slot(save, 2);
  await show(<TitleScreen />);
  expect(continueButton().textContent).toContain('turn 2');
  expect(continueButton().disabled).toBe(false);
});

it('offers the turn before when the newest save will not open, instead of a dead end', async () => {
  const save = await aSave();
  await slot(save, 1);
  await slot(save, 2, refusing({ ...save, turn: 2 }));
  await show(<TitleScreen />);
  expect(continueButton().textContent).toContain('turn 2');
  await press(continueButton());
  expect(useGame.getState().screen).not.toBe('game');
  expect(continueButton().textContent).toContain('turn 1');
  expect(continueButton().disabled).toBe(false);
  expect(continueButton().classList.contains('primary')).toBe(true);
  expect(host.querySelector('.title-notice')?.textContent).toBe('Your latest save (turn 2) could not be opened. Continue opens turn 1 instead.');
  await press(continueButton());
  expect(useGame.getState().screen).toBe('game');
  expect(useGame.getState().client?.save()).toBe(save.game);
});

it('skips a newest save that is damaged text, and says so', async () => {
  const save = await aSave();
  await slot(save, 1);
  await slot(save, 2, '{"format":"legatum-save"');
  await show(<TitleScreen />);
  expect(continueButton().textContent).toContain('turn 1');
  expect(host.querySelector('.title-notice')?.textContent).toBe('Your latest save (turn 2) could not be opened. Continue opens turn 1 instead.');
});

it('with no earlier turn to fall back on, Continue is retitled and disabled and a new game is primary', async () => {
  const save = await aSave();
  await slot(save, 2, refusing({ ...save, turn: 2 }));
  await show(<TitleScreen />);
  await press(continueButton());
  expect(continueButton().textContent).toContain('Saved game (unreadable)');
  expect(continueButton().disabled).toBe(true);
  expect(host.querySelector('.title-actions .primary')?.textContent).toMatch(/^Single Player/);
});

it('Load Game asks before a chosen file replaces a saved game, and loads at once when none is saved', async () => {
  const save = await aSave();
  vi.mocked(loadFromFile).mockResolvedValue(JSON.stringify(save));
  const loadButton = () => [...host.querySelectorAll('button')].find((b) => b.textContent?.startsWith('Load Game'))!;
  const dialog = () => host.querySelector('[data-testid="replace-game"]');
  HTMLDialogElement.prototype.showModal = function showModal(this: HTMLDialogElement) { this.setAttribute('open', ''); };
  // Nothing saved: no question.
  await show(<TitleScreen />);
  await press(loadButton());
  expect(dialog()).toBeNull();
  expect(useGame.getState().screen).toBe('game');
  // A saved game: the question comes first and Go back changes nothing.
  act(() => root.unmount());
  useGame.setState(useGame.getInitialState(), true);
  root = createRoot(host);
  await slot(save, 3);
  await show(<TitleScreen />);
  await press(loadButton());
  expect(dialog()).not.toBeNull();
  expect(dialog()!.textContent).toContain('Loading this file replaces it');
  expect(useGame.getState().screen).not.toBe('game');
  const buttons = [...dialog()!.querySelectorAll('button')];
  await press(buttons.find((b) => b.textContent === 'Go back')!);
  expect(dialog()).toBeNull();
  expect(useGame.getState().screen).not.toBe('game');
  await press(loadButton());
  await press([...dialog()!.querySelectorAll('button')].find((b) => b.textContent === 'Load the file')!);
  expect(useGame.getState().screen).toBe('game');
});
