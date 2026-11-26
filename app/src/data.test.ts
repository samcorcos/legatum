import { expect, it } from 'vitest';
import {
  DEFAULT_DECADENCE_SLOPE, ERA_NAMES, ageName, BUILDING_KINDS, BUILDING_STATS, DECADENCE_RULE, RULES, decadenceCurveText, decadencePercent, decadenceRules, isProductionBuilding, itemName, pluralUnit,
} from './data';

it('writes unit plurals the way the turn report and alarms say them', () => {
  expect(itemName({ kind: 'Unit', unit: 'Phalangite' }, 'Rome')).toBe('Legionary');
  expect(['Legionary', 'Spearman', 'Infantry', 'Archer', 'Combat Drones'].map(pluralUnit))
    .toEqual(['Legionaries', 'Spearmen', 'Infantry', 'Archers', 'Combat Drones']);
});

it('derives German Industry cost eligibility from each building stat line', () => {
  expect(BUILDING_KINDS.filter(isProductionBuilding)).toEqual(['Workshop', 'Pyramids', 'Forge', 'Factory']);
  expect(BUILDING_STATS.Forge.yields.food).toBeUndefined();
});

it('mirrors the core decadence rate: L * c / 100 to the nearest percent, nothing at or below the average', () => {
  expect(DEFAULT_DECADENCE_SLOPE).toBe(0);
  expect(decadencePercent(10_000)).toBe(0);
  const on = { decadenceSlope: 45 };
  expect([-50, 0].map((l) => decadencePercent(l, on))).toEqual([0, 0]);
  expect([1, 2, 10, 30, 45, 60, 100, 200, 250].map((l) => decadencePercent(l, on))).toEqual([0, 1, 5, 14, 20, 27, 45, 90, 113]);
  expect(decadencePercent(10_000, on)).toBe(4_500);
  // Twice as far ahead pays twice the rate, to the nearest percent, with no maximum.
  for (let l = 2; l <= 2000; l += 2) expect(Math.abs(decadencePercent(2 * l, on) - 2 * decadencePercent(l, on))).toBeLessThanOrEqual(1);
  // Other slopes follow the same integer maths; 0 is off.
  expect(decadencePercent(37, { decadenceSlope: 100 })).toBe(37);
  expect(decadencePercent(500, { decadenceSlope: 0 })).toBe(0);
});

it('writes the decadence rate from the same function, never typed numbers', () => {
  const text = decadenceCurveText();
  for (const l of [30, 60, 100, 200]) expect(text).toContain(`${l}% ahead cuts ${decadencePercent(l)}%`);
  expect(text).toContain("no maximum");
});

it('the decadence rule says it slows growth and names the three cut yields and their reasons', () => {
  expect(DECADENCE_RULE).toContain('slows your growth');
  expect(DECADENCE_RULE).not.toMatch(/cuts your Food/);
  for (const w of ['smaller families', 'Production', 'sloth', 'Gold', 'luxury', 'Science', 'complacency']) expect(DECADENCE_RULE).toContain(w);
});

it('reads the decadence slope from the rule switches', () => {
  expect(decadenceRules()).toEqual({ decadenceSlope: 0 });
  expect(decadenceRules({ decadence_slope: 25 })).toEqual({ decadenceSlope: 25 });
});

it('the building price table is the core price table', () => {
  for (const [kind, stats] of Object.entries(RULES.buildingStats)) {
    expect(BUILDING_STATS[kind as keyof typeof BUILDING_STATS].cost, kind).toBe(stats.cost);
  }
});

it('an Age is named once: the Machine Age era is not the "Machine Age Age"', () => {
  expect(ageName('Medieval')).toBe('Medieval Age');
  expect(ageName('Machine Age')).toBe('Machine Age');
  expect(ERA_NAMES).toContain('Machine Age');
  for (const era of ERA_NAMES) expect(ageName(era)).not.toMatch(/Age Age/);
});
