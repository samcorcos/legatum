// @vitest-environment happy-dom
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { FixtureClient } from '../test/fixtureEngine';
import { turnFlowFixture } from '../test/turnFlowFixture';
import { useGame } from '../store';
import { CityPanel } from './CityPanel';
import { CityList } from './CityList';
import type { Order } from '../types';

let root: Root;
let host: HTMLDivElement;
let map: HTMLCanvasElement;
beforeEach(() => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  useGame.setState(useGame.getInitialState(), true);
  const client = new FixtureClient(); client.load(JSON.stringify(turnFlowFixture()));
  const view = client.view(0), city = view.cities[0];
  useGame.setState({ client, views: [view], humans: [0], ready: [false], errors: [[]],
    orders: [[{ type: 'SkipUnit', unit: 0 }, { type: 'SkipUnit', unit: 4 }]], selection: { kind: 'city', id: city.id } });
  map = document.body.appendChild(document.createElement('canvas'));
  map.dataset.gameMap = ''; map.tabIndex = 0;
  host = document.body.appendChild(document.createElement('div'));
  root = createRoot(host);
  act(() => root.render(<CityPanel city={city} view={view} />));
});
afterEach(() => {
  act(() => root.unmount()); host.remove(); map.remove(); vi.unstubAllGlobals();
});
function choose(detail = 0, shiftKey = false) {
  const button = host.querySelector<HTMLButtonElement>('button.prod')!;
  act(() => { button.focus(); button.dispatchEvent(new MouseEvent('click', { bubbles: true, detail, shiftKey })); });
}

it('sets a distinct building focus without replacing the current production order', () => {
  const select = host.querySelector<HTMLSelectElement>('select[aria-label^="Build automatically"]')!;
  act(() => { select.value = 'Science'; select.dispatchEvent(new Event('change', { bubbles: true })); });
  expect(useGame.getState().orders[0]).toContainEqual({ type: 'SetCityAutomation', city: useGame.getState().views![0].cities[0].id, focus: 'Science' });
  expect(useGame.getState().orders[0].some((o) => o.type === 'SetProduction' || o.type === 'SetProductionQueue')).toBe(false);
  expect(host.textContent).toContain('Your queue goes first. When it empties, the city picks a Science build.');
  // An automated idle city is not asking for a choice: the heading says when it chooses, and the control stays beside the gold line, in view.
  const title = host.querySelector('.cp-choose-title')!;
  expect(title.textContent).toBe('Chooses at Ready · Science focus');
  expect(host.querySelector('.cp-automation-inline select[aria-label^="Build automatically"]')).not.toBeNull();
});

for (const [input, detail] of [['keyboard', 0], ['mouse', 1]] as const) {
  it(`${input} production advances to research and restores map focus for closing the dialog`, () => {
    choose(detail);
    expect(useGame.getState()).toMatchObject({ selection: null, techOpen: true, ready: [false], busy: null });
    expect(document.activeElement).toBe(map);
    expect(useGame.getState().orders[0].some((o) => o.type === 'SetProductionQueue')).toBe(true);
  });

  it(`${input} production visits a remaining unit after the last city decision`, () => {
    act(() => useGame.getState().editOrders(() => [{ type: 'SetResearch', tech: 'BronzeWorking' }]));
    choose(detail);
    expect(useGame.getState()).toMatchObject({ selection: { kind: 'unit', id: 0 }, techOpen: false });
    expect(document.activeElement).toBe(map);
  });

  it(`the last ${input} production decision returns to the map without automatically ending the turn`, () => {
    act(() => useGame.getState().editOrders((orders): Order[] => [...orders, { type: 'SetResearch', tech: 'BronzeWorking' }]));
    choose(detail);
    expect(useGame.getState()).toMatchObject({ selection: null, techOpen: false, ready: [false], busy: null });
    expect(document.activeElement).toBe(map);
  });

  it(`${input} production visits the next idle city before research`, () => {
    const view = useGame.getState().views![0];
    const nextCity = { ...structuredClone(view.cities[0]), id: view.cities[0].id + 1, name: 'Second city' };
    view.cities.push(nextCity);
    choose(detail);
    expect(useGame.getState()).toMatchObject({ selection: { kind: 'city', id: nextCity.id }, techOpen: false });
    expect(document.activeElement).toBe(map);
  });
}

it('mouse Build leaves the city reachable through Cities for inspection, queues and buying', () => {
  const view = useGame.getState().views![0], city = view.cities[0];
  act(() => useGame.getState().editOrders((orders): Order[] => [...orders, { type: 'SetResearch', tech: 'BronzeWorking' }]));
  choose(1);
  expect(useGame.getState().selection).toBeNull();
  act(() => root.render(<CityList view={view} />));
  act(() => host.querySelector<HTMLButtonElement>('.city-list-items button')!.click());
  expect(useGame.getState().selection).toEqual({ kind: 'city', id: city.id });
  act(() => root.render(<CityPanel city={city} view={view} />));
  expect(host.querySelector('button.prod[aria-pressed="true"]')).not.toBeNull();
  expect(host.querySelector('.queue-add')).not.toBeNull();
  expect(host.querySelector('.btn-buy')).not.toBeNull();
});

it('the Cities count is your cities; known foreign cities are counted separately', () => {
  const view = structuredClone(useGame.getState().views![0]);
  const mine = view.cities.filter((c) => c.owner === view.me).length;
  view.cities.push({ ...view.cities[0], id: 999, owner: view.me + 1, name: 'Old Relay', detail: null });
  act(() => root.render(<CityList view={view} />));
  expect(host.querySelector('.city-list summary')!.textContent).toBe(`Cities seen ${mine} · 1 foreign`);
  expect(host.querySelectorAll('.city-list-items button')).toHaveLength(mine + 1);
});

it('Shift-click and Queue keep the city open to continue editing the plan', () => {
  const selection = useGame.getState().selection;
  choose(1, true);
  expect(useGame.getState().selection).toEqual(selection);
  expect(useGame.getState().techOpen).toBe(false);
  act(() => host.querySelector<HTMLButtonElement>('.queue-add')!.click());
  expect(useGame.getState().selection).toEqual(selection);
  expect(useGame.getState().techOpen).toBe(false);
});
