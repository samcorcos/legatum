// @vitest-environment happy-dom
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { useGame } from '../store';
import { keptSeat, rememberSeat, rememberedSeat, setServerBase, type SeatStatus } from '../remote';
import { awayLine, awayPlayers, namesList } from '../awayPlayers';
import { configFromLobby, lobbyFromStaging, type StagingState } from '../staging';
import { newLobby } from '../lobby';
import { FakeSocket } from '../test/fakeSocket';
import type { PlayerView } from '../types';
import { JoinScreen } from '../screens/JoinScreen';
import { SeatsTable } from '../screens/lobby/SeatsTable';
import { OnlineSeats } from './SeatBar';

vi.mock('../screens/HexBackdrop', () => ({ HexBackdrop: () => null }));

let container: HTMLDivElement;
let root: Root;
let sockets: FakeSocket[];
const seats: SeatStatus[] = [
  { seat: 0, player: 0, name: 'Host', faction: 'India', connected: true, reserved: true, ready: false, orders: 0 },
  { seat: 1, player: 1, name: 'Alice', faction: 'Byzantium', connected: false, reserved: true, ready: false, orders: 2 },
  { seat: 2, player: 2, name: 'Bob', faction: 'Celts', connected: true, reserved: true, ready: true, orders: 0 },
  { seat: 3, player: 3, name: 'Open empire', faction: 'Germany', connected: false, reserved: false, ready: false, orders: 0 },
];

beforeEach(() => {
  vi.useFakeTimers();
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  sessionStorage.clear();
  sockets = [];
  vi.stubGlobal('WebSocket', class extends FakeSocket {
    constructor(url: string) { super(url); sockets.push(this); }
  });
  useGame.setState(useGame.getInitialState(), true);
  useGame.setState({
    net: {
      status: { mode: 'server', has_game: true, turn: 1, seats, is_local: true, staging: null },
      seats, mySeat: null, conn: 'closed',
    },
    refreshStatus: vi.fn().mockResolvedValue(null),
  });
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
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
  vi.useRealTimers();
});

const openButtons = () => [...container.querySelectorAll<HTMLButtonElement>('button.open-seat')];

function connectGame() {
  rememberSeat({ seat: 0, name: 'Host', gameId: 'g1', token: 'host-token' });
  useGame.getState().joinSeat(0, 'Host');
  sockets[0].open();
  sockets[0].push({ type: 'welcome', seat: 0, game_id: 'g1', orders: [] });
  sockets[0].push({ type: 'view', view: { turn: 1, me: 0, team: 0, players: [], units: [], cities: [], finished: false } as unknown as PlayerView });
}

it('only opens another disconnected reserved empire after host confirmation and server acknowledgment', async () => {
  connectGame();
  const confirm = vi.fn().mockReturnValue(false);
  vi.stubGlobal('confirm', confirm);
  await act(async () => root.render(<OnlineSeats />));
  expect(openButtons()).toHaveLength(1);
  expect(openButtons()[0].closest('[data-testid]')?.getAttribute('data-testid')).toBe('online-seat-1');
  await act(async () => openButtons()[0].click());
  expect(confirm).toHaveBeenCalledWith("Let another device take over Alice's empire?");
  expect(sockets[0].types()).toEqual(['join']);
  confirm.mockReturnValue(true);
  await act(async () => openButtons()[0].click());
  expect(sockets[0].sent.at(-1)).toEqual({ type: 'game_release', seat: 1 });
  expect(rememberedSeat()).toMatchObject({ seat: 0, token: 'host-token' });
  expect(openButtons()).toHaveLength(1); // No optimistic release before the server accepts it.
  await act(async () => sockets[0].push({ type: 'seats', seats: seats.map((s) => s.seat === 1 ? { ...s, reserved: false, orders: 0 } : s) }));
  expect(openButtons()).toHaveLength(0);
  expect(container.querySelector('[data-testid="online-seat-1"]')?.textContent).toContain('open');
});

