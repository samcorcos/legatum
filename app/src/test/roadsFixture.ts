// Roads on a real generated map, for the road-art shots and the screen tour: N empires (seat 0 a
// Mongols human), a Courier Hall in seat 0's capital, extra cities scattered over land at least
// five hexes apart, borders three hexes around each city, and the roads the core lays. Injected
// visual fixture only; not imported by the app.
import { distance, fromOffset, within } from '../hex';
import { FixtureClient } from './fixtureEngine';
import { refreshRoads } from './fixtureEngine';
import { createState, makeStandardSetup, spawnUnit } from './fixtureEngine';
import { idxOf, refreshVision } from './fixtureEngine';
import type { MCity, MState } from './fixtureState';
import type { Faction } from '../types';
import { moveCost } from '../data';

const OTHERS: Faction[] = ['Sparta', 'India', 'Switzerland', 'Byzantium', 'Celts', 'Germany', 'Athens'];

export function roadsFixture(opts: { players?: number; width?: number; height?: number; seed?: string; extra?: number; hall?: boolean; works?: 'offer' | 'built' } = {}) {
  const { players = 4, width = 48, height = 32, seed = '31337', extra = 14, hall = true, works } = opts;
  const setup = makeStandardSetup(seed, 'Mongols', 'Sparta', 'Ada', 'Marrow');
  setup.width = width; setup.height = height;
  const names = ['Ada', 'Marrow', 'Hive', 'Toll', 'Byzantium', 'Celts', 'Forge', 'Signal'];
  setup.players = setup.players.slice(0, players).map((p, i) => ({ ...p, team: 0, is_human: i === 0, faction: i === 0 ? 'Mongols' : OTHERS[i - 1], name: names[i] }));
  const s = createState(setup);
  // Every empire starts with its capital already founded (no starting Settler), so the capitals stay and
  // only the starting units go; their borders grow to three hexes below like every other city's.
  s.units = [];
  for (const p of s.players) p.explored.fill(1);
  const client = new FixtureClient();
  client.load(JSON.stringify(s));
  client.endTurn(Object.fromEntries(s.players.map((p) => [p.id, []])));
  const f: MState = client.debugState();
  f.turn = 40;
  for (const r of f.diplomacy?.relations ?? []) r.state = 'Peace';
  const found = (owner: number, col: number, row: number) => {
    const c: MCity = {
      id: ++f.nextCityId, owner, founder: owner, name: `Town ${f.nextCityId}`, pos: fromOffset(col, row), population: 3, food: 0, prod: 0,
      producing: null, bought: null, buildings: [], hp: 200, grownTiles: [], tilesClaimed: 0, isCapital: false, foundedTurn: 1, resistance: 0,
    };
    f.cities.push(c);
    return c;
  };
  f.nextCityId = Math.max(0, ...f.cities.map((c) => c.id));
  // Extra cities: a deterministic scan over land, each at least five hexes from every city.
  let n = 0;
  for (let row = 2; row < height - 2 && n < extra; row += 3) {
    for (let col = 2 + ((row * 5) % 7); col < width - 2 && n < extra; col += 7) {
      const h = fromOffset(col, row);
      const i = idxOf(f, h);
      if (moveCost(f.terrain[i]) === null || f.cities.some((c) => distance(c.pos, h) < 5)) continue;
      const near = f.cities.reduce((a, c) => (distance(c.pos, h) < distance(a.pos, h) ? c : a), f.cities[0]);
      found(near.owner, col, row);
      n++;
    }
  }
  for (const c of f.cities) {
    c.buildings = []; c.producing = null; c.prod = 0; c.productionQueue = []; c.productionBanked = [];
    for (const h of within(c.pos, 3)) { const i = idxOf(f, h); if (i >= 0 && moveCost(f.terrain[i]) !== null && f.owner[i] < 0) { f.owner[i] = c.owner; f.tileCity[i] = c.id; } }
  }
  // The Hall stands in the most central city, which seat 0 owns: the roads then radiate across the whole map.
  const middle = fromOffset(Math.floor(width / 2), Math.floor(height / 2));
  const mine = f.cities.reduce((a, c) => (distance(c.pos, middle) < distance(a.pos, middle) ? c : a), f.cities[0]);
  mine.owner = 0; mine.name = 'Crossroads';
  for (let i = 0; i < f.owner.length; i++) if (f.tileCity[i] === mine.id) f.owner[i] = 0;
  if (hall) {
    mine.buildings = ['CourierHall'];
    f.players[0].hallCity = mine.id;
    f.players[0].goalHolder = 0; f.players[0].factionGoalDone = true; f.players[0].factionGoalReachedBefore = true;
  }
  if (works) {
    // Roadworks: you know Field Logistics and can build them; 'built' stands in your central city, with its links.
    f.players[0].techs = [...new Set([...f.players[0].techs, 'Masonry', 'Engineering'] as const)];
    f.players[0].gold = 600;
    mine.population = 4;
    if (works === 'built') mine.buildings = ['Roadworks'];
  }
  spawnUnit(f, 0, 'Scout', fromOffset(Math.min(width - 2, mine.pos.q + 3), 8), false);
  if (hall || works === 'built') refreshRoads(f);
  refreshVision(f);
  return { fixture: f, hall: mine, hallPos: mine.pos, center: fromOffset(Math.floor(width / 2), Math.floor(height / 2)) };
}
