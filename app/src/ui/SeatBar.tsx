import { GoalActionRow } from './GoalActions';
import { SidebarFactionGoal, FactionGoalProgressLine } from './FactionGoalProgress';
import { AUTOMATION_NO_CHOICE, automatedUnits } from '../automation';
import { orderUnit, setUnitOrder } from '../orders';
import { Component, useRef, useState, type ErrorInfo, type ReactNode } from 'react';
import { nextDecision as nextPendingDecision, pendingDecisions } from '../turnFlow';
import { unitCardLeads } from './turnLead';
import { EndTurnConfirmation } from './EndTurnConfirmation';
import { readySelected } from '../commands';
import { feel } from '../feel';
import { canResolveToEnd, isEliminated, observerController, pendingAlarms, useGame, type Seat } from '../store';
import { useCelebration } from '../celebrate/MomentLane';
import type { Order, OrderError, PlayerView, UnitView } from '../types';
import { teamName } from '../diplomacy';
import { describeOrder, removeOrderAt } from '../orders';
import { playerColor, playerInk } from '../theme';
import { Icon, Kbd, Tip } from './index';
import { CrestMedallion, PanelHead } from '../brand/ornaments';
import { CityList } from './CityList';
import { AirWatch } from './AirWatch';
import { OpenSeatButton } from './OpenSeatButton';
import { RemoveSeatButton } from './RemoveSeatButton';
import { BUILDING_NAME, itemName } from '../data';
import { empireOfPlayer } from '../empires';
import { bankruptcyWarning, FORECAST_ASSUMPTIONS, idleProductionWarning, woundedNearEnemies } from './forecastWarnings';
import { usePhoneLayout } from './usePhoneLayout';
import { useCoarsePointer } from './useCoarsePointer';
import { LastTurnButton } from './TurnReport';
import { useScrollHint } from './useScrollHint';
import { useRowsBelow } from './useRowsBelow';
import { counted } from '../plural';
import { foreignCityChoice } from './ForeignCityActions';
import { Amounts } from './Amounts';
import { SiegeAlerts } from './SiegeAlerts';
import { useCityAlarms } from './useCityAlarms';

/** The next seat after `seat` (cyclically) that still has to plan, or null. */
class OrderListBoundary extends Component<{ children: ReactNode }, { failed: boolean }> {
  state = { failed: false };
  static getDerivedStateFromError() { return { failed: true }; }
  componentDidCatch(error: Error, _info: ErrorInfo) { console.warn('Could not display pending orders', error); }
  render() {
    return this.state.failed ? <div className="muted" role="status">Some pending orders could not be displayed. You can still use the rest of the game.</div> : this.props.children;
  }
}

function PendingOrderList({ orders, errors, automated, view, busy, eliminated, editOrders }: {
  orders: Order[]; errors: OrderError[]; automated: UnitView[]; view: PlayerView;
  busy: string | null; eliminated: boolean; editOrders: ReturnType<typeof useGame.getState>['editOrders'];
}) {
  return <>
    {orders.length === 0 && automated.length === 0 && <div className="muted">No orders yet. Units with a destination keep moving on their own.</div>}
    {automated.map((u) => <div key={`auto-${u.id}`} className="order-item">
      <span>{u.kind}: auto-explore (automated · route chosen at End turn)</span>
      <button className="x" disabled={!!busy || view.finished || eliminated} aria-label={`Stop auto-explore for ${u.kind} ${u.id}`}
        onClick={() => { editOrders((ord) => setUnitOrder(ord, { type: 'SetAutomation', unit: u.id, automation: null })); feel.select(); }}><Icon name="ui-cross" size={11} /></button>
    </div>)}
    {orders.map((o, i) => {
      const err = errors.find((e) => e.index === i);
      const description = describeOrder(o, view);
      return <div key={i} className={`order-item ${err ? 'bad' : ''}`}>
        <span><Amounts text={description} /></span>
        <button className="x" disabled={view.finished || eliminated} onClick={() => editOrders((ord) => removeOrderAt(ord, i))} title="Cancel" aria-label={`Cancel ${description}`}>
          <Icon name="ui-cross" size={11} />
        </button>
        {err && <div className="order-error">{err.message}</div>}
      </div>;
    })}
  </>;
}

