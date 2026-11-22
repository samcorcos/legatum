// A bare map for combat-view tests: every tile explored, visible plains, your units and enemy
// units placed by hand, everyone's strengths from the real unit tables.
import { FixtureClient } from './fixtureEngine';
import { unitStatsFor } from '../data';
import type { CityView, Faction, Hex, PlayerView, UnitKind, UnitView } from '../types';

/** Player 0 is you (a teammate is player 1); players 2 and up are other teams, at war with you. */
export async function arenaView(mine: Faction = 'Sparta'): Promise<PlayerView> {
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('7', mine, 'Byzantium', 'Me', 'Them'));
  const v = JSON.parse(JSON.stringify(client.view(0))) as PlayerView;
  v.tiles.forEach((t) => { t.explored = true; t.visible = true; t.terrain = 'Plains'; t.owner = null; t.resource = null; t.road = undefined; });
  v.units = [];
  v.cities = [];
  // Legacy diplomacy: every other team is at war with you.
  v.diplomacy = { ...v.diplomacy, legacy: true } as PlayerView['diplomacy'];
  return v;
}

let nextId = 100;
export function addUnit(v: PlayerView, owner: number, kind: UnitKind, pos: Hex, extra: Partial<UnitView> = {}): UnitView {
  const faction = v.players.find((p) => p.id === owner)!.faction;
  const s = unitStatsFor(faction, kind);
  const u: UnitView = {
    id: nextId++, owner, kind, pos, hp: 100, moves_left: s.moves, max_moves: s.moves, goto: null, fortify: 0,
    strength: s.strength, ranged: s.ranged, range: s.range, military: s.military, upgrade_cost: null, ...extra,
  };
  v.units.push(u);
  return u;
}

export function addCity(v: PlayerView, owner: number, name: string, pos: Hex, walls = true): CityView {
  const c: CityView = {
    id: nextId++, owner, name, pos, population: 3, hp: 200, max_hp: 200, strength: 10 + 6 + (walls ? 10 : 0),
    is_capital: false, walls, great_works: [], visible: true, detail: null,
  };
  v.cities.push(c);
  return c;
}
