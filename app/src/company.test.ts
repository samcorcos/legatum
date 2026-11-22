import { expect, it } from 'vitest';
import { COMPANY_UNIT_RULE, contractLeft, contractLine, contractTag, rosterLine, shopRows, companyName, companyStats } from './company';
import { describeOrder, hasBuy, toggleBuy, toggleHire } from './orders';
import { RULES } from './data';
import type { Order, PlayerView, UnitView } from './types';

const unit = (over: Partial<UnitView>): UnitView => ({ id: 7, kind: 'Soldier', owner: 0, pos: { q: 1, r: 1 }, hp: 100, strength: 20, military: true, ...over } as UnitView);
const view = (over: Partial<PlayerView> = {}): PlayerView => ({
  me: 0, team: 0, turn: 20,
  players: [
    { id: 0, team: 0, faction: 'Byzantium', name: 'Ada', alive: true, companies: 2 },
    { id: 1, team: 1, faction: 'Sparta', name: 'AI 2', alive: true },
    { id: 2, team: 2, faction: 'India', name: 'AI 3', alive: true },
  ],
  units: [unit({ id: 7, contract_until: 27, origin: 'Sparta' }), unit({ id: 8, contract_until: 22, origin: 'India' })],
  company_offers: [
    { seller: 2, unit: 'Knight', price: 204 }, { seller: 1, unit: 'Soldier', price: 80 },
  ],
  cities: [{ id: 3, name: 'Constantinople', owner: 0 }],
  ...over,
} as unknown as PlayerView);

it('reads the contract off the unit: turns left, the seller, and the last turn', () => {
  const v = view();
  expect(contractLeft(v.units[0], v)).toBe(7);
  expect(contractLeft(unit({}), v)).toBeNull();
  expect(contractLine(v.units[0], v)).toBe(`Hired from Sparta: 7 turns of contract left, upkeep ${RULES.company.upkeep} Gold a turn`);
  expect(contractLine(unit({ contract_until: 21, origin: 'Sparta' }), v)).toBe(`Hired from Sparta: 1 turn of contract left, upkeep ${RULES.company.upkeep} Gold a turn`);
  expect(contractLine(unit({ contract_until: 20, origin: 'Sparta' }), v)).toBe('Hired from Sparta: goes home this turn');
  expect(contractLine(unit({ contract_until: 15 }), v)).toBe('Hired from another empire: goes home this turn');
  expect(contractTag(0)).toBe('Last turn');
  expect(contractTag(4)).toBe('4 turns');
  expect(COMPANY_UNIT_RULE).toContain(`${RULES.company.turns} turns`);
});

it('counts an empire\'s Companies, their upkeep and when the soonest goes home', () => {
  const v = view();
  expect(rosterLine(v, 0, 2)).toBe(`Companies: 2 · upkeep ${2 * RULES.company.upkeep} Gold a turn · next goes home in 2 turns`);
  // A rival's Company out of sight: the count is public, the countdown is not guessed.
  expect(rosterLine(v, 0, 3)).toBe(`Companies: 3 · upkeep ${3 * RULES.company.upkeep} Gold a turn`);
});

it('lists the shop one row per seller in a stable order, each with its seller\'s faction', () => {
  const rows = shopRows(view());
  expect(rows.map((r) => [r.offer.seller, r.faction, r.offer.unit])).toEqual([[1, 'Sparta', 'Soldier'], [2, 'India', 'Knight']]);
  expect(companyName('India', 'Knight')).toBe('War Elephant');
  expect(companyStats('India', 'Knight')).toContain('3 moves');
});

it('a hire takes the city\'s one purchase, replacing a Buy and being replaced by one; clicking it again cancels', () => {
  const hire: Order = { type: 'HireCompany', city: 3, unit: 'Soldier', seller: 1 };
  const buy: Order = { type: 'BuyItem', city: 3, item: { kind: 'Unit', unit: 'Scout' } };
  const other: Order = { type: 'HireCompany', city: 3, unit: 'Ranger', seller: 2 };
  expect(toggleHire([buy], hire as never)).toEqual([hire]);
  expect(toggleHire([hire], other as never)).toEqual([other]);
  expect(toggleHire([hire], hire as never)).toEqual([]);
  expect(toggleBuy([hire], 3, { kind: 'Unit', unit: 'Scout' })).toEqual([buy]);
  expect(hasBuy([hire], 3)).toBe(true);
  expect(hasBuy([hire], 4)).toBe(false);
});

it('describes a queued hire with the seller and the price from the shop', () => {
  const v = view();
  expect(describeOrder({ type: 'HireCompany', city: 3, unit: 'Soldier', seller: 1 }, v)).toBe(`Constantinople: hire a Spartan Hoplite Company from Sparta for 80 Gold (${RULES.company.turns} turns)`);
});
