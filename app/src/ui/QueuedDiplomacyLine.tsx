import { useEffect, useRef, type ReactNode } from 'react';
import { Icon } from './bits';
import './DiplomacyPanel.css';

/**
 * True once `queued` turns on while the caller is already on screen: the player just queued the
 * order, so the ready line that replaces the clicked control should take focus. Opening a panel
 * that already shows queued orders leaves focus where the panel puts it.
 */
export function useJustQueued(queued: boolean): boolean {
  const before = useRef(queued);
  const just = useRef(false);
  if (queued !== before.current) { just.current = queued; before.current = queued; }
  return just.current;
}

/** A queued diplomacy order's ready line, with one explicit way to cancel it. */
export function QueuedDiplomacyLine({ name, children, onCancel, testId, focus = false }: { name: string; children: ReactNode; onCancel: () => void; testId: string; focus?: boolean }) {
  const cancel = useRef<HTMLButtonElement>(null);
  useEffect(() => { if (focus) cancel.current?.focus(); }, [focus]);
  return <p className="diplomacy-queued" data-testid={testId}>
    <Icon name="ui-check" size={13} className="diplomacy-queued-check" fallback="✓" />{' '}
    <strong>{name}:</strong> {children}{' '}
    <button ref={cancel} className="link" onClick={onCancel}>Cancel</button>
  </p>;
}
