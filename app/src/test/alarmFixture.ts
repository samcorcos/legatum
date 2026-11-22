import { fromOffset, within } from '../hex';
import { coreFixture } from './coreFixture';
import { createState, makeStandardSetup, spawnUnit } from './fixtureEngine';
import { idxOf, refreshVision } from './fixtureEngine';
import type { Event, Relation } from '../types';

/**
 * The lighter alarm screens (a broken treaty, a lost city, an ally's war), which the real AI can't
 * be scripted to trigger on cue. The save is the world just after such a turn: the Spartans
 * (team 2) hold Brasslock, taken from us; the Indians (team 1) are our allies, now at War with the
 * Spartans; the Swiss (team 3) have just left their Alliance with us inside its protected
 * window, a betrayal. `ALARM_EVENTS` are that
 * turn's events, which the tour feeds through the real alarm logic (alarmsForTurn) one at a time.
 */
export const ALARM_TURN = 12;
export const ALARM_CITY = { lost: 1 };
const BETRAYAL = { by: 3, against: 0, turn: ALARM_TURN, until: ALARM_TURN + 20 };

function alarmState() {
  const setup = makeStandardSetup('700043', 'Mongols', 'India', 'Ada', 'India envoy');
  setup.width = 30; setup.height = 20;
  const ai = (faction: 'India' | 'Sparta' | 'Switzerland', name: string, team: number) => ({ name, faction, team, is_human: false });
  setup.players = [
    { ...setup.players[0], team: 0, is_human: true },
    ai('India', 'AI 2', 1),
    ai('Sparta', 'AI 3', 2),
    ai('Switzerland', 'AI 4', 3),
  ];
  const s = createState(setup);
  s.turn = ALARM_TURN;
  s.terrain.fill('Grassland'); s.owner.fill(-1); s.tileCity.fill(-1); s.ruin.fill(0); s.wonder = undefined;
  s.units = []; s.nextUnitId = 0;
  const sites: [number, number, number, number, string][] = [
    [0, 0, 8, 8, 'Waystation'], [2, 0, 13, 8, 'Brasslock'], [1, 1, 8, 15, 'Hymnal Reach'],
    [2, 2, 20, 8, 'Oathhold'], [3, 3, 14, 3, 'Harmonia'],
  ];
  s.cities = sites.map(([owner, founder, c, r, name], id) => ({ id, owner, founder, name, pos: fromOffset(c, r),
    population: 2, food: 0, prod: 0, producing: null, bought: null, buildings: [], hp: id === ALARM_CITY.lost ? 40 : 200,
    grownTiles: [], tilesClaimed: 0, isCapital: owner === founder, foundedTurn: 1, resistance: id === ALARM_CITY.lost ? 3 : 0 }));
  s.nextCityId = s.cities.length;
  for (const c of s.cities) for (const h of within(c.pos, 1)) {
    const i = idxOf(s, h); s.owner[i] = c.owner; s.tileCity[i] = c.id;
  }
  for (const p of s.players) p.gold = 400;
  const relation = (a: number, b: number) => s.diplomacy!.relations.find((r) => r.a === Math.min(a, b) && r.b === Math.max(a, b))!;
  Object.assign(relation(0, 1), { state: 'Alliance', effective_turn: 4, guarantee_until: 7, version: 1,
    security: { id: 1, started_turn: 4, protected_until: 14 } });
  Object.assign(relation(0, 2), { state: 'War', effective_turn: 9, version: 1, war_id: 1, declarers: [2] });
  Object.assign(relation(1, 2), { state: 'War', effective_turn: ALARM_TURN, version: 1, war_id: 2, declarers: [1] });
  s.diplomacy!.betrayals = [BETRAYAL];
  s.diplomacy!.next_treaty_id = 3; s.diplomacy!.next_war_id = 3;
  // The Sparta garrison Brasslock with more on the road; our Soldier and Scout watch from Waystation.
  spawnUnit(s, 2, 'Trooper', fromOffset(13, 8), false);
  for (const [c, r] of [[12, 9], [13, 7]]) spawnUnit(s, 2, 'Trooper', fromOffset(c, r), false);
  spawnUnit(s, 0, 'Soldier', fromOffset(11, 8), false);
  spawnUnit(s, 0, 'Scout', fromOffset(13, 5), false);
  spawnUnit(s, 1, 'Soldier', fromOffset(9, 14), false);
  refreshVision(s);
  return s;
}

export function alarmCoreSave() {
  return coreFixture(alarmState());
}

const war = (a: number, b: number, declarers: number[], war_id: number): Relation =>
  ({ a, b, state: 'War', security: null, effective_turn: ALARM_TURN, guarantee_until: 0, version: 2, war_id, declarers });

/** One turn's events per lighter alarm, as the viewer (Ada, player 0, team 0) would receive them. */
export const ALARM_EVENTS: Record<'breach' | 'cityLost' | 'allyWar', Event[]> = {
  breach: [{ type: 'TreatyBreached', treaty_id: 2, by: 3, with: 0, player: 3, cost: 120, effective_turn: ALARM_TURN },
    { type: 'AllianceBetrayed', betrayal: BETRAYAL }],
  cityLost: [{ type: 'CityCaptured', city: ALARM_CITY.lost, from: 0, to: 2, bounty: 80 }],
  // The Sparta attacked Ada's ally; Ada defends it at once.
  allyWar: [{ type: 'WarDeclared', relation: war(1, 2, [2], 2) }, { type: 'WarJoined', team: 0, ally: 1, relation: war(0, 2, [2], 2) }],
};
