import { describe, expect, it } from 'vitest';
import { findPath, pathTurns, planTurns, previewPath, previewRoute, roadBonusNote, roadBonusText, type CostFn } from './path';
import { fromOffset, distance, tileIndex } from './hex';
import type { PlayerView, Terrain, TileView } from './types';
import { reachTiles } from './map/reach';

function grid(w: number, h: number, fill: Terrain) {
  const t: Terrain[] = Array(w * h).fill(fill);
  const cost: CostFn = (x) => {
    const i = tileIndex(x, w, h);
    if (i < 0) return null;
    const tt = t[i];
    return tt === 'Mountains' ? null : tt === 'Forest' || tt === 'Hills' ? 2 : 1;
  };
  return { t, cost };
}

describe('pathfinding (mirrors path.rs)', () => {
  it('straight path on open ground', () => {
    const { cost } = grid(10, 10, 'Grassland');
    const p = findPath(10, 10, fromOffset(1, 1), fromOffset(5, 1), cost)!;
    expect(p).toHaveLength(4);
    expect(p[p.length - 1]).toEqual(fromOffset(5, 1));
    // each step adjacent
    let prev = fromOffset(1, 1);
    for (const h of p) {
      expect(distance(prev, h)).toBe(1);
      prev = h;
    }
  });

  it('routes around mountains and fails when walled', () => {
    const { t, cost } = grid(10, 10, 'Grassland');
    for (let row = 0; row < 9; row++) t[tileIndex(fromOffset(4, row), 10, 10)] = 'Mountains';
    const from = fromOffset(1, 5);
    const to = fromOffset(8, 5);
    expect(findPath(10, 10, from, to, cost)!.length).toBeGreaterThan(7);
    t[tileIndex(fromOffset(4, 9), 10, 10)] = 'Mountains';
    expect(findPath(10, 10, from, to, cost)).toBeNull();
  });

  it('prefers cheap ground over forest', () => {
    const { t, cost } = grid(10, 3, 'Grassland');
    for (let c = 2; c < 8; c++) t[tileIndex(fromOffset(c, 1), 10, 3)] = 'Forest';
    const p = findPath(10, 3, fromOffset(1, 1), fromOffset(8, 1), cost)!;
    const total = p.reduce((s, h) => s + cost(h)!, 0);
    expect(total).toBeLessThan(12);
  });

  it('same tile is an empty path; impassable target is null', () => {
    const { t, cost } = grid(5, 5, 'Grassland');
    expect(findPath(5, 5, fromOffset(2, 2), fromOffset(2, 2), cost)).toEqual([]);
    t[tileIndex(fromOffset(4, 4), 5, 5)] = 'Mountains';
    expect(findPath(5, 5, fromOffset(0, 0), fromOffset(4, 4), cost)).toBeNull();
  });

  it('preview only walks explored tiles', () => {
    const w = 6;
    const h = 1;
    const tiles: TileView[] = Array.from({ length: w }, (_, i) => ({
      explored: i !== 3,
      visible: true,
      terrain: i === 3 ? null : 'Plains',
      ruin: false,
      owner: null,
    }));
    const view = { width: w, height: h, tiles } as unknown as PlayerView;
    expect(previewPath(view, fromOffset(0, 0), fromOffset(2, 0))).toHaveLength(2);
    expect(previewPath(view, fromOffset(0, 0), fromOffset(5, 0))).toBeNull();
  });

  it('allows allied city transit but refuses it as the route destination', () => {
    const width = 6, height = 1;
    const tiles: TileView[] = Array.from({ length: width }, () => ({ explored: true, visible: true, terrain: 'Plains', ruin: false, owner: null }));
    tiles[2].owner = 1;
    const city = { id: 1, owner: 1, name: 'Bern', pos: fromOffset(2, 0) };
    const mover = { id: 1, owner: 0, kind: 'Scout', military: false, pos: fromOffset(0, 0), max_moves: 3 };
    const view = {
      width, height, tiles, me: 0, team: 0, units: [mover], cities: [city], players: [{ id: 0, team: 0 }, { id: 1, team: 1 }],
      diplomacy: { relations: [{ a: 0, b: 1, state: 'Alliance' }] },
    } as unknown as PlayerView;
    expect(previewRoute(view, mover.pos, fromOffset(4, 0), [], mover.id).path).toEqual([1, 2, 3, 4].map((q) => fromOffset(q, 0)));
    expect(previewRoute(view, mover.pos, city.pos, [], mover.id).path).toBeNull();
  });

  it('Peaceful borders block a Scout route', () => {
    const width = 6, height = 1;
    const tiles: TileView[] = Array.from({ length: width }, () => ({ explored: true, visible: true, terrain: 'Plains', ruin: false, owner: 1 }));
    tiles[0].owner = 0;
    const city = { id: 1, owner: 1, name: 'Bern', pos: fromOffset(2, 0) };
    const mover = { id: 1, owner: 0, kind: 'Scout' as const, military: true, pos: fromOffset(0, 0), max_moves: 4 };
    const view = {
      width, height, tiles, me: 0, team: 0, units: [mover], cities: [city],
      players: [{ id: 0, team: 0, faction: 'Mongols' }, { id: 1, team: 1, faction: 'India' }],
      diplomacy: { relations: [{ a: 0, b: 1, state: 'Peace' }] },
    } as unknown as PlayerView;
    expect(previewRoute(view, mover.pos, fromOffset(4, 0), [], mover.id).path).toBeNull();
    expect(reachTiles(view, mover).has(tileIndex(city.pos, width, height))).toBe(false);
    expect(reachTiles(view, mover).has(tileIndex(fromOffset(4, 0), width, height))).toBe(false);
  });

  it('crosses an allied unit but does not end on it', () => {
    const width = 5, height = 1;
    const tiles: TileView[] = Array.from({ length: width }, () => ({ explored: true, visible: true, terrain: 'Plains', ruin: false, owner: null }));
    const mover = { id: 1, owner: 0, kind: 'Spearman', military: true, pos: fromOffset(0, 0), max_moves: 2 };
    const ally = { id: 2, owner: 1, kind: 'Spearman', military: true, pos: fromOffset(1, 0), max_moves: 2 };
    const view = { width, height, tiles, me: 0, team: 0, units: [mover, ally], cities: [],
      players: [{ id: 0, team: 0 }, { id: 1, team: 1 }],
      diplomacy: { relations: [{ a: 0, b: 1, state: 'Alliance' }] },
    } as unknown as PlayerView;
    expect(previewRoute(view, mover.pos, fromOffset(2, 0), [], mover.id).path).toEqual([1, 2].map((q) => fromOffset(q, 0)));
    expect(previewRoute(view, mover.pos, ally.pos, [], mover.id).path).toBeNull();
  });

  it('splits paths into turns with the one-step-at-full-moves rule', () => {
    const { t, cost } = grid(10, 1, 'Grassland');
    // G F F G G   (moves 2)
    t[tileIndex(fromOffset(1, 0), 10, 1)] = 'Forest';
    t[tileIndex(fromOffset(2, 0), 10, 1)] = 'Forest';
    const path = [1, 2, 3, 4].map((c) => fromOffset(c, 0));
    const steps = pathTurns(path, cost, 2);
    expect(steps.map((s) => s.turn)).toEqual([1, 2, 3, 3]);
    expect(steps.map((s) => s.endsTurn)).toEqual([true, true, false, true]);
    // A 1-move unit can still enter forest each turn (full-moves rule).
    expect(pathTurns(path.slice(0, 2), cost, 1).map((s) => s.turn)).toEqual([1, 2]);
  });
});

