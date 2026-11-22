// Movement reach for the planning overlay. Orders are planned for the coming turn, when the unit
// has its full movement again, so reach starts from max_moves: the view's moves_left is what was
// left after last turn's resolution, and a unit that spent everything would otherwise show none.
import type { PlayerView, UnitView } from '../types';
import { atWar, borderSteps, canEnter, neutralBlocker, relationWith } from '../diplomacy';
import { hexAt, tileIndex } from '../hex';
import { tileMoveCost } from '../data';
import { roadMoves } from '../path';
import { neighborIndex } from './grid';

/** Tile indices the unit can enter this turn (its own tile excluded). */
export function reachTiles(view: PlayerView, u: Pick<UnitView, 'pos' | 'max_moves' | 'kind'>): Set<number> {
  const W = view.width;
  const H = view.height;
  const start = tileIndex(u.pos, W, H);
  const out = new Set<number>();
  if (start < 0 || u.max_moves <= 0) return out;
  const canStep = borderSteps(view, u.kind, u.pos);
  // The road bonus (1, 2 with Railways) for a unit that starts its turn on a road (core `reset_turn`).
  const best = new Map<number, number>([[start, u.max_moves + (view.tiles[start].road ? roadMoves(view) : 0)]]);
  const q: number[] = [start];
  while (q.length) {
    const i = q.shift()!;
    const mv = best.get(i)!;
    if (mv <= 0) continue;
    for (let e = 0; e < 6; e++) {
      const nb = neighborIndex(i, e, W, H);
      if (nb < 0 || !canStep(hexAt(i, W), hexAt(nb, W))) continue;
      const t = view.tiles[nb];
      if (!t.explored || neutralBlocker(view, hexAt(nb, W))) continue;
      const cost = tileMoveCost(t.terrain, t.road);
      if (cost === null) continue;
      // The core's rule: one step is always allowed at full moves; otherwise moves >= cost.
      if (!(mv === u.max_moves || mv >= cost)) continue;
      const left = Math.max(0, mv - cost);
      if ((best.get(nb) ?? -1) >= left) continue;
      const city = (view.cities ?? []).find((c) => c.pos.q === hexAt(nb, W).q && c.pos.r === hexAt(nb, W).r);
      const cityTeam = city && view.players.find((p) => p.id === city.owner)?.team;
      const foreignCity = city !== undefined && cityTeam !== view.team;
      if (foreignCity && !atWar(view, city.owner) && relationWith(view, cityTeam ?? -1)?.state !== 'Alliance') continue;
      best.set(nb, left);
      const mayStop = !foreignCity || canEnter(view, u.kind, t.owner, true);
      const alliedUnit = (view.units ?? []).some((other) => other.pos.q === hexAt(nb, W).q && other.pos.r === hexAt(nb, W).r
        && relationWith(view, view.players.find((p) => p.id === other.owner)?.team ?? -1)?.state === 'Alliance');
      if (mayStop && !alliedUnit) out.add(nb);
      q.push(nb);
    }
  }
  out.delete(start);
  return out;
}
