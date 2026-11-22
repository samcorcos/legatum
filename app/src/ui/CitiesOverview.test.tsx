// @vitest-environment happy-dom
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { FixtureClient } from '../test/fixtureEngine';
import { useGame } from '../store';
import type { PlayerView } from '../types';
import { CitiesOverview } from './CitiesOverview';

let host: HTMLDivElement;
let root: Root;
let map: HTMLCanvasElement;

beforeEach(() => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  useGame.setState(useGame.getInitialState(), true);
  localStorage.removeItem('legatum.cities-sort.v1');
  host = document.body.appendChild(document.createElement('div'));
  root = createRoot(host);
  map = document.body.appendChild(document.createElement('canvas'));
  map.dataset.gameMap = '';
  map.tabIndex = 0;
});

afterEach(() => {
  act(() => root.unmount());
  host.remove(); map.remove(); vi.unstubAllGlobals();
});

async function citiesView(): Promise<PlayerView> {
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('42', 'India', 'Byzantium', 'Ada', 'Bo'));
  // The opening capital is ready for production on turn 1.
  const view = structuredClone(client.view(0));
  const first = view.cities.find((city) => city.owner === view.me)!;
  // These copies give the table several real city rows without depending on the map seed.
  const second = structuredClone(first); second.id = first.id + 100; second.name = 'Second city'; second.population = 3;
  const third = structuredClone(first); third.id = first.id + 200; third.name = 'Third city'; third.population = 2;
  for (const city of [second, third]) {
    city.detail!.options = structuredClone(first.detail!.options);
    city.detail!.unavailable = structuredClone(first.detail!.unavailable);
  }
  view.cities.push(second, third);
  // Deliberately distinct values make the sort direction visible and deterministic.
  first.detail!.yields.total.production = 2;
  second.detail!.yields.total.production = 8;
  third.detail!.yields.total.production = 5;
  return view;
}

async function openOverview(view?: PlayerView) {
  const v = view ?? await citiesView();
  useGame.setState({ views: [v], humans: [v.me], seat: 0, orders: [[]], citiesOpen: true });
  act(() => root.render(<CitiesOverview />));
  return v;
}

const rowIds = () => [...host.querySelectorAll<HTMLElement>('[data-testid="city-row"]')].map((row) => Number(row.dataset.city));

it('sorts a column ascending and reverses it on the next click', async () => {
  await openOverview();
  const city = [...host.querySelectorAll<HTMLButtonElement>('.co-sort')].find((button) => button.textContent?.includes('City'))!;
  act(() => city.click());
  const ascending = rowIds();
  expect(ascending).toHaveLength(3);
  const names = ascending.map((id) => host.querySelector(`[data-city="${id}"] .co-city`)!.textContent!);
  expect(names).toEqual([...names].sort((a, b) => a.localeCompare(b, undefined, { numeric: true })));
  act(() => city.click());
  expect(rowIds()).toEqual([...ascending].reverse());
});

it('selects and focuses a city, then closes the overview', async () => {
  const view = await openOverview();
  const target = view.cities.find((city) => city.name === 'Second city')!;
  act(() => host.querySelector<HTMLElement>(`tr[data-city="${target.id}"]`)!.click());
  await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()));
  expect(useGame.getState()).toMatchObject({ citiesOpen: false, selection: { kind: 'city', id: target.id } });
  expect(document.activeElement).toBe(map);
});

it('keeps the other rows in order after a production choice', async () => {
  const view = await openOverview();
  const idsBefore = rowIds();
  const targetId = idsBefore[1];
  const city = view.cities.find((entry) => entry.id === targetId)!;
  const item = city.detail!.options.find((option) => option.item.kind === 'Building')!.item;
  act(() => useGame.getState().editOrders((orders) => [...orders, { type: 'SetProduction', city: targetId, item }]));
  // A pending choice changes the table's build column, but ties remain name-stable.
  const after = rowIds();
  expect(after).toContain(targetId);
  expect(after.filter((id) => id !== targetId)).toEqual(idsBefore.filter((id) => id !== targetId));
});

it('sets different focuses in bulk and stops without changing production', async () => {
  const view = await openOverview();
  const ids = rowIds().slice(0, 2);
  for (const id of ids) act(() => host.querySelector<HTMLInputElement>(`[data-city="${id}"] input[type=checkbox]`)!.click());
  const focus = host.querySelector<HTMLSelectElement>('select[aria-label="Building focus for selected cities"]')!;
  act(() => { focus.value = 'Gold'; focus.dispatchEvent(new Event('change', { bubbles: true })); });
  act(() => [...host.querySelectorAll<HTMLButtonElement>('.co-automation button')].find((b) => b.textContent === 'Automate')!.click());
  for (const id of ids) expect(useGame.getState().orders[0]).toContainEqual({ type: 'SetCityAutomation', city: id, focus: 'Gold' });
  expect(host.querySelectorAll('.co-auto-badge')).toHaveLength(2);
  expect(host.querySelectorAll('[data-testid="city-auto-wait"]')).toHaveLength(2);
  act(() => [...host.querySelectorAll<HTMLButtonElement>('.co-automation button')].find((b) => b.textContent === 'Stop automating')!.click());
  for (const id of ids) expect(useGame.getState().orders[0]).toContainEqual({ type: 'SetCityAutomation', city: id, focus: null });
  expect(useGame.getState().orders[0].every((o) => o.type !== 'SetProductionQueue' && o.type !== 'SetProduction')).toBe(true);
  expect(view.cities).toHaveLength(4);
});
