// Mid-game territory for the border-legibility shots (never imported by the app): N empires of
// different factions on a generated map, each with its capital and a ring of towns, so every empire
// has several neighbours and their borders meet. Towns belong to the nearest capital's empire and
// claim land three hexes around themselves, first come first served. Seat 0 has explored the lot.
import { distance, fromOffset, within } from '../hex';
import { FixtureClient } from './fixtureEngine';
import { createState, makeStandardSetup, spawnUnit } from './fixtureEngine';
import { idxOf, refreshVision } from './fixtureEngine';
import type { MCity, MState } from './fixtureState';
import type { Faction } from '../types';
import { moveCost } from '../data';

const FACTIONS: Faction[] = ['Mongols', 'Sparta', 'India', 'Switzerland', 'Byzantium', 'Celts', 'Germany', 'Athens'];

export function bordersFixture(opts: { players?: number; width?: number; height?: number; seed?: string; spacing?: number } = {}) {
  const { players = 4, width = 48, height = 32, seed = '31337', spacing = 5 } = opts;
  const setup = makeStandardSetup(seed, 'Mongols', 'Sparta', 'Ada', 'Marrow');
  setup.width = width; setup.height = height;
  const names = ['Ada', 'Marrow', 'Hive', 'Toll', 'Byzantium', 'Celts', 'Forge', 'Signal'];
  setup.players = setup.players.slice(0, players).map((p, i) => ({ ...p, team: i, is_human: i === 0, faction: FACTIONS[i], name: names[i] }));
  const s = createState(setup);
  s.units = s.units.filter((u) => u.kind === 'Settler'); s.cities = []; s.nextCityId = 0;
  s.owner.fill(-1); s.tileCity.fill(-1);
  for (const p of s.players) p.explored.fill(1);
  const client = new FixtureClient();
  client.load(JSON.stringify(s));
  client.endTurn(Object.fromEntries(s.players.map((p) => [p.id, s.units.filter((u) => u.owner === p.id && u.kind === 'Settler').slice(0, 1).map((u) => ({ type: 'FoundCity' as const, unit: u.id }))])));
  const f: MState = client.debugState();
  f.turn = 90;
  for (const r of f.diplomacy?.relations ?? []) r.state = 'Peace';
  const capitals = f.cities.filter((c) => c.isCapital);
  f.nextCityId = Math.max(0, ...f.cities.map((c) => c.id));
  const found = (owner: number, col: number, row: number) => {
    const c: MCity = {
      id: ++f.nextCityId, owner, founder: owner, name: `Town ${f.nextCityId}`, pos: fromOffset(col, row), population: 4, food: 0, prod: 0,
      producing: null, bought: null, buildings: [], hp: 200, grownTiles: [], tilesClaimed: 0, isCapital: false, foundedTurn: 1, resistance: 0,
    };
    f.cities.push(c);
    return c;
  };
  // Towns: a deterministic scan over land, each at least `spacing` hexes from every city, owned by the nearest capital's empire.
  for (let row = 1; row < height - 1; row += 2) {
    for (let col = 1 + ((row * 3) % 5); col < width - 1; col += 3) {
      const h = fromOffset(col, row);
      const i = idxOf(f, h);
      if (moveCost(f.terrain[i]) === null || f.cities.some((c) => distance(c.pos, h) < spacing)) continue;
      const cap = capitals.reduce((a, c) => (distance(c.pos, h) < distance(a.pos, h) ? c : a), capitals[0]);
      found(cap.owner, col, row);
    }
  }
  for (const c of f.cities) {
    c.buildings = []; c.producing = null; c.prod = 0; c.productionQueue = []; c.productionBanked = [];
    if (c.isCapital) c.population = 7;
  }
  // Claim round by round (radius 1, 2, 3) so neighbours share the contested ground fairly rather than one empire taking all of it.
  for (let ring = 1; ring <= 3; ring++) {
    for (const c of f.cities) {
      for (const h of within(c.pos, ring)) {
        const i = idxOf(f, h);
        if (i >= 0 && moveCost(f.terrain[i]) !== null && f.owner[i] < 0) { f.owner[i] = c.owner; f.tileCity[i] = c.id; }
      }
    }
  }
  // Where the most different empires meet: the shots frame it, so the borders between neighbours are the subject.
  let meet = fromOffset(Math.floor(width / 2), Math.floor(height / 2));
  let most = 0;
  for (let i = 0; i < f.owner.length; i++) {
    if (f.owner[i] < 0) continue;
    const here = fromOffset(i % width, Math.floor(i / width));
    const seen = new Set<number>();
    for (const h of within(here, 4)) { const j = idxOf(f, h); if (j >= 0 && f.owner[j] >= 0) seen.add(f.owner[j]); }
    const score = seen.size * 100 - distance(here, fromOffset(Math.floor(width / 2), Math.floor(height / 2)));
    if (score > most) { most = score; meet = here; }
  }
  const mine = f.cities.find((c) => c.owner === 0 && c.isCapital)!;
  spawnUnit(f, 0, 'Scout', fromOffset(Math.min(width - 2, mine.pos.q + 3), 8), false);
  refreshVision(f);
  // Seat 0 has explored the whole map and knows every border and town as they stand (live through fog).
  f.players[0].explored.fill(1);
  f.knownOwner['0'] = Array.from(f.owner);
  f.knownCities = { ...(f.knownCities ?? {}), '0': Object.fromEntries(f.cities.map((c) => [c.id, structuredClone(c)])) };
  return { fixture: f, home: mine.pos, meet, center: fromOffset(Math.floor(width / 2), Math.floor(height / 2)), capitals: capitals.map((c) => c.pos) };
}
