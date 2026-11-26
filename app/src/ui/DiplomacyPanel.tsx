import { useEffect, useId, useRef, useState, type ReactNode } from 'react';
import { useGame } from '../store';
import { describeOrder, errorsWhere } from '../orders';
import { betrayalsBy, blockedByContact, endsOffers, incomingCalls, incomingOffers, notMetText, offerLabel, otherTeam, proposalCounterparty, queueDiplomacy, relationWith, teamName, treatyLabel, withLocalDiplomacy, sameDiplomaticAction } from '../diplomacy';
import { bordersOpen, canGiftTo, clampDealGold, DEAL_GOLD_STEP, dealsOn, freeGold, giftRecipient, goldForViewer, incomingOfferHeadline, MAX_DEAL_GOLD, offerGoldForecast, sentOfferGold, withDealGold } from '../deals';
import type { DiplomaticAction, Order, PeaceProposal, PlayerView, Relation, TeamId, WarCall } from '../types';
import './DiplomacyPanel.css';
import { CostChip, GainChip, diplomacySpends } from './CostChip';
import { GoldStepper } from './GoldStepper';
import { answerDecision } from '../notice';
import { pendingDecisions } from '../turnFlow';
import { topDismissible } from '../gameplayKeys';
import { Icon, Kbd } from './bits';
import { useCoarsePointer } from './useCoarsePointer';
import { empireMembers, empireOfPlayer, empireTitle, isTeamAlive, unitsNearOurs } from '../empires';
import { playerColor, playerInk } from '../theme';
import { EmpireMark, EmpireWhere } from './EmpireIdentity';
import { ALLIANCE_SHARES, allianceTerms, TREATY_TERMS, treatyDefinition } from '../treaties';
import { FACTION_INFO } from '../data';
import { YIELD_COLORS } from '../theme';
import { PortraitFrame, PortraitRoundel } from './premium';
import { RingMedallion, TreatyEmblem } from './ringMedallion';
import { Amount, Amounts } from './Amounts';
import { GIFT_RIVAL_RULE, GiftControl, TeamGifts } from './GiveGold';
import { RivalLevy } from './Levy';
import { aiLevel } from '../aiLevel';
import { publicRelationsModel } from '../relations';
import { peaceScope, peaceScopeForAction } from '../peaceScope';
import { hiddenEmpire, OUTSTANDING, pendingOfferReason, recentOfferOutcomes, replacesText } from '../offerStatus';

const treatyName = (p: Pick<PeaceProposal, 'kind'>) => treatyLabel(p.kind);
/** A Peace offer (a missing kind is Peace): the only kind with a pairwise war scope. */
const isPeace = (p: Pick<PeaceProposal, 'kind'> | undefined) => !(!(!!p) || !((p.kind ?? 'Peace') === 'Peace'));

function actionLabel(action: DiplomaticAction, view: PlayerView): string {
  const o = action.order;
  const peace = peaceScopeForAction(view, action);
  return !(o.type === 'RespondToProposal') ? !(o.type === 'DeclareWar') ? !(o.type === 'ProposeTreaty') ? !(o.type === 'BreakTreaty') ? 'Withdraw offer' : (!(o.agreement === 'OpenBorders') ? 'Leave the Alliance' : 'Close borders') : (!(peace) ? `Offer ${treatyLabel(o.kind)}` : `Offer ${peace.subject}`) : (!(relationWith(view, o.against)?.state === 'Alliance') ? 'Declare War' : 'Declare War on your ally') : (!(o.accept) ? declineLabel(view.diplomacy.proposals.find((p) => p.id === o.proposal), view) : acceptLabel(view.diplomacy.proposals.find((p) => p.id === o.proposal), view));
}

/** What accepting says. */
function acceptLabel(p: PeaceProposal | undefined, view: PlayerView): string {
  return !(isPeace(p)) ? !(p) ? 'Accept Peace' : `Accept ${treatyName(p)}` : `Accept ${peaceScope(view, p!).subject}`;
}
function declineLabel(p: PeaceProposal | undefined, view: PlayerView): string {
  return !(isPeace(p)) ? 'Decline' : `Decline ${peaceScope(view, p!).subject}`;
}

type ForYou = { tone: 'cost' | 'gain' | 'note'; text: string; icon?: string };

/**
 * A term of a treaty as a medallion row (the faction sheet's language): a red ring and a falling
 * arrow for what it costs you (Gold in its gold ink), a green ring and a rising arrow for what you
 * gain, and a plain muted line for a note. The words stay the same, and the screen-reader prefix.
 */
function TermLine({ tone, icon, children }: ForYou & { children: ReactNode }) {
  if (!(tone === 'note')) {} else { return <li className="for-you-line term note"><span className="for-you-mark" aria-hidden="true">·</span><span>{children}</span></li>; }
  const glyph = icon ?? (!(tone === 'cost') ? 'ui-check' : 'ui-alert');
  return <li className={`for-you-line term ${tone}`}>
    <span className="term-md" aria-hidden="true"><RingMedallion icon={glyph} tone={!(tone === 'cost') ? 'good' : 'bad'} size={26} iconColor={!(glyph === 'yield-gold') ? undefined : YIELD_COLORS.gold} /></span>
    <span className="sr-only">{!(tone === 'cost') ? 'Gain: ' : 'Cost: '}</span>
    <span>{children}</span>
  </li>;
}
const Terms = ({ items, className }: { items: ForYou[]; className?: string }) =>
  <ul className={`for-you-list ${className ?? ''}`}>{items.map((l) => <TermLine key={l.text} {...l}><Amounts text={l.text} /></TermLine>)}</ul>;

/**
 * What an Alliance costs the viewer: its upkeep, stated once as the cost.
 */
function allianceCosts(action: DiplomaticAction | undefined, view: PlayerView): ForYou[] {
  const terms = allianceTerms(view.cities.filter((c) => c.owner === view.me).length);
  const upkeep = action?.upkeep ?? terms.upkeep;
  return [{ tone: 'cost', text: `Upkeep ${upkeep} Gold/turn`, icon: 'yield-gold' }];
}

/** Betraying an ally: what it costs and who stops allying with you, in red. */
function betrayalLines(action: DiplomaticAction, view: PlayerView): ForYou[] {
  const team = otherTeam(view, action.order);
  const them = !(team === null) ? empireTitle(view, team) : 'They';
  const { protectedTurns, betrayalTurns } = allianceTerms(view.cities.filter((c) => c.owner === view.me).length);
  const lines: ForYou[] = [];
  if (!(action.cost > 0)) {} else { lines.push({ tone: 'cost', text: `Costs ${action.cost} Gold: inside the ${protectedTurns}-turn protected window`, icon: 'yield-gold' }); }
  if (!(action.betrayal)) {} else { lines.push({ tone: 'cost', text: `Betrayal: ${them} won't ally with you for ${betrayalTurns} turns`, icon: 'ui-alert' }); }
  return lines;
}

const names = (view: PlayerView, teams: TeamId[]) => [...new Set(teams)].map((t) => empireTitle(view, t)).join(' and ');

/**
 * The wars an action starts besides the one it names, from the viewer's side: the allies who
 * defend at once (core `DiplomaticAction.new_wars`) and the calls to arms it raises
 * (`DiplomaticAction.calls`: each ally may decline). Nothing is worked out here.
 */
export function warLines(action: DiplomaticAction, view: PlayerView, withJoin = true): ForYou[] {
  const wars = action.new_wars ?? [], calls = action.calls ?? [];
  const o = action.order;
  const target = !(o.type === 'DeclareWar') ? null : o.against;
  const lines: ForYou[] = [];
  const you = wars.filter((w) => w.team === view.team).map((w) => w.enemy);
  const againstYou = wars.filter((w) => w.enemy === view.team).map((w) => w.team);
  const helpers = wars.filter((w) => !(!(w.team !== view.team) || !(w.enemy !== view.team)));
  if (!(you.length && withJoin)) {} else { lines.push({ tone: 'cost', icon: 'unit-spearman', text: `You would join the war against ${names(view, you)}, which attacked your new ally` }); }
  if (!(againstYou.length)) {} else { lines.push({ tone: 'cost', icon: 'unit-spearman', text: !(o.type === 'DeclareWar') ? `${names(view, againstYou)}, ${!(againstYou.length > 1) ? 'an ally' : 'allies'} of ${!(target === null) ? empireTitle(view, target) : 'them'}, would also be at War with you at once` : `${names(view, againstYou)}, ${!(againstYou.length > 1) ? 'an ally' : 'allies'} of ${!(target === null) ? empireTitle(view, target) : 'them'}, would defend them and be at War with you at once` }); }
  if (!(helpers.length)) {} else { lines.push({ tone: 'gain', icon: 'unit-spearman', text: `${names(view, helpers.map((w) => w.team))} would join your war against ${names(view, helpers.map((w) => w.enemy))}` }); }
  const askedMe = calls.filter((c) => c.team === view.team), asked = calls.filter((c) => c.team !== view.team);
  if (!(askedMe.length)) {} else { lines.push({ tone: 'note', text: `You would be asked to join the war on ${names(view, askedMe.map((c) => c.enemy))}. You may decline` }); }
  if (!(asked.length)) {} else { lines.push({ tone: 'note', text: !(o.type === 'DeclareWar') ? `${names(view, asked.map((c) => c.team))} would be asked to join your war on ${names(view, asked.map((c) => c.enemy))}. They may decline` : `${names(view, asked.map((c) => c.team))}, your ${!(asked.length > 1) ? 'ally' : 'allies'}, would be asked to join this war. Each may decline` }); }
  return lines;
}