function nextPlanningSeat(seat: Seat, ready: boolean[], views: PlayerView[]): Seat | null {
  for (let k = 1; k < ready.length; k++) {
    const s = (seat + k) % ready.length;
    if (!ready[s] && !isEliminated(views[s])) return s;
  }
  return null;
}

/**
 * Mark the seat Ready and hand the screen to the next seat that still has to plan. With a single
 * human seat this resolves the turn right away.
 */
export function readyAndPass() {
  requestReady(useGame.getState().seat, true);
}

/** Every user-facing Ready/End Turn entry point must pass through this review gate. */
export function requestReady(seat: Seat, pass = false) {
  const s = useGame.getState();
  const view = s.views?.[seat];
  if (!view || view.finished || s.busy || s.resolvingToEnd) return;
  if (s.client?.kind === 'remote' && s.net?.conn !== 'open') return;
  if (!s.ready[seat] && pendingDecisions(view, s.orders[seat], s.errors[seat], s).length) {
    s.set({ endTurnConfirmation: { seat, view, orders: s.orders[seat], pass } });
    feel.select();
    return;
  }
  commitReady(seat, pass);
}

export function confirmEndTurn() {
  const s = useGame.getState();
  const review = s.endTurnConfirmation;
  s.set({ endTurnConfirmation: null });
  // A seat change, new turn, or edited plan needs its own review.
  if (!review || s.views?.[review.seat] !== review.view || s.orders[review.seat] !== review.orders) return;
  commitReady(review.seat, review.pass);
}

function commitReady(seat: Seat, passSeat: boolean) {
  const s = useGame.getState();
  const view = s.views?.[seat];
  if (!view || view.finished || s.busy || s.resolvingToEnd) return;
  if (s.client?.kind === 'remote' && s.net?.conn !== 'open') return;
  if (isEliminated(view) && !canResolveToEnd(view, s.client?.kind === 'remote' ? s.net : null)) {
    const next = nextPlanningSeat(seat, s.ready, s.views!);
    if (s.client?.kind !== 'remote' && next !== null) s.setSeat(next);
    return;
  }
  if (s.client?.kind === 'remote') {
    readySelected();
    return;
  }
  const wasReady = s.ready[seat];
  const result = readySelected(seat);
  const pass = () => {
    const after = useGame.getState();
    if (after.client !== s.client || after.views !== s.views || !after.ready[seat]) return;
    const next = nextPlanningSeat(seat, after.ready, after.views!);
    if (passSeat && !wasReady && next !== null && !after.busy) { after.setSeat(next); after.set({ handoff: next }); }
  };
  if (useGame.getState().busy === 'Planning automation…') void result.then(pass);
  else pass();
}

