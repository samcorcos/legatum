import { FACTION_GOAL, factionGoalVp } from './data';
import { pendingAction, pendingRetire } from './goalActions';
import type { Order, PlayerView } from './types';

export type GoalDecisionKind = 'Swarm' | 'Sakoku' | 'Broadcast' | 'CallTheWild' | 'Retire';

/** The action's own name (the primary's title) and what it scores (its line): the VP is kept for good for a one-time act, and counts toward a goal that takes several. */
export function goalDecisionText(kind: GoalDecisionKind, vp: number): { title: string; line: string } {
  const title = kind === 'Swarm' ? 'Kindle the New Fire' : kind === 'Retire' ? 'Retire a veteran' : kind === 'Sakoku' ? 'Close the country' : kind === 'CallTheWild' ? 'Raise the Warbands' : 'Share the Learning';
  return { title, line: kind === 'Swarm' || kind === 'Broadcast' ? `+${vp} VP, kept for good` : `Counts toward +${vp} VP` };
}

/** Timing calls wait in the sidebar until they pay, the windows the Hard AI uses (`goal_actions.rs`). Raise the Warbands: a camp's tile yields nothing, rivals come to clear it, and a city founded later breaks the goal, so it is offered once its VP would bring the team within 1 VP of victory (`wild_window`, `WILD_PLAYBOOK_SLACK`). Share the Learning hands every rival free technologies, so it is offered only once its VP would reach the victory target (`rite_pays`). */
export const WILD_OFFER_SLACK = 1;
export const BROADCAST_OFFER_SLACK = 0;
function closeToVictory(view: PlayerView, vp: number, slack: number): boolean {
  const s = view.standings.find((t) => t.team === view.team);
  return !!s && s.vp + vp + slack >= s.target;
}

/** The first available goal action is offered once; the sidebar remains its permanent home. */
export function readyGoalAction(view: PlayerView, orders: readonly Order[], seen: Record<number, boolean>): { kind: GoalDecisionKind; name: string; vp: number } | null {
  if (view.finished || seen[view.me]) return null;
  const me = view.players.find((p) => p.id === view.me);
  if (!me?.alive || view.faction_goal?.done) return null;
  const a = view.actions;
  const kind: GoalDecisionKind | null = a?.swarm && !a.swarm.blocker ? 'Swarm'
    : a?.sakoku && !a.sakoku.closed && !a.sakoku.blocker ? 'Sakoku'
      : a?.broadcast && !a.broadcast.blocker && closeToVictory(view, factionGoalVp(me.faction), BROADCAST_OFFER_SLACK) ? 'Broadcast'
        : a?.wild && !a.wild.blocker && closeToVictory(view, factionGoalVp(me.faction), WILD_OFFER_SLACK) ? 'CallTheWild'
          : a?.retire?.veterans.some((v) => !v.blocker && !pendingRetire(orders, v.unit)) ? 'Retire' : null;
  if (!kind || (kind !== 'Retire' && pendingAction(orders, kind))) return null;
  return { kind, name: kind === 'Sakoku' ? 'Sakoku' : FACTION_GOAL[me.faction].name, vp: factionGoalVp(me.faction) };
}

/** Leaving the first ready turn also dismisses the offer; the sidebar action stays available. */
export function seenGoalsAfterTurn(previous: readonly PlayerView[], seen: Record<number, boolean>): Record<number, boolean> {
  const next = { ...seen };
  for (const view of previous) if (readyGoalAction(view, [], next)) next[view.me] = true;
  return next;
}