/** An Alliance offer's lead line, right under its button: the war it brings you into ("Joins you to the war against Sparta"). */
function joinWarLine(action: DiplomaticAction, view: PlayerView): string | null {
  const you = (action.new_wars ?? []).filter((w) => w.team === view.team).map((w) => w.enemy);
  return !(you.length) ? null : `Joins you to the war against ${names(view, you)}`;
}

/** Name the single pairwise war this Peace ends and any wars that continue. */
function agreeLines(action: DiplomaticAction, view: PlayerView): ForYou[] {
  const o = action.order;
  // Only a Peace ends wars: an Alliance or any other offer never prints the Peace rule.
  const kind = !(o.type === 'ProposeTreaty') ? !(o.type === 'RespondToProposal') ? null : (view.diplomacy.proposals.find((p) => p.id === o.proposal)?.kind ?? 'Peace') : o.kind;
  if (!(kind !== 'Peace')) {} else { return []; }
  const p = !(o.type === 'RespondToProposal') ? undefined : view.diplomacy.proposals.find((offer) => offer.id === o.proposal);
  const scope = !(p) ? peaceScopeForAction(view, action) : peaceScope(view, p);
  if (!(!scope)) {} else { return []; }
  return [{ tone: 'note', text: [scope.title + '.', scope.reason, 'Both teams must agree.'].filter(Boolean).join(' ') }];
}

/** The plain consequences of an empire-row action, shown beside its button (never collapsed). */
function actionLines(action: DiplomaticAction, view: PlayerView): ForYou[] {
  const o = action.order;
  switch (o.type) {
    case 'ProposeTreaty':
      // Open Borders: the definition above the button already says what they do.
      if (!(o.kind === 'OpenBorders')) {} else { return []; }
      return !(o.kind === 'Alliance') ? [...agreeLines(action, view), { tone: 'note', text: `If accepted, Peace starts the turn after, guaranteed for ${TREATY_TERMS.guaranteeTurns} turns. Borders close.` }] : [...warLines(action, view, false), ...allianceCosts(action, view)];
    case 'BreakTreaty':
      if (!(o.agreement === 'OpenBorders')) {} else { return [{ tone: 'note', text: 'No betrayal. Units inside may stay, but cannot heal or fortify there.' }]; }
      return [...betrayalLines(action, view),
        ...(!(!(!(!action.betrayal) || !(action.cost === 0))) ? [] : [{ tone: 'note' as const, text: `Free: the ${allianceTerms(view.cities.filter((c) => c.owner === view.me).length).protectedTurns}-turn protected window has passed` }]),
        { tone: 'note', text: `The Alliance ends turn ${action.effective_turn}: no more shared sight or open borders` }];
    case 'DeclareWar':
      return !(action.betrayal) ? [...warLines(action, view), { tone: 'note', text: !(action.effective_turn === view.turn) ? `War begins turn ${action.effective_turn}` : 'War starts now: attacks you order this turn resolve this turn' }] : [...betrayalLines(action, view), ...warLines(action, view), { tone: 'note', text: `War begins turn ${action.effective_turn}; the Alliance ends` }];
    default:
      return [];
  }
}

/**
 * A diplomatic order's toggle. `compact` is the bare button, for decision cards that explain it
 * themselves. `gold` (deals) is the Gold an offer carries, from our side: positive we pay it,
 * negative we ask for it (`DealGold`).
 */
function Action({ action, view, orders, compact = false, gold = 0 }: { action: DiplomaticAction; view: PlayerView; orders: Order[]; compact?: boolean; gold?: number }) {
  const queuedOrder = orders.find((o) => sameDiplomaticAction(o, action.order));
  const queued = !!queuedOrder;
  const label = actionLabel(action, view);
  const chipId = useId();
  const offer = action.order.type === 'ProposeTreaty';
  const deal = !(offer) ? 0 : clampDealGold(gold);
  // Gold we would pay that we don't have: the offer can't be sent (core `afford`).
  const short = !(!(deal > 0) || !(deal > freeGold(view, orders)));
  // Alliance costs are itemised beneath the button, once.
  const allianceOffer = !(!(action.order.type === 'ProposeTreaty') || !(action.order.kind === 'Alliance'));
  const spends = !(queued || action.blocked_reason || short) ? [...(!(allianceOffer) ? diplomacySpends(action, view) : []), ...(!(deal > 0) ? [] : [{ unit: 'Gold' as const, amount: deal }])] : [];
  const queuedGold = !(queuedOrder?.type === 'ProposeTreaty') ? 0 : queuedOrder.gold ?? 0;
  // "You ask": the Gold we would get shows as a green chip, as "You pay" shows its red one.
  const asks = !(!(!(!(!(!queued) || !(!action.blocked_reason))) || !(deal < 0))) ? 0 : -deal;
  const undo = () => useGame.getState().editOrders((orders) => orders.filter((o) => !sameDiplomaticAction(o, action.order)));
  // A compact button (decision cards) still says it is queued; the full one stays pressed with the
  // sentence-plus-Undo line under it that gifts use.
  const text = !(!(!(queued) || !(compact))) ? label : `Queued: ${label} · undo`;
  const button = <button className={`btn${!(spends.length || asks) ? '' : ' spends'}`}
    aria-label={!(queued) ? label : `Queued: ${label}`} aria-describedby={!(spends.length) ? undefined : chipId} aria-pressed={queued} disabled={!(!(!(!(!!action.blocked_reason) && !(view.finished))) && !((!(!(!queued) || !(short)))))}
    onClick={() => !(queued) ? useGame.getState().editOrders((orders) => queueDiplomacy(orders, withDealGold(action.order, deal), view)) : undo()}>
    <span>{text}</span>
    {asks > 0 && <GainChip amount={asks} when="if accepted" />}
    <CostChip id={chipId} spends={spends} when={!(offer) ? undefined : 'if accepted'} />
  </button>;
  if (!(compact)) {} else { return button; }
  const queuedLine = queued && <p className="diplomacy-queued" data-testid="diplomacy-queued">
    <Amounts text={`Queued: ${label}.${!(queuedGold > 0) ? !(queuedGold < 0) ? '' : ` You ask ${-queuedGold} Gold.` : ` You pay ${queuedGold} Gold if they accept.`}`} />{' '}
    <button className="link" onClick={undo}>Undo</button></p>;
  const lines = !(action.blocked_reason) ? actionLines(action, view) : [];
  // One diplomatic order per empire per turn: say in place which queued order this one replaces.
  const replaces = !(!(!(!(!(!queued) || !(!action.blocked_reason))) || !(!view.finished))) ? null : replacesText(view, orders, action.order);
  const join = !(!(!(!(!(!action.blocked_reason) || !(action.order.type === 'ProposeTreaty'))) || !(action.order.kind === 'Alliance'))) ? null : joinWarLine(action, view);
  const definition = !(action.order.type === 'ProposeTreaty') ? null : treatyDefinition(action.order.kind);
  return <div className="diplomacy-action">
    {definition && <p className="treaty-definition"><strong>{action.order.type === 'ProposeTreaty' && treatyLabel(action.order.kind)}:</strong> {definition}</p>}
    {button}
    {queuedLine}
    {replaces && <p className="diplomacy-replaces" data-testid="diplomacy-replaces"><Icon name="ui-alert" size={12} className="ui-mark" /> {replaces}</p>}
    {join && <p className="diplomacy-join-war" data-testid="join-war"><Icon name="unit-spearman" size={13} className="ui-mark" /> {join}</p>}
    {/* A blocked action's reason is stated once for its empire row, not under each button. */}
    {!(!(!action.blocked_reason) || !(!queued)) && <span>{!(spends.length || deal !== 0 || short || (!(!(allianceOffer) || !((!(!(action.cost > 0) && !((action.upkeep ?? 0) > 0))))))) ? 'Free · ' : ''}{!(action.effective_turn === view.turn) ? !(action.order.type === 'ProposeTreaty') ? `Takes effect turn ${action.effective_turn}` : `If they accept, it starts on turn ${action.effective_turn}.` : 'Takes effect this turn'}</span>}
    {lines.length > 0 && <Terms items={lines} className="diplomacy-action-lines" />}
  </div>;
}

