// @vitest-environment happy-dom
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { FixtureClient } from '../test/fixtureEngine';
import type { Faction, FactionGoalView, PlayerView } from '../types';
import { FactionGoalProgressLine, HoardReadout } from './FactionGoalProgress';

let host: HTMLElement, root: Root, base: PlayerView;
beforeEach(async () => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('42', 'India', 'Byzantium', 'Ada', 'Bo'));
  base = client.view(0);
  host = document.body.appendChild(document.createElement('div'));
  root = createRoot(host);
});
afterEach(() => { act(() => root.unmount()); host.remove(); vi.unstubAllGlobals(); });

const goals: { faction: Faction; name: string; value: number; target: number; label: string }[] = [
  { faction: 'Switzerland', name: 'The Reserve', value: 3800, target: 5000, label: '3,800 of 5,000 Gold' },
  { faction: 'India', name: 'The Imperial Diwan', value: 1, target: 1, label: 'The Imperial Diwan stands in City' },
  { faction: 'Byzantium', name: 'The Heir to the Throne', value: 1, target: 1, label: 'Yours' },
  { faction: 'Germany', name: 'Five Ruhr Foundries', value: 4, target: 5, label: '4 of 5 Ruhr Foundries' },
  { faction: 'Celts', name: 'Raise the Warbands', value: 3, target: 4, label: '3 of 4 cities with a wild camp in their territory' },
  { faction: 'Mongols', name: 'The Yam Network', value: 1, target: 1, label: 'The Yam Relay Station stands in City' },
  { faction: 'Sparta', name: 'Honour the Veterans', value: 3, target: 3, label: '3 of 3 veterans retired' },
  { faction: 'Athens', name: 'Share the Learning', value: 1, target: 1, label: 'The Learning is shared' },
];

function withGoal(faction: Faction, value: number, target: number, label: string, done = false): PlayerView {
  return {
    ...base,
    players: base.players.map((p) => p.id === base.me ? { ...p, faction } : p),
    faction_goal: { value, target, label, live: faction === 'Switzerland' || faction === 'Germany' || faction === 'Celts' || faction === 'India', done } satisfies FactionGoalView,
  };
}

it.each(goals)('$faction shows its named goal and view progress when close', ({ faction, name, value, target, label }) => {
  const view = withGoal(faction, value, target, label);
  act(() => root.render(<><FactionGoalProgressLine view={view} /><HoardReadout view={view} /></>));
  expect(host.querySelector('.faction-goal-progress')?.textContent).toContain(name);
  expect(host.querySelector('.faction-goal-progress')?.textContent).toContain(label);
  expect(host.querySelector('.faction-goal-meter')?.getAttribute('aria-valuenow')).toBe(String(value));
  expect(host.querySelector('.gold-hoard-readout')?.textContent ?? '').toBe(faction === 'Switzerland' ? 'Reserve 3,800 / 5,000' : '');
});

it('keeps This Turn quiet below 75%, while the live Swiss Gold target remains visible in the top bar', () => {
  const view = withGoal('Switzerland', 3500, 5000, '3,500 of 5,000 Gold');
  act(() => root.render(<><FactionGoalProgressLine view={view} /><HoardReadout view={view} /></>));
  expect(host.querySelector('.faction-goal-progress')).toBeNull();
  expect(host.querySelector('.gold-hoard-readout')?.textContent).toBe('Reserve 3,500 / 5,000');
});

it('shows a completed live goal in This Turn', () => {
  const view = withGoal('Switzerland', 5000, 5000, '5,000 of 5,000 Gold', true);
  act(() => root.render(<FactionGoalProgressLine view={view} />));
  expect(host.querySelector('.faction-goal-progress')?.textContent).toContain('The Reserve');
});

it('leaves an achievable action goal to its own This Turn row', () => {
  const baseView = withGoal('Athens', 0, 1, 'The Learning is not shared yet');
  const view = { ...baseView, actions: { ...baseView.actions, broadcast: { techs_each: 3, blocker: null, receivers: [1] } } };
  act(() => root.render(<FactionGoalProgressLine view={view} />));
  expect(host.querySelector('.faction-goal-progress')).toBeNull();
});

it('shows India’s Diwan when held even if a legacy Sakoku action is present', () => {
  const baseView = withGoal('India', 1, 1, 'The Imperial Diwan stands in City', true);
  const view = { ...baseView, actions: { ...baseView.actions, sakoku: { closed: true, turns: 24, target: 30, holding: false, blocker: null } } };
  act(() => root.render(<FactionGoalProgressLine view={view} />));
  expect(host.querySelector('.faction-goal-progress')?.textContent).toContain('The Imperial Diwan');
});
