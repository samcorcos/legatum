import type { ReactNode } from 'react';
import './ProgressBar.css';

export type ProgressTone = 'good' | 'bad' | 'neutral' | 'paused';

/**
 * The one progress bar for anything that ticks over time (legatum-ui: "anything that ticks has a
 * visible bar with turns left"): a label, current/needed and turns left over an ink-ruled track.
 * Callers pass the core's numbers; this component never forecasts. `bad` (starvation) fills red.
 */
export function ProgressBar({ label, name, icon, value, max, color, amount, turns, foot, tone = 'neutral', testId, className }: {
  label: ReactNode;
  /** Accessible name when `label` is not plain text. */
  name?: string;
  icon?: ReactNode;
  value: number;
  max: number;
  /** Fill colour (a yield token); `bad` and `paused` tones override it. */
  color: string;
  /** Current/needed, e.g. "12/31 Food". Omitted when the bar has no meaningful count. */
  amount?: ReactNode;
  /** Turns left or the state instead ("4 turns", "Starving in 2", "Paused"). */
  turns?: ReactNode;
  /**
   * Moves the numbers under the track: `foot` sits on the left and amount and turns on the right
   * ("0/23 · 2 turns"), so the label above gets the whole width (the territory bar).
   */
  foot?: ReactNode;
  tone?: ProgressTone;
  testId?: string;
  className?: string;
}) {
  const f = max > 0 ? Math.max(0, Math.min(1, value / max)) : 0;
  const text = [typeof amount === 'string' ? amount : null, typeof turns === 'string' ? turns : null].filter(Boolean).join(' · ');
  return (
    <div className={`pbar ${tone} ${className ?? ''}`} data-testid={testId}>
      <div className="pbar-head">
        <span className="pbar-label">{icon}{label}</span>
        {foot === undefined && amount !== undefined && <span className="pbar-amount">{amount}</span>}
        {foot === undefined && turns !== undefined && <span className="pbar-turns">{turns}</span>}
      </div>
      <div className="pbar-track" role="progressbar" aria-label={name ?? (typeof label === 'string' ? label : undefined)} aria-valuemin={0} aria-valuemax={Math.max(0, max)} aria-valuenow={Math.max(0, Math.min(value, max))}
        aria-valuetext={text || undefined}>
        <div className="pbar-fill" style={{ width: `${f * 100}%`, ...(tone === 'bad' || tone === 'paused' ? {} : { background: color }) }} />
      </div>
      {foot !== undefined && (
        <div className="pbar-foot">
          <span className="pbar-foot-left">{foot}</span>
          <span className="pbar-foot-right">
            {amount !== undefined && <span className="pbar-amount">{amount}</span>}
            {amount !== undefined && turns !== undefined && <span className="pbar-amount"> · </span>}
            {turns !== undefined && <span className="pbar-turns">{turns}</span>}
          </span>
        </div>
      )}
    </div>
  );
}
