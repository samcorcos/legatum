import { useEffect, useId, useRef, useState, type ReactNode } from 'react';
import { useGame } from '../store';
import { describeOrder, errorsWhere } from '../orders';
import { betrayalsBy, blockedByContact, endsOffers, incomingCalls, incomingOffers, notMetText, offerLabel, otherTeam, proposalCounterparty, queueDiplomacy, relationWith, teamName, withLocalDiplomacy, sameDiplomaticAction } from '../diplomacy';
import type { DiplomaticAction, Order, PeaceProposal, PlayerView, Relation, TeamId, TreatyKind, WarCall } from '../types';
import './DiplomacyPanel.css';
import { CostChip, diplomacySpends } from './CostChip';
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
import { TeamGifts } from './GiveGold';
import { RivalLevy } from './Levy';
import { aiLevel } from '../aiLevel';
import { publicRelationsModel } from '../relations';
import { peaceScope, peaceScopeForAction } from '../peaceScope';

const treatyName = (p: Pick<PeaceProposal, 'kind'>) => p.kind ?? 'Peace';

function actionLabel(action: DiplomaticAction, view: PlayerView): string {
  const o = action.order;
  const peace = peaceScopeForAction(view, action);
  return o.type === 'RespondToProposal' ? (o.accept ? acceptLabel(view.diplomacy.proposals.find((p) => p.id === o.proposal), view) : declineLabel(view.diplomacy.proposals.find((p) => p.id === o.proposal), view))
    : o.type === 'DeclareWar' ? (relationWith(view, o.against)?.state === 'Alliance' ? 'Declare War on your ally' : 'Declare War')
    : o.type === 'ProposeTreaty' ? (peace ? `Offer ${peace.subject}` : `Offer ${o.kind}`)
    : o.type === 'BreakTreaty' ? 'Leave the Alliance' : 'Withdraw offer';
}

/** What accepting says. */
function acceptLabel(p: PeaceProposal | undefined, view: PlayerView): string {
  return p?.kind !== 'Alliance' && p ? `Accept ${peaceScope(view, p).subject}` : p ? `Accept ${treatyName(p)}` : 'Accept Peace';
}
function declineLabel(p: PeaceProposal | undefined, view: PlayerView): string {
  return p?.kind !== 'Alliance' && p ? `Decline ${peaceScope(view, p).subject}` : 'Decline';
}

type ForYou = { tone: 'cost' | 'gain' | 'note'; text: string; icon?: string };

/**
 * A term of a treaty as a medallion row (the faction sheet's language): a red ring and a falling
 * arrow for what it costs you (Gold in its gold ink), a green ring and a rising arrow for what you
 * gain, and a plain muted line for a note. The words stay the same, and the screen-reader prefix.
 */
function TermLine({ tone, icon, children }: ForYou & { children: ReactNode }) {
  if (tone === 'note') return <li className="for-you-line term note"><span className="for-you-mark" aria-hidden="true">·</span><span>{children}</span></li>;
  const glyph = icon ?? (tone === 'cost' ? 'ui-alert' : 'ui-check');
  return <li className={`for-you-line term ${tone}`}>
    <span className="term-md" aria-hidden="true"><RingMedallion icon={glyph} tone={tone === 'cost' ? 'bad' : 'good'} size={26} iconColor={glyph === 'yield-gold' ? YIELD_COLORS.gold : undefined} /></span>
    <span className="sr-only">{tone === 'cost' ? 'Cost: ' : 'Gain: '}</span>
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
  const them = team === null ? 'They' : empireTitle(view, team);
  const { protectedTurns, betrayalTurns } = allianceTerms(view.cities.filter((c) => c.owner === view.me).length);
  const lines: ForYou[] = [];
  if (action.cost > 0) lines.push({ tone: 'cost', text: `Costs ${action.cost} Gold: inside the ${protectedTurns}-turn protected window`, icon: 'yield-gold' });
  if (action.betrayal) lines.push({ tone: 'cost', text: `Betrayal: ${them} won't ally with you for ${betrayalTurns} turns`, icon: 'ui-alert' });
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
  const target = o.type === 'DeclareWar' ? o.against : null;
  const lines: ForYou[] = [];
  const you = wars.filter((w) => w.team === view.team).map((w) => w.enemy);
  const againstYou = wars.filter((w) => w.enemy === view.team).map((w) => w.team);
  const helpers = wars.filter((w) => w.team !== view.team && w.enemy !== view.team);
  if (you.length && withJoin) lines.push({ tone: 'cost', icon: 'unit-spearman', text: `You would join the war against ${names(view, you)}, which attacked your new ally` });
  if (againstYou.length) lines.push({ tone: 'cost', icon: 'unit-spearman', text: o.type === 'DeclareWar'
    ? `${names(view, againstYou)}, ${againstYou.length > 1 ? 'allies' : 'an ally'} of ${target === null ? 'them' : empireTitle(view, target)}, would defend them and be at War with you at once`
    : `${names(view, againstYou)}, ${againstYou.length > 1 ? 'allies' : 'an ally'} of ${target === null ? 'them' : empireTitle(view, target)}, would also be at War with you at once` });
  if (helpers.length) lines.push({ tone: 'gain', icon: 'unit-spearman', text: `${names(view, helpers.map((w) => w.team))} would join your war against ${names(view, helpers.map((w) => w.enemy))}` });
  const askedMe = calls.filter((c) => c.team === view.team), asked = calls.filter((c) => c.team !== view.team);
  if (askedMe.length) lines.push({ tone: 'note', text: `You would be asked to join the war on ${names(view, askedMe.map((c) => c.enemy))}. You may decline` });
  if (asked.length) lines.push({ tone: 'note', text: o.type === 'DeclareWar'
    ? `${names(view, asked.map((c) => c.team))}, your ${asked.length > 1 ? 'allies' : 'ally'}, would be asked to join this war. Each may decline`
    : `${names(view, asked.map((c) => c.team))} would be asked to join your war on ${names(view, asked.map((c) => c.enemy))}. They may decline` });
  return lines;
}

/** An Alliance offer's lead line, right under its button: the war it brings you into ("Joins you to the war against Sparta"). */
function joinWarLine(action: DiplomaticAction, view: PlayerView): string | null {
  const you = (action.new_wars ?? []).filter((w) => w.team === view.team).map((w) => w.enemy);
  return you.length ? `Joins you to the war against ${names(view, you)}` : null;
}

