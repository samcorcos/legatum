import { useEffect } from 'react';
import { useGame } from '../store';
import { NOTICE_MS, undoNotice } from '../notice';
import { Kbd } from './bits';
import { useCoarsePointer } from './useCoarsePointer';
import './DecisionNotice.css';

/** G19: Z (or Ctrl/Cmd+Z) undoes the last answer while its notice shows, except in text entry. */
function undoKey(e: KeyboardEvent) {
  if (e.defaultPrevented || e.isComposing || e.repeat || e.altKey || e.shiftKey || e.key.toLowerCase() !== 'z') return;
  if ((e.target as HTMLElement | null)?.closest?.('input, textarea, select, [contenteditable=""], [contenteditable="true"]')) return;
  e.preventDefault();
  e.stopPropagation();
  undoNotice();
}

/** G17: the non-blocking confirmation over the map after a decision; Undo is its one action. */
export function DecisionNotice() {
  const notice = useGame((s) => s.notice);
  const touch = useCoarsePointer();
  useEffect(() => {
    if (!notice) return;
    const timer = setTimeout(() => { if (useGame.getState().notice?.id === notice.id) useGame.getState().set({ notice: null }); }, NOTICE_MS);
    window.addEventListener('keydown', undoKey, true);
    return () => { clearTimeout(timer); window.removeEventListener('keydown', undoKey, true); };
  }, [notice]);
  if (!notice) return null;
  return <div className="toast decision-notice" role="status">
    <span>{notice.text}</span>
    <button className="btn quiet" aria-label="Undo" onClick={undoNotice}>Undo{!touch && <> <Kbd>Z</Kbd></>}</button>
  </div>;
}
