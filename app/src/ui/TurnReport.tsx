import { CALENDAR_NOTE, calendarLabel } from '../data';
import { useCelebration } from '../celebrate/MomentLane';
import { useEffect, useMemo, useRef, useState } from 'react';
import { stepReport, toggleLastTurnReport, useGame, viewedReport } from '../store';
import {
  CATEGORY_LABEL, contextFromViews, filterLines, groupLines, lineSegments, toneCounts,
  type EventCategory, type ReportFilter, type ReportLine, type Tone,
} from '../format';
import { EmpireMark } from './EmpireIdentity';
import { Icon, Kbd, ToneIcon } from './bits';
import { CATEGORY_ICON } from '../reportIcons';
import { RingMedallion } from './ringMedallion';
import { StarDivider } from './premium';
import { useCoarsePointer } from './useCoarsePointer';
import { Amounts } from './Amounts';
import { RankMark } from './RankMark';
import './why.css';

/** Never colour alone: each tone has a mark and words for screen readers. */
const TONE_MARK: Record<Tone, { label: string }> = {
  good: { label: 'Good for you' },
  bad: { label: 'Bad for you' },
  neutral: { label: '' },
};

/** "1 bad · 1 good · 2 other", or just the count when nothing is good or bad. */
export function chipCounts(lines: ReportLine[]): { text: string; tone: Tone }[] {
  const c = toneCounts(lines);
  if (!c.bad && !c.good) return [{ text: String(c.neutral), tone: 'neutral' }];
  return [
    ...(c.bad ? [{ text: `${c.bad} bad`, tone: 'bad' as const }] : []),
    ...(c.good ? [{ text: `${c.good} good`, tone: 'good' as const }] : []),
    ...(c.neutral ? [{ text: `${c.neutral} other`, tone: 'neutral' as const }] : []),
  ];
}

/**
 * The always-visible way back to the turn report (This Turn panel, phone sheet; L). It shows how
 * the last turn went for you, so a hit you missed while the report flashed by is still findable.
 */
export function LastTurnButton({ className = '' }: { className?: string }) {
  const report = useGame((s) => s.report);
  const open = useGame((s) => s.reportOpen);
  const seat = useGame((s) => s.seat);
  const touch = useCoarsePointer();
  const views = useGame((s) => s.views);
  const counts = useMemo(() => {
    if (!report || !views) return null;
    const ctx = contextFromViews(null, views[seat]);
    return toneCounts(filterLines(report.lines[seat] ?? [], 'team', ctx));
  }, [report, views, seat]);
  return (
    <button className={`btn quiet last-turn ${className}`} disabled={!report} aria-expanded={open}
      onClick={toggleLastTurnReport}
      title={report ? `${open ? 'Close' : 'Open'} the turn ${report.turn} report (L)` : 'No report yet: one appears when the next turn resolves'}>
      <span className="last-turn-label">Last turn{report ? ` · T${report.turn}` : ''}</span>
      {counts && (counts.bad > 0 || counts.good > 0) && <span className="last-turn-counts">
        {counts.bad > 0 && <span className="chip-count bad"><ToneIcon tone="bad" />{counts.bad}<span className="sr-only"> bad</span></span>}
        {counts.good > 0 && <span className="chip-count good"><ToneIcon tone="good" />{counts.good}<span className="sr-only"> good</span></span>}
      </span>}
      {!touch && <Kbd>L</Kbd>}
    </button>
  );
}

/**
 * The coach's question, at the top of the latest report: a waiting one reopens its card (Esc only
 * hides it), and the last one he skipped can still be answered (user, 2026-11-21).
 */
