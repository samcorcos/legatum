// The Heir's public birth for the screen tour (the real engine loads these through `coreFixture`):
//  'rival': you are Sparta (player 0); Byzantium (player 1, an AI) has a city far away in the fog, producing its Heir with the Production
//           already in the bank, so ending the turn trains him; the rival card, the fog halo, the minimap dot and the tile card follow.
//  'own':   you are Byzantium with a city producing the Heir; ending the turn trains him (the Heir card, with the city named).
// Injected fixture only; not imported by the app. No browser or store imports: Playwright specs load it in Node.
import { fromOffset, within } from '../hex';
import { idxOf, refreshVision } from './fixtureEngine';
import { createState, makeStandardSetup, meetAll, spawnUnit } from './fixtureEngine';
import type { MCity, MState } from './fixtureState';
import type { Faction } from '../types';

export interface HeirState { state: MState; turn: number; city: number; byzantineCity: number; rivalCity: number }

function city(s: MState, owner: number, col: number, row: number, name: string, extra: Partial<MCity> = {}): MCity {
  const c: MCity = {
    id: ++s.nextCityId, owner, founder: owner, name, pos: fromOffset(col, row), population: 3, food: 0, prod: 0, producing: null, bought: null, buildings: [],
    hp: 200, grownTiles: [], tilesClaimed: 0, isCapital: s.cities.every((o) => o.owner !== owner), foundedTurn: 1, resistance: 0, ...extra,
  };
  s.cities.push(c);
  for (const h of within(c.pos, 1)) { const i = idxOf(s, h); s.owner[i] = owner; s.tileCity[i] = c.id; }
  return c;
}

export function heirState(viewer: 'rival' | 'own'): HeirState {
  const factions: Faction[] = viewer === 'own' ? ['Byzantium', 'Sparta', 'India'] : ['Sparta', 'Byzantium', 'India'];
  const setup = makeStandardSetup('99125', factions[0], factions[1], 'AI 1', 'AI 2');
  setup.width = 30; setup.height = 16;
  setup.players = setup.players.slice(0, 3).map((p, i) => ({ ...p, team: i, is_human: i === 0, faction: factions[i], name: `AI ${i + 1}` }));
  const s = createState(setup);
  s.turn = 12;
  s.units = []; s.cities = []; s.nextUnitId = 0; s.nextCityId = 0;
  s.terrain.fill('Ocean'); s.ruin.fill(0); s.wonder = undefined; s.owner.fill(-1); s.tileCity.fill(-1);
  for (let row = 2; row <= 13; row++) for (let col = 1; col <= 28; col++) s.terrain[row * s.width + col] = (col + row) % 5 === 0 ? 'Forest' : (col * row) % 7 === 0 ? 'Grassland' : 'Plains';
  for (const r of s.diplomacy?.relations ?? []) r.state = 'Peace';
  meetAll(s);
  const heir = { kind: 'Unit' as const, unit: 'Heir' as const };
  const mine = city(s, 0, 5, 7, viewer === 'own' ? 'Constantinople' : 'Sparta', viewer === 'own' ? { producing: heir, prod: 1000 } : {});
  const byz = viewer === 'own' ? mine : city(s, 1, 24, 7, 'Constantinople', { producing: heir, prod: 1000 });
  const other = city(s, 2, 14, 12, 'Kyoto');
  if (viewer === 'rival') spawnUnit(s, 0, 'Soldier', fromOffset(6, 7), false);
  // Only your own land is known: the rest stays fog or unexplored.
  for (const p of s.players) p.explored.fill(0);
  refreshVision(s);
  return { state: s, turn: s.turn, city: mine.id, byzantineCity: byz.id, rivalCity: other.id };
}
