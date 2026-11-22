// Shared fixture for the VP hold rule (core `scoring::update_vp_holds`): a team wins by VP only after holding its
// target for `data::VP_HOLD_TURNS` consecutive turn ends. At the fixture's four-per-seat target, teams of three
// get 11 VP from faction goals and scored Age cards; a scored fourth-Age card supplies the last point.
import { fromOffset, within } from '../hex';
import { idxOf } from './fixtureEngine';
import { createState, makeStandardSetup, spawnUnit } from './fixtureEngine';
import type { MCity, MState } from './fixtureState';

export interface HoldSpec {
  /** Team of each player, in seat order. */
  teams: number[];
  /** Hold counts to start from (core `vp_holds`). */
  holds?: Record<number, number>;
  /** Population of a city founded for the player (omitted: no city). */
  cities?: Record<number, number>;
  /** Tiles around the city the player owns (default 1). */
  reach?: Record<number, number>;
  /** Players who start without a Settler: with no city either they are eliminated at the first turn end. */
  noSettler?: number[];
  /** Players without the usual scored Age card. */
  oneShort?: number[];
  /** Players with another scored Age card (the last point for a team of three). */
  bonusEra?: number[];
}

export function holdFixture(spec: HoldSpec): MState {
  const setup = makeStandardSetup('700001', 'India', 'India', 'P0', 'P1');
  setup.rule_experiments = { ...setup.rule_experiments, vp_target: 4 };
  setup.players = spec.teams.map((team, i) => ({ name: `P${i}`, faction: 'India', team, is_human: true }));
  const s = createState(setup);
  s.units = [];
  s.nextUnitId = 0;
  s.terrain.fill('Grassland');
  s.ruin.fill(0);
  s.owner.fill(-1);
  s.tileCity.fill(-1);
  s.cities = [];
  if (spec.holds) s.vpHolds = { ...spec.holds };
  for (const p of s.players) {
    p.techs = [];
    p.explored.fill(0);
    p.eras = { 2: { entered: 1, scored: { turn: 1, slot: 0 } } };
    if (p.id % 3 < 2 && !spec.oneShort?.includes(p.id)) p.eras[3] = { entered: 1, scored: { turn: 1, slot: 0 } };
    if (spec.bonusEra?.includes(p.id)) p.eras[4] = { entered: 1, scored: { turn: 1, slot: 0 } };
    // A permanent goal that is done: it scores from the flag whatever the faction.
    p.goalRule = 'Swarm';
    p.factionGoalDone = true;
    const col = 3 + 6 * (p.id % 3), row = 3 + 6 * Math.floor(p.id / 3), at = fromOffset(col, row);
    if (!spec.noSettler?.includes(p.id)) spawnUnit(s, p.id, 'Settler', fromOffset(col + 2, row), false);
    if (spec.noSettler?.includes(p.id)) spawnUnit(s, p.id, 'Spearman', fromOffset(col, row + 2), false);
    const population = spec.cities?.[p.id];
    if (population === undefined) continue;
    const city: MCity = {
      id: s.cities.length + 1, owner: p.id, founder: p.id, name: `C${p.id}`, pos: at,
      population, food: 0, prod: 0, producing: null, bought: null, buildings: [], hp: 200, tilesClaimed: 0,
      grownTiles: [], isCapital: true, foundedTurn: 1, resistance: 0,
    };
    s.cities.push(city);
    for (const h of within(at, spec.reach?.[p.id] ?? 1)) {
      s.owner[idxOf(s, h)] = p.id;
      s.tileCity[idxOf(s, h)] = city.id;
    }
  }
  s.nextCityId = s.cities.length;
  return s;
}
