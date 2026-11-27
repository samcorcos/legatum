// @vitest-environment jsdom
import { cleanup, render } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { FixtureClient } from '../test/fixtureEngine';
import { itemKey } from '../data';
import { useGame } from '../store';
import type { PlayerView, UnitView } from '../types';
import { targetElement } from './Advisor';
import { CityPanel } from './CityPanel';
import { UnitPanel } from './UnitPanel';
import { AirWing } from './AirWing';
import { PingControl } from './PingControl';
import { CitiesButton } from './CitiesOverview';
import { LensSwitcher } from '../map/LensSwitcher';
let view: PlayerView;
beforeEach(async () => {
  vi.stubGlobal('matchMedia', () => ({ matches: false, addEventListener: () => {}, removeEventListener: () => {} }));
  vi.stubGlobal('ResizeObserver', class { observe() {} disconnect() {} });
  vi.spyOn(HTMLElement.prototype, 'getClientRects').mockImplementation(() => [new DOMRect(0, 0, 100, 30)] as unknown as DOMRectList);
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('4545', 'Athens', 'India', 'Ada', 'Bo'));
  view = client.view(0);
  useGame.setState(useGame.getInitialState(), true);
  useGame.setState({ client, views: [view], humans: [view.me], orders: [[]], errors: [[]], ready: [false] });
});
afterEach(() => { cleanup(); vi.restoreAllMocks(); vi.unstubAllGlobals(); });
it('rings the real Automate and production queue controls', () => {
  const city = view.cities.find((c) => c.owner === view.me)!;
  render(<CityPanel city={city} view={view} />);
  expect(targetElement({ kind: 'control', name: 'automate' })?.getAttribute('aria-label')).toBe(`Automate ${city.name}`);
  const item = city.detail!.options.find((o) => o.item.kind === 'Building')!.item;
  expect(targetElement({ kind: 'control', name: 'queueBuild', value: itemKey(item) })?.getAttribute('aria-label')).toMatch(/^Queue /);
});
it('rings Heal and Auto-fire on real unit controls', () => {
  const city = view.cities.find((c) => c.owner === view.me)!;
  const original = view.units.find((u) => u.owner === view.me)!;
  const unit: UnitView = { ...original, kind: 'Archer', military: true, ranged: 3, range: 2, hp: 60, pos: city.pos, standing_order: null };
  render(<div className="unit-panel"><UnitPanel unit={unit} view={view} /></div>);
  expect(targetElement({ kind: 'control', name: 'heal' })?.textContent).toContain('Heal');
  expect(targetElement({ kind: 'control', name: 'autoFire' })?.textContent).toContain('Auto-fire from city');
});

it('rings modernization, aircraft recon and a supported teammate ping', () => {
  const city = view.cities.find((c) => c.owner === view.me)!;
  const original = view.units.find((u) => u.owner === view.me)!;
  view.research.known.push('Feudalism'); view.economy.gold = 1000;
  const unit: UnitView = { ...original, kind: 'Spearman', military: true, pos: city.pos, upgrade_to: 'Pikeman', upgrade_cost: 80, upgrade_breakdown: null, siege_offer: { price: { total: 80, parts: [['Siege', 80]] }, blocker: null } };
  const mounted = render(<div className="unit-panel"><UnitPanel unit={unit} view={view} /></div>);
  expect(targetElement({ kind: 'control', name: 'upgrade' })?.textContent).toContain('Upgrade to');
  expect(targetElement({ kind: 'control', name: 'siege' })?.textContent).toContain('Siege');
  view.aircraft = [{ id: 500, owner: view.me, kind: 'Biplane', city: city.id, hp: 100 }];
  mounted.rerender(<AirWing city={city} view={view} planeId={500} />);
  expect(targetElement({ kind: 'control', name: 'airRecon' })?.textContent).toContain('Recon');
  useGame.setState({ canPing: () => true });
  mounted.rerender(<PingControl />);
  expect(targetElement({ kind: 'control', name: 'ping' })?.textContent).toContain('Ping');
});

it('rings Cities and the map chooser before its hidden Yields control', () => {
  const mounted = render(<><CitiesButton /><LensSwitcher lens="default" onChange={() => {}} sites={[]} onInspect={() => {}} settlerId={null} players={view.players} me={view.me} collapsed /></>);
  expect(targetElement({ kind: 'control', name: 'cities' })?.textContent).toContain('Cities');
  expect(targetElement({ kind: 'control', name: 'mapYields' })?.className).toContain('lens-toggle');
  mounted.rerender(<LensSwitcher lens="default" onChange={() => {}} sites={[]} onInspect={() => {}} settlerId={null} players={view.players} me={view.me} />);
  expect(targetElement({ kind: 'control', name: 'mapYields' })?.getAttribute('data-overlay')).toBe('yields');
});
