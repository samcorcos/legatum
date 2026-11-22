// The p881 horseman, exactly: the user's Sparta Horseman at turns 29 and 30 of his 8.8.1 game,
// as player 0 saw them (positions, terrain, fog, walls, relations), replayed from the saved tape
// with every hash matching. situation.json is trimmed from the core's PlayerView; the rest of the
// view is boilerplate from a core save. The forecast is what the core said at turn 29.
import situation from './p881/situation.json' with { type: 'json' };
import { arenaView } from './arenaView';
import type { BattleForecast, CityView, Hex, PlayerView, Terrain, UnitView } from '../types';

type Raw = typeof situation;
type Turn = Raw['t29'];

export const HORSEMAN = 28;
export const ARCHER = 15;
export const archerTile: Hex = { q: 19, r: 13 };
/** What the user ordered at turn 30: a retreat toward Essen whose first step went sideways. */
export const playerRetreat: Hex[] = [{ q: 18, r: 13 }, { q: 17, r: 13 }, { q: 17, r: 12 }, { q: 17, r: 11 }];
export const retreatTo: Hex = { q: 17, r: 11 };

/** [explored, visible, terrain, owner (-1 none), road] per tile, row-major. */
export type RawTile = [number, number, string, number, number];

export async function p881View(turn: 29 | 30): Promise<PlayerView> {
  const s: Turn = turn === 29 ? situation.t29 : situation.t30;
  const v = await arenaView('Sparta');
  v.turn = s.turn;
  v.width = s.width;
  v.height = s.height;
  v.me = s.me;
  v.team = s.team;
  v.players = s.players.map((p, i) => ({ ...v.players[i], id: p.id, name: p.name, faction: p.faction as never, team: p.team }));
  v.diplomacy = { ...v.diplomacy, legacy: false, relations: s.relations as never };
  v.tiles = (s.tiles as RawTile[]).map(([explored, visible, terrain, owner, road]) => ({
    explored: !!explored, visible: !!visible, terrain: (terrain || null) as Terrain | null, ruin: false, owner: owner < 0 ? null : owner,
    ...(road ? { road } : {}),
  }));
  v.units = s.units.map((u) => ({ ...u, goto: null, upgrade_cost: null }) as UnitView);
  v.cities = s.cities.map((c) => ({ ...c, detail: null }) as CityView);
  return v;
}

export const forecastT29 = situation.forecastT29 as BattleForecast;
