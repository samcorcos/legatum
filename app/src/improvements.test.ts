import { expect, it } from 'vitest';
import { improvedResourceYields, improvementGain, improvementKindFor, improvementShortTitle, improvementTitle, improvementUpgradeNote, isDormant } from './improvements';
import { fromOffset } from './hex';

it('names each resource\'s improvement and prices the upgrade by technology', () => {
  expect(['Wheat', 'Cattle', 'Stone', 'Copper', 'Iron', 'Coal'].map((r) => improvementKindFor(r as never))).toEqual(['Farm', 'Pasture', 'Quarry', 'Mine', 'Mine', 'Mine']);
  const none = () => false, all = () => true;
  expect(improvedResourceYields('Wheat', none)).toEqual({ food: 1, production: 0, gold: 0, science: 0 });
  expect(improvedResourceYields('Wheat', all)).toEqual({ food: 2, production: 0, gold: 0, science: 0 });
  expect(improvedResourceYields('Copper', all)).toEqual({ food: 0, production: 1, gold: 3, science: 0 });
  // A hidden strategic deposit yields nothing until its revealing technology is known.
  expect(improvedResourceYields('Iron', none)).toEqual({ food: 0, production: 0, gold: 0, science: 0 });
});

it('titles an improvement by resource, distance and bearing, never coordinates', () => {
  const city = fromOffset(5, 8), tile = fromOffset(7, 8);
  expect(improvementTitle({ kind: 'Improvement', improvement: 'Mine', tile }, city, 'Iron')).toBe('Mine, Iron · 2\u00a0tiles\u00a0E');
  expect(improvementUpgradeNote('Mine')).toBe('Deep Mine with Iron Working');
});

it('dormant is one check: an owned, known, unimproved resource', () => {
  expect(isDormant({ resource: 'Wheat', owner: 0, improvement: null }, 0)).toBe(true);
  expect(isDormant({ resource: 'Iron', owner: 0 }, 0)).toBe(true);
  expect(isDormant({ resource: 'Wheat', owner: 0, improvement: { kind: 'Farm' } }, 0)).toBe(false);
  expect(isDormant({ resource: 'Wheat', owner: 1, improvement: null }, 0)).toBe(false);
  expect(isDormant({ resource: null, owner: 0, improvement: null }, 0)).toBe(false);
});

it('one gain for the tile card and the picker: the upgrade counts once its technology is known; a visible deposit is revealed', () => {
  // Iron Working both reveals Iron and upgrades the Mine: a visible Iron tile has both benefits.
  expect(improvementGain('Iron', [])).toEqual({ food: 0, production: 2, gold: 0, science: 0 });
  expect(improvementGain('Iron', ['IronWorking'])).toEqual({ food: 0, production: 2, gold: 0, science: 0 });
  expect(improvementGain('Wheat', ['IronWorking'])).toEqual({ food: 1, production: 0, gold: 0, science: 0 });
});

it('names an improvement on the production meter in one short line, bearing last', () => {
  const city = fromOffset(5, 8), tile = fromOffset(7, 8);
  expect(improvementShortTitle({ kind: 'Improvement', improvement: 'Quarry', tile }, city, 'Stone')).toBe('Quarry, Stone E');
});