describe('F8 pending movement reservations', () => {
  function scenario() {
    const middle = { q: 4, r: 4 };
    const unit = (id: number, pos: { q: number; r: number }) => ({ id, owner: 0, kind: 'Archer', pos, military: true, max_moves: 2, goto: null });
    const a = unit(1, { q: 5, r: 4 });
    const b = unit(2, { q: 4, r: 5 });
    const goals = [{ q: 4, r: 3 }, { q: 5, r: 3 }];
    const view = { width: 12, height: 10, me: 0, team: 0,
      units: [a, b], cities: [], players: [{ id: 0, team: 0 }],
      tiles: Array.from({ length: 120 }, () => ({ explored: true, visible: true, terrain: 'Grassland', owner: null, ruin: false } satisfies TileView)),
    } as unknown as PlayerView;
    return { middle, a, b, goals, view };
  }

  it('follows the first Ranger through its intermediate hex, even on a confirmed path', () => {
    const { middle, a, b, goals, view } = scenario();
    const first = previewPath(view, a.pos, goals[0], [], a.id)!;
    expect(first[0]).toEqual(middle);
    const pending = [{ type: 'MoveUnitPath' as const, unit: a.id, to: goals[0], path: first }];
    const second = previewPath(view, b.pos, goals[1], pending, b.id)!;
    expect(second).toHaveLength(2);
    expect(second).toContainEqual(middle);
    expect(second.at(-1)).toEqual(goals[1]);
    // Inspecting the earlier order cannot silently replan its confirmed path.
    expect(previewPath(view, a.pos, goals[0], [...pending, { type: 'MoveUnitPath', unit: b.id, to: goals[1], path: second }], a.id)).toEqual(first);
  });

  it('follows ordinary pending gotos and persistent routes, while keeping civilians independent', () => {
    const { middle, a, b, goals, view } = scenario();
    const route = previewPath(view, a.pos, goals[0], [], a.id)!;
    const pending = [{ type: 'MoveUnit' as const, unit: a.id, to: goals[0] }];
    view.units[0].kind = 'Scout';
    expect(previewPath(view, b.pos, goals[1], [{ type: 'SetAutomation', unit: a.id, automation: 'Explore', path: route }], b.id)).toContainEqual(middle);
    view.units[0].kind = 'Archer';
    expect(previewPath(view, b.pos, goals[1], pending, b.id)).toContainEqual(middle);
    Object.assign(view.units[0], { goto: goals[0], path: route });
    expect(previewPath(view, b.pos, goals[1], [], b.id)).toContainEqual(middle);
    view.units[0].military = false;
    expect(previewPath(view, b.pos, goals[1], [], b.id)).toContainEqual(middle);
  });

  it('never uses unexplored detours', () => {
    const { a, b, goals, view } = scenario();
    const route = previewPath(view, a.pos, goals[0], [], a.id)!;
    view.tiles[tileIndex(a.pos, view.width, view.height)] = { explored: false, visible: false, terrain: null, ruin: false, owner: null };
    const planned = previewPath(view, b.pos, goals[1], [{ type: 'MoveUnitPath', unit: a.id, to: goals[0], path: route }], b.id)!;
    for (const step of planned) expect(view.tiles[tileIndex(step, view.width, view.height)].explored).toBe(true);
  });
});

