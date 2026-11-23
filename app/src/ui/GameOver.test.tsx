// @vitest-environment happy-dom
import { act, StrictMode } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { GameOver, winReason } from './GameOver';
import type { TeamTrack } from '../vp';
import { saveFile, useGame } from '../store';
import { FixtureClient } from '../test/fixtureEngine';
import { coopVsAiSetup } from '../gameSetup';
import { FakeSocket } from '../test/fakeSocket';
import { useCombatFx } from '../feel';
import type { SaveFile } from '../store';

let root: Root;
let host: HTMLDivElement;
let opener: HTMLButtonElement;
let finishedSave: SaveFile;
let unfinishedSave: SaveFile;
beforeEach(async () => {
  useCombatFx.getState().update({ enabled: true });
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  document.body.innerHTML = '';
  opener = document.body.appendChild(document.createElement('button'));
  opener.textContent = 'Game menu';
  opener.focus();
  host = document.body.appendChild(document.createElement('div'));
  root = createRoot(host);
  useGame.setState(useGame.getInitialState(), true);
  const client = new FixtureClient();
  await client.newGame(coopVsAiSetup('42', [{ name: 'Ada', faction: 'India' }]));
  unfinishedSave = saveFile(client, [client.view(0)]);
  Object.assign(client.debugState(), { finished: true, winner: 0 });
  finishedSave = saveFile(client, [client.view(0)]);
  await useGame.getState().loadSave(finishedSave);
  act(() => root.render(<StrictMode><GameOver /></StrictMode>));
});
afterEach(() => {
  act(() => { root.unmount(); useGame.getState().leaveOnline(); });
  useCombatFx.getState().update({ enabled: true });
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

const key = (target: HTMLElement, key: string, shiftKey = false) => act(() => {
  target.dispatchEvent(new KeyboardEvent('keydown', { key, shiftKey, bubbles: true, cancelable: true }));
});
const dismiss = () => act(() => host.querySelector<HTMLButtonElement>('.go-actions button')!.click());

it('opens a labelled native modal, contains keyboard focus and shortcuts, and restores focus on dismissal', () => {
  const dialog = host.querySelector('dialog')!;
  expect(dialog.open).toBe(true);
  expect(document.getElementById(dialog.getAttribute('aria-labelledby')!)?.textContent).toBe('Victory');
  const [map, title] = dialog.querySelectorAll<HTMLButtonElement>('button');
  expect(document.activeElement).toBe(map);
  expect(map.textContent).toBe('View the map');
  key(map, 'Tab', true);
  expect(document.activeElement).toBe(title);
  key(title, 'Tab');
  expect(document.activeElement).toBe(map);
  const backgroundKeys = vi.fn();
  window.addEventListener('keydown', backgroundKeys);
  try {
    key(map, 't');
    expect(backgroundKeys).not.toHaveBeenCalled();
    key(map, 'Escape');
    expect(host.querySelector('dialog')).toBeNull();
    expect(document.activeElement).toBe(opener);
    expect(backgroundKeys).not.toHaveBeenCalled();
  } finally {
    window.removeEventListener('keydown', backgroundKeys);
  }
});

it('leads with the winning team\'s leader portraits and a ribbon, and lists every empire with a portrait thumbnail', () => {
  const dialog = host.querySelector('dialog')!;
  // The solo setup puts India on the winning team.
  expect(dialog.querySelectorAll('.go-hero .pf')).toHaveLength(1);
  expect(dialog.querySelector('.go-hero .pf')!.classList.contains('fallen')).toBe(false);
  expect(dialog.querySelector('.go-hero .rb')?.textContent).toBe('India');
  const rows = dialog.querySelectorAll('.go-row');
  expect(rows.length).toBeGreaterThan(1);
  expect([...rows].every((r) => r.querySelector('.pf-roundel'))).toBe(true);
  expect(dialog.querySelector('.go-row.win')?.textContent).toContain('You');
});

it('restores focus when View the map is clicked or the component unmounts', () => {
  dismiss();
  expect(document.activeElement).toBe(opener);
  act(() => useGame.setState({ outcomeDismissed: false }));
  expect(host.querySelector('dialog')!.open).toBe(true);
  act(() => root.render(null));
  expect(document.activeElement).toBe(opener);
});

it('resets dismissal when the same finished save is loaded again without unmounting', async () => {
  dismiss();
  expect(host.querySelector('dialog')).toBeNull();
  await act(async () => { await useGame.getState().loadSave(finishedSave); });
  expect(host.querySelector('dialog')!.open).toBe(true);
  expect(document.activeElement?.textContent).toBe('View the map');
});

it('shows the eventual outcome after loading an unfinished save', async () => {
  dismiss();
  await act(async () => { await useGame.getState().loadSave(unfinishedSave); });
  expect(host.querySelector('dialog')).toBeNull();
  expect(useGame.getState().outcomeDismissed).toBe(false);
  // The next turn finishes the installed game, retaining the same client and screen.
  act(() => useGame.setState({ views: useGame.getState().views!.map((v) => ({ ...v, finished: true, winner: v.team })) }));
  expect(host.querySelector('dialog')!.open).toBe(true);
});

it('resets dismissal for an installed online session and a replacement game on the same socket', () => {
  const socket = new FakeSocket('ws://host/ws');
  vi.stubGlobal('WebSocket', class { constructor() { return socket; } });
  const view = useGame.getState().views![0];
  dismiss();
  act(() => {
    useGame.getState().joinSeat(0, 'Ada');
    socket.open();
    socket.push({ type: 'welcome', seat: 0, game_id: 'g1', orders: [] });
    socket.push({ type: 'view', view });
  });
  expect(host.querySelector('dialog')!.open).toBe(true);
  dismiss();
  act(() => socket.push({ type: 'view', view }));
  expect(host.querySelector('dialog')).toBeNull(); // Same session's ordinary refresh retains dismissal.
  act(() => {
    socket.push({ type: 'welcome', seat: 0, game_id: 'g2', orders: [] });
    socket.push({ type: 'view', view });
  });
  expect(host.querySelector('dialog')!.open).toBe(true);
});

async function showDefeat(dead = [0]) {
  await act(async () => {
    await useGame.getState().loadSave(unfinishedSave);
    useGame.setState({
      views: useGame.getState().views!.map((v) => ({ ...v, turn: 9,
        players: v.players.map((p) => ({ ...p, alive: !dead.includes(p.id) })),
      })),
      eliminationTurns: { 0: 7, 1: 8 },
    });
  });
}

it('shows an unfinished eliminated empire a labelled Defeat modal with turn, cause and observer actions', async () => {
  await showDefeat();
  const dialog = host.querySelector('dialog')!;
  expect(dialog.open).toBe(true);
  expect(document.getElementById(dialog.getAttribute('aria-labelledby')!)?.textContent).toBe('Defeat');
  expect(dialog.textContent).toContain('Your empire has fallen');
  expect(dialog.textContent).toContain('Eliminated on turn 7');
  expect(dialog.textContent).toContain('No cities or settlers remain.');
  expect(document.activeElement?.textContent).toBe('Watch the rest of the game');
  // Your own leader, drained of colour, under a red ribbon.
  expect(dialog.querySelector('.go-hero .pf')!.classList.contains('fallen')).toBe(true);
  expect(dialog.querySelector('.go-hero .rb')?.textContent).toBe('Fallen');
  expect([...dialog.querySelectorAll('button')].map((b) => b.textContent)).toEqual(['Watch the rest of the game', 'Resolve to the end', 'Return to title']);
  dismiss();
  expect(host.querySelector('dialog')).toBeNull();
  expect(useGame.getState().outcomeDismissed).toBe(false);
  expect(useGame.getState().defeatDismissed).toEqual([0]);
});

it('offers automatic resolution only once all human empires are dead', async () => {
  await showDefeat([0]);
  const resolve = vi.fn();
  act(() => useGame.setState({ resolveToEnd: resolve }));
  const button = [...host.querySelectorAll('button')].find((b) => b.textContent === 'Resolve to the end')!;
  expect(button).toBeDefined();
  act(() => button.click());
  expect(resolve).toHaveBeenCalledOnce();
});

it('returns to title from Defeat and resets dismissal when an eliminated save is loaded', async () => {
  await showDefeat();
  const title = [...host.querySelectorAll('button')].find((b) => b.textContent === 'Return to title')!;
  act(() => title.click());
  expect(useGame.getState().screen).toBe('title');
  const client = new FixtureClient();
  client.load(unfinishedSave.game);
  client.debugState().players[0].alive = false;
  const defeatedSave = saveFile(client, [client.view(0)]);
  dismiss();
  await act(async () => { await useGame.getState().loadSave(defeatedSave); });
  expect(host.querySelector('dialog')!.open).toBe(true);
  expect(host.querySelector('h1')?.textContent).toBe('Defeat');
  // Saves have no elimination history: show the current turn, not an invented elimination turn.
  expect(host.querySelector('.go-kicker')?.textContent).toContain(`Turn ${defeatedSave.turn}`);
});

it('names how the game was won: VP target, last team standing, or turn limit', () => {
  const track = (team: number, vp: number, alive: boolean, isMine = false) => ({
    team, members: [{ id: team, name: `P${team}` }], label: `P${team}`, vp, target: 10, progress: vp / 10, needed: 10 - vp, isMine, alive, rank: 0,
  }) as unknown as TeamTrack;
  const me = track(0, 5, true, true);
  expect(winReason(me, [me, track(1, 3, false)], 10)).toBe('You are the last empire standing.');
  const rival = track(1, 3, true);
  expect(winReason(rival, [rival, track(0, 2, false, true)], 10)).toBe('P1 is the last empire standing.');
  expect(winReason(me, [me, track(1, 3, true)], 10)).toBe('You had the most VP at the turn limit.');
  const done = track(0, 10, true, true);
  expect(winReason(done, [done, track(1, 3, true)], 10)).toBe('You held the 10 VP target for 10 turns.');
  expect(winReason(done, [done, track(1, 3, true)], 5)).toBe('You held the 10 VP target for 5 turns.');
  const reason = "Ada holds the last capital: every rival's capital has fallen";
  expect(winReason(done, [done, track(1, 3, true)], 10, reason)).toBe("You hold the last original capital: every rival's has been taken or razed.");
  expect(winReason(rival, [rival, track(0, 2, true, true)], 10, reason)).toBe("P1 holds the last original capital: every rival's has been taken or razed.");
  // The GameWon reason decides: a team over its target at the turn limit had not finished holding.
  expect(winReason(done, [done, track(1, 3, true)], 10, 'Ada held 10 of 10 VP for 10 turns')).toBe('You held the 10 VP target for 10 turns.');
  expect(winReason(done, [done, track(1, 3, true)], 10, 'Ada has the most VP (11) at the turn limit')).toBe('You had the most VP at the turn limit.');
});

function withHistory() {
  act(() => useGame.setState({
    outcomeDismissed: false,
    views: useGame.getState().views!.map((v) => ({
      ...v,
      history: {
        first_turn: 1,
        empires: v.players.map((p) => ({ vp: [0, 1, 2 + p.id], cities: [1, 1, 2], population: [1, 2, 3], science: [1, 2, 3], gold: [10, 20, 30], army: [5, 6, 7], techs: [0, 1, 1], era: [1, 1, 2] })),
        chronicle: [
          { kind: 'Founded', turn: 1, player: 0, city: 'Kyoto' },
          { kind: 'Era', turn: 3, player: 0, era: 2, first: true },
          { kind: 'Won', turn: 3, team: v.team, reason: 'Ada held 10 of 10 VP for 10 turns' },
        ],
      },
    })),
  }));
}
const button = (name: string) => [...host.querySelectorAll<HTMLButtonElement>('button')].find((b) => b.textContent === name);

it('with a recorded history, the reveal leads to the game report, whose primary returns to title', () => {
  withHistory();
  const dialog = host.querySelector('dialog')!;
  expect(dialog.dataset.stage).toBe('reveal');
  expect(document.activeElement?.textContent).toBe('See the game report');
  expect([...dialog.querySelectorAll('.gr-actions button')].map((b) => b.textContent)).toEqual(['See the game report', 'Keep looking at the map', 'Return to title']);
  act(() => button('See the game report')!.click());
  expect(dialog.dataset.stage).toBe('report');
  expect(host.querySelector('[data-testid="postgame-report"]')).not.toBeNull();
  // Focus starts on the first tab, so a stray Return never leaves the game.
  expect(document.activeElement?.getAttribute('role')).toBe('tab');
  expect(document.activeElement?.textContent).toContain('Victory race');
  expect(host.querySelector('.pg-actions .btn.primary')?.textContent).toBe('Return to title');
  expect(host.querySelectorAll('.pg-legend .pg-key')).toHaveLength(useGame.getState().views![0].players.length);
  // The tabs switch with the arrow keys.
  key(document.activeElement as HTMLElement, 'ArrowRight');
  expect(document.activeElement?.textContent).toContain('Empires over time');
  act(() => button(' Timeline')?.click() ?? [...host.querySelectorAll<HTMLButtonElement>('[role="tab"]')][2].click());
  expect(host.querySelector('.pg-timeline')?.textContent).toContain('You founded Kyoto');
  act(() => [...host.querySelectorAll<HTMLButtonElement>('[role="tab"]')][3].click());
  expect(host.querySelectorAll('.pg-table tbody tr')).toHaveLength(useGame.getState().views![0].players.length);
});

it('closes the report to the map, reopens it straight to the report, and replays the reveal', () => {
  withHistory();
  act(() => button('See the game report')!.click());
  act(() => button('Keep looking at the map')!.click());
  expect(host.querySelector('dialog')).toBeNull();
  expect(useGame.getState().outcomeDismissed).toBe(true);
  act(() => useGame.getState().openGameReport());
  expect(host.querySelector('dialog')!.dataset.stage).toBe('report');
  act(() => button('Replay the reveal')!.click());
  expect(host.querySelector('dialog')!.dataset.stage).toBe('reveal');
  expect(host.querySelector('[data-testid="game-reveal"]')!.getAttribute('data-settled')).toBe('false');
  act(() => button('Return to title')!.click());
  expect(useGame.getState().screen).toBe('title');
});

it('Return during the reveal finishes it instead of acting; the next Return opens the report', () => {
  withHistory();
  const primary = document.activeElement as HTMLButtonElement;
  key(primary, 'Enter');
  expect(host.querySelector('[data-testid="game-reveal"]')!.getAttribute('data-settled')).toBe('true');
  expect(host.querySelector('dialog')!.dataset.stage).toBe('reveal');
  // The standings show their final numbers once settled.
  expect(host.querySelector('.go-row b')?.getAttribute('aria-label')).toMatch(/^\d+ of \d+ VP$/);
});

it('settles a running victory reveal when Map animations is switched Off', () => {
  expect(host.querySelector('[data-testid="game-reveal"]')?.getAttribute('data-settled')).toBe('false');
  act(() => useCombatFx.getState().update({ enabled: false }));
  expect(host.querySelector('[data-testid="game-reveal"]')?.getAttribute('data-settled')).toBe('true');
  expect(host.querySelector('.go-row b')?.getAttribute('aria-label')).toMatch(/^\d+ of \d+ VP$/);
});
