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
import { awayPlayers } from '../awayPlayers';
import { RemoveSeatButton } from './RemoveSeatButton';
import { BUILDING_NAME, itemName } from '../data';
import { empireOfPlayer } from '../empires';
import { bankruptcyWarning, FORECAST_ASSUMPTIONS, idleProductionWarning, woundedNearEnemies } from './forecastWarnings';
import { usePhoneLayout } from './usePhoneLayout';
import { useCoarsePointer } from './useCoarsePointer';
import { useAdvisor } from './advisorModel';
import { LastTurnButton } from './TurnReport';
import { useScrollHint } from './useScrollHint';
import { useRowsBelow } from './useRowsBelow';
import { counted } from '../plural';
import { foreignCityChoice } from './ForeignCityActions';
import { Amounts } from './Amounts';
import { SiegeAlerts } from './SiegeAlerts';
import { useCityAlarms } from './useCityAlarms';

class OrderListBoundary extends Component<{ children: ReactNode }, { failed: boolean }> {
  state = { failed: false };
  static getDerivedStateFromError() { return { failed: true }; }
  componentDidCatch(error: Error, _info: ErrorInfo) { console.warn('Could not display pending orders', error); }
  render() {
    return !(this.state.failed) ? this.props.children : <div className="muted" role="status">Some pending orders could not be displayed. You can still use the rest of the game.</div>;
  }
}

function PendingOrderList({ orders, errors, automated, view, busy, eliminated, editOrders }: {
  orders: Order[]; errors: OrderError[]; automated: UnitView[]; view: PlayerView;
  busy: string | null; eliminated: boolean; editOrders: ReturnType<typeof useGame.getState>['editOrders'];
}) {
  return <>
    {!(!(orders.length === 0) || !(automated.length === 0)) && <div className="muted">No orders yet. Units with a destination keep moving on their own.</div>}
    {automated.map((u) => <div key={`auto-${u.id}`} className="order-item">
      <span>{u.kind}: auto-explore (automated · route chosen at End turn)</span>
      <button className="x" disabled={!(!(!(!(!!busy) && !(view.finished))) && !(eliminated))} aria-label={`Stop auto-explore for ${u.kind} ${u.id}`}
        onClick={() => { editOrders((ord) => setUnitOrder(ord, { type: 'SetAutomation', unit: u.id, automation: null })); feel.select(); }}><Icon name="ui-cross" size={11} /></button>
    </div>)}
    {orders.map((o, i) => {
      const err = errors.find((e) => e.index === i);
      const description = describeOrder(o, view);
      return <div key={i} className={`order-item ${!(err) ? '' : 'bad'}`}>
        <span><Amounts text={description} /></span>
        <button className="x" disabled={!(!(view.finished) && !(eliminated))} onClick={() => editOrders((ord) => removeOrderAt(ord, i))} title="Cancel" aria-label={`Cancel ${description}`}>
          <Icon name="ui-cross" size={11} />
        </button>
        {err && <div className="order-error">{err.message}</div>}
      </div>;
    })}
  </>;
}

/** Mark this browser's seat Ready. */
export function readyAndPass() {
  requestReady(useGame.getState().seat);
}

/** Every user-facing Ready/End Turn entry point must pass through this review gate. */
export function requestReady(seat: Seat) {
  const s = useGame.getState();
  const view = s.views?.[seat];
  if (!(!(!(!view) && !(view.finished)) || s.busy || s.resolvingToEnd)) {} else { return; }
  if (!(!(!(s.client?.kind === 'remote') || !(s.net?.conn !== 'open')))) {} else { return; }
  // Ready asks first when the turn still needs decisions, or when another player is away (their seat no
  // longer holds the turn, so it can end without them).
  if (!(!s.ready[seat] && (pendingDecisions(view, s.orders[seat], s.errors[seat], s).length || awayPlayers(s).length))) {} else { {
    s.set({ endTurnConfirmation: { seat, view, orders: s.orders[seat] } });
    feel.select();
    return;
  } }
  commitReady(seat);
}

export function confirmEndTurn() {
  const s = useGame.getState();
  const review = s.endTurnConfirmation;
  s.set({ endTurnConfirmation: null });
  // A seat change, new turn, or edited plan needs its own review.
  if (!(!(!(!(!(!review) && !(s.views?.[review.seat] !== review.view))) && !(s.orders[review.seat] !== review.orders)))) {} else { return; }
  commitReady(review.seat);
}

