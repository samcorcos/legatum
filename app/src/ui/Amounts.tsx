import { Fragment, type CSSProperties, type ReactNode } from 'react';
import type { Yields } from '../types';
import { iconUrl, resourceArtUrl } from '../icons';
import { YIELD_COLORS } from '../theme';
import { AMOUNT_ICON, AMOUNT_YIELD, RESOURCE_UNITS, splitAmounts, yieldParts, type AmountUnit } from './amountParts';
import './Amounts.css';

/**
 * One amount of a yield or resource: a signed number and its icon, the way costs read ("146 ⚙").
 * The unit's word stays in the accessible name (a screen-reader-only span, so copied text and text
 * queries still read "+3 Science") and in the hover title. Every screen renders amounts through
 * this, so none hand-writes "+3 Science" (legatum-ui: yield amounts carry their icon everywhere).
 * `tight`: a punctuation mark follows the icon ("+3 [icon]; upgrades"), so the icon's own side space does not read as a stray space before it.
 */
export function Amount({ sign = '', n, unit, tight = false }: { sign?: '' | '+' | '−'; n: number | string; unit: AmountUnit; tight?: boolean }) {
  const named = RESOURCE_UNITS.includes(unit);
  const url = named ? resourceArtUrl(unit, 32) : iconUrl(AMOUNT_ICON[unit]);
  if (!url) return <>{sign}{n} {unit}</>;
  const yieldKey = AMOUNT_YIELD[unit];
  // A strategic resource is named beside its icon ("1 [icon] Iron"): their paintings are small and read alike.
  const style = { WebkitMaskImage: `url("${url}")`, maskImage: `url("${url}")`, backgroundColor: yieldKey ? YIELD_COLORS[yieldKey] : 'currentColor' } as CSSProperties;
  return <span className={`amt amt-${unit.toLowerCase()}${tight && !named ? ' amt-tight' : ''}`} title={unit} data-amount={unit}>
    <span className="amt-n">{sign}{n}</span>{named ? <img className="amt-icon" src={url} alt="" aria-hidden="true" /> : <span className="amt-icon" aria-hidden="true" style={style} />}{named ? <span className="amt-word"> {unit}</span> : <span className="sr-only"> {unit}</span>}
  </span>;
}

/** A number's own sign: "−3" for a loss, "3" for a plain quantity, "+3" for a gain when `plus` is set. */
export function Quantity({ value, unit, plus = false }: { value: number; unit: AmountUnit; plus?: boolean }) {
  return <Amount sign={value < 0 ? '−' : plus && value > 0 ? '+' : ''} n={Math.abs(value)} unit={unit} />;
}

/** Prose with its amounts turned into number + icon: "Upgrade from Library; +4 Science" and the like. */
export function Amounts({ text }: { text: string | null | undefined }) {
  const segments = splitAmounts(text);
  return <>{segments.map((s, i) => {
    if (!('amount' in s)) return <Fragment key={i}>{s.text}</Fragment>;
    const next = segments[i + 1];
    return <Amount key={i} {...s.amount} tight={!!next && 'text' in next && /^[;,.:)]/.test(next.text)} />;
  })}</>;
}

/** A tile's or building's yields as amounts, comma-separated: "+2 Food, +1 Production". */
export function YieldAmounts({ yields, none = null, signed = true }: { yields: Partial<Yields>; none?: ReactNode; signed?: boolean }) {
  const parts = yieldParts(yields, { signed });
  if (!parts.length) return <>{none}</>;
  return <>{parts.map((p, i) => <Fragment key={p.unit}>{i > 0 && ', '}<Amount {...p} tight={i < parts.length - 1} /></Fragment>)}</>;
}
