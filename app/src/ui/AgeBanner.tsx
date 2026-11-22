import { useEffect, useMemo, useState } from 'react';
import { pendingAlarms, useGame } from '../store';
import { cardName } from '../ageCards';
import { empireOfPlayer } from '../empires';
import { Icon } from './bits';
import { eraNumeral } from './eraResearch';
import { FAMILY_LABEL, eraObjectivesFor } from '../eraObjectives';
import './AgeBanner.css';

/** Anything that already owns the screen: a dialog, the era-up takeover, a moment card, an alarm. The banner waits behind it. */
const COVERS = 'dialog[open], [data-game-overlay], .era-takeover, .moment-card';

function useCovered(): boolean {
  const [covered, setCovered] = useState(() => !!document.querySelector(COVERS));
  useEffect(() => {
    const check = () => setCovered(!!document.querySelector(COVERS));
    const observer = new MutationObserver(check);
    observer.observe(document.body, { subtree: true, childList: true, attributes: true, attributeFilter: ['open', 'class'] });
    check();
    return () => observer.disconnect();
  }, []);
  return covered;
}

/**
 * A World Age has begun in a rival's era-up: the first empire entered an era, so its card is open to every empire at once.
 * This is the one announcement of it (the "entered first" toast is this banner). The empire that opened the Age sees its own
 * era-up takeover instead. It waits behind any dialog, takeover, moment card, alarm or the open turn report, and goes when the turn ends.
 */
export function AgeBanner() {
  const view = useGame((s) => (s.views ? s.views[s.seat] : null));
  const report = useGame((s) => s.report);
  const seat = useGame((s) => s.seat);
  const reportOpen = useGame((s) => s.reportOpen);
  const resolving = useGame((s) => s.resolvingToEnd);
  const alarmUp = useGame((s) => pendingAlarms(s).length > 0);
  const covered = useCovered();
  const [dismissed, setDismissed] = useState<string | null>(null);
  const opened = useMemo(() => {
    if (!report || !view) return null;
    const team = new Set(view.players.filter((p) => p.team === view.team).map((p) => p.id));
    const events = report.events[seat] ?? [];
    const i = events.findIndex((e) => e.type === 'EraEntered' && e.first && !team.has(e.player));
    const e = events[i];
    return e && e.type === 'EraEntered' ? { key: `${report.turn}:${i}`, era: e.era, by: empireOfPlayer(view, e.player) } : null;
  }, [report, seat, view]);
  if (!view || view.finished || !opened || dismissed === opened.key || resolving || reportOpen || alarmUp || covered) return null;
  const cards = eraObjectivesFor(view, opened.era);
  if (!cards) return null;
  const close = () => setDismissed(opened.key);
  return <section className="toast age-banner" role="status" data-testid="age-banner" data-era={opened.era} aria-label={`The ${cards.name} Age has begun`}>
    <header>
      <span className="age-banner-numeral" aria-hidden="true">{eraNumeral(opened.era - 1)}</span>
      <h3>The {cards.name} Age has begun</h3>
      <button className="icon-btn" aria-label="Dismiss" onClick={close}><Icon name="ui-cross" size={12} /></button>
    </header>
    <p className="age-banner-sub">{opened.by} entered the {cards.name} era first. Every empire now sees the same card. Score it for 1 VP, kept for good. It never expires.</p>
    <ul>
      {cards.lines.map((l) => <li key={l.slot} data-family={l.family}>
        <span className="age-banner-fam">{FAMILY_LABEL[l.family]}</span>
        <span className="age-banner-text" title={l.text}>{cardName(l.text)}</span>
      </li>)}
    </ul>
    <button className="btn primary age-banner-go" onClick={() => { useGame.getState().set({ vpExpanded: true }); close(); }}>See how you win</button>
  </section>;
}
