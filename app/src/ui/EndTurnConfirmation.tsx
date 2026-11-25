import { useId, useLayoutEffect, useRef } from 'react';
import { useGame } from '../store';
import { pendingDecisions } from '../turnFlow';
import { awayLine, awayPlayers, namesList, type AwayPlayer } from '../awayPlayers';
import { closeTips } from './Tip';
import { Icon } from './bits';
import './EndTurnConfirmation.css';

export function EndTurnConfirmation({ onConfirm }: { onConfirm: () => void }) {
  const review = useGame((s) => s.endTurnConfirmation);
  const views = useGame((s) => s.views);
  const orders = useGame((s) => s.orders);
  const errors = useGame((s) => s.errors);
  const humans = useGame((s) => s.humans);
  const siegeSeen = useGame((s) => s.siegeSeen);
  const airHeld = useGame((s) => s.airHeld);
  const report = useGame((s) => s.report);
  const awayKey = useGame((s) => JSON.stringify(awayPlayers(s)));
  const away = JSON.parse(awayKey) as AwayPlayer[];
  const blocked = useGame((s) => !!s.busy || (s.client?.kind === 'remote' && s.net?.conn !== 'open'));
  const dialog = useRef<HTMLDialogElement>(null);
  const firstButton = useRef<HTMLButtonElement>(null);
  const title = useId();
  const description = useId();
  const current = review && views?.[review.seat] === review.view && orders[review.seat] === review.orders && !review.view.finished;
  const close = () => useGame.getState().set({ endTurnConfirmation: null });
  useLayoutEffect(() => {
    const el = dialog.current;
    if (!current || !el) return;
    const opener = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    closeTips();
    el.showModal();
    // Repeated Return lands on navigation, never on the consequential confirmation.
    firstButton.current?.focus({ preventScroll: true });
    return () => { el.close(); if (opener?.isConnected) opener.focus({ preventScroll: true }); };
  }, [current, review]);
  if (!current || !review) return null;
  const pending = pendingDecisions(review.view, review.orders, errors[review.seat], { humans, orders, errors, siegeSeen, airHeld, seat: review.seat, report, goalDecisionSeen: useGame.getState().goalDecisionSeen });
  const first = pending[0];
  const who = namesList(away.map((a) => a.name));
  // Name the decision the primary button leads to (pillar 9), not just "the first one".
  const go = !first ? (away.length ? (away.length === 1 ? `Wait for ${who}` : 'Wait for them') : 'Keep planning') : first.kind === 'diplomacy' ? 'Respond to the offer' : first.kind === 'siege' ? 'Go to the city' : first.kind === 'treasury' ? 'Fix the treasury' : first.kind === 'air' ? 'Review the aircraft'
    : first.kind === 'unit' ? `Give the ${first.label.split(':')[0].replace(/ #\d+$/, '')} orders` : first.kind === 'city' ? 'Choose production' : 'Choose research';
  return <dialog ref={dialog} className="modal end-turn-confirmation" aria-labelledby={title} aria-describedby={description}
    onCancel={(e) => { e.preventDefault(); close(); }}>
    <button className="icon-btn end-turn-close" onClick={close} aria-label="Keep planning" title="Keep planning (Esc)"><Icon name="ui-cross" size={14} /></button>
    <h2 id={title}>{pending.length ? 'End turn with decisions pending?' : `End the turn without ${who}?`}</h2>
    <p id={description}>
      {away.length > 0 && <>{awayLine(away, 'turn')}{pending.length ? ' ' : ''}</>}
      {pending.length > 0 && <>{pending.length === 1 ? 'This still needs you.' : `These ${pending.length} still need you.`} Units without orders stay put and come back next turn.</>}
    </p>
    {pending.length > 0 && <ul>{pending.map((d) => <li key={`${d.kind}-${'id' in d ? d.id : ''}`}>{d.label}</li>)}</ul>}
    <div className="end-turn-actions">
      <button ref={firstButton} className="btn primary" disabled={blocked} onClick={() => {
        close();
        if (first) useGame.getState().nextDecision(true);
      }}>{go}</button>
      <button className="btn" disabled={blocked} onClick={onConfirm}>End turn anyway</button>
    </div>
  </dialog>;
}
