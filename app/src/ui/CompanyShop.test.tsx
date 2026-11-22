// @vitest-environment happy-dom
// Byzantium's shop in the city panel, and a hired Company in the unit panel and tile card.
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { CityPanel } from './CityPanel';
import { UnitPanel } from './UnitPanel';
import { TileCard } from './TileCard';
import { FixtureClient } from '../test/fixtureEngine';
import { useGame } from '../store';
import { RULES, SPECIAL_UNIT, specialUnitEdge, specialUnitName } from '../data';
import type { Faction, PlayerView } from '../types';

let container: HTMLDivElement;
let root: Root;
beforeEach(() => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  container = document.body.appendChild(document.createElement('div'));
  root = createRoot(container);
  useGame.setState(useGame.getInitialState(), true);
});
afterEach(() => { act(() => root.unmount()); container.remove(); vi.unstubAllGlobals(); });

/** A founded Byzantine (or other) capital, the shop with two sellers (Sparta's Hoplite, a dear Knight from the third seat), and a client whose validation refuses the dear one without 300 Gold. */
async function renderShop(faction: Faction = 'Byzantium', gold = 500, offers?: PlayerView['company_offers']) {
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('42', faction, 'Sparta', 'Ada', 'Bo'));
  client.endTurn({});
  client.debugState().players[0].gold = gold;
  const base = client.view(0);
  const special = (seat: number) => SPECIAL_UNIT[base.players.find((p) => p.id === seat)!.faction];
  const view: PlayerView = { ...base, company_offers: faction === 'Byzantium' ? offers ?? [
    { seller: 2, unit: special(2), price: 40 }, { seller: 3, unit: special(3), price: 300 },
  ] : [] };
  const city = view.cities.find((c) => c.owner === 0)!;
  (client as unknown as { validate: unknown }).validate = (_me: number, orders: { type: string; seller?: number; unit?: string }[]) =>
    orders.flatMap((o, index) => (o.type === 'HireCompany' && o.seller === 3 && gold < 300 ? [{ index, message: `Not enough Gold: the company costs 300 Gold; you have ${gold} (Need ${300 - gold} more Gold)` }] : []));
  useGame.setState({ client, views: [view], humans: [0], orders: [[]], errors: [[]], seat: 0 });
  act(() => root.render(<CityPanel city={city} view={view} />));
  return { client, city, view };
}
const rows = () => [...container.querySelectorAll<HTMLElement>('[data-testid="company-row"]')];
const orders = () => useGame.getState().orders[0];

it('shows the shop to Byzantium only, one row per seller', async () => {
  await renderShop('Switzerland');
  expect(container.querySelector('[data-testid="company-shop"]')).toBeNull();
  act(() => root.unmount());
  root = createRoot(container);
  await renderShop('Byzantium');
  const shop = container.querySelector('[data-testid="company-shop"]')!;
  expect(shop.textContent).toContain('Hire a Mercenary Company');
  expect(shop.textContent).toContain(`${RULES.company.turns}-turn contract`);
  expect(shop.textContent).toContain(`upkeep ${RULES.company.upkeep} Gold a turn each`);
  expect(rows()).toHaveLength(2);
  // The top-of-shop text: two plain lines at most, each empire sells only its own special unit and it arrives as that empire's version.
  const gets = shop.querySelector('[data-testid="company-gets"]')!.textContent!;
  expect(gets).toContain('only its own special unit');
  expect(gets).toContain("that empire's version");
  // The Gold-per-turn effect of a hire is said once here, never on a row.
  expect(shop.querySelector('.company-rule')!.textContent).toMatch(/Gold\/turn [+−]?\d+ → [+−]?\d+ after a hire/);
  expect(rows()[0].textContent).not.toContain('Gold/turn');
  // The seller's own name for its special unit, its stats and the edge that sets it apart, the offer's price.
  const first = rows().find((r) => r.dataset.seller === '2')!;
  const faction = useGame.getState().views![0]!.players.find((p) => p.id === 2)!.faction;
  expect(first.textContent).toContain(specialUnitName(faction));
  expect(first.textContent).toMatch(/melee/);
  expect(first.textContent).toContain(specialUnitEdge(faction));
  expect(first.querySelector('button')!.getAttribute('aria-label')).toMatch(new RegExp(`^Hire an? ${specialUnitName(faction)} Company from .* for 40 Gold$`));
});

