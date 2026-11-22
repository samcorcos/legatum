// The era table the UI reads from the core's rules (strategic supply starts in the Classical era).
import { expect, it } from 'vitest';
import { RULES } from './data';
import { RESOURCES, RESOURCE_INFO, RESOURCE_ORDER, STRATEGIC_RESOURCES, SUPPLY_TIERS, buildingNeed, eraResource, isStrategic, unitNeed, upgradeNeed, resourceBenefitLines, supplyRuleText, supplyPerMine, fixClause } from './resources';

it('the nine resources split into four bonus ones and five strategic ones in era order', () => {
  expect(RESOURCES).toHaveLength(9);
  expect(RESOURCES.filter(isStrategic).sort()).toEqual([...STRATEGIC_RESOURCES].sort());
  expect(STRATEGIC_RESOURCES).toEqual(['Iron', 'Niter', 'Coal', 'Oil', eraResource(8)]);
  expect(RESOURCE_INFO.Copper.class).toBe('Bonus');
  expect([...RESOURCE_ORDER].sort()).toEqual([...RESOURCES].sort());
});

it('every era resource is strategic and every strategic resource is some era’s', () => {
  const eras = RULES.supply.eraResources as (string | null)[];
  // Nine eras; Oil covers the Machine Age, World War and Modern eras (user, 2026-11-14, and the ninth era's placeholder).
  expect(eras).toHaveLength(9);
  expect([eras[5], eras[6], eras[7]]).toEqual(['Oil', 'Oil', 'Oil']);
  for (const r of eras.filter(Boolean)) expect(RESOURCE_INFO[r as keyof typeof RESOURCE_INFO].class).toBe('Strategic');
  expect([...new Set(eras.filter(Boolean))].sort()).toEqual([...STRATEGIC_RESOURCES].sort());
  expect(eraResource(99)).toBe(eraResource(8));
  expect(eraResource(-1)).toBeNull();
});

it('the supply tiers are the buildings with a need, lowest era first, and a Factory city keeps its Forge’s Iron', () => {
  expect([...SUPPLY_TIERS].sort()).toEqual(Object.keys(RULES.supply.buildingNeeds).sort());
  expect(SUPPLY_TIERS.map((b) => buildingNeed(b))).toEqual(['Iron', 'Iron', 'Niter', 'Coal', 'Oil']);
  expect(buildingNeed('Market')).toBeNull();
  expect(buildingNeed('Workshop')).toBeNull();
});

it('turn-1 units and civilians are free, a Classical unit uses Iron, and an upgrade across eras switches resource', () => {
  for (const u of ['Spearman', 'Archer', 'Horseman', 'Settler', 'Scout'] as const) expect(unitNeed(u), u).toBeNull();
  expect(unitNeed('Phalangite')).toBe('Iron');
  expect(unitNeed('Musketman')).toBe('Niter');
  expect(upgradeNeed('Spearman', 'Phalangite')).toBe('Iron');
  expect(upgradeNeed('Phalangite', 'Pikeman')).toBeNull(); // Iron to Iron: nothing new is needed
  expect(upgradeNeed('Pikeman', 'Musketman')).toBe('Niter');
});

it('German deposit, tile, and improvement copy uses doubled supply', () => {
  expect(supplyRuleText('Iron', 'Germany')).toContain('supplies 4 Iron, 6 with a Mine');
  expect(resourceBenefitLines('Iron', ['IronWorking'], 'Germany')).toContain('Supplies 4 Iron inside your borders, 6 with a Mine');
  expect(supplyPerMine('Germany')).toBe(2);
  expect(fixClause('Iron', true, 'Germany')).toContain('+2 Iron');
  expect(supplyRuleText('Iron', 'India')).toContain('supplies 2 Iron, 3 with a Mine');
});
