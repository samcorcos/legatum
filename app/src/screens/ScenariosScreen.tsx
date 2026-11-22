// The Scenario Lab: short games from one decision point, each built to answer a question only a
// human can (how a person plays a situation, how it feels). The list is a wall of cards, each with
// its painting, the question it asks, its length, your best result and the Hard AI's result from
// the same start. A card opens its briefing; the briefing's one primary action begins it.
// Predictions stay hidden until the scenario ends (see ScenarioResult).
import { useEffect, useMemo, useRef, useState } from 'react';
import { useGame } from '../store';
import { LobbyFrame } from './LobbyScreen';
import { factionColor } from '../theme';
import { factionName } from '../data';
import { Icon } from '../ui/bits';
import { PortraitFrame, PortraitRoundel } from '../ui/premium';
import { eraArt } from '../celebrate/eraArt';
import {
  SOON_SLOTS, aiResultLine, bestResultLine, leadTarget, readRecords, scenarioCatalog, targetText, turnsWord,
  type ScenarioEntry, type ScenarioRecord, type ScenarioTarget, type SoonSlot,
} from '../scenarios/scenarios';
import '../ui/Scenarios.css';

/** The card's painting: the seat's era, washed dark enough for white text. */
function artStyle(era: number | undefined): React.CSSProperties {
  const art = eraArt(era ?? 0);
  return art ? { backgroundImage: `url(${art.small})` } : {};
}

function ResultLines({ entry, record }: { entry: ScenarioEntry; record?: ScenarioRecord }) {
  const f = entry.file;
  const played = !!record && record.revision === f.revision;
  return (
    <span className="lab-results">
      <span className={`lab-you ${played ? 'played' : ''}`}>
        {played && <Icon name={record.bestOutcome === 'Won' || record.bestOutcome === 'Complete' ? 'ui-check' : 'ui-cross'} size={11} />}
        {bestResultLine(f, record)}
      </span>
      <span className="lab-ai">{aiResultLine(f)}</span>
    </span>
  );
}

function ScenarioCard({ entry, record, on, onOpen }: { entry: ScenarioEntry; record?: ScenarioRecord; on: boolean; onOpen: () => void }) {
  const f = entry.file;
  const lab = f.lab!;
  return (
    <button role="option" aria-selected={on} className={`lab-card ${on ? 'on' : ''}`} data-testid={`lab-card-${f.id}`}
      onClick={onOpen}>
      <span className="lab-art" style={artStyle(lab.era)}>
        <PortraitRoundel faction={f.faction} size={54} color={factionColor(f.faction)} className="lab-roundel" />
        <span className="lab-len">About {f.minutes} min · {turnsWord(f.objective.turns)}</span>
        {lab.opponent && <span className="lab-opp">Fixed opponent</span>}
      </span>
      <span className="lab-body">
        <span className="lab-title">{f.title}</span>
        <span className="lab-question">{lab.question}</span>
        <span className="lab-blurb">{lab.blurb}</span>
        <ResultLines entry={entry} record={record} />
      </span>
    </button>
  );
}

function SoonCard({ slot }: { slot: SoonSlot }) {
  return (
    <div className="lab-card soon" role="option" aria-selected={false} aria-disabled data-testid={`lab-soon-${slot.id}`}>
      <span className="lab-art soon-art"><span className="lab-len">Coming soon</span></span>
      <span className="lab-body">
        <span className="lab-title">{slot.title}</span>
        <span className="lab-question">{slot.question}</span>
        <span className="lab-blurb">{slot.blurb}</span>
        <span className="lab-results"><span className="lab-ai">Waiting for {slot.waitingFor}.</span></span>
      </span>
    </div>
  );
}

/** The objective's cities, named with where they are from your nearest city (empty until the
 *  engine has read the scenario's start state). */
function useTargets(entry: ScenarioEntry): ScenarioTarget[] {
  const [targets, setTargets] = useState<{ id: string; list: ScenarioTarget[] }>({ id: '', list: [] });
  useEffect(() => {
    let live = true;
    import('../wasmClient').then((m) => m.scenarioTargets(entry.text)).then((list) => { if (live) setTargets({ id: entry.file.id, list }); }).catch(() => undefined);
    return () => { live = false; };
  }, [entry]);
  return targets.id === entry.file.id ? targets.list : [];
}

