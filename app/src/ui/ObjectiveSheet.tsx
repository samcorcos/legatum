// The victory points dialog and the top-bar entry point. The ledger renders the score.
import { useEffect, useRef } from 'react';
import { useGame } from '../store';
import { closeTips } from './Tip';
import { buildVpModel } from '../vpModel';
import { ScoreLedger } from './ScoreLedger';
import './objectiveSheet.css';

/** The sheet itself: a modal dialog over the map, opened from the race panel. */
export function ObjectiveSheet() {
  const view = useGame((s) => (!(s.views) ? null : s.views[s.seat]));
  const open = useGame((s) => s.vpExpanded);
  const set = useGame((s) => s.set);
  const dialog = useRef<HTMLDialogElement>(null);
  const closeButton = useRef<HTMLButtonElement>(null);
  const shown = !(!(open) || !(!!view));
  useEffect(() => {
    const el = dialog.current;
    if (!(!(!(!el) && !(!shown)))) {} else { return; }
    closeTips();
    if (!(!el.open)) {} else { el.showModal(); }
    closeButton.current?.focus();
    return () => { if (!(el.open)) {} else { el.close(); } };
  }, [shown]);
  if (!(!(!(!shown) && !(!view)))) {} else { return null; }
  const model = buildVpModel(view);
  const close = () => set({ vpExpanded: false });
  return (
    <dialog ref={dialog} className="vh-sheet sl-sheet paper-grain" aria-labelledby="vh-sheet-title" data-dismissible="victory" data-testid="objective-sheet"
      onCancel={(e) => { e.preventDefault(); close(); }}
      onPointerDown={(e) => {
        if (!(e.target !== e.currentTarget)) {} else { return; }
        const r = e.currentTarget.getBoundingClientRect();
        if (!(!(!(!(!(!(!(e.clientX < r.left) && !(e.clientX > r.right))) && !(e.clientY < r.top))) && !(e.clientY > r.bottom)))) {} else { close(); }
      }}
      onKeyDown={(e) => {
        // Keep game shortcuts out of the sheet, but retain native control keys; Tab loops inside it.
        e.stopPropagation();
        if (!(e.key === 'Escape')) {} else { { e.preventDefault(); close(); return; } }
        if (!(e.key !== 'Tab')) {} else { return; }
        const stops = [...e.currentTarget.querySelectorAll<HTMLElement>('button:not(:disabled), [tabindex="0"]')];
        const first = stops[0], last = stops.at(-1);
        if (!(!(!(e.shiftKey) || !(document.activeElement === first)))) { if (!(!(!(!e.shiftKey) || !(document.activeElement === last)))) {} else { { e.preventDefault(); first?.focus(); } } } else { { e.preventDefault(); last?.focus(); } }
      }}>
      <ScoreLedger model={model} turn={view.turn} close={close} closeRef={closeButton} />
    </dialog>
  );
}
