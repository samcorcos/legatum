import { fromOffset, within } from '../hex';
import { coreFixture } from './coreFixture';
import { createState, makeStandardSetup, spawnUnit } from './fixtureEngine';
import { resolveFixture } from './fixtureEngine';
import { thresholdsFor } from '../promotion';
import { idxOf, refreshVision } from './fixtureEngine';
import type { Faction, TechId } from '../types';
import type { MCity, MState } from './fixtureState';

/** A city owning its ring and one grown tile per population above 1, like a real one (population is territory). */
function city(s: MState, owner: number, col: number, row: number, name: string, population: number, capital = false): MCity {
  const c: MCity = { id: s.nextCityId++, owner, founder: owner, name, pos: fromOffset(col, row), population,
    food: 4, prod: 0, producing: null, bought: null, buildings: [], hp: 200, grownTiles: [], tilesClaimed: 0,
    isCapital: capital, foundedTurn: 1, resistance: 0 };
  s.cities.push(c);
  for (const h of within(c.pos, 1)) { const i = idxOf(s, h); s.owner[i] = owner; s.tileCity[i] = c.id; }
  for (const h of within(c.pos, 7)) {
    if (c.grownTiles.length >= population - 1) break;
    const i = idxOf(s, h);
    if (i < 0 || s.owner[i] >= 0) continue;
    s.owner[i] = owner; s.tileCity[i] = c.id; c.grownTiles.push(h); c.tilesClaimed++;
  }
  return c;
}

function base(seed: string, human: Faction, ai: { faction: Faction; name: string }[]) {
  const setup = makeStandardSetup(seed, human, ai[0].faction, 'Ada', 'Bo');
  setup.width = 30; setup.height = 22;
  setup.players = [
    { ...setup.players[0], team: 0, is_human: true },
    ...ai.map((a, i) => ({ name: a.name, faction: a.faction, team: i + 1, is_human: false })),
  ];
  setup.barbarians = false;
  const s = createState(setup);
  s.terrain.fill('Grassland'); s.owner.fill(-1); s.tileCity.fill(-1); s.ruin.fill(0); s.wonder = undefined;
  s.units = []; s.nextUnitId = 0; s.cities = []; s.nextCityId = 0;
  for (const relation of s.diplomacy?.relations ?? []) relation.state = 'Peace';
  for (const p of s.players) p.gold = 300;
  return s;
}

/** Aztecs with 54 population to spare and four cities of different sizes, ready for the New Fire. */
export function swarmState(): MState {
  const s = base('710101', 'Aztecs', [{ faction: 'Celts', name: 'AI 2' }]);
  city(s, 0, 8, 7, 'Hive', 26, true);
  city(s, 0, 16, 10, 'Spore', 22);
  city(s, 0, 9, 15, 'Cell', 9);
  city(s, 0, 21, 16, 'Seed', 1);
  city(s, 1, 26, 4, 'Greenwatch', 4, true);
  spawnUnit(s, 0, 'Scout', fromOffset(12, 9), false);
  s.turn = 120;
  refreshVision(s);
  return s;
}

/**
 * A legacy Sakoku scenario with four cities and two rivals, at Peace. `turns` is the vow (null: the country is open).
 */
export function sakokuState(turns: number | null = null, enemyInLand = false): MState {
  const s = base('710106', 'India', [{ faction: 'Celts', name: 'AI 2' }, { faction: 'Mongols', name: 'AI 3' }]);
  const hive = city(s, 0, 8, 7, 'Hive', 12, true);
  city(s, 0, 14, 9, 'Spore', 8);
  city(s, 0, 9, 14, 'Cell', 6);
  city(s, 1, 26, 4, 'Greenwatch', 4, true);
  city(s, 2, 25, 17, 'Waystation', 4, true);
  spawnUnit(s, 0, 'Spearman', hive.pos, false);
  spawnUnit(s, 0, 'Scout', fromOffset(12, 9), false);
  s.players[0].goalRule = 'Sakoku';
  s.players[0].sakokuTurns = turns;
  if (turns !== null && turns >= 30) { s.players[0].factionGoalDone = true; s.players[0].factionGoalReachedBefore = true; }
  if (enemyInLand) {
    for (const r of s.diplomacy?.relations ?? []) if (r.a === 0 && r.b === 1) r.state = 'War';
    const at = hive.grownTiles[0] ?? fromOffset(9, 8);
    spawnUnit(s, 1, 'Spearman', at, false);
  }
  s.turn = turns === null ? 60 : 90;
  refreshVision(s);
  return s;
}

/**
 * A Celtic human at war with a closed India (an AI) whose vow is held: a Celtic Soldier already stands inside Indian land,
 * so the held VP is gone (`parked`), or a rival Soldier waits outside (`!parked`) and the VP is held.
 */
function sakokuRivalState(parked = true): MState {
  const s = base('710107', 'Celts', [{ faction: 'India', name: 'AI 2' }]);
  const home = city(s, 0, 6, 6, 'Greenhold', 6, true);
  const hive = city(s, 1, 17, 10, 'Hive', 10, true);
  city(s, 1, 22, 15, 'Spore', 6);
  spawnUnit(s, 0, 'Spearman', home.pos, false);
  spawnUnit(s, 0, 'Scout', fromOffset(9, 8), false);
  for (const r of s.diplomacy?.relations ?? []) if (r.a === 0 && r.b === 1) r.state = 'War';
  s.players[1].sakokuTurns = 40;
  s.players[1].goalRule = 'Sakoku';
  const inside = hive.grownTiles[0] ?? fromOffset(18, 10);
  if (parked) spawnUnit(s, 0, 'Spearman', inside, false);
  s.players[1].factionGoalDone = true;
  s.players[1].factionGoalReachedBefore = true;
  s.turn = 100;
  refreshVision(s);
  return s;
}

