// The hold-to-win rule in one place: a team wins by VP only after holding its VP target at the end
// of N consecutive turns (core `data::VP_HOLD_TURNS`). The turn it reaches the target is the
// first turn held, so the count starts at 1 and the win comes when it reaches N. Every screen that
// words the rule (alarm, guide, game over, tooltips) reads it from here.
import type { PlayerView, TeamId } from './types';

export const HOLD_TURNS = 10;

/** "10 turns", "1 turn". */
export const turnsWord = (n: number) => `${n} turn${n === 1 ? '' : 's'}`;

/** The rule as one sentence, for tooltips and hints. */
export const holdRule = (target: number, needed: number) =>
  `Reach ${target} VP and hold it for ${needed} turns to win. Falling below ${target} VP at any turn end resets the count to zero.`;

/** How a team's hold stands in this view, or null when it holds nothing. */
export function holdOf(view: PlayerView, team: TeamId): { held: number; needed: number; vp: number; target: number } | null {
  const st = view.standings.find((s) => s.team === team);
  const h = st?.victory_hold;
  if (!st || !h || h.turns_held < 1 || h.turns_needed < 1) return null;
  return { held: h.turns_held, needed: h.turns_needed, vp: st.vp, target: st.target };
}
