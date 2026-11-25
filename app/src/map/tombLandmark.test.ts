import { describe, expect, it } from 'vitest';
import { tombPieces, tombTile } from './tombLandmark';
import { constructionStage } from './constructionSites';
import { neighbors, tileIndex } from '../hex';
import type { PlayerView } from '../types';

const capital = { q: 4, r: 4 };
function view(opts: { turns?: number; work?: boolean; standing?: boolean } = {}): PlayerView {
  const tiles = Array.from({ length: 100 }, () => ({ explored: true, visible: true, owner: 1, terrain: 'Desert' })) as PlayerView['tiles'];
  const g = { of: 1, holder: 1, city: 3, at: capital, name: 'Memphis' };
  return { me: 0, width: 10, height: 10, tiles,
    players: [{ id: 0 }, { id: 1, tomb_work: opts.work ?? true, tomb_turns: opts.turns ?? 0 }],
    cities: [{ id: 3, owner: 1, pos: capital, name: 'Memphis' }],
    royal_tombs: opts.standing ? [g] : [], tomb_sites: opts.standing ? [] : [g] } as unknown as PlayerView;
}

describe('the Royal Tomb on the map', () => {
  it('stands on the first neighbouring tile of the holder\'s dry land, never on a town, water or another landmark', () => {
    const v = view();
    const [first, second] = neighbors(capital);
    expect(tombTile(v, capital, 1, new Set())?.hex).toEqual(first);
    v.tiles[tileIndex(first, 10, 10)].terrain = 'Coast';
    expect(tombTile(v, capital, 1, new Set())?.hex).toEqual(second);
    expect(tombTile(v, capital, 1, new Set([tileIndex(second, 10, 10)]))?.hex).not.toEqual(second);
    // Land another empire owns is not the holder's to build on.
    for (const n of neighbors(capital)) v.tiles[tileIndex(n, 10, 10)].owner = 0;
    expect(tombTile(v, capital, 1, new Set())).toBeNull();
  });

  it('is staged by the work\'s public turns, then stands finished', () => {
    for (const [turns, stage] of [[0, 0], [2, 1], [7, 2], [12, 3], [15, 3]] as const) {
      expect(constructionStage(tombPieces(view({ turns }))[0].k), `turn ${turns}`).toBe(stage);
    }
    expect(tombPieces(view({ standing: true }))[0].k).toBe(1);
    expect(constructionStage(tombPieces(view({ standing: true }))[0].k)).toBe(4);
  });
});