export function CoachLine() {
  const waiting = useGame((s) => s.whyQuestion);
  const recent = useGame((s) => s.whyRecent);
  const showWhy = useGame((s) => s.showWhy);
  const question = waiting ?? recent?.question;
  if (!question) return null;
  const status = waiting ? 'Waiting for your answer.' : recent!.outcome === 'skipped' ? 'You skipped it.' : 'You answered it.';
  const action = waiting ? 'Answer' : recent!.outcome === 'skipped' ? 'Answer now' : null;
  return (
    <div className="report-group report-coach" data-category="coach" data-testid="report-coach">
      <div className="report-cat"><span className="why-mark" aria-hidden="true">?</span><span>Coach question</span></div>
      <div className="rline neutral">
        <p className="report-coach-text">{question.text}</p>
        <div className="report-coach-foot">
          <span className="muted">{status}</span>
          {action && <button className="btn quiet report-coach-open" onClick={showWhy} data-testid="report-coach-open">{action}</button>}
        </div>
      </div>
    </div>
  );
}

/** Drawer with a turn's events, grouped and filterable, steppable through recent turns; click a line to jump there. */
export function TurnReport({ embedded = false }: { embedded?: boolean }) {
  const report = useGame(viewedReport);
  const latest = useGame((s) => s.report);
  const history = useGame((s) => s.reportHistory);
  const reportFocus = useGame((s) => s.reportFocus);
  const requested = useGame((s) => s.reportOpen);
  const { holding } = useCelebration();
  const open = requested && !holding;
  const seat = useGame((s) => s.seat);
  const views = useGame((s) => s.views);
  const set = useGame((s) => s.set);
  const jumpTo = useGame((s) => s.jumpTo);
  const touch = useCoarsePointer();
  const [filter, setFilter] = useState<ReportFilter>('team');
  const [hidden, setHidden] = useState<Set<EventCategory>>(new Set(['movement']));
  const [expanded, setExpanded] = useState<number | null>(null);
  // Phone sheet: the filters fold into one control so the events lead (playtest-6 H3).
  const [filtersOpen, setFiltersOpen] = useState(false);
  const closeButton = useRef<HTMLButtonElement>(null);
  useEffect(() => {
    if (!open || !reportFocus) return;
    setFilter('team');
    setHidden((old) => { const next = new Set(old); next.delete(reportFocus); return next; });
    const frame = requestAnimationFrame(() => document.querySelector(`.report-group[data-category="${reportFocus}"]`)?.scrollIntoView({ block: 'nearest', behavior: 'instant' }));
    return () => cancelAnimationFrame(frame);
  }, [open, reportFocus, report]);
  // A closed report reopens on the latest turn, and its arrow leaves the map.
  useEffect(() => {
    if (!requested) set({ reportTurn: null, reportArrow: null });
  }, [requested, set]);
  useEffect(() => { setExpanded(null); }, [report]);
  useEffect(() => {
    if (!open || !report) return;
    // The report is a non-modal drawer: it opens after most turns without taking focus, so arrows
    // keep panning and Return keeps selecting decisions. Opened from the game menu (a keyboard path), it
    // takes focus and gives it back.
    const opener = document.activeElement as HTMLElement | null;
    if (!opener?.closest('.menu')) return;
    closeButton.current?.focus();
    return () => { if (opener?.isConnected) opener.focus({ preventScroll: true }); };
  }, [open, !!report]); // eslint-disable-line react-hooks/exhaustive-deps

  const ctx = useMemo(() => (views ? contextFromViews(null, views[seat]) : null), [views, seat]);
  const lines = report && ctx ? filterLines(report.lines[seat] ?? [], filter, ctx) : [];
  const groups = groupLines(lines);
  if (!open || !report) return null;
  const all = new Map(groupLines(report.lines[seat] ?? []).map((g) => [g.category, g.lines]));
  const index = history.findIndex((r) => r.turn === report.turn);
  const older = index > 0 ? history[index - 1] : null;
  const newer = index >= 0 && index < history.length - 1 ? history[index + 1] : null;
  const isLatest = report === latest;
  const close = () => set({ reportOpen: false });
  const kicker = <div className="report-kicker">{isLatest ? 'Last turn' : `${(latest?.turn ?? report.turn) - report.turn} turn${(latest?.turn ?? report.turn) - report.turn === 1 ? '' : 's'} ago`}</div>;
  const show = (l: ReportLine) => {
    if (!l.at) return;
    jumpTo(l.at, true);
    if (l.category === 'combat') set({ reportArrow: { from: l.from ?? null, to: l.at, seq: Date.now() } });
  };

  return (
    <aside className="report" data-dismissible="report" data-overlay-slot={embedded ? undefined : 'story-report'} aria-label="Turn report" data-report-turn={report.turn} onKeyDown={(e) => {
      if (e.key === 'Escape') { e.preventDefault(); e.stopPropagation(); close(); }
    }}>
      <div className={`report-head${embedded ? ' compact' : ''}`}>
        <div className="report-head-main">
          {!embedded && kicker}
          {!embedded && <h3>What happened</h3>}
          <div className="report-stepper" role="group" aria-label="Turn">
            <button className="icon-btn report-step" disabled={!older} onClick={() => stepReport(-1)}
              aria-label={older ? `Previous turn: turn ${older.turn}` : 'No earlier report'} title={older ? `Turn ${older.turn} ([)` : `Reports are kept for the last ${history.length} turns`}>◀</button>
            <span className="report-turn" title={CALENDAR_NOTE}>Turn {report.turn} · {calendarLabel(report.turn)}{embedded && kicker}</span>
            <button className="icon-btn report-step" disabled={!newer} onClick={() => stepReport(1)}
              aria-label={newer ? `Next turn: turn ${newer.turn}` : 'This is the latest report'} title={newer ? `Turn ${newer.turn} (])` : 'Latest report'}>▶</button>
          </div>
        </div>
        {embedded && <button className="btn quiet report-filter-toggle" aria-expanded={filtersOpen} aria-controls="report-filters"
          onClick={() => setFiltersOpen(!filtersOpen)}>
          Filter <span className="muted">{filter === 'mine' ? 'Mine' : filter === 'team' ? 'Team' : 'Everyone'}{[...all.keys()].some((c) => hidden.has(c)) ? ` · ${[...all.keys()].filter((c) => hidden.has(c)).length} hidden` : ''}</span> <Icon name={filtersOpen ? 'ui-chevron-up' : 'ui-chevron-down'} size={12} className="ui-mark" />
        </button>}
        {!embedded && <button ref={closeButton} className="icon-btn" onClick={close} aria-label="Close report" title="Close (Esc, or L)">
          <Icon name="ui-cross" size={14} />
        </button>}
      </div>
      <div className="report-filters" id="report-filters" hidden={embedded && !filtersOpen}>
        <div className="seg small">
          {(['mine', 'team', 'all'] as ReportFilter[]).map((f) => (
            <button key={f} className={filter === f ? 'on' : ''} aria-pressed={filter === f} onClick={() => setFilter(f)}>
              {f === 'mine' ? 'Mine' : f === 'team' ? 'Team' : 'Everyone'}
            </button>
          ))}
        </div>
        <div className="chips">
          {[...all.keys()].map((c) => (
            <button
              key={c}
              className={`chip ${hidden.has(c) ? '' : 'on'}`}
              aria-pressed={!hidden.has(c)}
              data-category={c}
              onClick={() => {
                const n = new Set(hidden);
                if (n.has(c)) n.delete(c);
                else n.add(c);
                setHidden(n);
              }}
            >
              <span aria-hidden="true" className="chip-icon"><RingMedallion icon={CATEGORY_ICON[c]} tone="neutral" size={18} /></span>
              {CATEGORY_LABEL[c]}{' '}
              {chipCounts(all.get(c)!).map((p, i) => <span key={i} className={`chip-count ${p.tone}`}>{i > 0 && <span className="muted"> · </span>}{p.tone !== 'neutral' && <ToneIcon tone={p.tone} />}{p.text}</span>)}
            </button>
          ))}
        </div>
      </div>
      <div className="report-body">
        {isLatest && <CoachLine />}
        {groups.filter((g) => !hidden.has(g.category)).length === 0 && (
          <div className="muted report-empty">Nothing to report with these filters.</div>
        )}
        {groups
          .filter((g) => !hidden.has(g.category))
          .map((g, gi) => (
            <div key={g.category} className="report-group" data-category={g.category}>
              {gi > 0 && <StarDivider className="report-sd" />}
              <div className="report-cat">
                <RingMedallion icon={CATEGORY_ICON[g.category]} tone="neutral" size={22} />
                <span>{CATEGORY_LABEL[g.category]}</span>
              </div>
              {g.lines.map((l, i) => {
                const id = lines.indexOf(l) * 1000 + i;
                const isOpen = expanded === id;
                const detailsId = `report-details-${id}`;
                const Line = l.at || l.details.length || l.reopenTech || l.reopenEra ? 'button' : 'div';
                const mark = TONE_MARK[l.tone];
                return (
                  <div
                    key={i}
                    className={`rline ${l.tone} w${l.weight} ${l.at || l.reopenTech || l.reopenEra ? 'jump' : ''}`}
                    data-tone={l.tone}
                  >
                    <Line className="rline-text"
                      aria-expanded={l.details.length ? isOpen : undefined}
                      aria-controls={l.details.length ? detailsId : undefined}
                      title={l.reopenTech ? 'Show the unlock card again' : l.reopenEra ? (l.reopenEra.age != null ? 'Show the score check again' : 'Show the era card again') : l.at ? (l.from ? 'Show on the map, with an arrow from the attacker' : 'Show on the map') : undefined}
                      onClick={() => {
                        if (l.reopenTech) { useGame.getState().reopenUnlock(l.reopenTech); return; }
                        if (l.reopenEra) { useGame.getState().reopenEra(l.reopenEra.era, l.reopenEra.first, l.reopenEra.age); return; }
                        show(l);
                        if (l.details.length) setExpanded(isOpen ? null : id);
                      }}
                    >
                      <span className="rline-main">
                        <span className={`rline-tone ${l.tone}`} aria-hidden="true"><RingMedallion icon={l.icon ?? CATEGORY_ICON[l.category]} tone={l.tone} size={26} /></span>
                        {mark.label && <span className="sr-only">{mark.label}: </span>}
                        {l.empire !== undefined && views && <EmpireMark view={views[seat]} team={l.empire} size={18} />}
                        {l.promotion && <RankMark level={l.promotion.level} faction={l.promotion.faction} compact />}
                        <span>
                          {lineSegments(l).map((p, k) => p.tone
                            ? <b key={k} className={`hp-delta ${p.tone}`}>{p.text}</b>
                            : <span key={k}><Amounts text={p.text} /></span>)}
                        </span>
                      </span>
                      {l.details.length > 0 && <span className="rline-more">{isOpen ? '−' : '+'}</span>}
                    </Line>
                    {l.at && <button className="link rline-show-map" onClick={() => show(l)}>Show on map</button>}
                    {l.reopenTech && <button className="link rline-show-card" onClick={() => useGame.getState().reopenUnlock(l.reopenTech!)}>Show card</button>}
                    {l.reopenEra && <button className="link rline-show-card" onClick={() => useGame.getState().reopenEra(l.reopenEra!.era, l.reopenEra!.first, l.reopenEra!.age)}>{l.reopenEra.age != null ? 'Show score check' : 'Show card'}</button>}
                    {l.openLedger && <button className="link rline-show-card" onClick={() => set({ vpExpanded: true })}>See how you win</button>}
                    {l.reopenHold && <button className="link rline-show-card" onClick={() => set({ holdReopen: l.reopenHold! })}>Show card</button>}
                    {l.openInboxTeam !== undefined && <button className="btn primary" onClick={() => set({
                      diplomacyOpen: true, diplomacyTeam: l.openInboxTeam!, reportOpen: false,
                    })}>Open inbox</button>}
                    {l.details.length > 0 && (
                      <div id={detailsId} className="rline-details" hidden={!isOpen}>
                        {l.details.map((d, k) => (
                          <div key={k}><Amounts text={d} /></div>
                        ))}
                      </div>
                    )}
                  </div>
                );
              })}
            </div>
          ))}
      </div>
      {!touch && <div className="report-foot muted">Reopen any time: <b>Last turn</b> in This Turn, or <Kbd>L</Kbd></div>}
    </aside>
  );
}
