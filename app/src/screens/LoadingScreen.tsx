// Full-screen loading plate, shown while the engine loads, a map generates or a save opens:
// the wordmark at 320 px, a progress rule under it, the house star, the task, and a one-line tip.
// The work behind it reports no progress, so the rule is a slow indeterminate sweep.
import { useEffect, useState } from 'react';
import { Star, Wordmark } from '../brand/ornaments';
import { useReducedMotion } from '../ui/useReducedMotion';

const TIPS = [
  'Eight peoples from across history meet on one map.',
  'The victory race is public: every empire’s points are on the board, every turn.',
  'Reaching the VP target is only the start: hold it for 10 turns to win, and every rival will see the countdown.',
  'Swiss wealth buys buildings and troops cheaply, but the reserve can be lost.',
  'Athenian learning travels far beyond one city.',
  'Everyone plans at once. Orders resolve together when the last empire ends its turn.',
  'Hover any number for its breakdown. Nothing in Legatum is hidden math.',
  'Celtic sacks carry off Gold and raise a warband camp.',
  'Ancient ruins dot the land. The first unit to explore one is rewarded.',
  'A city’s forecast shows what it will make before you commit to it.',
  'A Mongol Keshig shoots or charges each turn, never both.',
  'Scored Age cards are kept for good; world wonders, held goals and rival capitals can be lost.',
  'Spartan soldiers train hard, but peace can empty their ranks.',
];

/** Fades in over what's behind it; `leaving` fades it out before it unmounts. */
export function LoadingScreen({ task, leaving }: { task: string; leaving?: boolean }) {
  const [tip] = useState(() => TIPS[Math.floor(Math.random() * TIPS.length)]);
  return (
    // While it fades out the game is already live (pointer-events: none), so keys and panning work too.
    <div className={`loading-screen ${leaving ? 'leaving' : ''}`} role="status" aria-live="polite" data-game-overlay={leaving ? undefined : ''}>
      <Rose />
      <div className="loading-inner">
        <Wordmark height={68} />
        <div className="loading-rule" aria-hidden>
          <i />
        </div>
        <Star size={14} className="loading-star" />
        <p className="loading-task">{task.replace(/…$/, '')}</p>
        <p className="loading-tip">{tip}</p>
      </div>
    </div>
  );
}

/** A large, faint compass rose behind the plate, as on an atlas's title page. */
function Rose() {
  const pts = (r: number, w: number, a0: number) =>
    Array.from({ length: 4 }, (_, k) => {
      const a = a0 + (k * Math.PI) / 2;
      const [c, s] = [Math.cos(a), Math.sin(a)];
      const [cl, sl] = [Math.cos(a - Math.PI / 4), Math.sin(a - Math.PI / 4)];
      const [cr, sr] = [Math.cos(a + Math.PI / 4), Math.sin(a + Math.PI / 4)];
      return `M0 0L${(cl * w).toFixed(1)} ${(sl * w).toFixed(1)}L${(c * r).toFixed(1)} ${(s * r).toFixed(1)}L${(cr * w).toFixed(1)} ${(sr * w).toFixed(1)}Z`;
    }).join('');
  return (
    <svg className="loading-rose" viewBox="-300 -300 600 600" aria-hidden>
      <circle r="290" />
      <circle r="276" />
      <circle r="200" />
      {Array.from({ length: 64 }, (_, k) => {
        const a = (k * Math.PI) / 32;
        const r0 = k % 8 === 0 ? 256 : k % 2 === 0 ? 266 : 270;
        return <line key={k} x1={Math.cos(a) * r0} y1={Math.sin(a) * r0} x2={Math.cos(a) * 276} y2={Math.sin(a) * 276} />;
      })}
      <path d={pts(190, 26, Math.PI / 4)} />
      <path d={pts(284, 34, 0)} />
    </svg>
  );
}

/** Keep the loading screen mounted briefly after the work ends, so it fades rather than cuts. */
export function useLoadingPlate(active: boolean, task: string | null) {
  const reducedMotion = useReducedMotion();
  const [plate, setPlate] = useState<{ task: string; leaving: boolean } | null>(active ? { task: task ?? '', leaving: false } : null);
  useEffect(() => {
    if (active) {
      setPlate({ task: task ?? '', leaving: false });
      return;
    }
    if (reducedMotion) { setPlate(null); return; }
    setPlate((p) => (p ? { ...p, leaving: true } : null));
    const t = window.setTimeout(() => setPlate(null), 450);
    return () => window.clearTimeout(t);
  }, [active, task, reducedMotion]);
  return plate;
}
