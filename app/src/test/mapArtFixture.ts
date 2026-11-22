// A busy map for art review (screens and shots, never imported by the app): eight empires of every
// faction, an Indian capital and nearby cities, a parade of every seat's counters (military and civilian),
// resources and ruins from the generated map, a guarded barbarian camp with raiders and an empty
// camp, and a Settler for the Settle lens. The viewer (seat 0) has explored the whole map.
import { distance, hexEq, tileIndex, within } from '../hex';
import type { Faction, Hex } from '../types';
import { FixtureClient } from './fixtureEngine';
import { createState, makeStandardSetup, spawnUnit } from './fixtureEngine';
import { idxOf, refreshVision } from './fixtureEngine';
import { moveCost } from '../data';
import type { MState } from './fixtureState';
type MCamp = NonNullable<MState['barbarians']>['camps'][number];

const FACTION_ORDER: Faction[] = ['India', 'Byzantium', 'Sparta', 'Switzerland', 'Athens', 'Germany', 'Celts', 'Mongols'];

export function mapArtFixture(seed = '531204', size?: { width: number; height: number }) {
  const setup = makeStandardSetup(seed, 'India', 'Byzantium', 'Aiko', 'Rival');
  if (size) Object.assign(setup, size);
  setup.barbarians = true;
  // Every founder receives an explicit order in this staged scene.
  setup.players = setup.players.map((p, i) => ({ ...p, faction: FACTION_ORDER[i], team: i, is_human: true }));
  const s = createState(setup);
  const land = (h: Hex) => { const i = tileIndex(h, s.width, s.height); return i >= 0 && moveCost(s.terrain[i]) !== null && !s.ruin[i]; };
  let live = s;
  const free = (h: Hex) => land(h) && !live.units.some((u) => hexEq(u.pos, h)) && !live.cities.some((c) => hexEq(c.pos, h));

  // Every empire founds its capital where it starts (the generated start positions).
  const founders = s.units.filter((u) => u.kind === 'Settler');
  const client = new FixtureClient();
  client.load(JSON.stringify(s));
  client.endTurn(Object.fromEntries(founders.map((u) => [u.owner, [{ type: 'FoundCity' as const, unit: u.id }]])));
  const f = client.debugState();
  live = f;
  f.turn = 12;
  const cap = f.cities.find((c) => c.owner === 0)!;
  const spots = (center: Hex, from: number, to: number) => within(center, to).filter((h) => distance(center, h) >= from && free(h) && land(h)).sort((a, b) => distance(center, a) - distance(center, b));

  // Seat 0's neighbours: a second city of seats 2-5 close by, so several borders meet in one view.
  const extra: Hex[] = [];
  for (const [seat, from] of [[0, 4], [0, 5], [2, 6], [3, 6], [4, 7], [5, 7]] as const) {
    const pos = spots(cap.pos, from, from + 3).find((h) => [...extra, ...f.cities.map((c) => c.pos)].every((o) => distance(o, h) >= 4));
    if (!pos) continue;
    extra.push(pos);
    spawnUnit(f, seat, 'Settler', pos, false);
  }
  client.load(JSON.stringify(f));
  client.endTurn(Object.fromEntries(extra.map((pos) => { const u = client.debugState().units.find((x) => hexEq(x.pos, pos) && x.kind === 'Settler')!; return [u.owner, [{ type: 'FoundCity' as const, unit: u.id }]]; })));
  const g = client.debugState();
  live = g;
  g.turn = 12;
  for (const c of g.cities) { c.population = c.isCapital ? 7 : 4; c.hp = c.hp; for (const h of within(c.pos, 2)) { const i = idxOf(g, h); if (i >= 0 && g.owner[i] < 0 && land(h)) { g.owner[i] = c.owner; g.tileCity[i] = c.id; } } }

  // The parade: every seat's soldier (military counter) and settler (civilian chit) in two rows near the capital.
  const home = g.cities.find((c) => c.owner === 0)!;
  const row = spots(home.pos, 2, 7).filter((h) => free(h));
  const parade: Hex[] = [];
  for (let seat = 0; seat < 8; seat++) {
    const h = row.find((x) => free(x) && !parade.some((p) => distance(p, x) < 1));
    if (!h) break;
    parade.push(h);
    spawnUnit(g, seat, 'Soldier', h, false);
  }
  const civ = spots(home.pos, 3, 8).filter((h) => free(h));
  for (let seat = 0; seat < 8; seat++) {
    const h = civ.find((x) => free(x));
    if (!h) break;
    spawnUnit(g, seat, 'Scout', h, false);
  }
  // A Settler for the Settle lens.
  const settlerAt = spots(home.pos, 3, 9).filter((h) => g.cities.every((c) => distance(c.pos, h) >= 4)).find(free);
  const settler = settlerAt ? spawnUnit(g, 0, 'Settler', settlerAt, false) : null;

  // Barbarians: a guarded camp (seen by a scout), two raiders beside it, and an empty camp far off (remembered).
  const campAt = spots(home.pos, 7, 10).find((h) => g.cities.every((c) => distance(c.pos, h) >= 5) && !g.resource?.[idxOf(g, h)]) ?? null;
  const camps: MCamp[] = [];
  if (campAt) {
    const guard = spawnUnit(g, 255, 'Soldier', campAt, false);
    const near = within(campAt, 1).filter((h) => !hexEq(h, campAt) && free(h));
    const raiders = near.slice(0, 2).map((h, k) => spawnUnit(g, 255, k ? 'Ranger' : 'Soldier', h, false).id);
    const watch = near.find(free);
    if (watch) spawnUnit(g, 0, 'Scout', watch, false);
    camps.push({ id: 0, pos: campAt, founded_turn: 6, guard: guard.id, scout: null, raiders, scout_report: null, raid: null, next_raid_turn: 99, next_spawn_turn: 99 });
  }
  const empty = spots(home.pos, 11, 15).find((h) => g.cities.every((c) => distance(c.pos, h) >= 6));
  if (empty) camps.push({ id: 1, pos: empty, founded_turn: 6, guard: null, scout: null, raiders: [], scout_report: null, raid: null, next_raid_turn: 99, next_spawn_turn: 99 });
  g.barbarians = { camps, next_camp_id: camps.length, last_camp_turn: 6, known: { '0': camps.map((c) => ({ ...c.pos })) } };

  for (let i = 1; i < g.players.length; i++) {
    g.players[i].isHuman = false;
    g.setup.players[i].is_human = false;
  }

  // The viewer has charted everything and remembers every border and city.
  refreshVision(g);
  g.players[0].explored.fill(1);
  g.knownOwner['0'] = Array.from(g.owner);
  g.knownCities = { ...(g.knownCities ?? {}), '0': Object.fromEntries(g.cities.map((c) => [c.id, structuredClone(c)])) };
  g.knownRuins = { ...(g.knownRuins ?? {}), '0': Array.from(g.ruin) };
  return { fixture: g, capital: home.id, settler: settler?.id ?? null, settlerAt: settlerAt ?? null, camp: campAt, parade, extra };
}