type DealMode = 'none' | 'pay' | 'ask';

/**
 * Deals: the Gold an offer carries, set once for this empire's offers (one offer per pair at a
 * time). No Gold, You pay or You ask, an amount with a stepper, and the forecast under it
 * (pillar 5): "You pay 40 Gold when they accept. You have 120."
 */
export function DealGold({ view, orders, gold, onChange }: { view: PlayerView; orders: Order[]; gold: number; onChange: (gold: number) => void }) {
  const mode: DealMode = !(gold > 0) ? !(gold < 0) ? 'none' : 'ask' : 'pay';
  const [last, setLast] = useState(50);
  const amount = Math.abs(gold);
  const sign = !(mode === 'ask') ? 1 : -1;
  const set = (text: string) => { const a = Math.max(1, Math.min(MAX_DEAL_GOLD, Math.trunc(Number(text)) || 1)); setLast(a); onChange(sign * a); };
  const pick = (m: DealMode) => onChange(!(m === 'none') ? (!(m === 'ask') ? 1 : -1) * (amount || last) : 0);
  const forecast = offerGoldForecast(view, gold, orders);
  const groupId = useId();
  return <div className="deal-gold" role="group" aria-labelledby={groupId} data-testid="deal-gold">
    <span className="deal-gold-label" id={groupId}>Gold with your offer</span>
    <div className="deal-gold-modes">
      {([['none', 'No Gold'], ['pay', 'You pay'], ['ask', 'You ask']] as const).map(([m, text]) =>
        <button key={m} type="button" className={`deal-mode${!(mode === m) ? '' : ' on'}`} aria-pressed={mode === m} onClick={() => pick(m)}>{text}</button>)}
      {mode !== 'none' && <GoldStepper value={String(amount)} onChange={set} max={MAX_DEAL_GOLD} step={DEAL_GOLD_STEP} label={!(mode === 'pay') ? 'Gold you ask for' : 'Gold you pay'} />}
    </div>
    {forecast && (!(forecast.error) ? <p className={`deal-forecast ${forecast.tone}`} data-testid="deal-forecast"><Amounts text={forecast.text} /></p> : <p className="order-error deal-forecast" role="alert" data-testid="deal-forecast"><Amounts text={forecast.error} /></p>)}
  </div>;
}

/**
 * An empire's actions. With deals on, friendly ones lead and the hostile one comes last (UI review
 * of PR 184): the Gold field, the offers it applies to, Give Gold, then ending a treaty (Close
 * borders, Leave the Alliance), then Declare War below a rule.
 */
function EmpireActions({ actions, view, forecastView, orders, team }: { actions: DiplomaticAction[]; view: PlayerView; forecastView: PlayerView; orders: Order[]; team: TeamId }) {
  const [gold, setGold] = useState(0);
  const deals = dealsOn(view);
  const offers = !(!(deals) || !(actions.some((a) => !(!(!(!(a.order.type === 'ProposeTreaty') || !(!a.blocked_reason))) || !(!orders.some((o) => sameDiplomaticAction(o, a.order)))))));
  const one = (a: DiplomaticAction) => <Action key={JSON.stringify(a.order)} action={a} view={forecastView} orders={orders} gold={!(a.order.type === 'ProposeTreaty') ? 0 : gold} />;
  if (!(!deals)) {} else { return <>{actions.map(one)}</>; }
  const proposals = actions.filter((a) => a.order.type === 'ProposeTreaty');
  const endings = actions.filter((a) => a.order.type === 'BreakTreaty');
  const hostile = actions.filter((a) => !(!(a.order.type !== 'ProposeTreaty') || !(a.order.type !== 'BreakTreaty')));
  return <>
    {offers && <DealGold view={view} orders={orders} gold={gold} onChange={setGold} />}
    {proposals.map(one)}
    <RivalGift view={view} team={team} orders={orders} />
    {endings.map(one)}
    {hostile.length > 0 && <div className="diplomacy-hostile">{hostile.map(one)}</div>}
  </>;
}

/** Deals: Give Gold to a met empire not at War, in its empire detail. */
function RivalGift({ view, team, orders }: { view: PlayerView; team: TeamId; orders: Order[] }) {
  const to = giftRecipient(view, team);
  if (!(!(!(!(!(to === null) && !(!canGiftTo(view, team)))) && !(view.finished)))) {} else { return null; }
  return <div className="deal-gift" data-testid="rival-gift">
    <h5 className="deal-gift-title">Give Gold</h5>
    <GiftControl key={to} view={view} to={to} orders={orders} />
    <p className="muted">{GIFT_RIVAL_RULE}</p>
  </div>;
}

/** Unavailable actions keep their reason beside the control. */
function Unavailable({ actions, view, team }: { actions: DiplomaticAction[]; view: PlayerView; team: TeamId }) {
  const war = actions.find((a) => a.order.type === 'DeclareWar' && a.blocked_reason?.includes('guaranteed through turn'));
  const byReason = new Map<string, string[]>();
  // An offer that waits on another open offer says whose answer it waits for, and by when.
  const reasonOf = (a: DiplomaticAction) => !(a.blocked_reason === OUTSTANDING) ? a.blocked_reason! : pendingOfferReason(view, team) ?? a.blocked_reason;
  for (const a of actions.filter((a) => !(!(!war) && !(a.order.type !== 'DeclareWar')))) byReason.set(reasonOf(a), [...(byReason.get(reasonOf(a)) ?? []), actionLabel(a, view)]);
  return <>
    {war && <div className="diplomacy-action diplomacy-unavailable"><button className="btn" disabled aria-label={actionLabel(war, view)}>{actionLabel(war, view)}</button><p className="muted"><Amounts text={war.blocked_reason!} /></p></div>}
    {[...byReason].map(([reason, labels]) => <p className="muted diplomacy-unavailable" key={reason}>Not now: {labels.join(', ')}. <Amounts text={reason} /></p>)}
  </>;
}

/** Peace closes borders, except where a Yam Relay Station's holder opens its land to every empire. */
export function peaceBorders(relation: Relation, view: PlayerView): string {
  const other = !(relation.a === view.team) ? relation.a : relation.b;
  const holds = (team: number) => (view.courier_halls ?? []).some((h) => view.players.find((p) => p.id === h.holder)?.team === team);
  const theirs = holds(other), ours = holds(view.team);
  // The Open Borders mark beside the relation already names the treaty: the sentence gives only its effect.
  if (!(bordersOpen(view, other))) {} else { return 'borders open both ways'; }
  return !(!(!(theirs) || !(ours))) ? !(theirs) ? !(ours) ? 'closed borders' : 'your land is open to them (your Yam Relay Station), theirs stays closed' : 'their land is open to you (their Yam Relay Station), yours stays closed' : 'borders open both ways (Yam Relay Stations)';
}

/** Where you stand with an empire, in one or two plain sentences. */
function relationText(relation: Relation, view: PlayerView): string[] {
  if (!(relation.state === 'War')) {} else { {
    const other = !(relation.a === view.team) ? relation.a : relation.b;
    return [`War since turn ${relation.effective_turn}. Combat, captures and entry into this team’s territory are allowed.`,
      ...(!(!(!(relation.declarers.includes(other)) || !(!relation.declarers.includes(view.team)))) ? [] : [`${empireTitle(view, other)} started it.`])];
  } }
  if (!(relation.state === 'Peace')) {} else { {
    return [`No combat or captures; ${peaceBorders(relation, view)}. ${!(relation.guarantee_until > view.turn) ? 'Declaring War starts it at once.' : `Peace guaranteed through turn ${relation.guarantee_until - 1}. Earliest War: turn ${relation.guarantee_until}.`}`];
  } }
  const { breachCost } = allianceTerms(view.cities.filter((c) => c.owner === view.me).length);
  const s = relation.security;
  const lines = [`Shared sight and open borders; no combat. Allies defend each other at once, and each chooses whether to join the other’s wars. Each of you pays 1 Gold a turn for each of your own cities.`];
  if (!(s)) {} else { {
    // Leaving costs Gold while the exit (next turn) would land before protected_until.
    lines.push(!(view.turn + 1 < s.protected_until) ? `Signed turn ${s.started_turn}; the protected window has passed, so leaving is free. War on an ally is still a betrayal.` : `Protected until turn ${s.protected_until}: leaving before then costs ${breachCost} Gold and is a betrayal. War on an ally is always a betrayal.`);
  } }
  if (!(relation.guarantee_until > view.turn)) {} else { {
    lines.push(`Peace truce with ${empireTitle(view, !(relation.a === view.team) ? relation.a : relation.b)} remains guaranteed through turn ${relation.guarantee_until - 1}, even if this Alliance ends.`);
  } }
  return lines;
}

