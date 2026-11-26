// @vitest-environment happy-dom
// A treaty taking effect for your team (Peace, an Alliance, Open Borders), with any deal Gold: one small moment card
// with both empires' crests either side of the treaty emblem, a sting, and no motion under reduced motion.
import { act, StrictMode } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { CelebrationProvider, CounterDelta, MomentLane } from './MomentLane';
import { treatyCelebrationFixture } from '../test/celebrationFixture';
import { useGame } from '../store';
import { cancelFeedback } from '../feel';
import { feedbackForTurn } from '../feel/turnFeedback';
import { momentsForTurn, planMoments, treatyMoment } from './moments';
import { soundHooksForMoments } from './feedback';
import { empireTitle } from '../empires';
import type { Event, PeaceProposal } from '../types';

let root: Root, host: HTMLDivElement;
let reduceMotion = false;
beforeEach(() => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  reduceMotion = false;
  vi.stubGlobal('matchMedia', (query: string) => ({ matches: !(!(query.includes('reduce')) || !(reduceMotion)), media: query, addEventListener() {}, removeEventListener() {}, addListener() {}, removeListener() {} }));
  document.body.innerHTML = '';
  host = document.body.appendChild(document.createElement('div'));
  root = createRoot(host);
  useGame.setState(useGame.getInitialState(), true);
  vi.useFakeTimers();
});
afterEach(() => {
  act(() => root.unmount());
  cancelFeedback();
  useGame.setState(useGame.getInitialState(), true);
  vi.useRealTimers(); vi.restoreAllMocks(); vi.unstubAllGlobals();
});
const render = () => act(() => root.render(<StrictMode><CelebrationProvider><MomentLane /><CounterDelta counter="gold" /></CelebrationProvider></StrictMode>));
const view = () => useGame.getState().views![0];

it('Open Borders with Gold received: a small card with both crests, the emblem and the Gold', async () => {
  await treatyCelebrationFixture('OpenBorders', 40);
  render();
  const card = host.querySelector('.moment-card')!;
  expect(card).not.toBeNull();
  expect(card.getAttribute('data-tier')).toBe('notable');
  const them = empireTitle(view(), 1);
  expect(card.querySelector('.moment-kicker')).toBeNull(); // the title names the event
  expect(card.querySelector('.moment-treaty-gold')!.classList.contains('gain')).toBe(true);
  expect(card.querySelector('h2')!.textContent).toBe(`Open Borders with ${them}`);
  const art = card.querySelector('[data-testid=moment-treaty]')!;
  expect(art.querySelectorAll('.medallion')).toHaveLength(2);
  expect(art.querySelector('[aria-label="Open Borders emblem"]')).not.toBeNull();
  expect(card.querySelector('[data-testid=moment-treaty-gold]')!.textContent).toBe(`+40 Gold from ${them}`);
  expect(card.querySelector('.moment-effect')!.textContent).toBe('Your units may enter each other’s land.');
  // The Gold ticks on the treasury counter, and the card is a short story card, not a held takeover.
  expect(host.querySelector('[data-celebrate-delta=gold]')!.textContent).toBe('+40');
  expect(host.querySelector('[data-celebrate-delta=gold]')!.classList.contains('cost')).toBe(false);
  act(() => vi.advanceTimersByTime(3500));
  expect(host.querySelector('.moment-card')).toBeNull();
});

it('an Alliance plays the major sting, and an offer between two other empires plays none', async () => {
  const events = await treatyCelebrationFixture('Alliance', 0);
  const after = view();
  const moments = useGame.getState().report!.moments![0];
  expect(feedbackForTurn(events, after)).toContain('vpGained');
  expect(soundHooksForMoments(moments, events)).toEqual({ vpGained: 'sting.major' });
  const others: Event[] = [{ type: 'ProposalAccepted', proposal: { ...((events[0] as { proposal: PeaceProposal }).proposal), from: 1, to: 2 }, effective_turn: after.turn }];
  expect(momentsForTurn(others, 0, null, after)).toEqual([]);
  expect(feedbackForTurn(others, after)).not.toContain('vpGained');
  expect(soundHooksForMoments([], others)).toEqual({});
});

it('Gold we paid shows as a payment and lowers the counter; a Peace names its guarantee', async () => {
  await treatyCelebrationFixture('Peace', -60);
  const after = view();
  const events = useGame.getState().report!.events[0];
  const card = treatyMoment(events, 0, 0, after)!;
  expect(card.kicker).toBeUndefined();
  expect(card.tier).toBe('notable');
  expect(card.treaty!.upkeep).toBeUndefined();
  expect(card.title).toBe(`Peace with ${empireTitle(after, 1)}`);
  expect(card.effect).toBe(`No War between you through turn ${after.turn + 9}.`);
  expect(card.treaty!.gold).toEqual({ amount: 60, received: false, text: `−60 Gold to ${empireTitle(after, 1)}` });
  expect(card.deltas).toEqual([{ counter: 'gold', amount: -60 }]);
  expect(card.treaty!.ours.id).toBe(0);
  expect(card.treaty!.theirs.id).toBe(after.players.find((p) => p.team === 1)!.id);
  // Never folded away by a busier turn: it is one of the shown cards when it is the only one.
  expect(planMoments(momentsForTurn(events, 0, null, after)).shown).toHaveLength(1);
});

it('reduced motion stands the card still', async () => {
  reduceMotion = true;
  await treatyCelebrationFixture('Alliance', 0);
  render();
  expect(host.querySelector('.moment-lane')!.classList.contains('reduced')).toBe(true);
  expect(host.querySelector('[aria-label="Alliance emblem"]')).not.toBeNull();
  // An Alliance is the framed big card, and states the upkeep that starts now (one Gold per city, minimum one).
  const card = host.querySelector('.moment-card')!;
  expect(card.getAttribute('data-tier')).toBe('major');
  const cities = Math.max(1, view().cities.filter((c) => c.owner === view().me).length);
  expect(card.querySelector('[data-testid=moment-treaty-upkeep]')!.textContent).toBe(`Upkeep −${cities} Gold a turn`);
});

it('Gold we paid ticks the treasury counter in red', async () => {
  await treatyCelebrationFixture('Peace', -60);
  render();
  const tick = host.querySelector('[data-celebrate-delta=gold]')!;
  expect(tick.textContent).toBe('−60');
  expect(tick.classList.contains('cost')).toBe(true);
  expect(host.querySelector('.moment-treaty-gold')!.classList.contains('cost')).toBe(true);
});
