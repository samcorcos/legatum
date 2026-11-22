// Walled and unwalled cities at War, for the walls-only city strike UI (map walls, city panel,
// tile card and attack forecast). Injected visual fixture only; not imported by the app.
import { fromOffset } from '../hex';
import { FixtureClient } from './fixtureEngine';
import { createState, makeStandardSetup, spawnUnit } from './fixtureEngine';
import { refreshVision } from './fixtureEngine';

export function wallsFixture() {
  const setup = makeStandardSetup('99123', 'India', 'Byzantium', 'Explorer', 'Rival');
  setup.width = 24; setup.height = 12;
  setup.players = setup.players.slice(0, 3).map((p, i) => ({ ...p, team: i, is_human: true, faction: (['India', 'Byzantium', 'Switzerland'] as const)[i] }));
  const s = createState(setup);
  s.units = []; s.cities = []; s.nextUnitId = 0; s.nextCityId = 0;
  s.terrain.fill('Ocean'); s.ruin.fill(0); s.wonder = undefined; s.owner.fill(-1); s.tileCity.fill(-1);
  for (let row = 2; row <= 9; row++) for (let col = 1; col <= 22; col++) s.terrain[row * s.width + col] = 'Plains';
  for (const p of s.players) { p.explored.fill(1); p.gold = 500; }
  const sites: [number, number, number][] = [[0, 3, 5], [1, 9, 4], [1, 13, 7], [2, 17, 4]];
  for (const [owner, col, row] of sites) spawnUnit(s, owner, 'Settler', fromOffset(col, row), false);
  refreshVision(s);
  const client = new FixtureClient();
  client.load(JSON.stringify(s));
  client.endTurn(Object.fromEntries([0, 1, 2].map((p) => [p, s.units.filter((u) => u.owner === p).map((u) => ({ type: 'FoundCity' as const, unit: u.id }))])));
  const f = client.debugState();
  const at = (col: number, row: number) => f.cities.find((c) => c.pos.q === fromOffset(col, row).q && c.pos.r === fromOffset(col, row).r)!;
  const own = at(3, 5), walled = at(9, 4), open = at(13, 7), bastion = at(17, 4);
  own.buildings = ['Walls', 'Castle'];
  walled.name = 'Berea';
  walled.buildings = ['Walls'];
  bastion.buildings = ['Walls'];
  for (const c of [own, walled, open, bastion]) c.population = 3;
  for (const r of f.diplomacy!.relations) r.state = 'War';
  // Your army: one Soldier beside each enemy city, so all four are in sight.
  const nearWalled = spawnUnit(f, 0, 'Spearman', fromOffset(8, 5), false);
  const nearOpen = spawnUnit(f, 0, 'Spearman', fromOffset(12, 7), false);
  spawnUnit(f, 0, 'Scout', fromOffset(16, 5), false);
  refreshVision(f);
  return { fixture: f, own: own.id, walled: walled.id, open: open.id, bastion: bastion.id, nearWalled: nearWalled.id, nearOpen: nearOpen.id };
}
