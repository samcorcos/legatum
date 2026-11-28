// @vitest-environment jsdom
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { FixtureClient, meetAll } from '../test/fixtureEngine';
import { useGame } from '../store';
import { CelebrationProvider } from './MomentLane';
import { PeaceTalks, WarPlate } from './PeaceTalks';
import type { PlayerView } from '../types';

let host: HTMLDivElement, root: Root, client: FixtureClient;
const render = () => act(() => root.render(<CelebrationProvider><PeaceTalks /></CelebrationProvider>));
const press = (label: string) => act(() => [...host.querySelectorAll<HTMLButtonElement>('button')].find((b) => b.textContent?.startsWith(label))!.click());

beforeEach(async () => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  vi.stubGlobal('matchMedia', () => ({ matches: false, addEventListener() {}, removeEventListener() {} }));
  HTMLDialogElement.prototype.showModal = function showModal(this: HTMLDialogElement) { this.setAttribute('open', ''); };
  HTMLDialogElement.prototype.close = function close(this: HTMLDialogElement) { this.removeAttribute('open'); };
  host = document.body.appendChild(document.createElement('div'));
  root = createRoot(host);
  useGame.setState(useGame.getInitialState(), true);
  client = new FixtureClient();
  await client.newGame(await client.standardSetup('700040', 'India', 'Byzantium', 'Ada', 'Bo'));
  meetAll(client.debugState());
  const relation = client.debugState().diplomacy!.relations.find((r) => r.a === 0 && r.b === 1)!;
  Object.assign(relation, { state: 'War', war_id: 1, effective_turn: 1, version: 1, declarers: [1] });
  client.debugState().diplomacy!.proposals = [{ id: 7, kind: 'Peace', from: 1, to: 0, proposer: 2, created_turn: 1, respond_by: 3, relation_version: 1, consents: [1] }];
  const view = client.view(0);
  useGame.setState({ client, views: [view], humans: [0], orders: [[]], errors: [[]], ready: [false], seat: 0, diplomacyOpen: false });
});
afterEach(() => { act(() => root.unmount()); host.remove(); vi.unstubAllGlobals(); });

it('opens a pending Peace offer once, lets the seat defer, and queues either answer', () => {
  render();
  expect(host.querySelector('[data-testid="peace-talks"]')).not.toBeNull();
  press('Decide later');
  expect(host.querySelector('[data-testid="peace-talks"]')).toBeNull();
  render();
  expect(host.querySelector('[data-testid="peace-talks"]')).toBeNull();
  act(() => useGame.setState({ views: [client.view(0), client.view(1)], humans: [0, 1], orders: [[], []], errors: [[], []], ready: [false, false], seat: 1 }));
  expect(host.querySelector('[data-testid="peace-talks"]')).not.toBeNull();
  press('Decide later');
  act(() => useGame.setState({ seat: 0 }));
  expect(host.querySelector('[data-testid="peace-talks"]')).toBeNull();
  act(() => useGame.getState().set({ diplomacyTakeover: { mode: 'offer', id: 7 } }));
  press('Accept Peace');
  expect(useGame.getState().orders[0]).toContainEqual({ type: 'RespondToProposal', proposal: 7, accept: true });
  act(() => useGame.getState().editOrders(() => []));
  act(() => useGame.getState().set({ diplomacyTakeover: { mode: 'offer', id: 7 } }));
  press('Refuse, fight on');
  expect(useGame.getState().orders[0]).toContainEqual({ type: 'RespondToProposal', proposal: 7, accept: false });
});

it('offers only signed one-time Gold, caps it, quotes the 3-turn answer window, and refuses an unaffordable payment', () => {
  client.debugState().diplomacy!.proposals = [];
  client.debugState().setup.deals = true;
  client.debugState().players[0].gold = 30;
  const view = client.view(0);
  useGame.setState({ views: [view], diplomacyTakeover: { mode: 'terms', team: 1 } });
  render();
  expect(host.textContent).toContain(`has up to 3 turns to answer (by turn ${view.turn + 3}).`);
  expect(document.activeElement?.textContent).toMatch(/^Send terms to /);
  press('You pay');
  expect(host.querySelector<HTMLInputElement>('input[aria-label="Gold you pay"]')?.max).toBe('999');
  expect(host.textContent).toContain('You have 30 Gold, so offer at most 30.');
  expect([...host.querySelectorAll<HTMLButtonElement>('button')].find((b) => b.textContent?.startsWith('Send terms'))!.disabled).toBe(true);
  for (let i = 0; i < 3; i++) act(() => host.querySelector<HTMLButtonElement>('button[aria-label="10 Gold less"]')!.click());
  expect(host.textContent).not.toContain('offer at most');
  press('Send terms');
  expect(useGame.getState().orders[0]).toContainEqual({ type: 'ProposeTreaty', to: 1, kind: 'Peace', gold: 20 });
  act(() => useGame.getState().editOrders(() => []));
  act(() => useGame.getState().set({ diplomacyTakeover: { mode: 'terms', team: 1 } }));
  render();
  press('You ask');
  press('Send terms');
  expect(useGame.getState().orders[0]).toContainEqual({ type: 'ProposeTreaty', to: 1, kind: 'Peace', gold: -50 });
});

