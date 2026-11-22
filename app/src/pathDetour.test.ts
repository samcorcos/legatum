// Geometry reconstructed by hash-checked replay of the turn-38 archive at Tessitura.
import { describe, expect, it } from 'vitest';
import type { Order, PlayerView, UnitView } from './types';
import { tileIndex } from './hex';
import { previewPath, previewRoute, routeRiskText } from './path';

const from = { q: 20, r: 14 }, gap = { q: 21, r: 14 }, target = { q: 22, r: 14 };
const unit = (id: number, owner: number, pos: typeof from): UnitView => ({
  id, owner, pos, kind: 'Soldier', military: true, max_moves: 2, goto: null,
} as UnitView);

function tessitura(): PlayerView {
  const width = 48, height = 32;
  const tiles = Array.from({ length: width * height }, () => ({ explored: true, visible: true, terrain: 'Mountains', owner: null }));
  const rows = [
    ['Mountains', 'Desert', 'Desert', 'Desert', 'Desert'],
    ['Plains', 'Plains', 'Plains', 'Desert', 'Desert'],
    ['Plains', 'Plains', 'Plains', 'Desert', 'Desert'],
    ['Plains', 'Plains', 'Plains', 'Plains', 'Desert'],
    ['Plains', 'Plains', 'Mountains', 'Mountains', 'Plains'],
  ];
  rows.forEach((row, ri) => row.forEach((terrain, qi) => {
    const hex = { q: qi + 19, r: ri + 11 };
    tiles[tileIndex(hex, width, height)] = { explored: true, visible: true, terrain, owner: null };
  }));
  const players = [{ id: 0, team: 0, faction: 'Sparta', name: 'Player 1' }, { id: 7, team: 7, faction: 'India', name: 'AI 8' }];
  return {
    turn: 38, me: 0, team: 0, width, height, tiles,
    players, diplomacy: { legacy: false, relations: [{ a: 0, b: 7, state: 'War' }] },
    units: [unit(3, 0, from), unit(26, 0, { q: 21, r: 13 }), unit(31, 0, { q: 20, r: 13 }), unit(24, 7, target)],
    cities: [{ id: 12, owner: 0, name: 'Tessitura', pos: { q: 21, r: 13 } }],
  } as unknown as PlayerView;
}

describe('Tessitura attack at turn 38', () => {
  // The order sequence that reproduced the archived five-step path exactly: the city's
  // Oathbearer (id 26) was first sent at the same enemy through the shared hex. Core declares
  // in unit-id order, so unit 3 takes that hex first; 26's route must not push 3 around the city.
  it('takes the shared hex while a higher-id friend is ordered through it', () => {
    const view = tessitura();
    const route26 = previewPath(view, { q: 21, r: 13 }, target, [], 26);
    expect(route26).toEqual([gap, target]);
    const orders: Order[] = [{ type: 'MoveUnitPath', unit: 26, to: target, path: route26! }];
    expect(previewRoute(view, from, target, orders, 3)).toEqual({ path: [gap, target], risk: null });
    // ...and 26 then routes around 3's declared step instead.
    const after: Order[] = [{ type: 'MoveUnitPath', unit: 3, to: target, path: [gap, target] }];
    expect(previewPath(view, { q: 21, r: 13 }, target, after, 26)?.[0]).not.toEqual(gap);
  });

  it('ignores a later unit’s speculative reservation of the empty shared neighbour', () => {
    const view = tessitura();
    const orders: Order[] = [
      { type: 'MoveUnitPath', unit: 26, to: { q: 22, r: 13 }, path: [gap, { q: 22, r: 13 }] },
      { type: 'MoveUnitPath', unit: 31, to: { q: 23, r: 10 }, path: [{ q: 20, r: 12 }, { q: 21, r: 11 }, { q: 22, r: 10 }, { q: 23, r: 10 }] },
    ];
    expect(previewRoute(view, from, target, orders, 3)).toEqual({ path: [gap, target], risk: null });
  });

  it('follows an earlier unit through a transit step, but ignores its later step', () => {
    const view = tessitura();
    view.units.find((u) => u.id === 26)!.id = 1;
    const later: Order[] = [{ type: 'MoveUnitPath', unit: 1, to: target,
      path: [{ q: 22, r: 13 }, gap, target] }];
    expect(previewRoute(view, from, target, later, 3)).toEqual({ path: [gap, target], risk: null });
    view.units.find((u) => u.id === 1)!.pos = { q: 20, r: 15 };
    const first: Order[] = [{ type: 'MoveUnitPath', unit: 1, to: { q: 22, r: 13 },
      path: [gap, { q: 22, r: 13 }] }];
    const following = previewRoute(view, from, target, first, 3);
    expect(following.path?.[0]).toEqual(gap);
    expect(routeRiskText(view, following.risk!)).toBe('Moves through this tile right behind your Spartan Hoplite.');
  });

  it('names Spartan Hoplite when the shared neighbour is occupied and must clear first', () => {
    const view = tessitura();
    view.units.find((u) => u.id === 26)!.pos = gap;
    const orders: Order[] = [{ type: 'MoveUnit', unit: 26, to: { q: 22, r: 13 } }];
    const result = previewRoute(view, from, target, orders, 3);
    expect(result.path).toEqual([gap, target]);
    expect(result.risk).toMatchObject({ hex: gap, unit: { id: 26 }, stays: false });
    expect(routeRiskText(view, result.risk!)).toBe('Path may be blocked by your Spartan Hoplite: it must move out first.');
    const parked = previewRoute(view, from, target, [], 3);
    expect(parked.path?.[0]).not.toEqual(gap);
    expect(routeRiskText(view, parked.risk!)).toBe('Direct path blocked by your Spartan Hoplite, which stays put this turn: taking a longer route.');
  });
});
