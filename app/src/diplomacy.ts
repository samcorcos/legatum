import { isBarbarian } from './barbarians';
import type { DiplomaticAction, Order, PlayerView, TeamId, TreatyKind, Hex, Event, UnitKind } from './types';
import { hexAt, neighbors, tileIndex } from './hex';
import { moveCost } from './data';

export function teamName(view: PlayerView, team: TeamId): string {
  return view.players.filter((p) => p.team === team).map((p) => p.name).join(' & ') || `Team ${team}`;
}

/** The teams fighting on `team`'s side: itself and everyone it is allied with (core `side_of`). */
export function sideTeams(view: Pick<PlayerView, 'diplomacy'>, team: TeamId): TeamId[] {
  return view.diplomacy?.alliances?.find((side) => side.includes(team)) ?? [team];
}
/** Whom `team` is dealing with in the pairwise proposal (core `counterparty`). */
export function proposalCounterparty(_view: Pick<PlayerView, 'diplomacy'>, p: { kind?: TreatyKind | null; from: TeamId; to: TeamId }, team: TeamId): TeamId {
  return team === p.from ? p.to : p.from;
}
/** Calls to arms waiting for this team's answer (core `DiplomacyView::calls`). */
export function incomingCalls(view: Pick<PlayerView, 'diplomacy'>) {
  return view.diplomacy?.calls ?? [];
}
/** Offers waiting for this team's answer. */
export function incomingOffers(view: PlayerView) {
  return view.diplomacy?.proposals.filter((p) => p.needs ? p.needs.includes(view.team) : p.to === view.team) ?? [];
}

export function diplomaticCost(view: PlayerView, order: Order): number | null {
  return view.diplomacy?.actions.find((a) => Object.entries(order).filter(([key])=>key!=='explanation').every(([key, value]) =>
    (a.order as unknown as Record<string, unknown>)[key] === value))?.cost ?? null;
}

/** Team IDs and player IDs differ in hotseat/team games. Public does not mean personal. */
export function diplomaticTeams(event: Event): TeamId[] {
  if ('relation' in event) return [event.relation.a, event.relation.b];
  if ('proposal' in event) return event.proposal.required_teams?.length
    ? event.proposal.required_teams : [event.proposal.from, event.proposal.to];
  if ('call' in event) return [event.call.ally, event.call.team];
  switch (event.type) {
    case 'TreatyEnded': return [event.a, event.b];
    case 'TreatyBreached': return [event.by, event.with];
    case 'AllianceBetrayed': return [event.betrayal.by, event.betrayal.against];
    case 'OrderSuperseded': return [event.decision.team];
    default: return [];
  }
}
export function relationWith(view: PlayerView, team: TeamId) {
  return view.diplomacy?.relations.find((r) => (r.a === view.team && r.b === team) || (r.b === view.team && r.a === team));
}
export function atWar(view: PlayerView, owner: number): boolean {
  // Barbarians are always hostile: no relation, no Declare War step.
  if (isBarbarian(owner)) return true;
  const team = view.players.find((p) => p.id === owner)?.team;
  if (team === undefined || team === view.team) return false;
  return !view.diplomacy || view.diplomacy.legacy || relationWith(view, team)?.state === 'War';
}
/**
 * Barbarians never attack the Celts (core `rules::barbarians::spares`, `PlayerView::barbarians_spare_me`):
 * a public rule, true when the viewer is the Celts.
 */
export function barbariansSpareMe(view: PlayerView): boolean {
  return view.players.find((p) => p.id === view.me)?.faction === 'Celts';
}

/**
 * Whether `owner` is a threat to the viewer (core `rules::is_threat`): at War, except barbarians when they spare the
 * viewer. Warnings, alerts, wake-ups and danger marks ask this; orders that attack ask `atWar`, because the Celts may
 * still fight barbarians when they choose to.
 */
export function isThreat(view: PlayerView, owner: number): boolean {
  return atWar(view, owner) && !(isBarbarian(owner) && barbariansSpareMe(view));
}

/**
 * A unit or city of another team at Peace at a hex (military units first). A move never attacks
 * it: attacking needs a War declaration first (playtest G2).
 */
export function peacefulAt(v: PlayerView, h: Hex): { kind: 'unit' | 'city'; owner: number; team: TeamId; id: number } | null {
  const teamOf = (owner: number) => v.players.find((p) => p.id === owner)?.team;
  const peaceful = (owner: number) => { const t = teamOf(owner); return t !== undefined && t !== v.team && !atWar(v, owner); };
  const here = (p: Hex) => p.q === h.q && p.r === h.r;
  const units = v.units.filter((u) => here(u.pos) && peaceful(u.owner)).sort((a, b) => Number(b.military) - Number(a.military));
  const c = v.cities.find((c) => here(c.pos) && peaceful(c.owner));
  const u = units[0];
  if (u) return { kind: 'unit', owner: u.owner, team: teamOf(u.owner)!, id: u.id };
  if (c) return { kind: 'city', owner: c.owner, team: teamOf(c.owner)!, id: c.id };
  return null;
}