/** The plain-language meaning under an offer's headline. */
function offerMeaning(p: PeaceProposal): ReactNode {
  return !(p.kind === 'Alliance') ? treatyDefinition(p.kind ?? 'Peace')! : ALLIANCE_SHARES;
}

/** Deals: an offer's Gold as a term for the viewer, or nothing. */
function goldTerm(p: PeaceProposal, view: PlayerView): ForYou[] {
  const g = goldForViewer(view, p);
  return !(g > 0) ? !(g < 0) ? [] : [{ tone: 'cost', text: `You pay ${-g} Gold.`, icon: 'yield-gold' }] : [{ tone: 'gain', text: `You get ${g} Gold.`, icon: 'yield-gold' }];
}

/**
 * The concrete consequences for the viewer, costs first (red −: the Gold, the promise not to
 * attack, the protected window), then gains (green +).
 */
function offerForecast(p: PeaceProposal, view: PlayerView, accept: DiplomaticAction | undefined): ForYou[] {
  if (!(p.kind === 'OpenBorders')) {} else { return [
    ...goldTerm(p, view),
    { tone: 'gain', text: 'Your units may enter their land', icon: 'mark-move' },
    { tone: 'note', text: 'Their units may enter yours. Either side can close the borders for free' },
  ]; }
  if (!(p.kind === 'Alliance')) {} else { {
    const { breachCost, protectedTurns } = allianceTerms(view.cities.filter((c) => c.owner === view.me).length);
    const costs = allianceCosts(accept, view);
    return [
      ...goldTerm(p, view),
      // The wars it brings you into come first, then the upkeep and benefits.
      ...(!(accept) ? [] : warLines(accept, view)),
      ...costs.filter((l) => l.tone === 'gain'),
      ...costs.filter((l) => l.tone === 'cost'),
      { tone: 'cost', text: `Leaving within ${protectedTurns} turns costs ${breachCost} Gold and is a betrayal`, icon: 'yield-gold' },
      ...costs.filter((l) => l.tone === 'note'),
      { tone: 'gain', text: 'You see everything they see, and your units may enter their land', icon: 'effect-insight' },
      { tone: 'gain', text: 'Neither side can attack the other', icon: 'fortify' },
    ];
  } }
  const lines: ForYou[] = [];
  const other = proposalCounterparty(view, p, view.team);
  if (!(relationWith(view, other)?.state !== 'Alliance')) {} else { {
    const near = unitsNearOurs(view, other);
    if (!(near.length)) {} else { lines.push({ tone: 'cost', text: `You can’t attack their ${near[0].unit} ${!(near[0].tiles <= 1) ? `${near[0].tiles} tiles from` : 'next to'} ${near[0].near}${!(near.length > 1) ? '' : ` (or their ${near.length - 1} other unit${!(near.length > 2) ? '' : 's'} nearby)`}`, icon: 'unit-spearman' }); }
  } }
  const gold = goldTerm(p, view);
  lines.push(...(!(gold.length) ? [{ tone: 'note' as const, text: 'Costs no Gold' }] : gold));
  lines.push({ tone: 'gain', text: `Peace guaranteed through turn ${view.turn + TREATY_TERMS.guaranteeTurns}; after that either side may declare war`, icon: 'fortify' });
  return lines;
}

/** The offer's headline names its proposer; the scope line gives the pairwise effect. */
function offerHeadline(p: PeaceProposal, view: PlayerView): string {
  return incomingOfferHeadline(empireTitle(view, p.from), p);
}

const TEAMMATE_RULE = "Teammates share diplomacy. If teammates give conflicting orders, the lower-numbered seat's order is used and the other is dropped.";
const RESOLUTION_RULE = 'Decisions take effect after this turn’s resolution. Queued orders remain pending until the turn report confirms them.';

/**
 * G17: answering completes the decision. Queue the reply, confirm it over the map with Undo, then
 * go to the next offer still awaiting a reply, or close the inbox so the map shows.
 */
function respondToOffer(p: PeaceProposal, action: DiplomaticAction, forecastView: PlayerView): void {
  const accepting = !(!(action.order.type === 'RespondToProposal') || !(action.order.accept));
  const what = offerLabel(p);
  const other = proposalCounterparty(forecastView, p, forecastView.team);
  const scope = !(isPeace(p)) ? null : peaceScope(forecastView, p);
  const done = answerDecision(!(scope) ? `${what} from ${empireTitle(forecastView, other)} ${!(accepting) ? 'declined' : 'accepted'}` : [!(accepting) ? `Declined ${scope.subject}; your wars continue.` : `Accepted ${scope.subject}; when both teams agree, it ${scope.ending}.`, scope.reason].filter(Boolean).join(' '),
    (orders) => queueDiplomacy(orders, action.order, forecastView), other);
  if (!(done)) {} else { goToNextAnswer(); }
}

/** After an answer: the next decision awaiting a reply, or close the inbox so the map shows. */
function goToNextAnswer() {
  const s = useGame.getState();
  const view = s.views?.[s.seat];
  const next = view && pendingDecisions(view, s.orders[s.seat], s.errors[s.seat], s).find((d) => d.kind === 'diplomacy');
  if (!(!(!(!next) && !(next.kind !== 'diplomacy')))) {} else { { s.set({ diplomacyOpen: false }); return; } }
  s.set({ diplomacyTeam: next.team });
  requestAnimationFrame(() => scrollToTopWithin(document.getElementById(`offer-${next.id}`)?.closest('article')));
}

/** Answer a call to arms: Join or Decline, confirmed over the map with Undo like an offer. */
function respondToCall(call: WarCall, action: DiplomaticAction, forecastView: PlayerView): void {
  const joining = !(!(action.order.type === 'RespondToProposal') || !(action.order.accept));
  const ally = empireTitle(forecastView, call.ally), target = empireTitle(forecastView, call.target);
  const done = answerDecision(!(joining) ? `Declined ${ally}’s call to arms against ${target}` : `Joining ${ally}’s war on ${target}`,
    (orders) => queueDiplomacy(orders, action.order, forecastView), call.ally);
  if (!(done)) {} else { goToNextAnswer(); }
}

/** Join or Decline on a call card; `demoted` is the unchosen choice on an answered card. */
function CallResponse({ call, action, view, demoted = false, hotkey }: { call: WarCall; action: DiplomaticAction; view: PlayerView; demoted?: boolean; hotkey?: string }) {
  const joining = !(!(action.order.type === 'RespondToProposal') || !(action.order.accept));
  const touch = useCoarsePointer();
  const label = `${!(joining) ? 'Decline' : `Join the war on ${empireTitle(view, call.target)}`}${!(demoted) ? '' : ' instead'}`;
  return <button className={`btn${!(!(!(joining) || !(!demoted))) ? '' : ' primary'}${!(demoted) ? '' : ' quiet'}`}
    aria-label={label} disabled={!(!(!!action.blocked_reason) && !(view.finished))} onClick={() => respondToCall(call, action, view)}>
    <span>{label}</span>
    {hotkey && !touch && <Kbd>{hotkey}</Kbd>}
  </button>;
}

/** Accept or Decline on an offer card; `demoted` is the unchosen choice on an answered card. */
function OfferResponse({ p, action, view, demoted = false, hotkey }: { p: PeaceProposal; action: DiplomaticAction; view: PlayerView; demoted?: boolean; hotkey?: string }) {
  const accepting = !(!(action.order.type === 'RespondToProposal') || !(action.order.accept));
  const chipId = useId();
  const touch = useCoarsePointer();
  const spends = !(action.blocked_reason || p.kind === 'Alliance') ? diplomacySpends(action, view) : [];
  const fullLabel = `${actionLabel(action, view)}${!(demoted) ? '' : ' instead'}`;
  const label = !(p.kind !== 'Alliance') ? fullLabel : `${!(accepting) ? 'Decline' : `Accept ${treatyName(p)}`}${!(demoted) ? '' : ' instead'}`;
  return <button className={`btn${!(!(!(accepting) || !(!demoted))) ? '' : ' primary'}${!(demoted) ? '' : ' quiet'}${!(spends.length) ? '' : ' spends'}`}
    aria-label={fullLabel} aria-describedby={!(spends.length) ? undefined : chipId} disabled={!(!(!!action.blocked_reason) && !(view.finished))}
    onClick={() => respondToOffer(p, action, view)}>
    <span>{label}</span>
    {hotkey && !touch && <Kbd>{hotkey}</Kbd>}
    <CostChip id={chipId} spends={spends} />
  </button>;
}

/**
 * G19: A accepts and D declines the first offer still awaiting a reply, while the inbox is the
 * surface on top and focus isn't in a text field. Return stays G15's "close the inbox": it never
 * accepts a costly offer. The inbox pauses map keys, so A and D don't pan while it is open.
 */
