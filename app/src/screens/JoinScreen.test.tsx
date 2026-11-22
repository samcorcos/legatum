// @vitest-environment happy-dom
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { JoinScreen } from './JoinScreen';
import { useGame } from '../store';
import { FakeSocket } from '../test/fakeSocket';
import type { PlayerView } from '../types';

vi.mock('./HexBackdrop', () => ({ HexBackdrop: () => null }));

class Socket extends FakeSocket {
  static all: Socket[] = [];
  constructor(url: string) { super(url); Socket.all.push(this); }
}

let root: Root;
let host: HTMLDivElement;
beforeEach(() => {
  vi.useFakeTimers();
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  vi.stubGlobal('WebSocket', Socket);
  // JoinScreen polls status; leave the installed status intact during this regression.
  vi.stubGlobal('fetch', vi.fn(async () => ({ ok: false })));
  Socket.all = [];
  sessionStorage.clear();
  document.body.innerHTML = '';
  host = document.body.appendChild(document.createElement('div'));
  root = createRoot(host);
  useGame.setState(useGame.getInitialState(), true);
  const seats = [{ seat: 0, player: 0, name: 'Ada', faction: 'India' as const, connected: false, reserved: false, ready: false, orders: 0 }];
  useGame.setState({ screen: 'join', net: {
    status: { mode: 'server', has_game: true, turn: 1, seats, is_local: false },
    mySeat: null, seats, conn: 'closed',
  } });
  act(() => root.render(<JoinScreen />));
});
afterEach(() => {
  act(() => { root.unmount(); useGame.getState().leaveOnline(); });
  vi.useRealTimers();
  vi.unstubAllGlobals();
});

it.each(['connecting', 'welcomed'] as const)('Back cancels a %s join, clears busy, and ignores late responses', (stage) => {
  act(() => host.querySelector<HTMLButtonElement>('.join-seat')!.click());
  const socket = Socket.all[0];
  if (stage === 'welcomed') act(() => {
    socket.open();
    socket.push({ type: 'welcome', seat: 0, game_id: 'old-game', orders: [] });
  });
  expect(useGame.getState().busy).toBe('Joining…');
  act(() => host.querySelector<HTMLButtonElement>('.setup-top button')!.click());
  expect(socket.readyState).toBe(3);
  expect(useGame.getState()).toMatchObject({ screen: 'title', busy: null, client: null, views: null, net: { conn: 'closed', mySeat: null } });

  const lateView = { me: 0, turn: 1, units: [], cities: [], finished: true } as unknown as PlayerView;
  act(() => {
    socket.open();
    socket.push({ type: 'welcome', seat: 0, game_id: 'old-game', orders: [] });
    socket.push({ type: 'view', view: lateView });
    socket.push({ type: 'error', message: 'Late join failure' });
    vi.advanceTimersByTime(10_000);
  });
  expect(useGame.getState()).toMatchObject({ screen: 'title', busy: null, error: null, client: null, views: null });
  expect(Socket.all).toHaveLength(1);

  // A new attempt is independent of any final response from the cancelled socket.
  act(() => { useGame.getState().goto('join'); useGame.getState().joinSeat(0, 'Ada'); });
  act(() => socket.push({ type: 'view', view: lateView }));
  expect(useGame.getState()).toMatchObject({ screen: 'join', busy: 'Joining…', client: null });
  const next = Socket.all[1];
  act(() => {
    next.open();
    next.push({ type: 'welcome', seat: 0, game_id: 'new-game', orders: [] });
    next.push({ type: 'view', view: lateView });
  });
  expect(useGame.getState()).toMatchObject({ screen: 'game', busy: null, client: { kind: 'remote' } });
});

it('shows a connection notice right under the lede, above the seats, as an alert', () => {
  act(() => useGame.setState({ error: 'Too many people on your internet connection have joined this game. Close any extra windows, then pick your seat again.' }));
  const notice = host.querySelector('.toast')!;
  expect(notice.getAttribute('role')).toBe('alert');
  const seats = host.querySelector('.join-seats')!;
  expect(notice.compareDocumentPosition(seats) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
  expect(host.querySelector('.setup-lede')!.nextElementSibling).toBe(notice);
});
