// @vitest-environment happy-dom
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { FixtureClient } from '../test/fixtureEngine';
import { researchFixture } from '../test/researchFixture';
import { useGame } from '../store';
import { TopBar } from './TopBar';
import { SHOW_DELAY } from './Tip';
import { withV2 } from '../test/vpV2Fixture';
import { decadencePercent } from '../data';

/** A real mouse hover (not touch): opens the hover tooltip after `SHOW_DELAY`, never pins it. */
const hoverIn = (el: Element) => act(() => {
  el.dispatchEvent(new PointerEvent('pointerover', { bubbles: true, pointerType: 'mouse', relatedTarget: document.body }));
});
const hoverOut = (el: Element) => act(() => {
  el.dispatchEvent(new PointerEvent('pointerout', { bubbles: true, pointerType: 'mouse', relatedTarget: document.body }));
});

let container: HTMLDivElement, root: Root;
beforeEach(async () => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  container = document.body.appendChild(document.createElement('div')); root = createRoot(container);
  useGame.setState(useGame.getInitialState(), true);
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('700040', 'India', 'Byzantium', 'Ada', 'Bo'));
  const view = client.view(0);
  useGame.setState({ client, views: [view], humans: [0], orders: [[]], errors: [[]], ready: [false], seat: 0 });
});
afterEach(() => { act(() => root.unmount()); container.remove(); vi.unstubAllGlobals(); });

it('shows Gold and Science totals', () => {
  act(() => root.render(<TopBar />));
  const stats = [...container.querySelectorAll('.tb-stats > .stat')].map((s) => s.className);
  expect(stats.some((c) => c.includes('gold-stat'))).toBe(true);
  expect(stats.some((c) => c.includes('science-stat'))).toBe(true);
  act(() => (container.querySelector('.gold-stat') as HTMLElement).click());
  expect(document.querySelector('[data-testid="desert-note"]')?.textContent).toContain('a unit deserts');
});

it.each([[1, '4000 BC'], [50, '1 BC'], [100, '1500 AD'], [250, '2026 AD'], [300, '2076 AD']])('shows the calendar year beside turn %i, never instead of it', (turn, year) => {
  const view = structuredClone(useGame.getState().views![0]);
  view.turn = turn;
  useGame.setState({ views: [view] });
  act(() => root.render(<TopBar />));
  expect(container.querySelector('[data-testid="turn-number"]')?.textContent).toBe(String(turn));
  expect(container.querySelector('[data-testid="turn-year"]')?.textContent).toBe(`· ${year}`);
  expect(container.querySelector('.turn')?.textContent).toContain(`Turn${turn}`);
});

it.each([[81, false], [274, false], [275, true], [300, true]])('keeps the turn limit out of the bar at turn %i until the last 25 turns: %s', (turn, shown) => {
  const view = structuredClone(useGame.getState().views![0]);
  view.turn = turn; view.turn_limit = 300;
  useGame.setState({ views: [view] });
  act(() => root.render(<TopBar />));
  expect(container.querySelector('[data-testid="turn-of"]')?.textContent).toBe(shown ? 'of 300' : undefined);
});

it('lists each Alliance\'s treaty upkeep in the Gold breakdown as a cost', () => {
  const view = structuredClone(useGame.getState().views![0]);
  view.economy.gold_per_turn = { total: 4, parts: [['Cities', 6], ['Unit upkeep', -1], ['Treaty upkeep: Alliance with Celts', -1]] };
  useGame.setState({ views: [view] });
  act(() => root.render(<TopBar />));
  act(() => (container.querySelector('.gold-stat') as HTMLElement).click());
  const row = [...document.querySelectorAll('.bd-row')].find((r) => r.textContent?.startsWith('Treaty upkeep: Alliance with Celts'))!;
  expect(row).toBeTruthy();
  const value = row.querySelector('span:last-child')!;
  expect(value.textContent).toBe('−1');
  expect(value.className).toBe('neg');
});

