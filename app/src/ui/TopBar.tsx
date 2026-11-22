import { useLayoutEffect, useRef, type RefObject } from 'react';
import { CounterDelta } from '../celebrate/MomentLane';
import { useGame } from '../store';
import { CALENDAR_NOTE, TECH_NAME, calendarLabel, decadenceRules } from '../data';
import { effectiveResearchQueue, pendingResearchQueue } from '../orders';
import { firstScienceForecast } from '../research';
import { eraName, eraNumeral, queueSchedule } from './eraResearch';
import { BreakdownTable, Icon, Tip, YieldIcon } from './index';
import { signed } from './bits';
import { GameMenu } from './GameMenu';
import { SupplyChips } from './SupplyChips';
import { Wordmark } from '../brand/ornaments';
import { itemName } from '../data';
import { bankruptcyWarning, FORECAST_ASSUMPTIONS } from './forecastWarnings';
import { usePhoneLayout } from './usePhoneLayout';
import { ScienceHoverSummary } from './ScienceHover';
import { Amount, Amounts, Quantity } from './Amounts';
import { VpChip } from './ObjectiveSheet';
import { CitiesButton } from './CitiesOverview';
import { HoardReadout, TombReadout } from './FactionGoalProgress';
import { LevyCounter } from './Levy';
import { DecadenceNote } from './DecadenceNote';
import { pendingSpend } from './productionPurchases';
import { TrainingLine, TrainingTip } from './Training';

/**
 * Desktop and tablet top bar compaction, in order: each step hides more secondary text (all of it
 * stays in the tooltips) until the totals fit and the research cell shows its text in full. The
 * Egypt's Royal Tomb readout goes to its short form ("6/15") early, before the brand or the Gold income go. The
 * first step drops the Technologies button (the research cell opens the same dialog); the resource names go late,
 * and the painted resource art keeps the strategic resources apart after them. The "Pop." and "Armed" words go first, and shortage chips keep their names at every size. The
 * research cell truncates only after the last step. Measured, not media queries, because what the
 * bar holds varies (0 to 6 supply chips, the treasury warning, a one- or two-line research plan).
 */
const TOP_BAR_COMPACTION = ['counts', 'nav', 'cities', 'tomb', 'warning', 'tight', 'score', 'era', 'brand', 'incomes', 'hoard', 'art', 'names', 'label', 'eraname', 'erachip'] as const;

function fitsTopBar(bar: HTMLElement) {
  const stats = bar.querySelector<HTMLElement>('.tb-stats');
  if (stats && stats.scrollWidth > stats.clientWidth + 1) return false;
  return [...bar.querySelectorAll<HTMLElement>('.research-chip b, .research-chip-plan > span')].every((e) => e.scrollWidth <= e.clientWidth);
}

function useTopBarCompaction(ref: RefObject<HTMLElement | null>, phone: boolean) {
  // After every render: the content decides how much room is needed.
  useLayoutEffect(() => {
    const bar = ref.current;
    if (!bar) return;
    const fit = () => {
      if (phone) { delete bar.dataset.compact; return; }
      let steps = 0;
      bar.dataset.compact = '';
      while (!fitsTopBar(bar) && steps < TOP_BAR_COMPACTION.length) bar.dataset.compact = TOP_BAR_COMPACTION.slice(0, ++steps).join(' ');
    };
    fit();
    if (typeof ResizeObserver === 'undefined') return;
    const observer = new ResizeObserver(fit);
    observer.observe(bar);
    let live = true;
    document.fonts?.ready.then(() => { if (live) fit(); });
    return () => { live = false; observer.disconnect(); };
  });
}

/** The turn limit joins the turn label only this close to it; before that it lives in the tooltip (user, 2026-11-15). */
const FINAL_TURNS = 25;

