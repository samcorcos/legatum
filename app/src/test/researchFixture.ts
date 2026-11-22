// A research-panel state for one faction, at Peace with two rival empires that know more
// technologies. Injected visual fixture only; not imported by the app.
import { fromOffset } from '../hex';
import { FixtureClient } from './fixtureEngine';
import { createState, makeStandardSetup, spawnUnit } from './fixtureEngine';
import { refreshVision } from './fixtureEngine';
import type { Faction, TechId } from '../types';

export function researchFixture(faction: Faction) {
  const setup = makeStandardSetup('99124', faction, 'Mongols', 'Ada', 'Rival');
  setup.width = 24; setup.height = 12;
  // All three seats must execute the staged FoundCity orders; the fixture is
  // switched back to its desired human/AI mix after the scene exists.
  setup.players = setup.players.slice(0, 3).map((p, i) => ({ ...p, team: i, is_human: true, faction: ([faction, 'Mongols', 'India'] as const)[i], name: ['Ada', 'Epoch', 'Hive'][i] }));
  const s = createState(setup);
  s.units = []; s.cities = []; s.nextUnitId = 0; s.nextCityId = 0;
  s.terrain.fill('Ocean'); s.ruin.fill(0); s.wonder = undefined; s.owner.fill(-1); s.tileCity.fill(-1);
  for (let row = 2; row <= 9; row++) for (let col = 1; col <= 22; col++) s.terrain[row * s.width + col] = (col + row) % 5 === 0 ? 'Forest' : (col * row) % 7 === 0 ? 'Grassland' : 'Plains';
  for (const p of s.players) p.explored.fill(1);
  const sites: [number, number, number][] = [[0, 4, 5], [0, 4, 9], [1, 9, 4], [2, 16, 6]];
  for (const [owner, col, row] of sites) spawnUnit(s, owner, 'Settler', fromOffset(col, row), false);
  refreshVision(s);
  const client = new FixtureClient();
  client.load(JSON.stringify(s));
  client.endTurn(Object.fromEntries([0, 1, 2].map((p) => [p, s.units.filter((u) => u.owner === p).map((u) => ({ type: 'FoundCity' as const, unit: u.id }))])));
  const f = client.debugState();
  for (let i = 1; i < f.players.length; i++) {
    f.players[i].isHuman = false;
    f.setup.players[i].is_human = false;
  }
  for (const r of f.diplomacy!.relations) r.state = 'Peace';
  for (const c of f.cities) c.population = 3;
  const known: TechId[] = ['Irrigation', 'BronzeWorking'];
  f.players[0].techs = [...known];
  f.players[0].science = 11;
  f.players[1].techs = [...known, 'Masonry', 'Writing', 'Currency'];
  f.players[2].techs = [...known, 'Writing', 'Currency'];
  spawnUnit(f, 0, 'Scout', fromOffset(6, 7), false);
  refreshVision(f);
  return f;
}
