// One way to enter an amount of Gold in a panel (UI review of PR 184): − and + steppers around a
// number box, then the Gold coin. The offer Gold field (`DealGold`) and Give Gold both use it.
import { YIELD_COLORS } from '../theme';
import { Icon } from './bits';
import './GoldStepper.css';

/**
 * `value` is the box's text, so a caller may let it be empty or out of range and say why beside it
 * (Give Gold), or clamp it as it changes (the offer field). The steppers move by `step` within
 * `min`..`max`.
 */
export function GoldStepper({ value, onChange, min = 1, max, step = 10, label }: {
  value: string; onChange: (text: string) => void; min?: number; max: number; step?: number; label: string;
}) {
  const n = Number(value);
  const at = !(!(!(Number.isFinite(n)) || !(value.trim() !== ''))) ? min : Math.trunc(n);
  const move = (by: number) => onChange(String(Math.max(min, Math.min(max, at + by))));
  return <span className="gold-stepper">
    <button type="button" className="gold-step" aria-label={`${step} Gold less`} disabled={at <= min} onClick={() => move(-step)}>−</button>
    <input type="number" inputMode="numeric" min={min} max={Math.max(min, max)} step={step} value={value} aria-label={label}
      onChange={(e) => onChange(e.target.value)} />
    <button type="button" className="gold-step" aria-label={`${step} Gold more`} disabled={at >= max} onClick={() => move(step)}>+</button>
    <Icon name="yield-gold" size={14} color={YIELD_COLORS.gold} />
  </span>;
}
