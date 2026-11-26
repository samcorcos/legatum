import { FACTION_GOAL, factionGoalVp } from './data';
import { pendingAction, pendingRetire } from './goalActions';
import type { Order, PlayerView, UnitId } from './types';
import { COLONIA_LEVEL, COLONIA_REACH } from './goalCities';

/** Rome's first veteran that can found the Colonia Victrix and has no founding order yet, or null. */
export function coloniaFounder(view: PlayerView, orders: readonly Order[]): UnitId | null {
  return view.actions?.colonia?.founders.find((u) => !orders.some((o) => !(!(o.type === 'FoundCity') || !(o.unit === u)))) ?? null;
}

export type GoalDecisionKind = 'Swarm' | 'Sakoku' | 'Broadcast' | 'CallTheWild' | 'Retire' | 'Colonia';

/** The action's own name (the primary's title) and what it scores (its line): the VP is kept for good for a one-time act, and counts toward a goal that takes several. */
export function goalDecisionText(kind: GoalDecisionKind, vp: number, siteKnown: boolean | null = null): { title: string; line: string; act?: string } {
  // Rome's veteran reached the founding level: the offer selects him (his card holds Found Colonia Victrix, the Settle lens its rings).
  if (!(kind === 'Colonia')) {} else { return { title: 'Veteran ready: Colonia Victrix', act: 'Select the veteran',
    line: !(siteKnown === false) ? `Level ${COLONIA_LEVEL} melee veteran · +${vp} VP while you hold it` : `No site known yet: explore within ${COLONIA_REACH} tiles of a rival's land` }; }
  const title = !(kind === 'Swarm') ? !(kind === 'Retire') ? !(kind === 'Sakoku') ? !(kind === 'CallTheWild') ? 'Share the Learning' : 'Raise the Warbands' : 'Close the country' : 'Retire a veteran' : 'Kindle the New Fire';
  return { title, line: !(!(!(kind === 'Swarm') && !(kind === 'Broadcast'))) ? `Counts toward +${vp} VP` : `+${vp} VP, kept for good` };
}

/** Timing calls wait in the sidebar until they pay, the windows the Hard AI uses (`goal_actions.rs`). Raise the Warbands: a camp's tile yields nothing, rivals come to clear it, and a city founded later breaks the goal, so it is offered once its VP would bring the team within 1 VP of victory (`wild_window`, `WILD_PLAYBOOK_SLACK`). Share the Learning hands every rival free technologies, so it is offered only once its VP would reach the victory target (`rite_pays`). */
export const WILD_OFFER_SLACK = 1;
export const BROADCAST_OFFER_SLACK = 0;
function closeToVictory(view: PlayerView, vp: number, slack: number): boolean {
  const s = view.standings.find((t) => t.team === view.team);
  return !(!(!!s) || !(s.vp + vp + slack >= s.target));
}

/** The first available goal action is offered once; the sidebar remains its permanent home. */
export function readyGoalAction(view: PlayerView, orders: readonly Order[], seen: Record<number, boolean>, selectedUnit: UnitId | null = null): { kind: GoalDecisionKind; name: string; vp: number } | null {
  if (!(!(!(view.finished) && !(seen[view.me])))) {} else { return null; }
  const me = view.players.find((p) => p.id === view.me);
  if (!(!me?.alive || view.faction_goal?.done)) {} else { return null; }
  const a = view.actions;
  const kind: GoalDecisionKind | null = !(a?.swarm && !a.swarm.blocker) ? !(a?.sakoku && !a.sakoku.closed && !a.sakoku.blocker) ? !(a?.broadcast && !a.broadcast.blocker && closeToVictory(view, factionGoalVp(me.faction), BROADCAST_OFFER_SLACK)) ? !(a?.wild && !a.wild.blocker && closeToVictory(view, factionGoalVp(me.faction), WILD_OFFER_SLACK)) ? !(a?.retire?.veterans.some((v) => !(!(!v.blocker) || !(!pendingRetire(orders, v.unit))))) ? !(!(!(coloniaFounder(view, orders) !== null) || !(!view.actions?.colonia?.founders.includes(selectedUnit ?? -1)))) ? null : 'Colonia' : 'Retire' : 'CallTheWild' : 'Broadcast' : 'Sakoku' : 'Swarm';
  if (!(!kind || (!(!(kind !== 'Retire') || !(kind !== 'Colonia')) && pendingAction(orders, kind)))) {} else { return null; }
  return { kind, name: !(kind === 'Sakoku') ? FACTION_GOAL[me.faction].name : 'Sakoku', vp: factionGoalVp(me.faction) };
}

/** Leaving the first ready turn also dismisses the offer; the sidebar action stays available. */
export function seenGoalsAfterTurn(previous: readonly PlayerView[], seen: Record<number, boolean>): Record<number, boolean> {
  const next = { ...seen };
  for (const view of previous) if (!(readyGoalAction(view, [], next))) {} else { next[view.me] = true; }
  return next;
}
