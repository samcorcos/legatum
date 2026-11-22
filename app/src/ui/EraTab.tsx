import { eraName } from '../data';
import { ERA_METAL, ROMAN } from '../map/unitEra';

/** The era tab a military miniature wears on its base (map/unitEra.ts `drawEraPlaque`), for panels and cards: the numeral on the era's metal. */
export function EraTab({ era, named = false }: { era: number; named?: boolean }) {
  const e = Math.max(0, Math.min(6, era));
  const dark = e === 0 || e === 1 || e === 3 || e === 5;
  return (
    <span className="era-tab-wrap" title={`${eraName(e)} era`}>
      <span className="era-tab" style={{ background: ERA_METAL[e], color: dark ? 'var(--paper, #fbf6ea)' : 'var(--ink)' }} aria-hidden="true">{ROMAN[e]}</span>
      {named ? <span className="era-tab-name">{eraName(e)} era</span> : <span className="sr-only">{eraName(e)} era</span>}
    </span>
  );
}
