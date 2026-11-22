// Original capitals in every state the map must tell apart, staged by the rules (city founder, owner and the
// original-capital flag; the holder of a rival's capital scores it while it holds it). Injected visual fixture only.
// You are India (0) with a teammate, the Aztecs (4). The rivals are Byzantium (1), Switzerland (2), Germany (3), the Mongols (5).
//   A  yours, still held by you and garrisoned (original capital, filled star in your ink)
//   B  Byzantium's original capital, held by its founder and garrisoned (filled star in its ink)
//   C  Switzerland's original capital, captured by Byzantium (the star stays in Switzerland's ink)
//   D  Switzerland's replacement capital: the current capital, no star and no VP
//   E  Germany's original capital, captured by you, who holds its point (hollow star)
//   G  Germany's replacement capital: no star
//   H  your teammate's own original capital (filled star in the Aztec ink)
//   T  the Mongols' original capital, captured by your teammate, who holds its point (filled star)
//   U  the Mongols' replacement capital
import { fromOffset } from '../hex';
import { FixtureClient, createState, makeStandardSetup, refreshVision, spawnUnit } from './fixtureEngine';
import type { MCity } from './fixtureState';

export function capitalsFixture() {
  const setup = makeStandardSetup('99124', 'India', 'Byzantium', 'Explorer', 'Rival');
  setup.width = 40; setup.height = 24;
  const factions = ['India', 'Byzantium', 'Switzerland', 'Germany', 'Aztecs', 'Mongols'] as const;
  const teams = [0, 1, 2, 3, 0, 4];
  setup.players = setup.players.slice(0, 6).map((p, i) => ({ ...p, team: teams[i], is_human: true, faction: factions[i] }));
  const s = createState(setup);
  s.units = []; s.cities = []; s.nextUnitId = 0; s.nextCityId = 0;
  s.terrain.fill('Ocean'); s.ruin.fill(0); s.wonder = undefined; s.owner.fill(-1); s.tileCity.fill(-1);
  for (let row = 2; row <= 9; row++) for (let col = 1; col <= 28; col++) s.terrain[row * s.width + col] = 'Plains';
  for (const p of s.players) { p.explored.fill(1); p.gold = 500; }
  // Founding order is the capital order: each empire's first Settler is its capital.
  const sites: [number, number, number][] = [[0, 3, 5], [1, 9, 4], [2, 16, 4], [3, 13, 8], [4, 26, 8], [5, 19, 9], [2, 22, 5], [3, 8, 8], [5, 27, 4]];
  for (const [owner, col, row] of sites) spawnUnit(s, owner, 'Settler', fromOffset(col, row), false);
  refreshVision(s);
  const client = new FixtureClient();
  client.load(JSON.stringify(s));
  client.endTurn(Object.fromEntries([0, 1, 2, 3, 4, 5].map((p) => [p, s.units.filter((u) => u.owner === p).map((u) => ({ type: 'FoundCity' as const, unit: u.id }))])));
  const f = client.debugState();
  const at = (col: number, row: number) => f.cities.find((c) => c.pos.q === fromOffset(col, row).q && c.pos.r === fromOffset(col, row).r)!;
  const a = at(3, 5), b = at(9, 4), c = at(16, 4), e = at(13, 8), h = at(26, 8), t = at(19, 9), d = at(22, 5), g = at(8, 8), u = at(27, 4);
  const retake = (city: MCity, owner: number) => {
    for (let i = 0; i < f.tileCity.length; i++) if (f.tileCity[i] === city.id) f.owner[i] = owner;
    city.owner = owner;
  };
  for (const city of [a, b, c, e, h, t]) { city.isCapital = true; city.originalCapital = true; }
  for (const city of [d, g, u]) { city.isCapital = false; city.originalCapital = false; }
  // C falls to Byzantium (Switzerland's replacement is D), E to you (Germany's is G), T to your teammate (the Mongols' is U).
  retake(c, 1); c.isCapital = false; d.isCapital = true;
  retake(e, 0); e.isCapital = false; g.isCapital = true;
  retake(t, 4); t.isCapital = false; u.isCapital = true;
  for (const city of [a, b, c, d, e, g, h, t, u]) city.population = 3;
  // Garrisons: your Soldier in your capital, Byzantium's in its own.
  spawnUnit(f, 0, 'Soldier', a.pos, false);
  spawnUnit(f, 1, 'Soldier', b.pos, false);
  // A scout beside each rival city keeps them in sight.
  for (const [col, row] of [[10, 5], [15, 5], [21, 6], [17, 8]] as const) spawnUnit(f, 0, 'Scout', fromOffset(col, row), false);
  refreshVision(f);
  return {
    fixture: f, own: a.id, held: b.id, captured: c.id, replacement: d.id, scored: e.id, replacementGermany: g.id,
    teammateOwn: h.id, teammateScored: t.id, replacementMongols: u.id,
  };
}
