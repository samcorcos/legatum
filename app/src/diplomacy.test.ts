import { describe, expect, it } from 'vitest';
import { atWar, queueDiplomacy, warDeclaration } from './diplomacy';
import { previewPath } from './path';
import { fromOffset, tileIndex } from './hex';
import { reachTiles } from './map/reach';
import type { Order, PlayerView } from './types';

function view(): PlayerView {
  return {
    me: 0, team: 0, width: 7, height: 7,
    players: [{ id: 0, team: 0 }, { id: 1, team: 1 }, { id: 2, team: 2 }],
    units: [], cities: [],
    tiles: Array.from({ length: 49 }, () => ({ explored: true, visible: true, terrain: 'Grassland', owner: null })),
    diplomacy: { legacy: false, queued: [], proposals: [], actions: [], superseded: [], peace_cooldowns: [],
      relations: [{ a: 0, b: 1, state: 'Peace', effective_turn: 1, guarantee_until: 0, version: 0, war_id: null, declarers: [] }] },
  } as unknown as PlayerView;
}

describe('diplomacy planning permissions', () => {
  it('shows the truce reason when an allied former enemy cannot be betrayed yet', () => {
    const v = view();
    v.diplomacy.relations[0].state = 'Alliance';
    v.diplomacy.relations[0].security = { id: 1, started_turn: 2, protected_until: 12 };
    v.diplomacy.relations[0].guarantee_until = 11;
    v.diplomacy.actions = [
      { order: { type: 'DeclareWar', against: 1, breach: false }, effective_turn: 3, cost: 0, blocked_reason: 'Leave the Alliance first' },
      { order: { type: 'DeclareWar', against: 1, breach: true }, effective_turn: 3, cost: 0, blocked_reason: 'Peace with Athens is guaranteed through turn 10' },
    ];
    expect(warDeclaration(v, 1)).toEqual({ blocked: 'Peace with Athens is guaranteed through turn 10' });
  });
  it('keeps a queued declaration from changing current-turn permissions', () => {
    const v = view();
    const queued = queueDiplomacy([], { type: 'DeclareWar', against: 1 }, v);
    expect(queued).toHaveLength(1);
    expect(atWar(v, 1)).toBe(false);
    v.diplomacy.relations[0].state = 'War';
    expect(atWar(v, 1)).toBe(true);
  });
  it('replaces decisions for the same pair and retains military orders', () => {
    const v = view();
    const initial: Order[] = [{ type: 'SleepUnit', unit: 7 }, { type: 'DeclareWar', against: 1 }, { type: 'DeclareWar', against: 2 }];
    const peace: Order = { type: 'ProposeTreaty', to: 1, kind: 'Peace' };
    expect(queueDiplomacy(initial, peace, v)).toEqual([initial[0], initial[2], peace]);
  });
  it('excludes closed territory and neutral occupants from path and reach previews', () => {
    const v = view(); const from = fromOffset(2, 3); const to = fromOffset(3, 3);
    const i = tileIndex(to, v.width, v.height);
    v.tiles[i].owner = 1;
    expect(previewPath(v, from, to)).toBeNull();
    expect(reachTiles(v, { pos: from, kind: 'Spearman', max_moves: 2 }).has(i)).toBe(false);
    v.tiles[i].owner = null;
    v.units.push({ id: 2, owner: 1, pos: to } as PlayerView['units'][number]);
    expect(previewPath(v, from, to)).toBeNull();
    expect(reachTiles(v, { pos: from, kind: 'Spearman', max_moves: 2 }).has(i)).toBe(false);
  });
  it('allows only decreasing exit distance for a unit left in foreign territory', () => {
    const v = view();
    for (let row = 1; row < 6; row++) for (let col = 1; col < 6; col++) v.tiles[tileIndex(fromOffset(col, row), 7, 7)].owner = 1;
    const center = fromOffset(3, 3); const edge = fromOffset(2, 3);
    expect(previewPath(v, center, edge)).not.toBeNull();
    expect(previewPath(v, edge, center)).toBeNull();
    expect(previewPath(v, edge, fromOffset(0, 3))).not.toBeNull();
  });
});
