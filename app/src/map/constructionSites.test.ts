import { describe, expect, it } from 'vitest';
import { constructionStage, sitesOf } from './constructionSites';
import { landmarksOf } from './landmarks';
import { tileIndex } from '../hex';
import type { PlayerView } from '../types';

const tile = { q: 4, r: 3 };
const city = { q: 3, r: 3 };
function view(k: number, rival = false): PlayerView {
  const tiles = Array.from({ length: 100 }, () => ({ explored: true, visible: !rival, owner: rival ? 1 : 0 })) as PlayerView['tiles'];
  return { me: 0, width: 10, height: 10, tiles, players: [{ id: 0, era: 2 }, { id: 1, era: 2 }],
    cities: [{ id: 7, owner: rival ? 1 : 0, pos: city, name: 'Bellinzona', visible: !rival,
      construction: { building: 'GreatLibrary', tile, stage: constructionStage(k), progress: Math.round(k * 1000) },
      detail: rival ? null : { producing: { kind: 'Wonder', building: 'GreatLibrary', tile }, production_stored: Math.round(k * 600), production_cost: 600 } }] } as unknown as PlayerView;
}

describe('construction sites', () => {
  it('uses the chosen tile at every threshold and keeps it for the finished landmark', () => {
    for (const [k, stage] of [[0, 0], [.1, 1], [.4, 2], [.75, 3]] as const) {
      const v = view(k);
      expect(sitesOf(v)[0]).toMatchObject({ tile: tileIndex(tile, 10, 10), stage, k });
      expect(sitesOf(v)).toBe(sitesOf(v));
    }
    const done = view(.75);
    done.cities[0].construction = null;
    done.cities[0].detail = null;
    done.tiles[tileIndex(tile, 10, 10)].landmark = { building: 'GreatLibrary', city: 7, of: 0 };
    expect(landmarksOf(done)[0].tile).toBe(tileIndex(tile, 10, 10));
  });

  it('renders a rival site from public progress while the city is in fog', () => {
    const site = sitesOf(view(.45, true))[0];
    expect(site).toMatchObject({ owner: 1, hex: tile, stage: 2, k: .45 });
  });
});
