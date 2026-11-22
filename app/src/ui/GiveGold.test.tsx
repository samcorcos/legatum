// @vitest-environment happy-dom
// The co-op Gold gift: a teammate's row in the diplomacy panel queues `GiveGold` with a forecast of
// both treasuries, a non-teammate gets no row, and the report names the gift for both players.
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { FixtureClient } from '../test/fixtureEngine';
import { useGame } from '../store';
import { contextFromViews, formatEvents } from '../format';
import type { PlayerView } from '../types';
import { TeamGifts, GIVE_GOLD_RULE } from './GiveGold';

let container: HTMLDivElement, root: Root, view: PlayerView;
beforeEach(async () => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  container = document.body.appendChild(document.createElement('div')); root = createRoot(container);
  useGame.setState(useGame.getInitialState(), true);
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('700040', 'India', 'Byzantium', 'Ada', 'Bo'));
  view = structuredClone(client.view(0));
  // Seat 1 joins our team; the rest stay rivals.
  view.players = view.players.map((p) => p.id === 1 ? { ...p, team: view.team, gold: 30 } : p);
  view.economy = { ...view.economy, gold: 120 };
  useGame.setState({ client, views: [view], humans: [0], orders: [[]], errors: [[]], ready: [false], seat: 0 });
});
afterEach(() => { act(() => root.unmount()); container.remove(); vi.unstubAllGlobals(); });

it('gives Gold to a teammate only, with the forecast of both treasuries, and says so in the report', () => {
  act(() => root.render(<TeamGifts view={view} />));
  const rows = container.querySelectorAll('[data-testid=gift-row]');
  expect(rows).toHaveLength(1);
  expect(rows[0].textContent).toContain(view.players[1].name);
  expect(container.textContent).toContain(GIVE_GOLD_RULE);
  // Default 50: you keep 70, the teammate has 80.
  expect(container.querySelector('[data-testid=gift-forecast]')!.textContent).toBe(`You keep 70 Gold; ${view.players[1].name} has 80 Gold next turn, before income and purchases.`);
  act(() => container.querySelector<HTMLButtonElement>('[data-testid=give-gold]')!.click());
  expect(useGame.getState().orders[0]).toEqual([{ type: 'GiveGold', to: 1, amount: 50 }]);
  expect(container.querySelector('[data-testid=gift-queued]')!.textContent).toContain('give 50 Gold');
  // The report line, for the giver and for the teammate.
  const ctx0 = contextFromViews(null, view);
  const ctx1 = { ...ctx0, me: 1 };
  const e = { type: 'GoldGiven' as const, from: 0, to: 1, amount: 50 };
  expect(formatEvents([e], ctx0)[0].text).toMatch(/^You gave \S.* 50 Gold$/);
  expect(formatEvents([e], ctx1)[0].text).toMatch(/ gave you 50 Gold$/);
});

it('shows nothing without a living teammate', () => {
  const alone = { ...view, players: view.players.map((p) => p.id === 1 ? { ...p, team: 99 } : p) };
  act(() => root.render(<TeamGifts view={alone} />));
  expect(container.querySelector('[data-testid=team-gifts]')).toBeNull();
});
