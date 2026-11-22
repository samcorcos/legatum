// @vitest-environment happy-dom
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { OnlineLobbyScreen } from './OnlineLobbyScreen';
import { useGame } from '../store';
import { setServerBase, serverBase } from '../remote';

vi.mock('./HexBackdrop', () => ({ HexBackdrop: () => null }));

const RELAY = 'https://relay.example';
const mine = { relay: RELAY, build: '8.9.74', rules_version: 12, protocol: 1 };
const room = (over: Record<string, unknown> = {}) => ({
  room: 'abcdefghij', name: 'Ada', build: '8.9.74', rules_version: 12, protocol: 1,
  players: 2, humans: 2, open_seats: 1, phase: 'staging', turn: 0, updated_ms: 1, ...over,
});

let root: Root;
let host: HTMLDivElement;
let lobby: () => Promise<{ ok: boolean; json(): Promise<unknown> }>;
const calls: string[] = [];

const flush = async () => { await act(async () => { await Promise.resolve(); await Promise.resolve(); }); };

beforeEach(() => {
  vi.useFakeTimers();
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  calls.length = 0;
  lobby = async () => ({ ok: true, json: async () => ({ rooms: [] }) });
  vi.stubGlobal('fetch', vi.fn(async (url: string) => {
    calls.push(url);
    if (url.startsWith(`${RELAY}/lobby`)) return lobby();
    return { ok: false, status: 404, text: async () => '' };
  }));
  setServerBase('');
  useGame.setState(useGame.getInitialState(), true);
  useGame.setState({ screen: 'online', online: mine });
  host = document.body.appendChild(document.createElement('div'));
  root = createRoot(host);
});
afterEach(() => {
  act(() => root.unmount());
  host.remove();
  setServerBase('');
  vi.useRealTimers();
  vi.unstubAllGlobals();
});

const mount = async () => { act(() => root.render(<OnlineLobbyScreen />)); await flush(); };
const lobbyCalls = () => calls.filter((c) => c === `${RELAY}/lobby`).length;

it('lists rooms with seats and phase, and one Join per row', async () => {
  lobby = async () => ({ ok: true, json: async () => ({ rooms: [
    room(),
    room({ room: 'klmnopqrst', name: 'Bo', phase: 'playing', turn: 14, players: 3, open_seats: 0 }),
  ] }) });
  await mount();
  const rows = [...host.querySelectorAll('.online-room')];
  expect(rows).toHaveLength(2);
  expect(rows[0].textContent).toContain('Ada');
  expect(rows[0].textContent).toContain('2 players · 1 open seat');
  expect(rows[1].textContent).toContain('3 players · Full');
  expect(rows[0].textContent).toContain('Waiting in lobby');
  expect(rows[1].textContent).toContain('Turn 14');
  expect(host.querySelectorAll('.online-room button')).toHaveLength(2);
  expect(rows[0].querySelector('button')!.disabled).toBe(false);
  expect(rows[1].querySelector('button')!.disabled).toBe(true); // no open seat
});

it('marks a locked game with the word Locked, and passes that on when joining', async () => {
  lobby = async () => ({ ok: true, json: async () => ({ rooms: [room({ locked: true }), room({ room: 'klmnopqrst', name: 'Bo' })] }) });
  await mount();
  const rows = [...host.querySelectorAll('.online-room')];
  expect(rows[0].textContent).toContain('Locked');
  expect(rows[0].querySelector('.online-room-lock svg')!.getAttribute('aria-hidden')).toBe('true');
  expect(rows[1].textContent).not.toContain('Locked');
  const join = vi.spyOn(useGame.getState(), 'joinOnline').mockResolvedValue(null);
  act(() => useGame.setState({ joinOnline: join }));
  const fresh = [...host.querySelectorAll('.online-room')];
  act(() => fresh[0].querySelector('button')!.click());
  await flush();
  expect(join).toHaveBeenCalledWith(RELAY, 'abcdefghij', true);
  act(() => fresh[1].querySelector('button')!.click());
  expect(join).toHaveBeenLastCalledWith(RELAY, 'klmnopqrst', false);
});

