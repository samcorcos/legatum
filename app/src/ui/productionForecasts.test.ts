// @vitest-environment happy-dom
import { act, createElement, Fragment } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { FixtureClient } from '../test/fixtureEngine';
import { spawnUnit } from '../test/fixtureEngine';
import { neighbors, tileIndex } from '../hex';
import { useGame } from '../store';
import { TopBar } from './TopBar';
import { SeatBar } from './SeatBar';
import { CityPanel } from './CityPanel';
import { TileCard } from './TileCard';
import { UnitPanel } from './UnitPanel';

let client: FixtureClient;
let container: HTMLDivElement;
let root: Root;
beforeEach(async () => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  client = new FixtureClient();
  await client.newGame(await client.standardSetup('42', 'Switzerland', 'India', 'Ada', 'Bo'));
  client.endTurn({});
  // This suite isolates production prompts; incoming diplomacy has its own primary-action tests.
  client.debugState().diplomacy!.proposals = [];
  useGame.setState(useGame.getInitialState(), true);
  container = document.createElement('div');
  document.body.append(container);
  root = createRoot(container);
});
afterEach(async () => {
  await act(async () => root.unmount());
  container.remove();
  vi.unstubAllGlobals();
});

async function renderForecasts() {
  const save = client.save();
  const view = client.view(0);
  expect(client.save(), 'view projections must not change the demo state').toBe(save);
  const city = view.cities.find((c) => c.owner === 0)!;
  await act(async () => {
    useGame.setState({ client, views: [view], humans: [0], seat: 0, errors: [[]], ready: [false],
      orders: [[{ type: 'SetResearch', tech: 'BronzeWorking' }, ...view.units.filter((u) => u.owner === view.me).map((u) => ({ type: 'SkipUnit' as const, unit: u.id }))]] });
    root.render(createElement(Fragment, null,
      createElement(TopBar), createElement(SeatBar), createElement(CityPanel, { city, view }),
      createElement(TileCard, { view, hex: city.pos, anchor: { x: 0, y: 0, r: 20, avoid: [] } })));
  });
  return city;
}

it('stacks the closed-territory warning with unit hints without promising healing', async () => {
  const s = client.debugState();
  const unit = s.units.find((u) => u.owner === 0 && u.kind === 'Spearman')!;
  unit.hp = 50;
  s.owner[tileIndex(unit.pos, s.width, s.height)] = 2;
  const view = client.view(0);
  await act(async () => {
    useGame.setState({ client, views: [view], humans: [0], seat: 0, orders: [[]], errors: [[]] });
    root.render(createElement(UnitPanel, { unit: view.units.find((u) => u.id === unit.id)!, view }));
  });
  const hints = container.querySelector('.up-hints')!;
  expect(hints.textContent).toContain('Closed foreign territory: no attacks, healing or fortification.');
  // Sight is a stat beside HP and Moves, not a note under the map.
  expect(container.querySelector('.up-stats')!.textContent).toContain('Sight');
  const move = [...container.querySelectorAll<HTMLButtonElement>('button.act')].find((b) => b.textContent === 'Move or attack')!;
  expect(move).toBeDefined();
  // The explicit movement control carries the help; the warning stack stays about this unit's situation.
  await act(async () => move.focus());
  expect(document.getElementById(move.getAttribute('aria-describedby')!)?.textContent).toContain('Choose a tile on the map');
  expect(hints.textContent).not.toContain('Rest here:');
  expect(container.querySelectorAll('.up-hints')).toHaveLength(1);
});

it('renders bankruptcy in the gold readout and review list, and waste in both production prompts', async () => {
  const s = client.debugState();
  const c = s.cities.find((c) => c.owner === 0)!;
  c.prod = 60;
  s.players[0].gold = 0;
  for (let i = 0; i < 9; i++) spawnUnit(s, 0, 'Pikeman', c.pos, false);
  const city = await renderForecasts();
  const warning = useGame.getState().views![0].economy.bankruptcy!;
  expect(container.querySelector('.gold-warning')?.textContent).toContain(`Treasury −${-warning.gold.total} Gold next turn Reisläufer will be disbanded`);
  expect(container.querySelector('.turn-review')?.textContent).toContain(`Next turn −${-warning.gold.total} Gold: Reisläufer #${warning.disband!.unit} will be disbanded`);
  // The coming disband is the first decision; disbanding a unit deals with it.
  expect(container.querySelector('.cta')?.textContent).toContain(`Treasury runs dryLose Reisläufer next turn`);
  await act(async () => useGame.getState().editOrders((orders) => [...orders.filter((o) => !('unit' in o && o.unit === warning.disband!.unit)), { type: 'DisbandUnit', unit: warning.disband!.unit! }]));
  expect(container.querySelector('.cta')?.textContent).toContain(`${city.detail!.idle_production_waste} Production wasted next turn`);
  expect(container.querySelector('.city-panel .production-warning')?.textContent).toContain(`${city.detail!.idle_production_waste} Production will be wasted`);
  await act(async () => useGame.getState().editOrders((orders) => [...orders,
    { type: 'SetProduction', city: city.id, item: { kind: 'Unit', unit: 'Spearman' } }]));
  expect(container.querySelector('.city-panel .production-warning')).toBeNull();
  expect(container.querySelector('.cta')?.textContent).not.toContain('Choose production');
});

it('renders the population stall on the current build and the city tooltip', async () => {
  // A city building a Settler that shrinks below population 2 next turn (core ProductionStall).
  const c = client.debugState().cities.find((c) => c.owner === 0)!;
  c.producing = { kind: 'Unit', unit: 'Settler' };
  c.prod = 1_000;
  c.population = 2;
  const city = await renderForecasts();
  city.detail!.production_stall = { kind: 'Population', required: 2, available: 1 };
  const view = useGame.getState().views![0];
  await act(async () => root.render(createElement(Fragment, null,
    createElement(CityPanel, { city, view }),
    createElement(TileCard, { view, hex: city.pos, anchor: { x: 0, y: 0, r: 20, avoid: [] } }))));
  expect(container.querySelector('.city-panel')?.textContent).toContain('Stalled');
  expect(container.querySelector('.city-panel .production-warning')?.textContent).toContain('Settler needs population 2');
  expect(container.querySelector('.tilecard .production-warning')?.textContent).toContain('Settler needs population 2');
});

it('renders a blocked spawn with an action to clear it', async () => {
  const s = client.debugState();
  const c = s.cities.find((c) => c.owner === 0)!;
  c.producing = { kind: 'Unit', unit: 'Spearman' };
  c.prod = 1_000;
  for (const pos of [c.pos, ...neighbors(c.pos)]) spawnUnit(s, 0, 'Spearman', pos, false);
  await renderForecasts();
  expect(container.querySelector('.city-panel .production-warning')?.textContent).toContain('Move a blocking unit to make room');
  expect(container.querySelector('.tilecard .production-warning')?.textContent).toContain('no free spawn tile');
});
