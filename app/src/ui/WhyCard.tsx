import { useEffect, useRef, useState, type KeyboardEvent } from 'react';
import { useGame } from '../store';
import './why.css';

/**
 * Coach questions: one card at a time, at the start of the turn after the decision. It never
 * blocks the turn or a teammate: it floats under the top bar, the map stays usable, and End Turn
 * works with it open (an unanswered question counts as skipped). Cmd or Ctrl+Return sends.
 * Esc only hides the card behind a small "Coach question" button (what he typed is kept), because
 * an Escape meant for the map used to skip the question for good (user, 2026-11-21). Skipping
 * takes the Skip button. The turn report (L) lists the question too and reopens it, even after a skip.
 */
export function WhyCard() {
  const question = useGame((s) => s.whyQuestion);
  const answer = useGame((s) => s.answerWhy);
  const [text, setText] = useState('');
  const hidden = useGame((s) => s.whyHidden);
  // The open turn report lists the question itself; the floating button would cover its header.
  const reportOpen = useGame((s) => s.reportOpen && !!s.report);
  const set = useGame((s) => s.set);
  const box = useRef<HTMLTextAreaElement>(null);
  const key = question ? `${question.turn}:${question.key}` : '';
  useEffect(() => {
    setText('');
    if (key && !useGame.getState().whyHidden) box.current?.focus();
  }, [key]);
  // Reopened (its button, or the turn report): the cursor goes back into the answer.
  useEffect(() => {
    if (key && !hidden) box.current?.focus();
  }, [hidden]); // eslint-disable-line react-hooks/exhaustive-deps
  if (!question) return null;
  const send = () => { if (text.trim()) answer(text); };
  const hide = () => {
    set({ whyHidden: true });
    document.querySelector<HTMLElement>('[data-game-map]')?.focus({ preventScroll: true });
  };
  const show = () => set({ whyHidden: false });
  if (hidden) {
    if (reportOpen) return null;
    return (
      <button className="why-reopen" onClick={show} data-testid="why-reopen"
        aria-label="Open the coach's question">
        <span className="why-mark" aria-hidden="true">?</span>
        Coach question
        {text.trim() && <span className="why-reopen-draft"> · draft kept</span>}
      </button>
    );
  }
  const onKey = (e: KeyboardEvent) => {
    if (e.key === 'Escape') { e.preventDefault(); e.stopPropagation(); hide(); }
    else if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) { e.preventDefault(); e.stopPropagation(); send(); }
  };
  return (
    <aside className="why-card" role="dialog" aria-modal="false" aria-label="A question from the coach" data-testid="why-card" onKeyDown={onKey}>
      <div className="why-kicker">Coach · Hard asks</div>
      <p className="why-text" data-testid="why-text">{question.text}</p>
      <textarea
        ref={box}
        className="why-input"
        rows={3}
        value={text}
        placeholder="Your reasons, in your own words"
        aria-label="Your answer"
        data-testid="why-input"
        onChange={(e) => setText(e.target.value)}
      />
      <div className="why-actions">
        <button className="btn quiet why-later" onClick={hide} data-testid="why-hide">Later <span className="kbd">Esc</span></button>
        <button className="btn quiet" onClick={() => answer(null)} data-testid="why-skip">Skip</button>
        <button className="btn primary" disabled={!text.trim()} onClick={send} data-testid="why-send">Send <span className="kbd">⌘ Return</span></button>
      </div>
    </aside>
  );
}