function Briefing({ entry, record }: { entry: ScenarioEntry; record?: ScenarioRecord }) {
  const f = entry.file;
  const lab = f.lab!;
  const lead = leadTarget(useTargets(entry), `${f.objective.summary} ${f.briefing}`);
  return (
    <section className="lab-brief" aria-labelledby="scn-detail-title" data-testid="lab-briefing">
      <div className="lab-brief-hero" style={artStyle(lab.era)}>
        <PortraitFrame faction={f.faction} crestColor={factionColor(f.faction)} className="lab-brief-portrait" />
        <div className="lab-brief-head">
          <div className="lbl lab-kicker">You play {factionName(f.faction)}</div>
          <h3 id="scn-detail-title">{f.title}</h3>
          <p className="lab-brief-question">{lab.question}</p>
        </div>
      </div>
      <div className="lab-brief-body">
        {/* Where the target is comes first: at 1000x600 the briefing pushes anything below it out of view. */}
        {lead && (
          <p className="scn-where" data-testid="scenario-briefing-target">
            <b>{targetText(lead)}.</b> It is marked on the map from your first turn.
          </p>
        )}
        <p className="scn-briefing">{f.briefing}</p>
        {lab.opponent && (
          <p className="lab-opponent" data-testid="lab-opponent"><b>Your opponent.</b> {lab.opponent}</p>
        )}
        <div className="scn-objective">
          <div className="lbl">Objective</div>
          <p>{f.objective.summary}</p>
          <div className="scn-facts">
            <span><b>{f.objective.turns}</b> {f.objective.turns === 1 ? 'turn' : 'turns'}</span>
            <span>about <b>{f.minutes}</b> minutes</span>
          </div>
          <ResultLines entry={entry} record={record} />
        </div>
        <p className="scn-note">Your turns are saved on this computer and compared with the Hard AI playing this seat from the same position. Afterwards you are asked how it felt.</p>
      </div>
    </section>
  );
}

/** How many scenario cards lie wholly below the visible part of the list, so a short window says there is more to scroll to ("2 more below"). */
function useCardsBelow(scroller: React.RefObject<HTMLElement | null>, active: boolean, count: number): number {
  const [below, setBelow] = useState(0);
  useEffect(() => {
    const el = scroller.current;
    if (!active || !el) { setBelow(0); return; }
    const measure = () => {
      const edge = el.getBoundingClientRect().bottom;
      setBelow([...el.querySelectorAll<HTMLElement>('.lab-card')].filter((c) => c.getBoundingClientRect().top >= edge - 4).length);
    };
    measure();
    el.addEventListener('scroll', measure, { passive: true });
    window.addEventListener('resize', measure);
    return () => { el.removeEventListener('scroll', measure); window.removeEventListener('resize', measure); };
  }, [scroller, active, count]);
  return below;
}