/** Name the single pairwise war this Peace ends and any wars that continue. */
function agreeLines(action: DiplomaticAction, view: PlayerView): ForYou[] {
  const o = action.order;
  // Only a Peace ends wars: an Alliance or any other offer never prints the Peace rule.
  const kind = o.type === 'ProposeTreaty' ? o.kind
    : o.type === 'RespondToProposal' ? (view.diplomacy.proposals.find((p) => p.id === o.proposal)?.kind ?? 'Peace') : null;
  if (kind !== 'Peace') return [];
  const p = o.type === 'RespondToProposal' ? view.diplomacy.proposals.find((offer) => offer.id === o.proposal) : undefined;
  const scope = p ? peaceScope(view, p) : peaceScopeForAction(view, action);
  if (!scope) return [];
  return [{ tone: 'note', text: [scope.title + '.', scope.reason, 'Both teams must agree.'].filter(Boolean).join(' ') }];
}

/** The plain consequences of an empire-row action, shown beside its button (never collapsed). */
function actionLines(action: DiplomaticAction, view: PlayerView): ForYou[] {
  const o = action.order;
  switch (o.type) {
    case 'ProposeTreaty':
      return o.kind === 'Alliance' ? [...warLines(action, view, false), ...allianceCosts(action, view)]
        : [...agreeLines(action, view), { tone: 'note', text: `If accepted, Peace starts the turn after, guaranteed for ${TREATY_TERMS.guaranteeTurns} turns. Borders close.` }];
    case 'BreakTreaty':
      return [...betrayalLines(action, view),
        ...(!action.betrayal && action.cost === 0 ? [{ tone: 'note' as const, text: `Free: the ${allianceTerms(view.cities.filter((c) => c.owner === view.me).length).protectedTurns}-turn protected window has passed` }] : []),
        { tone: 'note', text: `The Alliance ends turn ${action.effective_turn}: no more shared sight or open borders` }];
    case 'DeclareWar':
      return action.betrayal
        ? [...betrayalLines(action, view), ...warLines(action, view), { tone: 'note', text: `War begins turn ${action.effective_turn}; the Alliance ends` }]
        : [...warLines(action, view), { tone: 'note', text: action.effective_turn === view.turn ? 'War starts now: attacks you order this turn resolve this turn' : `War begins turn ${action.effective_turn}` }];
    default:
      return [];
  }
}

/** A diplomatic order's toggle. `compact` is the bare button, for decision cards that explain it themselves. */
function Action({ action, view, orders, compact = false }: { action: DiplomaticAction; view: PlayerView; orders: Order[]; compact?: boolean }) {
  const queued = orders.some((o) => sameDiplomaticAction(o, action.order));
  const label = actionLabel(action, view);
  const chipId = useId();
  // Alliance costs are itemised beneath the button, once.
  const allianceOffer = action.order.type === 'ProposeTreaty' && action.order.kind === 'Alliance';
  const spends = queued || action.blocked_reason || allianceOffer ? [] : diplomacySpends(action, view);
  const text = queued ? `Queued: ${label} · undo` : label;
  const button = <button className={`btn${spends.length ? ' spends' : ''}`}
    aria-label={text} aria-describedby={spends.length ? chipId : undefined} aria-pressed={queued} disabled={!!action.blocked_reason || view.finished}
    onClick={() => useGame.getState().editOrders((orders) => queued
      ? orders.filter((o) => !sameDiplomaticAction(o, action.order))
      : queueDiplomacy(orders, action.order, view))}>
    <span>{text}</span>
    <CostChip id={chipId} spends={spends} when={action.order.type === 'ProposeTreaty' ? 'if accepted' : undefined} />
  </button>;
  if (compact) return button;
  const lines = action.blocked_reason ? [] : actionLines(action, view);
  const join = !action.blocked_reason && action.order.type === 'ProposeTreaty' && action.order.kind === 'Alliance' ? joinWarLine(action, view) : null;
  const definition = action.order.type === 'ProposeTreaty' ? treatyDefinition(action.order.kind) : null;
  return <div className="diplomacy-action">
    {definition && <p className="treaty-definition"><strong>{action.order.type === 'ProposeTreaty' && action.order.kind}:</strong> {definition}</p>}
    {button}
    {join && <p className="diplomacy-join-war" data-testid="join-war"><Icon name="unit-spearman" size={13} className="ui-mark" /> {join}</p>}
    {/* A blocked action's reason is stated once for its empire row, not under each button. */}
    {!action.blocked_reason && <span>{spends.length || (allianceOffer && (action.cost > 0 || (action.upkeep ?? 0) > 0)) ? '' : 'Free · '}{action.effective_turn === view.turn ? 'Takes effect this turn'
      : `${action.order.type === 'ProposeTreaty' ? 'Earliest activation' : 'Takes effect'} turn ${action.effective_turn}`}</span>}
    {lines.length > 0 && <Terms items={lines} className="diplomacy-action-lines" />}
  </div>;
}

/** Unavailable actions keep their reason beside the control. */
function Unavailable({ actions, view }: { actions: DiplomaticAction[]; view: PlayerView }) {
  const war = actions.find((a) => a.order.type === 'DeclareWar' && a.blocked_reason?.includes('guaranteed through turn'));
  const byReason = new Map<string, string[]>();
  for (const a of actions.filter((a) => !war || a.order.type !== 'DeclareWar')) byReason.set(a.blocked_reason!, [...(byReason.get(a.blocked_reason!) ?? []), actionLabel(a, view)]);
  return <>
    {war && <div className="diplomacy-action diplomacy-unavailable"><button className="btn" disabled aria-label={actionLabel(war, view)}>{actionLabel(war, view)}</button><p className="muted"><Amounts text={war.blocked_reason!} /></p></div>}
    {[...byReason].map(([reason, labels]) => <p className="muted diplomacy-unavailable" key={reason}>Not now: {labels.join(', ')}. <Amounts text={reason} /></p>)}
  </>;
}

