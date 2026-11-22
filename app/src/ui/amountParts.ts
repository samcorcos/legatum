import type { Yields } from '../types';
import { YIELD_KEYS, YIELD_LABEL, type YieldKey } from './yieldKeys';

// Kept free of asset imports so the splitter loads in plain Node tests (like yieldKeys.ts).

/** What an amount counts: the four yields, Victory Points and the strategic resources. HP and
 * population have no icon in the set, so they stay words. */
export type AmountUnit = 'Food' | 'Production' | 'Gold' | 'Science' | 'VP' | 'Iron' | 'Niter' | 'Coal' | 'Oil' | 'Silicon';

/** The strategic resources as amount units (a resource is named beside its icon: their paintings are small and read alike). */
export const RESOURCE_UNITS: readonly AmountUnit[] = ['Iron', 'Niter', 'Coal', 'Oil', 'Silicon'];

export const AMOUNT_ICON: Record<AmountUnit, string> = {
  Food: 'yield-food', Production: 'yield-production', Gold: 'yield-gold', Science: 'yield-science',
  VP: 'vp', Iron: 'resource-iron', Niter: 'resource-niter', Coal: 'resource-coal', Oil: 'resource-oil', Silicon: 'resource-silicon',
};

/** The yield key behind a unit, for its tint; null for VP and resources (they take the text colour). */
export const AMOUNT_YIELD: Partial<Record<AmountUnit, YieldKey>> = { Food: 'food', Production: 'production', Gold: 'gold', Science: 'science' };

export type AmountPart = { sign: '' | '+' | '−'; n: string; unit: AmountUnit };
export type AmountSegment = { text: string } | { amount: AmountPart };

/**
 * A number, then a unit word: "+3 Science", "−1 Gold" (of "−1 Gold/turn"), "146 Production",
 * "10–20 Gold", "1,500 Science", "12/31 Food", "120 → 90 Gold". The number must stand alone (no letter, "/", "." or digit just
 * before it), so "T10 Gold" is not an amount, and the unit must be the whole word.
 * A unit with no number ("Science victory", "Gold project") never matches.
 */
const AMOUNT = /(?<![\w/.,\-–−+])([+−-]?)(\d[\d,]*(?:[–-]\d+|\/\d[\d,]*| → [+−-]?\d[\d,]*)?)[  ](Food|Production|Gold|Science|VP|Iron|Niter|Coal|Oil|Silicon)\b(?!-)/g;

/** Splits prose into plain text and amounts, in order. */
export function splitAmounts(text: string | null | undefined): AmountSegment[] {
  if (text == null) return [];
  const out: AmountSegment[] = [];
  let last = 0;
  for (const m of text.matchAll(AMOUNT)) {
    const at = m.index ?? 0;
    if (at > last) out.push({ text: text.slice(last, at) });
    const sign = m[1] === '-' || m[1] === '−' ? '−' : m[1] === '+' ? '+' : '';
    out.push({ amount: { sign, n: m[2], unit: m[3] as AmountUnit } });
    last = at + m[0].length;
  }
  if (last < text.length) out.push({ text: text.slice(last) });
  return out;
}

/** True when `text` holds at least one amount. */
export const hasAmounts = (text: string | null | undefined): boolean => splitAmounts(text).some((s) => 'amount' in s);

/** Signed amount for a number: "+3", "−1", "0". */
export const signedAmount = (n: number): '' | '+' | '−' => (n > 0 ? '+' : n < 0 ? '−' : '');

/** The nonzero entries of a Yields object as amount parts, in yield order. */
export function yieldParts(y: Partial<Yields>, opts: { signed?: boolean } = {}): AmountPart[] {
  return YIELD_KEYS.filter((k) => y[k]).map((k) => ({
    sign: opts.signed === false ? (y[k]! < 0 ? '−' : '') : signedAmount(y[k]!),
    n: String(Math.abs(y[k]!)),
    unit: YIELD_LABEL[k] as AmountUnit,
  }));
}
