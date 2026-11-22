import { expect, it } from 'vitest';
import { counterFootprint, improvementDetail, improvementSize, improvementStem, missingImprovementArt } from './improvementArt';

it('leaves the counters\' footprint bare: wider for a stack, lower on a city tile', () => {
  const one = counterFootprint(40, 1, false), two = counterFootprint(40, 2, false), town = counterFootprint(40, 1, true);
  expect(one.x).toBeCloseTo(-(one.x + one.w));
  expect(two.w).toBeGreaterThan(one.w * 1.8);
  expect(two.h).toBeGreaterThan(one.h);
  expect(town.y).toBeGreaterThan(one.y);
  expect(one.w).toBeGreaterThan(2 * 17 * 0.9);
});

it('ships a painted form for every resource improvement and every visible upgrade', () => {
  expect(missingImprovementArt()).toEqual([]);
  expect(['Farm', 'Pasture', 'Quarry', 'Mine'].map((kind) => improvementStem(kind as 'Farm' | 'Pasture' | 'Quarry' | 'Mine', false))).toEqual(['farm', 'pasture', 'quarry', 'mine']);
  expect(['Farm', 'Pasture', 'Quarry', 'Mine'].map((kind) => improvementStem(kind as 'Farm' | 'Pasture' | 'Quarry' | 'Mine', true))).toEqual(['irrigated-farm', 'ranch', 'deep-quarry', 'deep-mine']);
  expect(['SaltpetreWorks', 'OilWell', 'QuartzMine'].map((kind) => improvementStem(kind as 'SaltpetreWorks' | 'OilWell' | 'QuartzMine', false))).toEqual(['saltpetre-works', 'oil-well', 'quartz-mine']);
});

it('uses paintings at normal zoom and the existing badge tier farther out', () => {
  expect(improvementDetail(22)).toBe('art');
  expect(improvementDetail(21.9)).toBe('badge');
  expect(improvementSize(45)).toBeLessThan(92); // smaller than the standard city miniature (CITY_WORLD)
});