/** Peace closes borders, except where a Yam Relay Station's holder opens its land to every empire. */
export function peaceBorders(relation: Relation, view: PlayerView): string {
  const other = relation.a === view.team ? relation.b : relation.a;
  const holds = (team: number) => (view.courier_halls ?? []).some((h) => view.players.find((p) => p.id === h.holder)?.team === team);
  const theirs = holds(other), ours = holds(view.team);
  return theirs && ours ? 'borders open both ways (Yam Relay Stations)'
    : theirs ? 'their land is open to you (their Yam Relay Station), yours stays closed'
      : ours ? 'your land is open to them (your Yam Relay Station), theirs stays closed' : 'closed borders';
}

/** Where you stand with an empire, in one or two plain sentences. */
function relationText(relation: Relation, view: PlayerView): string[] {
  if (relation.state === 'War') {
    const other = relation.a === view.team ? relation.b : relation.a;
    return [`War since turn ${relation.effective_turn}. Combat, captures and entry into this team’s territory are allowed.`,
      ...(relation.declarers.includes(other) && !relation.declarers.includes(view.team) ? [`${empireTitle(view, other)} started it.`] : [])];
  }
  if (relation.state === 'Peace') {
    return [`No combat or captures; ${peaceBorders(relation, view)}. ${relation.guarantee_until > view.turn
      ? `Peace guaranteed through turn ${relation.guarantee_until - 1}. Earliest War: turn ${relation.guarantee_until}.`
      : 'Declaring War starts it at once.'}`];
  }
  const { breachCost } = allianceTerms(view.cities.filter((c) => c.owner === view.me).length);
  const s = relation.security;
  const lines = [`Shared sight and open borders; no combat. Allies defend each other at once, and each chooses whether to join the other’s wars. Each of you pays 1 Gold a turn for each of your own cities.`];
  if (s) {
    // Leaving costs Gold while the exit (next turn) would land before protected_until.
    lines.push(view.turn + 1 < s.protected_until
      ? `Protected until turn ${s.protected_until}: leaving before then costs ${breachCost} Gold and is a betrayal. War on an ally is always a betrayal.`
      : `Signed turn ${s.started_turn}; the protected window has passed, so leaving is free. War on an ally is still a betrayal.`);
  }
  if (relation.guarantee_until > view.turn) {
    lines.push(`Peace truce with ${empireTitle(view, relation.a === view.team ? relation.b : relation.a)} remains guaranteed through turn ${relation.guarantee_until - 1}, even if this Alliance ends.`);
  }
  return lines;
}

const offerNoun = (kind: TreatyKind) => kind === 'Peace' ? 'Peace' : 'an Alliance';

/** The plain-language meaning under an offer's headline. */
function offerMeaning(p: PeaceProposal): ReactNode {
  return p.kind === 'Alliance' ? ALLIANCE_SHARES : treatyDefinition(p.kind ?? 'Peace')!;
}

/**
 * The concrete consequences for the viewer, costs first (red −: the Gold, the promise not to
 * attack, the protected window), then gains (green +).
 */
function offerForecast(p: PeaceProposal, view: PlayerView, accept: DiplomaticAction | undefined): ForYou[] {
  if (p.kind === 'Alliance') {
    const { breachCost, protectedTurns } = allianceTerms(view.cities.filter((c) => c.owner === view.me).length);
    const costs = allianceCosts(accept, view);
    return [
      // The wars it brings you into come first, then the upkeep and benefits.
      ...(accept ? warLines(accept, view) : []),
      ...costs.filter((l) => l.tone === 'gain'),
      ...costs.filter((l) => l.tone === 'cost'),
      { tone: 'cost', text: `Leaving within ${protectedTurns} turns costs ${breachCost} Gold and is a betrayal`, icon: 'yield-gold' },
      ...costs.filter((l) => l.tone === 'note'),
      { tone: 'gain', text: 'You see everything they see, and your units may enter their land', icon: 'effect-insight' },
      { tone: 'gain', text: 'Neither side can attack the other', icon: 'fortify' },
    ];
  }
  const lines: ForYou[] = [];
  const other = proposalCounterparty(view, p, view.team);
  if (relationWith(view, other)?.state !== 'Alliance') {
    const near = unitsNearOurs(view, other);
    if (near.length) lines.push({ tone: 'cost', text: `You can’t attack their ${near[0].unit} ${near[0].tiles <= 1 ? 'next to' : `${near[0].tiles} tiles from`} ${near[0].near}${near.length > 1 ? ` (or their ${near.length - 1} other unit${near.length > 2 ? 's' : ''} nearby)` : ''}`, icon: 'unit-spearman' });
  }
  lines.push({ tone: 'note', text: 'Costs no Gold' });
  lines.push({ tone: 'gain', text: `Peace guaranteed through turn ${view.turn + TREATY_TERMS.guaranteeTurns}; after that either side may declare war`, icon: 'fortify' });
  return lines;
}

/** The offer's headline names its proposer; the scope line gives the pairwise effect. */
function offerHeadline(p: PeaceProposal, view: PlayerView): string {
  const kind = p.kind ?? 'Peace';
  return `${empireTitle(view, p.from)} offers ${offerNoun(kind)}`;
}

const TEAMMATE_RULE = "Teammates share diplomacy. If teammates give conflicting orders, the lower-numbered seat's order is used and the other is dropped.";
const RESOLUTION_RULE = 'Decisions take effect after this turn’s resolution. Queued orders remain pending until the turn report confirms them.';

/**
 * G17: answering completes the decision. Queue the reply, confirm it over the map with Undo, then
 * go to the next offer still awaiting a reply, or close the inbox so the map shows.
 */
function respondToOffer(p: PeaceProposal, action: DiplomaticAction, forecastView: PlayerView): void {
  const accepting = action.order.type === 'RespondToProposal' && action.order.accept;
  const what = offerLabel(p);
  const other = proposalCounterparty(forecastView, p, forecastView.team);
  const scope = p.kind !== 'Alliance' ? peaceScope(forecastView, p) : null;
  const done = answerDecision(scope ? [accepting ? `Accepted ${scope.subject}; when both teams agree, it ${scope.ending}.` : `Declined ${scope.subject}; your wars continue.`, scope.reason].filter(Boolean).join(' ') : `${what} from ${empireTitle(forecastView, other)} ${accepting ? 'accepted' : 'declined'}`,
    (orders) => queueDiplomacy(orders, action.order, forecastView), other);
  if (done) goToNextAnswer();
}