export function TopBar() {
  const view = useGame((s) => (s.views ? s.views[s.seat] : null));
  const orders = useGame((s) => s.orders[s.seat]);
  const errors = useGame((s) => s.errors[s.seat]);
  const set = useGame((s) => s.set);
  const net = useGame((s) => (s.client?.kind === 'remote' ? s.net : null));
  const phone = usePhoneLayout();
  const bar = useRef<HTMLElement>(null);
  useTopBarCompaction(bar, phone);
  if (!view) return null;
  const e = view.economy;
  const gold = pendingSpend(view, orders ?? [], errors ?? []).gold;
  const bankruptcy = bankruptcyWarning(view);
  const noScience = firstScienceForecast(view);
  const r = view.research;
  const pending = pendingResearchQueue(orders);
  const queue = effectiveResearchQueue(orders, r);
  const researching = queue[0];
  const forecasts = queueSchedule(r, queue, e.science_per_turn.total, view.turn);
  const mePublic = view.players.find((p) => p.id === view.me);
  const decadence = decadenceRules(view.rule_experiments).decadenceSlope > 0 ? mePublic?.decadence_percent ?? 0 : 0;
  const queued = e.gold - gold;
  const goldNotes = <>
    {queued > 0 && <div className="bd-note" data-testid="queued-gold-note">Treasury {e.gold} · queued this turn −{queued} · left to spend {gold}</div>}
    {decadence > 0 && mePublic && <DecadenceNote percent={decadence} lead={mePublic.lead_percent ?? 0} />}
    <div className="bd-note" data-testid="desert-note">If your treasury cannot pay what your units cost, a unit deserts.</div>
  </>;
  const goldSummary = [
    queued > 0 ? `Treasury ${e.gold}, ${queued} queued this turn` : '',
    decadence > 0 ? `Decadence −${decadence}% of growth, Production, Gold and Science` : '',
  ].filter(Boolean).join(' · ');
  const warning = bankruptcy && e.bankruptcy ? <>
    <div><Amounts text={bankruptcy} /></div>
    <BreakdownTable b={e.bankruptcy.gold} title="Next turn Gold before bankruptcy" />
    <div className="bd-note">{FORECAST_ASSUMPTIONS}</div>
  </> : null;
  const turnTip = `${view.turn_limit === null
    ? 'No turn limit; play until victory.'
    : `${Math.max(0, view.turn_limit - view.turn)} turns until the turn limit. At turn ${view.turn_limit} the highest VP wins.`} ${CALENDAR_NOTE}`;
  return (
    <header ref={bar} className="topbar" data-overlay-slot="top-bar">
      <div className="brand">
        <Wordmark height={24} />
      </div>
      <div className="tb-stats">
        <Tip content={<>{phone && warning}<BreakdownTable b={e.gold_per_turn} title="Gold per turn" />{goldNotes}</>} summary={goldSummary ? `Gold per turn: ${e.gold_per_turn.total} · ${goldSummary}` : undefined} className={`stat gold-stat${phone && warning ? ' gold-stat-warn' : ''}`}>
          {/* Phones have no room for the warning cell: the Gold cell carries the alarm and its tip the details. */}
          {phone && warning && <span className="gold-warn-mark" role="img" aria-label="Treasury warning"><Icon name="ui-alert" size={14} /></span>}
          <YieldIcon k="gold" size={16} />
          <b data-celebration-counter="gold">{gold}<CounterDelta counter="gold" /></b>
          <HoardReadout view={view} gold={gold} />
          <span className={e.gold_per_turn.total < 0 ? 'neg' : 'pos'}>({signed(e.gold_per_turn.total)})</span>
        </Tip>
        <TombReadout view={view} />
        {!phone && warning && e.bankruptcy && (
          <Tip className="gold-warning" content={warning}>
            <span className="gold-warning-text"><b><Icon name="ui-alert" size={13} className="ui-mark" /> Treasury <Quantity value={e.bankruptcy.gold.total} unit="Gold" /><span className="gw-long"> next turn</span></b><span className="gw-long"> {e.bankruptcy.disband ? `${itemName({ kind: 'Unit', unit: e.bankruptcy.disband.kind }, view.players.find((p) => p.id === view.me)?.faction)} will be disbanded` : 'treasury resets to 0'}</span></span>
          </Tip>
        )}
        <Tip
          content={
            <>
              <BreakdownTable b={e.science_per_turn} title="Science per turn" />
              {decadence > 0 && mePublic && <DecadenceNote percent={decadence} lead={mePublic.lead_percent ?? 0} />}
              <div className="bd-note"><Amount n={r.progress} unit="Science" /> stored toward research.</div>
              {noScience && <div className="bd-note"><Amounts text={noScience} /></div>}
              {e.upkeep > 0 && <div className="bd-note">Upkeep <Amount n={e.upkeep} unit="Gold" /> a turn: military units (the first 2 and all Scouts are free) and Alliances.{e.upkeep_breakdown && <BreakdownTable b={e.upkeep_breakdown} title="Upkeep" unit="Gold/turn" />}</div>}
            </>
          }
          summary={noScience ?? <ScienceHoverSummary view={view} />}
          className="stat science-stat"
        >
          <YieldIcon k="science" size={16} />
          <span className="pos" data-celebration-counter="science">{signed(e.science_per_turn.total)}<CounterDelta counter="science" /></span>
        </Tip>
        <SupplyChips view={view} />
        <LevyCounter />
        <span className="population-counter" data-celebration-counter="population" title="Total population in your cities"><Icon name="title-most-populous" size={12} className="count-icon ui-mark" label="Population" /><span className="count-label">Pop. </span><b>{view.cities.filter((c) => c.owner === view.me && c.detail).reduce((n, c) => n + c.population, 0)}</b><CounterDelta counter="population" /></span>
        {!phone && <Tip content={<>
          <div className="bd-title">{eraName(r.era)} era</div>
          <div>{r.era_known} of {r.era_total} {eraName(r.era)} technologies known. Only this era's technologies can be researched; finishing them all enters the next era.</div>
          <div className="bd-note">Click to open research.</div>
        </>} summary={`${eraName(r.era)} era: ${r.era_known} of ${r.era_total} technologies known`} className="era-chip-tip">
          <button className="era-chip" data-testid="era-chip" aria-label={`${eraName(r.era)} era, ${r.era_known} of ${r.era_total} technologies known. Open research`} onClick={() => set({ techOpen: true })}>
            <span className="era-chip-numeral" aria-hidden="true">{eraNumeral(r.era)}</span>
            <span className="era-chip-name">{eraName(r.era)}</span>
            <small className="era-chip-count">{r.era_known}/{r.era_total}</small>
          </button>
        </Tip>}
        <div className="research-slot">
          <button className={`research-chip ${researching ? '' : 'needs'}`} onClick={() => set({ techOpen: true })}>
            <span className="research-label">Research</span>
            {/* Phones hide the plan; the cell still names what is being researched. */}
            {researching && <span className="research-phone-tech">{TECH_NAME[researching]}</span>}
            {researching ? <span className="research-chip-plan">
              {forecasts.slice(0, 2).map((f, i) => <span key={f.tech}><span className="research-queue-number">{i + 1}</span> {TECH_NAME[f.tech]} <small>{f.arrivalTurn === null ? noScience ? 'No Science yet' : 'Needs Science' : `T${f.arrivalTurn}`}</small>{i === 1 && queue.length > 2 && ` +${queue.length - 2}`}</span>)}
            </span> : <b>Choose a technology</b>}
            {pending !== null && <span className="pending-dot" aria-label="Pending until Ready" />}
            {/* Progress toward the current technology, as the city panel shows production. */}
            {researching === r.current && r.current && r.cost ? <span className="research-progress" aria-hidden="true">
              <span style={{ width: `${Math.min(100, Math.max(0, r.progress / r.cost * 100))}%` }} />
            </span> : null}
          </button>
        </div>
      </div>
      <div className="tb-right">
        {net && (
          <Tip content={net.status.is_local ? 'You host this game in this tab. Keep it open.' : net.conn === 'open' ? 'Connected to the host' : 'Connection lost: reconnecting automatically'} className={`net ${net.conn}`}>
            <span className={`conn-dot ${net.conn === 'open' ? 'on' : 'warn'}`} />
            {net.conn === 'open' ? 'Online' : net.conn === 'taken' ? 'Seat taken' : 'Reconnecting'}

          </Tip>
        )}
        <Tip content={<>{turnTip}<TrainingTip /></>} className="turn">
          <span className="turn-label">Turn</span>
          <b data-testid="turn-number">{view.turn}</b>
          {view.turn_limit !== null && view.turn_limit - view.turn <= FINAL_TURNS && <span className="turn-of" data-testid="turn-of">of {view.turn_limit}</span>}
          <span className="turn-year" data-testid="turn-year">· {calendarLabel(view.turn)}</span>
          <TrainingLine />
        </Tip>
        <VpChip />
        {/* The research cell opens the same dialog, so the most crowded bar drops this button. */}
        <button className="tb-nav tb-nav-tech" onClick={() => set({ techOpen: true })}>
          Technologies
        </button>
        <CitiesButton />
        <GameMenu />
      </div>
    </header>
  );
}
