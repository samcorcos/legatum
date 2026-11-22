// Promoted units at levels 1-3 beside an enemy veteran, at War, for promotion UI checks.
import { turnFlowFixture } from './turnFlowFixture';
import { spawnUnit } from './fixtureEngine';
import { fromOffset } from '../hex';
import { PROMOTION_XP } from '../promotion';

export function promotionFixture() {
  const s = turnFlowFixture();
  for (let col = 2; col <= 15; col++) for (const row of [3, 5]) s.terrain[row * s.width + col] = 'Plains';
  const xp = (level: number) => PROMOTION_XP[level - 1];
  const soldier = s.units.find((u) => u.id === 0)!;
  soldier.xp = xp(1) + 5;
  const ranger = spawnUnit(s, 0, 'Ranger', fromOffset(5, 4), false);
  ranger.xp = xp(2) + 3;
  const trooper = spawnUnit(s, 0, 'Trooper', fromOffset(6, 4), false);
  trooper.xp = xp(3) + 12;
  const enemy = spawnUnit(s, 1, 'Soldier', fromOffset(7, 4), false);
  enemy.xp = xp(2);
  for (const r of s.diplomacy!.relations) r.state = 'War';
  for (const p of s.players) p.explored.fill(1);
  return { fixture: s, soldier: soldier.id, ranger: ranger.id, trooper: trooper.id, enemy: enemy.id };
}
