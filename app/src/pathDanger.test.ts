import { describe, expect, it } from 'vitest';
import { enemyReach } from './enemyReach';
import { planTurns, previewPath, previewRoute, routeDangerText } from './path';
import { addUnit, arenaView } from './test/arenaView';

const at = (q: number, r: number) => ({ q, r });
const from = at(10, 10);
const to = at(4, 10);

async function scene(enemy?: { q: number; r: number }, kind: 'Soldier' | 'Horseman' = 'Soldier') {
  const v = await arenaView('Switzerland');
  const me = addUnit(v, 0, 'Horseman', from);
  if (enemy) addUnit(v, 2, kind, enemy);
  return { v, me };
}

describe('routes avoid what known enemies can strike (p881 retreat)', () => {
  it('takes a clear first step when a same-length route has one', async () => {
    const { v, me } = await scene(at(7, 10));
    const plain = previewRoute(v, from, to, [], me.id);
    expect(plain.path![0]).toEqual(at(9, 10));
    const safe = previewRoute(v, from, to, [], me.id, { avoidEnemyReach: true });
    expect(safe.path![0]).not.toEqual(at(9, 10));
    expect(safe.danger).toBeUndefined();
    expect(safe.path!.length).toBe(plain.path!.length);
    const turns = (p: typeof plain.path) => planTurns(v, p!, from, me.max_moves).at(-1)!.turn;
    expect(turns(safe.path)).toBe(turns(plain.path));
    expect(safe.path!.at(-1)).toEqual(to);
  });

  it('warns "caught leaving" when every first step is in reach', async () => {
    const { v, me } = await scene(at(11, 10));
    const r = previewRoute(v, from, to, [], me.id, { avoidEnemyReach: true });
    expect(r.danger).toBe('leaving');
    expect(routeDangerText('leaving')).toBe('Enemies can catch this unit leaving.');
    expect(r.path![0]).toEqual(at(9, 10));
  });

  it('keeps the plain route, with no warning, when nothing threatens it', async () => {
    const { v, me } = await scene(at(10, 3));
    const plain = previewRoute(v, from, to, [], me.id);
    const safe = previewRoute(v, from, to, [], me.id, { avoidEnemyReach: true });
    expect(safe.path).toEqual(plain.path);
    expect(safe.danger).toBeUndefined();
    expect((await scene()).v.units).toHaveLength(1);
  });

  it('leaves the default routing alone (the AI calls previewPath without the option)', async () => {
    const { v, me } = await scene(at(7, 10));
    expect(previewPath(v, from, to, [], me.id)![0]).toEqual(at(9, 10));
    expect(previewPath(v, from, to, [], me.id, {})![0]).toEqual(at(9, 10));
  });

  it('never reroutes an attack, and does not count the destination', async () => {
    const { v, me } = await scene(at(7, 10));
    const attack = previewRoute(v, from, at(7, 10), [], me.id, { avoidEnemyReach: true });
    expect(attack.danger).toBeUndefined();
    expect(attack.path).toEqual(previewRoute(v, from, at(7, 10), [], me.id).path);
    const toNextTo = previewRoute(v, from, at(8, 10), [], me.id, { avoidEnemyReach: true });
    expect(toNextTo.danger).toBeUndefined();
    expect(toNextTo.path!.at(-1)).toEqual(at(8, 10));
  });

  it('keeps the route and reports the danger when the only way is exposed (a one-tile corridor)', async () => {
    const { v, me } = await scene(at(11, 10));
    v.tiles.forEach((t, i) => { if ([8, 9, 11, 12].includes(Math.floor(i / v.width))) t.terrain = 'Mountains'; });
    const r = previewRoute(v, from, to, [], me.id, { avoidEnemyReach: true });
    expect(r.path![0]).toEqual(at(9, 10));
    expect(r.danger).toBe('leaving');
  });

  it('counts only units in the view', async () => {
    const { v } = await scene();
    expect(enemyReach(v).any).toBe(false);
  });

  it('ignores one-move enemies for a unit that has stepped away, and enemies that cannot strike in time', async () => {
    const { v } = await scene(at(8, 10), 'Soldier');
    v.units[1].max_moves = 1;
    const reach = enemyReach(v);
    expect(reach.threatened(at(9, 10), 1)).toBe(false);
    v.units[1].max_moves = 2;
    expect(enemyReach(v).threatened(at(9, 10), 1)).toBe(true);
    // Round 2 needs a third move: a 2-move enemy cannot strike a unit that is already two steps on.
    expect(enemyReach(v).threatened(at(6, 10), 2)).toBe(false);
  });
});
