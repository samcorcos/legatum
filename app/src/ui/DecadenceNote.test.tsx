// @vitest-environment happy-dom
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { FixtureClient } from '../test/fixtureEngine';
import { useGame } from '../store';
import { withV2 } from '../test/vpV2Fixture';
import { decadencePercent } from '../data';
import type { CityYields, PlayerView } from '../types';
import { YieldSourceTable } from './bits';
import { ScienceHoverSummary } from './ScienceHover';

let host: HTMLElement, root: Root, view: PlayerView;
beforeEach(async () => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  useGame.setState(useGame.getInitialState(), true);
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('42', 'India', 'Byzantium', 'Ada', 'Bo'));
  client.endTurn({});
  view = withV2(structuredClone(client.view(0)), { era: 2, lead: 60 });
  useGame.setState({ client, views: [view], humans: [0], orders: [[]], errors: [[]], seat: 0 });
  host = document.body.appendChild(document.createElement('div'));
  root = createRoot(host);
});
afterEach(() => { act(() => root.unmount()); host.remove(); vi.unstubAllGlobals(); });

const pct = decadencePercent(60, { decadenceSlope: 45 });
const yields = (): CityYields => ({
  total: { food: 8, production: 8, gold: 5, science: 4 },
  sources: [
    { label: 'Citizens', yields: { food: 8, production: 9, gold: 5, science: 4 } },
    { label: `Decadence −${pct}% of Production: sloth`, yields: { food: 0, production: -1, gold: 0, science: 0 } },
    { label: `Decadence −${pct}% of Science: complacency`, yields: { food: 0, production: 0, gold: 0, science: -1 } },
  ],
  worked: [], focus: 'Balanced', focus_reason: 'default',
} as unknown as CityYields);

it('a city yield breakdown shows each Decadence line with its reason and one note under the table', () => {
  act(() => root.render(<YieldSourceTable yields={yields()} k="production" />));
  expect(host.textContent).toContain(`Decadence −${pct}% of Production: sloth`);
  const note = host.querySelector('[data-testid="decadence-note"]')!.textContent!;
  expect(note).toContain(`your Power is 60% above the average empire’s, so your Production, Gold and Science are cut by ${pct}% and your cities grow ${pct}% slower`);
  expect(note).toContain('no city starves');
  expect(host.querySelectorAll('[data-testid="decadence-note"]')).toHaveLength(1);
  act(() => root.render(<YieldSourceTable yields={yields()} k="science" />));
  expect(host.textContent).toContain(`Decadence −${pct}% of Science: complacency`);
  // A yield with no decadence line (Gold here) has no note.
  act(() => root.render(<YieldSourceTable yields={yields()} k="gold" />));
  expect(host.querySelector('[data-testid="decadence-note"]')).toBeNull();
});

it('Food has no decadence line: its hover points to the growth bar instead', () => {
  act(() => root.render(<YieldSourceTable yields={yields()} k="food" />));
  expect(host.textContent).not.toContain('Decadence −');
  expect(host.querySelector('[data-testid="decadence-food-pointer"]')!.textContent).toBe('Decadence also slows growth: see the city’s growth bar.');
  expect(host.querySelector('[data-testid="decadence-note"]')).toBeNull();
  act(() => useGame.setState({ views: [withV2(structuredClone(view), { era: 2 })] }));
  expect(host.querySelector('[data-testid="decadence-food-pointer"]')).toBeNull();
});

it('no decadence in the view still names the rate from the line itself', () => {
  const flat = withV2(structuredClone(view), { era: 2 });
  act(() => useGame.setState({ views: [flat] }));
  act(() => root.render(<YieldSourceTable yields={yields()} k="production" />));
  expect(host.querySelector('[data-testid="decadence-note"]')!.textContent).toContain(`cut by ${pct}%`);
});

it('the top-bar Science hover lists decadence as its own row, not as a faction bonus', () => {
  const mine = view.cities.find((c) => c.owner === view.me)!;
  mine.detail = { ...mine.detail!, yields: { ...yields(), total: { food: 6, production: 8, gold: 5, science: 4 } } };
  view.economy.science_per_turn = { total: 3, parts: [['Rome', 3]] };
  act(() => useGame.setState({ views: [view] }));
  act(() => root.render(<ScienceHoverSummary view={view} />));
  const rows = [...host.querySelectorAll('.science-hover-row')].map((r) => r.textContent!);
  expect(rows.some((r) => /^Decadence: complacency\s*−1/.test(r))).toBe(true);
  expect(rows.some((r) => r.startsWith('Faction bonus'))).toBe(false);
});
