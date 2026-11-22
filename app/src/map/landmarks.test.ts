import { describe, expect, it } from 'vitest';
import { landmarkArtUrl, landmarkBox, landmarkBoxes, landmarkCore, landmarksOf, shortLabel } from './landmarks';
import { itemArtUrl } from '../unlockArt';
import { GREAT_WORKS } from '../data';
import { CITY_WORLD } from './terrain';
import { HEX_SIZE } from './grid';
import { tileIndex } from '../hex';
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
      { tile: a, building: 'RelayUplink' as const, owner: 1, of: 0 },
      { tile: b, building: 'CourierHall' as const, owner: 2, of: 0 },
    ];
    expect(landmarksOf(view(sites))).toMatchObject([
      { building: 'RelayUplink', owner: 1, tile: tileIndex(a, W, H) },
      { building: 'CourierHall', owner: 2, tile: tileIndex(b, W, H) },
    ]);
    expect(landmarksOf(view([]))).toEqual([]);
  });

  it('uses the tile record under fog without relying on private city detail', () => {
    const tile = { q: 4, r: 4 };
    const v = view([{ tile, building: 'RelayUplink', owner: 1, of: 0 }]);
    expect(v.tiles[tileIndex(tile, W, H)].visible).toBe(false);
    expect(landmarksOf(v)).toMatchObject([{ city: 3, owner: 1, building: 'RelayUplink', hex: tile }]);
  });

  it('has painted art for every world wonder and the Yam Relay Station, so no stand-in is drawn', () => {
    for (const building of [...GREAT_WORKS, 'CourierHall'] as const) {
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
});

describe('landmark map art and card art', () => {
  // The map keeps the palette-graded painting; cards use the ungraded copy (tools/art/card-grade.py, 2026-11-22).
  it('the map draws the graded copy, the cards the unlock copy', () => {
    for (const building of ['Pyramids', 'HagiaSophia', 'EiffelTower', 'IterReactor', 'RelayUplink', 'WorldExchange', 'PlanetaryFoundry'] as BuildingKind[]) {
      const map = landmarkArtUrl(building, undefined);
      const card = itemArtUrl({ kind: 'Building', building }, undefined, 256);
      expect(map, building).toMatch(/assets\/landmarks\//);
      expect(card, building).toMatch(/assets\/unlocks\//);
    }
  });
  it('a landmark with no graded copy falls back to its unlock art', () => {
    expect(landmarkArtUrl('CourierHall' as BuildingKind, undefined)).toBe(itemArtUrl({ kind: 'Building', building: 'CourierHall' as BuildingKind }, undefined, 256));
  });
});