/** The Athens with two rivals: Share the Learning is available. */
export function broadcastState(): MState {
  const s = base('710102', 'Athens', [{ faction: 'Celts', name: 'AI 2' }, { faction: 'Mongols', name: 'AI 3' }]);
  city(s, 0, 8, 7, 'Listening Deep', 6, true);
  city(s, 0, 15, 12, 'Quiet Relay', 4);
  city(s, 1, 25, 5, 'Greenwatch', 4, true);
  city(s, 2, 24, 17, 'Waystation', 4, true);
  spawnUnit(s, 0, 'Scout', fromOffset(12, 9), false);
  s.turn = 90;
  refreshVision(s);
  return s;
}

const KNOWN: TechId[] = ['BronzeWorking', 'Irrigation', 'Currency'];

/** A Celts human who has just received the Athens' broadcast: three free technologies to choose. */
export function freeTechsState(): MState {
  const s = base('710103', 'Celts', [{ faction: 'Athens', name: 'AI 2' }, { faction: 'Mongols', name: 'AI 3' }]);
  city(s, 0, 8, 7, 'Greenhold', 5, true);
  city(s, 0, 15, 12, 'Thornwatch', 3);
  city(s, 1, 25, 5, 'Listening Deep', 6, true);
  city(s, 2, 24, 17, 'Waystation', 4, true);
  spawnUnit(s, 0, 'Scout', fromOffset(12, 9), false);
  s.players[0].techs = [...KNOWN];
  s.players[0].freeTechsPending = 3;
  s.players[0].freeTechsFrom = 1;
  s.players[1].factionGoalDone = true;
  s.turn = 91;
  refreshVision(s);
  return s;
}

/**
 * The Sparta with a War Shrine in Ironhold, a plain Stonewatch, and veterans: two level-3 and a
 * level-6 in the shrine city, a level-5 waiting in Stonewatch, and a green soldier. Honour the Fallen
 * is 0 of 3 (`retired` sets how many are done already).
 */
export function retireState(retired = 0): MState {
  const s = base('710104', 'Sparta', [{ faction: 'Celts', name: 'AI 2' }]);
  const shrine = city(s, 0, 8, 7, 'Ironhold', 6, true);
  shrine.buildings = ['Agoge'];
  const plain = city(s, 0, 17, 11, 'Stonewatch', 4);
  city(s, 1, 25, 5, 'Greenwatch', 4, true);
  const xp = (level: number) => thresholdsFor('Sparta')[level - 1];
  const stand = (kind: 'Spearman' | 'Pikeman', at: MCity, level: number, hp = 100) => { const u = spawnUnit(s, 0, kind, at.pos, false); u.xp = level ? xp(level) : 0; u.hp = hp; return u; };
  stand('Spearman', shrine, 4); stand('Spearman', shrine, 6, 70); stand('Pikeman', shrine, 4, 85);
  stand('Spearman', plain, 5); stand('Spearman', plain, 1);
  s.players[0].veteransRetired = retired;
  if (retired >= 3) s.players[0].factionGoalDone = true;
  s.turn = 130;
  refreshVision(s);
  return s;
}

/** The Celts with three cities whose territory reaches three hexes out: Call the warbands is available, no camp stands. */
export function wildState(): MState {
  const s = base('710105', 'Celts', [{ faction: 'Mongols', name: 'AI 2' }]);
  const cities = [city(s, 0, 8, 7, 'Greenhold', 8, true), city(s, 0, 17, 9, 'Thornwatch', 8), city(s, 0, 11, 16, 'Mossfen', 8)];
  for (const c of cities) for (const h of within(c.pos, 3)) { const i = idxOf(s, h); if (s.owner[i] < 0 || s.tileCity[i] === c.id) { s.owner[i] = 0; s.tileCity[i] = c.id; } }
  city(s, 1, 26, 4, 'Waystation', 4, true);
  spawnUnit(s, 0, 'Spearman', fromOffset(8, 8), false);
  // Every milestone is already claimed, so a turn's report holds only what the rite did.
s.turn = 100;
  refreshVision(s);
  return s;
}

/** The Celts after the call: a camp in every city (guards and scouts too) and the live goal held. */
export function wildCalledState(): MState {
  const s = wildState();
  s.barbarians = { camps: [], next_camp_id: 0, last_camp_turn: 0, known: {} };
  return resolveFixture(s, { 0: [{ type: 'Action', action: { action: 'CallTheWild' } }] });
}

/** The Celts's camp in Greenhold's land has just lost its guard to a rival soldier standing on it: the next turn clears it, the rival is paid and the Celts lose the VP. */
function wildClearedState(): MState {
  const s = wildCalledState();
  const camp = s.barbarians!.camps[0];
  s.units = s.units.filter((u) => u.id !== camp.guard);
  camp.guard = null;
  // At Peace a rival must leave closed Celts land; at war it may stand and clear the camp.
  for (const relation of s.diplomacy?.relations ?? []) relation.state = 'War';
  spawnUnit(s, 1, 'Spearman', camp.pos, false);
  refreshVision(s);
  return s;
}

export const retireCoreSave = (retired = 0) => coreFixture(retireState(retired));
export const wildCoreSave = () => coreFixture(wildState());
export const wildClearedCoreSave = () => coreFixture(wildClearedState());
export const sakokuRivalCoreSave = (parked = true) => coreFixture(sakokuRivalState(parked));
export const sakokuCoreSave = (turns: number | null = null, enemyInLand = false) => coreFixture(sakokuState(turns, enemyInLand));
export const broadcastCoreSave = () => coreFixture(broadcastState());
export const freeTechsCoreSave = () => coreFixture(freeTechsState());
