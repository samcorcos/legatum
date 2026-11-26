import { describe, expect, it } from 'vitest';
import { mapUnitArtSlug, mapUnitMiniatureUrl, unitMiniatureUrl } from '../unitArt';

describe('mapUnitMiniatureUrl (the art every map draws for a unit)', () => {
  const factionOf = new Map([[0, 'Aztecs' as const], [1, 'Switzerland' as const]]);
  it("uses the owner's own faction, not a seat colour", () => {
    expect(mapUnitMiniatureUrl({ kind: 'Spearman', owner: 0 }, factionOf, false)).toBe(unitMiniatureUrl('Spearman', 'Aztecs'));
  });
  it("prefers the unit's origin (a hired Company keeps its people's figure)", () => {
    expect(mapUnitMiniatureUrl({ kind: 'Pikeman', owner: 0, origin: 'Switzerland' }, factionOf, false)).toBe(unitMiniatureUrl('Pikeman', 'Switzerland'));
  });
  it("gives barbarians their own raider art", () => {
    const url = mapUnitMiniatureUrl({ kind: 'Spearman', owner: 255 }, factionOf, true);
    expect(url).toBeTruthy();
    expect(url).not.toBe(unitMiniatureUrl('Spearman'));
  });
});

describe('mapUnitArtSlug (the 3D map keys unit models by it)', () => {
  const factionOf = new Map([[0, 'Aztecs' as const], [1, 'Switzerland' as const], [2, 'Sparta' as const]]);
  it('gives a plain Spearman the spearman art, so it gets the Spearman model', () => {
    expect(mapUnitArtSlug({ kind: 'Spearman', owner: 1 }, factionOf, false)).toBe('spearman');
  });
  it("keeps a faction's own figure and a barbarian raider off the plain Spearman's model", () => {
    expect(mapUnitArtSlug({ kind: 'Spearman', owner: 0 }, factionOf, false)).toBe('eagle-warrior');
    expect(mapUnitArtSlug({ kind: 'Spearman', owner: 2 }, factionOf, false)).toBe('spartan-hoplite');
    expect(mapUnitArtSlug({ kind: 'Spearman', owner: 255 }, factionOf, true)).not.toBe('spearman');
  });
});