export function ScenariosScreen() {
  const goto = useGame((s) => s.goto);
  const busy = useGame((s) => s.busy);
  const error = useGame((s) => s.error);
  const startScenario = useGame((s) => s.startScenario);
  const bundled = useMemo(scenarioCatalog, []);
  // Type to filter: every word typed must appear in the card's title, faction, question or blurb.
  const [filter, setFilter] = useState('');
  const words = filter.toLowerCase().split(/\s+/).filter(Boolean);
  const matches = (text: string) => words.every((w) => text.toLowerCase().includes(w));
  const entries = bundled.filter((e) => matches(`${e.file.title} ${factionName(e.file.faction)} ${e.file.lab?.question ?? ''} ${e.file.lab?.blurb ?? ''}`));
  const soon = words.length ? [] : SOON_SLOTS;
  const filterBox = useRef<HTMLInputElement>(null);
  const records = useMemo(readRecords, []);
  const [selected, setSelected] = useState(() => bundled.find((e) => !records[e.file.id])?.file.id ?? bundled[0]?.file.id);
  const [view, setView] = useState<'list' | 'brief'>('list');
  const scroller = useRef<HTMLDivElement>(null);
  const below = useCardsBelow(scroller, view === 'list', entries.length + soon.length);
  const entry = entries.find((e) => e.file.id === selected) ?? entries[0];
  const start = () => {
    if (entry && !busy) void startScenario(entry);
  };
  const back = () => (view === 'brief' ? setView('list') : goto('title'));
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.target instanceof HTMLTextAreaElement) return;
      const typing = e.target instanceof HTMLInputElement;
      // In the filter box Left and Right move the caret; Escape clears the filter before it goes back.
      if (typing && (e.key === 'ArrowLeft' || e.key === 'ArrowRight')) return;
      if (typing && e.key === 'Escape' && filter) { setFilter(''); return; }
      // A letter typed anywhere on the list goes to the filter box.
      if (view === 'list' && !typing && e.key.length === 1 && !e.metaKey && !e.ctrlKey && !e.altKey && e.key !== ' ') filterBox.current?.focus();
      const i = entries.findIndex((x) => x.file.id === selected);
      if (view === 'list' && ['ArrowDown', 'ArrowUp', 'ArrowLeft', 'ArrowRight'].includes(e.key)) {
        e.preventDefault();
        // Up and Down move a row (as many cards as share the first card's top edge), Left and Right one card.
        const cards = [...document.querySelectorAll<HTMLElement>('.lab-card[data-testid^="lab-card-"]')];
        const cols = Math.max(1, cards.filter((c) => c.offsetTop === cards[0]?.offsetTop).length);
        const delta = e.key === 'ArrowRight' ? 1 : e.key === 'ArrowLeft' ? -1 : e.key === 'ArrowDown' ? cols : -cols;
        const next = entries[Math.min(entries.length - 1, Math.max(0, i + delta))];
        if (next) setSelected(next.file.id);
        document.querySelector(`[data-testid="lab-card-${next?.file.id}"]`)?.scrollIntoView({ block: 'nearest' });
      } else if (e.key === 'Enter' && !(e.target instanceof HTMLButtonElement && !e.target.classList.contains('lab-card'))) {
        e.preventDefault();
        if (view === 'list') setView('brief');
        else start();
      } else if (e.key === 'Escape') back();
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  });

  const dock = entry && (view === 'list' ? (
    <>
      <div className="lobby-dock-status scn-dock-status" role="status">
        <span className="muted">{entry.file.title} · about {entry.file.minutes} minutes</span>
      </div>
      <button className="btn big primary lobby-go" onClick={() => setView('brief')} data-testid="lab-open">
        Read the briefing <kbd>Return</kbd>
      </button>
    </>
  ) : (
    <>
      <div className="lobby-dock-status scn-dock-status" role="status">
        <span className="muted">{turnsWord(entry.file.objective.turns)} · about {entry.file.minutes} minutes</span>
      </div>
      <button className="btn big primary lobby-go" disabled={!!busy} onClick={start}>
        {busy ?? <>Begin {entry.file.title} <kbd>Return</kbd></>}
      </button>
    </>
  ));

  return (
    <LobbyFrame title={view === 'brief' && entry ? entry.file.title : 'Scenario Lab'} onBack={back} backDisabled={!!busy} error={error} dock={dock}>
      <div className="scn-scroll" ref={scroller}>
        {bundled.length === 0 ? (
          <p className="scn-intro">No scenarios are bundled with this build.</p>
        ) : view === 'brief' && entry ? (
          <Briefing entry={entry} record={records[entry.file.id]} />
        ) : (
          <>
            <p className="scn-intro">Short games built to answer one question each, the kind only a person can answer. Play one, then say how it felt.</p>
            <div className="lab-filter">
              <input ref={filterBox} autoFocus type="search" value={filter} onChange={(e) => setFilter(e.target.value)}
                placeholder="Type to filter by name, people or question" aria-label="Filter scenarios" data-testid="lab-filter" />
              {words.length > 0 && <span className="lab-filter-count" data-testid="lab-filter-count">{entries.length} of {bundled.length}</span>}
            </div>
            {words.length > 0 && entries.length === 0 && (
              <p className="scn-intro" data-testid="lab-filter-empty">No scenario matches "{filter.trim()}". Press Esc to clear the filter.</p>
            )}
            <div className="lab-grid" role="listbox" aria-label="Scenarios">
              {entries.map((e) => (
                <ScenarioCard key={e.file.id} entry={e} record={records[e.file.id]} on={e.file.id === entry?.file.id}
                  onOpen={() => { setSelected(e.file.id); setView('brief'); }} />
              ))}
              {soon.map((slot) => <SoonCard key={slot.id} slot={slot} />)}
            </div>
          </>
        )}
      </div>
      {/* Below the scroller, never over a card: the pill has a row of its own. */}
      {view === 'list' && below > 0 && <div className="lab-more-dock">
        <button type="button" className="lab-more" data-testid="lab-more" onClick={() => scroller.current?.scrollBy({ top: scroller.current.clientHeight * 0.8, behavior: 'smooth' })}>
          {below} more below <Icon name="ui-chevron-down" size={12} />
        </button>
      </div>}
    </LobbyFrame>
  );
}