it.each(['Athens', 'Byzantium'] as const)("a %s Science hover breaks down Capital, cities, buildings and faction bonus", (faction) => {
  // Athens rename Laboratory to "Listening House"; the hover's generic "Buildings" line
  // must still pick it up, since it classifies by the faction's own display name.
  const f = researchFixture(faction);
  const capital = f.cities.find((c) => c.owner === 0 && c.isCapital)!;
  capital.buildings = ['Laboratory'];
  const client = new FixtureClient();
  client.load(JSON.stringify(f));
  const view = client.view(0);
  useGame.setState({ client, views: [view], humans: [0], orders: [[]], errors: [[]], ready: [false], seat: 0, report: null });
  vi.useFakeTimers();
  try {
    act(() => root.render(<TopBar />));
    hoverIn(container.querySelector('.science-stat')!);
    act(() => { vi.advanceTimersByTime(SHOW_DELAY); });
    const summary = document.querySelector('.science-hover')!;
    expect(summary).toBeTruthy();
    const rows = [...summary.querySelectorAll('.science-hover-row')].map((r) => r.textContent ?? '');
    expect(rows.some((t) => t.startsWith('Capital +'))).toBe(true);
    expect(rows.some((t) => /^\d+ cit(y|ies) \+\d/.test(t))).toBe(true);
    expect(rows.some((t) => t.startsWith('Buildings +'))).toBe(true);
    expect(rows.some((t) => /^Total \+\d/.test(t))).toBe(true);
    // No salvage rows: every faction's Science comes from its cities.
    expect(rows.some((t) => t.includes('Salvage') || t.includes('Knowledge from'))).toBe(false);
  } finally {
    vi.useRealTimers();
  }
});

it('explains opted-in decadence below the Gold and Science breakdowns', () => {
  const view = withV2(structuredClone(useGame.getState().views![0]), { era: 2, lead: 60 });
  view.economy.gold_per_turn = { total: -3, parts: [['Rome', 18]] };
  view.economy.gold = 8;
  useGame.setState({ views: [view] });
  act(() => root.render(<TopBar />));
  act(() => (container.querySelector('.gold-stat') as HTMLElement).click());
  const cut = decadencePercent(60, { decadenceSlope: 45 });
  expect(document.querySelector('[data-testid="overreach-note"]')).toBeNull();
  const note = document.querySelector('[data-testid="decadence-note"]')!.textContent!;
  expect(note).toContain(`your Power is 60% above the average empire’s, so your Production, Gold and Science are cut by ${cut}% and your cities grow ${cut}% slower`);
  expect(note).toContain('The rate is in proportion to your lead: the further ahead you are, the more your people grow complacent.');
  expect(note).toContain('Food itself is never cut, so no city starves');
  expect(document.querySelector('[data-testid="desert-note"]')!.textContent).toContain('a unit deserts');
});

it('names opted-in decadence in the Gold summary', () => {
  const view = withV2(structuredClone(useGame.getState().views![0]), { era: 2, lead: 100 });
  view.economy.gold_per_turn = { total: 2, parts: [['Rome', 2]] };
  useGame.setState({ views: [view] });
  vi.useFakeTimers();
  act(() => root.render(<TopBar />));
  const gold = container.querySelector('.gold-stat') as HTMLElement;
  hoverIn(gold);
  act(() => { vi.advanceTimersByTime(SHOW_DELAY + 5); });
  expect(document.querySelector('.tip-hover')?.textContent).toContain(`Decadence −${decadencePercent(100, { decadenceSlope: 45 })}% of growth, Production, Gold and Science`);
  hoverOut(gold);
  vi.useRealTimers();
  act(() => gold.click());
  expect(document.querySelector('[data-testid="overreach-note"]')).toBeNull();
  expect(document.querySelector('[data-testid="decadence-note"]')!.textContent).toContain(`are cut by ${decadencePercent(100, { decadenceSlope: 45 })}%`);
});