it('shows the core refusal on Peace terms, disables sending, and focuses Cancel instead of disabled Send', () => {
  const view = useGame.getState().views![0];
  const action = view.diplomacy.actions.find((a) => a.order.type === 'ProposeTreaty' && a.order.kind === 'Peace' && a.order.to === 1);
  expect(action?.blocked_reason).toContain('outstanding proposal');
  useGame.setState({ diplomacyTakeover: { mode: 'terms', team: 1 } });
  render();
  expect(host.textContent).toContain('outstanding proposal');
  const send = [...host.querySelectorAll<HTMLButtonElement>('button')].find((b) => b.textContent?.startsWith('Send terms'))!;
  const cancel = [...host.querySelectorAll<HTMLButtonElement>('button')].find((b) => b.textContent === 'Cancel')!;
  expect(send.disabled).toBe(true);
  expect(document.activeElement).toBe(cancel);
});

it('isolates focus inside the modal dialog on Tab and Shift+Tab and restores focus to the opener on close', () => {
  const showModal = vi.spyOn(HTMLDialogElement.prototype, 'showModal');
  const opener = document.body.appendChild(document.createElement('button'));
  opener.textContent = 'Map HUD control';
  opener.focus();
  expect(document.activeElement).toBe(opener);

  render();
  const dialog = host.querySelector<HTMLDialogElement>('dialog[data-testid="peace-talks"]')!;
  expect(dialog).not.toBeNull();
  expect(showModal).toHaveBeenCalledOnce();

  const buttons = [...dialog.querySelectorAll<HTMLButtonElement>('button:not([disabled])')];
  expect(buttons.length).toBeGreaterThanOrEqual(2);
  const first = buttons[0], last = buttons[buttons.length - 1];

  // Shift+Tab on the first button wraps to the last button inside the dialog, never to the opener.
  first.focus();
  act(() => { window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Tab', shiftKey: true, bubbles: true, cancelable: true })); });
  expect(document.activeElement).toBe(last);
  expect(dialog.contains(document.activeElement)).toBe(true);

  // Tab on the last button wraps back to the first button inside the dialog.
  last.focus();
  act(() => { window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Tab', bubbles: true, cancelable: true })); });
  expect(document.activeElement).toBe(first);
  expect(dialog.contains(document.activeElement)).toBe(true);

  // Closing the plate restores focus to the opener.
  press('Decide later');
  expect(host.querySelector('dialog[data-testid="peace-talks"]')).toBeNull();
  expect(document.activeElement).toBe(opener);
  opener.remove();
  showModal.mockRestore();
});

it('the War plate names defenders and calls, and keeps Peace on Return', () => {
  client.debugState().diplomacy!.proposals = [];
  const relation = client.debugState().diplomacy!.relations.find((r) => r.a === 0 && r.b === 1)!;
  relation.state = 'Peace';
  const view = client.view(0) as PlayerView;
  const action = view.diplomacy.actions.find((a) => a.order.type === 'DeclareWar' && a.order.against === 1 && !a.order.breach)!;
  action.new_wars = [{ team: 2, enemy: 0 }];
  action.calls = [{ team: 2, enemy: 1 }];
  const confirm = vi.fn(), close = vi.fn();
  act(() => root.render(<WarPlate view={view} team={1} onConfirm={confirm} onClose={close} />));
  expect(host.textContent).toContain('would defend them and be at War with you at once');
  expect(host.textContent).toContain('would be asked to join this war');
  const keep = [...host.querySelectorAll<HTMLButtonElement>('button')].find((b) => b.textContent === 'Keep the Peace')!;
  expect(document.activeElement).toBe(keep);
  act(() => keep.click());
  expect(close).toHaveBeenCalledOnce();
  expect(confirm).not.toHaveBeenCalled();
  action.blocked_reason = 'Peace guaranteed through turn 10';
  act(() => root.render(<WarPlate view={{ ...view }} team={1} onConfirm={confirm} onClose={close} />));
  expect(host.textContent).toContain('Peace guaranteed through turn 10');
  expect([...host.querySelectorAll('button')].some((b) => b.textContent === 'Declare War')).toBe(false);
});
