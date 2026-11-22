// @vitest-environment happy-dom
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { FixtureClient } from '../test/fixtureEngine';
import { RemoteClient, type SeatStatus } from '../remote';
import { useGame } from '../store';
import { SeatBar } from './SeatBar';

let container: HTMLDivElement;
let root: Root;
beforeEach(() => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  container = document.body.appendChild(document.createElement('div'));
  root = createRoot(container);
  useGame.setState(useGame.getInitialState(), true);
});
afterEach(() => {
  act(() => root.unmount());
  container.remove();
  vi.unstubAllGlobals();
});

it('waits for living online players and drops eliminated players from the waiting text', async () => {
  const mock = new FixtureClient();
  await mock.newGame(await mock.standardSetup('7', 'India', 'Byzantium', 'Ana', 'Ben'));
  const view = mock.view(0);
  const seats: SeatStatus[] = view.players.slice(0, 3).map((p, seat) => ({
    seat, player: p.id, name: p.name, faction: p.faction,
    connected: true, reserved: true, ready: seat === 0, orders: 0,
  }));
  useGame.setState({
    client: new RemoteClient(0, 'Ana'), views: [view], humans: [0], seat: 0,
    ready: [true], orders: [[]], errors: [[]],
    net: {
      status: { mode: 'server', has_game: true, turn: view.turn, seats, is_local: true },
      seats, mySeat: 0, conn: 'open',
    },
  });
  act(() => root.render(<SeatBar />));
  const label = () => container.querySelector('.cta-label')?.textContent;
  expect(label()).toBe(`Waiting for ${seats[1].name}, ${seats[2].name}…`);
  act(() => useGame.setState({
    views: [{ ...view, players: view.players.map((p) => p.id === seats[1].player ? { ...p, alive: false } : p) }],
  }));
  expect(label()).toBe(`Waiting for ${seats[2].name}…`);
  const net = useGame.getState().net!;
  act(() => useGame.setState({
    net: { ...net, seats: seats.map((s) => s.seat === 2 ? { ...s, ready: true } : s) },
  }));
  expect(label()).toBe('Resolving…');
});