/** After an answer: the next decision awaiting a reply, or close the inbox so the map shows. */
function goToNextAnswer() {
  const s = useGame.getState();
  const view = s.views?.[s.seat];
  const next = view && pendingDecisions(view, s.orders[s.seat], s.errors[s.seat], s).find((d) => d.kind === 'diplomacy');
  if (!next || next.kind !== 'diplomacy') { s.set({ diplomacyOpen: false }); return; }
  s.set({ diplomacyTeam: next.team });
  requestAnimationFrame(() => scrollToTopWithin(document.getElementById(`offer-${next.id}`)?.closest('article')));
}

/** Answer a call to arms: Join or Decline, confirmed over the map with Undo like an offer. */
function respondToCall(call: WarCall, action: DiplomaticAction, forecastView: PlayerView): void {
  const joining = action.order.type === 'RespondToProposal' && action.order.accept;
  const ally = empireTitle(forecastView, call.ally), target = empireTitle(forecastView, call.target);
  const done = answerDecision(joining ? `Joining ${ally}’s war on ${target}` : `Declined ${ally}’s call to arms against ${target}`,
    (orders) => queueDiplomacy(orders, action.order, forecastView), call.ally);
  if (done) goToNextAnswer();
}

/** Join or Decline on a call card; `demoted` is the unchosen choice on an answered card. */
function CallResponse({ call, action, view, demoted = false, hotkey }: { call: WarCall; action: DiplomaticAction; view: PlayerView; demoted?: boolean; hotkey?: string }) {
  const joining = action.order.type === 'RespondToProposal' && action.order.accept;
  const touch = useCoarsePointer();
  const label = `${joining ? `Join the war on ${empireTitle(view, call.target)}` : 'Decline'}${demoted ? ' instead' : ''}`;
  return <button className={`btn${joining && !demoted ? ' primary' : ''}${demoted ? ' quiet' : ''}`}
    aria-label={label} disabled={!!action.blocked_reason || view.finished} onClick={() => respondToCall(call, action, view)}>
    <span>{label}</span>
    {hotkey && !touch && <Kbd>{hotkey}</Kbd>}
  </button>;
}

/** Accept or Decline on an offer card; `demoted` is the unchosen choice on an answered card. */
function OfferResponse({ p, action, view, demoted = false, hotkey }: { p: PeaceProposal; action: DiplomaticAction; view: PlayerView; demoted?: boolean; hotkey?: string }) {
  const accepting = action.order.type === 'RespondToProposal' && action.order.accept;
  const chipId = useId();
  const touch = useCoarsePointer();
  const spends = action.blocked_reason || p.kind === 'Alliance' ? [] : diplomacySpends(action, view);
  const fullLabel = `${actionLabel(action, view)}${demoted ? ' instead' : ''}`;
  const label = p.kind !== 'Alliance' ? `${accepting ? 'Accept Peace' : 'Decline'}${demoted ? ' instead' : ''}` : fullLabel;
  return <button className={`btn${accepting && !demoted ? ' primary' : ''}${demoted ? ' quiet' : ''}${spends.length ? ' spends' : ''}`}
    aria-label={fullLabel} aria-describedby={spends.length ? chipId : undefined} disabled={!!action.blocked_reason || view.finished}
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
  if (e.defaultPrevented || e.repeat || e.isComposing || e.metaKey || e.ctrlKey || e.altKey || e.shiftKey) return;
  const key = e.key.toLowerCase();
  if (key !== 'a' && key !== 'd') return;
  if ((e.target as HTMLElement | null)?.closest?.('input, textarea, select, [contenteditable=""], [contenteditable="true"]')) return;
  if (topDismissible(document) !== 'diplomacy') return;
  const s = useGame.getState();
  const view = s.views?.[s.seat];
  if (!view || view.finished) return;
  const next = pendingDecisions(view, s.orders[s.seat], s.errors[s.seat], s).find((d) => d.kind === 'diplomacy');
  const answer = (id: number) => view.diplomacy.actions.find((a) => a.order.type === 'RespondToProposal' && a.order.proposal === id && a.order.accept === (key === 'a'));
  const call = next?.kind === 'diplomacy' ? incomingCalls(view).find((x) => x.id === next.id) : undefined;
  if (call) {
    const action = answer(call.id);
    if (!action || action.blocked_reason) return;
    e.preventDefault();
    e.stopPropagation();
    respondToCall(call, action, withLocalDiplomacy(view, s.humans, s.orders));
    return;
  }
  const p = next?.kind === 'diplomacy' ? view.diplomacy.proposals.find((x) => x.id === next.id) : undefined;
  const action = p && answer(p.id);
  if (!p || !action || action.blocked_reason) return;
  e.preventDefault();
  e.stopPropagation();
  respondToOffer(p, action, withLocalDiplomacy(view, s.humans, s.orders));
}

