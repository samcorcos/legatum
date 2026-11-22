// Mercenary Companies for the screen tour and the UI review: a Byzantine capital beside two met sellers (Sparta, India) that know
// the early technologies, two hired Companies next to the capital (one with a week left, one on its last turn) beside an ordinary Soldier, and a
// rival Byzantium with a Company of its own in sight. The state is the demo engine's; the tour loads it into the real engine
// (`coreFixture`), which then offers the shop, prices it and refuses rows in its own words.
// Injected fixture only; not imported by the app. No browser or store imports: Playwright specs load it in Node.
import { fromOffset } from '../hex';
import { FixtureClient } from './fixtureEngine';
import { createState, makeStandardSetup, meetAll, spawnUnit } from './fixtureEngine';
import { refreshVision } from './fixtureEngine';
import { TECHS, TECH_ERA } from '../data';
import type { Faction, Hex } from '../types';
import type { MState } from './fixtureState';

export interface CompanyState {
  client: FixtureClient;
  state: MState;
  turn: number;
  city: number;
  /** Your two Companies: the one with a week left, and the one on its last turn. */
  week: number;
  last: number;
  /** The rival Byzantium's Company, in sight, with 19 turns left. */
  rival: number;
  ordinary: number;
  hex: { week: Hex; last: Hex; rival: Hex };
}

/** `gold`: Byzantium's treasury (a low one greys the dearer rows with their reasons). */
export function companyState(gold = 400): CompanyState {
  const factions: Faction[] = ['Byzantium', 'Sparta', 'India', 'Byzantium'];
  const count = factions.length;
  const setup = makeStandardSetup('99124', factions[0], factions[1], 'AI 1', 'AI 2');
  setup.width = 24; setup.height = 14;
  setup.players = factions.map((faction, i) => ({ ...(setup.players[i] ?? setup.players[1]), id: i, team: i, is_human: i === 0, faction, name: `AI ${i + 1}` }));
  const s = createState(setup);
  s.units = []; s.cities = []; s.nextUnitId = 0; s.nextCityId = 0;
  s.terrain.fill('Ocean'); s.ruin.fill(0); s.wonder = undefined; s.owner.fill(-1); s.tileCity.fill(-1);
  for (let row = 2; row <= setup.height - 3; row++) for (let col = 1; col <= setup.width - 2; col++) s.terrain[row * s.width + col] = (col + row) % 5 === 0 ? 'Forest' : (col * row) % 7 === 0 ? 'Grassland' : 'Plains';
  for (const p of s.players) p.explored.fill(1);
  spawnUnit(s, 0, 'Settler', fromOffset(6, 6), false);
  spawnUnit(s, 1, 'Settler', fromOffset(17, 4), false);
  spawnUnit(s, 2, 'Settler', fromOffset(18, 10), false);
  spawnUnit(s, 3, 'Settler', fromOffset(10, 10), false);
  refreshVision(s);
  const client = new FixtureClient();
  client.load(JSON.stringify(s));
  client.endTurn(Object.fromEntries(Array.from({ length: count }, (_, p) => p).map((p) => [p, s.units.filter((u) => u.owner === p).map((u) => ({ type: 'FoundCity' as const, unit: u.id }))])));
  const f = client.debugState();
  meetAll(f);
  for (const r of f.diplomacy!.relations) r.state = 'Peace';
  for (const c of f.cities) { c.population = 3; if (c.owner === 0) { c.buildings = []; c.producing = null; c.prod = 0; c.productionQueue = []; c.productionBanked = []; } }
  f.players[0].gold = gold;
  f.players[0].techs = ['Irrigation', 'BronzeWorking', 'Masonry', 'IronWorking'];
  // The sellers know the first two eras, so the shop lists what their land can field then.
  for (const seller of [1, 2]) f.players[seller].techs = TECHS.filter((t) => TECH_ERA[t] <= 2);
  const capital = f.cities.find((c) => c.owner === 0)!;
  const at = (dq: number, dr: number) => ({ q: capital.pos.q + dq, r: capital.pos.r + dr });
  const week = spawnUnit(f, 0, 'Soldier', at(1, 0), false);
  const last = spawnUnit(f, 0, 'Knight', at(0, 1), false);
  const ordinary = spawnUnit(f, 0, 'Soldier', at(-1, 1), false);
  const rival = spawnUnit(f, 3, 'Soldier', at(2, 0), false);
  Object.assign(week, { contract_until: f.turn + 7, origin: 'Sparta' as const });
  Object.assign(last, { contract_until: f.turn, origin: 'India' as const });
  Object.assign(rival, { contract_until: f.turn + 19, origin: 'Sparta' as const });
  refreshVision(f);
  return { client, state: f, turn: f.turn, city: capital.id, week: week.id, last: last.id, rival: rival.id, ordinary: ordinary.id,
    hex: { week: week.pos, last: last.pos, rival: rival.pos } };
}
