import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { LANDMARK_REF_ZOOM, townCover, landmarkArtUrl, landmarkBox, landmarkBoxes, landmarkCore, landmarksOf, shortLabel } from './landmarks';
import { itemArtUrl } from '../unlockArt';
import { GREAT_WORKS } from '../data';
import { CITY_WORLD } from './terrain';
import { HEX_SIZE } from './grid';
import { hexToPixel, tileIndex } from '../hex';
import type { BuildingKind, Hex, PlayerView } from '../types';

const W = 12, H = 10;
function view(sites: { tile: Hex; building: BuildingKind; owner: number; of: number }[]): PlayerView {
  const tiles = Array.from({ length: W * H }, () => ({ explored: false, visible: false, terrain: null, owner: null })) as PlayerView['tiles'];
  for (const site of sites) {
    tiles[tileIndex(site.tile, W, H)] = { explored: false, visible: false, terrain: null, owner: site.owner,
      landmark: { building: site.building, city: 3, of: site.of } } as PlayerView['tiles'][number];
  }
  return { me: 0, width: W, height: H, tiles, players: [{ id: 0, faction: 'Athens' }], units: [], cities: [] } as unknown as PlayerView;
}

describe('constructed wonder landmarks', () => {
  it('gives every painted Great Work at least the width of a normal-zoom city and moves clear of a banner', () => {
    const zoom = 0.9;
    const hexPx = HEX_SIZE * zoom;
    const centre = { x: 200, y: 200 };
    const ordinary = landmarkBox(centre, hexPx);
    // The Courier Hall is the narrowest painting: 224 of its 256 source pixels.
    expect(ordinary.s * (224 / 256)).toBeGreaterThanOrEqual(CITY_WORLD * zoom);
    const banner = { x: ordinary.x + 20, y: ordinary.y, w: 70, h: 25 };
    const placed = landmarkBox(centre, hexPx, [banner]);
    const overlap = Math.max(0, Math.min(placed.x + placed.s, banner.x + banner.w) - Math.max(placed.x, banner.x))
      * Math.max(0, Math.min(placed.y + placed.s, banner.y + banner.h) - Math.max(placed.y, banner.y));
    expect(overlap).toBe(0);
  });
  it('draws each public wonder at its stored tile, including a rival holder', () => {
    const a = { q: 3, r: 4 }, b = { q: 6, r: 5 };
    const sites = [
      { tile: a, building: 'GreatLibrary' as const, owner: 1, of: 0 },
      { tile: b, building: 'YamRelayStation' as const, owner: 2, of: 0 },
    ];
    expect(landmarksOf(view(sites))).toMatchObject([
      { building: 'GreatLibrary', owner: 1, tile: tileIndex(a, W, H) },
      { building: 'YamRelayStation', owner: 2, tile: tileIndex(b, W, H) },
    ]);
    expect(landmarksOf(view([]))).toEqual([]);
  });

  it('uses the tile record under fog without relying on private city detail', () => {
    const tile = { q: 4, r: 4 };
    const v = view([{ tile, building: 'GreatLibrary', owner: 1, of: 0 }]);
    expect(v.tiles[tileIndex(tile, W, H)].visible).toBe(false);
    expect(landmarksOf(v)).toMatchObject([{ city: 3, owner: 1, building: 'GreatLibrary', hex: tile }]);
  });

  it('has painted art for every world wonder and the Yam Relay Station, so no stand-in is drawn', () => {
    for (const building of [...GREAT_WORKS, 'YamRelayStation'] as const) {
      for (const size of [64, 128, 256] as const) {
        expect(String(itemArtUrl({ kind: 'Building', building }, undefined, size)), `${building} ${size}`).toContain(`building-${building.toLowerCase()}-${size}`);
      }
    }
  });

  it('shortens a long wonder label so it stays on its tile', () => {
    expect(shortLabel('ITER Fusion Reactor')).toBe('ITER Reactor');
    expect(shortLabel('Great Library')).toBe('Great Library');
    expect(shortLabel('Hagia Sophia')).toBe('Hagia Sophia');
  });

  it('keeps a site at the same footprint across consecutive frames as a unit passes beside it', () => {
    const sites = [{ tile: 5, hex: { q: 2, r: 2 } }, { tile: 6, hex: { q: 3, r: 2 } }];
    const toScreen = (p: { x: number; y: number }) => p;
    const banners = [{ x: 30, y: 20, w: 70, h: 25 }];
    const first = landmarkBoxes(sites, toScreen, 40, banners);
    const unitBeside = { x: first.get(5)!.x + 20, y: first.get(5)!.y + 30, w: 30, h: 30 };
    expect(landmarkCore(first.get(5)!)).toBeDefined();
    expect(unitBeside.x).toBeGreaterThan(first.get(5)!.x);
    const next = landmarkBoxes(sites, toScreen, 40, banners);
    expect(next.get(5)).toEqual(first.get(5));
    expect(next.get(6)).toEqual(first.get(6));
    expect(next.get(5)).not.toEqual(next.get(6));
  });
  it('keeps a wonder on the same spot of its tile at every zoom and pan (Sam, 2026-11-22: it hopped a hex while zooming)', () => {
    // A site beside a city whose banner sits over the centred footprint, as beside Patna and Ahmedabad.
    const hex = { q: 5, r: 4 };
    const world = hexToPixel(hex, HEX_SIZE);
    const ref = { x: world.x * LANDMARK_REF_ZOOM, y: world.y * LANDMARK_REF_ZOOM };
    const refPx = HEX_SIZE * LANDMARK_REF_ZOOM;
    const banner = { x: ref.x - 60, y: ref.y - refPx * 1.1, w: 120, h: 26 };
    const spotOf = (zoom: number, panX: number, panY: number) => {
      const toScreen = (p: { x: number; y: number }) => ({ x: (p.x - panX) * zoom + 500, y: (p.y - panY) * zoom + 300 });
      const hexPx = HEX_SIZE * zoom;
      const c = toScreen(world);
      const b = landmarkBoxes([{ tile: 9, hex }], toScreen, hexPx, [banner]).get(9)!;
      expect(b.s).toBeCloseTo(hexPx * 3);
      return [Math.round((b.x - (c.x - b.s / 2)) / hexPx * 100) / 100, Math.round((b.y - (c.y - b.s * 0.36)) / hexPx * 100) / 100];
    };
    const at = spotOf(LANDMARK_REF_ZOOM, world.x, world.y);
    expect(at).not.toEqual([0, 0]);
    for (const zoom of [0.62, 0.9, 1.3, 1.8, 2.4]) {
      for (const [px, py] of [[world.x, world.y], [world.x + 400, world.y - 250], [world.x - 900, world.y + 600]]) expect(spotOf(zoom, px, py), `zoom ${zoom}`).toEqual(at);
    }
  });
  it('only the landmark footprint uses the reference-zoom banners; labels, badges and medallions avoid the banners as drawn', () => {
    // PR #81 first fed the reference covers to everything that keeps clear of banners, in the wrong coordinates.
    const src = readFileSync(resolve(__dirname, 'render.ts'), 'utf8');
    expect(src).toMatch(/const bannerCovers = banners\.map\(\(b\) => b\.cover\);/);
    const uses = src.split('referenceBannerCovers(').length - 1;
    expect(uses, 'the definition and the one landmarkBoxes call').toBe(2);
    expect(src).toMatch(/landmarkBoxes\([^;]*referenceBannerCovers\(/);
  });
  it('keeps a wonder beside a city off the painted town', () => {
    const city = { q: 4, r: 4 }, site = { q: 5, r: 4 };
    const refPx = HEX_SIZE * LANDMARK_REF_ZOOM;
    const ref = (h: Hex) => { const p = hexToPixel(h, HEX_SIZE); return { x: p.x * LANDMARK_REF_ZOOM, y: p.y * LANDMARK_REF_ZOOM }; };
    const town = townCover(ref(city), refPx);
    const b = landmarkBoxes([{ tile: 1, hex: site }], (p) => ({ x: p.x * LANDMARK_REF_ZOOM, y: p.y * LANDMARK_REF_ZOOM }), refPx, [town]).get(1)!;
    const core = landmarkCore(b);
    expect(Math.max(0, Math.min(core.x + core.w, town.x + town.w) - Math.max(core.x, town.x))).toBe(0);
  });
});

describe('landmark map art and card art', () => {
  // The map keeps the palette-graded painting; cards use the ungraded copy (tools/art/card-grade.py, 2026-11-22).
  it('the map draws the graded copy, the cards the unlock copy', () => {
    for (const building of ['Pyramids', 'HagiaSophia', 'EiffelTower', 'IterFusionReactor', 'GreatLibrary', 'GrandBazaar', 'RoyalArsenal'] as BuildingKind[]) {
      const map = landmarkArtUrl(building, undefined);
      const card = itemArtUrl({ kind: 'Building', building }, undefined, 256);
      expect(map, building).toMatch(/assets\/landmarks\//);
      expect(card, building).toMatch(/assets\/unlocks\//);
    }
  });
  it('a landmark with no graded copy falls back to its unlock art', () => {
    expect(landmarkArtUrl('YamRelayStation' as BuildingKind, undefined)).toBe(itemArtUrl({ kind: 'Building', building: 'YamRelayStation' as BuildingKind }, undefined, 256));
  });
});