export function SeatBar({ phoneDetails = false, onOpenDetails, race }: { phoneDetails?: boolean; onOpenDetails?: () => void; race?: ReactNode }) {
  const views = useGame((s) => s.views);
  const humans = useGame((s) => s.humans);
  const seat = useGame((s) => s.seat);
  const ready = useGame((s) => s.ready);
  const orders = useGame((s) => s.orders);
  const errors = useGame((s) => s.errors);
  const setSeat = useGame((s) => s.setSeat);
  const toggleReady = requestReady;
  const nextDecision = useGame((s) => s.nextDecision);
  const selection = useGame((s) => s.selection);
  const attacker = useGame((s) => s.attacker);
  const diplomacyOpen = useGame((s) => s.diplomacyOpen);
  const editOrders = useGame((s) => s.editOrders);
  const busy = useGame((s) => s.busy);
  const resolvingToEnd = useGame((s) => s.resolvingToEnd);
  const resolveToEnd = useGame((s) => s.resolveToEnd);
  const stopResolving = useGame((s) => s.stopResolving);
  const [showOrders, setShowOrders] = useState(false);
  const { scroll: body, overflow: bodyScrolls } = useScrollHint();
  const sidebarScroll = useRef<HTMLDivElement>(null);
  const empiresBelow = useRowsBelow(sidebarScroll, '.vp-row');
  const phone = usePhoneLayout();
  const touch = useCoarsePointer();
  const net = useGame((s) => s.net);
  const siegeSeen = useGame((s) => s.siegeSeen);
  const airHeld = useGame((s) => s.airHeld);
  const goalDecisionSeen = useGame((s) => s.goalDecisionSeen);
  const alarms = useCityAlarms();
  // Pillar 9: one Return on screen. A moment card or an interstitial on screen owns Return, so the
  // turn's call to action steps back and drops its key chip until it closes.
  const { active: moment, deferred: momentDeferred } = useCelebration();
  const alarmUp = useGame((s) => pendingAlarms(s).length > 0);
  const returnOwned = alarmUp || (!!moment && moment.tier !== 'minor' && !momentDeferred);
  const report = useGame((s) => s.report);
  const scenario = useGame((s) => s.scenario);
  const remote = useGame((s) => s.client?.kind === 'remote');
  const disconnected = remote && net?.conn !== 'open';
  // The host's seat list opens by itself only while a lost player's seat can be opened. Remove is rare
  // and waits behind Turn details, so the race stays in view (UI review, 2026-11-22).
  const hostCanOpenSeat = remote && net?.status.is_local && net.seats.some((st) =>
    st.seat !== net.mySeat && !st.connected && st.reserved !== false);
  if (!views) return null;
  const view = views[seat];
  const eliminated = isEliminated(view);
  const observerCanAdvance = canResolveToEnd(view, remote ? net : null);
  const controller = remote && net && canResolveToEnd(view) ? observerController(net) : null;
  const nameOf = (id: number) => view.players.find((p) => p.id === id)?.name ?? `Player ${id + 1}`;
  // Who we're waiting on: living online seats that aren't Ready, or local seats still planning.
  const waitingOn = remote
    ? (net?.seats ?? []).filter((x) => x.seat !== net?.mySeat && !x.ready && view.players.find((p) => p.id === x.player)?.alive !== false).map((x) => x.name)
    : humans.filter((_, i) => i !== seat && !ready[i] && !isEliminated(views[i])).map(nameOf);
  const automated = automatedUnits(view, orders[seat]).filter((u) => !orders[seat].some((o) => orderUnit(o) === u.id));
  const decisions = pendingDecisions(view, orders[seat], errors[seat], { humans, orders, errors, siegeSeen, airHeld, seat, report, goalDecisionSeen });
  const needing = decisions.filter((d) => d.kind === 'unit' || d.kind === 'air');
  // Name all outstanding work, including the selection. Keep the CTA's navigation
  // within that priority; Return still cycles through the whole decision list.
  const following = nextPendingDecision(decisions, selection);
  const wrapPriority = !following?.issue && following?.kind !== decisions[0]?.kind;
  const next = wrapPriority ? decisions[0] : following;
  const goToPending = () => nextDecision(wrapPriority);
  const nextCity = next?.kind === 'city' ? view.cities.find((c) => c.id === next.id) : null;
  const automationNeedsChoice = next?.kind === 'city' && next.label.includes(AUTOMATION_NO_CHOICE);
  const bankruptcy = eliminated ? null : bankruptcyWarning(view);
  const wounded = eliminated ? [] : woundedNearEnemies(view);
  const myErrors = errors[seat].filter((e) => e.index >= 0);

  let cta: { label: string; sub: string; onClick: () => void; kind: string; disabled?: boolean };
  if (scenario && scenario.status.outcome !== 'Playing') {
    cta = { label: 'Scenario over', sub: 'See the result', onClick: () => useGame.getState().set({ scenario: { ...scenario, resultOpen: true } }), kind: 'todo' };
  } else if (view.finished && view.history) {
    cta = { label: 'Game over', sub: 'Game report', onClick: () => useGame.getState().openGameReport(), kind: 'todo' };
  } else if (view.finished) {
    cta = { label: 'Game finished', sub: 'View the final map', onClick: () => {}, kind: 'waiting', disabled: true };
  } else if (eliminated) {
    const next = nextPlanningSeat(seat, ready, views);
    cta = observerCanAdvance ? {
      label: ready[seat] ? 'Waiting for the next turn…' : 'End turn',
      sub: 'Observe with no orders',
      onClick: readyAndPass,
      kind: 'ready',
      disabled: ready[seat] || resolvingToEnd,
    } : {
      label: !remote && next !== null ? 'Pass to next player' : 'Observing',
      sub: controller ? `${controller.name} controls the remaining turns`
        : waitingOn.length ? `Waiting for ${waitingOn.join(', ')}` : 'The remaining players control the turns',
      onClick: readyAndPass,
      kind: 'waiting',
      disabled: remote || next === null,
    };
  } else if (ready[seat]) {
    cta = {
      label: waitingOn.length === 0 ? 'Resolving…' : `Waiting for ${waitingOn.join(', ')}…`,
      sub: 'Click to keep planning',
      onClick: () => toggleReady(seat),
      kind: 'waiting',
    };
  } else if (next?.kind === 'diplomacy') {
    // One short line: the phone's half-width turn dock has no room for a wrapped deadline.
    const offer = view.diplomacy?.proposals.find((p) => p.id === next.id);
    const call = view.diplomacy?.calls?.find((c) => c.id === next.id);
    cta = call
      ? { label: 'Answer call to arms', sub: `${teamName(view, call.ally)} · war on ${teamName(view, call.target)} · by T${call.respond_by}`, onClick: goToPending, kind: 'todo' }
      : { label: 'Respond to offer', sub: offer ? `Reply by T${offer.respond_by}` : next.label, onClick: goToPending, kind: 'todo' };
  } else if (next?.kind === 'siege') {
    // A city an enemy can strike next turn outranks every routine decision (user, 2026-10-08).
    const alarm = alarms.find((a) => a.cityId === next.id);
    cta = { label: 'City under attack', sub: alarm ? `${alarm.name} · ${alarm.forecast.short}` : next.label, onClick: goToPending, kind: 'todo' };
  } else if (next?.kind === 'treasury') {
    const kind = view.economy.bankruptcy?.disband?.kind;
    cta = { label: 'Treasury runs dry', sub: `Lose ${kind ? itemName({ kind: 'Unit', unit: kind }, view.players.find((p) => p.id === view.me)?.faction) : 'a unit'} next turn`, onClick: goToPending, kind: 'todo' };
  } else if (next?.kind === 'goal') {
    cta = { label: next.title, sub: next.line, onClick: goToPending, kind: 'todo' };
  } else if (next?.kind === 'unit' || next?.kind === 'air') {
    cta = { label: next.kind === 'unit' && next.issue ? 'Fix order' : needing.length === 1 ? '1 unit needs orders' : `${needing.length} units need orders`,
      // The label already says to fix it; the sub says what is wrong, in one sentence.
      sub: (next.kind === 'unit' ? next.fix ?? (next.healed ? `${itemName({ kind: 'Unit', unit: view.units.find((u) => u.id === next.id)?.kind ?? 'Spearman' }, view.players.find((p) => p.id === view.me)?.faction)}: healed to full, needs orders` : next.released ? `${itemName({ kind: 'Unit', unit: view.units.find((u) => u.id === next.id)?.kind ?? 'Spearman' }, view.players.find((p) => p.id === view.me)?.faction)}: ${next.released} is secure, needs orders` : undefined) : next.label) ?? 'Next decision', onClick: goToPending, kind: 'todo' };
  } else if (nextCity) {
    cta = {
      label: 'Choose production',
      sub: automationNeedsChoice ? `${nextCity.name} · ${AUTOMATION_NO_CHOICE}`
        : nextCity.detail?.wonder_lost ? `${nextCity.name} · ${BUILDING_NAME[nextCity.detail.wonder_lost.building]} lost to ${empireOfPlayer(view, nextCity.detail.wonder_lost.winner)}`
        : `${nextCity.name}${idleProductionWarning(nextCity.detail) ? ` · ${nextCity.detail!.idle_production_waste} Production wasted next turn` : ''}`,
      onClick: goToPending,
      kind: 'todo',
    };
  } else if (next?.kind === 'freeTechs') {
    cta = { label: next.label, sub: 'A rival shared its learning', onClick: goToPending, kind: 'todo' };
  } else if (next?.kind === 'research') {
    cta = { label: report?.events[seat]?.some((e) => e.type === 'ResearchQueueAdvanced' && e.player === view.me && e.next === null) ? 'Research queue finished: choose next' : 'Choose research', sub: 'Open technologies', onClick: goToPending, kind: 'todo' };
  } else {
    const solo = !remote && humans.length === 1;
    const lastTurn = scenario?.status.turns_left === 1;
    cta = { label: solo ? 'End turn' : 'Ready', sub: lastTurn ? 'Last turn of the scenario' : solo ? 'Resolve the turn' : 'End planning for this seat', onClick: readyAndPass, kind: 'ready' };
  }

  // Beside the race, a This Turn with nothing in it (no report link, order, seat or review) is only a heading over
  // "0 orders": it hides (sidebar.css also waits for no siege or air card) until it has something to show.
  const quiet = !!race && !phoneDetails && !report && !remote && humans.length <= 1 && !bankruptcy && !wounded.length && orders[seat].length + automated.length === 0;
  const details = <div id="turn-details" className={`turn-details${quiet ? ' quiet' : ''}`}>
      {/* Once a turn has resolved, the subtitle line becomes the way back to its report. */}
      <PanelHead title="This Turn" sub={report ? <LastTurnButton /> : race ? undefined : humans.length > 1 || remote ? 'Every seat plans at once' : 'Plan, then end the turn'} />
      {/* Everything that grows (seats, cities, orders, reviews) scrolls in this body on desktop, so
          the turn's primary action below stays pinned in view (styles.css, "Seats, orders"). */}
      <div ref={body} className={`turn-body${bodyScrolls ? ' scrolls' : ''}`}>
      {/* A city under attack is the turn's first concern: its card leads the body, outside the collapsed details. */}
      <SiegeAlerts />
      {!race && <><GoalActionRow /><FactionGoalProgressLine view={view} /></>}
      <details className="sidebar-turn-more" open={!race || hostCanOpenSeat || undefined}><summary hidden={!race}>Turn details · {counted(orders[seat].length + automated.length, 'order')}</summary>
      <AirWatch view={view} />
      {remote && net ? (
        <OnlineSeats />
      ) : humans.length > 1 ? (
      <div className="seats">
        {humans.map((id, s) => {
          const p = views[s].players.find((x) => x.id === id)!;
          return (
            <div
              key={s}
              className={`seat local-seat ${seat === s ? 'on' : ''} ${ready[s] ? 'ready' : ''}`}
              style={{ ['--pc' as string]: playerInk(id) }}
            >
              <button className="seat-select" aria-pressed={seat === s} onClick={() => setSeat(s)}>
                <CrestMedallion faction={p.faction} color={playerColor(id)} size={36} />
                <span className="seat-name">{p.name}</span>
                <span className="seat-meta">
                  {p.alive ? `${orders[s].length} order${orders[s].length === 1 ? '' : 's'}` : 'Observing'}
                </span>
              </button>
              <Tip className="seat-readiness" content={!p.alive ? 'Eliminated: this seat observes' : ready[s] ? 'Ready: click to un-ready' : 'Planning: click to mark Ready'}>
                <button
                  className={`seat-ready ${ready[s] ? 'on' : ''}`}
                  aria-label={`${p.name}: ready`}
                  aria-pressed={ready[s]}
                  disabled={!!busy || view.finished || !p.alive}
                  onClick={() => toggleReady(s)}
                >
                  {!p.alive ? 'Eliminated' : ready[s] ? <><Icon name="ui-check" size={12} className="ui-mark" /> Ready</> : 'Planning'}
                </button>
              </Tip>
              {s < 9 && <Kbd>{s + 1}</Kbd>}
            </div>
          );
        })}
      </div>
      ) : null}
      <CityList view={view} />
      <div className="orders-box">
        <button className="orders-toggle" aria-expanded={showOrders} aria-controls="pending-orders" onClick={() => setShowOrders(!showOrders)}>
          {/* The same disclosure mark as Cities above: a lone accent chevron read as a red dot. */}
          <span className={`orders-disclosure${showOrders ? ' open' : ''}`} aria-hidden="true">▶</span>
          Pending orders <b>{orders[seat].length + automated.length}</b>
          {myErrors.length > 0 && <span className="err-count">{myErrors.length} issue{myErrors.length > 1 ? 's' : ''}</span>}
        </button>
        <div id="pending-orders" className="orders-list" hidden={!showOrders}>
          <OrderListBoundary><PendingOrderList orders={orders[seat]} errors={errors[seat]} automated={automated} view={view}
            busy={busy} eliminated={eliminated} editOrders={editOrders} /></OrderListBoundary>
        </div>
      </div>
      {(bankruptcy || wounded.length > 0) && (
        <div className="turn-review">
          <div className="lbl">Things to review</div>
          <ul>
            {bankruptcy && <li><Tip content={FORECAST_ASSUMPTIONS}><Amounts text={bankruptcy} /></Tip></li>}
            {wounded.map(({ unit, text }) => <li key={unit.id} className="wounded-warning">
              <button className="link" onClick={() => useGame.getState().select({ kind: 'unit', id: unit.id }, true)}>{text}</button>
            </li>)}
          </ul>
        </div>
      )}
      </details>
      </div>
  </div>;
  const observerControl = observerCanAdvance && (
    <button className="btn" onClick={resolvingToEnd ? stopResolving : resolveToEnd} disabled={!resolvingToEnd && (!!busy || disconnected)}>
      {resolvingToEnd ? 'Stop' : 'Resolve to the end'}
    </button>
  );
  if (phone && phoneDetails) return <div className="phone-turn-details">{details}{observerControl}</div>;
  // Pillar 9: one primary on screen. A selected unit or city of ours shows its own primary action
  // in the sheet above, and the open diplomacy inbox has its Accept, so the turn row steps back
  // until the selection or the inbox is closed.
  const stepBack = returnOwned || diplomacyOpen || (!!selection && (selection.kind === 'unit' ? view.units : selection.kind === 'air' ? view.aircraft ?? [] : view.cities)
    .some((x) => x.id === selection.id && x.owner === view.me));
  if (phone) return <div className="phone-turn-dock" data-overlay-slot="turn-dock" data-pending-decisions={decisions.length}>
    <button className="btn" onClick={onOpenDetails} aria-controls="phone-sheet-content">This Turn</button>
    {/* While a decision is pending, Next names it, is the primary call to action, and goes where its
        label says (the CTA's decision, as on desktop), not to whatever follows the selection. */}
    <button className={`btn${cta.kind === 'todo' && !stepBack ? ' primary' : ''}`} onClick={() => (cta.kind === 'todo' ? cta.onClick() : nextDecision())} aria-label={`Next decision: ${cta.label}`}
      disabled={!!busy || view.finished || eliminated || disconnected}>
      {cta.kind !== 'todo' ? 'Next' : `Next: ${next?.kind === 'diplomacy' ? 'Offer' : next?.kind === 'city' ? 'Build' : next?.kind === 'research' ? 'Research' : next?.kind === 'freeTechs' ? 'Technologies' : 'Unit'}`}
    </button>
    <button className={`btn${cta.kind === 'todo' || stepBack ? '' : ' primary'}`} onClick={readyAndPass}
      disabled={!!busy || view.finished || disconnected || (eliminated && !observerCanAdvance) || resolvingToEnd}>
      {ready[seat] ? 'Keep planning' : !remote && humans.length === 1 ? 'End Turn' : 'Ready'}
    </button>
    <EndTurnConfirmation onConfirm={confirmEndTurn} />
  </div>;

  // Pillar 9: a rival city's card offering an attack makes that attack the one primary, so the turn's
  // call to action steps back while the card is open (a broken order still outranks it).
  const selectedCity = selection?.kind === 'city' ? view.cities.find((c) => c.id === selection.id) : undefined;
  const selectedOwnUnit = (selection?.kind === 'unit' && view.units.some((u) => u.id === selection.id && u.owner === view.me))
    || (selection?.kind === 'air' && !!view.aircraft?.some((a) => a.id === selection.id && a.owner === view.me));
  const stepped = returnOwned || (!!selectedCity && !!foreignCityChoice(view, selectedCity, attacker, orders[seat])?.canAttack && !next?.issue)
    // A selected unit's card holds the primary while the turn's call to action is routine (turnLead.ts).
    || (selectedOwnUnit && cta.kind === 'todo' && unitCardLeads(decisions, selection));
  return (
    <div className="panel seatbar" data-overlay-slot="turn-dock">
      <div ref={sidebarScroll} className="sidebar-scroll" tabIndex={0} aria-label="Race and turn details">{race && <SidebarFactionGoal view={view} />}{race}{details}
        {empiresBelow > 0 && <button type="button" className="sidebar-more" data-testid="sidebar-more" onClick={() => sidebarScroll.current?.scrollBy({ top: sidebarScroll.current.clientHeight * 0.8, behavior: 'smooth' })}>+{empiresBelow} more {empiresBelow === 1 ? 'empire' : 'empires'}</button>}
      </div>
      <EndTurnConfirmation onConfirm={confirmEndTurn} />
      <button className={`cta ${cta.kind}${next?.kind === 'diplomacy' && view.diplomacy?.calls?.some((c) => c.id === next.id) ? ' cta-call' : ''}${stepped ? ' stepped' : ''}${cta.label === 'Choose production' ? ` cta-production${automationNeedsChoice ? ' cta-long' : ''}` : cta.sub.length > 32 || next?.kind === 'siege' ? ' cta-long' : ''}`} onClick={cta.onClick} disabled={!!busy || (view.finished && cta.kind !== 'todo') || disconnected || cta.disabled}>
        <span className="cta-label">{cta.label}</span>
        <span className="cta-sub">
          <span><Amounts text={cta.sub} /></span> {!touch && !returnOwned && ((cta.kind === 'todo' && !ready[seat]) || cta.kind === 'ready') && <Kbd>Return</Kbd>}
        </span>
      </button>
      {observerCanAdvance && (
        <button className="btn" onClick={resolvingToEnd ? stopResolving : resolveToEnd} disabled={!resolvingToEnd && (!!busy || disconnected)}>
          {resolvingToEnd ? 'Stop' : 'Resolve to the end'}
        </button>
      )}
      {cta.kind === 'todo' && !ready[seat] && !view.finished && (
        <button className="ready-anyway" onClick={readyAndPass} disabled={!!busy || disconnected}>
          Ready anyway {!touch && <Kbd>Shift+Return</Kbd>}
        </button>
      )}
      <div className="touch-turn-controls touch-only">
        <button className="btn" onClick={() => nextDecision()} disabled={!!busy || view.finished || eliminated || disconnected}>Next decision</button>
        <button className="btn primary" onClick={readyAndPass} disabled={!!busy || view.finished || disconnected || (eliminated && !observerCanAdvance) || resolvingToEnd}>
          {ready[seat] ? 'Keep planning' : !remote && humans.length === 1 ? 'End Turn' : 'Ready'}
        </button>
      </div>
    </div>
  );
}

