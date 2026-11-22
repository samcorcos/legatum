// The Mongol Keshig in the field (user, 2026-10-17): your Keshig on open ground with an Athenian Spearman next to
// it (shot and charge both on offer), an Archer three hexes off (a charge only: out of bow range) and a Spearman
// beside your city. A save for the real core (`coreFixture`), built with Rust-backed fixture helpers. Visual
// fixture only; not imported by the app.
import { fromOffset, neighbors, hexEq, tileIndex } from '../hex';
import type { Hex, UnitKind } from '../types';
import { FixtureClient } from './fixtureEngine';
import { createState, makeStandardSetup, spawnUnit } from './fixtureEngine';
import { refreshVision } from './fixtureEngine';

export function keshigFixture() {
  const setup = makeStandardSetup('99124', 'Mongols', 'Athens', 'Temujin', 'Pericles');
  setup.width = 26; setup.height = 14;
  setup.players = setup.players.slice(0, 2).map((p, i) => ({ ...p, team: i, is_human: i === 0, faction: (['Mongols', 'Athens'] as const)[i] }));
  const s = createState(setup);
  s.units = []; s.cities = []; s.nextUnitId = 0; s.nextCityId = 0;
  s.terrain.fill('Ocean'); s.ruin.fill(0); s.wonder = undefined; s.owner.fill(-1); s.tileCity.fill(-1);
  s.resource?.fill(null);
  for (let row = 1; row <= 12; row++) for (let col = 1; col <= 24; col++) {
    const k = (col * 7 + row * 13) % 11;
    s.terrain[row * s.width + col] = k === 0 || k === 5 ? 'Forest' : k === 3 ? 'Hills' : k === 8 ? 'Grassland' : 'Plains';
  }
  for (const p of s.players) { p.explored.fill(1); p.gold = 500; }
  spawnUnit(s, 0, 'Settler', fromOffset(8, 6), false);
  spawnUnit(s, 1, 'Settler', fromOffset(20, 6), false);
  refreshVision(s);
  const client = new FixtureClient();
  client.load(JSON.stringify(s));
  client.endTurn(Object.fromEntries([0, 1].map((p) => [p, s.units.filter((u) => u.owner === p).map((u) => ({ type: 'FoundCity' as const, unit: u.id }))])));
  const f = client.debugState();
  f.turn = 60;
  for (const c of f.cities) c.population = 6;
  f.cities[0].name = 'Karakorum';
  f.cities[1].name = 'Athens';
  for (const r of f.diplomacy!.relations) r.state = 'War';
  f.players[0].techs.push('BronzeWorking', 'Masonry', 'IronWorking');
  const at = (col: number, row: number) => fromOffset(col, row);
  const free = (h: Hex) => f.terrain[tileIndex(h, f.width, f.height)] !== 'Ocean' && !f.units.some((u) => hexEq(u.pos, h)) && !f.cities.some((c) => hexEq(c.pos, h));
  const put = (owner: number, kind: UnitKind, h: Hex) => { if (!free(h)) throw new Error(`occupied ${h.q},${h.r}`); return spawnUnit(f, owner, kind, h, false); };
  const keshig = put(0, 'Cataphract', at(13, 6));
  const east = neighbors(keshig.pos).filter(free).sort((a, b) => b.q - a.q)[0];
  const spearman = put(1, 'Soldier', east);
  const far = [at(16, 5), at(16, 6), at(16, 7), at(16, 4)].find(free)!;
  const archer = put(1, 'Ranger', far);
  // A Scout beside the Archer so your side can see it: three hexes is beyond the Keshig's own sight.
  put(0, 'Scout', neighbors(far).filter(free).sort((a, b) => a.q - b.q)[0]);
  refreshVision(f);
  f.players[0].explored.fill(1);
  return { fixture: f, keshig: keshig.id, spearman: spearman.id, spearmanAt: spearman.pos, archer: archer.id, archerAt: archer.pos, centre: at(15, 6) };
}
