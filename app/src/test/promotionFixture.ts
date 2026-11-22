// Promoted units at levels 1-3 beside an enemy veteran, at War, for promotion UI checks.
import { turnFlowFixture } from './turnFlowFixture';
import { spawnUnit } from './fixtureEngine';
import { fromOffset } from '../hex';
import { PROMOTION_XP } from '../promotion';

export function promotionFixture() {
  const s = turnFlowFixture();
  for (let col = 2; col <= 15; col++) for (const row of [3, 5]) s.terrain[row * s.width + col] = 'Plains';
  const xp = (level: number) => PROMOTION_XP[level - 1];
  const spearman = s.units.find((u) => u.id === 0)!;
  spearman.xp = xp(1) + 5;
  const archer = spawnUnit(s, 0, 'Archer', fromOffset(5, 4), false);
  archer.xp = xp(2) + 3;
  const pikeman = spawnUnit(s, 0, 'Pikeman', fromOffset(6, 4), false);
  pikeman.xp = xp(3) + 12;
  const enemy = spawnUnit(s, 1, 'Spearman', fromOffset(7, 4), false);
  enemy.xp = xp(2);
  for (const r of s.diplomacy!.relations) r.state = 'War';
  for (const p of s.players) p.explored.fill(1);
  return { fixture: s, spearman: spearman.id, archer: archer.id, pikeman: pikeman.id, enemy: enemy.id };
}
