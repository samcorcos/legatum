// Pure helpers for building and editing a seat's pending orders. Orders stay pending until the
// seat is Ready; every edit goes through here so the rules (one unit action per unit, one
// production/buy per city, one research choice) hold before validate() is even called.
import type {
  CityId,
  Hex,
  Order,
  OrderError,
  PlayerView,
  ProductionItem,
  ResearchView,
  TechId,
  UnitId,
  UnitView,
} from './types';
import { RULES, TECH_NAME, UNIT_NAME, itemName, sameItem } from './data';
import { hexEq, toOffset } from './hex';
import { diplomaticCost, offerLabel, teamName, treatyLabel } from './diplomacy';
import { ownerName } from './barbarians';
import { empireOfPlayer, placeName } from './empires';
import { withArticle, withThe } from './resources';
import { aircraftNeedingOrders, type Hold } from './airWing';
import { isKnownOrderKind } from './orderDecoding';
import { peaceScope, peaceScopeForAction } from './peaceScope';

export const UNIT_ACTIONS = new Set<Order['type']>([
  'SetAutomation',
  'SetAutoFire',
  'MoveUnit',
  'MoveUnitPath',
  'SleepUnit',
  'SkipUnit',
  'FoundCity',
  'DisbandUnit',
  'RangedAttack',
  'UpgradeUnit',
  'UpgradeSiege',
]);

/** Orders of an aircraft in a hangar (core `rules::air`): one per plane per turn. */
export const AIR_ORDERS = new Set<Order['type']>(['AirStrike', 'AirRecon', 'Rebase', 'UpgradeAircraft']);
const orderPlane = (o: Order): UnitId | null => ('plane' in o ? o.plane : null);

export function setAirOrder(orders: Order[], order: Order): Order[] {
  const plane = orderPlane(order);
  if (plane === null || !AIR_ORDERS.has(order.type)) throw new Error(`${order.type} is not an air order`);
  return [...orders.filter((o) => orderPlane(o) !== plane), order];
}

export function clearAirOrder(orders: Order[], plane: UnitId): Order[] {
  return orders.filter((o) => orderPlane(o) !== plane);
}

export function pendingAirOrder(orders: Order[], plane: UnitId): Order | undefined {
  return orders.find((o) => AIR_ORDERS.has(o.type) && orderPlane(o) === plane);
}

export function orderUnit(o: Order): UnitId | null {
  return 'unit' in o && o.type !== 'HireCompany' ? o.unit : null;
}

export function orderCity(o: Order): CityId | null {
  return 'city' in o ? o.city : null;
}

/** Replace any unit action for the same unit (a unit gets one action per turn). */
export function setUnitOrder(orders: Order[], order: Order): Order[] {
  const unit = orderUnit(order);
  if (unit === null || !UNIT_ACTIONS.has(order.type)) throw new Error(`${order.type} is not a unit action`);
  return [...orders.filter((o) => !(UNIT_ACTIONS.has(o.type) && orderUnit(o) === unit)), order];
}

export function clearUnitOrder(orders: Order[], unit: UnitId): Order[] {
  return orders.filter((o) => !(UNIT_ACTIONS.has(o.type) && orderUnit(o) === unit));
}

export function pendingUnitOrder(orders: Order[], unit: UnitId): Order | undefined {
  return orders.find((o) => UNIT_ACTIONS.has(o.type) && orderUnit(o) === unit);
}

/** A direct replacement keeps the purchase independent of the construction choice. */
export function setProduction(orders: Order[], city: CityId, item: ProductionItem, current: ProductionItem | null): Order[] {
  const rest = orders.filter((o) => !((o.type === 'SetProduction' || o.type === 'SetProductionQueue') && o.city === city));
  return sameItem(item, current) ? rest : [...rest, { type: 'SetProduction', city, item }];
}

export function setProductionQueue(orders: Order[], city: CityId, queue: ProductionItem[]): Order[] {
  const rest = orders.filter((o) => !((o.type === 'SetProduction' || o.type === 'SetProductionQueue') && o.city === city));
  const buy = rest.findIndex((o) => isPurchase(o, city));
  const index = buy < 0 ? rest.length : buy;
  return [...rest.slice(0, index), { type: 'SetProductionQueue', city, queue }, ...rest.slice(index)];
}

