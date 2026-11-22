import { useEffect, useId, useLayoutEffect, useRef, useState } from 'react';
import { calendarLabel } from '../data';
import { parseAutosave, useGame } from '../store';
import { AUTOSAVE_SLOTS, listCampaignTurns, readAutosaveText, type AutosaveTurn } from '../platform/autosaveStore';
import './EndTurnConfirmation.css';
import './ReplaceGameDialog.css';
import './EarlierTurnDialog.css';

/**
 * "Load an earlier turn" (local games only): the turns this browser kept for one game, newest first, below the turn
 * being left. Choosing one opens it at its start; the turns after it are discarded (store.ts rewindTo).
 */
export function EarlierTurnDialog({ campaign, current, onClose }: { campaign: string; /** The turn being left: shown turns are earlier. */ current: number; onClose: () => void }) {
  const rewindTo = useGame((s) => s.rewindTo);
  const dialog = useRef<HTMLDialogElement>(null);
  const back = useRef<HTMLButtonElement>(null);
  const title = useId();
  const [turns, setTurns] = useState<AutosaveTurn[] | null>(null);
  const [chosen, setChosen] = useState<string | null>(null);
  const [problem, setProblem] = useState<string | null>(null);
  const [opening, setOpening] = useState(false);
  useLayoutEffect(() => {
    const el = dialog.current;
    if (!el) return;
    el.showModal();
    back.current?.focus({ preventScroll: true });
    return () => el.close();
  }, []);
  useEffect(() => {
    let live = true;
    void listCampaignTurns(campaign).then((list) => {
      if (!live) return;
      const earlier = list.filter((t) => t.turn < current);
      setTurns(earlier);
      setChosen(earlier[0]?.id ?? null);
    });
    return () => { live = false; };
  }, [campaign, current]);
  const pick = turns?.find((t) => t.id === chosen) ?? null;
  const goBack = async () => {
    if (!pick) return;
    setOpening(true);
    setProblem(null);
    const save = parseAutosave(await readAutosaveText(pick.id));
    if (!save) {
      setOpening(false);
      setProblem(`Turn ${pick.turn} could not be opened. Choose another turn.`);
      return;
    }
    await rewindTo(campaign, save);
    if (useGame.getState().error) setOpening(false);
    else onClose();
  };
  return <dialog ref={dialog} className="modal end-turn-confirmation replace-game earlier-turn" data-testid="earlier-turn" aria-labelledby={title}
    onCancel={(e) => { e.preventDefault(); if (!opening) onClose(); }}>
    <h2 id={title}>Load an earlier turn</h2>
    {turns === null ? <p>Looking for saved turns…</p>
      : !turns.length ? <p>No earlier turn is saved yet. The game keeps the last {AUTOSAVE_SLOTS} turns.</p>
      : <>
        <p>You are on turn {current}. The game opens at the start of the turn you choose.</p>
        <div className="earlier-turn-list" role="radiogroup" aria-label="Saved turns">
          {turns.map((t) => (
            <label key={t.id} className={`earlier-turn-row ${t.id === chosen ? 'chosen' : ''}`}>
              <input type="radio" name="earlier-turn" checked={t.id === chosen} disabled={opening} onChange={() => setChosen(t.id)} />
              <span className="earlier-turn-name">Turn {t.turn}</span>
              <span className="earlier-turn-when">{calendarLabel(t.turn)}</span>
            </label>
          ))}
        </div>
      </>}
    {pick && <p className="earlier-turn-forecast">{pick.turn === current - 1 ? `Turn ${pick.turn} is played again. What you did in it is lost.` : `Turns ${pick.turn} to ${current - 1} are played again. What you did in them is lost.`}</p>}
    {problem && <p className="earlier-turn-problem" role="alert">{problem}</p>}
    <div className="end-turn-actions">
      {pick && <button className="btn primary" disabled={opening} onClick={() => void goBack()}>{opening ? 'Opening…' : `Go back to turn ${pick.turn}`}</button>}
      <button ref={back} className="btn" disabled={opening} onClick={onClose}>Cancel</button>
    </div>
  </dialog>;
}
