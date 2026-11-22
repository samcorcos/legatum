import { expect, it } from 'vitest';
import { rangedClear, shotBlockReason, viewTerrain, walledGarrisonAt } from './los';
import type { PlayerView } from './types';

it('names a line-of-sight blocker by the nearest known city', () => {
  const tiles = Array.from({ length: 100 }, () => ({ terrain: 'Plains' }));
  tiles[12].terrain = 'Forest';
  const view = { width: 10, height: 10, tiles, cities: [{ name: 'Waystation', pos: { q: 1, r: 1 } }] } as unknown as PlayerView;
  const reason = shotBlockReason(view, { q: 1, r: 1 }, { q: 3, r: 1 });
  expect(reason).toContain('Forest 1 tile E of Waystation blocks the shot');
  expect(reason).not.toMatch(/\(\d+, \d+\)/);
});


it('Steel guns ignore terrain blockers, while earlier ranged units still need a clear line', () => {
  const tiles = Array.from({ length: 100 }, () => ({ terrain: 'Plains' }));
  tiles[12].terrain = 'Forest';
  const view = { width: 10, height: 10, tiles, cities: [], research: { known: ['Steel'] } } as unknown as PlayerView;
  const from = { q: 1, r: 1 }, to = { q: 3, r: 1 };
  expect(shotBlockReason(view, from, to, 'Artillery')).toBeNull();
  expect(shotBlockReason(view, from, to, 'CombatDrone')).toBeNull();
  expect(shotBlockReason(view, from, to, 'FieldCannon')).toContain('No line of sight');
  view.research.known = [];
  expect(shotBlockReason(view, from, to, 'Artillery')).toContain('No line of sight');
});

it('walled garrisons bypass Forest, while unwalled cities and field hills keep ordinary LOS', () => {
  const tiles = Array.from({ length: 100 }, () => ({ terrain: 'Plains' }));
  tiles[12].terrain = 'Forest';
  const from = { q: 1, r: 1 }, to = { q: 3, r: 1 };
  const view = { me: 0, width: 10, height: 10, tiles,
    cities: [{ owner: 0, pos: from, walls: false, name: 'Home' }], research: { known: [] } } as unknown as PlayerView;
  expect(shotBlockReason(view, from, to, 'Archer')).toContain('blocks the shot. Fire from a hill, move adjacent, or build Walls in Home.');
  expect(rangedClear('Archer', from, to, viewTerrain(view))).toBe(false);
  view.cities[0].walls = true;
  expect(walledGarrisonAt(view, 0, from)).toBe(true);
  expect(shotBlockReason(view, from, to, 'Archer')).toBeNull();
  expect(rangedClear('Archer', from, to, viewTerrain(view), false, true)).toBe(true);
  view.tiles[11].terrain = 'Hills';
  view.tiles[12].terrain = 'Mountains';
  const field = { q: 1, r: 1 };
  view.cities[0].pos = { q: 0, r: 0 };
  expect(walledGarrisonAt(view, 0, field)).toBe(false);
  expect(shotBlockReason(view, field, to, 'Archer')).toContain('No line of sight');
});
