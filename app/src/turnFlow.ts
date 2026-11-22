import type { CityView, Event, Faction, Order, OrderError, PlayerView, ProductionItem } from './types';
import { effectiveResearchQueue, pendingUnitOrder, productionQueue, unitsNeedingOrders } from './orders';
import type { ReleasedUnit } from './releasedGarrison';
import { idleCitiesNeedingProduction } from './ui/forecastWarnings';
import { incomingCalls, incomingOffers, offerSettledBy, proposalCounterparty, teamName, withLocalDiplomacy } from './diplomacy';
import { peaceScope } from './peaceScope';
import { itemName, sameItem } from './data';
import { disbandUnitLabel } from './ui/forecastWarnings';
import { freeTechTarget } from './goalActions';
import { citiesUnderAttack } from './siege';
import { aircraftNeedingOrders, type Hold } from './airWing';
import { strikeRange } from './airText';
import { goalDecisionText, readyGoalAction } from './goalDecision';
import { AUTOMATION_NO_CHOICE, cityAutomationPreview } from './automation';
import { tombOnAfterOrders } from './goalCities';
import { coloniaSiteKnown } from './settlement';

export type DecisionTarget = { kind: 'unit' | 'city' | 'siege'; id: number } | { kind: 'air'; id: number; city?: number } | { kind: 'research' } | { kind: 'freeTechs' } | { kind: 'diplomacy'; id: number; team: number }
  | { kind: 'goal'; action: 'Swarm' | 'Sakoku' | 'Broadcast' | 'CallTheWild' | 'Retire' | 'Colonia'; title: string; line: string; act?: string }
  | { kind: 'treasury'; target: { kind: 'unit' | 'city'; id: number } | null };
/** `issue`: why the unit's queued order fails; `fix`: that as one sentence naming the unit. */
/** `healed`: a unit whose Heal order ended at full health in the turn that just resolved. */
export type PendingDecision = DecisionTarget & { label: string; issue?: string; fix?: string; healed?: boolean;
  /** The city whose occupation lock just lifted: this unit was holding it and asks for orders once. */
  released?: string };

/** Why a retained queue head can't start: core's placement blockers (population, spawn room). */
function waitingReason(city: CityView, item: ProductionItem, faction?: Faction): string {
  const name = itemName(item, faction);
  if (item.kind === 'Unit' && item.unit === 'Settler' && city.population < 2) return `${name} needs population 2`;
  return item.kind === 'Unit' ? `${name} has no free spawn tile` : `${name} is blocked`;
}

/** An idle city chooses a build; a waiting queue says what it waits for. */
function cityDecisionLabel(view: PlayerView, city: CityView, orders: Order[]): string {
  const waiting = productionQueue(orders, city.id, city.detail?.producing ?? null, city.detail?.production_queue)[0];
  const faction = view.players.find((p) => p.id === city.owner)?.faction;
  return waiting ? `${city.name}: queue waiting (${waitingReason(city, waiting, faction)})` : `Choose what ${city.name} builds`;
}

/**
 * Next turn's treasury goes negative and core will disband a unit (rules unchanged). The forecast
 * ignores this turn's orders, so a pending disband, or a production change away from the unit a
 * city is completing, counts as dealing with it.
 */
function treasuryDecision(view: PlayerView, orders: Order[]): PendingDecision | null {
  const forecast = view.economy.bankruptcy;
  const disband = forecast?.disband;
  if (!forecast || forecast.gold.total >= 0 || !disband) return null;
  if (orders.some((o) => o.type === 'DisbandUnit')) return null;
  const city = disband.city !== null ? view.cities.find((c) => c.id === disband.city) : undefined;
  if (city?.detail) {
    const next = productionQueue(orders, city.id, city.detail.producing, city.detail.production_queue)[0] ?? null;
    if (!sameItem(next, { kind: 'Unit', unit: disband.kind })) return null;
  }
  const target = city ? { kind: 'city' as const, id: city.id } : disband.unit !== null ? { kind: 'unit' as const, id: disband.unit } : null;
  return { kind: 'treasury', target,
    label: `Treasury ${forecast.gold.total} next turn: ${disbandUnitLabel(view)} will be disbanded; change production or upkeep` };
}

/** The latest resolution's events per seat; a heal that finished in it names itself in the queue. */
type LocalReport = { events: Event[][]; released?: ReleasedUnit[][] } | null;

/** This seat's units the occupation lock released in the turn that just resolved (unanswered ones stay in the queue). */
function releasedUnits(view: PlayerView, orders: Order[], local?: { humans: number[]; seat?: number; report?: LocalReport }): ReleasedUnit[] {
  const seat = local?.seat ?? local?.humans.indexOf(view.me) ?? -1;
  return (local?.report?.released?.[seat] ?? []).filter((r) => {
    const unit = view.units.find((u) => u.id === r.unit);
    return unit?.owner === view.me && unit.occupation_lock == null && !pendingUnitOrder(orders, r.unit);
  });
}