it('blocks a game from a different version with the exact words', async () => {
  lobby = async () => ({ ok: true, json: async () => ({ rooms: [room({ build: '8.9.73' })] }) });
  await mount();
  const row = host.querySelector('.online-room')!;
  expect(row.textContent).toContain('This game is on version 8.9.73 and you have 8.9.74. Update the older one to play together.');
  expect(row.querySelector('button')!.disabled).toBe(true);
  const join = vi.spyOn(useGame.getState(), 'joinOnline');
  act(() => row.querySelector('button')!.click());
  expect(join).not.toHaveBeenCalled();
});

it.each([
  ['rules', { rules_version: 11 }],
  ['link', { protocol: 2 }],
  ['unknown build', { build: '' }],
])('blocks a different %s version', async (_n, over) => {
  lobby = async () => ({ ok: true, json: async () => ({ rooms: [room(over)] }) });
  await mount();
  expect(host.querySelector('.online-room')!.textContent).toContain('This game is on a different version from yours. Update both to the newest to play together.');
});

it('shows the empty state', async () => {
  await mount();
  expect(host.textContent).toContain('No games are being hosted right now.');
  expect(host.querySelector('.online-room')).toBeNull();
});

it('shows a plain error, keeps retrying by itself, and recovers', async () => {
  lobby = async () => { throw new Error('offline'); };
  await mount();
  expect(host.querySelector('[data-state="error"]')?.textContent).toContain('The list of games could not be loaded.');
  expect(host.textContent).toContain('Trying again');
  lobby = async () => ({ ok: true, json: async () => ({ rooms: [room()] }) });
  await act(async () => { vi.advanceTimersByTime(3000); });
  await flush();
  expect(host.querySelector('[data-state="error"]')).toBeNull();
  expect(host.querySelectorAll('.online-room')).toHaveLength(1);
});

it('polls every 3 seconds while open and stops when closed', async () => {
  await mount();
  expect(lobbyCalls()).toBe(1);
  await act(async () => { vi.advanceTimersByTime(3000); });
  expect(lobbyCalls()).toBe(2);
  await act(async () => { vi.advanceTimersByTime(6000); });
  expect(lobbyCalls()).toBe(4);
  act(() => root.unmount());
  await act(async () => { vi.advanceTimersByTime(12000); });
  expect(lobbyCalls()).toBe(4);
  root = createRoot(host);
});

it('Join points the app at that room and continues as a guest', async () => {
  const status = { mode: 'server', has_game: false, turn: 0, seats: [], is_local: false, staging: null };
  vi.stubGlobal('fetch', vi.fn(async (url: string) => {
    calls.push(url);
    if (url.startsWith(`${RELAY}/lobby`)) return { ok: true, json: async () => ({ rooms: [room()] }) };
    if (url === `${RELAY}/rooms/abcdefghij/api/status`) return { ok: true, status: 200, text: async () => JSON.stringify(status) };
    return { ok: false, status: 404, text: async () => '' };
  }));
  await mount();
  await act(async () => host.querySelector<HTMLButtonElement>('.online-room button')!.click());
  await flush();
  expect(serverBase()).toBe(`${RELAY}/rooms/abcdefghij`);
  expect(useGame.getState().screen).toBe('join');
  expect(useGame.getState().net?.status.is_local).toBe(false);
});

it('Join on a room that has gone says so and keeps the list', async () => {
  lobby = async () => ({ ok: true, json: async () => ({ rooms: [room()] }) });
  await mount();
  await act(async () => host.querySelector<HTMLButtonElement>('.online-room button')!.click());
  await flush();
  expect(serverBase()).toBe('');
  expect(useGame.getState().screen).toBe('online');
  // The failure shows inside the row that failed.
  expect(host.querySelector('.online-room .online-room-note')?.textContent).toContain('This game has ended.');
  expect(host.querySelector('.toast')).toBeNull();
});

it('Back returns to the title and leaves no room set', async () => {
  await mount();
  await act(async () => host.querySelector<HTMLButtonElement>('.setup-top button')!.click());
  expect(useGame.getState().screen).toBe('title');
});

it('Esc goes back to the title', async () => {
  await mount();
  await act(async () => { window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' })); });
  expect(useGame.getState().screen).toBe('title');
});

it('a game with no name is listed as Unnamed game', async () => {
  lobby = async () => ({ ok: true, json: async () => ({ rooms: [room({ name: '' })] }) });
  await mount();
  expect(host.querySelector('.online-room-name')?.textContent).toBe('Unnamed game');
});
