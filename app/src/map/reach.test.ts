import { describe, expect, it } from 'vitest';
import { reachTiles } from './reach';
import { distance, fromOffset, tileIndex } from '../hex';
import type { PlayerView, Terrain, TileView } from '../types';

function view(w: number, h: number, fill: Terrain): PlayerView {
  const tiles: TileView[] = Array.from({ length: w * h }, () => ({ explored: true, visible: true, terrain: fill, ruin: false, owner: null }));
  return { width: w, height: h, tiles } as unknown as PlayerView;
}

describe('planning reach', () => {
  it('uses full movement even when the unit spent it all last turn', () => {
    const v = view(9, 9, 'Grassland');
    const pos = fromOffset(4, 4);
    // moves_left 0 is what the view reports after a unit spent both moves last turn.
    const reach = reachTiles(v, { pos, kind: 'Soldier', max_moves: 2, moves_left: 0 } as never);
    expect(reach.size).toBe(18); // every hex within 2 on open ground
    for (const i of reach) {
      const hx = fromOffset(i % 9, Math.floor(i / 9));
      expect(distance(hx, pos)).toBeLessThanOrEqual(2);
    }
    expect(reach.has(tileIndex(pos, 9, 9))).toBe(false);
  });

  it('allows one costly step at full moves, then stops', () => {
    const v = view(9, 9, 'Forest');
    const reach = reachTiles(v, { pos: fromOffset(4, 4), kind: 'Soldier', max_moves: 2 });
    expect(reach.size).toBe(6); // forest costs 2: one step in each direction
  });

  it('skips unexplored and impassable tiles', () => {
    const v = view(9, 9, 'Grassland');
    for (const t of v.tiles) t.explored = false;
    expect(reachTiles(v, { pos: fromOffset(4, 4), kind: 'Soldier', max_moves: 2 }).size).toBe(0);
    const m = view(9, 9, 'Mountains');
    expect(reachTiles(m, { pos: fromOffset(4, 4), kind: 'Soldier', max_moves: 2 }).size).toBe(0);
  });

  it('routes through an allied city but does not show the city as a stopping tile', () => {
    const v = view(8, 1, 'Grassland');
    const cityPos = fromOffset(2, 0);
    v.tiles[tileIndex(cityPos, 8, 1)].owner = 1;
    Object.assign(v, {
      me: 0, team: 0,
      players: [{ id: 0, team: 0 }, { id: 1, team: 1 }],
      diplomacy: { relations: [{ a: 0, b: 1, state: 'Alliance' }] },
      cities: [{ id: 1, owner: 1, name: 'Bern', pos: cityPos }],
      units: [],
    });
    const reach = reachTiles(v, { pos: fromOffset(0, 0), kind: 'Scout', max_moves: 3 });
    expect(reach.has(tileIndex(cityPos, 8, 1))).toBe(false);
    expect(reach.has(tileIndex(fromOffset(3, 0), 8, 1))).toBe(true);
  });
});