/**
 * A military unit or city of another team at Peace (or under a treaty) that a ranged attack could
 * hit once War is declared. Lone civilians can't be shot, so they never prompt a declaration.
 */
export function peacefulRangedAt(v: PlayerView, h: Hex): { kind: 'unit' | 'city'; owner: number; team: TeamId; id: number } | null {
  const target = peacefulAt(v, h);
  if (!target) return null;
  if (target.kind === 'city' || v.units.find((u) => u.id === target.id)?.military) return target;
  const c = v.cities.find((c) => c.pos.q === h.q && c.pos.r === h.r && c.owner === target.owner);
  return c ? { kind: 'city', owner: c.owner, team: target.team, id: c.id } : null;
}

/** Own team, War, an Alliance (passage), or the land of a Courier Hall's holder (open to every empire). */
export function canEnter(view: PlayerView, _kind: UnitKind, owner: number | null, city = false): boolean {
  const team = owner === null ? undefined : view.players.find((p) => p.id === owner)?.team;
  // A closed country (Sakoku) lets nobody in at Peace: no Alliance, no courier passage, no open road.
  const closed = owner !== null && view.players.some((p) => p.id === owner && p.sakoku_turns !== undefined && p.sakoku_turns !== null);
  const allied = team !== undefined && relationWith(view, team)?.state === 'Alliance';
  if (city && team !== undefined && team !== view.team && !atWar(view, owner!)) return false;
  return owner === null || team === view.team || atWar(view, owner)
    || (!closed && (allied
      || (view.courier_halls ?? []).some((h) => h.holder === owner)));
}
export function neutralBlocker(view: PlayerView, at: Hex): boolean {
  return (view.units ?? []).some((u) => u.pos.q === at.q && u.pos.r === at.r
    && view.players.find((p) => p.id === u.owner)?.team !== view.team
    && relationWith(view, view.players.find((p) => p.id === u.owner)?.team ?? -1)?.state !== 'Alliance'
    && !atWar(view, u.owner));
}
export function otherTeam(view: PlayerView, order: Order): TeamId | null {
  if (order.type === 'BreakTreaty') return order.with;
  if (order.type === 'DeclareWar') return order.against;
  if (order.type === 'ProposeTreaty') return order.to;
  if (order.type === 'RespondToProposal' || order.type === 'WithdrawProposal') {
    const call = order.type === 'RespondToProposal' ? incomingCalls(view).find((c) => c.id === order.proposal) : undefined;
    if (call) return call.ally;
    const p = view.diplomacy.proposals.find((p) => p.id === order.proposal);
    return p ? proposalCounterparty(view, p, view.team) : null;
  }
  return null;
}
/**
 * Orders that end every offer between the two teams when the turn resolves: War and leaving an
 * Alliance each change the relation, and core withdraws the pair's offers then.
 */
export const endsOffers = (o: Order) => o.type === 'DeclareWar' || o.type === 'BreakTreaty';

/**
 * The queued order (ours or a teammate's) that already settles incoming offer `p`: a reply to it,
 * or a war or exit toward its sender (playtest 8.9.2: declaring War on Switzerland left
 * its Alliance offer as the turn's primary decision, and answering it again cancelled the war).
 */
export function offerSettledBy(view: PlayerView, queued: { order: Order }[], p: { id: number; kind?: TreatyKind | null; from: TeamId; to: TeamId }): Order | undefined {
  const other = proposalCounterparty(view, p, view.team);
  return queued.find((q) => (q.order.type === 'RespondToProposal' && q.order.proposal === p.id)
    || (endsOffers(q.order) && otherTeam(view, q.order) === other))?.order;
}

/**
 * One diplomatic order per other team (core's rule), so a new one replaces the old. Declining an
 * offer from an empire this plan already declares War on (leaves, pays off) changes nothing:
 * that order ends the offer anyway, and the reply must never discard it.
 */
export function queueDiplomacy(orders: Order[], order: Order, view: PlayerView): Order[] {
  const team = otherTeam(view, order);
  if (order.type === 'RespondToProposal' && !order.accept && team !== null
    && orders.some((o) => endsOffers(o) && otherTeam(view, o) === team)) return orders;
  return [...orders.filter((o) => team === null || otherTeam(view, o) !== team), order];
}