it('hides release for online guests and disables it while the host reconnects', async () => {
  connectGame();
  const net = useGame.getState().net!;
  useGame.setState({ net: { ...net, status: { ...net.status, is_local: false } } });
  await act(async () => root.render(<OnlineSeats />));
  expect(openButtons()).toHaveLength(0);
  useGame.getState().releaseOnlineSeat(1);
  expect(sockets[0].types()).toEqual(['join']);
  await act(async () => {
    useGame.setState({ net });
    sockets[0].drop();
  });
  expect(openButtons()[0].disabled).toBe(true);
  useGame.getState().releaseOnlineSeat(1);
  expect(sockets[0].types()).toEqual(['join']);
  expect(useGame.getState().error).toContain('connected again');
});

it('rechecks live seat ownership and connection state before sending a release', () => {
  connectGame();
  for (const seat of [0, 2, 3, 8]) useGame.getState().releaseOnlineSeat(seat);
  expect(sockets[0].types()).toEqual(['join']);
  sockets[0].push({ type: 'seats', seats: seats.map((s) => ({ ...s, connected: true })) });
  useGame.getState().releaseOnlineSeat(1);
  expect(sockets[0].types()).toEqual(['join']);
});

function StagingSeats({ guest = false }: { guest?: boolean }) {
  const state = useGame((s) => s.staging)!;
  return <SeatsTable lobby={lobbyFromStaging(state)} room={{ state, guest }} />;
}

function connectStaging() {
  const { seed, ...config } = configFromLobby(newLobby('host', '7'));
  const state: StagingState = {
    room_id: 'r1', config, seed,
    occupants: config.seats.map((s, i) => i < 2 ? { name: i === 0 ? 'Host' : 'Alice', faction: s.faction, connected: i === 0, ready: false, is_host: i === 0 } : null),
  };
  useGame.getState().enterStaging(true);
  sockets[0].open();
  sockets[0].push({ type: 'staging_taken', room_id: 'r1', seat: 0, token: 'host-token' });
  sockets[0].push({ type: 'staging', state });
  return state;
}

it('shows another player’s chosen faction in their own seat row, not as a picker this guest can open', async () => {
  const state = connectStaging();
  state.occupants[1] = { ...state.occupants[1]!, faction: 'Byzantium' };
  sockets[0].push({ type: 'staging', state });
  await act(async () => root.render(<StagingSeats guest />));
  const row = container.querySelectorAll('.seat-row')[2];
  expect(row.textContent).toContain('Byzantium');
  const shown = row.querySelector<HTMLButtonElement>('button[aria-label^="Seat 2 faction"]');
  expect(shown?.getAttribute('aria-label')).toBe('Seat 2 faction: Byzantium');
  expect(shown?.disabled).toBe(true);
});

it('releases a staging reservation in one confirmed action without changing seat type or host credential', async () => {
  const state = connectStaging();
  const confirm = vi.fn().mockReturnValue(false);
  vi.stubGlobal('confirm', confirm);
  await act(async () => root.render(<StagingSeats />));
  expect(openButtons()).toHaveLength(1);
  await act(async () => openButtons()[0].click());
  expect(sockets[0].types()).toEqual(['staging_watch']);
  confirm.mockReturnValue(true);
  await act(async () => openButtons()[0].click());
  expect(sockets[0].sent.at(-1)).toEqual({ type: 'staging_release', seat: 1 });
  expect(rememberedSeat()).toMatchObject({ seat: 0, token: 'host-token', roomId: 'r1' });
  expect(useGame.getState().staging?.config.seats[1].kind).toBe('open');
  await act(async () => sockets[0].push({ type: 'staging', state: { ...state, occupants: state.occupants.map((o, i) => i === 1 ? null : o) } }));
  expect(openButtons()).toHaveLength(0);
  expect(container.textContent).toContain('Open · waiting for a player');
});

it('does not expose staging release to guests or allow it for connected, own, or empty seats', async () => {
  const state = connectStaging();
  await act(async () => root.render(<StagingSeats guest />));
  expect(openButtons()).toHaveLength(0);
  const net = useGame.getState().net!;
  await act(async () => {
    useGame.setState({ net: { ...net, status: { ...net.status, is_local: false } } });
    root.render(<StagingSeats />);
  });
  expect(openButtons()).toHaveLength(0);
  useGame.getState().releaseStagingSeat(1);
  await act(async () => {
    useGame.setState({ net });
    sockets[0].push({ type: 'staging', state: { ...state, occupants: state.occupants.map((o) => o && { ...o, connected: true }) } });
  });
  expect(openButtons()).toHaveLength(0);
  for (const seat of [0, 1, 2, 8]) useGame.getState().releaseStagingSeat(seat);
  expect(sockets[0].types()).toEqual(['staging_watch']);
});

