// Visual reconstruction of the Tessitura turn-38 approach, with the selected unit ordered
// before its companions. The private save stays outside the repository.
import { tileIndex } from '../hex';
import { createState, makeStandardSetup, spawnUnit } from './fixtureEngine';
import { refreshVision } from './fixtureEngine';
import type { MCity } from './fixtureState';

export function pathDetourFixture() {
  const setup = makeStandardSetup('3814', 'Sparta', 'India', 'Player 1', 'AI 8');
  setup.players = setup.players.slice(0, 2).map((p, i) => ({ ...p, team: i, is_human: true }));
  const s = createState(setup);
  s.turn = 38;
  s.units = []; s.cities = []; s.nextUnitId = 0; s.nextCityId = 0;
  s.terrain.fill('Mountains'); s.ruin.fill(0); s.wonder = undefined; s.owner.fill(-1); s.tileCity.fill(-1);
  const rows = [
    ['Mountains', 'Desert', 'Desert', 'Desert', 'Desert'],
    ['Plains', 'Plains', 'Plains', 'Desert', 'Desert'],
    ['Plains', 'Plains', 'Plains', 'Desert', 'Desert'],
    ['Plains', 'Plains', 'Plains', 'Plains', 'Desert'],
    ['Plains', 'Plains', 'Mountains', 'Mountains', 'Plains'],
  ] as const;
  rows.forEach((row, ri) => row.forEach((terrain, qi) => {
    s.terrain[tileIndex({ q: qi + 19, r: ri + 11 }, s.width, s.height)] = terrain;
  }));
  const city: MCity = {
    id: 12, owner: 0, founder: 1, name: 'Tessitura', pos: { q: 21, r: 13 },
    population: 3, food: 0, prod: 0, producing: null, bought: null, buildings: [], hp: 200,
    tilesClaimed: 0, grownTiles: [], isCapital: true, foundedTurn: 20, resistance: 0,
  };
  s.cities.push(city); s.nextCityId = 13;
  s.owner[tileIndex(city.pos, s.width, s.height)] = 0;
  s.tileCity[tileIndex(city.pos, s.width, s.height)] = city.id;
  for (const p of s.players) p.explored.fill(1);
  for (const relation of s.diplomacy!.relations) relation.state = 'War';
  const attacker = spawnUnit(s, 0, 'Soldier', { q: 20, r: 14 }, false);
  const upper = spawnUnit(s, 0, 'Soldier', { q: 21, r: 13 }, false);
  const left = spawnUnit(s, 0, 'Soldier', { q: 20, r: 13 }, false);
  spawnUnit(s, 1, 'Soldier', { q: 22, r: 14 }, false);
  refreshVision(s);
  return { fixture: s, attacker: attacker.id, upper: upper.id, left: left.id,
    gap: { q: 21, r: 14 }, target: { q: 22, r: 14 } };
}
