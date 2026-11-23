import { useEffect, useId, useLayoutEffect, useRef, useState } from 'react';
import { useGame } from '../store';
import { Icon } from './bits';
import { exportRetainedHistories } from '../platform/archive';
import { saveGameToFile } from './GameMenu';
import './EndTurnConfirmation.css';
import './ReplaceGameDialog.css';

/** Downloads the game as it stands (the start of the stuck turn) and its history. Returns false when something failed. */
export async function saveAndReport(): Promise<boolean> {
  const { client, set } = useGame.getState();
  if (!client) return false;
  saveGameToFile();
  if (useGame.getState().error) return false;
  if (client.kind === 'wasm') {
    try {
      await exportRetainedHistories(client.archive?.());
    } catch (e) {
      console.error('legatum: game history was not exported', e);
      set({ error: null });
      return false;
    }
  }
  return true;
}

/**
 * Shown when End Turn failed twice in a row from the same state. Save and report is the one primary;
 * ending the turn with the orders cleared is the way to keep playing. Esc closes it (the next failure opens it again).
 */
export function TurnFailureDialog() {
  const failure = useGame((s) => s.turnFailure);
  const set = useGame((s) => s.set);
  const endWithoutOrders = useGame((s) => s.endTurnWithoutOrders);
  const dialog = useRef<HTMLDialogElement>(null);
  const [note, setNote] = useState<string | null>(null);
  const title = useId();
  const description = useId();
  const open = !!failure;
  useLayoutEffect(() => {
    const el = dialog.current;
    if (!el || !open) return;
    el.showModal();
    return () => el.close();
  }, [open]);
  // A note from an earlier state (Saved.) must not carry into the next failure.
  useEffect(() => setNote(null), [failure]);
  if (!failure) return null;
  const onSave = async () => {
    const ok = await saveAndReport();
    setNote(ok ? 'Saved. Your game file and its history are in your downloads.' : 'Could not save the file.');
  };
  return <dialog ref={dialog} className="modal end-turn-confirmation replace-game" data-testid="turn-failure" aria-labelledby={title} aria-describedby={description}
    onCancel={(e) => { e.preventDefault(); set({ turnFailure: null }); }}>
    <button className="icon-btn end-turn-close" onClick={() => set({ turnFailure: null })} aria-label="Close" title="Close (Esc)"><Icon name="ui-cross" size={14} /></button>
    <h2 id={title}>This turn will not finish</h2>
    <p id={description}>
      {failure.cleared
        ? 'It failed again, even without your orders. Your game is safe at the start of this turn.'
        : 'The game stopped while ending this turn, and it happened again. Your game is safe at the start of this turn.'}
    </p>
    <p>Save and report downloads your game and its history, so you can send it with a bug report.</p>
    {!failure.cleared && <p>Ending the turn with your orders cleared drops what you planned this turn. The other empires still play as usual.</p>}
    <div className="end-turn-actions">
      <button className="btn primary" autoFocus onClick={() => void onSave()}>Save and report</button>
      {!failure.cleared && <button className="btn" onClick={() => endWithoutOrders()}>End the turn with my orders cleared</button>}
    </div>
    <div role="status" style={{ marginTop: 12 }}>{note && <p>{note}</p>}</div>
  </dialog>;
}
