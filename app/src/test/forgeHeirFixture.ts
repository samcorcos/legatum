// Faction goals increment 4 for the screen tour: a Germany empire with five cities (four have a
// Ruhr Foundry), or a Byzantium empire with its Heir at home, in a rival's hands, or not trained yet.
// Injected visual fixture only; not imported by the app.
import { fromOffset } from '../hex';
import { TECHS } from '../data';
import { FixtureClient } from './fixtureEngine';
import { createState, makeStandardSetup, spawnUnit } from './fixtureEngine';
import { refreshVision } from './fixtureEngine';
import type { Faction } from '../types';

export type HeirState = 'none' | 'own' | 'captured';

/** `viewer`: Germany (player 0 with five cities, the capital without a Ruhr Foundry) or Byzantium (player 0, its Heir as asked). Player 1 is the Byzantium (for the Germany viewer) or a Mongols rival. */
export function forgeHeirFixture(viewer: 'Germany' | 'Byzantium', heir: HeirState = 'none') {
  const factions: Faction[] = viewer === 'Germany' ? ['Germany', 'Byzantium', 'Mongols'] : ['Byzantium', 'Mongols', 'India'];
  const setup = makeStandardSetup('99124', factions[0], factions[1], 'Ada', 'Rival');
  setup.width = 24; setup.height = 12;
  setup.players = setup.players.slice(0, 3).map((p, i) => ({ ...p, team: i, is_human: i === 0, faction: factions[i], name: ['Ada', 'Epoch', 'Hive'][i] }));
  const s = createState(setup);
  s.units = []; s.cities = []; s.nextUnitId = 0; s.nextCityId = 0;
  s.terrain.fill('Ocean'); s.ruin.fill(0); s.wonder = undefined; s.owner.fill(-1); s.tileCity.fill(-1);
  for (let row = 2; row <= 9; row++) for (let col = 1; col <= 22; col++) s.terrain[row * s.width + col] = (col + row) % 5 === 0 ? 'Forest' : (col * row) % 7 === 0 ? 'Grassland' : 'Plains';
  for (const p of s.players) p.explored.fill(1);
  const mine: [number, number][] = viewer === 'Germany' ? [[3, 3], [3, 8], [9, 3], [9, 8], [15, 9]] : [[4, 5], [4, 9]];
  for (const [col, row] of mine) spawnUnit(s, 0, 'Settler', fromOffset(col, row), false);
  spawnUnit(s, 1, 'Settler', fromOffset(16, 5), false);
  spawnUnit(s, 2, 'Settler', fromOffset(20, 9), false);
  refreshVision(s);
  const client = new FixtureClient();
  client.load(JSON.stringify(s));
  client.endTurn(Object.fromEntries([0, 1, 2].map((p) => [p, s.units.filter((u) => u.owner === p).map((u) => ({ type: 'FoundCity' as const, unit: u.id }))])));
  const f = client.debugState();
  for (const r of f.diplomacy!.relations) r.state = 'Peace';
  if (viewer === 'Germany') f.turn = 200;
  for (const c of f.cities) c.population = viewer === 'Germany' && c.owner === 0 ? 6 : 3;
  const own = f.cities.filter((c) => c.owner === 0);
  const capital = own.find((c) => c.isCapital)!;
  for (const c of own) { c.buildings = []; c.producing = null; c.prod = 0; c.productionQueue = []; c.productionBanked = []; }
  if (viewer === 'Germany') for (const c of own.filter((x) => x !== capital)) c.buildings = ['Forge'];
  f.players[0].techs = viewer === 'Germany'
    ? TECHS.slice(0, TECHS.indexOf('Industrialization') + 1)
    : ['Irrigation', 'BronzeWorking'];
  if (viewer === 'Byzantium' && heir !== 'none') {
    // The Heir stands beside the capital; "captured" puts him in the Mongols rival's hands, in view.
    const at = { q: capital.pos.q + 1, r: capital.pos.r };
    const unit = spawnUnit(f, heir === 'own' ? 0 : 1, 'HeirToTheThrone', at, false);
    unit.heir_of = 0;
    // Your Spearman stands beside him, so the tile can offer the capture.
    if (heir === 'captured') for (const r of f.diplomacy!.relations) if (r.a + r.b === 1) r.state = 'War';
    if (heir === 'captured') { spawnUnit(f, 0, 'Spearman', { q: capital.pos.q, r: capital.pos.r + 1 }, false); }
  }
  refreshVision(f);
  const spearman = f.units.find((u) => u.owner === 0 && u.kind === 'Spearman')?.id ?? null;
  const heirUnit = f.units.find((u) => u.kind === 'HeirToTheThrone')?.id ?? null;
  return { fixture: f, capital: capital.id, spearman, heirUnit, forged: own.filter((x) => x !== capital).map((c) => c.id), heirHex: viewer === 'Byzantium' ? { q: capital.pos.q + 1, r: capital.pos.r } : null };
}
