// Where known enemies can catch a unit that is walking away (the p881 horseman, 2026-10-11).
//
// Every unit takes one step per movement round whatever its moves (core `rules/movement.rs`), and a
// melee unit that is ordered to attack follows its target. So an enemy with `m` moves that starts
// `k` steps from a tile next to my unit's position can strike in round `k + 1` (needs `k + 1 <= m`),
// and a unit that steps away is hit where it stood, without fortification (G21, "caught leaving").
// My unit stands on its j-th route tile at the start of round j + 1, so that tile is dangerous when
// some enemy can be next to it by then. Only enemies in the view count (fog is never read), and
// ranged shots and city strikes land before anyone moves, so no route can dodge them.
import type { Hex, PlayerView, UnitView } from './types';
import { neighbors, tileIndex } from './hex';
import { isThreat } from './diplomacy';
import { tileMoveCost } from './data';

/** Steps from `unit` to every explored, passable tile within `limit` steps (breadth-first; terrain cost is ignored: a round is one step). */
function stepField(view: PlayerView, unit: UnitView, limit: number): Map<number, number> {
  const steps = new Map<number, number>();
  const start = tileIndex(unit.pos, view.width, view.height);
  if (start < 0) return steps;
  steps.set(start, 0);
  let frontier = [unit.pos];
  for (let step = 1; step <= limit; step++) {
    const next: Hex[] = [];
    for (const from of frontier) {
      for (const h of neighbors(from)) {
        const i = tileIndex(h, view.width, view.height);
        const tile = view.tiles[i];
        if (i < 0 || steps.has(i) || !tile?.explored || tileMoveCost(tile.terrain, tile.road) === null) continue;
        steps.set(i, step);
        next.push(h);
      }
    }
    frontier = next;
  }
  return steps;
}

/** The viewer's known military threats (enemies at war; barbarians never count for the Celts), with the tiles they can reach by step count. */
export interface EnemyReach {
  /** True when no known enemy can strike a unit standing on `hex` at the start of round `round` (1 is before anyone has moved). */
  threatened(hex: Hex, round: number, final?: boolean): boolean;
  /** Whether any known enemy could ever matter (skips work for a quiet map). */
  readonly any: boolean;
}

export function enemyReach(view: PlayerView): EnemyReach {
  const enemies = view.units.filter((u) => u.military && u.hp > 0 && u.max_moves > 0 && isThreat(view, u.owner));
  const fields = enemies.map((u) => ({ moves: u.max_moves, steps: stepField(view, u, Math.max(0, u.max_moves - 1)) }));
  return {
    any: enemies.length > 0,
    threatened(hex, round, final = false) {
      return fields.some(({ moves, steps }) => {
        // The enemy acts in rounds 1..moves; striking a unit that arrives at `round` means acting in round + 1 or later.
        if (round + 1 > moves) return false;
        // A unit that has stopped for the turn is attackable in every later round, so it needs only `moves - 1` steps.
        const within = final ? moves - 1 : round;
        return neighbors(hex).some((n) => {
          const s = steps.get(tileIndex(n, view.width, view.height));
          return s !== undefined && s <= within;
        });
      });
    },
  };
}
