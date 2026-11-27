// Byzantium's Buy a city for the screen tour and the wasm tests (the real engine loads these through `coreFixture`).
// A small strip of land: Byzantium's Constantinople and Nicaea in the west, Sparta's Pylos touching Nicaea's land, and
// Sparta's capital in the east. Everyone has met and is at Peace.
//  'buyer':  you are Byzantium (player 0) with Gold to spare; Pylos is for sale.
//  'seller': you are Sparta (player 0); Byzantium (player 1) made an offer for Pylos last turn, so it is under offer now.
// Injected fixture only; not imported by the app. No browser or store imports: Playwright specs load it in Node.
import { fromOffset, within } from '../hex';
import { idxOf, refreshVision } from './fixtureEngine';
import { createState, makeStandardSetup, meetAll, spawnUnit } from './fixtureEngine';
import type { MCity, MState } from './fixtureState';
import type { Event, Faction } from '../types';

export interface BuyCityState { state: MState; turn: number; constantinople: number; nicaea: number; pylos: number; sparta: number; byzantium: number; seller: number; events: Event[] }

function city(s: MState, owner: number, col: number, row: number, name: string, extra: Partial<MCity> = {}): MCity {
  const c: MCity = {
    id: ++s.nextCityId, owner, founder: owner, name, pos: fromOffset(col, row), population: 3, food: 0, prod: 0, producing: null, bought: null, buildings: [],
    hp: 200, grownTiles: [], tilesClaimed: 0, isCapital: s.cities.every((o) => o.owner !== owner), foundedTurn: 1, resistance: 0, ...extra,
  };
  // A first city is its empire's original capital, as in a real game (the rivals table reads "Capital fallen" otherwise).
  if (c.isCapital) c.originalCapital = true;
  s.cities.push(c);
  for (const h of within(c.pos, 1)) { const i = idxOf(s, h); s.owner[i] = owner; s.tileCity[i] = c.id; }
  return c;
}

/** Pylos at population 4 with a Granary and one Spartan Spearman beside it: the escrow the 'seller' offer holds. */
export const PYLOS_ESCROW = 600;

export function buyCityState(viewer: 'buyer' | 'seller'): BuyCityState {
  const factions: Faction[] = viewer === 'buyer' ? ['Byzantium', 'Sparta', 'India'] : ['Sparta', 'Byzantium', 'India'];
  const byzantium = factions.indexOf('Byzantium'), sparta = factions.indexOf('Sparta');
  const setup = makeStandardSetup('99126', factions[0], factions[1], 'AI 1', 'AI 2');
  setup.width = 30; setup.height = 16;
  setup.players = setup.players.slice(0, 3).map((p, i) => ({ ...p, team: i, is_human: i === 0, faction: factions[i], name: `AI ${i + 1}` }));
  const s = createState(setup);
  s.turn = 12;
  s.units = []; s.cities = []; s.nextUnitId = 0; s.nextCityId = 0;
  s.terrain.fill('Ocean'); s.ruin.fill(0); s.wonder = undefined; s.owner.fill(-1); s.tileCity.fill(-1);
  for (let row = 2; row <= 13; row++) for (let col = 1; col <= 28; col++) s.terrain[row * s.width + col] = (col + row) % 5 === 0 ? 'Forest' : (col * row) % 7 === 0 ? 'Grassland' : 'Plains';
  for (const r of s.diplomacy?.relations ?? []) r.state = 'Peace';
  meetAll(s);
  const constantinople = city(s, byzantium, 4, 7, 'Constantinople', { population: 6 });
  const nicaea = city(s, byzantium, 8, 7, 'Nicaea');
  const spartaCity = city(s, sparta, 21, 7, 'Sparta', { population: 5 });
  const pylos = city(s, sparta, 11, 7, 'Pylos', { population: 4, buildings: ['Granary'] });
  city(s, 2, 16, 12, 'Kyoto');
  s.players[byzantium].gold = 1500;
  spawnUnit(s, sparta, 'Spearman', fromOffset(12, 6), false);
  spawnUnit(s, byzantium, 'Spearman', fromOffset(5, 7), false);
  const events: Event[] = [];
  if (viewer === 'seller') {
    // Made at the end of turn 11: it ends at the end of turn 14, three planning turns from now.
    s.players[byzantium].cityOffer = { city: pylos.id, seller: sparta, escrow: PYLOS_ESCROW, base: 480, started: 11, ends_turn: 14 };
    s.players[byzantium].gold -= PYLOS_ESCROW;
    events.push({ type: 'CityOfferMade', city: pylos.id, buyer: byzantium, seller: sparta, escrow: PYLOS_ESCROW, ends_turn: 14 });
    // A Spartan Scout in sight of Constantinople, so the seller knows the Byzantine capital.
    spawnUnit(s, sparta, 'Scout', fromOffset(4, 5), false);
  }
  // The whole strip is known to everyone, so every banner shows.
  for (const p of s.players) p.explored.fill(1);
  refreshVision(s);
  return { state: s, turn: s.turn, constantinople: constantinople.id, nicaea: nicaea.id, pylos: pylos.id, sparta: spartaCity.id, byzantium, seller: sparta, events };
}
