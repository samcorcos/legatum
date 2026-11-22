// Rome's Colonia Victrix and Egypt's Royal Tomb for the screen tour and tests: each state a player meets.
// Injected visual fixture only; not imported by the app.
//  romeBefore:  you are Rome; your best veteran is level 2
//  romeReady:   you are Rome; a level 3 Soldier stands on a site within 2 tiles of Sparta's land
//  romeHeld:    you are Rome; your Colonia Victrix stands beside Sparta's border
//  romeLost:    you are Rome; Sparta captured your Colonia Victrix
//  romeRival:   you are Sparta; Rome's Colonia Victrix stands beside your border, in sight
//  egyptIdle:   you are Egypt; no work on the Royal Tomb yet
//  egyptWork:   you are Egypt; 6 of 15 turns of work on the Royal Tomb, the work on
//  egyptRival:  you are Sparta; Egypt is working on its Royal Tomb
//  tombRival:   you are Sparta; Egypt's Royal Tomb stands in its capital, in sight
//  egyptPaused: you are Egypt; the work paused at 6 of 15 turns
//  egyptRivalPaused: you are Sparta; Egypt paused its Royal Tomb at 6 of 15 turns
//  egyptHeld:   you are Egypt; your Royal Tomb stands in your capital
//  tombLost:    you are Egypt; Sparta captured the city where your Royal Tomb stands
//  coloniaThird: you are Egypt, a third empire; Sparta holds Rome's captured Colonia Victrix, in sight
//  romeNoSite:  you are Rome; a level 3 veteran, but you have explored no land near a rival
//  romeOffRing: you are Rome; a level 3 veteran stands on your own land, off the gilt rings
import { fromOffset, within } from '../hex';
import { FixtureClient } from './fixtureEngine';
import { createState, makeStandardSetup, spawnUnit } from './fixtureEngine';
import { idxOf, refreshVision } from './fixtureEngine';
import type { Faction } from '../types';

export type GoalCityMode = 'romeBefore' | 'romeReady' | 'romeHeld' | 'romeLost' | 'romeRival' | 'egyptIdle' | 'egyptWork' | 'egyptRival' | 'tombRival'
  | 'egyptPaused' | 'egyptRivalPaused' | 'egyptHeld' | 'tombLost' | 'coloniaThird' | 'romeNoSite' | 'romeOffRing';

/** Level 2 and level 3 need these cumulative XP (`data::PROMOTION_XP`). */
const XP_LEVEL_2 = 46, XP_LEVEL_3 = 90;

