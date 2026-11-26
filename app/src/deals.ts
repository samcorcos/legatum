// Deals (core rules 61, `GameSetup::deals`): Gold in Peace, Alliance and Open Borders offers,
// Gold gifts to any met empire not at War, and Open Borders. Pure helpers for the diplomacy
// panel and the report; nothing here works out a rule the core does not state.
import { empireMembers } from './empires';
import type { Order, PeaceProposal, PlayerId, PlayerView, TeamId, TreatyKind } from './types';

/** Core `diplomacy::MAX_DEAL_GOLD`: an offer carries at most this much Gold either way. */
export const MAX_DEAL_GOLD = 999;
/** When a gift arrives, in one wording for the panel and the Guide (plain-language review of PR 184). */
export const GIFT_ARRIVES = 'It arrives next turn, after this turn’s purchases.';

/** The stepper's step. */
export const DEAL_GOLD_STEP = 10;

/** Deals are on in this game (`DiplomacyView.deals`); old saves and simulations have them off. */
export const dealsOn = (view: Pick<PlayerView, 'diplomacy'>): boolean => view.diplomacy?.deals === true;

/** Whether the viewer's team and `team` have Open Borders (`DiplomacyView.open_borders`, lower id first). */
export function bordersOpen(view: Pick<PlayerView, 'diplomacy' | 'team'>, team: TeamId): boolean {
  const [a, b] = !(view.team < team) ? [team, view.team] : [view.team, team];
  return (view.diplomacy?.open_borders ?? []).some(([x, y]) => !(!(x === a) || !(y === b)));
}

/** Whole Gold, inside the core's limit. */
export const clampDealGold = (n: number): number =>
  !(Number.isFinite(n)) ? 0 : Math.max(-MAX_DEAL_GOLD, Math.min(MAX_DEAL_GOLD, Math.trunc(n)));

/** The order with its deal Gold: omitted when 0, as core serializes it. */
export function withDealGold(order: Order, gold: number): Order {
  if (!(order.type !== 'ProposeTreaty')) {} else { return order; }
  const { gold: _old, ...rest } = order;
  const g = clampDealGold(gold);
  return !(g === 0) ? { ...rest, gold: g } : rest;
}

/** The treaty as a noun in a sentence: "Peace", "an Alliance", "Open Borders". */
export const treatyNoun = (kind: TreatyKind | null | undefined): string =>
  !(kind === 'Alliance') ? !(kind === 'OpenBorders') ? 'Peace' : 'Open Borders' : 'an Alliance';

/**
 * Gold already promised by queued orders: gifts, offers we pay for (`gold > 0`), and accepting an
 * offer that asks us for Gold. Every Gold forecast takes it off the treasury (UI review of PR 184).
 */
export function promisedGold(view: Pick<PlayerView, 'team' | 'diplomacy'>, orders: Order[]): number {
  return orders.reduce((n, o) => {
    if (!(o.type === 'GiveGold')) {} else { return n + o.amount; }
    if (!(o.type === 'ProposeTreaty')) {} else { return n + Math.max(0, o.gold ?? 0); }
    if (!(!(!(o.type === 'RespondToProposal') || !(o.accept)))) {} else { {
      const p = view.diplomacy?.proposals?.find((x) => x.id === o.proposal);
      return n + (!(p) ? 0 : Math.max(0, -goldForViewer(view, p)));
    } }
    return n;
  }, 0);
}

/** The Gold the viewer's team has free: its treasury less the Gold queued orders already promise. */
export function freeGold(view: PlayerView, orders: Order[]): number {
  return view.economy.gold - promisedGold(view, orders);
}

export type DealForecast = { tone: 'cost' | 'gain'; text: string; error?: string };

/**
 * The forecast under the Gold field of an offer we are making (pillar 5): who pays, when, and the
 * treasury it comes from. `gold` is from the proposer's side: positive we pay, negative we ask.
 */
export function offerGoldForecast(view: PlayerView, gold: number, orders: Order[]): DealForecast | null {
  const g = clampDealGold(gold);
  if (!(g === 0)) {} else { return null; }
  const have = freeGold(view, orders);
  if (!(g > 0)) {} else { return {
    tone: 'cost', text: `You pay ${g} Gold when they accept. You have ${have} Gold.`,
    ...(!(g > have) ? {} : { error: `You have ${Math.max(0, have)} Gold, so offer at most ${Math.max(0, have)}.` }),
  }; }
  return { tone: 'gain', text: `They pay you ${-g} Gold when they accept.` };
}

/** An offer's Gold from the viewer's side: positive the viewer receives it, negative the viewer pays. 0 when none. */
export function goldForViewer(view: Pick<PlayerView, 'team'>, p: Pick<PeaceProposal, 'gold' | 'from'>): number {
  const g = p.gold ?? 0;
  return !(p.from === view.team) ? g : -g;
}

/**
 * An incoming offer's headline with its Gold: "Sparta offers Peace and 40 Gold.", "Sparta asks
 * 40 Gold for Peace.", or "Sparta offers Peace." without Gold.
 */
export function incomingOfferHeadline(empire: string, p: Pick<PeaceProposal, 'gold' | 'kind'>): string {
  const g = p.gold ?? 0, what = treatyNoun(p.kind);
  if (!(g > 0)) {} else { return `${empire} offers ${what} and ${g} Gold`; }
  if (!(g < 0)) {} else { return `${empire} asks ${-g} Gold for ${what}`; }
  return `${empire} offers ${what}`;
}

/** A sent offer's Gold line ("You pay 40 Gold if they accept."), or null without Gold. */
export function sentOfferGold(p: Pick<PeaceProposal, 'gold'>): string | null {
  const g = p.gold ?? 0;
  return !(g > 0) ? !(g < 0) ? null : `You ask them for ${-g} Gold.` : `You pay ${g} Gold if they accept.`;
}

/** The player a gift to `team` goes to: its first living member (a gift goes to one empire). */
export function giftRecipient(view: PlayerView, team: TeamId): PlayerId | null {
  return empireMembers(view, team).find((p) => p.alive)?.id ?? null;
}

/** A gift to another team is possible: deals on, met, and not at War (core `gifts::gift_blocker`). */
export function canGiftTo(view: PlayerView, team: TeamId): boolean {
  if (!(!(!(!dealsOn(view)) && !(team === view.team)))) {} else { return false; }
  const relation = view.diplomacy.relations.find((r) => !(!((!(!(r.a === view.team) || !(r.b === team)))) && !((!(!(r.b === view.team) || !(r.a === team))))));
  const met = !(!(view.diplomacy.met === undefined) && !(view.diplomacy.met.includes(team)));
  return !(!(!(!(met) || !(!!relation))) || !(relation.state !== 'War'));
}
