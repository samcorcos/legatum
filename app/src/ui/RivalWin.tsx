import { useGame } from '../store';
import { queueDiplomacy } from '../diplomacy';
import { empireHeadName, empireTitle } from '../empires';
import { atWarLine, rivalLine, type RivalWin as Rival } from '../rivalWin';
import { Amounts } from './Amounts';
import type { PlayerView } from '../types';

/** Opens the Declare War dialog for `team` (the same confirmation as an attack on its city), with no attack attached. */
export function openDeclareWar(view: PlayerView, team: number) {
  const s = useGame.getState();
  s.set({ warPrompt: { seat: s.seat, view, unit: null, target: null, team } });
}

/** The plain facts and the actions against a team close to winning: who it is, who fights it, and what you may do about it. */
export function RivalWin({ view, rival, declare = true }: { view: PlayerView; rival: Rival; declare?: boolean }) {
  const orders = useGame((s) => s.orders[s.seat] ?? []);
  const set = useGame((s) => s.set);
  const editOrders = useGame((s) => s.editOrders);
  const at = atWarLine(view, rival);
  const people = empireHeadName(view, rival.team);
  const blocked = rival.declare && 'blocked' in rival.declare ? rival.declare.blocked : null;
  return <div className="rival-win" data-testid="rival-win">
    <p className="rival-win-line">{rivalLine(view, rival)}</p>
    {at && <p className="rival-win-line muted">{at}</p>}
    {blocked && <p className="rival-win-line muted" data-testid="rival-declare-blocked">You cannot declare war on {people} now: <Amounts text={blocked} /></p>}
    <div className="rival-win-actions">
      {declare && rival.declare && 'action' in rival.declare && <button className="btn primary" onClick={() => openDeclareWar(view, rival.team)}>Declare war on {people}</button>}
      {rival.offers.map(({ team, action }) => {
        const queued = orders.some((o) => o.type === 'ProposeTreaty' && o.to === team && o.kind === 'Alliance');
        const cost = [action.cost > 0 ? `${action.cost} Gold now` : '', (action.upkeep ?? 0) > 0 ? `${action.upkeep} Gold a turn upkeep` : ''].filter(Boolean).join(', ');
        return <span key={team} className="rival-offer">
          <button className="btn" disabled={queued || view.finished} onClick={() => editOrders((o) => queueDiplomacy(o, action.order, view))}>
            {queued ? `Alliance offered to ${empireTitle(view, team)}` : `Offer Alliance to ${empireTitle(view, team)}`}
          </button>
          {cost && <small className="rival-cost">Costs {cost}</small>}
        </span>;
      })}
      <button className="link" onClick={() => set({ vpExpanded: true })}>Show what holds their VP</button>
    </div>
  </div>;
}
