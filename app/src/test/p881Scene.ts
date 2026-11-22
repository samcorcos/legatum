// The p881 horseman as a playable fixture for the real app (WASM core): the turn-29 map, units,
// walls and fog from situation.json, so the attack forecast the core computes is the one the user
// had. `variant: 'seen'` is the same charge with Nara's garrison in sight. Injected visual fixture
// only; the app never imports it.
import situation from './p881/situation.json' with { type: 'json' };
import { createState, makeStandardSetup, spawnUnit } from './fixtureEngine';
import { refreshVision } from './fixtureEngine';
import type { MCity, MState } from './fixtureState';
import type { Faction, Hex, Terrain, UnitKind } from '../types';
import type { RawTile } from './p881Fixture';
import { tileIndex } from '../hex';

/** fog: turn 29 exactly. seen: the same charge with the garrison in sight. retreat: turn 30 exactly. */
export type P881Variant = 'fog' | 'seen' | 'retreat';

export function p881Scene(variant: P881Variant = 'fog') {
  const day = variant === 'fog' ? situation.t29 : situation.t30;
  const setup = makeStandardSetup('99123', 'Sparta', 'India', 'Sam', 'India');
  setup.width = day.width; setup.height = day.height;
  setup.players = day.players.map((p) => ({ name: p.name, faction: p.faction as Faction, team: p.team, is_human: true }));
  const s: MState = createState(setup);
  s.turn = day.turn;
  s.units = []; s.cities = []; s.nextUnitId = 0; s.nextCityId = 0;
  s.ruin.fill(0); s.wonder = undefined; s.resource?.fill(null); s.improved?.fill(0);
  s.road?.fill(0); s.owner.fill(-1); s.tileCity.fill(-1);
  (day.tiles as RawTile[]).forEach(([, , terrain, owner], i) => { s.terrain[i] = (terrain || 'Ocean') as Terrain; s.owner[i] = owner; });
  for (const p of s.players) { p.explored.fill(0); p.gold = 200; }
  (day.tiles as RawTile[]).forEach(([explored], i) => { s.players[0].explored[i] = explored; });
  s.knownOwner = { '0': (day.tiles as RawTile[]).map(([explored, , , owner]) => explored ? owner : -1) };
  s.knownTerrain = { '0': (day.tiles as RawTile[]).map(([explored]) => explored) };
  s.knownRuins = { '0': Array(s.terrain.length).fill(0) };
  for (const p of s.players.slice(1)) p.explored.fill(1);

  for (const r of s.diplomacy!.relations) {
    const real = (day.relations as { a: number; b: number; state: string }[]).find((x) => x.a === r.a && x.b === r.b);
    r.state = (real?.state ?? 'Peace') as typeof r.state;
  }
  const ids: Record<string, number> = {};
  for (const c of day.cities) {
    const city: MCity = {
      id: c.id, owner: c.owner, founder: c.owner, name: c.name, pos: c.pos, population: c.population, food: 0, prod: 0,
      producing: null, bought: null, buildings: c.walls ? ['Walls'] : [], hp: c.hp, tilesClaimed: 0, grownTiles: [], isCapital: c.is_capital, foundedTurn: 5, resistance: 0,
    };
    s.cities.push(city);
    s.nextCityId = Math.max(s.nextCityId, c.id + 1);
    s.tileCity[tileIndex(c.pos, s.width, s.height)] = c.id;
    ids[c.name] = c.id;
  }
  s.knownCities = { '0': Object.fromEntries(s.cities
    .filter((c) => s.players[0].explored[tileIndex(c.pos, s.width, s.height)])
    .map((c) => [c.id, structuredClone(c)])) };
  // The turn-29 fog: Nara is known (walls and all) from earlier but out of sight, so it is seen once and then left.
  if (variant !== 'retreat') {
    const scout = spawnUnit(s, 0, 'Scout', variant === 'fog' ? { q: 19, r: 14 } : { q: 20, r: 14 }, false);
    refreshVision(s);
    if (variant === 'fog') { s.units = s.units.filter((u) => u !== scout); refreshVision(s); }
  }
  for (const u of day.units) {
    if (variant === 'seen' && u.id === 28) continue;
    const unit = spawnUnit(s, u.owner, u.kind as UnitKind, u.pos, false);
    unit.hp = u.hp; unit.fortify = u.fortify;
    ids[`${u.kind}#${u.id}`] = unit.id;
  }
  if (variant === 'seen') {
    // Turn 30's Horseman is wounded and elsewhere: this is the same charge with the garrison in sight.
    const horse = spawnUnit(s, 0, 'Horseman', { q: 19, r: 11 }, false);
    ids['Horseman#28'] = horse.id;
    const archer = spawnUnit(s, 1, 'Ranger', { q: 19, r: 13 }, false);
    archer.fortify = 6;
    ids['Archer#15'] = archer.id;
  }
  refreshVision(s);
  return { fixture: s, horseman: ids['Horseman#28'], archer: ids['Ranger#15'] ?? ids['Archer#15'], nara: ids.Nara, archerTile: { q: 19, r: 13 } as Hex };
}