it('one click queues the hire on its row, and a second click takes it back', async () => {
  await renderShop();
  const hire = rows().find((r) => r.dataset.seller === '2')!.querySelector('button')!;
  act(() => hire.click());
  expect(orders()).toEqual([{ type: 'HireCompany', city: expect.any(Number), unit: expect.any(String), seller: 2 }]);
  const status = container.querySelector('.purchase-status')!.textContent!;
  expect(status).toMatch(/\d+ Gold \([+−]\d+\/turn\) to spend/);
  expect(container.querySelector('[data-testid="company-row"].chosen')).not.toBeNull();
  // One purchase per city per turn: the other row is greyed, with the reason, until this one is cancelled.
  const other = rows().find((r) => r.dataset.seller === '3')!;
  expect(other.querySelector('button')!.disabled).toBe(true);
  // Said once for the section, not under every row.
  expect(other.querySelector('.company-why')).toBeNull();
  expect(container.querySelector('#company-one-purchase')!.textContent).toContain('one purchase a turn');
  act(() => rows().find((r) => r.dataset.seller === '2')!.querySelector('button')!.click());
  expect(orders()).toEqual([]);
  act(() => rows().find((r) => r.dataset.seller === '3')!.querySelector('button')!.click());
  expect(orders()[0]).toMatchObject({ type: 'HireCompany', seller: 3 });
  act(() => rows().find((r) => r.dataset.seller === '3')!.querySelector('button')!.click());
  expect(orders()).toEqual([]);
});

it('a row that cannot be hired is greyed with its shortfall where the button would be, and the price stays on the row', async () => {
  await renderShop('Byzantium', 120);
  const dear = rows().find((r) => r.dataset.seller === '3')!;
  const button = dear.querySelector('button')!;
  expect(button.disabled).toBe(true);
  expect(button.querySelector('.company-why')!.textContent).toContain('Need 180 more');
  expect(button.getAttribute('aria-describedby')).toBe(button.querySelector('.company-why')!.id);
  expect(dear.querySelector('.company-price')!.textContent).toContain('300');
  act(() => button.click());
  expect(orders()).toEqual([]);
});

it('shows the hired Company\'s net Gold and first unpaid turn from the treasury after its price', async () => {
  const { city, view } = await renderShop('Byzantium', 46);
  view.economy.gold_per_turn.total = -3;
  useGame.setState({ views: [view] });
  act(() => root.render(<CityPanel city={city} view={view} />));
  const row = rows().find((r) => r.dataset.seller === '2')!;
  expect(container.querySelector('.company-rule')!.textContent).toContain('Gold/turn −3 → −5 after a hire');
  expect(row.querySelector('.company-impact')!.textContent).toContain(`At your current income, treasury cannot pay at the end of turn ${view.turn + 1}`);
  expect(row.querySelector('.company-impact')!.textContent).toContain('an unpaid Company turns barbarian where it stands');
});

it('a seller that cannot field its unit yet is one muted line saying why, with nothing to press', async () => {
  await renderShop('Byzantium', 500, [
    { seller: 2, unit: 'Soldier', price: 40 },
    { seller: 3, unit: 'Tank', price: 300, unavailable: 'Panzer needs Armoured Warfare' },
  ]);
  const shop = container.querySelector('[data-testid="company-shop"]')!;
  expect(shop.textContent).toContain('1 for hire');
  const muted = shop.querySelector('[data-testid="company-unavailable"]')!;
  expect(muted.textContent).toMatch(/: Panzer needs Armoured Warfare$/);
  expect(muted.closest('[data-testid="company-row"]')!.querySelector('button')).toBeNull();
  expect(rows()).toHaveLength(2);
});

it('says why the shop is empty before first contact', async () => {
  const { client, city } = await renderShop();
  const view = { ...client.view(0), company_offers: [] };
  useGame.setState({ views: [view] });
  act(() => root.render(<CityPanel city={city} view={view} />));
  expect(container.querySelector('[data-testid="company-empty"]')!.textContent).toMatch(/met and are not at War with/);
});

it('a hired Company says whose it was, its countdown and its upkeep in the unit panel and tile card; the last turn says so', async () => {
  const { client, view } = await renderShop();
  const soldier = view.units.find((u) => u.owner === 0 && u.kind !== 'Settler') ?? client.view(0).units.find((u) => u.owner === 0)!;
  const hired = { ...soldier, kind: 'Soldier' as const, military: true, contract_until: view.turn + 7, origin: 'Sparta' as const };
  const v = { ...view, units: [hired, ...view.units.filter((u) => u.id !== soldier.id)] };
  useGame.setState({ views: [v], selection: { kind: 'unit', id: hired.id } });
  act(() => root.render(<UnitPanel unit={hired} view={v} />));
  expect(container.querySelector('[data-testid="company-tag"]')!.textContent).toContain(`Hired from Sparta: 7 turns of contract left, upkeep ${RULES.company.upkeep} Gold a turn`);
  expect(container.querySelector('[data-testid="company-note"]')!.textContent).toContain('cannot be upgraded');
  // No upgrade row for a Company: it is replaced by hiring the next kind.
  expect(container.textContent).not.toMatch(/Upgrade →/);
  // The last turn, in words.
  const last = { ...hired, contract_until: v.turn };
  const v2 = { ...v, units: [last] };
  act(() => root.render(<UnitPanel unit={last} view={v2} />));
  expect(container.querySelector('[data-testid="company-tag"]')!.textContent).toContain('Hired from Sparta: goes home this turn');
  act(() => root.render(<TileCard view={v2} hex={last.pos} />));
  expect(container.querySelector('[data-testid="tile-company"]')!.textContent).toContain('goes home this turn');
});
