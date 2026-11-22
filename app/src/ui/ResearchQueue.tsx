import { TECH_NAME } from '../data';
import { effectiveResearchQueue, pendingResearchQueue, setResearchQueue } from '../orders';
import { firstScienceForecast } from '../research';
import { useGame } from '../store';
import type { Order, PlayerView, TechId } from '../types';
import { Icon } from './bits';
import { arrivalText, cheapestFirst, moveInQueue, queueSchedule } from './eraResearch';

/** Replace the research queue with `techs` (the one place the panel edits it). */
export function editQueue(view: PlayerView, techs: TechId[]) {
  useGame.getState().editOrders((o) => setResearchQueue(o, techs, view.research), 'queue');
}

/**
 * The research queue and its forecast: each queued tech with the turn it arrives, in the order the player
 * set, with up/down controls that are always visible. The forecast follows an unsent reorder too.
 * Empty, it makes the next action obvious: one primary button queues the whole era, cheapest first.
 * The strip never changes height: it always shows `slotCount` slots, queued ones filled and the rest as empty
 * placeholders, so queueing never moves the cards underneath the pointer.
 */
export function ResearchQueue({ view, orders, slotCount, landed }: { view: PlayerView; orders: Order[]; slotCount: number; landed?: TechId | null }) {
  const r = view.research;
  const queue = effectiveResearchQueue(orders, r);
  const science = view.economy.science_per_turn.total;
  const noScience = !!firstScienceForecast(view);
  const slots = queueSchedule(r, queue, science, view.turn);
  const pending = pendingResearchQueue(orders) !== null;
  const open = r.catalog.filter((t) => t.researchable);
  const idle = !queue.length;
  const order = cheapestFirst(r);
  const unqueued = order.filter((t) => !queue.includes(t));
  const disabled = view.finished;
  const total = Math.max(queue.length, slotCount);
  const queueAll = (techs: TechId[]) => editQueue(view, techs);

  if (idle && !open.length) return <div className="research-queue empty" aria-label="Research queue"><span className="research-queue-title">Research queue</span><span className="muted">Nothing left to research.</span></div>;

  return <div className={`research-queue${idle ? ' idle' : ''}`} aria-label="Research queue">
    <div className="research-queue-head">
      <span className="research-queue-title">Research queue</span>
      {!idle && <span className="research-queue-hint muted">Researched in order, the first one now.</span>}
      {idle && <span className="research-queue-hint muted">Nothing is being researched.</span>}
      {!idle && slots.at(-1)?.turns != null && <span className="research-queue-total muted" data-testid="queue-total">{queue.length > 1 ? 'All done' : 'Done'} T{slots.at(-1)!.arrivalTurn}</span>}
      {pending && <span className="research-queue-pending"><span className="pending-dot" aria-hidden="true" /><span className="pending-label" title="Pending until Ready">pending</span></span>}
      {idle && <span className="research-queue-cta">
        <button className="btn primary" data-primary-action data-testid="queue-all" disabled={disabled} onClick={() => queueAll(order)}>
          {order.length > 1 ? `Queue all ${order.length}, cheapest first` : `Research ${TECH_NAME[order[0]]}`}</button>
      </span>}
      <span className="research-queue-actions">
        {unqueued.length > 1 && !idle && <button className="btn quiet" disabled={disabled} data-testid="queue-rest" onClick={() => queueAll([...queue, ...unqueued])}>Queue the rest, cheapest first</button>}
        {!idle && <button className="btn quiet research-queue-clear" disabled={disabled} aria-label="Clear research queue" onClick={() => editQueue(view, [])}>Clear queue</button>}
      </span>
    </div>
    <div className="research-queue-slots">
    <ol role="list">{slots.map((slot, i) => <li role="listitem" key={slot.tech} className={`${i === 0 ? 'first' : ''}${landed === slot.tech ? ' landed' : ''}`} data-tech={slot.tech}>
      <span className="rq-number">{i + 1}</span>
      <b className="rq-name" title={TECH_NAME[slot.tech]}>{TECH_NAME[slot.tech]}</b>
      <button className="icon-btn rq-remove" title="Remove from the queue" disabled={disabled} aria-label={`Remove ${TECH_NAME[slot.tech]} from the queue`} onClick={() => editQueue(view, queue.filter((t) => t !== slot.tech))}><Icon name="ui-cross" size={12} className="ui-mark" /></button>
      <small className="rq-arrival" data-testid="queue-arrival">{arrivalText(slot, noScience)}</small>
      <span className="research-queue-controls">
        <button className="icon-btn" title="Research earlier" disabled={disabled || i === 0} aria-label={`Move ${TECH_NAME[slot.tech]} earlier`} onClick={() => editQueue(view, moveInQueue(queue, i, -1))}><span aria-hidden="true">◀</span></button>
        <button className="icon-btn" title="Research later" disabled={disabled || i === queue.length - 1} aria-label={`Move ${TECH_NAME[slot.tech]} later`} onClick={() => editQueue(view, moveInQueue(queue, i, 1))}><span aria-hidden="true">▶</span></button>
      </span>
    </li>)}</ol>
    {/* Every card has its own Research now and + Queue buttons, so an empty slot only says so. */}
    {Array.from({ length: total - queue.length }, (_, k) => <div key={`empty-${queue.length + k}`} className="rq-ghost" data-testid="queue-empty" aria-hidden="true">
      <span className="rq-number">{queue.length + k + 1}</span>
      <span className="rq-ghost-text">Empty</span>
    </div>)}
    </div>
  </div>;
}
