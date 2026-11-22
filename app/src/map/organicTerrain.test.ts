import { describe, expect, it } from 'vitest';
import { centerX, centerY, nearestTile, neighborIndex } from './grid';
import { MASSIF, SMALL_REGION, computeRegions, organicKind, organicSig, planOrganic, type OrganicKind, type Regions } from './organicTerrain';
import type { Terrain } from '../types';

const W = 20;
const H = 16;
const at = (c: number, r: number) => r * W + c;

/** A map from a list of painted tiles; everything else is Grassland. */
function map(paint: [number, number, Terrain][]): Terrain[] {
  const t: Terrain[] = new Array(W * H).fill('Grassland');
  for (const [c, r, k] of paint) t[at(c, r)] = k;
  return t;
}
const block = (c0: number, r0: number, c1: number, r1: number, k: Terrain): [number, number, Terrain][] => {
  const out: [number, number, Terrain][] = [];
  for (let r = r0; r <= r1; r++) for (let c = c0; c <= c1; c++) out.push([c, r, k]);
  return out;
};
const regionsOf = (t: Terrain[], explored: (i: number) => boolean = () => true) => computeRegions(W, H, (i) => (explored(i) ? organicKind(t[i]) : null));
const plan = (R: Regions, t: Terrain[], far = false) =>
  planOrganic({ regions: R, isCity: () => false, tundraNear: (i) => [0, 1, 2, 3, 4, 5].some((e) => { const j = neighborIndex(i, e, W, H); return j >= 0 && t[j] === 'Tundra'; }), far });

describe('computeRegions', () => {
  it('finds connected components per kind, with sizes and tiles', () => {
    const t = map([...block(2, 2, 5, 5, 'Forest'), [10, 3, 'Forest'], [11, 3, 'Forest'], [10, 8, 'Mountains'], [11, 8, 'Mountains'], [12, 8, 'Hills']]);
    const R = regionsOf(t);
    const sizes = R.kind.map((k, r) => `${k}:${R.size[r]}`).sort();
    expect(sizes).toEqual(['Forest:16', 'Forest:2', 'Hills:1', 'Mountains:2']);
    expect(R.id[at(2, 2)]).toBe(R.id[at(5, 5)]);
    expect(R.id[at(10, 3)]).not.toBe(R.id[at(2, 2)]);
    // Hills next to mountains are a separate region.
    expect(R.id[at(12, 8)]).not.toBe(R.id[at(11, 8)]);
    expect(R.id[at(0, 0)]).toBe(-1);
  });

  it('measures each tile’s distance to its region’s edge', () => {
    const t = map(block(2, 2, 8, 8, 'Forest'));
    const R = regionsOf(t);
    expect(R.edge[at(2, 2)]).toBe(0);
    expect(R.edge[at(8, 5)]).toBe(0);
    expect(R.edge[at(3, 4)]).toBe(1);
    expect(R.edge[at(5, 5)]).toBe(3);
  });

  it('treats the map edge as more of the region but unexplored tiles as outside', () => {
    const t = map(block(0, 0, 4, 4, 'Forest'));
    const R = regionsOf(t);
    expect(R.edge[at(0, 0)]).toBeGreaterThan(0); // off-map neighbours do not make an edge
    const fogged = regionsOf(t, (i) => i !== at(2, 2));
    expect(fogged.id[at(2, 2)]).toBe(-1);
    for (let e = 0; e < 6; e++) expect(fogged.edge[neighborIndex(at(2, 2), e, W, H)]).toBe(0);
  });

  it('runs a spine along a long range and links it tile to tile', () => {
    const line: [number, number, Terrain][] = [2, 3, 4, 5, 6, 7, 8, 9].map((c) => [c, 6, 'Mountains']);
    const R = regionsOf(map(line));
    for (const [c] of line) expect(R.spine[at(c, 6)]).toBe(1);
    expect(R.links).toHaveLength(7);
    for (const [a, b] of R.links) expect([0, 1, 2, 3, 4, 5].some((e) => neighborIndex(a, e, W, H) === b)).toBe(true);
  });

  it('gives small ranges and forests no spine', () => {
    const R = regionsOf(map([[2, 2, 'Mountains'], [3, 2, 'Mountains'], ...block(6, 6, 9, 9, 'Forest')]));
    expect([...R.spine].every((v) => v === 0)).toBe(true);
  });
});

