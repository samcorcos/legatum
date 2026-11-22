// Same-class units of different eras face each other (user, 2026-10-11, playing Sparta: "The Hoplite
// and the plus-one era from the Hoplite looked nearly identical"). Your Spartan line (Hoplite,
// Phalangite, Pikeman, an Archer and a Horseman) meets Athens at War: every melee era from Spearman
// to Musketman, every ranged era from Archer to Arquebusier and every mounted era from Horseman to
// Cuirassier. Mixed open terrain with woods and hills, as in play. Injected visual fixture only; not
// imported by the app.
import { distance, fromOffset, hexEq, neighbors, tileIndex } from '../hex';
import type { Hex, UnitKind } from '../types';
import { FixtureClient } from './fixtureEngine';
import { createState, makeStandardSetup, spawnUnit } from './fixtureEngine';
import { refreshVision } from './fixtureEngine';

export function unitEraFixture() {
  const setup = makeStandardSetup('99124', 'Sparta', 'Athens', 'Leonidas', 'Pericles');
  setup.width = 26; setup.height = 14;
  setup.players = setup.players.slice(0, 2).map((p, i) => ({ ...p, team: i, is_human: true, faction: (['Sparta', 'Athens'] as const)[i] }));
  const s = createState(setup);
  s.units = []; s.cities = []; s.nextUnitId = 0; s.nextCityId = 0;
  s.terrain.fill('Ocean'); s.ruin.fill(0); s.wonder = undefined; s.owner.fill(-1); s.tileCity.fill(-1);
  s.resource?.fill(null);
  for (let row = 1; row <= 12; row++) for (let col = 1; col <= 24; col++) {
    const k = (col * 7 + row * 13) % 11;
    s.terrain[row * s.width + col] = k === 0 || k === 5 ? 'Forest' : k === 3 ? 'Hills' : k === 8 ? 'Grassland' : 'Plains';
  }
  for (const p of s.players) { p.explored.fill(1); p.gold = 500; }
  spawnUnit(s, 0, 'Settler', fromOffset(3, 7), false);
  spawnUnit(s, 1, 'Settler', fromOffset(21, 6), false);
  refreshVision(s);
  const client = new FixtureClient();
  client.load(JSON.stringify(s));
  client.endTurn(Object.fromEntries([0, 1].map((p) => [p, s.units.filter((u) => u.owner === p).map((u) => ({ type: 'FoundCity' as const, unit: u.id }))])));
  const f = client.debugState();
  f.turn = 60;
  for (const c of f.cities) c.population = 6;
  f.cities[0].name = 'Sparta';
  f.cities[1].name = 'Athens';
  for (const r of f.diplomacy!.relations) r.state = 'War';
  const at = (col: number, row: number) => fromOffset(col, row);
  const free = (h: Hex) => f.terrain[tileIndex(h, f.width, f.height)] !== 'Ocean' && !f.units.some((u) => hexEq(u.pos, h)) && !f.cities.some((c) => hexEq(c.pos, h));
  const put = (owner: number, kind: UnitKind, h: Hex) => { if (!free(h)) throw new Error(`occupied ${h.q},${h.r}`); return spawnUnit(f, owner, kind, h, false); };

  // Your Spartan line on columns 10-11; Athens fills columns 12-13, every unit within your units' sight.
  const hoplite = put(0, 'Spearman', at(11, 5));
  const mine = [
    put(0, 'Phalangite', at(11, 3)),
    put(0, 'Pikeman', at(11, 7)),
    put(0, 'Archer', at(10, 4)),
    put(0, 'Horseman', at(10, 8)),
  ];
  const own = [hoplite, ...mine];
  // Athens: the Phalangite right beside the Hoplite (the user's fight), then every era of each class.
  const besideEast = neighbors(hoplite.pos).filter(free).sort((a, b) => b.q - a.q)[0];
  const phalangite = put(1, 'Phalangite', besideEast);
  const slots: Hex[] = [];
  for (const col of [12, 13]) for (let row = 2; row <= 9; row++) {
    const h = at(col, row);
    if (free(h) && own.some((u) => distance(u.pos, h) <= 2)) slots.push(h);
  }
  const kinds: UnitKind[] = ['Spearman', 'Pikeman', 'Musketman', 'Archer', 'CompositeArcher', 'Crossbowman', 'Arquebusier', 'Horseman', 'Cataphract', 'Knight', 'Cuirassier'];
  if (slots.length < kinds.length) throw new Error(`only ${slots.length} slots`);
  const theirs = kinds.map((k, i) => put(1, k, slots[i]));
  refreshVision(f);
  f.players[0].explored.fill(1);
  return { fixture: f, hoplite: hoplite.id, phalangite: phalangite.id, phalangiteAt: phalangite.pos, mine: mine.map((u) => u.id), theirs: theirs.map((u) => u.id), centre: at(12, 6) };
}