export function productionQueue(orders: Order[], city: CityId, current: ProductionItem | null, tail: ProductionItem[] = []): ProductionItem[] {
  const pending = orders.find((o) => (o.type === 'SetProduction' || o.type === 'SetProductionQueue') && o.city === city);
  if (pending?.type === 'SetProductionQueue') return pending.queue;
  if (pending?.type === 'SetProduction') return [pending.item];
  return [...(current ? [current] : []), ...tail];
}

/**
 * The queue after "Build": the item goes first and replaces the current build (whose progress stays banked with it). A
 * waiting queue behind an idle city keeps its other items. A building is never queued twice.
 */
export function queueAfterBuild(queue: ProductionItem[], producing: ProductionItem | null, item: ProductionItem): ProductionItem[] {
  const rest = queue.slice(producing || sameItem(item, queue[0]) ? 1 : 0);
  return [item, ...rest.filter((queued) => (item.kind !== 'Building' && item.kind !== 'Improvement') || !sameItem(item, queued))];
}

/** Retained, blocked queue entries are not the active build. Pending choices override it. */
export function effectiveProduction(orders: Order[], city: CityId, current: ProductionItem | null): ProductionItem | null {
  return productionQueue(orders, city, current)[0] ?? null;
}

export function pendingProduction(orders: Order[], city: CityId): ProductionItem | null {
  const o = orders.find((x) => (x.type === 'SetProduction' || x.type === 'SetProductionQueue') && x.city === city);
  return o?.type === 'SetProduction' ? o.item : o?.type === 'SetProductionQueue' ? o.queue[0] ?? null : null;
}

/** A city has one purchase a turn: a Buy, a tile, or a hire (Byzantium's Mercenary Company). */
export const isPurchase = (o: Order, city: CityId) => (o.type === 'BuyItem' || o.type === 'HireCompany' || o.type === 'BuyTile') && o.city === city;

export function toggleBuy(orders: Order[], city: CityId, item: ProductionItem): Order[] {
  const buy = orders.find((o) => isPurchase(o, city));
  const rest = orders.filter((o) => !isPurchase(o, city));
  return buy?.type === 'BuyItem' && sameItem(buy.item, item) ? rest : [...rest, { type: 'BuyItem', city, item }];
}

/** Hire (or, clicked again, cancel) one Company in `city`: it replaces any other purchase there, as a second Buy does. */
export function toggleHire(orders: Order[], hire: Extract<Order, { type: 'HireCompany' }>): Order[] {
  const same = orders.some((o) => o.type === 'HireCompany' && o.city === hire.city && o.unit === hire.unit && o.seller === hire.seller);
  const rest = orders.filter((o) => !isPurchase(o, hire.city));
  return same ? rest : [...rest, hire];
}

export type TileOrder = Extract<Order, { type: 'BuyTile' }>;

/** The tile `city` is queued to buy, if any. */
export const pendingTile = (orders: Order[], city: CityId) => orders.find((o): o is TileOrder => o.type === 'BuyTile' && o.city === city);

/** Buy (or, clicked again, cancel) one tile for `city`: it replaces any other purchase there, as a second Buy does. */
export function toggleBuyTile(orders: Order[], city: CityId, tile: Hex): Order[] {
  const same = pendingTile(orders, city);
  const rest = orders.filter((o) => !isPurchase(o, city));
  return same && hexEq(same.tile, tile) ? rest : [...rest, { type: 'BuyTile', city, tile }];
}

export function hasBuy(orders: Order[], city: CityId): boolean {
  return orders.some((o) => isPurchase(o, city));
}

export function setResearch(orders: Order[], tech: TechId, current: TechId | null): Order[] {
  const rest = orders.filter((o) => o.type !== 'SetResearch' && o.type !== 'SetResearchQueue');
  return tech === current ? rest : [...rest, { type: 'SetResearch', tech }];
}

export function pendingResearch(orders: Order[]): TechId | null {
  return pendingResearchQueue(orders)?.[0] ?? null;
}

/** null means no edit; [] is an explicit clear, never a fallback to current. */
export function pendingResearchQueue(orders: Order[]): TechId[] | null {
  const o = orders.findLast((x) => x.type === 'SetResearch' || x.type === 'SetResearchQueue');
  return o?.type === 'SetResearchQueue' ? o.techs : o?.type === 'SetResearch' ? [o.tech] : null;
}

