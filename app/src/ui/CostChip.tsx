import type { ReactNode } from 'react';
import { Icon } from './bits';
import type { DiplomaticAction, PlayerView } from '../types';
import './CostChip.css';

/** What a pool loses: a yield or a strategic resource. */
type SpendUnit = 'Gold' | 'Science' | 'Iron' | 'Niter' | 'Coal' | 'Oil' | 'Silicon' | 'population';

export type Spend = {
  unit: SpendUnit;
  /** What pressing the button spends (a positive number). */
  amount: number;
  /** The pool now and after this spend, when the player's own pool is known. */
  from?: number;
  to?: number;
  /** A recurring cost: `amount` every turn (an Alliance's upkeep), shown as "−1 Gold/turn". */
  perTurn?: boolean;
};

const ICON: Partial<Record<SpendUnit, string>> = { Gold: 'yield-gold', Science: 'yield-science', Iron: 'resource-iron', Niter: 'resource-niter', Coal: 'resource-coal', Oil: 'resource-oil', Silicon: 'resource-silicon' };

/** A pool below 0 is debt. */
const inDebt = (s: Spend) => s.to !== undefined && s.to < 0;
const signed = (n: number) => n < 0 ? `−${-n}` : `${n}`;
const afterText = (s: Spend) => s.from === undefined || s.to === undefined ? null
  : `${signed(s.from)} → ${signed(s.to)}${inDebt(s) ? ', debt' : ''}`;

/** "Costs 30 Gold (120 → 90) and 1 Gold per turn", for screen readers and tooltips. */
function spendText(spends: Spend[], when?: string): string {
  const parts = spends.filter((s) => s.amount > 0).map((s) => `${s.amount} ${s.unit}${s.perTurn ? ' per turn' : ''}${afterText(s) ? ` (${afterText(s)})` : ''}`);
  return parts.length ? `Costs ${parts.join(' and ')}${when ? ` ${when}` : ''}` : '';
}

/**
 * G16 house rule: every action that spends something shows it on (or right beside) its button, in
 * the spend color, with the pool's after-state ("−30 Gold · 120 → 90"), so a player sees at a
 * glance, without reading, that pressing costs something. A spend into debt says so in the chip.
 * Free actions render nothing. `when` qualifies a deferred cost ("if accepted").
 */
export function CostChip({ spends, when, lead, id, children }: { spends: Spend[]; when?: string; lead?: string; id?: string; children?: ReactNode }) {
  const shown = spends.filter((s) => s.amount > 0);
  if (!shown.length) return null;
  const debt = shown.some(inDebt);
  // One chip, one sign: good news ("Free to sign") is its own green chip, never inside the red cost pill.
  return <>{lead && <span className="cost-lead-chip" data-lead-chip>{lead}</span>}<span className={`cost-chip${debt ? ' debt' : ''}`} id={id} data-cost-chip>
    {shown.map((s) => <span className="cost-part" key={`${s.unit}${s.perTurn ? '/turn' : ''}`} aria-hidden="true">
      <span className="cost-amount">−{s.amount}{ICON[s.unit]
        ? <Icon name={ICON[s.unit]!} size={12} color="currentColor" fallback={<span className="cost-unit"> {s.unit}</span>} />
        : <span className="cost-unit"> {s.unit}</span>}{s.perTurn && <span className="cost-unit">/turn</span>}</span>
      {afterText(s) && <span className="cost-after">{afterText(s)}</span>}
    </span>)}
    {when && <span className="cost-when" aria-hidden="true">{when}</span>}
    {children}
    <span className="sr-only">{spendText(shown, when)}</span>
  </span></>;
}

/** A "For you" line: − for what you give up (red), + for what you gain (green). */
export function ForYouLine({ tone, children }: { tone: 'cost' | 'gain' | 'note'; children: ReactNode }) {
  return <li className={`for-you-line ${tone}`}>
    <span className="for-you-mark" aria-hidden="true">{tone === 'cost' ? '−' : tone === 'gain' ? '+' : '·'}</span>
    {tone !== 'note' && <span className="sr-only">{tone === 'cost' ? 'Cost: ' : 'Gain: '}</span>}
    <span>{children}</span>
  </li>;
}

/**
 * What a diplomatic action spends, all in Gold: the price of leaving an Alliance early
 * with the treasury's after-state, and an Alliance's upkeep per turn.
 * A proposal pays on acceptance, turns from now: no after-state against today's treasury.
 */
export function diplomacySpends(action: DiplomaticAction, view: PlayerView): Spend[] {
  const spends: Spend[] = [];
  const gold = view.economy?.gold;
  if (action.cost > 0) spends.push(action.order.type === 'ProposeTreaty' || gold === undefined
    ? { unit: 'Gold', amount: action.cost }
    : { unit: 'Gold', amount: action.cost, from: gold, to: gold - action.cost });
  if ((action.upkeep ?? 0) > 0) spends.push({ unit: 'Gold', amount: action.upkeep!, perTurn: true });
  return spends;
}
