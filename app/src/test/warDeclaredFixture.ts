import { fromOffset, within } from '../hex';
import { coreFixture } from './coreFixture';
import { createState, makeStandardSetup, spawnUnit } from './fixtureEngine';
import { idxOf, refreshVision } from './fixtureEngine';

/**
 * The real AI declares war on the human (playtest 6: Sparta's war went unnoticed). An
 * Sparta army of five Pikemen stands beside our lone city with a Scout in sight of it, the
 * scripted AI's "viable campaign" (crates/ai/tests/diplomacy.rs). Nothing is injected: ending
 * turn 1 resolves the AI's own DeclareWar. `teammate` adds a second human on our team (online co-op).
 */
function warDeclaredState({ teammate = false } = {}) {
  const setup = makeStandardSetup('700042', 'Mongols', 'Celts', 'Ada', 'Bo');
  setup.width = 28; setup.height = 20;
  const humans = teammate ? 2 : 1;
  const ai = (faction: 'Sparta' | 'Athens', name: string, team: number) => ({ name, faction, team, is_human: false });
  setup.players = [
    { ...setup.players[0], team: 0, is_human: true },
    ...(teammate ? [{ ...setup.players[1], team: 0, is_human: true }] : []),
    ai('Sparta', 'AI 2', 1),
    ai('Athens', 'AI 3', 2),
  ];
  const s = createState(setup);
  s.terrain.fill('Grassland'); s.owner.fill(-1); s.tileCity.fill(-1); s.ruin.fill(0); s.wonder = undefined;
  s.units = []; s.nextUnitId = 0;
  const oath = humans, far = humans + 1;
  const sites: [number, number, number, string][] = [
    [0, 11, 6, 'Waystation'], [oath, 4, 6, 'Oathhold'], [far, 24, 15, 'Listening Deep'],
    ...(teammate ? [[1, 13, 12, 'Brasslock'] as [number, number, number, string]] : []),
  ];
  s.cities = sites.map(([owner, c, r, name], id) => ({ id, owner, founder: owner, name, pos: fromOffset(c, r), population: 2,
    food: 0, prod: 0, producing: null, bought: null, buildings: [], hp: 200, grownTiles: [], tilesClaimed: 0,
    isCapital: true, foundedTurn: 1, resistance: 0 }));
  s.nextCityId = s.cities.length;
  for (const c of s.cities) for (const h of within(c.pos, 1)) {
    const i = idxOf(s, h); s.owner[i] = c.owner; s.tileCity[i] = c.id;
  }
  for (const p of s.players) p.gold = 1000;
  for (const [c, r] of [[5, 6], [6, 6], [5, 7], [6, 7], [7, 7]]) spawnUnit(s, oath, 'Pikeman', fromOffset(c, r), false);
  spawnUnit(s, oath, 'Scout', fromOffset(9, 6), false);
  // Our Scouts watch their army and their city, so the declaration can say where they are.
  spawnUnit(s, 0, 'Scout', fromOffset(8, 5), false);
  spawnUnit(s, 0, 'Scout', fromOffset(5, 4), false);
  if (teammate) spawnUnit(s, 1, 'Scout', fromOffset(14, 13), false);
  refreshVision(s);
  return s;
}

export function warDeclaredCoreSave(options: { teammate?: boolean } = {}) {
  return coreFixture(warDeclaredState(options));
}
