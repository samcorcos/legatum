// @vitest-environment jsdom
import { act } from 'react';
import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { FixtureClient } from '../test/fixtureEngine';
import { itemKey } from '../data';
import { useGame } from '../store';
import type { PlayerView, UnitView } from '../types';
import { Advisor, targetElement } from './Advisor';
import { useAdvisor } from './advisorModel';
import { CityPanel } from './CityPanel';
import { UnitPanel } from './UnitPanel';

let view: PlayerView;
let client: FixtureClient;
beforeEach(async () => {
  vi.stubGlobal('matchMedia', () => ({ matches: false, addEventListener: () => {}, removeEventListener: () => {} }));
  vi.stubGlobal('ResizeObserver', class { observe() {} disconnect() {} });
  vi.spyOn(HTMLElement.prototype, 'getClientRects').mockImplementation(() => [new DOMRect(0, 0, 100, 30)] as unknown as DOMRectList);
  client = new FixtureClient();
  await client.newGame(await client.standardSetup('4545', 'Athens', 'India', 'Ada', 'Bo'));
  view = client.view(0);
  act(() => {
    useGame.setState(useGame.getInitialState(), true);
    useGame.setState({ client, views: [view], seat: 0, humans: [view.me], orders: [[]], errors: [[]], ready: [false] });
    useAdvisor.setState({ enabled: true, seen: [], skipped: [], skippedTurn: -1 });
  });
});
afterEach(() => { cleanup(); vi.restoreAllMocks(); vi.unstubAllGlobals(); });

const control = (name: 'found' | 'fortify' | 'explore' | 'ranged' | 'build', value?: string) => targetElement({ kind: 'control', name, value });

it('finds real unit action buttons on mounted panels', () => {
  const original = view.units.find((u) => u.owner === view.me && u.military)!;
  const unit: UnitView = { ...original, kind: 'Spearman', military: true, ranged: 3, range: 2 };
  const mounted = render(<div className="unit-panel"><UnitPanel unit={unit} view={view} /></div>);
  expect(control('fortify')?.textContent).toContain('Fortify');
  expect(control('ranged')?.textContent).toContain('Ranged Attack');
  mounted.rerender(<div className="unit-panel"><UnitPanel unit={{ ...unit, kind: 'Settler', military: false, ranged: 0 }} view={view} /></div>);
  expect(control('found')?.textContent).toContain('Found City');
  expect(control('fortify')?.textContent).toContain('Wait');
  mounted.rerender(<div className="unit-panel"><UnitPanel unit={{ ...unit, kind: 'Scout', military: false, ranged: 0 }} view={view} /></div>);
  expect(control('explore')?.textContent).toContain('Auto-explore');
});

it('finds Build on the chosen item in a mounted city panel', () => {
  const city = view.cities.find((c) => c.owner === view.me)!;
  const item = city.detail!.options[0].item;
  render(<CityPanel city={city} view={view} />);
  expect(control('build', itemKey(item))).toBeInstanceOf(HTMLButtonElement);
  expect(control('build', itemKey(item))?.getAttribute('aria-label')).toMatch(/^Build /);
});

it('turns off, moves on, and pauses for busy, ready and blocking overlays', async () => {
  render(<div className="game"><Advisor /></div>);
  expect(screen.getByTestId('advisor').dataset.advisorId).toBe('welcome');
  act(() => useAdvisor.getState().markSeen('welcome'));
  const first = screen.getByTestId('advisor').dataset.advisorId;
  fireEvent.click(screen.getByRole('button', { name: 'Not now' }));
  expect(screen.queryByTestId('advisor')?.getAttribute('data-advisor-id')).not.toBe(first);
  const pauses: Record<string, object> = {
    busy: { busy: 'working' }, ready: { ready: [true] },
  };
  for (const patch of Object.values(pauses)) {
    act(() => useGame.setState(patch));
    expect(screen.queryByTestId('advisor')).toBeNull();
    act(() => useGame.setState({ busy: null, ready: [false] }));
    expect(screen.queryByTestId('advisor')).not.toBeNull();
  }
  const overlay = document.body.appendChild(document.createElement('div'));
  overlay.setAttribute('data-game-overlay', '');
  await act(async () => { await Promise.resolve(); });
  expect(screen.queryByTestId('advisor')).toBeNull();
  overlay.remove();
  await act(async () => { await Promise.resolve(); });
  expect(screen.queryByTestId('advisor')).not.toBeNull();
  fireEvent.click(screen.getByRole('button', { name: 'Turn off' }));
  expect(screen.queryByTestId('advisor')).toBeNull();
});
