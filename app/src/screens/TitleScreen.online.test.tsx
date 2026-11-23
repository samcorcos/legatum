// @vitest-environment happy-dom
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { TitleScreen } from './TitleScreen';
import { useGame } from '../store';
import { RelayRefusal } from '../net/browserHost';

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
  expect(start).toHaveBeenCalledWith('A Legatum player', undefined);
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
  expect(start).toHaveBeenCalledWith('Bo', undefined);
  expect(localStorage.getItem('legatum.playerName')).toBe('Bo');
});

it('Join Game opens the lobby list', async () => {
  await act(async () => must(/^Join Game/).click());
  expect(useGame.getState().screen).toBe('online');
});

it('a build with no relay disables multiplayer and explains why', async () => {
  await act(async () => must(/Back/).click());
  act(() => useGame.setState({ online: null }));
  expect(host.querySelector('.host-name')).toBeNull();
  expect(button(/^Host Game/)).toBeUndefined();
  expect(button(/^Join Game/)).toBeUndefined();
  expect(must(/^Multiplayer/).disabled).toBe(true);
  expect(must(/^Multiplayer/).textContent).toContain('Online play is unavailable');
  expect(must(/^Single Player/).classList.contains('primary')).toBe(true);
});

it('with a game open, Resume stays the only primary button when hosting fails', async () => {
  act(() => useGame.setState({ views: [{ turn: 5, players: [{ id: 0, is_human: true, faction: 'Rome' }] }] as never }));
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

it.each([
  ['too_fast', 300, 'Try again in 5 minutes', false],
  ['too_fast', null, 'Try again in a few minutes', false],
  ['too_many', null, 'Close one of your games first', false],
  ['closed', null, 'Paused for now', false],
  ['full', null, 'Try again', true],
  ['offline', null, 'Try again', true],
  ['other', null, 'Try again', true],
] as const)('after a %s refusal (wait %s) Host Game says "%s" and is primary: %s', async (kind, wait, line, retryable) => {
  hosting(async () => {
    const refusal = new RelayRefusal(kind, 'x', wait);
    useGame.setState({ hostRefusal: refusal });
    return refusal.message;
  });
  await act(async () => must(/^Host Game/).click());
  await flush();
  expect(must(/^Host Game/).textContent).toContain(line);
  expect(must(/^Host Game/).classList.contains('primary')).toBe(retryable);
  expect(must(/^Join Game/).classList.contains('primary')).toBe(!retryable);
  expect(host.querySelectorAll('button.primary:disabled').length).toBe(0);
});

it('the name note replaces the label, so it adds no height', async () => {
  const input = host.querySelector<HTMLInputElement>('.host-name input')!;
  const type = async (v: string) => {
    await act(async () => {
      Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value')!.set!.call(input, v);
      input.dispatchEvent(new Event('input', { bubbles: true }));
    });
  };
  const label = () => host.querySelector('.host-name:not(.host-code) > span')!;
  expect(label().textContent).toBe('Your name in the game list');
  await type('Ada \u{1F600}');
  expect(label().textContent).toBe('Shown in the game list as “Ada”.');
  await type('\u{1F600}\u{1F600}');
  expect(label().textContent).toBe('Listed as “Game” and four characters.');
  expect(host.querySelectorAll('.host-name:not(.host-code) > span').length).toBe(1);
});