/** Units whose Heal order ended at full health in the turn that just resolved (this seat's own notices). */
function healedUnits(view: PlayerView, local?: { humans: number[]; seat?: number; report?: LocalReport }): Set<number> {
  const seat = local?.seat ?? local?.humans.indexOf(view.me) ?? -1;
  const events = local?.report?.events?.[seat] ?? [];
  return new Set(events.flatMap((e) => (e.type === 'UnitHealed' && e.owner === view.me ? [e.unit] : [])));
}

/** The cycle and End Turn review share one list. Invalid orders never hide a decision. */
/**
 * A city an enemy can strike next turn that the player has not opened this turn (user, 2026-10-08:
 * "a city under attack is never a surprise"). Opening the city, from This Turn, the banner or the
 * Cities list, answers it; it stays in This Turn and on the map while the siege lasts.
 */
function siegeDecisions(view: PlayerView, seen: { turn: number; cities: number[] } | undefined): PendingDecision[] {
  const opened = seen?.turn === view.turn ? seen.cities : [];
  return citiesUnderAttack(view, null)
    .filter((a) => a.kind === 'attack' && a.attackers.length > 0 && !opened.includes(a.cityId))
    .map((a): PendingDecision => ({ kind: 'siege', id: a.cityId, label: `${a.name} is under attack: ${a.forecast.text.charAt(0).toLowerCase()}${a.forecast.text.slice(1)}` }));
}