it('tells a fresh guest to ask the host, then offers Rejoin as the empire when opened', async () => {
  const net = useGame.getState().net!;
  useGame.setState({ net: { ...net, status: { ...net.status, is_local: false } } });
  await act(async () => root.render(<JoinScreen />));
  const alice = () => [...container.querySelectorAll<HTMLButtonElement>('.join-seat')].find((b) => b.textContent?.includes('Alice'))!;
  expect(alice().disabled).toBe(true);
  expect(alice().textContent).toContain('Reserved');
  expect(alice().textContent).toContain('Ask the host to open this seat');
  expect(alice().querySelector('.join-status')?.textContent).not.toContain('Open');
  await act(async () => useGame.setState({ net: { ...net, status: { ...net.status, is_local: false, seats: seats.map((s) => s.seat === 1 ? { ...s, reserved: false } : s) } } }));
  expect(alice().disabled).toBe(false);
  expect(alice().textContent).toContain('Rejoin as Alice');
  await act(async () => alice().click());
  sockets[0].open();
  expect(sockets[0].sent).toEqual([{ type: 'game_take', seat: 1 }]);
});

it('a guest picks their own faction in their own seat row', async () => {
  const state = connectStaging();
  const onMyFaction = vi.fn();
  await act(async () => root.render(<SeatsTable lobby={lobbyFromStaging(state)} room={{ state, guest: true, mySeat: 1, onMyFaction }} />));
  const row = container.querySelectorAll('.seat-row')[2];
  const mine = row.querySelector<HTMLButtonElement>('button[aria-label^="Your faction"]')!;
  expect(mine.disabled).toBe(false);
  expect(row.textContent).toContain('You');
  // Nobody else's picker can be opened from this browser.
  expect([...container.querySelectorAll<HTMLButtonElement>('button[aria-label^="Seat "][aria-label*="faction"]')].every((b) => b.disabled)).toBe(true);
});

it('after a restart, the game list offers this device its kept seat back without the host opening it', async () => {
  setServerBase('https://relay.example/rooms/r1');
  try {
    rememberSeat({ seat: 1, name: 'Alice', gameId: 'g1', token: 'alice-token' });
    sessionStorage.clear(); // The restart: the tab's own copy is gone, the device's kept copy is not.
    expect(keptSeat('https://relay.example/rooms/r1')).toMatchObject({ seat: 1, token: 'alice-token' });
    expect(keptSeat('https://relay.example/rooms/other')).toBeNull();
    const net = useGame.getState().net!;
    useGame.setState({ net: { ...net, status: { ...net.status, is_local: false } } });
    await act(async () => root.render(<JoinScreen />));
    const alice = [...container.querySelectorAll<HTMLButtonElement>('button.join-seat')][1];
    expect(alice.disabled).toBe(false);
    expect(alice.textContent).toContain('Rejoin as Alice');
    await act(async () => alice.click());
    expect(rememberedSeat()).toMatchObject({ seat: 1, token: 'alice-token' });
    sockets[0].open();
    expect(sockets[0].sent[0]).toMatchObject({ type: 'join', seat: 1, token: 'alice-token' });
  } finally {
    setServerBase('');
    localStorage.clear();
  }
});

it('names the players who are away, so the turn never ends without them unannounced', () => {
  connectGame();
  sockets[0].push({ type: 'seats', seats });
  expect(awayPlayers(useGame.getState()).map((a) => a.name)).toEqual(['Alice', 'Open empire']);
  expect(awayLine(awayPlayers(useGame.getState()).slice(0, 1), 'until')).toBe('Alice is not connected. Byzantium gets no orders until Alice rejoins.');
  expect(namesList(['Alice', 'Bo', 'Cy'])).toBe('Alice, Bo and Cy');
  sockets[0].push({ type: 'seats', seats: seats.map((s) => ({ ...s, connected: true })) });
  expect(awayPlayers(useGame.getState())).toEqual([]);
});
