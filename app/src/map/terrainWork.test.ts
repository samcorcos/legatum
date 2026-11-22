import { describe, expect, it } from 'vitest';
import { landChanged } from './terrain';

// The tile signature of TerrainCache.update: explored 1, visible 2, terrain << 2, owner + 1 << 6, city population + 1 << 11,
// capital 1 << 17, walls 1 << 18, city era + 1 << 21, organic region << 25.
const sig = (o: { explored?: boolean; visible?: boolean; terrain?: number; owner?: number; pop?: number; walls?: boolean; era?: number; region?: number }) =>
  (o.explored ? 1 : 0) | (o.visible ? 2 : 0) | ((o.terrain ?? 0) << 2) | (((o.owner ?? -1) + 1) << 6) | ((o.pop ?? 0) << 11) | (o.walls ? 1 << 18 : 0) | ((o.era ?? 0) << 21) | ((o.region ?? 0) << 25);

describe('what rebuilds the blurred land fields', () => {
  const plain = { explored: true, visible: true, terrain: 3, owner: 1 };

  it('sight, owners, city growth, walls and eras do not', () => {
    expect(landChanged(sig(plain), sig({ ...plain, visible: false }))).toBe(false);
    expect(landChanged(sig(plain), sig({ ...plain, owner: 4 }))).toBe(false);
    expect(landChanged(sig({ ...plain, pop: 5, era: 2 }), sig({ ...plain, pop: 6, era: 3, walls: true }))).toBe(false);
  });

  it('a tile being explored, its terrain, a city appearing or going, and its organic region do', () => {
    expect(landChanged(-1, sig(plain))).toBe(true);
    expect(landChanged(sig({ terrain: 3 }), sig({ explored: true, terrain: 3 }))).toBe(true);
    expect(landChanged(sig(plain), sig({ ...plain, terrain: 7 }))).toBe(true);
    expect(landChanged(sig(plain), sig({ ...plain, pop: 2 }))).toBe(true);
    expect(landChanged(sig({ ...plain, pop: 2 }), sig(plain))).toBe(true);
    expect(landChanged(sig({ ...plain, region: 9 }), sig({ ...plain, region: 70 }))).toBe(true);
  });
});

describe('where the coast formula is settled', () => {
  it('finds the field value past which a rising distance stays over or under a target', async () => {
    const { coastBound } = await import('./terrain');
    const ds = (a: number) => (a - 0.5) * 100;
    const hi = coastBound(ds, 5, true), lo = coastBound(ds, -20, false);
    expect(hi).toBeGreaterThanOrEqual(0.55);
    expect(hi).toBeCloseTo(0.55, 6);
    expect(ds(hi)).toBeGreaterThanOrEqual(5);
    expect(lo).toBeLessThanOrEqual(0.3);
    expect(lo).toBeCloseTo(0.3, 6);
    expect(ds(lo)).toBeLessThanOrEqual(-20);
  });

  it('is out of reach when the distance never gets there', async () => {
    const { coastBound } = await import('./terrain');
    const flat = (a: number) => a - 0.5;
    expect(coastBound(flat, 5, true)).toBe(2);
    expect(coastBound(flat, -5, false)).toBe(-1);
  });
});
