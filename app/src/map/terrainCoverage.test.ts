// Every painted feature tile still reads as its type (user, 2026-10-17: forests, playtest 8.9.2: "the 3 hexes that are
// technically part of the same forest kind of look like those hexes are not forests"; hills, playtest 8.9.3: "I didn't
// realise these were hills because only a small amount of the hex is covered in hills... if there are similar issues
// elsewhere in the game fix those too"). Coverage is pixel coverage: the share of a hex's area that sits on an opaque pixel
// of a sprite the plan paints, read from the shipped sprite PNGs, so it measures what the player sees.
// Before the fixes: wood edge hexes 0.55 (one-hex arms 0.25, groves 0.5), hills edge hexes 0.28 (arms 0.25, a lone hill 0.33),
// mountains already 0.45+ (their peaks are big and opaque).
// Desert and Tundra are not painted by this layer (a tinted biome colour with a small glyph in terrain.ts), so they are not
// measured here.
import { describe, expect, it } from 'vitest';
import { HEX_SIZE, K, centerX, centerY, neighborIndex } from './grid';
import { computeRegions, organicKind, planOrganic, type OrganicElem, type OrganicKind, type Plan, type Regions } from './organicTerrain';
import { alphaOf } from '../test/pngAlpha';
import type { Terrain } from '../types';

const W = 24;
const H = 20;
const at = (c: number, r: number) => r * W + c;
const SIDE = 58 * K;
const load = (dir: string, names: string[]) => names.map((n) => alphaOf(new URL(`../../public/terrain/${dir}/128/${n}.png`, import.meta.url).pathname));
const SPRITES: Record<OrganicKind, ReturnType<typeof load>> = {
  Forest: load('forests', ['forest-mixed-01', 'forest-mixed-02', 'forest-mixed-03', 'forest-tundra-conifers']),
  Hills: load('hills', ['downs-01', 'downs-02', 'downs-03', 'downs-04']),
  Mountains: load('mountains', ['mountain-01', 'mountain-02', 'mountain-03', 'mountain-04']),
};

function fixture(kind: Terrain, paint: (c: number, r: number) => boolean): { regions: Regions; plan: Plan } {
  const terrain: Terrain[] = [];
  for (let r = 0; r < H; r++) for (let c = 0; c < W; c++) terrain.push(paint(c, r) ? kind : 'Grassland');
  const regions = computeRegions(W, H, (i) => organicKind(terrain[i]));
  return { regions, plan: planOrganic({ regions, isCity: () => false, tundraNear: () => false, far: false }) };
}

/** Share of the tile's area (a 2-unit grid inside the hex) lying on an opaque sprite pixel of `kind`; `part` limits it to a region of the hex. */
function coverage(plan: Plan, kind: OrganicKind, tile: number, part: (dx: number, dy: number) => boolean = () => true): number {
  const cx = centerX(tile, W);
  const cy = centerY(tile, W);
  const near = plan.elems.filter((e) => e.kind === kind && Math.hypot(e.x - cx, e.y - cy) < HEX_SIZE + SIDE * 1.6);
  const opaque = (e: OrganicElem, x: number, y: number) => {
    const s = SPRITES[kind][e.sprite];
    const side = SIDE * e.scale;
    const u = Math.floor(((x - (e.x - side / 2)) / side) * s.width);
    const v = Math.floor(((y - (e.y - side / 2)) / side) * s.height);
    return u >= 0 && v >= 0 && u < s.width && v < s.height && s.alpha[v * s.width + u] > 100;
  };
  let hit = 0;
  let n = 0;
  for (let y = -HEX_SIZE; y <= HEX_SIZE; y += 2) {
    for (let x = -HEX_SIZE; x <= HEX_SIZE; x += 2) {
      const ax = Math.abs(x);
      if (ax > (Math.sqrt(3) / 2) * HEX_SIZE || Math.abs(y) > HEX_SIZE - ax / Math.sqrt(3) || !part(x, y)) continue;
      n++;
      if (near.some((e) => opaque(e, cx + x, cy + y))) hit++;
    }
  }
  return hit / n;
}

