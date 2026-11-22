import { useId, useLayoutEffect, useRef } from 'react';
import './EndTurnConfirmation.css';

/** Starting a new game when a saved one exists: asks once. Start new game is the one primary; Go back has focus, so a stray Return keeps the game. */
export function ReplaceGameDialog({ saved, file = false, onStart, onCancel }: { saved: { label: string; turn: number }; /** A game chosen with Load Game takes the place, not a new game. */ file?: boolean; onStart: () => void; onCancel: () => void }) {
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
    <p id={description}>You have a game in progress{saved.label ? `: ${saved.label}` : ''}, turn {saved.turn}. {file ? 'The game in the file' : 'A new game'} takes its place once its first turn is saved.</p>
    <p>Want to keep it? Go back, choose Continue, and use Save to file in the game menu.</p>
    <div className="end-turn-actions">
      <button className="btn primary" onClick={onStart}>{file ? 'Load the file' : 'Start new game'}</button>
      <button ref={back} className="btn" onClick={onCancel}>Go back</button>
    </div>
  </dialog>;
}
