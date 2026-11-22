import { fromOffset, within } from '../hex';
import { coreFixture } from './coreFixture';
import { createState, makeStandardSetup, meetAll, spawnUnit } from './fixtureEngine';
import { idxOf, refreshVision } from './fixtureEngine';

/** The Athens (team 4) are eliminated; four other rivals live on. */
export const ELIMINATED = { team: 4, player: 4, faction: 'Athens' } as const;
export const ELIMINATED_TURN = 40;

/**
 * A six-empire race in which one rival has been knocked out: no cities, no units, `alive: false`
 * (core's definition, `check_elimination`), but still a standing, a War relation and two VP from a
 * milestone it earned before it fell. The Switzerland broke an alliance with the dead empire,
 * a betrayal record that must not show. For the active-list screens and tests.
 */
export function eliminatedState() {
  const setup = makeStandardSetup('700045', 'India', 'India', 'Ada', 'Ada');
  setup.width = 30; setup.height = 20;
  const ai = (faction: 'Sparta' | 'Switzerland' | 'Mongols' | 'Athens' | 'Celts', name: string, team: number) => ({ name, faction, team, is_human: false });
  setup.players = [
    { ...setup.players[0], team: 0, is_human: true },
    ai('Sparta', 'AI 2', 1),
    ai('Switzerland', 'AI 3', 2),
    ai('Mongols', 'AI 4', 3),
    ai(ELIMINATED.faction, 'AI 5', ELIMINATED.team),
    ai('Celts', 'AI 6', 5),
  ];
  const s = createState(setup);
  s.turn = ELIMINATED_TURN;
  s.terrain.fill('Grassland'); s.owner.fill(-1); s.tileCity.fill(-1); s.ruin.fill(0); s.wonder = undefined;
  s.units = []; s.nextUnitId = 0;
  const sites: [number, number, number, string][] = [
    [0, 5, 5, 'Waystation'], [1, 20, 5, 'Oathhold'], [2, 5, 14, 'Harmonia'], [3, 14, 10, 'Glasswell'], [5, 22, 15, 'Thorn'],
  ];
  s.cities = sites.map(([owner, c, r, name], id) => ({ id, owner, founder: owner, name, pos: fromOffset(c, r),
    population: 2, food: 0, prod: 0, producing: null, bought: null, buildings: [], hp: 200,
    grownTiles: [], tilesClaimed: 0, isCapital: true, foundedTurn: 1, resistance: 0 }));
  s.nextCityId = s.cities.length;
  for (const c of s.cities) for (const h of within(c.pos, 1)) {
    const i = idxOf(s, h); s.owner[i] = c.owner; s.tileCity[i] = c.id;
  }
  for (const p of s.players) { p.gold = 300; p.explored.fill(1); }
  const dead = s.players[ELIMINATED.player];
  dead.alive = false;
for (const c of s.cities) spawnUnit(s, c.owner, 'Spearman', c.pos, false);
  s.diplomacy!.betrayals = [{ by: 2, against: ELIMINATED.team, turn: 30, until: 50 }];
  refreshVision(s);
  meetAll(s);
  return s;
}

export function eliminatedCoreSave() {
  return coreFixture(eliminatedState());
}
