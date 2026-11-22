import { describe, expect, it } from 'vitest';
import { RULES } from './data';
import { nileYields, resourceBenefitLines, tileYields } from './resources';

// The Nile (core `economy::nile`): Egypt sees double Food and no Production on its tiles and cards.
describe('Egypt sees no Production', () => {
  it('doubles Food and drops Production on a worked tile', () => {
    const tile = { terrain: 'Plains', resource: null, improvement: null, landmark: null } as const;
    const plain = tileYields('India', tile);
    expect(plain.production).toBeGreaterThan(0);
    expect(tileYields('Egypt', tile)).toEqual({ ...plain, food: (plain.food * RULES.egyptNileFoodPercent) / 100, production: 0 });
  });

  it('passes other factions through', () => {
    const y = { food: 1, production: 2, gold: 3, science: 0 };
    expect(nileYields('Germany', y)).toBe(y);
  });

  it('says a Quarry gives Egypt nothing, and keeps a strategic deposit to its supply', () => {
    expect(resourceBenefitLines('Stone', [], 'Egypt')).toEqual(['Nothing to build here: you have no Production']);
    expect(resourceBenefitLines('Stone', [], 'India')[0]).toContain('Production');
    const iron = resourceBenefitLines('Iron', [], 'Egypt');
    expect(iron.join(' ')).not.toContain('Production');
    expect(iron[0]).toMatch(/^Supplies/);
    expect(resourceBenefitLines('Wheat', [], 'Egypt')[0]).toMatch(/^\+2 Food/);
  });
});
