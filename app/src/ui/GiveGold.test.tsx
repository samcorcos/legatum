// @vitest-environment happy-dom
// The co-op Gold gift: a teammate's row in the diplomacy panel queues `GiveGold` with a forecast of
// both treasuries, a non-teammate gets no row, and the report names the gift for both players.
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { FixtureClient } from '../test/fixtureEngine';
import { useGame } from '../store';
import { contextFromViews, formatEvents } from '../format';
import { describeOrder } from '../orders';
import { withThe } from '../resources';
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
  // Seat 1 (the Aztecs) joins our team; the rest stay rivals. No rival plays the Aztecs, so the name has no " · seat".
  view.players = view.players.map((p) => p.id === 1 ? { ...p, faction: 'Aztecs', team: view.team, gold: 30 } : p.faction === 'Aztecs' ? { ...p, faction: 'Rome' } : p);
  view.economy = { ...view.economy, gold: 120 };
  useGame.setState({ client, views: [view], humans: [0], orders: [[]], errors: [[]], ready: [false], seat: 0 });
});
afterEach(() => { act(() => root.unmount()); container.remove(); vi.unstubAllGlobals(); });

const input = () => container.querySelector<HTMLInputElement>('input[type=number]')!;
const type = (value: string) => act(() => {
  const set = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value')!.set!;
  set.call(input(), value);
  input().dispatchEvent(new Event('input', { bubbles: true }));
});

it('gives Gold to a teammate only, named as the empire rows name it, with the forecast of both treasuries', () => {
  act(() => root.render(<TeamGifts view={view} />));
  const rows = container.querySelectorAll('[data-testid=gift-row]');
  expect(rows).toHaveLength(1);
  expect(rows[0].querySelector('.empire-pick-name')!.textContent).toBe('Aztecs');
  expect(input().getAttribute('aria-label')).toBe('Gold to give the Aztecs');
  expect(container.textContent).toContain(GIVE_GOLD_RULE);
  // Default 50: you keep 70, the teammate will have 80.
  expect(container.querySelector('[data-testid=gift-forecast]')!.textContent).toBe('You keep 70 Gold. The Aztecs will have 80 Gold.');
  act(() => container.querySelector<HTMLButtonElement>('[data-testid=give-gold]')!.click());
  const orders = useGame.getState().orders[0];
  expect(orders).toEqual([{ type: 'GiveGold', to: 1, amount: 50 }]);
  expect(describeOrder(orders[0], view)).toBe('Give 50 Gold to the Aztecs');
  expect(container.querySelector('[data-testid=gift-queued]')!.textContent).toContain('give 50 Gold to the Aztecs');
});

it('an empty field stays empty and an amount out of range is an alert', () => {
  act(() => root.render(<TeamGifts view={view} />));
  type('');
  expect(input().value).toBe('');
  expect(container.querySelector<HTMLButtonElement>('[data-testid=give-gold]')!.disabled).toBe(true);
  type('9999');
  const alert = container.querySelector('[data-testid=gift-forecast]')!;
  expect(alert.getAttribute('role')).toBe('alert');
  expect(alert.classList.contains('order-error')).toBe(true);
  expect(alert.textContent).toBe('Give between 1 and 120 Gold.');
});

it('the report names the gift for the giver and the teammate, with the article a people takes', () => {
  const ctx0 = contextFromViews(null, view);
  const e = { type: 'GoldGiven' as const, from: 0, to: 1, amount: 120 };
  expect(formatEvents([e], ctx0)[0].text).toBe('You gave the Aztecs 120 Gold');
  expect(formatEvents([{ ...e, from: 1, to: 0, amount: 35 }], ctx0)[0].text).toBe('The Aztecs gave you 35 Gold');
  expect(withThe('Switzerland')).toBe('Switzerland');
  expect(withThe('Athens')).toBe('Athens');
  expect(withThe('Mongols')).toBe('the Mongols');
});

it('shows nothing without a living teammate', () => {
  const alone = { ...view, players: view.players.map((p) => p.id === 1 ? { ...p, team: 99 } : p) };
  act(() => root.render(<TeamGifts view={alone} />));
  expect(container.querySelector('[data-testid=team-gifts]')).toBeNull();
});
