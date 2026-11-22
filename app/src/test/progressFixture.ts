// Two of your cities about to hit milestones, for the progress-bar screens: the capital grows to
// population 2 next turn (unlocking the Settler and claiming a tile with it),
// and a second city at population 3 is starving; research is part-way done. Injected visual fixture only; not imported by the app.
import { fromOffset, within } from '../hex';
import { FixtureClient } from './fixtureEngine';
import { createState, makeStandardSetup, spawnUnit } from './fixtureEngine';
import { growthThreshold, idxOf, refreshVision } from './fixtureEngine';

export function progressFixture() {
  const setup = makeStandardSetup('77311', 'India', 'Byzantium', 'Explorer', 'Rival');
  setup.width = 24; setup.height = 14;
  setup.players = setup.players.slice(0, 2).map((p, i) => ({ ...p, team: i, is_human: i === 0, faction: (['Athens', 'Byzantium'] as const)[i] }));
  const s = createState(setup);
  s.units = []; s.cities = []; s.nextUnitId = 0; s.nextCityId = 0;
  s.terrain.fill('Ocean'); s.ruin.fill(0); s.wonder = undefined; s.owner.fill(-1); s.tileCity.fill(-1);
  s.resource?.fill(null);
  for (let row = 2; row <= 11; row++) for (let col = 2; col <= 21; col++) s.terrain[row * s.width + col] = (col + row) % 5 === 0 ? 'Hills' : (col * 3 + row) % 7 === 0 ? 'Forest' : 'Grassland';
  for (const p of s.players) p.explored.fill(1);
  spawnUnit(s, 0, 'Settler', fromOffset(6, 6), false);
  spawnUnit(s, 0, 'Settler', fromOffset(12, 5), false);
  spawnUnit(s, 1, 'Settler', fromOffset(19, 9), false);
  refreshVision(s);
  const client = new FixtureClient();
  client.load(JSON.stringify(s));
  client.endTurn({ 0: s.units.filter((u) => u.owner === 0).map((u) => ({ type: 'FoundCity' as const, unit: u.id })), 1: [{ type: 'FoundCity', unit: s.units.find((u) => u.owner === 1)!.id }] });
  const f = client.debugState();
  const capital = f.cities.find((c) => c.owner === 0 && c.isCapital)!;
  const second = f.cities.find((c) => c.owner === 0 && !c.isCapital)!;
  // The capital grows next turn and claims a tile with it: Food one surplus short.
  capital.food = growthThreshold(capital) - 1;
  capital.producing = { kind: 'Building', building: 'Granary' };
  capital.prod = 22;
  // The second city starves: population 3 on desert with a little Food left.
  for (const h of within(second.pos, 1)) f.terrain[idxOf(f, h)] = 'Desert';
  second.population = 3;
  second.food = 5;
  second.producing = { kind: 'Unit', unit: 'Scout' };
  second.prod = 12;
  // Research part-way through, for the top bar's research gauge.
  f.players[0].research = 'Irrigation';
  f.players[0].research_queue = ['Irrigation'];
  f.players[0].science = 9;
  refreshVision(f);
  return { fixture: f, capital: capital.id, second: second.id };
}
