import { useMemo } from 'react';
import { useGame } from '../store';
import { citiesUnderAttack, type CityAlarm } from '../siege';

const NONE: CityAlarm[] = [];

/**
 * The viewer's cities under attack (or about to be): from the fogged view, plus last turn's visible
 * events while the report is still the latest turn's (a loaded game has none; the view's own
 * `attacked_last_turn` covers that).
 */
export function useCityAlarms(): CityAlarm[] {
  const view = useGame((s) => s.views?.[s.seat]);
  const report = useGame((s) => s.report);
  const seat = useGame((s) => s.seat);
  return useMemo(() => {
    if (!view) return NONE;
    const hits = report && report.turn === view.turn - 1 ? report.hits?.[seat] ?? null : null;
    const found = citiesUnderAttack(view, hits);
    return found.length ? found : NONE;
  }, [view, report, seat]);
}
