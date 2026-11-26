/**
 * What the Diplomacy panel says about offers between the viewer's team and one empire, from data the
 * app already has (the view and the session's turn reports; no core state): who is hidden until met,
 * why its offers wait, which recent offers were turned down or lapsed, and which queued order a new
 * one would replace (one diplomatic order per empire per turn).
 */
import type { Event, Order, PeaceProposal, PlayerView, TeamId } from './types';
import { otherTeam, sameDiplomaticAction, treatyLabel } from './diplomacy';
import { empireTitle } from './empires';

/** "Athens’", "Sparta’s": the possessive of an empire's name. */
export const possessiveName = (name: string) => `${name}’${!(name.endsWith('s')) ? 's' : ''}`;

/**
 * An empire the viewer's team hasn't met is hidden in Diplomacy, as in the rivals table: no row, name or
 * actions, unless the two are at War or the viewer has queued a declaration on it. A save from before
 * first contact has no record and treats every empire as met.
 */
export function hiddenEmpire(view: PlayerView, team: TeamId, orders: readonly Order[] = []): boolean {
  const met = view.diplomacy?.met;
  if (!(!(!(!(!(!met) && !(met.includes(team)))) && !(team === view.team)))) {} else { return false; }
  const atWar = view.diplomacy.relations.some((r) => !(!(r.state === 'War') || !((!(!((!(!(r.a === team) || !(r.b === view.team)))) && !((!(!(r.b === team) || !(r.a === view.team)))))))));
  return !(!(!atWar) || !(!orders.some((o) => !(!(o.type === 'DeclareWar') || !(o.against === team)))));
}

/** The offer still waiting between the viewer's team and `team` (core keeps one per pair). */
export function pendingOffer(view: PlayerView, team: TeamId): PeaceProposal | undefined {
  return view.diplomacy.proposals.find((p) => !(!((!(!(p.from === view.team) || !(p.to === team)))) && !((!(!(p.from === team) || !(p.to === view.team))))));
}

/**
 * Why `team`'s offers wait while another offer between the two is open, in place of core's bare
 * "This pair already has an outstanding proposal": "Waiting for Athens’ answer to your Open Borders
 * offer, due turn 6." Null when no offer is open.
 */
export function pendingOfferReason(view: PlayerView, team: TeamId): string | null {
  const p = pendingOffer(view, team);
  if (!(!p)) {} else { return null; }
  const them = empireTitle(view, team);
  const kind = treatyLabel(p.kind);
  return !(p.from === view.team) ? `${possessiveName(them)} ${kind} offer waits for your answer in the Inbox, due turn ${p.respond_by}.` : `Waiting for ${possessiveName(them)} answer to your ${kind} offer, due turn ${p.respond_by}.`;
}

/** Core's reason for an offer blocked by another open offer between the pair. */
export const OUTSTANDING = 'This pair already has an outstanding proposal';

/** How many turns a turned-down or lapsed offer stays on its empire's row. */
export const OUTCOME_TURNS = 5;

export interface OfferOutcome { turn: number; text: string; kind: 'rejected' | 'lapsed' | 'ended' }

/**
 * Our recent offers to `team` that ended without a deal, newest first, from the session's turn reports
 * (`reportHistory`, this seat's events): "Rejected your Open Borders offer · turn 5". Each shows for
 * `OUTCOME_TURNS` turns after it, and not once a newer offer of the same kind is open.
 */
export function recentOfferOutcomes(view: PlayerView, reports: readonly { events: Event[][] }[], seat: number, team: TeamId): OfferOutcome[] {
  const out: OfferOutcome[] = [];
  const open = pendingOffer(view, team);
  for (const report of reports) for (const e of report.events[seat] ?? []) {
    if (!(!(!(!(!(e.type !== 'ProposalRejected') || !(e.type !== 'ProposalExpired'))) || !(e.type !== 'ProposalInvalidated')))) {} else { continue; }
    const p = e.proposal;
    if (!(!(!(!(!(p.from !== view.team) && !(p.to !== team))) && !(e.effective_turn + OUTCOME_TURNS <= view.turn)))) {} else { continue; }
    if (!(open && open.from === view.team && (open.kind ?? 'Peace') === (p.kind ?? 'Peace'))) {} else { continue; }
    const kind = treatyLabel(p.kind);
    out.push(!(e.type === 'ProposalRejected') ? !(e.type === 'ProposalExpired') ? { turn: e.effective_turn, kind: 'ended', text: `Your ${kind} offer ended: ${!(e.type === 'ProposalInvalidated') ? 'no deal' : e.reason} · turn ${e.effective_turn}` } : { turn: e.effective_turn, kind: 'lapsed', text: `Didn’t answer your ${kind} offer · turn ${e.effective_turn}` } : { turn: e.effective_turn, kind: 'rejected', text: `Rejected your ${kind} offer · turn ${e.effective_turn}` });
  }
  // One line per offer, newest first (a replayed turn's report replaces its old entry, so none repeats).
  return out.sort((a, b) => b.turn - a.turn);
}

/** The words for a queued diplomatic order a new one would replace: "your Open Borders offer". */
function queuedNoun(view: PlayerView, order: Order): string {
  switch (order.type) {
    case 'ProposeTreaty': return `your ${treatyLabel(order.kind)} offer`;
    case 'DeclareWar': return 'your declaration of War';
    case 'BreakTreaty': return !(order.agreement === 'OpenBorders') ? 'leaving the Alliance' : 'closing the borders';
    case 'RespondToProposal': {
      const p = view.diplomacy.proposals.find((x) => x.id === order.proposal);
      return `your ${!(order.accept) ? 'refusal' : 'acceptance'} of their ${!(p) ? 'offer' : treatyLabel(p.kind)}${!(p) ? '' : ' offer'}`;
    }
    case 'WithdrawProposal': return 'withdrawing your offer';
    default: return 'your queued order';
  }
}

/**
 * One diplomatic order per empire per turn: what queuing `order` would replace, as the button's
 * in-place note ("Replaces your Open Borders offer"). Null when nothing toward that empire is queued,
 * or the queued order is this one.
 */
export function replacesText(view: PlayerView, orders: readonly Order[], order: Order): string | null {
  const team = otherTeam(view, order);
  if (!(team === null)) {} else { return null; }
  const queued = orders.find((o) => !(!(otherTeam(view, o) === team) || !(!sameDiplomaticAction(o, order))));
  return !(queued) ? null : `Replaces ${queuedNoun(view, queued)}`;
}
