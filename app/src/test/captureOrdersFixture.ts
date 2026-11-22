// Playtest 8.4, turn 30 (user): five Soldiers ordered into a fallen city, one garrison soldier
// left at 1 HP. One Soldier takes the city; the other four must be ready for orders afterwards.
import { fromOffset, neighbors } from '../hex';
import { createState, makeStandardSetup, spawnUnit } from './fixtureEngine';
import { idxOf, refreshVision } from './fixtureEngine';
import type { MCity, MState } from './fixtureState';

export const CAPTURE_AT = fromOffset(5, 5);

function city(s: MState, id: number, owner: number, name: string, pos: { q: number; r: number }, hp: number): MCity {
  const c: MCity = { id, owner, founder: owner, name, pos, population: 3, food: 0, prod: 0, producing: null, bought: null,
    buildings: [], hp, grownTiles: [], tilesClaimed: 0, isCapital: true, foundedTurn: 1, resistance: 0 };
  s.cities.push(c); s.nextCityId = Math.max(s.nextCityId, id + 1);
  for (const h of [pos, ...neighbors(pos)]) s.owner[idxOf(s, h)] = owner;
  s.tileCity[idxOf(s, pos)] = id;
  return c;
}

export function captureOrdersFixture() {
  const setup = makeStandardSetup('7', 'Sparta', 'Switzerland', 'Player 1', 'AI 2');
  setup.width = 20; setup.height = 12;
  setup.players = setup.players.slice(0, 2).map((p, i) => ({ ...p, team: i, is_human: i === 0 }));
  const s = createState(setup);
  for (const relation of s.diplomacy!.relations) relation.state = 'War';
  s.units = []; s.cities = []; s.nextUnitId = 0; s.nextCityId = 0;
  s.terrain.fill('Grassland'); s.ruin.fill(0); s.wonder = undefined; s.owner.fill(-1); s.tileCity.fill(-1);
  s.knownOwner = {}; s.knownCities = {}; s.knownRuins = {};
  for (const p of s.players) { p.gold = 10000; p.explored.fill(0); }
  city(s, 1, 1, 'Vestige', CAPTURE_AT, 0);
  city(s, 2, 1, 'Rearguard', fromOffset(17, 9), 200);
  const home = city(s, 3, 0, 'Home', fromOffset(1, 10), 200);
  home.producing = { kind: 'Unit', unit: 'Soldier' };
  spawnUnit(s, 1, 'Soldier', CAPTURE_AT, false).hp = 1;
  const attackers = neighbors(CAPTURE_AT).slice(0, 5).map((h) => {
    const u = spawnUnit(s, 0, 'Soldier', h, false); u.goto = { ...CAPTURE_AT }; return u.id;
  });
  refreshVision(s);
  return { state: s, attackers };
}
