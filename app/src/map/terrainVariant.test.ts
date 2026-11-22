import { describe, expect, it } from 'vitest';
import { TERRAIN_VARIANTS, drawFeatureArt, getTerrainVariant, isOrganic, setOrganic, setTerrainVariant } from './terrainVariant';
import { ATLAS } from './style';
import { miniatureGeometry } from './sprites';

describe('terrain variant', () => {
  it('is organic S2 by default, with the old look kept for comparison', () => {
    expect(getTerrainVariant()).toBe('s2');
    expect([...TERRAIN_VARIANTS]).toEqual(['current', 's2']);
  });

  it('is organic only once the sprites exist, and never on the old look', () => {
    // Nothing is loaded in unit tests, so the drawn glyphs stand in rather than a blank map.
    expect(isOrganic()).toBe(false);
    expect(drawFeatureArt({} as CanvasRenderingContext2D, { kind: 'Forest', c: 1, r: 1, zoom: 1, devPerWorld: 1, alpha: 1, tundra: false })).toBe(false);
    setOrganic(true);
    setTerrainVariant('current');
    expect(isOrganic()).toBe(false);
    setTerrainVariant('s2');
  });
});

describe('grid over the art', () => {
  it('is a pale casing under an ink core, so tiles stay countable in woods and ranges', () => {
    const a = ATLAS.gridArt;
    expect(a.casing).toBeGreaterThan(a.core);
    expect(a.core).toBeGreaterThanOrEqual(1);
    // Over open ground the plain line stays faint.
    expect(Number(/([\d.]+)\)$/.exec(ATLAS.grid)![1])).toBeLessThan(0.2);
  });
});

describe('miniature geometry', () => {
  it('puts the base under the figure and the halo around both', () => {
    const g = miniatureGeometry(14);
    expect(g.baseDy).toBeGreaterThan(g.bodyDy);
    expect(g.halo).toBeGreaterThanOrEqual(g.S * 0.5);
  });
});
