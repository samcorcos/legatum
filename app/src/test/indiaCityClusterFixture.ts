// A Indian capital with nearby and distant cities, for reviewing ordinary historical map art.
// Injected visual fixture only; not imported by the app.
import { hex } from '../hex';
import { FixtureClient } from './fixtureEngine';
import { createState, makeStandardSetup, spawnUnit } from './fixtureEngine';
import { refreshVision } from './fixtureEngine';

export function indiaCityClusterFixture() {
  const setup = makeStandardSetup('531204', 'India', 'Byzantium', 'Aiko', 'Rival');
  setup.width = 32; setup.height = 26;
  setup.players = setup.players.slice(0, 2).map((p, i) => ({ ...p, team: i, is_human: i === 0, faction: (['India', 'Byzantium'] as const)[i] }));
  const s = createState(setup);
  s.units = []; s.cities = []; s.nextUnitId = 0; s.nextCityId = 0;
  s.terrain.fill('Plains'); s.ruin.fill(0); s.wonder = undefined; s.owner.fill(-1); s.tileCity.fill(-1);
  for (const p of s.players) { p.explored.fill(1); p.gold = 500; }

  // Positions as axial hexes (not offset): capital, close neighbors, one middling and one far.
  const sites = [hex(14, 14), hex(18, 14), hex(14, 10), hex(7, 21), hex(26, 2)];
  for (const pos of sites) spawnUnit(s, 0, 'Settler', pos, false);
  // A distant Byzantium city, so the Byzantium isn't eliminated on the spot (no "last empire
  // standing" victory screen covering the map).
  spawnUnit(s, 1, 'Settler', hex(2, 20), false);
  refreshVision(s);
  const client = new FixtureClient();
  client.load(JSON.stringify(s));
  client.endTurn({
    0: s.units.filter((u) => u.owner === 0).map((u) => ({ type: 'FoundCity' as const, unit: u.id })),
    1: s.units.filter((u) => u.owner === 1).map((u) => ({ type: 'FoundCity' as const, unit: u.id })),
  });
  const f = client.debugState();
  const at = (pos: { q: number; r: number }) => f.cities.find((c) => c.pos.q === pos.q && c.pos.r === pos.r)!;
  const capital = f.cities.find((c) => c.isCapital)!;
  const near = at(sites[1]);
  const linked = at(sites[2]);
  const mid = at(sites[3]);
  const far = at(sites[4]);
  capital.population = 9; // a well-built capital: plenty of building blocks around the spire's base
  near.population = 4;
  linked.population = 4;
  mid.population = 3;
  far.population = 2;
  refreshVision(f);
  return { fixture: f, capital: capital.id, near: near.id, linked: linked.id, mid: mid.id, far: far.id };
}