describe('planOrganic', () => {
  const big = map([
    ...block(2, 2, 9, 9, 'Forest'), // 64-tile wood
    [14, 2, 'Forest'], [15, 2, 'Forest'], // grove
    ...[3, 4, 5, 6, 7, 8].flatMap((c): [number, number, Terrain][] => [[c, 12, 'Mountains'], [c, 13, 'Mountains']]), // 12-tile massif
    [14, 8, 'Mountains'], [15, 8, 'Mountains'], // two peaks
    ...block(12, 11, 17, 14, 'Hills'),
  ]);
  const R = regionsOf(big);
  const P = plan(R, big);

  it('is deterministic', () => {
    const again = plan(regionsOf(big), big);
    expect(again.elems).toEqual(P.elems);
    expect(plan(R, big, true).elems).toEqual(plan(R, big, true).elems);
  });

  it('leaves every feature tile at least one sprite inside its own hex, so its type reads on hover', () => {
    for (let i = 0; i < W * H; i++) {
      if (!organicKind(big[i])) continue;
      expect(P.covered[i]).toBe(1);
      const own = (P.byTile.get(i) ?? []).map((k) => P.elems[k]).filter((e) => nearestTile(e.x, e.y, W, H) === i && !e.foothill);
      expect(own.length, `tile ${i % W},${Math.floor(i / W)}`).toBeGreaterThan(0);
    }
  });

  it('puts every sprite on a tile of its own kind (no forest on the grassland, no peaks on the plain)', () => {
    for (const e of P.elems) {
      const t = big[nearestTile(e.x, e.y, W, H)];
      expect(t, JSON.stringify(e)).toBe(e.kind);
    }
  });

  it('makes a big wood dense and a grove a main cluster per hex with two small companions', () => {
    const woodTiles = R.tiles[R.id[at(5, 5)]];
    const woodElems = woodTiles.reduce((n, i) => n + (P.byTile.get(i)?.length ?? 0), 0);
    expect(woodElems / woodTiles.length).toBeGreaterThan(2);
    for (const i of [at(14, 2), at(15, 2)]) {
      const own = (P.byTile.get(i) ?? []).map((k) => P.elems[k]);
      expect(own).toHaveLength(3);
      // The main cluster stands at the hex centre; the companions are smaller, so the grove's hexes still fill.
      expect(own.filter((e) => e.scale >= 0.92 && Math.hypot(e.x - centerX(i, W), e.y - centerY(i, W)) < 1)).toHaveLength(1);
    }
  });

  it('thins a wood toward its edge', () => {
    const wood = R.tiles[R.id[at(5, 5)]];
    const per = (pred: (i: number) => boolean) => {
      const ts = wood.filter(pred);
      return ts.reduce((n, i) => n + (P.byTile.get(i)?.length ?? 0), 0) / ts.length;
    };
    expect(per((i) => R.edge[i] >= 2)).toBeGreaterThan(per((i) => R.edge[i] === 0));
  });

  it('raises a massif to bigger peaks than a pair of peaks, and keeps edge peaks classic size', () => {
    const peaks = (r: number) => P.elems.filter((e) => e.kind === 'Mountains' && !e.foothill && R.id[e.tile] === r).map((e) => e.scale);
    const massif = R.id[at(5, 12)];
    const pair = R.id[at(14, 8)];
    expect(R.size[massif]).toBeGreaterThanOrEqual(MASSIF);
    expect(R.size[pair]).toBeLessThanOrEqual(SMALL_REGION);
    expect(Math.max(...peaks(massif))).toBeGreaterThan(Math.max(...peaks(pair)) * 1.15);
    // Edge peaks stay classic size, except along a massif's spine, where a peak may overhang its lowland neighbours by a flank.
    for (const e of P.elems) if (e.kind === 'Mountains' && !e.foothill && R.edge[e.tile] === 0) expect(e.scale).toBeLessThanOrEqual(R.spine[e.tile] ? 1.31 : 1.06);
  });

  it('draws saddles between spine tiles, so a range has no gaps', () => {
    const massif = R.id[at(5, 12)];
    const links = R.links.filter(([a]) => R.id[a] === massif);
    expect(links.length).toBeGreaterThan(3);
    for (const [a, b] of links) {
      const mx = (centerX(a, W) + centerX(b, W)) / 2;
      const my = (centerY(a, W) + centerY(b, W)) / 2;
      expect(P.elems.some((e) => e.kind === 'Mountains' && Math.hypot(e.x - mx, e.y - my) < 6)).toBe(true);
    }
  });

  it('zoomed out, keeps one bold mark centred on every forest and hill tile', () => {
    const far = plan(R, big, true);
    for (let i = 0; i < W * H; i++) {
      if (big[i] !== 'Forest' && big[i] !== 'Hills') continue;
      const own = (far.byTile.get(i) ?? []).map((k) => far.elems[k]);
      expect(own).toHaveLength(1);
      expect(Math.hypot(own[0].x - centerX(i, W), own[0].y - centerY(i, W))).toBeLessThan(1e-6);
      expect(own[0].scale).toBeGreaterThanOrEqual(0.92);
    }
  });

  it('skips city tiles', () => {
    const city = at(5, 5);
    const p = planOrganic({ regions: R, isCity: (i) => i === city, tundraNear: () => false, far: false });
    expect(p.covered[city]).toBe(0);
    expect(p.elems.some((e) => e.tile === city || nearestTile(e.x, e.y, W, H) === city)).toBe(false);
  });

  it('takes the conifer sprite beside tundra', () => {
    const t = map([...block(2, 2, 7, 7, 'Forest'), ...block(2, 8, 7, 8, 'Tundra')]);
    const p = plan(regionsOf(t), t);
    const nearTundra = p.elems.filter((e) => e.kind === 'Forest' && e.tile >= at(0, 7));
    expect(nearTundra.length).toBeGreaterThan(0);
    for (const e of nearTundra) expect(e.sprite).toBe(3);
  });
});

