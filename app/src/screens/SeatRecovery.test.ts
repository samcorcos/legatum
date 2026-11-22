// @vitest-environment happy-dom
import { act, createElement } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { useGame } from '../store';
import { rememberSeat, rememberedSeat, type ServerStatus } from '../remote';
import { configFromLobby, type StagingState } from '../staging';
import { newLobby } from '../lobby';
import type { PlayerView } from '../types';
import { FakeSocket } from '../test/fakeSocket';
import { JoinScreen } from './JoinScreen';
import { StagingScreen } from './StagingScreen';
import { hostThisTab, joinedFromThisTab } from '../test/fakeHosting';
import { setOnlineApi } from '../platform/online';

vi.mock('../net/hosting', async () => (await import('../test/fakeHosting')).fakeHostingModule());

vi.mock('./HexBackdrop', () => ({ HexBackdrop: () => null }));

let container: HTMLDivElement;
let root: Root;
let sockets: FakeSocket[];
const status: ServerStatus = {
  mode: 'server', has_game: true, turn: 5, is_local: true, staging: null,
  seats: [{ seat: 0, player: 0, name: 'Host', faction: 'India', connected: false, reserved: true, ready: false, orders: 0 }],
};

beforeEach(() => {
  vi.useFakeTimers();
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  sessionStorage.clear();
  sockets = [];
  vi.stubGlobal('WebSocket', class extends FakeSocket {
    constructor(url: string) { super(url); sockets.push(this); }
  });
  useGame.setState(useGame.getInitialState(), true);
  container = document.createElement('div');
  document.body.append(container);
  root = createRoot(container);
});

afterEach(async () => {
  await act(async () => {
    root.unmount();
    useGame.getState().exitStaging();
    useGame.getState().leaveOnline();
  });
  container.remove();
  setOnlineApi();
  vi.useRealTimers();
  vi.unstubAllGlobals();
});

const button = (label: string) => [...container.querySelectorAll('button')].find((b) => b.textContent?.trim() === label);

it.each([true, false])('offers explicit reclaim after an invalid saved credential only to loopback (local=%s)', async (isLocal) => {
  rememberSeat({ seat: 0, name: 'Host', gameId: 'old-game', token: 'expired-token' });
  const currentStatus = { ...status, is_local: isLocal };
  vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ ok: true, text: async () => JSON.stringify(currentStatus) }));
  if (isLocal) hostThisTab(); else joinedFromThisTab();
  await useGame.getState().boot();
  sockets[0].open();
  expect(sockets[0].sent).toEqual([{ type: 'join', seat: 0, name: 'Host', token: 'expired-token' }]);
  useGame.getState().reclaimOnlineSeat();
  expect(sockets).toHaveLength(1); // No rejection, no reassignment.
  sockets[0].push({ type: 'error', code: 'invalid_seat_credential', message: 'Invalid seat credential.' });
  expect(useGame.getState().screen).toBe('join');
  await act(async () => root.render(createElement(JoinScreen)));
  // The Join screen's status refresh must not forget the rejection.
  expect(useGame.getState().net?.credentialRejected).toBe(true);
  expect(!!button('Reclaim my seat')).toBe(isLocal);

  await act(async () => {
    sockets[0].drop();
    vi.advanceTimersByTime(500);
    sockets[1].open();
  });
  expect(sockets[1].sent).toEqual([{ type: 'join', seat: 0, name: 'Host', token: 'expired-token' }]);
  if (!isLocal) {
    useGame.getState().reclaimOnlineSeat();
    expect(sockets).toHaveLength(2);
    expect(rememberedSeat()?.token).toBe('expired-token');
    return;
  }
  await act(async () => button('Reclaim my seat')!.click());
  expect(button('Reclaim my seat')!.disabled).toBe(true);
  sockets[2].open();
  expect(sockets[2].sent).toEqual([{ type: 'game_take', seat: 0 }]);
  expect(rememberedSeat()?.token).toBe('expired-token');
  await act(async () => {
    sockets[2].push({ type: 'seat_token', seat: 0, game_id: 'restarted', token: 'fresh-token' });
    sockets[2].push({ type: 'welcome', seat: 0, game_id: 'restarted', orders: [] });
    sockets[2].push({ type: 'view', view: { turn: 5, me: 0, team: 0, units: [], cities: [], players: [], finished: false } as unknown as PlayerView });
  });
  expect(sockets[2].sent.at(-1)).toEqual({ type: 'join', seat: 0, name: 'Host', token: 'fresh-token' });
  expect(rememberedSeat()).toEqual({ seat: 0, name: 'Host', gameId: 'restarted', token: 'fresh-token' });
  expect(useGame.getState()).toMatchObject({ screen: 'game', busy: null, error: null, net: { conn: 'open', credentialRejected: false } });
  expect(button('Reclaim my seat')).toBeUndefined();
});