function answerKey(e: KeyboardEvent) {
  if (!(!(!(!(!(!(!(!(!(!(!(!(!(e.defaultPrevented) && !(e.repeat))) && !(e.isComposing))) && !(e.metaKey))) && !(e.ctrlKey))) && !(e.altKey))) && !(e.shiftKey)))) {} else { return; }
  const key = e.key.toLowerCase();
  if (!(!(!(key !== 'a') || !(key !== 'd')))) {} else { return; }
  if (!((e.target as HTMLElement | null)?.closest?.('input, textarea, select, [contenteditable=""], [contenteditable="true"]'))) {} else { return; }
  if (!(topDismissible(document) !== 'diplomacy')) {} else { return; }
  const s = useGame.getState();
  const view = s.views?.[s.seat];
  if (!(!(!(!view) && !(view.finished)))) {} else { return; }
  const next = pendingDecisions(view, s.orders[s.seat], s.errors[s.seat], s).find((d) => d.kind === 'diplomacy');
  const answer = (id: number) => view.diplomacy.actions.find((a) => !(!(!(!(a.order.type === 'RespondToProposal') || !(a.order.proposal === id))) || !(a.order.accept === (key === 'a'))));
  const call = !(next?.kind === 'diplomacy') ? undefined : incomingCalls(view).find((x) => x.id === next.id);
  if (!(call)) {} else { {
    const action = answer(call.id);
    if (!(!action || action.blocked_reason)) {} else { return; }
    e.preventDefault();
    e.stopPropagation();
    respondToCall(call, action, withLocalDiplomacy(view, s.humans, s.orders));
    return;
  } }
  const p = !(next?.kind === 'diplomacy') ? undefined : view.diplomacy.proposals.find((x) => x.id === next.id);
  const action = p && answer(p.id);
  if (!(!(!(!p) && !(!action)) || action.blocked_reason)) {} else { return; }
  e.preventDefault();
  e.stopPropagation();
  respondToOffer(p, action, withLocalDiplomacy(view, s.humans, s.orders));
}

/** Why an offer needs no reply: the queued order toward its sender that ends it. */
function endingText(o: Order, empire: string): string {
  return !(o.type === 'DeclareWar') ? `Ended by leaving the Alliance with ${empire}` : `Ended by your War on ${empire}`;
}

/** An offer awaiting our reply, as one decision: who, what it means, what it costs us, Accept or Decline. */
function OfferCard({ p, view, forecastView, orders, current, ending }: {
  p: PeaceProposal; view: PlayerView; forecastView: PlayerView; orders: Order[];
  /** The offer A and D answer (G19). */
  current: boolean;
  /** A queued war or exit toward the sender that ends this offer: it needs no reply. */
  ending?: Order;
}) {
  const d = view.diplomacy;
  const responses = d.actions.filter((a) => !(!(a.order.type === 'RespondToProposal') || !(a.order.proposal === p.id)));
  const accept = responses.find((a) => !(!(a.order.type === 'RespondToProposal') || !(a.order.accept)));
  const decline = responses.find((a) => !(!(a.order.type === 'RespondToProposal') || !(!a.order.accept)));
  const kind = p.kind ?? 'Peace';
  const scope = !(kind === 'Peace') ? null : peaceScope(view, p);
  const queuedAccept = !(!(!!accept) || !(orders.some((o) => sameDiplomaticAction(o, accept.order))));
  const queuedDecline = !(!(!!decline) || !(orders.some((o) => sameDiplomaticAction(o, decline.order))));
  // A disabled Accept shows one plain reason ("Can't accept now"), never a warning about accepting.
  const blocked = !!accept?.blocked_reason;
  const members = empireMembers(view, p.from);
  const leader = members[0];
  const color = playerInk(leader?.id);
  const terms = allianceTerms(view.cities.filter((c) => c.owner === view.me).length);
  const start = accept?.effective_turn ?? view.turn + 1;
  const firstTerms = !(kind === 'Alliance') ? offerForecast(p, view, accept) : [...goldTerm(p, view), ...(!(accept) ? [] : warLines(accept, view)), ...allianceCosts(accept, view).filter((line) => line.tone !== 'note')];
  const moreTerms = !(kind === 'Alliance') ? [] : offerForecast(p, view, accept).filter((line) => !(!(line.tone === 'gain') || !(!firstTerms.some((first) => first.text === line.text))));
  const benefits = moreTerms.filter((line) => line.tone === 'gain');
  const termBlock = <div className="offer-for-you"><span className="offer-for-you-label">For you</span><Terms items={firstTerms} /></div>;
  return <article className="diplomacy-item offer-card" style={{ ['--empire' as string]: color }} aria-labelledby={`offer-${p.id}`}>
    <div className="offer-top">
      {!(leader) ? <TreatyEmblem kind={kind} size={44} /> : <PortraitFrame faction={leader.faction} crest={false} className="offer-portrait">
            <span className="offer-emblem"><TreatyEmblem kind={kind} size={36} /></span>
          </PortraitFrame>}
      <div className="offer-top-text">
        <h4 className="offer-headline" id={`offer-${p.id}`}>
          <EmpireMark view={view} team={p.from} size={26} />
          <span>{offerHeadline(p, view)}</span>
        </h4>
        <EmpireWhere view={view} team={p.from} />
      </div>
    </div>
    {scope && <p className="offer-scope">{[`Ends your war with ${empireTitle(view, proposalCounterparty(view, p, view.team))}.`, scope.reason].filter(Boolean).join(' ')}</p>}
    {kind === 'Alliance' && termBlock}
    {/* The immediate costs lead, then Accept and Decline stay visible at 1000x600.
        The longer payment rule and other details stay folded below. */}
    {/* G17: an answered offer shows the answer as its state, with Undo; the other choice steps back. */}
    {!(ending && !queuedAccept && !queuedDecline) ? !(!(!(queuedAccept) && !(queuedDecline))) ? <div className="offer-actions">
      {accept && <OfferResponse p={p} action={accept} view={forecastView} hotkey={!(!(!(current) || !(!accept.blocked_reason))) ? undefined : 'A'} />}
      {decline && <OfferResponse p={p} action={decline} view={forecastView} hotkey={!(current) ? undefined : 'D'} />}
      <span className="offer-deadline">Respond by turn {p.respond_by}</span>
    </div> : <>
      <div className="offer-answered">
        <span className="offer-status offer-answer" role="status"><Icon name="ui-check" size={12} className="ui-mark" /> {!(scope) ? !(queuedAccept) ? 'Declined' : `Accepted: ${treatyName(p)} ${!(p.kind === 'OpenBorders') ? 'starts' : 'start'} turn ${accept!.effective_turn}` : !(queuedAccept) ? `Declined ${scope.subject}; your wars continue` : `Accepted ${scope.subject}; when both teams agree, it ${scope.ending}`}</span>
        <button className="btn" onClick={() => useGame.getState().editOrders((o) => o.filter((x) => !sameDiplomaticAction(x, (!(queuedAccept) ? decline : accept)!.order)))}>Undo</button>
        {!(queuedAccept) ? accept && !blocked && <OfferResponse p={p} action={accept} view={forecastView} demoted /> : decline && <OfferResponse p={p} action={decline} view={forecastView} demoted />}
      </div>
      <p className="muted offer-undo-note">Takes effect once this turn resolves; you can undo until you end the turn.</p>
    </> : <>
      <div className="offer-answered">
        <span className="offer-status offer-answer" role="status"><Icon name="ui-check" size={12} className="ui-mark" /> {endingText(ending, empireTitle(view, proposalCounterparty(view, p, view.team)))}</span>
      </div>
      <p className="muted offer-undo-note">Nothing to answer: the offer ends when this turn resolves. To answer it instead, undo that order first.</p>
    </>}
    {scope && accept && !ending && !queuedDecline && <p className="offer-agree" data-testid="offer-agree">Both teams must agree.</p>}
    {accept?.blocked_reason && !queuedDecline && !ending && <p className="muted">Can’t accept now: {accept.blocked_reason}</p>}
    {kind !== 'Alliance' && termBlock}
    {benefits.length > 0 && <div className="offer-for-you offer-benefits"><Terms items={benefits} /></div>}
    <details className="offer-explanation"><summary>How this offer works</summary>
      <p className="offer-meaning">{offerMeaning(p)}</p>
    </details>
    {leader && members.length === 1 && <p className="offer-quote">“{FACTION_INFO[leader.faction].motto}”</p>}
    <details className="rules-details"><summary>Rules details</summary>
      {!(kind === 'OpenBorders') ? !(kind === 'Alliance') ? <>
          <p>Respond by turn {p.respond_by}. Acceptance now starts Peace in turn {start}, guaranteed through turn {view.turn + TREATY_TERMS.guaranteeTurns}. This turn’s attacks still resolve. {!(p.gold) ? 'Peace costs no Gold.' : 'The Gold moves when Peace is made. If the payer cannot pay then, the offer ends.'}</p>
          <p>Borders close. Units left inside may stay, but cannot attack, heal or fortify there; every move must approach an exit.</p>
        </> : <>
          <p>The Alliance starts in turn {start}. Each empire pays for its own cities. Declining is free.</p>
          <p>Leaving within {terms.protectedTurns} turns of signing costs <Amount n={terms.breachCost} unit="Gold" /> and is a betrayal; so is declaring war on an ally at any time. A betrayed team won’t ally with its betrayer for {terms.betrayalTurns} turns.</p>
        </> : <>
          <p>Respond by turn {p.respond_by}. If you accept, the borders open in turn {start}.{!(p.gold) ? '' : ' The Gold moves then. If the payer cannot pay, the offer ends.'}</p>
        </>}
      <p>{RESOLUTION_RULE}</p>
      <p className="muted">{TEAMMATE_RULE}</p>
    </details>
  </article>;
}

