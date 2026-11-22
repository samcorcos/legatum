// Five cities of yours for the Cities overview and "where to build this", each different on purpose:
//   Bern    the capital, a Library, a Workshop, building a Market (the Great Library is built here)
//   Zurich  a Library and 200 Production already banked toward the Great Library
//   Basel   the most Production but no Library, so the Great Library is blocked here
//   Geneva  small, no Library, and idle (nothing chosen)
//   Lugano  a young city on Forest building a Soldier
// Philosophy and Electricity are known, so the Great Library and Research Lab are on offer.
// Injected visual fixture only; not imported by the app.
import { fromOffset, neighbors } from '../hex';
import { FixtureClient } from './fixtureEngine';
import { createState, makeStandardSetup, spawnUnit } from './fixtureEngine';
import { refreshVision } from './fixtureEngine';
import type { BuildingKind, Faction, ProductionItem, TechId } from '../types';

export function citiesFixture(faction: Faction = 'Switzerland') {
  const setup = makeStandardSetup('99124', faction, 'Mongols', 'Ada', 'Rival');
  setup.width = 28; setup.height = 14;
  setup.players = setup.players.slice(0, 2).map((p, i) => ({ ...p, team: i, is_human: i === 0, faction: ([faction, 'Mongols'] as const)[i], name: ['Ada', 'Epoch'][i] }));
  const s = createState(setup);
  s.units = []; s.cities = []; s.nextUnitId = 0; s.nextCityId = 0;
  s.terrain.fill('Ocean'); s.ruin.fill(0); s.wonder = undefined; s.owner.fill(-1); s.tileCity.fill(-1);
  s.resource?.fill(null);
  for (let row = 2; row <= 11; row++) for (let col = 1; col <= 26; col++) s.terrain[row * s.width + col] = (col + row) % 5 === 0 ? 'Hills' : (col * row) % 7 === 0 ? 'Forest' : (col + 2 * row) % 3 === 0 ? 'Grassland' : 'Plains';
  for (const p of s.players) p.explored.fill(1);
  const sites: [number, number, number][] = [[0, 4, 4], [0, 4, 9], [0, 9, 3], [0, 9, 8], [0, 14, 6], [1, 22, 6]];
  for (const [owner, col, row] of sites) spawnUnit(s, owner, 'Settler', fromOffset(col, row), false);
  refreshVision(s);
  const client = new FixtureClient();
  client.load(JSON.stringify(s));
  client.endTurn(Object.fromEntries([0, 1].map((p) => [p, s.units.filter((u) => u.owner === p).map((u) => ({ type: 'FoundCity' as const, unit: u.id }))])));
  const f = client.debugState();
  for (const r of f.diplomacy!.relations) r.state = 'Peace';
  const mine = f.cities.filter((c) => c.owner === 0).sort((a, b) => a.id - b.id);
  const spec: { name: string; population: number; buildings: BuildingKind[]; producing: ProductionItem | null; prod: number; banked?: number }[] = [
    { name: 'Bern', population: 6, buildings: ['Laboratory', 'Workshop'], producing: item('Market'), prod: 40 },
    { name: 'Zurich', population: 4, buildings: ['Laboratory'], producing: item('Granary'), prod: 10, banked: 200 },
    { name: 'Basel', population: 7, buildings: ['Workshop'], producing: item('Granary'), prod: 20 },
    { name: 'Geneva', population: 2, buildings: [], producing: null, prod: 0 },
    { name: 'Lugano', population: 3, buildings: [], producing: { kind: 'Unit', unit: 'Soldier' }, prod: 12 },
  ];
  mine.forEach((c, i) => {
    const sp = spec[i];
    c.name = sp.name; c.population = sp.population; c.buildings = sp.buildings; c.producing = sp.producing; c.prod = sp.prod;
    c.isCapital = i === 0;
    if (sp.banked) c.productionBanked = [{ item: { kind: 'Wonder', building: 'RelayUplink', tile: neighbors(c.pos)[0] }, progress: sp.banked }];
  });
  const known: TechId[] = ['Irrigation', 'BronzeWorking', 'Masonry', 'Currency', 'Writing', 'Philosophy', 'Electricity'];
  f.players[0].techs = [...known];
  f.players[0].science = 11;
  f.players[0].gold = 300;
  spawnUnit(f, 0, 'Scout', fromOffset(6, 7), false);
  refreshVision(f);
  return f;
}

function item(building: BuildingKind): ProductionItem { return { kind: 'Building' as const, building }; }
