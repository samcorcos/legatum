// Pillar 9: one primary per screen. While a unit of ours is selected, its card holds the primary
// (improvement, movement, Auto-explore, Found City) and the dock's call to action steps back, unless the call to
// action is urgent (an attack on a city, an offer to answer, a treasury run dry, a faction goal ready to take, a broken order) or
// nothing is pending at all (then End turn is the one primary and the card's choice steps back).
import { useGame } from '../store';
import { nextDecision, pendingDecisions, type PendingDecision } from '../turnFlow';

/** The decision the dock's call to action names (SeatBar uses the same rule). */
function ctaDecision(decisions: PendingDecision[], selection: Parameters<typeof nextDecision>[1]): PendingDecision | undefined {
  const following = nextDecision(decisions, selection);
  return following?.issue ? following : following?.kind !== decisions[0]?.kind ? decisions[0] : following;
}

export function unitCardLeads(decisions: PendingDecision[], selection: Parameters<typeof nextDecision>[1]): boolean {
  const next = ctaDecision(decisions, selection);
  if (!next) return false;
  return !(next.kind === 'siege' || next.kind === 'diplomacy' || next.kind === 'treasury' || next.kind === 'goal' || (next.kind === 'unit' && !!next.issue));
}

/** Whether the selected unit's card holds the screen's one primary (the dock's call to action steps back). */
export function useUnitCardLeads(): boolean {
  const view = useGame((s) => s.views?.[s.seat]);
  const selection = useGame((s) => s.selection);
  const orders = useGame((s) => s.orders);
  const errors = useGame((s) => s.errors);
  const seat = useGame((s) => s.seat);
  const humans = useGame((s) => s.humans);
  const siegeSeen = useGame((s) => s.siegeSeen);
  const airHeld = useGame((s) => s.airHeld);
  const goalDecisionSeen = useGame((s) => s.goalDecisionSeen);
  const report = useGame((s) => s.report);
  const ready = useGame((s) => s.ready[s.seat]);
  if (!view || ready) return true;
  return unitCardLeads(pendingDecisions(view, orders[seat], errors[seat], { humans, orders, errors, siegeSeen, airHeld, seat, report, goalDecisionSeen }), selection);
}
