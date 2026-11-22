// @vitest-environment happy-dom
// The Ping control shows only with a human teammate, starts ping mode, and offers a way to a teammate's live ping.
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { FixtureClient } from '../test/fixtureEngine';
import { coopVsAiSetup } from '../gameSetup';
import { useGame } from '../store';
import { PingControl } from './PingControl';
import { drawPings } from '../map/pingDraw';

vi.mock('../client', async (orig) => ({
  ...(await orig<typeof import('../client')>()),
  createClient: async () => new FixtureClient(),
}));
vi.mock('../platform/archive', () => ({
  recordGameHistory: vi.fn().mockResolvedValue(undefined),
  abandonGameHistory: vi.fn().mockResolvedValue(undefined),
}));

let root: Root, host: HTMLDivElement;
beforeEach(() => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  useGame.setState(useGame.getInitialState(), true);
  host = document.body.appendChild(document.createElement('div'));
  root = createRoot(host);
});
afterEach(() => { act(() => root.unmount()); host.remove(); vi.unstubAllGlobals(); });

it('is not shown in a solo game', async () => {
  await useGame.getState().startGame(coopVsAiSetup('42', [{ name: 'Ada', faction: 'India' }]));
  act(() => root.render(<PingControl />));
  expect(host.querySelector('[data-testid=ping-control]')).toBeNull();
});

it('with a teammate: Ping starts ping mode, and a teammate\'s live ping gets a way to it', async () => {
  const setup = coopVsAiSetup('42', [{ name: 'Ada', faction: 'India' }, { name: 'Bo', faction: 'Byzantium' }]);
  setup.players = setup.players.slice(0, 4);
  await useGame.getState().startGame(setup);
  act(() => root.render(<PingControl />));
  const button = host.querySelector<HTMLButtonElement>('[data-testid=ping-button]')!;
  expect(button.textContent).toBe('Ping');
  expect(button.getAttribute('aria-pressed')).toBe('false');
  act(() => button.click());
  expect(useGame.getState().pingMode).toBe(true);
  expect(button.textContent).toBe('Click a tile');
  expect(button.getAttribute('aria-pressed')).toBe('true');
  expect(host.querySelector('[data-testid=ping-go]')).toBeNull();
  // Bo pings; Ada sees the way to it.
  const view = useGame.getState().views![0];
  act(() => useGame.setState({ pings: [{ from: 1, team: view.team, hex: { q: 2, r: 2 }, turn: view.turn, seq: 1 }] }));
  const go = host.querySelector<HTMLElement>('[data-testid=ping-go]')!;
  expect(go.textContent).toBe('Go to ping');
  expect(go.title).toBe('Bo pinged a tile. Go there.');
});

it('draws a ring, with spreading rings only while it is new', () => {
  const calls: string[] = [];
  const ctx = new Proxy({}, { get: (_t, k) => (typeof k === 'string' && ['save', 'restore', 'beginPath', 'arc', 'stroke', 'fill'].includes(k) ? () => calls.push(k) : undefined), set: () => true }) as unknown as CanvasRenderingContext2D;
  const mark = { hex: { q: 1, r: 1 }, color: '#a00', own: false };
  drawPings(ctx, [{ ...mark, age: 100 }], () => ({ x: 0, y: 0 }), 30, '#fff', false);
  const fresh = calls.filter((c) => c === 'arc').length;
  calls.length = 0;
  drawPings(ctx, [{ ...mark, age: 5000 }], () => ({ x: 0, y: 0 }), 30, '#fff', false);
  const settled = calls.filter((c) => c === 'arc').length;
  calls.length = 0;
  drawPings(ctx, [{ ...mark, age: 100 }], () => ({ x: 0, y: 0 }), 30, '#fff', true);
  expect(settled).toBe(4);
  expect(fresh).toBe(5);
  expect(calls.filter((c) => c === 'arc').length).toBe(4);
});
