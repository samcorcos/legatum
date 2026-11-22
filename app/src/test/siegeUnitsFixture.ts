// Injected visual fixture only; not imported by the production app. Your archer and your spearman each have Siege
// (the WASM engine), a spearman without it beside them, and a walled rival city a few tiles east. One unit per era band:
// a Renaissance Arquebusier and AssaultTrooper with Siege (they wear a bombard and ladders), and an Industrial Rifleman and
// Field Cannon with none (Siege is built in from the Industrial era, so they wear no gear).
import { fromOffset, hexEq, neighbors, tileIndex } from '../hex';
import type { Hex, TechId, UnitKind } from '../types';
import { FixtureClient } from './fixtureEngine';
import { createState, makeStandardSetup, spawnUnit } from './fixtureEngine';
import { refreshVision } from './fixtureEngine';
import { useGame } from '../store';
import { coreFixture } from './coreFixture';

export function siegeUnitsFixture(tier: 'Mathematics' | 'Machinery' | 'Metallurgy' = 'Mathematics') {
  const setup = makeStandardSetup('99125', 'India', 'Athens', 'Mara', 'Pericles');
  setup.width = 26; setup.height = 14;
  setup.players = setup.players.slice(0, 2).map((p, i) => ({ ...p, team: i, is_human: i === 0, faction: (['India', 'Athens'] as const)[i] }));
  const s = createState(setup);
  s.units = []; s.cities = []; s.nextUnitId = 0; s.nextCityId = 0;
  s.terrain.fill('Ocean'); s.ruin.fill(0); s.wonder = undefined; s.owner.fill(-1); s.tileCity.fill(-1);
  s.resource?.fill(null);
  for (let row = 1; row <= 12; row++) for (let col = 1; col <= 24; col++) s.terrain[row * s.width + col] = (col * 7 + row * 13) % 11 === 3 ? 'Hills' : 'Plains';
  for (const p of s.players) { p.explored.fill(1); p.gold = 500; }
  spawnUnit(s, 0, 'Settler', fromOffset(6, 6), false);
  spawnUnit(s, 1, 'Settler', fromOffset(19, 6), false);
  refreshVision(s);
  const client = new FixtureClient();
  client.load(JSON.stringify(s));
  client.endTurn(Object.fromEntries([0, 1].map((p) => [p, s.units.filter((u) => u.owner === p).map((u) => ({ type: 'FoundCity' as const, unit: u.id }))])));
  const f = client.debugState();
  f.turn = 60;
  for (const c of f.cities) c.population = 4;
  f.cities[0].name = 'Kyoto';
  f.cities[1].name = 'Athens';
  f.cities[1].buildings.push('Walls');
  for (const r of f.diplomacy!.relations) r.state = 'War';
  const techs: TechId[] = ['BronzeWorking', 'Masonry', 'Mathematics', ...(tier === 'Mathematics' ? [] : ['Machinery' as const]), ...(tier === 'Metallurgy' ? ['Metallurgy' as const] : [])];
  for (const t of techs) if (!f.players[0].techs.includes(t)) f.players[0].techs.push(t);
  const at = (col: number, row: number) => fromOffset(col, row);
  const free = (h: Hex) => f.terrain[tileIndex(h, f.width, f.height)] !== 'Ocean' && !f.units.some((u) => hexEq(u.pos, h)) && !f.cities.some((c) => hexEq(c.pos, h));
  const put = (owner: number, kind: UnitKind, h: Hex) => { if (!free(h)) throw new Error(`occupied ${h.q},${h.r}`); return spawnUnit(f, owner, kind, h, false); };
  const archerAt = at(14, 4), spearAt = at(18, 6), bareAt = at(15, 6);
  const archer = put(0, 'Ranger', archerAt);
  archer.siege = true;
  const spear = put(0, 'Soldier', spearAt);
  spear.siege = true;
  const bare = put(0, 'Soldier', bareAt);
  put(0, 'Scout', neighbors(at(17, 6)).filter(free)[0]);
  const arquebusier = put(0, 'Arquebusier', at(14, 7));
  arquebusier.siege = true;
  const trooper = put(0, 'AssaultTrooper', at(16, 8));
  trooper.siege = true;
  const rifleman = put(0, 'Rifleman', neighbors(at(19, 6)).filter(free).sort((a, b) => b.r - a.r)[0]);
  const cannon = put(0, 'FieldCannon', at(16, 9));
  refreshVision(f);
  f.players[0].explored.fill(1);
  client.load(coreFixture(f));
  return { client, centre: at(15, 7), archer: archer.id, spear: spear.id, bare: bare.id, arquebusier: arquebusier.id, trooper: trooper.id, rifleman: rifleman.id, cannon: cannon.id, cityAt: at(19, 6), east: at(21, 6) };
}

/** Puts the fixture on screen with your spearman selected. */
export function showSiegeUnitsFixture(tier: 'Mathematics' | 'Machinery' | 'Metallurgy' = 'Mathematics') {
  const fx = siegeUnitsFixture(tier);
  const view = fx.client.view(0);
  useGame.setState({ client: fx.client, humans: [0], views: [view], screen: 'game', booted: true, seat: 0,
    orders: [[]], errors: [[]], ready: [false], selection: { kind: 'unit', id: fx.spear }, mode: 'normal',
    reportOpen: false, techOpen: false, vpExpanded: false, busy: null, error: null, anim: null, net: null });
  return fx;
}
