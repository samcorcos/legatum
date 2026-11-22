import type { CityYields } from '../types';
import { YIELD_LABEL, type YieldKey } from './bits';
import type { AmountUnit } from './amountParts';
import { Quantity } from './Amounts';
import { wonderLineLabel, wonderOfLabel } from '../wonders';

/** How many named sources the hover lists before folding the rest into "Other". */
const SHOWN = 4;

/**
 * A city yield's hover: its biggest sources with amounts, then (for Food) what the population eats and what is left, the same
 * shape as the top-bar hovers. The click-through table beside it lists every source.
 */
export function CityYieldHover({ yields, k, eaten }: { yields: CityYields; k: YieldKey; eaten?: number }) {
  const unit = YIELD_LABEL[k] as AmountUnit;
  const sources = yields.sources
    .map((s) => ({ label: wonderOfLabel(s.label) ? wonderLineLabel(wonderOfLabel(s.label)!) : s.label, amount: s.yields[k] }))
    .filter((s) => s.amount !== 0);
  const ranked = [...sources].sort((a, b) => Math.abs(b.amount) - Math.abs(a.amount));
  const shown = ranked.length > SHOWN ? ranked.slice(0, SHOWN - 1) : ranked;
  const rest = ranked.slice(shown.length).reduce((n, s) => n + s.amount, 0);
  const total = yields.total[k];
  return (
    <div className="science-hover" data-testid={`yield-hover-${k}`}>
      {shown.map((s, i) => (
        <div className="science-hover-row" key={`${s.label}-${i}`}>
          <span>{s.label}</span> <b><Quantity plus value={s.amount} unit={unit} /></b>
        </div>
      ))}
      {ranked.length > shown.length && <div className="science-hover-row"><span>Other</span> <b><Quantity plus value={rest} unit={unit} /></b></div>}
      {sources.length === 0 && <div className="science-hover-row"><span>No sources</span></div>}
      {eaten !== undefined ? <>
        <div className="science-hover-row"><span>Grown</span> <b><Quantity value={total} unit={unit} /></b></div>
        <div className="science-hover-row"><span>Eaten by its population</span> <b><Quantity value={-eaten} unit={unit} /></b></div>
        <div className="science-hover-row science-hover-total"><span>Left each turn</span> <b className={total - eaten < 0 ? 'neg' : undefined}><Quantity plus value={total - eaten} unit={unit} /></b></div>
      </> : <div className="science-hover-row science-hover-total"><span>Total</span> <b><Quantity plus value={total} unit={unit} /></b></div>}
    </div>
  );
}
