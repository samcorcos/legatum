import { useEffect, useMemo, useState } from 'react';
import { useGame } from '../store';
import { empireOfPlayer } from '../empires';
import { Icon } from './bits';
import { eraName, eraNumeral } from './eraResearch';
import './EraNotice.css';

/** How long a rival's era banner stays over the map. */
const ERA_NOTICE_MS = 7000;

export interface EraNoticeLine { key: string; /** 1-based, as the event. */ era: number; first: boolean; text: string }

/**
 * Rival empires entering an era, from the turn's public `EraEntered` events: "India entered the Industrial era first".
 * Your own and your teammates' era-ups have their own cards (`EraUpTakeover`); everyone's stay in the turn report.
 */
function useEraNotices(): EraNoticeLine[] {
  const report = useGame((s) => s.report);
  const seat = useGame((s) => s.seat);
  const view = useGame((s) => (s.views ? s.views[s.seat] : null));
  return useMemo(() => {
    if (!report || !view) return [];
    const team = new Set(view.players.filter((p) => p.team === view.team).map((p) => p.id));
    // The first empire into an era opens the Age, and the Age's score check names it (its opener line), so this notice skips it.
    return (report.events[seat] ?? []).flatMap((e, i) => e.type === 'EraEntered' && !team.has(e.player) && !e.first
      ? [{ key: `${report.turn}:${i}`, era: e.era, first: e.first,
        text: `${empireOfPlayer(view, e.player)} entered the ${eraName(e.era - 1)} era${e.first ? ' first' : ''}` }] : []);
    // The first empire into an era leads; the rest follow.
  }, [report, seat, view]).sort((a, b) => Number(b.first) - Number(a.first));
}

/** A banner over the map for rivals' era-ups; one at a time is enough, the rest wait in the report. */
export function EraNotice() {
  const notices = useEraNotices();
  const [dismissed, setDismissed] = useState<string | null>(null);
  const shown = notices[0] ?? null;
  const key = shown?.key ?? null;
  useEffect(() => {
    if (!key) return;
    const timer = setTimeout(() => setDismissed(key), ERA_NOTICE_MS);
    return () => clearTimeout(timer);
  }, [key]);
  if (!shown || dismissed === shown.key) return null;
  const more = notices.length - 1;
  return <div className={`toast era-notice${shown.first ? ' first' : ''}`} role="status" data-testid="era-notice" data-first={shown.first} onClick={() => setDismissed(shown.key)}>
    <span className="era-notice-numeral" aria-hidden="true">{eraNumeral(shown.era - 1)}</span>
    <span className="era-notice-text"><b>{shown.text}</b>{more > 0 && <small> · {more} more in the turn report</small>}</span>
    <button className="icon-btn" aria-label="Dismiss" onClick={(e) => { e.stopPropagation(); setDismissed(shown.key); }}><Icon name="ui-cross" size={12} /></button>
  </div>;
}