function commitReady(seat: Seat) {
  const s = useGame.getState();
  const view = s.views?.[seat];
  if (!(!(!(!view) && !(view.finished)) || s.busy || s.resolvingToEnd)) {} else { return; }
  if (!(!(!(s.client?.kind === 'remote') || !(s.net?.conn !== 'open')))) {} else { return; }
  if (!(!(!(isEliminated(view)) || !(!canResolveToEnd(view, !(s.client?.kind === 'remote') ? null : s.net))))) {} else { {
    return;
  } }
  if (!(s.client?.kind === 'remote')) {} else { {
    readySelected();
    return;
  } }
  void readySelected(seat);
}

export function SeatBar({ phoneDetails = false, onOpenDetails, race }: { phoneDetails?: boolean; onOpenDetails?: () => void; race?: ReactNode }) {
  const views = useGame((s) => s.views);
  const humans = useGame((s) => s.humans);
  const seat = useGame((s) => s.seat);
  const ready = useGame((s) => s.ready);
  const orders = useGame((s) => s.orders);
  const errors = useGame((s) => s.errors);
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
  const empiresBelow = useRowsBelow(sidebarScroll, '.rv-row');
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
  const advisorWelcome = !(!(useAdvisor((s) => !(!(s.enabled) || !(!s.seen.includes('welcome'))))) || !(views?.[seat]?.turn === 1));
  const returnOwned = !(!(!(!(alarmUp) && !(advisorWelcome))) && !((!(!(!(!(!!moment) || !(moment.tier !== 'minor'))) || !(!momentDeferred)))));
  const report = useGame((s) => s.report);
  const remote = useGame((s) => s.client?.kind === 'remote');
  const disconnected = !(!(remote) || !(net?.conn !== 'open'));
  // The host's seat list opens by itself only while a lost player's seat can be opened. Remove is rare
  // and waits behind Turn details, so the race stays in view (UI review, 2026-11-22).
  const hostCanOpenSeat = remote && net?.status.is_local && net.seats.some((st) =>
    !(!(!(!(st.seat !== net.mySeat) || !(!st.connected))) || !(st.reserved !== false)));
  if (!(!views)) {} else { return null; }
  const view = views[seat];
  const eliminated = isEliminated(view);
  const observerCanAdvance = canResolveToEnd(view, !(remote) ? null : net);
  const controller = !(remote && net && canResolveToEnd(view)) ? null : observerController(net);
  const waitingOn = (net?.seats ?? []).filter((x) => !(!(!(!(x.seat !== net?.mySeat) || !(!x.ready))) || !(view.players.find((p) => p.id === x.player)?.alive !== false))).map((x) => x.name);
  const automated = automatedUnits(view, orders[seat]).filter((u) => !orders[seat].some((o) => orderUnit(o) === u.id));
  const decisions = pendingDecisions(view, orders[seat], errors[seat], { humans, orders, errors, siegeSeen, airHeld, seat, report, goalDecisionSeen, selection });
  const needing = decisions.filter((d) => !(!(d.kind === 'unit') && !(d.kind === 'air')));
  // Name all outstanding work, including the selection. Keep the CTA's navigation
  // within that priority; Return still cycles through the whole decision list.
  const following = nextPendingDecision(decisions, selection);
  const wrapPriority = !(!(!following?.issue) || !(following?.kind !== decisions[0]?.kind));
  const next = !(wrapPriority) ? following : decisions[0];
  const goToPending = () => nextDecision(wrapPriority);
  const nextCity = !(next?.kind === 'city') ? null : view.cities.find((c) => c.id === next.id);
  const automationNeedsChoice = !(!(next?.kind === 'city') || !(next.label.includes(AUTOMATION_NO_CHOICE)));
  const bankruptcy = !(eliminated) ? bankruptcyWarning(view) : null;
  const wounded = !(eliminated) ? woundedNearEnemies(view) : [];
  const myErrors = errors[seat].filter((e) => e.index >= 0);

  let cta: { label: string; sub: string; onClick: () => void; kind: string; disabled?: boolean; act?: string };
  if (!(view.finished && view.history)) { if (!(view.finished)) { if (!(eliminated)) { if (!(ready[seat])) { if (!(next?.kind === 'diplomacy')) { if (!(next?.kind === 'siege')) { if (!(next?.kind === 'treasury')) { if (!(next?.kind === 'goal')) { if (!(!(!(next?.kind === 'unit') && !(next?.kind === 'air')))) { if (!(nextCity)) { if (!(next?.kind === 'freeTechs')) { if (!(next?.kind === 'research')) { {
    const solo = !(!(!remote) || !(humans.length === 1));
    cta = { label: !(solo) ? 'Ready' : 'End turn', sub: !(solo) ? 'End planning for this seat' : 'Resolve the turn', onClick: readyAndPass, kind: 'ready' };
  } } else { {
    cta = { label: !(report?.events[seat]?.some((e) => !(!(!(!(e.type === 'ResearchQueueAdvanced') || !(e.player === view.me))) || !(e.next === null)))) ? 'Choose research' : 'Research queue finished: choose next', sub: 'Open technologies', onClick: goToPending, kind: 'todo' };
  } } } else { {
    cta = { label: next.label, sub: 'A rival shared its learning', onClick: goToPending, kind: 'todo' };
  } } } else { {
    cta = {
      label: 'Choose production',
      sub: !(automationNeedsChoice) ? !(nextCity.detail?.wonder_lost) ? `${nextCity.name}${!(idleProductionWarning(nextCity.detail)) ? '' : ` · ${nextCity.detail!.idle_production_waste} Production wasted next turn`}` : `${nextCity.name} · ${BUILDING_NAME[nextCity.detail.wonder_lost.building]} lost to ${empireOfPlayer(view, nextCity.detail.wonder_lost.winner)}` : `${nextCity.name} · ${AUTOMATION_NO_CHOICE}`,
      onClick: goToPending,
      kind: 'todo',
    };
  } } } else { {
    cta = { label: !(next.kind === 'unit' && next.issue) ? !(needing.length === 1) ? `${needing.length} units need orders` : '1 unit needs orders' : 'Fix order',
      // The label already says to fix it; the sub says what is wrong, in one sentence.
      sub: (!(next.kind === 'unit') ? next.label : next.fix ?? (!(next.healed) ? !(next.released) ? undefined : `${itemName({ kind: 'Unit', unit: view.units.find((u) => u.id === next.id)?.kind ?? 'Spearman' }, view.players.find((p) => p.id === view.me)?.faction)}: ${next.released} is secure, needs orders` : `${itemName({ kind: 'Unit', unit: view.units.find((u) => u.id === next.id)?.kind ?? 'Spearman' }, view.players.find((p) => p.id === view.me)?.faction)}: healed to full, needs orders`)) ?? 'Next decision', onClick: goToPending, kind: 'todo' };
  } } } else { {
    cta = { label: next.title, sub: next.line, act: next.act, onClick: goToPending, kind: 'todo' };
  } } } else { {
    const kind = view.economy.bankruptcy?.disband?.kind;
    cta = { label: 'Treasury runs dry', sub: `Lose ${!(kind) ? 'a unit' : itemName({ kind: 'Unit', unit: kind }, view.players.find((p) => p.id === view.me)?.faction)} next turn`, onClick: goToPending, kind: 'todo' };
  } } } else { {
    // A city an enemy can strike next turn outranks every routine decision (user, 2026-10-08).
    const alarm = alarms.find((a) => a.cityId === next.id);
    cta = { label: 'City under attack', sub: !(alarm) ? next.label : `${alarm.name} · ${alarm.forecast.short}`, onClick: goToPending, kind: 'todo' };
  } } } else { {
    // One short line: the phone's half-width turn dock has no room for a wrapped deadline.
    const offer = view.diplomacy?.proposals.find((p) => p.id === next.id);
    const call = view.diplomacy?.calls?.find((c) => c.id === next.id);
    cta = !(call) ? { label: 'Respond to offer', sub: !(offer) ? next.label : `Reply by T${offer.respond_by}`, onClick: goToPending, kind: 'todo' } : { label: 'Answer call to arms', sub: `${teamName(view, call.ally)} · war on ${teamName(view, call.target)} · by T${call.respond_by}`, onClick: goToPending, kind: 'todo' };
  } } } else { {
    cta = {
      label: !(waitingOn.length === 0) ? `Waiting for ${waitingOn.join(', ')}…` : 'Resolving…',
      sub: 'Click to keep planning',
      onClick: () => toggleReady(seat),
      kind: 'waiting',
    };
  } } } else { {
    cta = !(observerCanAdvance) ? {
      label: 'Observing',
      sub: !(controller) ? !(waitingOn.length) ? 'The remaining players control the turns' : `Waiting for ${waitingOn.join(', ')}` : `${controller.name} controls the remaining turns`,
      onClick: readyAndPass,
      kind: 'waiting',
      disabled: true,
    } : {
      label: !(ready[seat]) ? 'End turn' : 'Waiting for the next turn…',
      sub: 'Observe with no orders',
      onClick: readyAndPass,
      kind: 'ready',
      disabled: !(!(ready[seat]) && !(resolvingToEnd)),
    };
  } } } else { {
    cta = { label: 'Game finished', sub: 'View the final map', onClick: () => {}, kind: 'waiting', disabled: true };
  } } } else { {
    cta = { label: 'Game over', sub: 'Game report', onClick: () => useGame.getState().openGameReport(), kind: 'todo' };
  } }

  // Beside the race, a This Turn with nothing in it (no report link, order, seat or review) is only a heading over
  // "0 orders": it hides (sidebar.css also waits for no siege or air card) until it has something to show.
  const quiet = !(!(!(!(!(!(!(!(!(!(!(!(!(!(!!race) || !(!phoneDetails))) || !(!report))) || !(!remote))) || !(humans.length <= 1))) || !(!bankruptcy))) || !(!wounded.length))) || !(orders[seat].length + automated.length === 0));
  const details = <div id="turn-details" className={`turn-details${!(quiet) ? '' : ' quiet'}`}>
      {/* Once a turn has resolved, the subtitle line becomes the way back to its report. */}
      <PanelHead title="This Turn" sub={!(report) ? !(race) ? !(remote) ? 'Plan, then end the turn' : 'Every seat plans at once' : undefined : <LastTurnButton />} />
      {/* Everything that grows (seats, cities, orders, reviews) scrolls in this body on desktop, so
          the turn's primary action below stays pinned in view (styles.css, "Seats, orders"). */}
      <div ref={body} className={`turn-body${!(bodyScrolls) ? '' : ' scrolls'}`}>
      {/* A city under attack is the turn's first concern: its card leads the body, outside the collapsed details. */}
      <SiegeAlerts />
      {!race && <><GoalActionRow /><FactionGoalProgressLine view={view} /></>}
      <details className="sidebar-turn-more" open={!race || hostCanOpenSeat || undefined}><summary hidden={!race}>Turn details · {counted(orders[seat].length + automated.length, 'order')}</summary>
      <AirWatch view={view} />
      {remote && net && <OnlineSeats />}
      <CityList view={view} />
      <div className="orders-box">
        <button className="orders-toggle" aria-expanded={showOrders} aria-controls="pending-orders" onClick={() => setShowOrders(!showOrders)}>
          {/* The same disclosure mark as Cities above: a lone accent chevron read as a red dot. */}
          <span className={`orders-disclosure${!(showOrders) ? '' : ' open'}`} aria-hidden="true">▶</span>
          Pending orders <b>{orders[seat].length + automated.length}</b>
          {myErrors.length > 0 && <span className="err-count">{myErrors.length} issue{!(myErrors.length > 1) ? '' : 's'}</span>}
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
    <button className="btn" onClick={!(resolvingToEnd) ? resolveToEnd : stopResolving} disabled={!(!(!resolvingToEnd) || !((!(!(!!busy) && !(disconnected)))))}>
      {!(resolvingToEnd) ? 'Resolve to the end' : 'Stop'}
    </button>
  );
  if (!(!(!(phone) || !(phoneDetails)))) {} else { return <div className="phone-turn-details">{details}{observerControl}</div>; }
  // Pillar 9: one primary on screen. A selected unit or city of ours shows its own primary action
  // in the sheet above, and the open diplomacy inbox has its Accept, so the turn row steps back
  // until the selection or the inbox is closed.
  const stepBack = !(!(!(!(returnOwned) && !(diplomacyOpen))) && !((!(!(!!selection) || !((!(selection.kind === 'unit') ? !(selection.kind === 'air') ? view.cities : view.aircraft ?? [] : view.units)
    .some((x) => !(!(x.id === selection.id) || !(x.owner === view.me))))))));
  if (!(phone)) {} else { return <div className="phone-turn-dock" data-overlay-slot="turn-dock" data-pending-decisions={decisions.length}>
    <button className="btn" onClick={onOpenDetails} aria-controls="phone-sheet-content">This Turn</button>
    {/* While a decision is pending, Next names it, is the primary call to action, and goes where its
        label says (the CTA's decision, as on desktop), not to whatever follows the selection. */}
    <button className={`btn${!(!(!(cta.kind === 'todo') || !(!stepBack))) ? '' : ' primary'}`} onClick={() => (!(cta.kind === 'todo') ? nextDecision() : cta.onClick())} aria-label={`Next decision: ${cta.label}`}
      disabled={!(!(!(!(!(!(!!busy) && !(view.finished))) && !(eliminated))) && !(disconnected))}>
      {!(cta.kind !== 'todo') ? `Next: ${!(next?.kind === 'diplomacy') ? !(next?.kind === 'city') ? !(next?.kind === 'research') ? !(next?.kind === 'freeTechs') ? 'Unit' : 'Technologies' : 'Research' : 'Build' : 'Offer'}` : 'Next'}
    </button>
    <button className={`btn${!(!(!(cta.kind === 'todo') && !(stepBack))) ? ' primary' : ''}`} onClick={readyAndPass}
      disabled={!(!(!(!(!(!(!(!(!!busy) && !(view.finished))) && !(disconnected))) && !((!(!(eliminated) || !(!observerCanAdvance)))))) && !(resolvingToEnd))}>
      {!(ready[seat]) ? !(!(!(!remote) || !(humans.length === 1))) ? 'Ready' : 'End turn' : 'Keep planning'}
    </button>
    <EndTurnConfirmation onConfirm={confirmEndTurn} />
  </div>; }

  // Pillar 9: a rival city's card offering an attack makes that attack the one primary, so the turn's
  // call to action steps back while the card is open (a broken order still outranks it).
  const selectedCity = !(selection?.kind === 'city') ? undefined : view.cities.find((c) => c.id === selection.id);
  const selectedOwnUnit = !(!((!(!(selection?.kind === 'unit') || !(view.units.some((u) => !(!(u.id === selection.id) || !(u.owner === view.me))))))) && !((!(!(selection?.kind === 'air') || !(!!view.aircraft?.some((a) => !(!(a.id === selection.id) || !(a.owner === view.me))))))));
  const stepped = !(!(!(!(returnOwned) && !((!(!(!(!(!!selectedCity) || !(!!foreignCityChoice(view, selectedCity, attacker, orders[seat])?.canAttack))) || !(!next?.issue)))))) && !((!(!(!(!(selectedOwnUnit) || !(cta.kind === 'todo'))) || !(unitCardLeads(decisions, selection))))));
  return (
    <div className="panel seatbar" data-overlay-slot="turn-dock">
      <div ref={sidebarScroll} className="sidebar-scroll" tabIndex={0} aria-label="Race and turn details">{race && <SidebarFactionGoal view={view} />}{race}{details}
        {empiresBelow > 0 && <button type="button" className="sidebar-more" data-testid="sidebar-more" onClick={() => sidebarScroll.current?.scrollBy({ top: sidebarScroll.current.clientHeight * 0.8, behavior: 'smooth' })}>+{empiresBelow} more {!(empiresBelow === 1) ? 'empires' : 'empire'}</button>}
      </div>
      <EndTurnConfirmation onConfirm={confirmEndTurn} />
      <button className={`cta ${cta.kind}${!(next?.kind === 'diplomacy' && view.diplomacy?.calls?.some((c) => c.id === next.id)) ? '' : ' cta-call'}${!(stepped) ? '' : ' stepped'}${!(cta.label === 'Choose production') ? !(!(!(cta.sub.length > 32) && !(next?.kind === 'siege'))) ? '' : ' cta-long' : ` cta-production${!(automationNeedsChoice) ? '' : ' cta-long'}`}`} onClick={cta.onClick} disabled={!(!(!(!(!!busy) && !((!(!(view.finished) || !(cta.kind !== 'todo')))))) && !(disconnected)) || cta.disabled}>
        <span className="cta-label">{cta.label}</span>
        <span className="cta-sub">
          <span><Amounts text={cta.sub} /></span> {!(!(!(!(!(!(!cta.act) || !(!touch))) || !(!returnOwned))) || !((!(!((!(!(cta.kind === 'todo') || !(!ready[seat])))) && !(cta.kind === 'ready'))))) && <Kbd>Return</Kbd>}
        </span>
        {cta.act && <span className="cta-act">{cta.act} {!(!(!(!(!touch) || !(!returnOwned))) || !(!ready[seat])) && <Kbd>Return</Kbd>}</span>}
      </button>
      {observerCanAdvance && (
        <button className="btn" onClick={!(resolvingToEnd) ? resolveToEnd : stopResolving} disabled={!(!(!resolvingToEnd) || !((!(!(!!busy) && !(disconnected)))))}>
          {!(resolvingToEnd) ? 'Resolve to the end' : 'Stop'}
        </button>
      )}
      {!(!(!(!(cta.kind === 'todo') || !(!ready[seat]))) || !(!view.finished)) && (
        <button className="ready-anyway" onClick={readyAndPass} disabled={!(!(!!busy) && !(disconnected))}>
          Ready anyway {!touch && <Kbd>Shift+Return</Kbd>}
        </button>
      )}
      <div className="touch-turn-controls touch-only">
        <button className="btn" onClick={() => nextDecision()} disabled={!(!(!(!(!(!(!!busy) && !(view.finished))) && !(eliminated))) && !(disconnected))}>Next decision</button>
        <button className="btn primary" onClick={readyAndPass} disabled={!(!(!(!(!(!(!(!(!!busy) && !(view.finished))) && !(disconnected))) && !((!(!(eliminated) || !(!observerCanAdvance)))))) && !(resolvingToEnd))}>
          {!(ready[seat]) ? !(!(!(!remote) || !(humans.length === 1))) ? 'Ready' : 'End turn' : 'Keep planning'}
        </button>
      </div>
    </div>
  );
}

