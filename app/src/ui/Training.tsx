import { useEffect, useState } from 'react';
import { useGame } from '../store';
import { battleAction, cardText, chipText } from '../training';
import './training.css';

/** Re-render every half second while `on`, for the elapsed seconds. */
function useTick(on: boolean): number {
  const [now, setNow] = useState(() => performance.now());
  useEffect(() => {
    if (!on) return;
    const id = setInterval(() => setNow(performance.now()), 500);
    return () => clearInterval(id);
  }, [on]);
  return now;
}

/**
 * Top bar, while he plans: a quiet line under the turn number ("Training · 1") saying the AI
 * empires on Training are studying their battles. It has no cell of its own, so it never costs
 * the bar a navigation entry; the turn cell's tooltip explains it.
 */
export function TrainingLine() {
  const training = useGame((s) => s.training);
  if (!training || training.waiting || training.progress.done) return null;
  const p = training.progress;
  return (
    <span className="turn-training" data-testid="training-chip" aria-label={chipText(p)}>
      <span className="training-chip-dot" aria-hidden="true" />
      Training · {p.battles.filter((b) => !b.done).length}
    </span>
  );
}

/** The turn cell's tooltip lines while the Training search runs. */
export function TrainingTip() {
  const training = useGame((s) => s.training);
  const view = useGame((s) => (s.views ? s.views[s.seat] : null));
  if (!training || training.waiting || training.progress.done) return null;
  const p = training.progress;
  return <>
    <div className="bd-title">{chipText(p)}</div>
    <div>Training is a research tool. While you plan, AI empires on Training study each of their battles using the true positions of nearby units, including ones they cannot see. A turn with battles can wait up to {Math.round(training.capMs / 1000)} s after End Turn.</div>
    <ul className="training-chip-list">{p.battles.map((b, i) => <li key={i}>{battleAction(b, view)}{b.done ? ' · done' : ''}</li>)}</ul>
  </>;
}

/** End Turn while the search still runs: a calm card naming the battle and the time spent. */
export function TrainingCard() {
  const training = useGame((s) => s.training);
  const view = useGame((s) => (s.views ? s.views[s.seat] : null));
  const now = useTick(Boolean(training?.waiting));
  if (!training) return null;
  const { title, line, fraction } = cardText(training, view, now);
  return (
    <div className="training-card" data-testid="training-card">
      <div className="training-card-head"><div className="spinner" />{title}</div>
      <div className="training-card-line">{line}</div>
      <div className="training-card-track" role="progressbar" aria-label={title} aria-valuemin={0} aria-valuemax={100} aria-valuenow={Math.round(fraction * 100)}>
        <span style={{ width: `${Math.round(fraction * 100)}%` }} />
      </div>
      <div className="training-card-note">Your turn is in. The AI has up to {Math.round(training.capMs / 1000)} s, then plays its best orders.</div>
    </div>
  );
}