it('does not offer reclaim for an unrelated error or an ordinary explicit Join', async () => {
  rememberSeat({ seat: 0, name: 'Host', gameId: 'old-game', token: 'expired-token' });
  vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ ok: true, text: async () => JSON.stringify(status) }));
  hostThisTab();
  await useGame.getState().boot();
  sockets[0].open();
  sockets[0].push({ type: 'error', code: 'bad_request', message: 'Invalid seat credential.' });
  await act(async () => root.render(createElement(JoinScreen)));
  expect(button('Reclaim my seat')).toBeUndefined();
  await act(async () => useGame.getState().joinSeat(0, 'Host'));
  sockets[1].open();
  expect(sockets[1].sent).toEqual([{ type: 'join', seat: 0, name: 'Host', token: 'expired-token' }]);
});

it('disables Leave with a reason while disconnected, then retains UI ownership until release is acknowledged', async () => {
  const { seed: _seed, ...config } = configFromLobby(newLobby('host', '7'));
  const state: StagingState = { room_id: 'room', config, occupants: config.seats.map((s, i) => i < 2
    ? { name: i === 0 ? 'Host' : 'Alice', faction: s.faction, ready: false, connected: true, is_host: i === 0 } : null) };
  rememberSeat({ seat: 1, name: 'Alice', gameId: null, roomId: 'room', token: 'secret' });
  useGame.setState({ screen: 'staging', net: { status: { ...status, is_local: false, staging: state }, seats: [], mySeat: null, conn: 'closed' } });
  await act(async () => root.render(createElement(StagingScreen)));
  await act(async () => {
    sockets[0].open();
    sockets[0].push({ type: 'staging_taken', room_id: 'room', seat: 1, token: 'secret' });
    sockets[0].push({ type: 'staging', state });
  });
  expect(button('Leave seat')!.disabled).toBe(false);
  await act(async () => sockets[0].drop());
  expect(button('Leave seat')!.disabled).toBe(true);
  expect(button('Leave seat')!.getAttribute('aria-describedby')).toBe('leave-seat-reason');
  expect(container.querySelector('#leave-seat-reason')?.textContent).toContain('connected again');
  await act(async () => {
    button('Leave seat')!.click();
    useGame.getState().leaveStagingSeat(); // The store also rejects a disconnected action.
  });
  expect(useGame.getState().stagingSeat).toBe(1);
  expect(rememberedSeat()?.token).toBe('secret');
  expect(sockets[0].types()).not.toContain('staging_leave');
  await act(async () => {
    vi.advanceTimersByTime(1000);
    sockets[1].open();
    sockets[1].push({ type: 'staging_taken', room_id: 'room', seat: 1, token: 'secret' });
    sockets[1].push({ type: 'staging', state });
  });
  expect(sockets[1].sent.at(-1)).toEqual({ type: 'staging_take', seat: 1, name: 'Alice', token: 'secret' });
  expect(button('Leave seat')!.disabled).toBe(false);
  await act(async () => button('Leave seat')!.click());
  expect(sockets[1].sent.at(-1)).toEqual({ type: 'staging_leave' });
  expect(button('Leave seat')!.disabled).toBe(true);
  expect(container.querySelector('#leave-seat-reason')?.textContent).toContain('Leaving your seat');
  expect(useGame.getState().stagingSeat).toBe(1);
  expect(rememberedSeat()?.token).toBe('secret');
  await act(async () => sockets[1].push({ type: 'staging_left', room_id: 'room' }));
  expect(useGame.getState()).toMatchObject({ stagingSeat: null, stagingLeaving: false });
  expect(rememberedSeat()).toBeNull();
  expect(button('Leave seat')).toBeUndefined();
});