/** Why an offer needs no reply: the queued order toward its sender that ends it. */
function endingText(o: Order, empire: string): string {
  return o.type === 'DeclareWar' ? `Ended by your War on ${empire}` : `Ended by leaving the Alliance with ${empire}`;
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
  const responses = d.actions.filter((a) => a.order.type === 'RespondToProposal' && a.order.proposal === p.id);
  const accept = responses.find((a) => a.order.type === 'RespondToProposal' && a.order.accept);
  const decline = responses.find((a) => a.order.type === 'RespondToProposal' && !a.order.accept);
  const kind = p.kind ?? 'Peace';
  const scope = kind === 'Peace' ? peaceScope(view, p) : null;
  const queuedAccept = !!accept && orders.some((o) => sameDiplomaticAction(o, accept.order));
  const queuedDecline = !!decline && orders.some((o) => sameDiplomaticAction(o, decline.order));
  // A disabled Accept shows one plain reason ("Can't accept now"), never a warning about accepting.
  const blocked = !!accept?.blocked_reason;
  const members = empireMembers(view, p.from);
  const leader = members[0];
  const color = playerInk(leader?.id);
  const terms = allianceTerms(view.cities.filter((c) => c.owner === view.me).length);
  const start = accept?.effective_turn ?? view.turn + 1;
  const firstTerms = kind === 'Alliance' ? [...(accept ? warLines(accept, view) : []), ...allianceCosts(accept, view).filter((line) => line.tone !== 'note')] : offerForecast(p, view, accept);
  const moreTerms = kind === 'Alliance' ? offerForecast(p, view, accept).filter((line) => line.tone === 'gain' && !firstTerms.some((first) => first.text === line.text)) : [];
  const benefits = moreTerms.filter((line) => line.tone === 'gain');
  const termBlock = <div className="offer-for-you"><span className="offer-for-you-label">For you</span><Terms items={firstTerms} /></div>;
  return <article className="diplomacy-item offer-card" style={{ ['--empire' as string]: color }} aria-labelledby={`offer-${p.id}`}>
    <div className="offer-top">
      {leader
        ? <PortraitFrame faction={leader.faction} crest={false} className="offer-portrait">
            <span className="offer-emblem"><TreatyEmblem kind={kind} size={36} /></span>
          </PortraitFrame>
        : <TreatyEmblem kind={kind} size={44} />}
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
    {ending && !queuedAccept && !queuedDecline ? <>
      <div className="offer-answered">
        <span className="offer-status offer-answer" role="status"><Icon name="ui-check" size={12} className="ui-mark" /> {endingText(ending, empireTitle(view, proposalCounterparty(view, p, view.team)))}</span>
      </div>
      <p className="muted offer-undo-note">Nothing to answer: the offer ends when this turn resolves. To answer it instead, undo that order first.</p>
    </> : queuedAccept || queuedDecline ? <>
      <div className="offer-answered">
        <span className="offer-status offer-answer" role="status"><Icon name="ui-check" size={12} className="ui-mark" /> {scope ? queuedAccept ? `Accepted ${scope.subject}; when both teams agree, it ${scope.ending}` : `Declined ${scope.subject}; your wars continue` : queuedAccept ? `Accepted: ${kind} starts turn ${accept!.effective_turn}` : 'Declined'}</span>
        <button className="btn" onClick={() => useGame.getState().editOrders((o) => o.filter((x) => !sameDiplomaticAction(x, (queuedAccept ? accept : decline)!.order)))}>Undo</button>
        {queuedAccept ? decline && <OfferResponse p={p} action={decline} view={forecastView} demoted />
          : accept && !blocked && <OfferResponse p={p} action={accept} view={forecastView} demoted />}
      </div>
      <p className="muted offer-undo-note">Takes effect once this turn resolves; you can undo until you end the turn.</p>
    </> : <div className="offer-actions">
      {accept && <OfferResponse p={p} action={accept} view={forecastView} hotkey={current && !accept.blocked_reason ? 'A' : undefined} />}
      {decline && <OfferResponse p={p} action={decline} view={forecastView} hotkey={current ? 'D' : undefined} />}
      <span className="offer-deadline">Respond by turn {p.respond_by}</span>
    </div>}
    {scope && accept && !ending && !queuedDecline && <p className="offer-agree" data-testid="offer-agree">Both teams must agree.</p>}
    {accept?.blocked_reason && !queuedDecline && !ending && <p className="muted">Can’t accept now: {accept.blocked_reason}</p>}
    {kind !== 'Alliance' && termBlock}
    {benefits.length > 0 && <div className="offer-for-you offer-benefits"><Terms items={benefits} /></div>}
    <details className="offer-explanation"><summary>How this offer works</summary>
      <p className="offer-meaning">{offerMeaning(p)}</p>
    </details>
    {leader && members.length === 1 && <p className="offer-quote">“{FACTION_INFO[leader.faction].motto}”</p>}
    <details className="rules-details"><summary>Rules details</summary>
      {kind === 'Alliance'
        ? <>
          <p>The Alliance starts in turn {start}. Each empire pays for its own cities. Declining is free.</p>
          <p>Leaving within {terms.protectedTurns} turns of signing costs <Amount n={terms.breachCost} unit="Gold" /> and is a betrayal; so is declaring war on an ally at any time. A betrayed team won’t ally with its betrayer for {terms.betrayalTurns} turns.</p>
        </>
        : <>
          <p>Respond by turn {p.respond_by}. Acceptance now starts Peace in turn {start}, guaranteed through turn {view.turn + TREATY_TERMS.guaranteeTurns}. This turn’s attacks still resolve. Peace costs no Gold.</p>
          <p>Borders close. Units left inside may stay, but cannot attack, heal or fortify there; every move must approach an exit.</p>
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
  const responses = view.diplomacy.actions.filter((a) => a.order.type === 'RespondToProposal' && a.order.proposal === call.id);
  const join = responses.find((a) => a.order.type === 'RespondToProposal' && a.order.accept);
  const decline = responses.find((a) => a.order.type === 'RespondToProposal' && !a.order.accept);
  const queuedJoin = !!join && orders.some((o) => sameDiplomaticAction(o, join.order));
  const queuedDecline = !!decline && orders.some((o) => sameDiplomaticAction(o, decline.order));
  const members = empireMembers(view, call.ally);
  const leader = members[0];
  const ally = empireTitle(view, call.ally), target = empireTitle(view, call.target);
  const defenders = (join?.new_wars ?? []).map((w) => w.team);
  const lines: ForYou[] = [
    { tone: 'cost', icon: 'unit-spearman', text: `Joining puts you at War with ${target} from turn ${join?.effective_turn ?? view.turn + 1}` },
    ...(defenders.length ? [{ tone: 'cost' as const, icon: 'unit-spearman', text: `${names(view, defenders)}, ${defenders.length > 1 ? 'allies' : 'an ally'} of ${target}, would be at War with you too` }] : []),
    { tone: 'gain', icon: 'ui-check', text: `Declining keeps your Alliance with ${ally} and your Peace with ${target}` },
    { tone: 'note', text: 'Declining costs nothing.' },
  ];
  return <article className="diplomacy-item offer-card call-card" style={{ ['--empire' as string]: playerInk(leader?.id) }} aria-labelledby={`offer-${call.id}`}>
    <div className="offer-top">
      {leader ? <PortraitFrame faction={leader.faction} crest={false} className="offer-portrait"><span className="offer-emblem"><Icon name="unit-spearman" size={30} /></span></PortraitFrame>
        : <Icon name="unit-spearman" size={44} />}
      <div className="offer-top-text">
        <h4 className="offer-headline" id={`offer-${call.id}`}>
          <EmpireMark view={view} team={call.ally} size={26} />
          <span>Your ally {ally} calls you to war with {target}.</span>
        </h4>
        <EmpireWhere view={view} team={call.ally} />
      </div>
    </div>
    {queuedJoin || queuedDecline ? <>
      <div className="offer-answered">
        <span className="offer-status offer-answer" role="status"><Icon name="ui-check" size={12} className="ui-mark" /> {queuedJoin ? `Joining the war on ${target} from turn ${join!.effective_turn}` : 'Declined: the Alliance stands'}</span>
        <button className="btn" onClick={() => useGame.getState().editOrders((o) => o.filter((x) => !sameDiplomaticAction(x, (queuedJoin ? join : decline)!.order)))}>Undo</button>
        {queuedJoin ? decline && <CallResponse call={call} action={decline} view={forecastView} demoted />
          : join && !join.blocked_reason && <CallResponse call={call} action={join} view={forecastView} demoted />}
      </div>
      <p className="muted offer-undo-note">Takes effect once this turn resolves; you can undo until you end the turn.</p>
    </> : <div className="offer-actions">
      {join && <CallResponse call={call} action={join} view={forecastView} hotkey={current && !join.blocked_reason ? 'A' : undefined} />}
      {decline && <CallResponse call={call} action={decline} view={forecastView} hotkey={current ? 'D' : undefined} />}
      <span className="offer-deadline">Answer by turn {call.respond_by}</span>
    </div>}
    {join?.blocked_reason && !queuedDecline && <p className="muted">Can’t join now: {join.blocked_reason}</p>}
    <p className="offer-meaning">{ally} declared this war. You may join or decline; your Alliance stands either way. No answer counts as a decline.</p>
    <div className="offer-for-you"><span className="offer-for-you-label">For you</span>
      <Terms items={lines} />
    </div>
    <details className="rules-details"><summary>Rules details</summary>
      <p>An ally that is attacked is defended by its allies at once. A war an ally starts is a call to arms: join it or decline, and the Alliance stands either way. Joining resolves when the turn does, like a declaration, and its allies’ enemies defend against you at once.</p>
      <p>The call stays open for turn {call.created_turn}{call.respond_by > call.created_turn ? ` and turn ${call.respond_by}` : ''}. It ends early if the war ends or the Alliance does.</p>
      <p>{RESOLUTION_RULE}</p>
      <p className="muted">{TEAMMATE_RULE}</p>
    </details>
  </article>;
}

/** An offer we made, or a Peace we have agreed to, waiting on the others. */
function SentOffer({ p, view, forecastView, orders }: { p: PeaceProposal; view: PlayerView; forecastView: PlayerView; orders: Order[] }) {
  const terms = p.kind === 'Alliance' ? view.diplomacy.actions.find((a) => a.order.type === 'ProposeTreaty' && a.order.to === p.to && a.order.kind === 'Alliance') : undefined;
  const upkeep = terms?.upkeep ?? allianceTerms(view.cities.filter((c) => c.owner === view.me).length).upkeep;
  const ours = p.from === view.team;
  const other = proposalCounterparty(view, p, view.team);
  const lead = empireMembers(view, ours ? p.to : other)[0];
  const waiting = (p.needs ?? []).filter((t) => t !== view.team);
  const scope = p.kind !== 'Alliance' ? peaceScope(view, p) : null;
  return <article className="diplomacy-item sent-offer">
    <div className="sent-top">
      {lead && <PortraitRoundel faction={lead.faction} size={44} />}
      <div className="sent-top-text">
        <strong className="empire-line"><EmpireMark view={view} team={ours ? p.to : other} size={22} />{scope ? `${ours ? 'Offered' : 'Agreed to'} ${scope.title}` : ours ? `${offerLabel(p)} offered to ${empireTitle(view, p.to)}` : `You agreed to Peace with ${empireTitle(view, other)}`}</strong>
        {scope && <p className="offer-scope">{[scope.reason, 'Both teams must agree.'].filter(Boolean).join(' ')}</p>}
        <p>{ours ? `Awaiting their reply by turn ${p.respond_by}.` : `Waiting for the others by turn ${p.respond_by}.`}{waiting.length > 1 || (!ours && waiting.length) ? ` Still to agree: ${names(view, waiting)}.` : ''}{p.kind === 'Alliance' && ' If they accept, you pay upkeep for your cities (one per city, minimum one).'}</p>
      </div>
      <TreatyEmblem kind={p.kind === 'Alliance' ? 'Alliance' : 'Peace'} size={36} />
    </div>
    {p.kind === 'Alliance' && <Terms items={[
      ...(terms ? warLines(terms, view) : []),
      { tone: 'cost', text: `Upkeep ${upkeep} Gold/turn`, icon: 'yield-gold' },
    ]} />}
    <div className="diplomacy-actions-row">
      {view.diplomacy.actions.filter((a) => 'proposal' in a.order && a.order.proposal === p.id).map((a, i) => <Action key={i} action={a} view={forecastView} orders={orders} compact />)}
    </div>
  </article>;
}

/** The empire-row actions that fit the relation: one way to War, one way to change the treaty. */
function fitsRelation(a: DiplomaticAction, relation: Relation): boolean {
  const o = a.order;
  if (o.type === 'ProposeTreaty') return o.kind === 'Peace' ? relation.state === 'War' : o.kind === 'Alliance' && relation.state === 'Peace';
  // At Peace a plain declaration; with an ally only the explicit betrayal (the plain one is refused).
  if (o.type === 'DeclareWar') return relation.state !== 'War' && !!o.breach === (relation.state === 'Alliance');
  if (o.type === 'BreakTreaty') return relation.state === 'Alliance';
  return true;
}

/** Bring `el` to the top of the panel's scroll area. scrollIntoView also scrolls overflow-hidden
 * ancestors (the game screen), which shifted the whole UI up at 1000x600: put those back. */
function scrollToTopWithin(el: Element | null | undefined) {
  el?.scrollIntoView?.({ block: 'start' });
  for (let a = el?.parentElement; a; a = a.parentElement) if (a.scrollTop && !/auto|scroll/.test(getComputedStyle(a).overflowY)) a.scrollTop = 0;
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
  const closeButton = useRef<HTMLButtonElement>(null);
  const relationRef = useRef<HTMLHeadingElement>(null);
  const openRowRef = useRef<HTMLLIElement>(null);
  const awaitingReply = useGame((s) => { const v = s.views?.[s.seat]; return !!v && (incomingOffers(v).length > 0 || incomingCalls(v).length > 0); });
  const [tab, setTab] = useState<'inbox' | 'relations'>('inbox');
  useEffect(() => { if (open && selected !== null) setTab('inbox'); }, [open, selected]);
  useEffect(() => {
    if (!open) return;
    window.addEventListener('keydown', answerKey, true);
    return () => window.removeEventListener('keydown', answerKey, true);
  }, [open]);
  useEffect(() => {
    if (!open) return;
    const opener = document.activeElement as HTMLElement | null;
    closeButton.current?.focus();
    return () => { if (opener?.isConnected) opener.focus(); };
  }, [open]);
  // Opened for one empire (its VP-track row or a city card): show that empire's row and actions,
  // not the inbox boilerplate or the empires listed above it. An offer awaiting a reply keeps the
  // inbox first; it is the decision.
  useEffect(() => {
    if (open && selected !== null && !awaitingReply) scrollToTopWithin(openRowRef.current ?? relationRef.current);
  }, [open, selected, awaitingReply]);
  if (!view || !open) return null;
  const d = view.diplomacy;
  const teams = view.standings.filter((s) => s.team !== view.team && isTeamAlive(view, s.team));
  const team = selected !== null && teams.some((s) => s.team === selected) ? selected : teams[0]?.team;
  const relation = team === undefined ? undefined : relationWith(view, team);
  const name = (p: number) => empireOfPlayer(view, p);
  const forecastView = withLocalDiplomacy(view, humans, localOrders);
  const queued = forecastView.diplomacy.queued.filter((q) => q.player !== view.me);
  const errs = errorsWhere(orders, errors, (o) => otherTeam(view, o) !== null);
  const incoming = incomingOffers(view);
  const calls = incomingCalls(view);
  const pendingOffers = pendingDecisions(view, orders, errors, { humans, orders: localOrders, errors: localErrors }).flatMap((x) => x.kind === 'diplomacy' ? [x.id] : []);
  const currentOffer = pendingOffers[0];
  const outgoing = d.proposals.filter((p) => !incoming.includes(p));
  const teamActions = relation ? d.actions.filter((a) => otherTeam(view, a.order) === team && !('proposal' in a.order) && fitsRelation(a, relation)) : [];
  const unmet = teamActions.find(blockedByContact);
  return <aside className="report diplomacy-panel" aria-label="Diplomacy" data-game-overlay data-dismissible="diplomacy" onKeyDown={(e) => {
    e.stopPropagation();
    if (e.key === 'Escape') { e.preventDefault(); useGame.getState().set({ diplomacyOpen: false }); }
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
      {tab === 'relations' ? <RelationsView view={view} /> : <>
      {d.legacy ? <p>This save uses the original permanent-war rules.</p> : <>
        {incoming.length > 0 || calls.length > 0
          ? <section aria-label="Needs your decision"><h4 className="diplomacy-section">Needs your decision</h4>
              {calls.map((c) => <CallCard key={`call-${c.id}`} call={c} view={view} forecastView={forecastView} orders={orders} current={c.id === currentOffer} />)}
              {incoming.map((p) => <OfferCard key={p.id} p={p} view={view} forecastView={forecastView} orders={orders} current={p.id === currentOffer}
                ending={pendingOffers.includes(p.id) ? undefined : forecastView.diplomacy.queued.find((q) => endsOffers(q.order) && otherTeam(view, q.order) === proposalCounterparty(view, p, view.team))?.order} />)}
            </section>
          : <><h4 className="diplomacy-section">Inbox</h4><p className="muted">No pending proposals. Nothing needs your decision.</p></>}
        {outgoing.map((p) => <SentOffer key={p.id} p={p} view={view} forecastView={forecastView} orders={orders} />)}
        {d.superseded.map((s, i) => <article className="diplomacy-item" key={`superseded-${i}`}>
          <strong>{name(s.player)}’s order was dropped</strong>
          <p><Amounts text={describeOrder(s.order, { ...view, turn: s.reported_turn - 1 })} /></p>
          <p>{name(s.winner)} had the lower-numbered seat: <Amounts text={describeOrder(s.winner_order, { ...view, turn: s.reported_turn - 1 })} />.</p>
        </article>)}
        {queued.map((s, i) => <p key={`queued-${i}`}>Teammate queued: {name(s.player)} · <Amounts text={describeOrder(s.order, view)} /></p>)}
        {(d.replies??[]).length > 0 && <details className="diplomatic-replies" open><summary>Recent diplomatic replies</summary>{[...(d.replies??[])].reverse().slice(0,8).map((r)=><article className="diplomacy-item" key={`${r.proposal.id}-${r.effective_turn}`}><strong>{teamName(view,r.proposal.to)} {r.accepted?'accepted':'declined'} {treatyName(r.proposal)} · turn {r.effective_turn}</strong>{r.proposal.explanation?.reasons.map((reason,i)=><p key={i}>{reason}</p>)}</article>)}</details>}
        <TeamGifts view={view} />
        <h4 ref={relationRef} className="diplomacy-section">Empires</h4>
        <ul className="empire-list" aria-label="Empires">
          {teams.map((s) => {
            const rowRelation = relationWith(view, s.team);
            const state = rowRelation?.state ?? 'War';
            const isOpen = s.team === team;
            const aiMembers = empireMembers(view, s.team).filter((p) => !p.is_human);
            return <li key={s.team} ref={isOpen ? openRowRef : undefined} className={`empire-row${isOpen ? ' open' : ''}`} style={{ ['--empire' as string]: playerColor(empireMembers(view, s.team)[0]?.id) }}>
              <button className="empire-pick" aria-expanded={isOpen} aria-controls={isOpen ? 'empire-detail' : undefined}
                onClick={() => useGame.getState().set({ diplomacyTeam: s.team })}>
                <EmpireMark view={view} team={s.team} size={24} />
                <span className="empire-pick-identity"><span className="empire-pick-name">{empireTitle(view, s.team)}</span>{aiMembers.length > 0 && <small className="empire-pick-level">{aiMembers.map((p) => aiMembers.length === 1 ? `AI · ${aiLevel(p)}` : `${p.name}: ${aiLevel(p)}`).join(' · ')}</small>}</span>
                <span className={`empire-relation ${state.toLowerCase()}`} title={treatyDefinition(state) ?? undefined}><TreatyEmblem kind={state} size={16} />{state}{state !== 'War' && rowRelation && rowRelation.guarantee_until > view.turn && <> · truce through turn {rowRelation.guarantee_until - 1}</>}</span>
                <span className="empire-vp"><b>{s.vp}</b>/<Amount n={s.target} unit="VP" /></span>
              </button>
              <EmpireWhere view={view} team={s.team} />
              {/* Public record: an empire that broke an alliance, and how long its victim refuses it. */}
              {betrayalsBy(view, s.team).filter((b) => isTeamAlive(view, b.against)).map((b) => <p className="betrayal-line" key={`${b.against}-${b.turn}`}>
                Broke an alliance on turn {b.turn}: {b.against === view.team ? 'your team' : empireTitle(view, b.against)} won't ally until turn {b.until}
              </p>)}
              {isOpen && relation && <div className="empire-detail" id="empire-detail">
                <div className="relation-head">
                  {empireMembers(view, s.team)[0] && <PortraitRoundel faction={empireMembers(view, s.team)[0].faction} size={38} />}
                  {empireMembers(view, s.team).length === 1 && <p className="offer-quote">“{FACTION_INFO[empireMembers(view, s.team)[0].faction].motto}”</p>}
                  <TreatyEmblem kind={relation.state} size={38} />
                </div>
                {/* A Peace or an Alliance row's own lines state its terms in one sentence; only a War keeps its definition (how a Peace ends it). */}
                {relation.state === 'War' && treatyDefinition(relation.state) && <p className="treaty-definition"><strong>{relation.state}:</strong> {treatyDefinition(relation.state)}</p>}
                {relationText(relation, view).map((line) => <p key={line} className={line.startsWith('Protected until') ? 'diplomacy-protected' : undefined}><Amounts text={line} /></p>)}
                <RivalLevy view={view} team={s.team} />
                {/* An empire you haven't met: say so once; its Alliance offer shows disabled. */}
                {unmet && <p className="diplomacy-blocked-reason">{notMetText(unmet.blocked_reason!, empireTitle(view, team))}</p>}
                {teamActions.filter((a) => !a.blocked_reason || blockedByContact(a)).map((a, i) => <Action key={i} action={a} view={forecastView} orders={orders} />)}
                <Unavailable actions={teamActions.filter((a) => a.blocked_reason && !blockedByContact(a))} view={view} />
              </div>}
            </li>;
          })}
        </ul>
        {errs.map((e, i) => <p className="order-error" key={i}>{e}</p>)}
        <details className="diplomacy-rules"><summary>How diplomacy works</summary>
          <p><strong>Peace:</strong> {treatyDefinition('Peace')}</p>
          <p><strong>Alliance:</strong> {treatyDefinition('Alliance')}</p>
          <p><strong>War:</strong> {treatyDefinition('War')}</p>
          {d.exploration && <p>Own-team exploration: {d.exploration.native_land} land tiles. Borrowed current sight: {d.exploration.borrowed_current} tiles.</p>}
          <p>{RESOLUTION_RULE}</p>
          <p className="muted">{TEAMMATE_RULE}</p>
        </details>
      </>}
      </>}
    </div>
  </aside>;
}

function RelationsView({ view }: { view: PlayerView }) {
  const model = publicRelationsModel(view);
  const mark = (team: number) => model.empires.find((e) => e.team === team)?.known
    ? <EmpireMark view={view} team={team} size={24} />
    : <span className="relations-unknown" aria-hidden="true">?</span>;
  return <section className="relations-view" aria-label="Relations between empires">
    <h4 className="diplomacy-section">Relations</h4>
    <p className="muted relations-key"><span className="relations-sample war"><TreatyEmblem kind="War" size={20} /> War</span><span className="relations-sample alliance"><TreatyEmblem kind="Alliance" size={20} /> Alliance</span><span className="relations-sample"><TreatyEmblem kind="Peace" size={20} /> Peace</span></p>
    <div className="relations-scroll" role="region" aria-label="Relations table" tabIndex={0}>
      <table className="relations-table" style={{ minWidth: 140 + model.empires.length * 60 }}>
        <thead><tr><th scope="col">Empire</th>{model.empires.map((empire) => <th scope="col" key={empire.team} title={empire.name}>{mark(empire.team)}<span className="sr-only">{empire.name}</span></th>)}</tr></thead>
        <tbody>{model.empires.map((empire, row) => <tr key={empire.team}>
          <th scope="row" title={empire.name}>{mark(empire.team)}<span>{empire.name}</span></th>
          {model.cells[row].map((cell) => <td key={cell.b} className={cell.state === 'Self' ? 'relations-self' : undefined}>
            {cell.state === 'Self' ? <span role="img" aria-label={`${empire.name}, itself`} title={empire.name} />
              : cell.state === 'Unknown' ? <span className="relations-unknown" title={cell.title} aria-label={cell.title}>?</span>
                : cell.state === 'War' ? <span className="relations-cell war" title={cell.title} aria-label={cell.title}><TreatyEmblem kind="War" size={23} /></span>
                  : cell.state === 'Alliance' ? <span className="relations-cell alliance" title={cell.title} aria-label={cell.title}><TreatyEmblem kind="Alliance" size={23} /></span>
                    : <span className="relations-cell" title={cell.title} aria-label={cell.title}><TreatyEmblem kind="Peace" size={23} /></span>}
          </td>)}
        </tr>)}</tbody>
      </table>
    </div>
    <p className="muted relations-note">{view.ai_alliances && 'In this game the AI empires started allied in pairs. '}Wars and Alliances are public. Unknown empires are hidden until you meet them.{model.empires.length > 4 && ' Scroll sideways to see every empire.'}</p>
  </section>;
}
