// @vitest-environment happy-dom
// The Ping control shows only with a human teammate, starts ping mode, and offers a way to a teammate's live ping.
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { FixtureClient } from '../test/fixtureEngine';
import { coopVsAiSetup } from '../gameSetup';
import { useGame } from '../store';
import { PingControl } from './PingControl';
import { drawPingGhost, drawPings } from '../map/pingDraw';

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
  const client = new FixtureClient();
  await client.newGame(setup);
  useGame.setState({ client, humans: [0, 1], views: [client.view(0), client.view(1)], canPing: () => true });
  act(() => root.render(<PingControl />));
  const button = host.querySelector<HTMLButtonElement>('[data-testid=ping-button]')!;
  // Idle: "Ping" with its P key chip, like the neighbouring controls.
  expect(button.textContent).toBe('Ping P');
  expect(button.querySelector('kbd')?.textContent).toBe('P');
  expect(button.getAttribute('aria-pressed')).toBe('false');
  act(() => button.click());
  expect(useGame.getState().pingMode).toBe(true);
  // Armed: Cancel ping with an Esc chip, the instruction beside it.
  expect(button.textContent).toBe('Cancel ping Esc');
  expect(button.getAttribute('aria-pressed')).toBe('true');
  expect(host.querySelector('[data-testid=ping-hint]')?.textContent).toBe('Click a tile to ping it');
  expect(host.querySelector('[data-testid=ping-go]')).toBeNull();
  // Bo pings; Ada sees the way to it.
  const view = useGame.getState().views![0];
  act(() => useGame.setState({ pings: [{ from: 1, team: view.team, hex: { q: 2, r: 2 }, turn: view.turn, seq: 1 }] }));
  const go = host.querySelector<HTMLElement>('[data-testid=ping-go]')!;
  expect(go.textContent).toBe('Bo pinged · Go to ping');
  expect(go.querySelector('.medallion')).not.toBeNull(); // the sender's crest
  expect(go.title).toBe('Bo pinged a tile. Go there.');
});

it('draws a ring, with spreading rings only while it is new', () => {
  const calls: string[] = [];
  const ctx = new Proxy({}, { get: (_t, k) => (typeof k === 'string' && ['save', 'restore', 'beginPath', 'arc', 'stroke', 'fill', 'moveTo', 'lineTo'].includes(k) ? () => calls.push(k) : undefined), set: () => true }) as unknown as CanvasRenderingContext2D;
  const mark = { hex: { q: 1, r: 1 }, color: '#a00', own: false };
  drawPings(ctx, [{ ...mark, age: 100 }], () => ({ x: 0, y: 0 }), 30, '#fff', false);
  const fresh = calls.filter((c) => c === 'arc').length;
  calls.length = 0;
  drawPings(ctx, [{ ...mark, age: 5000 }], () => ({ x: 0, y: 0 }), 30, '#fff', false);
  const settled = calls.filter((c) => c === 'arc').length;
  calls.length = 0;
  drawPings(ctx, [{ ...mark, age: 100 }], () => ({ x: 0, y: 0 }), 30, '#fff', true);
  // Three ring strokes (keyline, halo, colour) and three centre dots; a fresh ping adds a wave ring.
  expect(settled).toBe(6);
  expect(fresh).toBe(7);
  expect(calls.filter((c) => c === 'arc').length).toBe(6);
});

it('has its own shape: four ticks across the ring, inside an ink keyline, and a ghost while armed', () => {
  const moves: number[] = [];
  const styles: string[] = [];
  let n = 0;
  const ctx = new Proxy({}, {
    get: (_t, k) => (k === 'moveTo' ? () => { n++; } : k === 'stroke' ? () => { moves.push(n); n = 0; } : typeof k === 'string' && ['save', 'restore', 'beginPath', 'arc', 'fill', 'lineTo'].includes(k) ? () => {} : undefined),
    set: (_t, k, v) => { if (k === 'strokeStyle') styles.push(String(v)); return true; },
  }) as unknown as CanvasRenderingContext2D;
  drawPings(ctx, [{ hex: { q: 1, r: 1 }, color: '#d8dde3', age: 5000, own: false }], () => ({ x: 0, y: 0 }), 30, '#fffaf0', true);
  expect(moves).toEqual([4, 4, 4]); // four ticks on each of the three strokes
  expect(styles).toEqual(['#2b2419', '#fffaf0', '#d8dde3']); // ink keyline outermost, then the halo, then the empire colour
  moves.length = 0; styles.length = 0;
  drawPingGhost(ctx, { x: 0, y: 0 }, 30, '#fffaf0');
  expect(moves).toEqual([4, 4, 4]);
});
