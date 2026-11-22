import { fromOffset, neighbors, within } from '../hex';
import { coreFixture } from './coreFixture';
import { createState, makeStandardSetup, spawnUnit } from './fixtureEngine';
import { idxOf, refreshVision } from './fixtureEngine';

/** Playtest G2: the human's Soldier stands next to an AI Scout of a team still at Peace. */
export const WAR_PROMPT = { soldier: fromOffset(9, 8), scout: neighbors(fromOffset(9, 8))[3] };
/** At War instead, with the Scout two tiles away, so it can move before the attack lands. */
export const PURSUIT = { soldier: fromOffset(9, 8), scout: neighbors(neighbors(fromOffset(9, 8))[0])[0] };

export function warPromptCoreSave(war = false, ranged = false) {
  const setup = makeStandardSetup('700041', 'Celts', 'Switzerland', 'Ada', 'Swiss envoy');
  setup.width = 28; setup.height = 18;
  setup.players = setup.players.slice(0, 2).map((p, id) => ({ ...p, team: id, is_human: id === 0 }));
  const s = createState(setup);
  s.terrain.fill('Grassland'); s.owner.fill(-1); s.tileCity.fill(-1); s.ruin.fill(0); s.wonder = undefined;
  s.units = []; s.nextUnitId = 0;
  s.cities = [0, 1].map((owner) => ({ id: owner, owner, founder: owner, name: owner ? 'Harmonia' : 'Astra',
    pos: fromOffset(3 + owner * 20, 8), population: 2, food: 0, prod: 0, producing: null, bought: null,
    buildings: [], hp: 200, grownTiles: [], tilesClaimed: 0, isCapital: true, foundedTurn: 1, resistance: 0 }));
  s.nextCityId = 2;
  for (const c of s.cities) for (const h of within(c.pos, 1)) {
    const i = idxOf(s, h); s.owner[i] = c.owner; s.tileCity[i] = c.id;
  }
  for (const p of s.players) p.gold = 1000;
  const at = war ? PURSUIT : WAR_PROMPT;
  if (war) for (const r of s.diplomacy!.relations) r.state = 'War';
  spawnUnit(s, 0, 'Soldier', at.soldier, false);
  if (ranged) spawnUnit(s, 0, 'Ranger', fromOffset(21, 8), false);
  spawnUnit(s, 1, 'Scout', at.scout, false);
  if (!ranged) spawnUnit(s, 1, 'Soldier', s.cities[1].pos, false);
  refreshVision(s);
  return coreFixture(s);
}
