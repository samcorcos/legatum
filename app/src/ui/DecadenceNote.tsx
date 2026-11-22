import { DECADENCE_FOOD_RULE, decadenceRules } from '../data';
import { useGame } from '../store';

/** A source or breakdown line the core labels "Decadence −11% of Production: sloth" (or "of interest: luxury" for Switzerland's interest). */
export const isDecadenceLine = (label: string) => label.startsWith('Decadence');

/** The percent in a "Decadence −11% of ..." label, so a note can still name the rate when the view is unavailable. */
export const decadenceLinePercent = (label: string) => Number(/Decadence\s*[−-](\d+)%/.exec(label)?.[1] ?? 0);

/**
 * The one explanation of the Decadence lines (legatum-ui pillar 5: no hidden math), printed below any breakdown that
 * has one. Numbers are the viewer's own `decadence_percent` and `lead_percent`; `fallback` is the rate read from a line's
 * label when the view has none.
 */
export function DecadenceNote({ percent, lead }: { percent: number; lead: number }) {
  return <div className="bd-note decadence-note" data-testid="decadence-note">
    <b>Decadence:</b> your Power is {lead}% above the average empire&rsquo;s, so your Production, Gold and Science are cut by {percent}% and your cities grow {percent}% slower. The rate is in proportion to your lead: the further ahead you are, the more your people grow complacent. {DECADENCE_FOOD_RULE}
  </div>;
}

/** Where Food is concerned: no line in the yield breakdown, so a one-line pointer to the growth bar while you have decadence. */
export function FoodDecadencePointer() {
  const percent = useGame((s) => { const v = s.views?.[s.seat]; return v && decadenceRules(v.rule_experiments).decadenceSlope > 0 ? v.players.find((p) => p.id === v.me)?.decadence_percent ?? 0 : 0; });
  return percent > 0 ? <div className="bd-note" data-testid="decadence-food-pointer">Decadence also slows growth: see the city&rsquo;s growth bar.</div> : null;
}

/** The note for the viewer's own empire, or nothing while it has no decadence. `fallback` is a rate read from a line label. */
export function OwnDecadenceNote({ fallback = 0 }: { fallback?: number }) {
  const enabled = useGame((s) => { const v = s.views?.[s.seat]; return decadenceRules(v?.rule_experiments).decadenceSlope > 0; });
  const mine = useGame((s) => { const v = s.views?.[s.seat]; return v?.players.find((p) => p.id === v.me); });
  const percent = mine?.decadence_percent || fallback;
  if (!enabled || !percent) return null;
  return <DecadenceNote percent={percent} lead={mine?.lead_percent ?? 0} />;
}
