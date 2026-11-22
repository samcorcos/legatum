import { describe, expect, it } from 'vitest';
import type { PlayerView } from './types';
import { empireTitle } from './empires';
import { publicRelationsModel } from './relations';

function view(): PlayerView {
  return {
    team: 0, me: 0, turn: 48,
    players: [
      { id: 0, team: 0, name: 'AI 0', faction: 'Byzantium', alive: true },
      { id: 1, team: 1, name: 'Swiss envoy', faction: 'Switzerland', alive: true },
      { id: 2, team: 2, name: 'AI 2', faction: 'Sparta', alive: true },
      { id: 3, team: 3, name: 'AI 3', faction: 'Athens', alive: true },
    ],
    standings: [0, 1, 2, 3].map((team) => ({ team, members: [team], vp: 1, target: 10 })),
    diplomacy: {
      met: [1, 2],
      relations: [
        { a: 0, b: 1, state: 'Peace', effective_turn: 8, guarantee_until: 0, version: 0, war_id: null, declarers: [] },
        { a: 0, b: 2, state: 'War', effective_turn: 41, guarantee_until: 0, version: 0, war_id: 7, declarers: [0] },
        { a: 1, b: 2, state: 'Peace', effective_turn: 2, guarantee_until: 0, version: 0, war_id: null, declarers: [] },
        { a: 1, b: 3, state: 'War', effective_turn: 44, guarantee_until: 0, version: 0, war_id: 8, declarers: [1] },
      ],
    },
  } as unknown as PlayerView;
}

describe('public diplomacy relations', () => {
  it('shows pair states and turns only for empires the viewer has met', () => {
    const v = view();
    const model = publicRelationsModel(v);
    expect(model.empires.map((e) => e.name)).toEqual(['Your empire', empireTitle(v, 1), empireTitle(v, 2), 'Unknown empire 1']);
    expect(model.cells[0][2]).toMatchObject({ state: 'War', title: `Your empire and ${empireTitle(v, 2)}: at War since turn 41.` });
    expect(model.cells[1][2]).toMatchObject({ state: 'Peace', title: `${empireTitle(v, 1)} and ${empireTitle(v, 2)}: at Peace since turn 2.` });
    expect(model.cells[1][3]).toMatchObject({ state: 'Unknown', title: 'Unknown relation' });
    expect(model.cells[2][3].state).toBe('Unknown');
  });
});
