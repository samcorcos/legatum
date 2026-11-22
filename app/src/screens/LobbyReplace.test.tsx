// @vitest-environment happy-dom
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { IDBFactory } from 'fake-indexeddb';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { useGame } from '../store';
import { LobbyScreen } from './LobbyScreen';
import { resetAutosaveStoreForTests, writeAutosave } from '../platform/autosaveStore';

vi.mock('./HexBackdrop', () => ({ HexBackdrop: () => null }));

let host: HTMLDivElement;
let root: Root;
const show = async (el: import('react').ReactElement) => { await act(async () => root.render(el)); await act(async () => { await new Promise((r) => setTimeout(r, 100)); }); };
const press = async (b: HTMLElement) => { await act(async () => b.click()); await act(async () => { await new Promise((r) => setTimeout(r, 300)); }); };
const startGame = vi.fn(async () => {});

beforeEach(() => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  vi.stubGlobal('fetch', vi.fn(async () => new Response('{"message":"offline in tests"}', { status: 503, headers: { 'content-type': 'application/json' } })));
  vi.stubGlobal('indexedDB', new IDBFactory());
  resetAutosaveStoreForTests();
  startGame.mockClear();
  useGame.setState(useGame.getInitialState(), true);
  useGame.setState({ screen: 'setup', setupMode: 'solo', startGame });
  host = document.body.appendChild(document.createElement('div'));
  root = createRoot(host);
  // happy-dom has no modal dialog: open the element the way showModal would.
  HTMLDialogElement.prototype.showModal = function showModal(this: HTMLDialogElement) { this.setAttribute('open', ''); };
});
afterEach(() => {
  act(() => root.unmount());
  host.remove();
  vi.unstubAllGlobals();
});

const saveSomething = () => writeAutosave({ campaign: 'old', turn: 42, savedAt: '2026-11-22T10:00:00Z', label: 'Ann & Bo', faction: 'Athens', text: '{}' });
const startButton = () => host.querySelector<HTMLButtonElement>('.lobby-go')!;
const dialog = () => host.querySelector<HTMLDialogElement>('[data-testid="replace-game"]');
const dialogButton = (name: string) => [...dialog()!.querySelectorAll('button')].find((b) => b.textContent === name)!;

it('starts at once when no game is saved', async () => {
  await show(<LobbyScreen />);
  await press(startButton());
  expect(dialog()).toBeNull();
  expect(startGame).toHaveBeenCalledTimes(1);
});

it('asks once before a new game replaces a saved one, with one primary action', async () => {
  await saveSomething();
  await show(<LobbyScreen />);
  await press(startButton());
  expect(startGame).not.toHaveBeenCalled();
  expect(dialog()).not.toBeNull();
  expect(dialog()!.querySelector('h2')!.textContent).toBe('Replace your saved game?');
  expect(dialog()!.textContent).toContain('Athens, turn 42. Starting a new game replaces it.');
  expect(dialog()!.textContent).toContain('To keep it, choose Go back, then Continue, then Menu → Save to file.');
  expect(dialog()!.querySelectorAll('.btn.primary')).toHaveLength(1);
  expect(dialogButton('Start new game').classList.contains('primary')).toBe(true);
  expect(document.activeElement).toBe(dialogButton('Go back'));

  await press(dialogButton('Go back'));
  expect(dialog()).toBeNull();
  expect(startGame).not.toHaveBeenCalled();

  await press(startButton());
  await press(dialogButton('Start new game'));
  expect(startGame).toHaveBeenCalledTimes(1);
  expect(dialog()).toBeNull();
});

it('Escape keeps the game', async () => {
  await saveSomething();
  await show(<LobbyScreen />);
  await press(startButton());
  await act(async () => { dialog()!.dispatchEvent(new Event('cancel', { cancelable: true })); });
  expect(dialog()).toBeNull();
  expect(startGame).not.toHaveBeenCalled();
});