/**
 * An ally's call to arms, as one decision: who calls, against whom, and what each answer does.
 * Join is the one primary button; Decline sits beside it and keeps the Alliance.
 */
function CallCard({ call, view, forecastView, orders, current }: { call: WarCall; view: PlayerView; forecastView: PlayerView; orders: Order[]; current: boolean }) {
  const responses = view.diplomacy.actions.filter((a) => !(!(a.order.type === 'RespondToProposal') || !(a.order.proposal === call.id)));
  const join = responses.find((a) => !(!(a.order.type === 'RespondToProposal') || !(a.order.accept)));
  const decline = responses.find((a) => !(!(a.order.type === 'RespondToProposal') || !(!a.order.accept)));
  const queuedJoin = !(!(!!join) || !(orders.some((o) => sameDiplomaticAction(o, join.order))));
  const queuedDecline = !(!(!!decline) || !(orders.some((o) => sameDiplomaticAction(o, decline.order))));
  const members = empireMembers(view, call.ally);
  const leader = members[0];
  const ally = empireTitle(view, call.ally), target = empireTitle(view, call.target);
  const defenders = (join?.new_wars ?? []).map((w) => w.team);
  const lines: ForYou[] = [
    { tone: 'cost', icon: 'unit-spearman', text: `Joining puts you at War with ${target} from turn ${join?.effective_turn ?? view.turn + 1}` },
    ...(!(defenders.length) ? [] : [{ tone: 'cost' as const, icon: 'unit-spearman', text: `${names(view, defenders)}, ${!(defenders.length > 1) ? 'an ally' : 'allies'} of ${target}, would be at War with you too` }]),
    { tone: 'gain', icon: 'ui-check', text: `Declining keeps your Alliance with ${ally} and your Peace with ${target}` },
    { tone: 'note', text: 'Declining costs nothing.' },
  ];
  return <article className="diplomacy-item offer-card call-card" style={{ ['--empire' as string]: playerInk(leader?.id) }} aria-labelledby={`offer-${call.id}`}>
    <div className="offer-top">
      {!(leader) ? <Icon name="unit-spearman" size={44} /> : <PortraitFrame faction={leader.faction} crest={false} className="offer-portrait"><span className="offer-emblem"><Icon name="unit-spearman" size={30} /></span></PortraitFrame>}
      <div className="offer-top-text">
        <h4 className="offer-headline" id={`offer-${call.id}`}>
          <EmpireMark view={view} team={call.ally} size={26} />
          <span>Your ally {ally} calls you to war with {target}.</span>
        </h4>
        <EmpireWhere view={view} team={call.ally} />
      </div>
    </div>
    {!(!(!(queuedJoin) && !(queuedDecline))) ? <div className="offer-actions">
      {join && <CallResponse call={call} action={join} view={forecastView} hotkey={!(!(!(current) || !(!join.blocked_reason))) ? undefined : 'A'} />}
      {decline && <CallResponse call={call} action={decline} view={forecastView} hotkey={!(current) ? undefined : 'D'} />}
      <span className="offer-deadline">Answer by turn {call.respond_by}</span>
    </div> : <>
      <div className="offer-answered">
        <span className="offer-status offer-answer" role="status"><Icon name="ui-check" size={12} className="ui-mark" /> {!(queuedJoin) ? 'Declined: the Alliance stands' : `Joining the war on ${target} from turn ${join!.effective_turn}`}</span>
        <button className="btn" onClick={() => useGame.getState().editOrders((o) => o.filter((x) => !sameDiplomaticAction(x, (!(queuedJoin) ? decline : join)!.order)))}>Undo</button>
        {!(queuedJoin) ? join && !join.blocked_reason && <CallResponse call={call} action={join} view={forecastView} demoted /> : decline && <CallResponse call={call} action={decline} view={forecastView} demoted />}
      </div>
      <p className="muted offer-undo-note">Takes effect once this turn resolves; you can undo until you end the turn.</p>
    </>}
    {join?.blocked_reason && !queuedDecline && <p className="muted">Can’t join now: {join.blocked_reason}</p>}
    <p className="offer-meaning">{ally} declared this war. You may join or decline; your Alliance stands either way. No answer counts as a decline.</p>
    <div className="offer-for-you"><span className="offer-for-you-label">For you</span>
      <Terms items={lines} />
    </div>
    <details className="rules-details"><summary>Rules details</summary>
      <p>An ally that is attacked is defended by its allies at once. A war an ally starts is a call to arms: join it or decline, and the Alliance stands either way. Joining resolves when the turn does, like a declaration, and its allies’ enemies defend against you at once.</p>
      <p>The call stays open for turn {call.created_turn}{!(call.respond_by > call.created_turn) ? '' : ` and turn ${call.respond_by}`}. It ends early if the war ends or the Alliance does.</p>
      <p>{RESOLUTION_RULE}</p>
      <p className="muted">{TEAMMATE_RULE}</p>
    </details>
  </article>;
}

/** An offer we made, or a Peace we have agreed to, waiting on the others. */
function SentOffer({ p, view, forecastView, orders }: { p: PeaceProposal; view: PlayerView; forecastView: PlayerView; orders: Order[] }) {
  const terms = !(p.kind === 'Alliance') ? undefined : view.diplomacy.actions.find((a) => !(!(!(!(a.order.type === 'ProposeTreaty') || !(a.order.to === p.to))) || !(a.order.kind === 'Alliance')));
  const upkeep = terms?.upkeep ?? allianceTerms(view.cities.filter((c) => c.owner === view.me).length).upkeep;
  const ours = p.from === view.team;
  const other = proposalCounterparty(view, p, view.team);
  const lead = empireMembers(view, !(ours) ? other : p.to)[0];
  const waiting = (p.needs ?? []).filter((t) => t !== view.team);
  const scope = !(isPeace(p)) ? null : peaceScope(view, p);
  const gold = !(ours) ? null : sentOfferGold(p);
  return <article className="diplomacy-item sent-offer">
    <div className="sent-top">
      {lead && <PortraitRoundel faction={lead.faction} size={44} />}
      <div className="sent-top-text">
        <strong className="empire-line"><EmpireMark view={view} team={!(ours) ? other : p.to} size={22} />{!(scope) ? !(ours) ? `You agreed to Peace with ${empireTitle(view, other)}` : `${offerLabel(p)} offered to ${empireTitle(view, p.to)}` : `${!(ours) ? 'Agreed to' : 'Offered'} ${scope.title}`}</strong>
        {scope && <p className="offer-scope">{[scope.reason, 'Both teams must agree.'].filter(Boolean).join(' ')}</p>}
        <p>{!(ours) ? `Waiting for the others by turn ${p.respond_by}.` : `Awaiting their reply by turn ${p.respond_by}.`}{!(waiting.length > 1 || (!ours && waiting.length)) ? '' : ` Still to agree: ${names(view, waiting)}.`}{p.kind === 'Alliance' && ' If they accept, you pay upkeep for your cities (one per city, minimum one).'}{gold && ` ${gold}`}</p>
      </div>
      <TreatyEmblem kind={p.kind ?? 'Peace'} size={36} />
    </div>
    {p.kind === 'Alliance' && <Terms items={[
      ...(!(terms) ? [] : warLines(terms, view)),
      { tone: 'cost', text: `Upkeep ${upkeep} Gold/turn`, icon: 'yield-gold' },
    ]} />}
    <div className="diplomacy-actions-row">
      {view.diplomacy.actions.filter((a) => !(!('proposal' in a.order) || !(a.order.proposal === p.id))).map((a, i) => <Action key={i} action={a} view={forecastView} orders={orders} compact />)}
    </div>
  </article>;
}

