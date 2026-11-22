import { BUILDING_KINDS, itemName } from '../data';
import type { PlayerView } from '../types';
import { Quantity } from './Amounts';
import { isDecadenceLine } from './DecadenceNote';
import { wonderOfLabel } from '../wonders';

interface ScienceLine {
  key: string;
  /** The row's text: the amount sits in its own right-aligned column, so no trailing punctuation. */
  label: string;
  amount: number;
}

/** Capital, cities, buildings and any faction bonus, from each city's own yield breakdown (the
 * same per-city sources the city panel's Science hover shows) summed across the empire. */
function scienceLines(view: PlayerView): ScienceLine[] {
  const faction = view.players.find((p) => p.id === view.me)?.faction;
  const buildingNames = new Set(BUILDING_KINDS.map((k) => itemName({ kind: 'Building', building: k }, faction)));
  const mine = view.cities.filter((c) => c.owner === view.me && c.detail);
  let capital = 0, citizens = 0, buildings = 0, bonus = 0, wonders = 0;
  for (const c of mine) {
    for (const s of c.detail!.yields.sources) {
      const v = s.yields.science;
      if (!v || isDecadenceLine(s.label)) continue;
      if (s.label === 'Capital') capital += v;
      else if (s.label === 'Citizens') citizens += v;
      else if (buildingNames.has(s.label)) buildings += v;
      else if (wonderOfLabel(s.label)) wonders += v;
      else bonus += v;
    }
  }
  const lines: ScienceLine[] = [];
  if (capital) lines.push({ key: 'capital', label: 'Capital', amount: capital });
  if (citizens) lines.push({ key: 'cities', label: `${mine.length} ${mine.length === 1 ? 'city' : 'cities'}`, amount: citizens });
  if (buildings) lines.push({ key: 'buildings', label: 'Buildings', amount: buildings });
  if (wonders) lines.push({ key: 'wonders', label: 'Natural wonders', amount: wonders });
  if (bonus) lines.push({ key: 'bonus', label: 'Faction bonus', amount: bonus });
  return lines;
}

/** What decadence takes from the Science of every city (the cities' own "Decadence ... complacency" lines), one row. */
function decadenceLines(view: PlayerView): ScienceLine[] {
  let taken = 0;
  for (const c of view.cities) {
    if (c.owner !== view.me || !c.detail) continue;
    for (const s of c.detail.yields.sources) if (isDecadenceLine(s.label)) taken += s.yields.science;
  }
  return taken ? [{ key: 'decadence', label: 'Decadence: complacency', amount: taken }] : [];
}

/**
 * The top-bar Science hover: its main sources with amounts (legatum-ui: "Yield hovers show where
 * the number comes from"). Sums each city's own yield sources, the same ones the click-through
 * breakdown lists, so the hover and the full breakdown never disagree.
 */
export function ScienceHoverSummary({ view }: { view: PlayerView }) {
  const lines = [...scienceLines(view), ...decadenceLines(view)];
  if (!lines.length) return null;
  // The rows add up to the bar's number: anything the list doesn't name is one "Other" row.
  const total = view.economy.science_per_turn.total;
  const other = total - lines.reduce((n, l) => n + l.amount, 0);
  if (other > 0) lines.push({ key: 'other', label: 'Other', amount: other });
  return (
    <div className="science-hover">
      {lines.map((l) => (
        <div className="science-hover-row" key={l.key}>
          <span>{l.label}</span> <b><Quantity plus value={l.amount} unit="Science" /></b>
        </div>
      ))}
      <div className="science-hover-row science-hover-total">
        <span>Total</span> <b><Quantity plus value={total} unit="Science" /></b>
      </div>
    </div>
  );
}