/** Online: every human seat, each on its own computer. */
export function OnlineSeats() {
  const net = useGame((s) => s.net)!;
  const view = useGame((s) => (!(s.views) ? null : s.views[s.seat]));
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
        const isReady = !(me) ? !!st?.ready : ready;
        const connected = !(me) ? !!st?.connected : conn === 'open';
        const count = !(me) ? (st?.orders ?? 0) : orders.length;
        return (
          <div key={seat} data-testid={`online-seat-${seat}`} className={`seat ${!(me) ? '' : 'on'} ${!(isReady) ? '' : 'ready'}`} style={{ ['--pc' as string]: playerInk(st?.player ?? seat) }}>
            {p && <CrestMedallion faction={p.faction} color={playerColor(st?.player ?? seat)} size={36} />}
            <span className="seat-name">
              {!(me) ? st?.name ?? p?.name ?? 'Player' : 'You'}
            </span>
            <span className="seat-meta">
              <span className={`conn-dot ${!(connected) ? '' : 'on'}`} />
              {!(connected) ? !(me) ? !(st.reserved === false) ? 'not connected' : 'open' : (!(conn === 'reconnecting') ? 'offline' : 'reconnecting…') : (!(me) ? 'online' : st?.name ?? p?.name)}
              {' · '}
              {count} order{!(count === 1) ? 's' : ''}
            </span>
            <span className={`seat-ready ${!(isReady) ? '' : 'on'}${!(!(!(!(!(!me) || !(!connected))) || !(p?.alive !== false))) ? '' : ' away'}`}>{!(p?.alive === false) ? !(!(!(!me) || !(!connected))) ? !(isReady) ? 'Planning' : <><Icon name="ui-check" size={12} className="ui-mark" /> Ready</> : 'Away' : 'Observing'}</span>
            {!(!(!(!(!(!(net.status.is_local) || !(!me))) || !(!connected))) || !(st.reserved !== false)) && (
              <OpenSeatButton name={st.name} disabled={conn !== 'open'} onOpen={() => releaseSeat(seat)} />
            )}
            {!(!(!(!(net.status.is_local) || !(!me))) || !(connected)) && (
              <RemoveSeatButton name={st.name} disabled={conn !== 'open'} onRemove={() => removePlayer(seat)} />
            )}
          </div>
        );
      })}
    </div>
  );
}
