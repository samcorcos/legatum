import { fromOffset, within } from '../hex';
import { coreFixture } from './coreFixture';
import { createState, makeStandardSetup, meetAll, spawnUnit } from './fixtureEngine';
import { idxOf, refreshVision } from './fixtureEngine';

/** Seeded two-empire scenario. No proposals/orders are injected: the real AI negotiates.
 * `turn` lets a scenario start later in the game (the AI waits to know a team before allying).
 * `met` puts our Scout in sight of the AI's city and an AI Scout near ours, so each side's
 * location and nearby units are known. The empires are in first contact unless `strangers`
 * (then an Alliance can't be offered: "You haven't met the … yet"). */
export function diplomacyCoreSave(turn = 1, { met = false, strangers = false } = {}) {
  return coreFixture(diplomacyState(turn, { met, strangers }));
}

/** The same scenario for WASM regression tests that vary the recipient's budget. */
export function diplomacyState(turn = 1, { met = false, strangers = false } = {}) {
  const setup = makeStandardSetup('700040', 'Byzantium', 'Switzerland', 'Ada', 'Swiss envoy');
  setup.width = 28; setup.height = 18;
  setup.players = setup.players.slice(0, 2).map((p, id) => ({ ...p, team: id, is_human: id === 0 }));
  const s = createState(setup);
  s.terrain.fill('Grassland'); s.owner.fill(-1); s.tileCity.fill(-1); s.ruin.fill(0); s.wonder = undefined;
  s.units = []; s.nextUnitId = 0;
  s.cities = [0, 1].map((owner) => ({ id: owner, owner, founder: owner, name: owner ? 'Harmonia' : 'Astra',
    pos: fromOffset(5 + owner * 16, 8), population: 2, food: 0, prod: 0, producing: null, bought: null,
    buildings: [], hp: owner ? 60 : 200, grownTiles: [], tilesClaimed: 0, isCapital: true, foundedTurn: 1, resistance: 0 }));
  s.nextCityId = 2;
  for (const c of s.cities) for (const h of within(c.pos, 1)) {
    const i = idxOf(s, h); s.owner[i] = c.owner; s.tileCity[i] = c.id;
  }
  for (const p of s.players) p.gold = 1000;
  for (const pos of [fromOffset(5, 8), fromOffset(4, 8), fromOffset(5, 9)]) spawnUnit(s, 0, 'Soldier', pos, false);
  spawnUnit(s, 1, 'Soldier', s.cities[1].pos, false);
  if (met) { spawnUnit(s, 0, 'Scout', fromOffset(19, 8), false); spawnUnit(s, 1, 'Scout', fromOffset(7, 7), false); }
  refreshVision(s);
  if (strangers) s.diplomacy!.contacts = []; else meetAll(s);
  s.turn = turn;
  return s;
}