describe('the road bonus is said where the route is previewed (Railways, hidden math)', () => {
  // A 12-tile corridor of Grassland; the unit (2 moves) starts on a road tile.
  function corridor(known: string[]) {
    const w = 12;
    const tiles = Array.from({ length: w }, (_, i) => ({ explored: true, visible: true, terrain: 'Grassland', owner: null, ruin: false, ...(i === 0 ? { road: 64 } : {}) } satisfies TileView));
    return { width: w, height: 1, me: 0, team: 0, units: [], cities: [], players: [{ id: 0, team: 0 }], research: { known }, tiles } as unknown as PlayerView;
  }
  const from = fromOffset(0, 0);
  const path = (n: number) => Array.from({ length: n }, (_, i) => fromOffset(i + 1, 0));
  it('names the bonus: +1 on a road, +2 with Railways', () => {
    expect(roadBonusText(corridor([]))).toBe('+1 move');
    expect(roadBonusText(corridor(['Railways']))).toBe('+2 moves (Railways)');
  });
  it('says so when starting on a road saves a turn, and only then', () => {
    const plain = corridor([]);
    const rail = corridor(['Railways']);
    // 4 tiles at 2 moves: 2 turns without the bonus, 2 with +1 (3 moves), 1 with Railways (4 moves).
    expect(planTurns(plain, path(4), from, 2).at(-1)!.turn).toBe(2);
    expect(planTurns(rail, path(4), from, 2).at(-1)!.turn).toBe(1);
    expect(roadBonusNote(rail, planTurns(rail, path(4), from, 2), from, 2)).toBe('Includes +2 moves (Railways) from starting on a road.');
    expect(roadBonusNote(plain, planTurns(plain, path(4), from, 2), from, 2)).toBeNull();
    // Too short to matter: the same single turn either way.
    expect(roadBonusNote(rail, planTurns(rail, path(2), from, 2), from, 2)).toBeNull();
  });
});
