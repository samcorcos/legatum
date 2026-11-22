// @vitest-environment happy-dom
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { TitleScreen } from './TitleScreen';
import { useGame } from '../store';
import { setOnlineApi, type OnlineApi } from '../platform/online';

vi.mock('./TitleArt', () => ({ TitleArt: () => null }));

const info = { relay: 'https://relay.example', build: '8.9.74', rules_version: 12, protocol: 1 };
let root: Root;
let host: HTMLDivElement;
let assign: ReturnType<typeof vi.fn>;
const flush = async () => { await act(async () => { await Promise.resolve(); await Promise.resolve(); }); };
const button = (label: RegExp) => [...host.querySelectorAll('button')].find((b) => label.test(b.textContent ?? ''))!;

beforeEach(async () => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  localStorage.clear();
  assign = vi.fn();
  vi.stubGlobal('location', { ...location, assign, search: '', hostname: 'tauri.localhost' });
  useGame.setState(useGame.getInitialState(), true);
  useGame.setState({ online: info });
  host = document.body.appendChild(document.createElement('div'));
  root = createRoot(host);
  act(() => root.render(<TitleScreen />));
  await flush();
  await act(async () => button(/^Multiplayer/).click());
});
afterEach(() => {
  act(() => root.unmount());
  host.remove();
  setOnlineApi();
  vi.unstubAllGlobals();
});

const api = (hostStart: OnlineApi['hostStart']): OnlineApi => ({ info: async () => info, hostStart, hostStop: async () => {} });

it('Host Game starts the game with the app and opens its window', async () => {
  const start = vi.fn(async () => ({ url: 'http://127.0.0.1:5555/', room: 'r1' }));
  setOnlineApi(api(start));
  await act(async () => button(/^Host Game/).click());
  await flush();
  expect(start).toHaveBeenCalledWith('A Legatum player');
  expect(localStorage.getItem('legatum.playerName')).toBe('A Legatum player');
  expect(assign).toHaveBeenCalledWith('http://127.0.0.1:5555/?host=1');
});

it('shows a clear error with a retry when starting fails, then succeeds', async () => {
  const start = vi.fn()
    .mockRejectedValueOnce(new Error('boom'))
    .mockResolvedValueOnce({ url: 'http://127.0.0.1:5555/', room: 'r1' });
  setOnlineApi(api(start));
  await act(async () => button(/^Host Game/).click());
  await flush();
  expect(assign).not.toHaveBeenCalled();
  expect(host.querySelector('.title-notice')?.textContent).toBe('Could not open your game. Check your internet connection and try again.');
  expect(button(/^Host Game/).textContent).toContain('Try again');
  expect(button(/^Host Game/).classList.contains('primary')).toBe(true);
  expect(button(/^Join Game/).classList.contains('primary')).toBe(false);
  await act(async () => button(/^Host Game/).click());
  await flush();
  expect(start).toHaveBeenCalledTimes(2);
  expect(assign).toHaveBeenCalledWith('http://127.0.0.1:5555/?host=1');
});

it('the name field is prefilled, can be changed, and is saved and sent', async () => {
  localStorage.setItem('legatum.playerName', 'Ada');
  act(() => root.unmount());
  root = createRoot(host);
  act(() => root.render(<TitleScreen />));
  await flush();
  await act(async () => button(/^Multiplayer/).click());
  const input = host.querySelector<HTMLInputElement>('.host-name input')!;
  expect(input.value).toBe('Ada');
  const start = vi.fn(async () => ({ url: 'http://127.0.0.1:5555/', room: null }));
  setOnlineApi(api(start));
  const set = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value')!.set!;
  await act(async () => { set.call(input, 'Bo'); input.dispatchEvent(new Event('input', { bubbles: true })); });
  await act(async () => button(/^Host Game/).click());
  await flush();
  expect(start).toHaveBeenCalledWith('Bo');
  expect(localStorage.getItem('legatum.playerName')).toBe('Bo');
});

it('Join Game opens the lobby list', async () => {
  await act(async () => button(/^Join Game/).click());
  expect(useGame.getState().screen).toBe('online');
});

it('a build with no relay offers no online hosting and no primary button on the disabled Join Game', async () => {
  act(() => useGame.setState({ online: { ...info, relay: null } }));
  expect(host.querySelector('.host-name')).toBeNull();
  expect(button(/^Host Game/).disabled).toBe(true);
  expect(button(/^Host Game/).textContent).not.toContain('others can find and join');
  expect(button(/^Join Game/).disabled).toBe(true);
  expect(button(/^Join Game/).classList.contains('primary')).toBe(false);
});

it('with a game open, Resume stays the only primary button when hosting fails', async () => {
  act(() => useGame.setState({ views: [{ turn: 5 }] as never }));
  setOnlineApi(api(vi.fn().mockRejectedValue(new Error('boom'))));
  await act(async () => button(/^Host Game/).click());
  await flush();
  expect(button(/^Host Game/).textContent).toContain('Try again');
  expect([...host.querySelectorAll('button.primary')].map((b) => b.textContent)).toEqual([expect.stringMatching(/^Resume/)]);
});

it('no disabled button is primary while a game is opening', async () => {
  setOnlineApi(api(() => new Promise(() => {})));
  await act(async () => button(/^Host Game/).click());
  expect(host.querySelectorAll('button.primary:disabled').length).toBe(0);
});
