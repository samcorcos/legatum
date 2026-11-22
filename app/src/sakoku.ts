// Sakoku stays reserved for Japan. Legacy scenarios can still select its goal rule explicitly.
import { factionGoalVp } from './data';
import type { PlayerPublic } from './types';

export interface SakokuStatus {
  /** Turn ends spent closed. */
  turns: number;
  /** Turn ends the country must stay closed before it holds its VP. */
  target: number;
  /** Turns of the countdown still to run (0 once it is done). */
  left: number;
  /** The VP scores now (the countdown is done). */
  holding: boolean;
}

/** Core `goals::sakoku_target`: turn ends closed before the VP holds. */
export const sakokuTarget = (): number => 30;

/** The empire's public vow, or null while its country is open. */
export function sakokuStatus(p: Pick<PlayerPublic, 'sakoku_turns' | 'faction_goal_done'>): SakokuStatus | null {
  if (p.sakoku_turns === undefined || p.sakoku_turns === null) return null;
  const target = sakokuTarget(), turns = p.sakoku_turns;
  return { turns, target, left: Math.max(0, target - turns), holding: turns >= target && p.faction_goal_done };
}

/** The public countdown as one line: "Closed 12 of 30 turns · holds 2 VP in 18". */
export function sakokuLine(st: SakokuStatus): string {
  const vp = factionGoalVp('India');
  return st.holding ? `Closed ${st.turns} turns · holding ${vp} VP` : `Closed ${st.turns} of ${st.target} turns · holds ${vp} VP in ${st.left}`;
}

/** The short form for a chip: "12/30" or "holding". */
export function sakokuChip(st: SakokuStatus): string {
  return st.holding ? 'holding' : `${st.turns}/${st.target}`;
}
