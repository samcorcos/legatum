import { useEffect, useLayoutEffect, useRef, useState } from 'react';
import { useCelebration } from '../celebrate/MomentLane';
import { useGame } from '../store';
import { Star } from '../brand/ornaments';
import { useGuide } from './guideState';
import { eligibleHints, hintCopy, useHintPreferences, type HintId } from './hintState';
import './FirstGameHints.css';
import { usePhoneLayout } from './usePhoneLayout';

export function FirstGameHints() {
  const phone = usePhoneLayout();
  const panel = useRef<HTMLElement>(null);
  const view = useGame((s) => s.views?.[s.seat]);
  const seat = useGame((s) => s.seat);
  const allOrders = useGame((s) => s.orders);
  const errors = useGame((s) => s.errors);
  const humans = useGame((s) => s.humans);
  const orders = allOrders[seat];
  // The report and the diplomacy inbox share the hint's story area: one at a time.
  // Scenarios are for players who know the game: their objective bar takes the hint's place.
  const paused = useGame((s) => !!s.scenario || !!s.busy || !!s.endTurnConfirmation || s.techOpen || s.reportOpen || s.diplomacyOpen || !!s.ready[s.seat] || !!s.place ||
    (s.client?.kind === 'remote' && s.net?.conn !== 'open'));
  const guideOpen = useGuide((s) => s.open);
  const { enabled, seen, resetVersion, markSeen } = useHintPreferences();
  const [active, setActive] = useState<{ id: HintId; version: number } | null>(null);
  const { active: celebration, holding } = useCelebration();
  const hidden = paused || holding || !!celebration || guideOpen || !enabled;

  useEffect(() => {
    if (hidden || !view) return;
    const eligible = eligibleHints(view, orders, errors[seat], { humans, orders: allOrders, errors });
    if (active?.version === resetVersion && eligible.includes(active.id)) return;
    const id = eligible.find((candidate) => !seen.includes(candidate));
    setActive(id ? { id, version: resetVersion } : null);
    // Persist on display, not just dismissal, so reloads and new games do not repeat a tip.
    if (id) markSeen(id);
  }, [view, orders, seat, allOrders, errors, humans, hidden, active, seen, resetVersion, markSeen]);

  // On narrow screens the destination card uses the space between the hint and lens dock.
  useLayoutEffect(() => {
    const el = panel.current, game = el?.closest<HTMLElement>('.game');
    if (!el || !game) return;
    const measure = () => game.style.setProperty('--hint-bottom', `${el.getBoundingClientRect().bottom - game.getBoundingClientRect().top}px`);
    measure();
    const resize = new ResizeObserver(measure);
    resize.observe(el);
    return () => { resize.disconnect(); game.style.removeProperty('--hint-bottom'); };
  }, [hidden, active, resetVersion, view?.finished]);

  if (hidden || !active || active.version !== resetVersion || !view || view.finished) return null;
  const hint = hintCopy(active.id, view);
  return (
    <aside ref={panel} className="first-game-hint" data-overlay-slot="story-hint" data-testid="first-game-hint" aria-label="First-game hint" onKeyDown={(e) => e.stopPropagation()}>
      {phone ? <>
        <button className="phone-hint-guide" onClick={() => useGuide.getState().show(hint.section)} aria-label={`Read the guide: ${hint.title}`}>
          <span className="first-game-hint-label">First-game hint · Read guide →</span>
          <span className="phone-hint-title" role="status">{hint.title}</span>
        </button>
        <button className="btn quiet" onClick={() => setActive(null)} aria-label={`Dismiss hint: ${hint.title}`}>Got it</button>
      </> : <>
      <div className="first-game-hint-copy" role="status" aria-live="polite" aria-atomic="true">
        <div className="first-game-hint-label"><Star size={12} /> First-game hint</div>
        <h3>{hint.title}</h3>
        <p>{hint.body}</p>
      </div>
      <div className="first-game-hint-actions">
        <button onClick={() => useGuide.getState().show(hint.section)}>Read the guide</button>
        <button onClick={() => setActive(null)} aria-label={`Dismiss hint: ${hint.title}`}>Got it</button>
      </div>
      </>}
    </aside>
  );
}

/** Reused by title Settings and the in-game menu; reset also re-enables the hints. */
export function HintSettings() {
  const { enabled, seen, resetVersion, setEnabled, reset } = useHintPreferences();
  const [resetAt, setResetAt] = useState<number | null>(null);
  return (
    <div className="hint-settings" onKeyDown={(e) => e.stopPropagation()}>
      <label><input type="checkbox" checked={enabled} onChange={(e) => setEnabled(e.target.checked)} />First-game hints</label>
      <div className="hint-settings-foot">
        <span role="status">{resetAt === resetVersion && !seen.length ? 'Hints reset' : 'Shown once in this browser'}</span>
        <button onClick={() => { reset(); setResetAt(useHintPreferences.getState().resetVersion); }}>Reset hints</button>
      </div>
    </div>
  );
}
