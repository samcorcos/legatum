// A narrow island: several turns of safe exploration, one ruin, then a finite frontier.
import { RULES } from '../data';
import { fromOffset } from '../hex';
import { makeStandardSetup, createState, spawnUnit } from './fixtureEngine';
import { refreshVision } from './fixtureEngine';
import type { MState } from './fixtureState';

export function explorationFixture(): MState {
  const setup = makeStandardSetup('99123', 'India', 'Byzantium', 'Explorer', 'Rival');
  setup.width = 20; setup.height = 10;
  setup.players = setup.players.slice(0, 2).map((p, i) => ({ ...p, team: i, is_human: i === 0 }));
  const s = createState(setup);
  s.units = []; s.cities = []; s.nextUnitId = 0; s.nextCityId = 0;
  s.terrain.fill('Ocean'); s.ruin.fill(0); s.wonder = undefined; s.owner.fill(-1); s.tileCity.fill(-1);
  for (let col = 2; col <= 15; col++) s.terrain[4 * s.width + col] = 'Plains';
  s.terrain[0] = 'Plains';
  s.ruin[4 * s.width + 8] = 1;
  for (const p of s.players) { p.explored.fill(0); p.gold = 10000; }
  spawnUnit(s, 0, 'Scout', fromOffset(3, 4), false).id = 0;
  spawnUnit(s, 0, 'Settler', fromOffset(2, 4), false).id = 1;
  spawnUnit(s, 1, 'Settler', fromOffset(0, 0), false).id = 2;
  refreshVision(s);
  return s;
}

/** Translate fixture data only; zero bitsets avoid JSON's u64 precision limit. */
export function explorationCoreSave(s = explorationFixture()): string {
  return JSON.stringify({
    save_version: RULES.saveVersion,
    setup: s.setup, turn: s.turn, winner: null, finished: false,
    map: { width: s.width, height: s.height, tiles: s.terrain.map((terrain, i) => ({ terrain, ruin: !!s.ruin[i], owner: null, city: null })) },
    players: s.players.map((p) => ({
      id: p.id, name: p.name, faction: p.faction, team: p.team, is_human: p.isHuman, alive: true,
      gold: p.gold, science_stored: 0, researching: null, techs: [],
      explored: { len: s.terrain.length, words: Array(Math.ceil(s.terrain.length / 64)).fill(0) },
      tile_memory: Array(s.terrain.length).fill(null), ruins_explored: 0, settlers_trained: 0,
      science_history: [], faction_goal_done: false,
    })),
    cities: {}, units: Object.fromEntries(s.units.map((u) => [u.id, {
      id: u.id, owner: u.owner, kind: u.kind, pos: u.pos, hp: u.hp, moves_left: u.moves,
      goto: u.goto, fortify: u.fortify, acted: u.acted,
      ...(u.standing_order ? { standing_order: u.standing_order } : {}),
      ...(u.automation ? { automation: u.automation } : {}),
      ...(u.needs_orders ? { needs_orders: true } : {}),
    }])),
    next_unit_id: s.nextUnitId, next_city_id: 0,
  });
}