export function effectiveResearchQueue(orders: Order[], research: ResearchView): TechId[] {
  return pendingResearchQueue(orders) ?? (research.queue?.length ? research.queue : research.current ? [research.current] : []);
}

export function setResearchQueue(orders: Order[], techs: TechId[], research: ResearchView): Order[] {
  const rest = orders.filter((o) => o.type !== 'SetResearch' && o.type !== 'SetResearchQueue');
  const current = effectiveResearchQueue([], research);
  return current.length === techs.length && current.every((t, i) => t === techs[i]) ? rest : [...rest, { type: 'SetResearchQueue', techs }];
}

export function removeOrderAt(orders: Order[], index: number): Order[] {
  return orders.filter((_, i) => i !== index);
}

/** Resting defence never suspends the cycle; only explicit standing orders do. */
export function unitsNeedingOrders(view: PlayerView, orders: Order[]): UnitView[] {
  return view.units
    .filter((u) => {
      if (u.owner !== view.me) return false;
      const pending = pendingUnitOrder(orders, u.id);
      if (pending) return pending.type === 'SetAutomation' && !pending.automation && !pending.path?.length;
      return u.standing_order === 'AutoFire' ? false : u.needs_orders || (!u.automation && !u.goto && !u.standing_order);
    })
    .sort((a, b) => a.id - b.id);
}

/** The next unit needing orders after `current` (wrapping), or null when none remain. */
export function nextUnitNeedingOrders(
  view: PlayerView,
  orders: Order[],
  current: UnitId | null,
  held?: Hold,
): { kind: 'unit' | 'air'; id: number } | null {
  const list = [
    ...unitsNeedingOrders(view, orders).map((u) => ({ kind: 'unit' as const, id: u.id })),
    ...aircraftNeedingOrders(view, orders, held).map(({ plane }) => ({ kind: 'air' as const, id: plane.id })),
  ].sort((a, b) => a.id - b.id);
  if (!list.length) return null;
  if (current === null) return list[0];
  return list.find((u) => u.id > current) ?? list[0];
}

/** Errors that belong to orders matching `pred`, with the order they refer to. */
export function errorsWhere(orders: Order[], errors: OrderError[], pred: (o: Order) => boolean): string[] {
  return errors.filter((e) => orders[e.index] && pred(orders[e.index])).map((e) => e.message);
}

export function coord(h: { q: number; r: number }): string {
  const [c, r] = toOffset(h);
  return `${c},${r}`;
}

/** One sentence for a Fortify, Wait or Heal order, pending or standing: the unit card, the order list and the tile card all use it. */
export function sleepOrderText(unit: { military?: boolean; kind: string } | undefined, untilHealed: boolean): string {
  if (untilHealed) return 'Healing until full health, then asks for orders';
  return unit?.military && unit.kind !== 'Scout' ? 'Fortified until woken' : 'Waiting until woken';
}

