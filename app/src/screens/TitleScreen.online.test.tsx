// @vitest-environment happy-dom
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { TitleScreen } from './TitleScreen';
import { useGame } from '../store';

vi.mock('./TitleArt', () => ({ TitleArt: () => null }));

const info = { relay: 'https://relay.example', build: '8.9.74', rules_version: 12, protocol: 1 };
let root: Root;
let host: HTMLDivElement;
const flush = async () => { await act(async () => { await Promise.resolve(); await Promise.resolve(); }); };
const button = (label: RegExp) => [...host.querySelectorAll('button')].find((b) => label.test(b.textContent ?? ''));
const must = (label: RegExp) => button(label)!;
/** Host Game calls the store's hostOnline, which runs the game in this tab. */
const hosting = (impl: (name: string) => Promise<string | null>) => {
  const fn = vi.fn(impl);
  useGame.setState({ hostOnline: fn });
  return fn;
};

beforeEach(async () => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  localStorage.clear();
  useGame.setState(useGame.getInitialState(), true);
  useGame.setState({ online: info });
  host = document.body.appendChild(document.createElement('div'));
  root = createRoot(host);
  act(() => root.render(<TitleScreen />));
  await flush();
  await act(async () => must(/^Multiplayer/).click());
});
afterEach(() => {
  act(() => root.unmount());
  host.remove();
  vi.unstubAllGlobals();
});

it('Host Game opens the game in this tab with the player name', async () => {
  const start = hosting(async () => null);
  await act(async () => must(/^Host Game/).click());
  await flush();
  expect(start).toHaveBeenCalledWith('A Legatum player');
  expect(localStorage.getItem('legatum.playerName')).toBe('A Legatum player');
  expect(host.querySelector('.title-notice')).toBeNull();
});

it('shows a clear error with a retry when starting fails, then succeeds', async () => {
  const start = hosting(vi.fn<(n: string) => Promise<string | null>>()
    .mockResolvedValueOnce('Could not open your game. Check your internet connection and try again.')
    .mockResolvedValueOnce(null));
  await act(async () => must(/^Host Game/).click());
  await flush();
  expect(host.querySelector('.title-notice')?.textContent).toBe('Could not open your game. Check your internet connection and try again.');
  expect(must(/^Host Game/).textContent).toContain('Try again');
  expect(must(/^Host Game/).classList.contains('primary')).toBe(true);
  expect(must(/^Join Game/).classList.contains('primary')).toBe(false);
  await act(async () => must(/^Host Game/).click());
  await flush();
  expect(start).toHaveBeenCalledTimes(2);
  expect(host.querySelector('.title-notice')).toBeNull();
});

it('the name field is prefilled, can be changed, and is saved and sent', async () => {
  localStorage.setItem('legatum.playerName', 'Ada');
  act(() => root.unmount());
  root = createRoot(host);
  act(() => root.render(<TitleScreen />));
  await flush();
  await act(async () => must(/^Multiplayer/).click());
  const input = host.querySelector<HTMLInputElement>('.host-name input')!;
  expect(input.value).toBe('Ada');
  const start = hosting(async () => null);
  const set = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value')!.set!;
  await act(async () => { set.call(input, 'Bo'); input.dispatchEvent(new Event('input', { bubbles: true })); });
  await act(async () => must(/^Host Game/).click());
  await flush();
  expect(start).toHaveBeenCalledWith('Bo');
  expect(localStorage.getItem('legatum.playerName')).toBe('Bo');
});

it('Join Game opens the lobby list', async () => {
  await act(async () => must(/^Join Game/).click());
  expect(useGame.getState().screen).toBe('online');
});

it('a build with no relay offers hot seat only, and says so', async () => {
  act(() => useGame.setState({ online: null }));
  expect(host.querySelector('.host-name')).toBeNull();
  expect(button(/^Host Game/)).toBeUndefined();
  expect(button(/^Join Game/)).toBeUndefined();
  expect(must(/^Hot Seat/)).toBeTruthy();
  expect(host.textContent).toContain('This version has no online play.');
});

it('with a game open, Resume stays the only primary button when hosting fails', async () => {
  act(() => useGame.setState({ views: [{ turn: 5 }] as never }));
  hosting(async () => 'Could not open your game. Try again.');
  await act(async () => must(/^Host Game/).click());
  await flush();
  expect(must(/^Host Game/).textContent).toContain('Try again');
  expect([...host.querySelectorAll('button.primary')].map((b) => b.textContent)).toEqual([expect.stringMatching(/^Resume/)]);
});

it('no disabled button is primary while a game is opening', async () => {
  hosting(() => new Promise(() => {}));
  await act(async () => must(/^Host Game/).click());
  expect(must(/^Host Game/).textContent).toContain('Opening your game');
  expect(host.querySelectorAll('button.primary:disabled').length).toBe(0);
});