export function goalCitiesFixture(mode: GoalCityMode) {
  const third = mode === 'coloniaThird';
  const rome = mode.startsWith('rome') || third, rival = mode === 'romeRival' || mode === 'egyptRival' || mode === 'egyptRivalPaused' || mode === 'tombRival';
  const goal: Faction = rome ? 'Rome' : 'Egypt';
  // Seat 0 is the viewer: the goal's empire, Sparta (a rival), or Egypt (a third empire).
  const factions: Faction[] = third ? ['Egypt', 'Rome', 'Sparta'] : rival ? ['Sparta', goal, rome ? 'Egypt' : 'Rome'] : [goal, 'Sparta', rome ? 'Egypt' : 'Rome'];
  const setup = makeStandardSetup('99126', factions[0], factions[1], 'Ada', 'Rival');
  setup.width = 30; setup.height = 14;
  setup.players = setup.players.slice(0, 3).map((p, i) => ({ ...p, team: i, is_human: i === 0, faction: factions[i], name: ['Ada', 'Marrow', 'Hive'][i] }));
  const s = createState(setup);
  s.units = []; s.cities = []; s.nextUnitId = 0; s.nextCityId = 0;
  s.terrain.fill('Ocean'); s.ruin.fill(0); s.wonder = undefined; s.owner.fill(-1); s.tileCity.fill(-1);
  for (let row = 1; row <= 12; row++) for (let col = 1; col <= 28; col++) s.terrain[row * s.width + col] = (col * 3 + row) % 7 === 0 ? 'Forest' : (col + row * 2) % 11 === 0 ? 'Hills' : (col * row) % 9 === 0 ? 'Grassland' : 'Plains';
  for (const p of s.players) p.explored.fill(1);
  // The goal's empire (seat g) owns A, B, C in the west; Sparta (seat o) owns D, E in the east.
  const g = third || rival ? 1 : 0, o = third ? 2 : rival ? 0 : 1, t = third ? 0 : 2;
  const colonia = mode === 'romeHeld' || mode === 'romeLost' || mode === 'romeRival' || third;
  const sites: [number, number, number][] = [[g, 5, 5], [g, 12, 7], [g, 6, 10], [o, 23, 5], [o, 23, 10], [t, 28, 3]];
  if (colonia) sites.push([g, 19, 6]);
  for (const [owner, col, row] of sites) spawnUnit(s, owner, 'Settler', fromOffset(col, row), false);
  refreshVision(s);
  const client = new FixtureClient();
  client.load(JSON.stringify(s));
  client.endTurn(Object.fromEntries([0, 1, 2].map((p) => [p, s.units.filter((u) => u.owner === p).map((u) => ({ type: 'FoundCity' as const, unit: u.id }))])));
  const f = client.debugState();
  f.turn = 40;
  for (const r of f.diplomacy!.relations) r.state = 'Peace';
  const at = (col: number, row: number) => f.cities.find((c) => c.pos.q === fromOffset(col, row).q && c.pos.r === fromOffset(col, row).r)!;
  const A = at(5, 5), B = at(12, 7), C = at(6, 10), D = at(23, 5), E = at(23, 10);
  const V = colonia ? at(19, 6) : undefined;
  for (const c of f.cities) { c.population = 4; c.buildings = []; c.producing = null; c.prod = 0; c.productionQueue = []; c.productionBanked = []; c.isCapital = false; }
  A.isCapital = true; A.population = 7; D.isCapital = true; D.population = 6;
  // Only the capital is called Rome: founding order can give that name to another Roman city.
  const romeNamed = f.cities.find((c) => c.name === 'Rome' && !c.isCapital);
  const romeCapital = romeNamed && f.cities.find((c) => c.owner === romeNamed.owner && c.isCapital);
  if (romeNamed && romeCapital) [romeNamed.name, romeCapital.name] = [romeCapital.name, romeNamed.name];
  B.population = 5; B.producing = { kind: 'Building', building: 'Granary' } as never; B.prod = 12;
  C.producing = { kind: 'Unit', unit: 'Spearman' } as never; C.prod = 8;
  // Borders reach two hexes around each city (three for capitals), so the Colonia's site has rival land in reach.
  for (const c of [A, B, C, D, E, ...(V ? [V] : [])]) {
    for (const h of within(c.pos, c.isCapital ? 3 : 2)) { const i = idxOf(f, h); if (i >= 0 && f.terrain[i] !== 'Ocean' && f.owner[i] < 0) { f.owner[i] = c.owner; f.tileCity[i] = c.id; } }
  }
  const P = f.players[g];
  if (colonia) {
    P.coloniaCity = V!.id;
    const holder = mode === 'romeLost' || third ? o : g;
    P.goalHolder = holder; P.factionGoalDone = holder === g; P.factionGoalReachedBefore = true;
    V!.population = 3;
    if (holder !== g) { V!.owner = holder; for (let i = 0; i < f.owner.length; i++) if (f.tileCity[i] === V!.id) f.owner[i] = holder; }
  }
  if (mode === 'egyptWork' || mode === 'egyptRival') { P.tombWork = true; P.tombTurns = 6; }
  if (mode === 'egyptPaused' || mode === 'egyptRivalPaused') { P.tombWork = false; P.tombTurns = 6; }
  if (mode === 'tombRival' || mode === 'egyptHeld') { P.tombCity = A.id; P.goalHolder = g; P.factionGoalDone = true; P.factionGoalReachedBefore = true; }
  if (mode === 'tombLost') {
    P.tombCity = A.id; P.goalHolder = o; P.factionGoalDone = false; P.factionGoalReachedBefore = true;
    A.owner = o; A.isCapital = false; B.isCapital = true;
    for (let i = 0; i < f.owner.length; i++) if (f.tileCity[i] === A.id) f.owner[i] = o;
  }
  // Rome's veteran: level 2 on its home land, or level 3 on the Colonia's site (or off it).
  let veteran: number | undefined;
  if (mode === 'romeBefore' || mode === 'romeReady' || mode === 'romeNoSite' || mode === 'romeOffRing') {
    const u = spawnUnit(f, 0, 'Spearman', mode === 'romeOffRing' ? fromOffset(15, 4) : fromOffset(mode === 'romeReady' ? 19 : mode === 'romeNoSite' ? 9 : 13, 6), false);
    u.xp = mode === 'romeBefore' ? XP_LEVEL_2 : XP_LEVEL_3;
    veteran = u.id;
  }
  // romeNoSite: Rome knows only its own western land, so no site it knows lies near a rival.
  if (mode === 'romeNoSite') {
    const team = f.players[0].team;
    for (let i = 0; i < f.players[0].explored.length; i++) if (i % f.width > 14) {
      f.players[0].explored[i] = 0;
      if (f.knownTerrain?.[team]) f.knownTerrain[team][i] = 0;
      if (f.knownOwner?.[team]) f.knownOwner[team][i] = -1;
    }
    if (f.knownCities?.[team]) for (const [k, c] of Object.entries(f.knownCities[team])) if (c.owner !== 0) delete f.knownCities[team][k as never];
  }
  // A Scout of the viewer's beside the goal city, so a rival sees it.
  const scoutAt = mode === 'romeRival' || third ? fromOffset(20, 8) : mode === 'tombRival' || mode === 'egyptRival' || mode === 'egyptRivalPaused' || mode === 'tombLost' ? fromOffset(8, 5) : fromOffset(4, 6);
  const scout = spawnUnit(f, 0, 'Scout', scoutAt, false);
  // Sparta at war with Rome, a Soldier beside the Colonia: the attack forecast says what a capture scores.
  let attacker: number | undefined;
  if (mode === 'romeRival') {
    for (const r of f.diplomacy!.relations) if (r.a + r.b === 1) r.state = 'War';
    attacker = spawnUnit(f, 0, 'Spearman', fromOffset(20, 6), false).id;
  }
  f.players[0].gold = 300;
  refreshVision(f);
  return { fixture: f, a: A.id, b: B.id, c: C.id, d: D.id, e: E.id, colonia: V?.id, coloniaPos: V?.pos ?? fromOffset(19, 6), capitalPos: A.pos, veteran, attacker, scout: scout.id, goalSeat: g, otherSeat: o };
}
