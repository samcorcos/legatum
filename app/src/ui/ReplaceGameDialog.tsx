import { useId, useLayoutEffect, useRef } from 'react';
import './EndTurnConfirmation.css';
import './ReplaceGameDialog.css';

/**
 * Starting a new game, or choosing a file with Load Game, when a saved game exists: asks once. Go back is the safe
 * choice: it has focus and Esc means it, so a stray Return or Esc keeps the game. Start new game (or Load the file) is the one primary.
 */
export function ReplaceGameDialog({ saved, file = false, onStart, onCancel }: { saved: { label: string; turn: number }; /** A game chosen with Load Game, not a new game. */ file?: boolean; onStart: () => void; onCancel: () => void }) {
  const dialog = useRef<HTMLDialogElement>(null);
  const back = useRef<HTMLButtonElement>(null);
  const title = useId();
  const description = useId();
  useLayoutEffect(() => {
    const el = dialog.current;
    if (!el) return;
    el.showModal();
    back.current?.focus({ preventScroll: true });
    return () => el.close();
  }, []);
  return <dialog ref={dialog} className="modal end-turn-confirmation replace-game" data-testid="replace-game" aria-labelledby={title} aria-describedby={description}
    onCancel={(e) => { e.preventDefault(); onCancel(); }}>
    <h2 id={title}>Replace your saved game?</h2>
    <p id={description}>You have a game in progress{saved.label ? `: ${saved.label}` : ''}, turn {saved.turn}. {file ? 'Loading this file replaces it.' : 'Starting a new game replaces it.'}</p>
    <p>To keep it, choose Go back, then Continue, then Menu → Save to file.</p>
    <div className="end-turn-actions">
      <button className="btn primary" onClick={onStart}>{file ? 'Load the file' : 'Start new game'}</button>
      <button ref={back} className="btn" onClick={onCancel}>Go back</button>
    </div>
  </dialog>;
}
