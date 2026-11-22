// Faction goals increment 5 for the screen tour and tests: the Yam Relay Station in each state a player
// meets it. Injected visual fixture only; not imported by the app.
//  own:       you are Mongols; the Station stands in your city B (its land is open to everyone), A is walled
//  ownWalled: as `own`, but the Station's own city is walled (the plaque must clear the wall ring)
//  buildable: you are Mongols with no Station yet: any city can build it
//  building:  as `buildable`, but C is already building it, so A's card is locked
//  lost:      you are Mongols; a rival (Sparta) captured the Station's city
//  rival:     you are Sparta; the Mongols rival holds the Station in a city you can see
//  captured:  you are Sparta and own the Mongols rival's Station city (you took it)
import { fromOffset, within } from '../hex';
import { FixtureClient } from './fixtureEngine';
import { createState, makeStandardSetup, spawnUnit } from './fixtureEngine';
import { idxOf, refreshVision } from './fixtureEngine';
import { refreshRoads } from './fixtureEngine';
import type { Faction } from '../types';

export type HallMode = 'own' | 'ownWalled' | 'buildable' | 'building' | 'lost' | 'rival' | 'captured';

export function hallFixture(mode: HallMode) {
  const glass = mode !== 'rival' && mode !== 'captured';
  const factions: Faction[] = glass ? ['Mongols', 'Sparta', 'India'] : ['Sparta', 'Mongols', 'India'];
  const setup = makeStandardSetup('99125', factions[0], factions[1], 'Ada', 'Rival');
  setup.width = 30; setup.height = 14;
  setup.players = setup.players.slice(0, 3).map((p, i) => ({ ...p, team: i, is_human: i === 0, faction: factions[i], name: ['Ada', 'Marrow', 'Hive'][i] }));
  const s = createState(setup);
  s.units = []; s.cities = []; s.nextUnitId = 0; s.nextCityId = 0;
  s.terrain.fill('Ocean'); s.ruin.fill(0); s.wonder = undefined; s.owner.fill(-1); s.tileCity.fill(-1);
  for (let row = 1; row <= 12; row++) for (let col = 1; col <= 28; col++) s.terrain[row * s.width + col] = (col * 3 + row) % 7 === 0 ? 'Forest' : (col + row * 2) % 11 === 0 ? 'Hills' : (col * row) % 9 === 0 ? 'Grassland' : 'Plains';
  for (const p of s.players) p.explored.fill(1);
  // The Mongols empire (seat 0 or 1) owns A, B, C; the other empire has two cities east of them.
  const g = glass ? 0 : 1, o = glass ? 1 : 0;
  const mine: [number, number, number][] = [[g, 5, 5], [g, 12, 7], [g, 6, 10], [o, 22, 5], [o, 23, 10], [2, 28, 3]];
  for (const [owner, col, row] of mine) spawnUnit(s, owner, 'Settler', fromOffset(col, row), false);
  refreshVision(s);
  const client = new FixtureClient();
  client.load(JSON.stringify(s));
  client.endTurn(Object.fromEntries([0, 1, 2].map((p) => [p, s.units.filter((u) => u.owner === p).map((u) => ({ type: 'FoundCity' as const, unit: u.id }))])));
  const f = client.debugState();
  f.turn = 12;
  for (const r of f.diplomacy!.relations) r.state = 'Peace';
  const at = (col: number, row: number) => f.cities.find((c) => c.pos.q === fromOffset(col, row).q && c.pos.r === fromOffset(col, row).r)!;
  const A = at(5, 5), B = at(12, 7), C = at(6, 10), D = at(22, 5), E = at(23, 10);
  for (const c of f.cities) { c.population = 4; c.buildings = []; c.producing = null; c.prod = 0; c.productionQueue = []; c.productionBanked = []; }
  // The Mongols empire's borders reach three hexes around each city, so the open road has land to cross.
  for (const c of [A, B, C, D, E]) {
    for (const h of within(c.pos, 3)) { const i = idxOf(f, h); if (i >= 0 && f.terrain[i] !== 'Ocean' && f.owner[i] < 0) { f.owner[i] = c.owner; f.tileCity[i] = c.id; } }
  }
  A.buildings = ['Walls']; A.population = 6;
  const holderOf = (mode: HallMode) => (mode === 'lost' ? 1 : mode === 'captured' ? 0 : g);
  if (mode === 'own' || mode === 'ownWalled' || mode === 'lost' || mode === 'rival' || mode === 'captured') {
    B.buildings = mode === 'ownWalled' ? ['Walls', 'YamRelayStation'] : ['YamRelayStation'];
    B.population = 6;
    f.players[g].hallCity = B.id;
    f.players[g].goalHolder = holderOf(mode);
    f.players[g].factionGoalDone = holderOf(mode) === g;
    f.players[g].factionGoalReachedBefore = true;
    if (holderOf(mode) !== g) {
      B.owner = holderOf(mode);
      for (let i = 0; i < f.owner.length; i++) if (f.tileCity[i] === B.id) f.owner[i] = B.owner;
    }
  }
  // The Station's roads, as the first turn after it completed would have laid them.
  if (mode === 'building') { C.producing = { kind: 'Wonder', building: 'YamRelayStation', tile: fromOffset(7, 10) }; C.prod = 40; }
  f.players[0].gold = 600;
  // A scout beside the rival's Station city, so it is in sight.
  const scout = spawnUnit(f, 0, 'Scout', fromOffset(mode === 'rival' || mode === 'captured' ? 12 : 4, mode === 'rival' || mode === 'captured' ? 6 : 6), false);
  if (f.players[g].hallCity) refreshRoads(f);
  refreshVision(f);
  return { fixture: f, a: A.id, b: B.id, c: C.id, d: D.id, e: E.id, hallPos: B.pos, scout: scout.id, glassSeat: g, otherSeat: o };
}