describe('fog of war', () => {
  // The viewer knows only the west half of the map.
  const known = (i: number) => i % W < 10;
  const base = map([...block(2, 2, 14, 9, 'Forest'), ...[4, 5, 6, 7, 8, 9, 10, 11, 12].map((c): [number, number, Terrain] => [c, 12, 'Mountains'])]);
  const other = base.slice();
  // Different (secret) terrain in the unexplored east.
  for (let i = 0; i < W * H; i++) if (!known(i)) other[i] = (['Desert', 'Mountains', 'Forest', 'Hills'] as Terrain[])[i % 4];

  it('builds regions only from explored tiles: unexplored terrain changes nothing', () => {
    const a = regionsOf(base, known);
    const b = regionsOf(other, known);
    expect([...a.id]).toEqual([...b.id]);
    expect([...a.edge]).toEqual([...b.edge]);
    expect([...a.spine]).toEqual([...b.spine]);
    expect(plan(a, base).elems).toEqual(plan(b, other).elems);
  });

  it('never anchors or centres a sprite on an unexplored tile', () => {
    const R = regionsOf(base, known);
    for (const far of [false, true]) {
      const p = plan(R, base, far);
      for (const e of p.elems) {
        expect(known(e.tile)).toBe(true);
        expect(known(nearestTile(e.x, e.y, W, H))).toBe(true);
      }
    }
  });

  it('a known tile beside the unknown is an edge, not interior, so the wood thins there', () => {
    const R = regionsOf(base, known);
    expect(R.edge[at(9, 5)]).toBe(0);
  });

  it('the per-tile signature changes when exploring reveals more of a region, so its chunks repaint', () => {
    const a = regionsOf(base, known);
    const b = regionsOf(base, () => true);
    expect(organicSig(a, at(9, 5))).not.toBe(organicSig(b, at(9, 5)));
    expect(organicSig(a, at(0, 0))).toBe(0);
  });
});

describe('kinds', () => {
  it('only forests, hills and mountains are organic', () => {
    const all: Terrain[] = ['Ocean', 'Coast', 'Grassland', 'Plains', 'Forest', 'Hills', 'Desert', 'Tundra', 'Mountains'];
    expect(all.map(organicKind).filter(Boolean) as OrganicKind[]).toEqual(['Forest', 'Hills', 'Mountains']);
  });
});
