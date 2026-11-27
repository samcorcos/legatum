// @vitest-environment jsdom
import { cleanup, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { FixtureClient } from '../test/fixtureEngine';
import type { PlayerView, TechId } from '../types';
import { TechTree } from './TechTree';

let view: PlayerView;
beforeEach(async () => {
  vi.stubGlobal('matchMedia', () => ({ matches: false, addEventListener: () => {}, removeEventListener: () => {} }));
  vi.stubGlobal('ResizeObserver', class { observe() {} disconnect() {} });
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('4545', 'Athens', 'India', 'Ada', 'Bo'));
  view = client.view(0);
});
afterEach(() => { cleanup(); vi.unstubAllGlobals(); });

const tree = (goal: TechId) => render(<TechTree view={view} faction={view.players.find((p) => p.id === view.me)!.faction} queue={[]} schedule={[]}
  goal={goal} locked={false} onResearchNow={() => {}} onQueue={() => {}} onRemove={() => {}} onGoal={() => {}} />);

it('a goal past the last turn at today’s Science says so instead of naming a turn the game never reaches', () => {
  const far = view.research.catalog[view.research.catalog.length - 1].tech;
  view.turn_limit = 300;
  view.economy.science_per_turn.total = 2;
  tree(far);
  expect(screen.getByTestId('tree-goal').textContent).toContain('The game ends before you get there at your current Science.');
  cleanup();
  // A game with no configured limit still ends at the default turn (rules 62).
  view.turn_limit = null;
  tree(far);
  expect(screen.getByTestId('tree-goal').textContent).toContain('The game ends before you get there at your current Science.');
  cleanup();
  view.turn_limit = 1_000_000;
  tree(far);
  expect(screen.getByTestId('tree-goal').textContent).toMatch(/About turn \d+/);
});
