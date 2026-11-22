// The victory points dialog and the top-bar entry point. The ledger renders the score.
import { useEffect, useRef } from 'react';
import { useGame } from '../store';
import { Icon } from './bits';
import { closeTips } from './Tip';
import { buildVpModel } from '../vpModel';
import { ScoreLedger } from './ScoreLedger';
import './objectiveSheet.css';

/** The sheet itself: a modal dialog over the map, opened from the VP chip or the race panel. */
export function ObjectiveSheet() {
  const view = useGame((s) => (s.views ? s.views[s.seat] : null));
  const open = useGame((s) => s.vpExpanded);
  const set = useGame((s) => s.set);
  const dialog = useRef<HTMLDialogElement>(null);
  const closeButton = useRef<HTMLButtonElement>(null);
  const shown = open && !!view;
  useEffect(() => {
    const el = dialog.current;
    if (!el || !shown) return;
    closeTips();
    if (!el.open) el.showModal();
    closeButton.current?.focus();
    return () => { if (el.open) el.close(); };
  }, [shown]);
  if (!shown || !view) return null;
  const model = buildVpModel(view);
  const close = () => set({ vpExpanded: false });
  return (
    <dialog ref={dialog} className="vh-sheet sl-sheet paper-grain" aria-labelledby="vh-sheet-title" data-dismissible="victory" data-testid="objective-sheet"
      onCancel={(e) => { e.preventDefault(); close(); }}
      onPointerDown={(e) => {
        if (e.target !== e.currentTarget) return;
        const r = e.currentTarget.getBoundingClientRect();
        if (e.clientX < r.left || e.clientX > r.right || e.clientY < r.top || e.clientY > r.bottom) close();
      }}
      onKeyDown={(e) => {
        // Keep game shortcuts out of the sheet, but retain native control keys; Tab loops inside it.
        e.stopPropagation();
        if (e.key === 'Escape') { e.preventDefault(); close(); return; }
        if (e.key !== 'Tab') return;
        const stops = [...e.currentTarget.querySelectorAll<HTMLElement>('button:not(:disabled), [tabindex="0"]')];
        const first = stops[0], last = stops.at(-1);
        if (e.shiftKey && document.activeElement === first) { e.preventDefault(); last?.focus(); }
        else if (!e.shiftKey && document.activeElement === last) { e.preventDefault(); first?.focus(); }
      }}>
      <ScoreLedger model={model} turn={view.turn} close={close} closeRef={closeButton} />
    </dialog>
  );
}

/** The top bar's entry: your VP, the leading rival's crest and VP; one click opens the sheet. */
export function VpChip() {
  const view = useGame((s) => (s.views ? s.views[s.seat] : null));
  const open = useGame((s) => s.vpExpanded);
  const set = useGame((s) => s.set);
  if (!view) return null;
  return <button className="tb-nav vh-chip" aria-label="Score ledger" aria-haspopup="dialog" aria-expanded={open} title="Score ledger" onClick={() => set({ vpExpanded: !open })}>
    <Icon name="vp" size={14} className="vh-chip-icon" /> <span className="vh-chip-label">Score ledger</span>
  </button>;
}