/** Sum of the vectors from the tile to its neighbours outside its region (the way the open ground lies), and how many there are. */
function openSide(regions: Regions, i: number) {
  let x = 0;
  let y = 0;
  let n = 0;
  for (let e = 0; e < 6; e++) {
    const j = neighborIndex(i, e, W, H);
    if (j < 0 || regions.id[j] === regions.id[i]) continue;
    x += centerX(j, W) - centerX(i, W);
    y += centerY(j, W) - centerY(i, W);
    n++;
  }
  return { x, y, n };
}
const mean = (v: number[]) => v.reduce((a, b) => a + b, 0) / v.length;

/** The floors per kind: edge hexes of a 12x10 block (mean, minimum), its interior (mean), a one-hex arm, a lone hex, a three-hex grove. */
const FLOORS: Record<OrganicKind, { edgeMean: number; edgeMin: number; interior: number; arm: number; one: number; grove: number; thinner: boolean }> = {
  Forest: { edgeMean: 0.68, edgeMin: 0.5, interior: 0.9, arm: 0.45, one: 0.55, grove: 0.6, thinner: true },
  Hills: { edgeMean: 0.68, edgeMin: 0.4, interior: 0.8, arm: 0.5, one: 0.6, grove: 0.6, thinner: true },
  // A range's peaks are big and opaque, so its edges are not thinned: the floors only guard against a regression.
  Mountains: { edgeMean: 0.6, edgeMin: 0.4, interior: 0.65, arm: 0.6, one: 0.6, grove: 0.45, thinner: false },
};

for (const kind of ['Forest', 'Hills', 'Mountains'] as const) {
  describe(`${kind} coverage`, () => {
    const block = fixture(kind, (c, r) => c >= 6 && c <= 17 && r >= 5 && r <= 14);
    const tiles = (d: (i: number) => boolean) => [...Array(W * H).keys()].filter((i) => block.regions.id[i] >= 0 && d(i));
    const f = FLOORS[kind];

    it('edge hexes of a big region keep most of their art, interiors stay fuller', () => {
      const edge = tiles((i) => block.regions.edge[i] === 0).map((i) => coverage(block.plan, kind, i));
      const interior = tiles((i) => block.regions.edge[i] >= 2).map((i) => coverage(block.plan, kind, i));
      expect(mean(edge)).toBeGreaterThan(f.edgeMean);
      expect(Math.min(...edge)).toBeGreaterThan(f.edgeMin);
      expect(mean(interior)).toBeGreaterThan(f.interior);
      // Edges still thin: the region is not a block of identical hexes.
      if (f.thinner) expect(mean(edge)).toBeLessThan(mean(interior) - 0.05);
    });

    it('a one-hex-wide arm, a lone hex and a three-hex group still read as the type', () => {
      const arm = fixture(kind, (c, r) => r === 8 && c >= 4 && c <= 18);
      for (let c = 4; c <= 18; c++) expect(coverage(arm.plan, kind, at(c, 8))).toBeGreaterThan(f.arm);
      const one = fixture(kind, (c, r) => r === 8 && c === 8);
      expect(coverage(one.plan, kind, at(8, 8))).toBeGreaterThan(f.one);
      const grove = fixture(kind, (c, r) => r === 8 && c >= 8 && c <= 10);
      for (let c = 8; c <= 10; c++) expect(coverage(grove.plan, kind, at(c, 8))).toBeGreaterThan(f.grove);
    });

    if (kind === 'Mountains') return;
    it('the gap faces the open neighbour, not the middle of the hex', () => {
      const sides = tiles((i) => block.regions.edge[i] === 0 && openSide(block.regions, i).n === 1);
      expect(sides.length).toBeGreaterThan(4);
      const part = (i: number, inlandHalf: boolean) => {
        const o = openSide(block.regions, i);
        const len = Math.hypot(o.x, o.y);
        return coverage(block.plan, kind, i, inlandHalf ? (dx, dy) => (dx * o.x + dy * o.y) / len < -HEX_SIZE * 0.15 : (dx, dy) => (dx * o.x + dy * o.y) / len > HEX_SIZE * 0.45);
      };
      const inland = mean(sides.map((i) => part(i, true)));
      const outer = mean(sides.map((i) => part(i, false)));
      expect(inland).toBeGreaterThan(0.85 * f.interior);
      expect(outer).toBeLessThan(inland - 0.15);
    });
  });
}
