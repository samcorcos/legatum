// Fixture for the Spartan Hoplite phalanx:
//   - `phalanxFixture`: your Spartans, two Hoplites side by side, face an Athenian Spearman at War. With the first Hoplite
//     selected, the attack forecast lists "Phalanx: next to another Hoplite".
// Injected visual fixtures only; not imported by the app.
import { fromOffset, hexEq, neighbors, tileIndex } from '../hex';
import type { Faction, Hex, UnitKind } from '../types';
import { FixtureClient } from './fixtureEngine';
import { createState, makeStandardSetup, spawnUnit } from './fixtureEngine';
import { refreshVision } from './fixtureEngine';

function base(factions: [Faction, Faction]) {
  const setup = makeStandardSetup('99125', factions[0], factions[1], 'Leonidas', 'Pericles');
  setup.width = 20; setup.height = 12;
  setup.players = setup.players.slice(0, 2).map((p, i) => ({ ...p, team: i, is_human: true, faction: factions[i] }));
  const s = createState(setup);
  s.units = []; s.cities = []; s.nextUnitId = 0; s.nextCityId = 0;
  s.terrain.fill('Ocean'); s.ruin.fill(0); s.wonder = undefined; s.owner.fill(-1); s.tileCity.fill(-1);
  s.resource?.fill(null);
  for (let row = 1; row <= 10; row++) for (let col = 1; col <= 18; col++) s.terrain[row * s.width + col] = 'Plains';
  for (const p of s.players) { p.explored.fill(1); p.gold = 500; }
  spawnUnit(s, 0, 'Settler', fromOffset(3, 6), false);
  spawnUnit(s, 1, 'Settler', fromOffset(16, 6), false);
  refreshVision(s);
  const client = new FixtureClient();
  client.load(JSON.stringify(s));
  client.endTurn(Object.fromEntries([0, 1].map((p) => [p, s.units.filter((u) => u.owner === p).map((u) => ({ type: 'FoundCity' as const, unit: u.id }))])));
  const f = client.debugState();
  f.turn = 40;
  for (const c of f.cities) c.population = 4;
  return f;
}

export function phalanxFixture() {
  const f = base(['Sparta', 'Athens']);
  f.cities[0].name = 'Sparta';
  f.cities[1].name = 'Athens';
  for (const r of f.diplomacy!.relations) r.state = 'War';
  const free = (h: { q: number; r: number }) => f.terrain[tileIndex(h, f.width, f.height)] !== 'Ocean' && !f.units.some((u) => hexEq(u.pos, h)) && !f.cities.some((c) => hexEq(c.pos, h));
  const put = (owner: number, kind: UnitKind, h: Hex) => { if (!free(h)) throw new Error(`occupied ${h.q},${h.r}`); return spawnUnit(f, owner, kind, h, false); };
  const first = put(0, 'Soldier', fromOffset(9, 6));
  const east = neighbors(first.pos).filter(free).sort((a, b) => b.q - a.q)[0];
  const enemy = put(1, 'Soldier', east);
  // The second Hoplite stands beside the first, away from the enemy.
  const side = neighbors(first.pos).filter(free).sort((a, b) => a.q - b.q)[0];
  const second = put(0, 'Soldier', side);
  refreshVision(f);
  f.players[0].explored.fill(1);
  return { fixture: f, hoplite: first.id, partner: second.id, enemy: enemy.id, enemyAt: enemy.pos, centre: first.pos };
}