/** Online: every human seat, each on its own computer. */
export function OnlineSeats() {
  const net = useGame((s) => s.net)!;
  const view = useGame((s) => (s.views ? s.views[s.seat] : null));
  const ready = useGame((s) => s.ready[s.seat]);
  const orders = useGame((s) => s.orders[s.seat]);
  const releaseSeat = useGame((s) => s.releaseOnlineSeat);
  const removePlayer = useGame((s) => s.removeOnlinePlayer);
  const conn = net.conn;
  const mySeat = net.mySeat;
  const rows = net.seats.map((st) => {
    const p = view?.players.find((x) => x.id === st.player);
    return { seat: st.seat, st, p };
  });
  return (
    <div className="seats">
      {rows.map(({ seat, st, p }) => {
        const me = seat === mySeat;
        const isReady = me ? ready : !!st?.ready;
        const connected = me ? conn === 'open' : !!st?.connected;
        const count = me ? orders.length : (st?.orders ?? 0);
        return (
          <div key={seat} data-testid={`online-seat-${seat}`} className={`seat ${me ? 'on' : ''} ${isReady ? 'ready' : ''}`} style={{ ['--pc' as string]: playerInk(st?.player ?? seat) }}>
            {p && <CrestMedallion faction={p.faction} color={playerColor(st?.player ?? seat)} size={36} />}
            <span className="seat-name">
              {me ? 'You' : st?.name ?? p?.name ?? 'Player'}
            </span>
            <span className="seat-meta">
              <span className={`conn-dot ${connected ? 'on' : ''}`} />
              {connected ? (me ? st?.name ?? p?.name : 'online') : me ? (conn === 'reconnecting' ? 'reconnecting…' : 'offline') : st.reserved === false ? 'open' : 'not connected'}
              {' · '}
              {count} order{count === 1 ? '' : 's'}
            </span>
            <span className={`seat-ready ${isReady ? 'on' : ''}`}>{p?.alive === false ? 'Observing' : isReady ? <><Icon name="ui-check" size={12} className="ui-mark" /> Ready</> : 'Planning'}</span>
            {net.status.is_local && !me && !connected && st.reserved !== false && (
              <OpenSeatButton name={st.name} disabled={conn !== 'open'} onOpen={() => releaseSeat(seat)} />
            )}
            {net.status.is_local && !me && connected && (
              <RemoveSeatButton name={st.name} disabled={conn !== 'open'} onRemove={() => removePlayer(seat)} />
            )}
          </div>
        );
      })}
    </div>
  );
}