export function pendingDecisions(view: PlayerView, orders: Order[], errors: OrderError[] = [], local?: { humans: number[]; orders: Order[][]; errors: OrderError[][]; siegeSeen?: Record<number, { turn: number; cities: number[] }>; airHeld?: Record<number, Hold>; seat?: number; report?: LocalReport; goalDecisionSeen?: Record<number, boolean>; selection?: { kind: string; id?: number } | null }): PendingDecision[] {
  const healed = healedUnits(view, local);
  if (view.finished || view.players.find((p) => p.id === view.me)?.alive === false) return [];
  const valid = orders.filter((_, i) => !errors.some((e) => e.index === i));
  const unchosen = new Set(cityAutomationPreview(view, valid)?.unchosen_cities ?? []);
  const idleCityDecisions: PendingDecision[] = idleCitiesNeedingProduction(view, valid)
    .filter((c) => !unchosen.has(c.id)).sort((a, b) => a.id - b.id)
    .map((c) => ({ kind: 'city', id: c.id, label: cityDecisionLabel(view, c, valid) }));
  const plans = view.diplomacy ? withLocalDiplomacy(view, local?.humans ?? [view.me], local
    ? local.orders.map((os, seat) => os.filter((_, i) => !local.errors[seat]?.some((e) => e.index === i))) : [valid]).diplomacy.queued : [];
  return [
    // Answered, or ended by a queued war or exit toward its sender: no longer a decision.
    ...incomingOffers(view).filter((p) => !offerSettledBy(view, plans, p))
      .sort((a, b) => a.id - b.id).map((p): PendingDecision => ({ kind: 'diplomacy', id: p.id, team: proposalCounterparty(view, p, view.team),
        label: p.kind === 'Alliance' ? `Alliance offer: respond by turn ${p.respond_by}`
          : `${peaceScope(view, p).title} · respond by turn ${p.respond_by}` })),
    // A call to arms is answered with Join or Decline; a queued reply settles it. Letting it lapse declines.
    ...incomingCalls(view).filter((c) => !plans.some((q) => q.order.type === 'RespondToProposal' && q.order.proposal === c.id))
      .sort((a, b) => a.id - b.id).map((c): PendingDecision => ({ kind: 'diplomacy', id: c.id, team: c.ally,
        label: `Call to arms: ${teamName(view, c.ally)} asks you to join its war on ${teamName(view, c.target)} (answer by turn ${c.respond_by})` })),
    // A rival's broadcast left free technologies to choose: the prompt persists until they are chosen.
    ...siegeDecisions(view, local?.siegeSeen?.[view.me]),
    ...[readyGoalAction(view, valid, local?.goalDecisionSeen ?? {}, local?.selection?.kind === 'unit' ? local.selection.id ?? null : null)].filter((g): g is NonNullable<typeof g> => !!g)
      .map((g): PendingDecision => { const t = goalDecisionText(g.kind, g.vp, g.kind === 'Colonia' ? coloniaSiteKnown(view) : null); return { kind: 'goal', action: g.kind, title: t.title, line: t.line, act: t.act, label: `${t.title}: ${t.line}` }; }),
    ...(view.actions?.free_techs && !valid.some((o) => o.type === 'Action' && o.action.action === 'ChooseFreeTechs')
      ? [{ kind: 'freeTechs' as const, label: `Choose ${freeTechTarget(view)} free ${freeTechTarget(view) === 1 ? 'technology' : 'technologies'}` }] : []),
    ...[treasuryDecision(view, valid)].filter((d): d is PendingDecision => d !== null),
    ...view.cities.filter((c) => c.owner === view.me && unchosen.has(c.id) && !tombOnAfterOrders(view, valid)).sort((a, b) => a.id - b.id)
      .map((c): PendingDecision => ({ kind: 'city', id: c.id, label: `Choose what ${c.name} builds · ${AUTOMATION_NO_CHOICE}` })),
    // The founding decision is already made: the first playable choice is the capital's build.
    ...(view.turn === 1 ? idleCityDecisions : []),
    ...unitsNeedingOrders(view, valid).map((u): PendingDecision => {
      const invalid = orders.findIndex((o, i) => 'unit' in o && o.unit === u.id && errors.some((e) => e.index === i));
      const issue = invalid >= 0 ? errors.find((e) => e.index === invalid) : undefined;
      const name = `${itemName({ kind: 'Unit', unit: u.kind }, view.players.find((p) => p.id === u.owner)?.faction)} #${u.id}`;
      const fix = issue ? `${name}: ${issue.message}${/[.!?]$/.test(issue.message) ? '' : '.'}` : undefined;
      return { kind: 'unit', id: u.id, issue: issue?.message, fix, ...(healed.has(u.id) && !fix ? { healed: true } : {}),
        label: fix ? `${fix} Change or remove its order.` : `${name}: ${healed.has(u.id) ? 'healed to full, needs orders' : 'needs orders'}` };
    }),
    // A garrison the occupation lock fortified for you asks for orders once when its city is secure.
    ...releasedUnits(view, valid, local)
      .filter((r) => !unitsNeedingOrders(view, valid).some((u) => u.id === r.unit))
      .map((r): PendingDecision => ({ kind: 'unit', id: r.unit, released: r.city,
        label: `${itemName({ kind: 'Unit', unit: view.units.find((u) => u.id === r.unit)!.kind }, view.players.find((p) => p.id === view.me)?.faction)} #${r.unit}: ${r.city} is secure, needs orders` })),
    // An aircraft with no order, hold or sleep yet needs orders like any unit, with a target in reach or not.
    ...aircraftNeedingOrders(view, valid, local?.airHeld?.[view.me]).map(({ plane, city, targets }): PendingDecision => {
      const name = itemName({ kind: 'Unit', unit: plane.kind }, view.players.find((p) => p.id === view.me)?.faction);
      return { kind: 'air', id: plane.id, city: city.id,
        label: targets
          ? `${name} in ${city.name}: ${targets} target${targets === 1 ? '' : 's'} within ${strikeRange(plane.kind)} tiles, strike or ${plane.kind === 'Fighter' ? 'stay on guard' : 'hold'}`
          : `${name} in ${city.name}: needs orders` };
    }),
    ...(view.turn === 1 ? [] : idleCityDecisions),
    ...(!effectiveResearchQueue(valid, view.research).length && view.research.available.length
      ? [{ kind: 'research' as const, label: 'Choose research: the queue is empty' }] : []),
  ];
}

/** Incoming offers take priority; then visit units, cities and research without readying. */
export function nextDecision(list: PendingDecision[], current: DecisionTarget | null): PendingDecision | null {
  if (!list.length) return null;
  if (list[0].kind === 'diplomacy') return list[0];
  // A city an enemy can strike next turn comes before every routine decision.
  const siege = list.find((d) => d.kind === 'siege');
  if (siege) return siege;
  if (current?.kind === 'unit') {
    const fix = list.find((d) => d.kind === 'unit' && d.id === current.id && d.issue);
    if (fix) return fix;
  }
  // A broken order is the next decision on every surface: it outranks the other units, cities and
  // research (an offer or the treasury, listed first, still come first).
  const broken = list.find((d) => d.issue);
  if (broken && list[0].kind !== 'treasury') return broken;
  if (!current || current.kind === 'research' || current.kind === 'treasury' || current.kind === 'freeTechs') return list[0];
  if (list[0].kind === 'city' && current.kind === 'city' && current.id === list[0].id) {
    return list.find((d) => d.kind === 'unit') ?? list.find((d) => d.kind === 'research') ?? list[0];
  }
  const rank = { diplomacy: -2, siege: -1.8, goal: -1.6, freeTechs: -1.5, treasury: -1, unit: 0, air: 0.5, city: 1, research: 2 };
  return list.find((d) => rank[d.kind] > rank[current.kind] ||
    (d.kind === current.kind && 'id' in d && 'id' in current && d.id > current.id)) ?? list[0];
}