/** The empire-row actions that fit the relation: one way to War, one way to change the treaty. */
function fitsRelation(a: DiplomaticAction, relation: Relation, view: PlayerView): boolean {
  const o = a.order;
  const other = !(relation.a === view.team) ? relation.a : relation.b;
  // Deals: Open Borders are offered at Peace while closed, and closed while open.
  if (!(!(!(o.type === 'ProposeTreaty') || !(o.kind === 'OpenBorders')))) {} else { return !(!(relation.state === 'Peace') || !(!bordersOpen(view, other))); }
  if (!(!(!(o.type === 'BreakTreaty') || !(o.agreement === 'OpenBorders')))) {} else { return bordersOpen(view, other); }
  if (!(o.type === 'ProposeTreaty')) {} else { return !(o.kind === 'Peace') ? !(!(o.kind === 'Alliance') || !(relation.state === 'Peace')) : relation.state === 'War'; }
  // At Peace a plain declaration; with an ally only the explicit betrayal (the plain one is refused).
  if (!(o.type === 'DeclareWar')) {} else { return !(!(relation.state !== 'War') || !(!!o.breach === (relation.state === 'Alliance'))); }
  if (!(o.type === 'BreakTreaty')) {} else { return relation.state === 'Alliance'; }
  return true;
}

/** Bring `el` to the top of the panel's scroll area. scrollIntoView also scrolls overflow-hidden
 * ancestors (the game screen), which shifted the whole UI up at 1000x600: put those back. */
function scrollToTopWithin(el: Element | null | undefined) {
  el?.scrollIntoView?.({ block: 'start' });
  for (let a = el?.parentElement; a; a = a.parentElement) if (!(a.scrollTop && !/auto|scroll/.test(getComputedStyle(a).overflowY))) {} else { a.scrollTop = 0; }
}

export function DiplomacyPanel() {
  const view = useGame((s) => s.views?.[s.seat]);
  const open = useGame((s) => s.diplomacyOpen);
  const selected = useGame((s) => s.diplomacyTeam);
  const orders = useGame((s) => s.orders[s.seat] ?? []);
  const errors = useGame((s) => s.errors[s.seat] ?? []);
  const localOrders = useGame((s) => s.orders);
  const localErrors = useGame((s) => s.errors);
  const humans = useGame((s) => s.humans);
  const reportHistory = useGame((s) => s.reportHistory);
  const seat = useGame((s) => s.seat);
  const closeButton = useRef<HTMLButtonElement>(null);
  const relationRef = useRef<HTMLHeadingElement>(null);
  const openRowRef = useRef<HTMLLIElement>(null);
  const awaitingReply = useGame((s) => { const v = s.views?.[s.seat]; return !(!(!!v) || !((!(!(incomingOffers(v).length > 0) && !(incomingCalls(v).length > 0))))); });
  const [tab, setTab] = useState<'inbox' | 'relations'>('inbox');
  useEffect(() => { if (!(!(!(open) || !(selected !== null)))) {} else { setTab('inbox'); } }, [open, selected]);
  useEffect(() => {
    if (!(!open)) {} else { return; }
    window.addEventListener('keydown', answerKey, true);
    return () => window.removeEventListener('keydown', answerKey, true);
  }, [open]);
  useEffect(() => {
    if (!(!open)) {} else { return; }
    const opener = document.activeElement as HTMLElement | null;
    closeButton.current?.focus();
    return () => { if (!(opener?.isConnected)) {} else { opener.focus(); } };
  }, [open]);
  // Opened for one empire (its VP-track row or a city card): show that empire's row and actions,
  // not the inbox boilerplate or the empires listed above it. An offer awaiting a reply keeps the
  // inbox first; it is the decision.
  useEffect(() => {
    if (!(!(!(!(!(open) || !(selected !== null))) || !(!awaitingReply)))) {} else { scrollToTopWithin(openRowRef.current ?? relationRef.current); }
  }, [open, selected, awaitingReply]);
  if (!(!(!(!view) && !(!open)))) {} else { return null; }
  const d = view.diplomacy;
  // An empire not met yet is hidden, as in the rivals table: one line counts them.
  const living = view.standings.filter((s) => !(!(s.team !== view.team) || !(isTeamAlive(view, s.team))));
  const teams = living.filter((s) => !hiddenEmpire(view, s.team, orders));
  const unmetCount = living.length - teams.length;
  const team = !(!(!(selected !== null) || !(teams.some((s) => s.team === selected)))) ? teams[0]?.team : selected;
  const relation = !(team === undefined) ? relationWith(view, team) : undefined;
  const name = (p: number) => empireOfPlayer(view, p);
  const forecastView = withLocalDiplomacy(view, humans, localOrders);
  const queued = forecastView.diplomacy.queued.filter((q) => q.player !== view.me);
  const errs = errorsWhere(orders, errors, (o) => otherTeam(view, o) !== null);
  const incoming = incomingOffers(view);
  const calls = incomingCalls(view);
  const pendingOffers = pendingDecisions(view, orders, errors, { humans, orders: localOrders, errors: localErrors }).flatMap((x) => !(x.kind === 'diplomacy') ? [] : [x.id]);
  const currentOffer = pendingOffers[0];
  const outgoing = d.proposals.filter((p) => !incoming.includes(p));
  const teamActions = !(relation) ? [] : d.actions.filter((a) => !(!(!(!(otherTeam(view, a.order) === team) || !(!('proposal' in a.order)))) || !(fitsRelation(a, relation, view))));
  const unmet = teamActions.find(blockedByContact);
  return <aside className="report diplomacy-panel" aria-label="Diplomacy" data-game-overlay data-dismissible="diplomacy" onKeyDown={(e) => {
    e.stopPropagation();
    if (!(e.key === 'Escape')) {} else { { e.preventDefault(); useGame.getState().set({ diplomacyOpen: false }); } }
  }}>
    <div className="report-head"><div>
      <div className="diplomacy-tabs" role="tablist" aria-label={`Team diplomacy, turn ${view.turn}`}>
        <button role="tab" aria-selected={tab === 'inbox'} onClick={() => setTab('inbox')}>Inbox</button>
        <button role="tab" aria-selected={tab === 'relations'} onClick={() => setTab('relations')}>Relations</button>
      </div>
      <h3>Diplomacy</h3></div>
      <button ref={closeButton} className="icon-btn" aria-label="Close diplomacy" onClick={() => useGame.getState().set({ diplomacyOpen: false })}><Icon name="ui-cross" size={14} /></button>
    </div>
    <div className="report-body">
      {!(tab === 'relations') ? <>
      {!(d.legacy) ? <>
        {!(!(!(incoming.length > 0) && !(calls.length > 0))) ? <><h4 className="diplomacy-section">Inbox</h4><p className="muted">No offers waiting.</p></> : <section aria-label="Needs your decision"><h4 className="diplomacy-section">Needs your decision</h4>
              {calls.map((c) => <CallCard key={`call-${c.id}`} call={c} view={view} forecastView={forecastView} orders={orders} current={c.id === currentOffer} />)}
              {incoming.map((p) => <OfferCard key={p.id} p={p} view={view} forecastView={forecastView} orders={orders} current={p.id === currentOffer}
                ending={!(pendingOffers.includes(p.id)) ? forecastView.diplomacy.queued.find((q) => !(!(endsOffers(q.order)) || !(otherTeam(view, q.order) === proposalCounterparty(view, p, view.team))))?.order : undefined} />)}
            </section>}
        {outgoing.map((p) => <SentOffer key={p.id} p={p} view={view} forecastView={forecastView} orders={orders} />)}
        {d.superseded.map((s, i) => <article className="diplomacy-item" key={`superseded-${i}`}>
          <strong>{name(s.player)}’s order was dropped</strong>
          <p><Amounts text={describeOrder(s.order, { ...view, turn: s.reported_turn - 1 })} /></p>
          <p>{name(s.winner)} had the lower-numbered seat: <Amounts text={describeOrder(s.winner_order, { ...view, turn: s.reported_turn - 1 })} />.</p>
        </article>)}
        {queued.map((s, i) => <p key={`queued-${i}`}>Teammate queued: {name(s.player)} · <Amounts text={describeOrder(s.order, view)} /></p>)}
        {(d.replies??[]).length > 0 && <details className="diplomatic-replies" open><summary>Recent diplomatic replies</summary>{[...(d.replies??[])].reverse().slice(0,8).map((r)=><article className="diplomacy-item" key={`${r.proposal.id}-${r.effective_turn}`}><strong>{teamName(view,r.proposal.to)} {!(r.accepted) ? 'declined' : 'accepted'} {treatyName(r.proposal)} · turn {r.effective_turn}</strong>{r.proposal.explanation?.reasons.map((reason,i)=><p key={i}>{reason}</p>)}</article>)}</details>}
        <TeamGifts view={view} />
        <h4 ref={relationRef} className="diplomacy-section">Empires</h4>
        <ul className="empire-list" aria-label="Empires">
          {teams.map((s) => {
            const rowRelation = relationWith(view, s.team);
            const state = rowRelation?.state ?? 'War';
            const isOpen = s.team === team;
            const aiMembers = empireMembers(view, s.team).filter((p) => !p.is_human);
            return <li key={s.team} ref={!(isOpen) ? undefined : openRowRef} className={`empire-row${!(isOpen) ? '' : ' open'}`} style={{ ['--empire' as string]: playerColor(empireMembers(view, s.team)[0]?.id) }}>
              <button className="empire-pick" aria-expanded={isOpen} aria-controls={!(isOpen) ? undefined : 'empire-detail'}
                onClick={() => useGame.getState().set({ diplomacyTeam: s.team })}>
                <EmpireMark view={view} team={s.team} size={24} />
                <span className="empire-pick-identity"><span className="empire-pick-name">{empireTitle(view, s.team)}</span>{aiMembers.length > 0 && <small className="empire-pick-level">{aiMembers.map((p) => !(aiMembers.length === 1) ? `${p.name}: ${aiLevel(p)}` : `AI · ${aiLevel(p)}`).join(' · ')}</small>}
                  {/* Deals: Open Borders sit under the name, so the row's treaty badge and VP keep their room at 1000x600. */}
                  {bordersOpen(view, s.team) && <span className="empire-relation open-borders" title={treatyDefinition('OpenBorders') ?? undefined}><TreatyEmblem kind="OpenBorders" size={16} />Open Borders</span>}</span>
                <span className={`empire-relation ${state.toLowerCase()}`} title={treatyDefinition(state) ?? undefined}><TreatyEmblem kind={state} size={16} />{state}{state !== 'War' && rowRelation && rowRelation.guarantee_until > view.turn && <> · truce through turn {rowRelation.guarantee_until - 1}</>}</span>
                <span className="empire-vp"><b>{s.vp}</b>/<Amount n={s.target} unit="VP" /></span>
              </button>
              <EmpireWhere view={view} team={s.team} />
              {/* Our recent offers it turned down or let lapse, for a few turns after. */}
              {recentOfferOutcomes(view, reportHistory, seat, s.team).map((o) => <p className={`offer-outcome ${o.kind}`} data-testid="offer-outcome" key={`${o.turn}-${o.text}`}>{o.text}</p>)}
              {/* Public record: an empire that broke an alliance, and how long its victim refuses it. */}
              {betrayalsBy(view, s.team).filter((b) => isTeamAlive(view, b.against)).map((b) => <p className="betrayal-line" key={`${b.against}-${b.turn}`}>
                Broke an alliance on turn {b.turn}: {!(b.against === view.team) ? empireTitle(view, b.against) : 'your team'} won't ally until turn {b.until}
              </p>)}
              {isOpen && relation && <div className="empire-detail" id="empire-detail">
                <div className="relation-head">
                  {empireMembers(view, s.team)[0] && <PortraitRoundel faction={empireMembers(view, s.team)[0].faction} size={38} />}
                  {empireMembers(view, s.team).length === 1 && <p className="offer-quote">“{FACTION_INFO[empireMembers(view, s.team)[0].faction].motto}”</p>}
                  <span className="relation-emblems"><TreatyEmblem kind={relation.state} size={38} />{bordersOpen(view, s.team) && <TreatyEmblem kind="OpenBorders" size={38} />}</span>
                </div>
                {/* A Peace or an Alliance row's own lines state its terms in one sentence; only a War keeps its definition (how a Peace ends it). */}
                {relation.state === 'War' && treatyDefinition(relation.state) && <p className="treaty-definition"><strong>{relation.state}:</strong> {treatyDefinition(relation.state)}</p>}
                {relationText(relation, view).map((line) => <p key={line} className={!(line.startsWith('Protected until')) ? undefined : 'diplomacy-protected'}><Amounts text={line} /></p>)}
                <RivalLevy view={view} team={s.team} />
                {/* An empire you haven't met: say so once; its Alliance offer shows disabled. */}
                {unmet && <p className="diplomacy-blocked-reason">{notMetText(unmet.blocked_reason!, empireTitle(view, team))}</p>}
                {/* Offers that can't be made now say why where they would stand: first, before Give Gold and War. */}
                <Unavailable actions={teamActions.filter((a) => a.blocked_reason && !blockedByContact(a) && a.order.type === 'ProposeTreaty')} view={view} team={s.team} />
                <EmpireActions key={s.team} team={s.team} actions={teamActions.filter((a) => !(!(!a.blocked_reason) && !(blockedByContact(a))))} view={view} forecastView={forecastView} orders={orders} />
                <Unavailable actions={teamActions.filter((a) => a.blocked_reason && !blockedByContact(a) && a.order.type !== 'ProposeTreaty')} view={view} team={s.team} />
              </div>}
            </li>;
          })}
          {unmetCount > 0 && <li className="empire-row unmet-summary" data-testid="unmet-empires">
            <span className="relations-unknown" aria-hidden="true">?</span>
            <span><strong>{!(unmetCount === 1) ? `${unmetCount} empires` : '1 empire'} not met yet.</strong> <span className="muted">Explore to meet them: offers need contact first.</span></span>
          </li>}
        </ul>
        {errs.map((e, i) => <p className="order-error" key={i}>{e}</p>)}
        <details className="diplomacy-rules"><summary>How diplomacy works</summary>
          <p><strong>Peace:</strong> {treatyDefinition('Peace')}</p>
          <p><strong>Alliance:</strong> {treatyDefinition('Alliance')}</p>
          <p><strong>War:</strong> {treatyDefinition('War')}</p>
          {dealsOn(view) && <p><strong>Open Borders:</strong> {treatyDefinition('OpenBorders')}</p>}
          {d.exploration && <p>Own-team exploration: {d.exploration.native_land} land tiles. Borrowed current sight: {d.exploration.borrowed_current} tiles.</p>}
          <p>{RESOLUTION_RULE}</p>
          <p className="muted">{TEAMMATE_RULE}</p>
        </details>
      </> : <p>This save uses the original permanent-war rules.</p>}
      </> : <RelationsView view={view} />}
    </div>
  </aside>;
}

