// @vitest-environment happy-dom
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { FixtureClient } from '../test/fixtureEngine';
import { useGame } from '../store';
import type { PlayerView, ProductionItem } from '../types';
import { WhereToBuild } from './WhereToBuild';

let host: HTMLDivElement;
let root: Root;

beforeEach(() => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  useGame.setState(useGame.getInitialState(), true);
  host = document.body.appendChild(document.createElement('div'));
  root = createRoot(host);
});

afterEach(() => {
  act(() => root.unmount());
  host.remove(); vi.unstubAllGlobals();
});

async function fixture(): Promise<{ view: PlayerView; item: ProductionItem; cityIds: number[] }> {
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('42', 'India', 'Byzantium', 'Ada', 'Bo'));
  // Resolve one turn so the founded capital has current production options.
  client.endTurn({});
  const view = structuredClone(client.view(0));
  const first = view.cities.find((city) => city.owner === view.me)!;
  const second = structuredClone(first); second.id = first.id + 100; second.name = 'Second city';
  const third = structuredClone(first); third.id = first.id + 200; third.name = 'Third city';
  second.detail!.options = structuredClone(first.detail!.options);
  third.detail!.options = structuredClone(first.detail!.options);
  const item = first.detail!.options.find((option) => option.item.kind === 'Building')!.item;
  // Keep this an available item from the real fixture, while making it costly enough to compare.
  for (const city of [first, second, third]) {
    const option = city.detail!.options.find((candidate) => sameItemForTest(candidate.item, item))!;
    option.cost = 400;
    option.turns = city === first ? 8 : city === second ? 3 : 5;
    option.production_stored = city === second ? 40 : 0;
  }
  const unlock = view.research.unlocks.find((candidate) => sameItemForTest(candidate.item, item));
  if (unlock) {
    unlock.cost_breakdown = { total: 200, parts: [['Listed cost', 200]] };
    if (!view.research.known.includes(unlock.tech)) view.research.known.push(unlock.tech);
  }
  view.cities.push(second, third);
  return { view, item, cityIds: [first.id, second.id, third.id] };
}

function sameItemForTest(a: ProductionItem, b: ProductionItem): boolean {
  return JSON.stringify(a) === JSON.stringify(b);
}

function render(view: PlayerView, item: ProductionItem) {
  useGame.setState({ views: [view], humans: [view.me], seat: 0, orders: [[]] });
  act(() => root.render(<WhereToBuild item={item} open />));
  return host.querySelector<HTMLElement>('[data-testid="where-to-build"]')!;
}

const rows = (list: Element) => [...list.querySelectorAll<HTMLElement>('[data-testid="wtb-row"]')];
const turns = (row: Element) => row.querySelector('.wtb-turns')?.textContent ?? null;

it('lists the fastest city first for the item', async () => {
  const { view, item, cityIds } = await fixture();
  const list = render(view, item);
  expect(rows(list).map((row) => Number(row.dataset.city))).toEqual([cityIds[1], cityIds[2], cityIds[0]]);
  expect(rows(list)[0].textContent).toContain('3 turns');
});

it('shows a blocked city with a readable reason', async () => {
  const { view, item, cityIds } = await fixture();
  const blocked = view.cities.find((city) => city.id === cityIds[2])!;
  blocked.detail!.options = blocked.detail!.options.filter((option) => !sameItemForTest(option.item, item));
  blocked.detail!.unavailable.push({ item, cost: 400, reason: 'Build in a city with Library', production_banked: 0 });
  const list = render(view, item);
  const row = list.querySelector(`[data-city="${blocked.id}"]`)!;
  expect(row.getAttribute('data-state')).toBe('blocked');
  expect(row.textContent).toContain('Needs a Library first');
});

it('marks a once-per-empire building that is already being built elsewhere', async () => {
  const { view, cityIds } = await fixture();
  const item: ProductionItem = { kind: 'Building', building: 'CourierHall' };
  const builder = view.cities.find((city) => city.id === cityIds[0])!;
  const template = builder.detail!.options[0];
  for (const cityId of cityIds) {
    const city = view.cities.find((entry) => entry.id === cityId)!;
    city.detail!.options.push({ ...template, item, cost: 400, turns: 6, production_stored: 0 });
  }
  builder.detail!.producing = item;
  builder.detail!.production_queue = [item];
  const list = render(view, item);
  const second = list.querySelector(`[data-city="${cityIds[1]}"]`)!;
  expect(second.getAttribute('data-state')).toBe('elsewhere');
  expect(second.textContent).toContain(`Already building in ${builder.name}; one per empire`);
});

it('opens the city tile picker for a Great Work without queuing a building without a tile', async () => {
  const { view, cityIds } = await fixture();
  view.research.known.push('Philosophy');
  const item: ProductionItem = { kind: 'Building', building: 'RelayUplink' };
  for (const cityId of cityIds) {
    const city = view.cities.find((entry) => entry.id === cityId)!;
    city.detail!.options.push({ ...city.detail!.options[0], item: { kind: 'Wonder', building: 'RelayUplink', tile: { q: city.pos.q + 1, r: city.pos.r } }, cost: 600, turns: 8, production_stored: 0 });
  }
  const list = render(view, item);
  const action = list.querySelector<HTMLButtonElement>(`[data-city="${cityIds[1]}"] button[aria-label^="Choose tile in Second city:"]`);
  expect(action).not.toBeNull();
  expect(list.querySelector('button[aria-label^="Build here"]')).toBeNull();
  act(() => action!.click());
  expect(useGame.getState().selection).toEqual({ kind: 'city', id: cityIds[1] });
  expect(useGame.getState().orders[0]).toEqual([]);
});

it.each([
  ['Build here', 'Build here in Second city:'],
  ['Queue here', 'Queue here in Second city:'],
] as const)('%s sets the order for that city and leaves the other rows unchanged', async (_label, accessiblePrefix) => {
  const { view, item, cityIds } = await fixture();
  const list = render(view, item);
  const before = rows(list).map((row) => ({ city: row.dataset.city, turns: turns(row) }));
  const targetCity = cityIds[1];
  const target = list.querySelector<HTMLLIElement>(`[data-city="${targetCity}"]`)!;
  const action = target.querySelector<HTMLButtonElement>(`button[aria-label^="${accessiblePrefix}"]`)!;
  expect(action).not.toBeNull();
  act(() => action.click());
  const order = useGame.getState().orders[0].find((candidate) => candidate.type === 'SetProductionQueue' && candidate.city === targetCity);
  expect(order).toBeDefined();
  if (order?.type === 'SetProductionQueue') expect(order.queue).toContainEqual(item);
  expect(useGame.getState().errors[0]).toEqual([]);

  const afterRows = rows(list).map((row) => ({ city: row.dataset.city, turns: turns(row) }));
  expect(afterRows.map((row) => row.city)).toEqual(before.map((row) => row.city));
  expect(afterRows.filter((row) => row.city !== String(targetCity))).toEqual(before.filter((row) => row.city !== String(targetCity)));
});