/** A short human description of an order, for the pending orders list. */
export function describeOrder(o: Order, view: PlayerView): string {
  // A corrupted or newer server order may reach a row before the next decoded snapshot.
  if (!o || !isKnownOrderKind(o.type)) return `Unknown order (${String(o?.type ?? 'missing kind')})`;
  const faction = (owner: number | undefined) => view.players.find((p) => p.id === owner)?.faction;
  const cityFaction = (id: CityId) => faction(view.cities.find((c) => c.id === id)?.owner);
  const unit = (id: UnitId) => {
    const u = view.units.find((u) => u.id === id);
    return u ? itemName({ kind: 'Unit', unit: u.kind }, faction(u.owner)) : `Unit ${id}`;
  };
  const city = (id: CityId) => view.cities.find((c) => c.id === id)?.name ?? `City ${id}`;
  const plane = (id: UnitId) => {
    const a = view.aircraft?.find((a) => a.id === id);
    return a ? itemName({ kind: 'Unit', unit: a.kind }, faction(a.owner)) : `Aircraft ${id}`;
  };
  // A target is named (its city, or whose unit), never a raw coordinate unless nothing is known there.
  const target = (at: Hex) => {
    const c = view.cities.find((c) => hexEq(c.pos, at));
    if (c) return c.name;
    const u = view.units.find((u) => hexEq(u.pos, at) && u.owner !== view.me);
    if (u) return `${ownerName(view, u.owner)} ${itemName({ kind: 'Unit', unit: u.kind }, faction(u.owner))}`;
    return placeName(view, at) ?? coord(at);
  };
  const price = diplomaticCost(view, o);
  const cost = !price ? '' : ` · ${price} Gold`;
  const call = o.type === 'RespondToProposal' ? view.diplomacy?.calls?.find((c) => c.id === o.proposal) : undefined;
  const proposal = 'proposal' in o ? view.diplomacy?.proposals.find((p) => p.id === o.proposal) : undefined;
  const kind = 'proposal' in o ? (proposal ? offerLabel(proposal) : treatyLabel(undefined)) : '';
  // An offer is named by the other empire, not by its internal id.
  const offer = (dir: 'from' | 'to') => (proposal ? `offer ${dir} ${teamName(view, dir === 'from' ? proposal.from : proposal.to)}` : 'offer');
  switch (o.type) {
    case 'BreakTreaty': return `Leave the Alliance with ${teamName(view, o.with)}${cost} · ends turn ${view.turn + 1}`;
    case 'DeclareWar': {
      const allied = view.diplomacy?.relations.some((r) => r.state === 'Alliance' && [r.a, r.b].includes(view.team) && [r.a, r.b].includes(o.against));
      return `Declare War on ${allied ? 'your ally ' : ''}${teamName(view, o.against)}${allied ? `${cost} · betrayal` : ''} · begins turn ${view.turn + 1}`;
    }
    case 'ProposeTreaty': {
      const action = view.diplomacy.actions.find((a) => a.order.type === 'ProposeTreaty' && a.order.to === o.to && a.order.kind === o.kind);
      const scope = action && peaceScopeForAction(view, action);
      const label = scope ? [`Offer ${scope.title}.`, scope.reason].filter(Boolean).join(' ')
        : `Offer ${o.kind} to ${teamName(view, o.to)}`;
      return `${label}${cost} on activation · arrives turn ${view.turn + 1}`;
    }
    case 'RespondToProposal':
      if (call) return o.accept ? `Join ${teamName(view, call.ally)}'s war on ${teamName(view, call.target)} · at War from turn ${view.turn + 1}`
        : `Decline ${teamName(view, call.ally)}'s call to arms against ${teamName(view, call.target)} · the Alliance stands`;
      if (proposal && proposal.kind !== 'Alliance') {
        const scope = peaceScope(view, proposal);
        const label = o.accept ? `Accept ${scope.subject}; when both teams agree, it ${scope.ending}.`
          : `Decline ${scope.subject}; your wars continue.`;
        return `${[label, scope.reason].filter(Boolean).join(' ')}${cost} · resolves turn ${view.turn + 1}`;
      }
      return `${o.accept ? 'Accept' : 'Decline'} ${kind} ${offer('from')}${cost} · resolves turn ${view.turn + 1}`;
    case 'WithdrawProposal': return `Withdraw ${kind} ${offer('to')} · resolves turn ${view.turn + 1}`;
    case 'SetAutomation':
      return `${unit(o.unit)}: ${o.automation ? 'auto-explore (automated)' : 'stop auto-explore'}${o.path?.length ? ` → ${coord(o.path[o.path.length - 1])}` : ''}`;
    case 'SetAutoFire': return `${unit(o.unit)}: ${o.enabled ? 'auto-fire from city' : 'stop auto-fire'}`;
    case 'MoveUnitPath': case 'MoveUnit':
      return `${unit(o.unit)}: move to ${view.cities.find((c) => hexEq(c.pos, o.to))?.name ?? placeName(view, o.to) ?? coord(o.to)}`;
    case 'SleepUnit': {
      const subject = view.units.find((u) => u.id === o.unit);
      const text = sleepOrderText(subject, !!o.until_healed);
      return `${unit(o.unit)}: ${text.charAt(0).toLowerCase()}${text.slice(1)}`;
    }
    case 'SkipUnit':
      return `${unit(o.unit)}: skip this turn`;
    case 'FoundCity':
      return `${unit(o.unit)}: found a city`;
    case 'DisbandUnit':
      return `${unit(o.unit)}: disband`;
    case 'RangedAttack':
      return `${unit(o.unit)}: ranged attack on ${target(o.target)}`;
    case 'UpgradeUnit':
      return `${unit(o.unit)}: upgrade`;
    case 'UpgradeSiege':
      return `${unit(o.unit)}: buy Siege`;
    case 'AirStrike': return `${plane(o.plane)}: air strike on ${target(o.target)}`;
    case 'AirRecon': return `${plane(o.plane)}: recon of ${target(o.target)}`;
    case 'Rebase': return `${plane(o.plane)}: rebase to ${city(o.to)}`;
    case 'UpgradeAircraft': {
      const a = view.aircraft?.find((p) => p.id === o.plane);
      const to = a?.upgrade_options?.find((u) => (o.to ? u.via === o.to : true))?.to;
      return `${plane(o.plane)}: upgrade${to ? ` to ${UNIT_NAME[to]}` : ''}`;
    }
    case 'SetProduction':
      return `${city(o.city)}: build ${itemName(o.item, cityFaction(o.city))}`;
    case 'SetCityAutomation':
      return `${city(o.city)}: ${o.focus ? `build automatically: ${o.focus}${o.picked ? ` · chose ${itemName(o.picked, cityFaction(o.city))}` : ''}` : 'stop building automatically'}`;
    case 'SetProductionQueue':
      return `${city(o.city)}: ${o.queue.length ? o.queue.map((item, i) => `${i + 1}. ${itemName(item, cityFaction(o.city))}`).join(' → ') : 'clear production queue'}`;
    case 'BuyItem':
      return `${city(o.city)}: buy ${itemName(o.item, cityFaction(o.city))}`;
    case 'HireCompany': {
      const price = view.company_offers?.find((x) => x.seller === o.seller && x.unit === o.unit)?.price;
      return `${city(o.city)}: hire ${withArticle(itemName({ kind: 'Unit', unit: o.unit }, faction(o.seller)))} Company from ${empireOfPlayer(view, o.seller)}${price === undefined ? '' : ` for ${price} Gold`} (${RULES.company.turns} turns)`;
    }
    case 'BuyTile': {
      const tilePrice = view.cities.find((c) => c.id === o.city)?.detail?.buyable_tiles.find((b) => hexEq(b.tile, o.tile))?.price;
      return `${city(o.city)}: buy the tile at ${coord(o.tile)}${tilePrice === undefined ? '' : ` for ${tilePrice} Gold`}`;
    }
    case 'SetResearch':
      return `Research ${TECH_NAME[o.tech]}`;
    case 'SetResearchQueue':
      return o.techs.length ? `Research ${o.techs.map((t) => TECH_NAME[t]).join(' → ')}` : 'Clear research queue';
    case 'SetScienceConversion': return `Convert Science to ${o.to}`;
    case 'GiveGold': return `Give ${o.amount} Gold to ${withThe(empireOfPlayer(view, o.to))}`;
    case 'Action':
      switch (o.action.action) {
        case 'Swarm': return `Kindle the New Fire: offer ${view.actions?.swarm?.population ?? 50} population · +2 VP, permanent`;
        case 'Sakoku': return o.action.close
          ? 'Close the country (Sakoku): no war, no Alliances, no tech diffusion · +2 VP after 30 turns closed'
          : 'Reopen the country: the Sakoku count and its VP are lost';
        case 'Broadcast': return `Share the Learning: every rival gains ${view.actions?.broadcast?.techs_each ?? 3} free technologies of its choice · +2 VP, permanent`;
        case 'RetireVeteran': return `${unit(o.action.unit)}: retire with honour · ${(view.actions?.retire?.retired ?? 0) + 1} of ${view.actions?.retire?.target ?? 3} toward Honour the Veterans`;
        case 'CallTheWild': return `Raise the Warbands: a warband camp in the territory of ${view.actions?.wild?.sites.length ?? 'every'} ${view.actions?.wild?.sites.length === 1 ? 'city' : 'cities'}`;
        case 'ChooseFreeTechs': return `Take free ${o.action.techs.length === 1 ? 'technology' : 'technologies'}: ${o.action.techs.map((t) => TECH_NAME[t]).join(', ')}`;
      }
  }
  return `Unknown order (${String((o as { type?: unknown }).type ?? 'missing kind')})`;
}