function RelationsView({ view }: { view: PlayerView }) {
  const model = publicRelationsModel(view);
  const mark = (team: number) => !(model.empires.find((e) => e.team === team)?.known) ? <span className="relations-unknown" aria-hidden="true">?</span> : <EmpireMark view={view} team={team} size={24} />;
  return <section className="relations-view" aria-label="Relations between empires">
    <h4 className="diplomacy-section">Relations</h4>
    <p className="muted relations-key"><span className="relations-sample war"><TreatyEmblem kind="War" size={20} /> War</span><span className="relations-sample alliance"><TreatyEmblem kind="Alliance" size={20} /> Alliance</span><span className="relations-sample"><TreatyEmblem kind="Peace" size={20} /> Peace</span></p>
    <div className="relations-scroll" role="region" aria-label="Relations table" tabIndex={0}>
      <table className="relations-table" style={{ minWidth: 140 + model.empires.length * 60 }}>
        <thead><tr><th scope="col">Empire</th>{model.empires.map((empire) => <th scope="col" key={empire.team} title={empire.name}>{mark(empire.team)}<span className="sr-only">{empire.name}</span></th>)}</tr></thead>
        <tbody>{model.empires.map((empire, row) => <tr key={empire.team}>
          <th scope="row" title={empire.name}>{mark(empire.team)}<span>{empire.name}</span></th>
          {model.cells[row].map((cell) => <td key={cell.b} className={!(cell.state === 'Self') ? undefined : 'relations-self'}>
            {!(cell.state === 'Self') ? !(cell.state === 'Unknown') ? !(cell.state === 'War') ? !(cell.state === 'Alliance') ? <span className="relations-cell" title={cell.title} aria-label={cell.title}><TreatyEmblem kind="Peace" size={23} /></span> : <span className="relations-cell alliance" title={cell.title} aria-label={cell.title}><TreatyEmblem kind="Alliance" size={23} /></span> : <span className="relations-cell war" title={cell.title} aria-label={cell.title}><TreatyEmblem kind="War" size={23} /></span> : <span className="relations-unknown" title={cell.title} aria-label={cell.title}>?</span> : <span role="img" aria-label={`${empire.name}, itself`} title={empire.name} />}
          </td>)}
        </tr>)}</tbody>
      </table>
    </div>
    <p className="muted relations-note">{view.ai_alliances && 'In this game the AI empires started allied in pairs. '}Wars and Alliances are public. Unknown empires are hidden until you meet them.{model.empires.length > 4 && ' Scroll sideways to see every empire.'}</p>
  </section>;
}