/** Edge permissions mirror core egress distance, using only the team's known map. */
export function borderSteps(view: PlayerView, kind: UnitKind, from: Hex): (a: Hex, b: Hex) => boolean {
  const index = (h: Hex) => tileIndex(h, view.width, view.height);
  const open = (h: Hex) => index(h) >= 0 && canEnter(view, kind, view.tiles[index(h)].owner);
  if (open(from)) return (_, to) => open(to);
  const dist = new Int32Array(view.tiles.length).fill(0x7fffffff);
  const queue: number[] = [];
  // A closed city is a wall, not a stepping stone: nobody stops in a foreign city at Peace, so
  // no exit route runs through one (core `path::exit_distances_around`).
  const cityTiles = new Set((view.cities ?? []).map((c) => index(c.pos)));
  view.tiles.forEach((t, i) => {
    if (moveCost(t.terrain ?? 'Grassland') !== null && canEnter(view, kind, t.owner)) {
      dist[i] = 0; queue.push(i);
    }
  });
  for (let head = 0; head < queue.length; head++) {
    const i = queue[head];
    for (const nb of neighbors(hexAt(i, view.width))) {
      const j = index(nb);
      if (j < 0 || moveCost(view.tiles[j].terrain ?? 'Grassland') === null || cityTiles.has(j) || dist[j] <= dist[i] + 1) continue;
      dist[j] = dist[i] + 1; queue.push(j);
    }
  }
  return (a, b) => open(b) || (!open(a) && index(a) >= 0 && index(b) >= 0 && dist[index(b)] < dist[index(a)]);
}

/** Merge only local human teammates' diplomatic plans into the same contract online publishes. */
export function withLocalDiplomacy(view: PlayerView, humans: number[], orders: Order[][]): PlayerView {
  const local = new Set(humans);
  const queued = [...view.diplomacy.queued.filter((q) => !local.has(q.player)), ...orders.flatMap((os, seat) => view.players.some((p) => p.id === humans[seat] && p.team === view.team) ? os.filter((o) => otherTeam(view, o) !== null).map((order) => ({ player: humans[seat], order })) : [])];
  queued.sort((a, b) => a.player - b.player);
  return { ...view, diplomacy: { ...view.diplomacy, queued } };
}
/** Player-facing name of a proposal kind (a missing kind is a Peace offer). */
export const offerLabel = (p: { kind?: TreatyKind | null }) => treatyLabel(p.kind);
export const treatyLabel = (kind?: TreatyKind | null) => kind ?? 'Peace';
/** Betrayal records still refusing Alliances where `team` broke an alliance. */
export function betrayalsBy(view: PlayerView, team: TeamId) {
  return (view.diplomacy?.betrayals ?? []).filter((b) => b.by === team);
}


export function sameDiplomaticAction(a: Order,b: Order): boolean {
  if(a.type==='RespondToProposal'&&b.type==='RespondToProposal')return a.proposal===b.proposal&&a.accept===b.accept;
  return JSON.stringify(a)===JSON.stringify(b);
}

/** Core's `diplomacy::NOT_MET` prefix: an Alliance needs first contact. */
const NOT_MET_PREFIX = "You haven't met this empire yet";

/** Whether a blocked action is blocked only because the empires haven't met. */
export const blockedByContact = (action: DiplomaticAction) => !!action.blocked_reason?.startsWith(NOT_MET_PREFIX);

/** Core's "haven't met" reason with the empire named ("You haven't met India yet: …"). */
export function notMetText(reason: string, empire: string): string {
  return reason.startsWith(NOT_MET_PREFIX) ? `You haven't met ${empire} yet${reason.slice(NOT_MET_PREFIX.length)}` : reason;
}

/** The declaration an attack on `team` needs: the plain one, or a paid breach if a treaty protects them. */
export function warDeclaration(v: PlayerView, team: TeamId): { action: DiplomaticAction } | { blocked: string } {
  const all = (v.diplomacy?.actions ?? []).filter((a) => a.order.type === 'DeclareWar' && a.order.against === team);
  const plain = all.find((a) => a.order.type === 'DeclareWar' && !a.order.breach);
  const breach = all.find((a) => a.order.type === 'DeclareWar' && a.order.breach);
  if (plain && !plain.blocked_reason) return { action: plain };
  if (relationWith(v, team)?.security && breach && !breach.blocked_reason) return { action: breach };
  return { blocked: (relationWith(v, team)?.security ? breach?.blocked_reason : plain?.blocked_reason)
    ?? plain?.blocked_reason ?? breach?.blocked_reason ?? 'War cannot be declared on this team now.' };
}

/** War on `team` is already queued this turn and opens before attacks resolve (a surprise war). */
export function warQueuedNow(v: PlayerView, orders: Order[], team: TeamId): boolean {
  const war = warDeclaration(v, team);
  return 'action' in war && war.action.effective_turn === v.turn
    && orders.some((o) => o.type === 'DeclareWar' && o.against === team);
}
