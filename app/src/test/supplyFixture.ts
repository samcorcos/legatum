// Strategic supply scenes for the screen tour: the improvements capital (population 5, an Iron deposit it owns, so
// 2 Iron supplied, no Coal deposit) with the supply users and the records the core would hold at the end of a turn.
import { ring } from '../hex';
import { spawnUnit } from './fixtureEngine';
import { resolveFixture } from './fixtureEngine';
import type { TechId } from '../types';
import { UPGRADE_TECHS, improvementCity } from './improvementsFixture';

// Industrialization reveals Coal: without it the Factory's Coal is hidden and its shortage has no chip.
const TECHS: TechId[] = [...UPGRADE_TECHS, 'Feudalism', 'Industrialization', 'Metallurgy'];

/**
 * `short`: a Forge and a Factory plus three Pikemen against 2 Iron and no Coal. The Factory is idle (no Coal) and the
 * two newest Pikemen are idle (4 Iron in use of 2 supplied), so both top bar chips show a shortage.
 * `blocked`: two Pikemen use all of the 2 Iron, so the Forge's build card is greyed out with the core's reason.
 */
export function supplyCity(scene: 'short' | 'blocked') {
  const s = improvementCity({ techs: TECHS });
  const c = s.cities[0];
  c.buildings.push('Workshop');
  if (scene === 'short') c.buildings.push('MachineShop', 'Autoworks');
  const spots = ring(c.pos, 1);
  for (const at of scene === 'short' ? [spots[1], spots[2], spots[4]] : [spots[1], spots[2]]) spawnUnit(s, 0, 'Trooper', at, false);
  const after = resolveFixture(s);
  for (const p of s.players) p.supply = after.players[p.id].supply;
  return s;
}
