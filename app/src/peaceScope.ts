import { empireTitle } from './empires';
import type { DiplomaticAction, PeaceProposal, PlayerView, TeamId } from './types';

/** A Peace binds its two named teams; other wars continue, including wars with either ally. */
export function peaceScope(view: PlayerView, p: Pick<PeaceProposal, 'from' | 'to' | 'required_teams'>) {
  const other = p.from === view.team ? p.to : p.from;
  const name = empireTitle(view, other);
  const subject = `Peace with ${name}`;
  const title = `${subject}: ends only your war with ${name}`;
  const wars = view.diplomacy.relations.filter((r) => r.state === 'War' &&
    (r.a === p.from || r.b === p.from || r.a === p.to || r.b === p.to)
    && !((r.a === p.from && r.b === p.to) || (r.a === p.to && r.b === p.from)));
  const reason = wars.map((r) => {
    const owner: TeamId = r.a === view.team || r.b === view.team ? view.team
      : r.a === p.from || r.a === p.to ? r.a : r.b;
    const enemy = r.a === owner ? r.b : r.a;
    return owner === view.team ? `Your war with ${empireTitle(view, enemy)} continues.`
      : `The war between ${empireTitle(view, owner)} and ${empireTitle(view, enemy)} continues.`;
  }).join(' ');
  return { subject, title, ending: 'ends that war', reason };
}

export function peaceScopeForAction(view: PlayerView, action: DiplomaticAction) {
  const order = action.order;
  if (order.type !== 'ProposeTreaty' || order.kind !== 'Peace') return null;
  return peaceScope(view, { from: view.team, to: order.to });
}
