// Playtest-6.2 "The Last Bridge": three of your Spearmen in a row south of an enemy city. The
// two beside it are ordered to attack; the rightmost must reach the free tile beside the city,
// not route through its neighbour. A second group waits in a one-tile pass behind a parked
// Spearman, to show the route warning. Injected visual fixture only; not imported by the app.
import type { Hex } from '../types';
import { fromOffset, neighbors } from '../hex';
import { FixtureClient } from './fixtureEngine';
import { createState, makeStandardSetup, spawnUnit } from './fixtureEngine';
import { refreshVision } from './fixtureEngine';

export function lastBridgeFixture() {
  const setup = makeStandardSetup('62062', 'Mongols', 'Sparta', 'Explorer', 'Rival');
  setup.width = 24; setup.height = 14;
  setup.players = setup.players.slice(0, 2).map((p, i) => ({ ...p, team: i, is_human: true, faction: (['Switzerland', 'Sparta'] as const)[i] }));
  const s = createState(setup);
  s.units = []; s.cities = []; s.nextUnitId = 0; s.nextCityId = 0;
  s.terrain.fill('Ocean'); s.ruin.fill(0); s.wonder = undefined; s.owner.fill(-1); s.tileCity.fill(-1);
  for (let row = 2; row <= 11; row++) for (let col = 1; col <= 22; col++) s.terrain[row * s.width + col] = 'Plains';
    // A dead-end pass: mountains above, below and at its east end, so it opens only to the west.
  for (let col = 14; col <= 22; col++) for (const row of [9, 11]) s.terrain[row * s.width + col] = 'Mountains';
  s.terrain[10 * s.width + 22] = 'Mountains';
  for (const p of s.players) { p.explored.fill(1); p.gold = 500; }
  spawnUnit(s, 0, 'Settler', fromOffset(4, 9), false);
  spawnUnit(s, 1, 'Settler', fromOffset(11, 4), false);
  refreshVision(s);
  const client = new FixtureClient();
  client.load(JSON.stringify(s));
  client.endTurn(Object.fromEntries([0, 1].map((p) => [p, s.units.filter((u) => u.owner === p).map((u) => ({ type: 'FoundCity' as const, unit: u.id }))])));
  const f = client.debugState();
  for (const r of f.diplomacy!.relations) r.state = 'War';
  const city = f.cities.find((c) => c.owner === 1)!;
  city.name = 'Last Bridge'; city.population = 3;
  const c = city.pos;
  // South-west and south-east of the city, and the tile two steps south-east.
  const [se, sw] = [neighbors(c).find((h) => h.r === c.r + 1 && h.q === c.q)!, neighbors(c).find((h) => h.r === c.r + 1 && h.q === c.q - 1)!];
  const e = neighbors(c).find((h) => h.r === c.r && h.q === c.q + 1)!;
  const right: Hex = { q: se.q + 1, r: se.r };
  const units = [sw, se, right].map((h) => spawnUnit(f, 0, 'Spearman', h, false));
  // The pass: a parked Spearman at its mouth and another behind it.
  const parked = spawnUnit(f, 0, 'Spearman', fromOffset(15, 10), false);
  const behind = spawnUnit(f, 0, 'Spearman', fromOffset(14, 10), false);
  refreshVision(f);
  return {
    fixture: f, city: city.id, cityPos: c, free: e,
    attackers: [units[0].id, units[1].id], neighbour: se, rightmost: units[2].id,
    parked: parked.id, behind: behind.id, passGoal: fromOffset(16, 10),
  };
}
