import type { GameClient } from '../client';
import { itemKey, moveCost, sameItem, SIEGE_USES_IRON, UNIT_STATS } from '../data';
import { canEnter } from '../diplomacy';
import { hexEq, neighbors, tileIndex } from '../hex';
import { isPurchase, orderUnit, pendingTile } from '../orders';
import { tilePriceTags } from '../map/buyTilePrices';
import { pendingHire } from '../company';
import type { CityView, Order, OrderError, PlayerView, ProductionItem, ProductionOption, Resource } from '../types';
import { unitNeed } from '../resources';

/** The refusal every other row shares once this city has a purchase pending. */
export const ONE_PURCHASE = 'One purchase per city per turn; cancel the current purchase first';
const SPAWN_BLOCKER = 'No free spawn tile in or beside the city; move a blocking unit first';

function validDisbands(view: PlayerView, orders: Order[], errors: OrderError[]): Set<number> {
  const disbanded = new Set<number>();
  if (!view.players.find((p) => p.id === view.me)?.alive) return disbanded;
  const acted = new Set<number>();
  orders.forEach((order, index) => {
    const unit = orderUnit(order);
    if (unit === null || !view.units.some((u) => u.id === unit && u.owner === view.me) || acted.has(unit)) return;
    // Core reserves the action even when that first order fails. An empty online error
    // cache is not validation: ownership and action conflicts must be checked here.
    acted.add(unit);
    if (order.type === 'DisbandUnit' && !errors.some((e) => e.index === index)) disbanded.add(unit);
  });
  return disbanded;
}

function disbandFreesSpawn(city: CityView, view: PlayerView, item: ProductionItem, disbanded: Set<number>): boolean {
  if (item.kind !== 'Unit' || !disbanded.size) return false;
  const military = UNIT_STATS[item.unit].military;
  const friendly = (owner: number) => view.players.find((p) => p.id === owner)?.team === view.team;
  // Match core can_spawn_at, using only visible occupancy.
  // Only a tile actually freed by a valid disband can invalidate the snapshot reason.
  return [city.pos, ...neighbors(city.pos)].some((pos) => {
    const tile = view.tiles[tileIndex(pos, view.width, view.height)];
    if (!tile?.visible || moveCost(tile.terrain) === null || !canEnter(view, item.unit, tile.owner)) return false;
    const units = view.units.filter((u) => hexEq(u.pos, pos));
    return units.some((u) => disbanded.has(u.id))
      && !view.cities.some((c) => hexEq(c.pos, pos) && !friendly(c.owner))
      && units.every((u) => disbanded.has(u.id) || (friendly(u.owner) && u.military !== military));
  });
}

/** One row of Byzantium's shop: its price, why it cannot be hired now (if so), whether it is the pending hire and the treasury it is paid from. */
export interface HirePreview { price: number; reason: string | null; selected: boolean; gold: number }

const earlierSettlers = (orders: Order[], errors: OrderError[], before: number) => orders.slice(0, before)
  .filter((o, i) => o.type === 'BuyItem' && o.item.kind === 'Unit' && o.item.unit === 'Settler' && !errors.some((e) => e.index === i)).length;
const quotedBuyCost = (option: ProductionOption, orders: Order[], errors: OrderError[], before: number) => {
  const quote = option.buy_quote;
  return quote ? Math.floor((quote.production.total + quote.additional_settler_production * earlierSettlers(orders, errors, before)) * quote.gold_percent / 100) : option.buy_cost;
};

type SpendOrder = Extract<Order, { type: 'UpgradeUnit' | 'UpgradeSiege' | 'UpgradeAircraft' | 'BuyItem' | 'HireCompany' | 'BuyTile' }>;
const isSpend = (order: Order): order is SpendOrder => ['UpgradeUnit', 'UpgradeSiege', 'UpgradeAircraft', 'BuyItem', 'HireCompany', 'BuyTile'].includes(order.type);
const phase = (order: SpendOrder) => order.type === 'UpgradeUnit' || order.type === 'UpgradeSiege' ? 0
  : order.type === 'BuyItem' || order.type === 'HireCompany' || order.type === 'BuyTile' ? 1 : 2;

/** "you have N", or, when queued orders already reserve part of the treasury, "N left after your queued orders". */
export const goldOnHand = (gold: number, view: PlayerView) => gold === view.economy.gold ? `you have ${gold}` : `${gold} left after your queued orders`;

/** Core `rules/mod.rs::resolve_turn`: unit orders (including Siege and land upgrades),
 * `cities::apply_city_orders` in list order, then `air::resolve_ranged_phase` upgrades by plane id.
 * Prices come only from the core view's quotes, never from copied game rules. */
export function pendingSpend(view: PlayerView, orders: Order[], errors: OrderError[]) {
  const rejected = new Set(errors.map((error) => error.index));
  const quote = (order: SpendOrder, index: number): { gold: number; supply: Resource | null } => {
    if (order.type === 'UpgradeUnit') {
      const unit = view.units.find((u) => u.id === order.unit);
      return { gold: unit?.upgrade_breakdown?.total ?? unit?.upgrade_cost ?? 0, supply: unit?.upgrade_need ?? null };
    }
    if (order.type === 'UpgradeSiege') return { gold: view.units.find((u) => u.id === order.unit)?.siege_offer?.price.total ?? 0, supply: SIEGE_USES_IRON ? 'Iron' : null };
    if (order.type === 'UpgradeAircraft') {
      const plane = view.aircraft?.find((a) => a.id === order.plane);
      const option = plane?.upgrade_options?.find((o) => (order.to ? o.via === order.to : true));
      return { gold: option?.cost ?? plane?.upgrade_cost ?? 0, supply: option && plane && unitNeed(option.to) !== unitNeed(plane.kind) ? unitNeed(option.to) : null };
    }
    if (order.type === 'HireCompany') return { gold: view.company_offers?.find((o) => o.seller === order.seller && o.unit === order.unit)?.price ?? 0, supply: null };
    if (order.type === 'BuyTile') return { gold: view.cities.find((c) => c.id === order.city)?.detail?.buyable_tiles.find((b) => hexEq(b.tile, order.tile))?.price ?? 0, supply: null };
    const option = view.cities.find((c) => c.id === order.city)?.detail?.options.find((o) => sameItem(o.item, order.item));
    if (!option) return { gold: 0, supply: null };
    return { gold: quotedBuyCost(option, orders, errors, index) ?? 0, supply: option.supply_need ?? null };
  };
  const rank = (order: SpendOrder, index: number) => [phase(order), order.type === 'UpgradeAircraft' ? order.plane : index];
  const earlier = (a: SpendOrder, ai: number, b: SpendOrder, bi: number) => {
    const [ap, an] = rank(a, ai), [bp, bn] = rank(b, bi);
    return ap < bp || (ap === bp && an < bn);
  };
  const quoted = orders.flatMap((order, index) => isSpend(order) && !rejected.has(index) ? [{ order, index, ...quote(order, index) }] : []);
  quoted.sort((a, b) => { const [ap, an] = rank(a.order, a.index), [bp, bn] = rank(b.order, b.index); return ap - bp || an - bn; });
  let remaining = view.economy.gold;
  // Air validation checks the starting treasury; at resolution it runs after city spending.
  // A later unaffordable order therefore reserves neither Gold nor supply.
  const entries = quoted.filter((entry) => { if (entry.gold > remaining) return false; remaining -= entry.gold; return true; });
  const before = (order: SpendOrder, index = orders.length) => {
    const preceding = entries.filter((entry) => earlier(entry.order, entry.index, order, index));
    return { gold: view.economy.gold - preceding.reduce((n, entry) => n + entry.gold, 0),
      supply: preceding.reduce((map, entry) => { if (entry.supply) map.set(entry.supply, (map.get(entry.supply) ?? 0) + 1); return map; }, new Map<Resource, number>()) };
  };
  const gold = remaining;
  const supply = entries.reduce((map, entry) => { if (entry.supply) map.set(entry.supply, (map.get(entry.supply) ?? 0) + 1); return map; }, new Map<Resource, number>());
  /** Spendable if this action is added now; an already selected action gets its own reservation back. */
  const available = (order: SpendOrder, index = orders.length) => {
    const own = entries.find((entry) => entry.index === index && entry.order.type === order.type);
    const held = new Map(supply);
    if (own?.supply) held.set(own.supply, (held.get(own.supply) ?? 0) - 1);
    return { gold: gold + (own?.gold ?? 0), supply: held };
  };
  return { gold, supply, entries, before, available };
}

/** Recomputed only when the view or pending orders change, never by tooltip/UI renders. */
export function purchasePreviews(client: GameClient | null, city: CityView, view: PlayerView, orders: Order[], errors: OrderError[]) {
  const purchase = orders.find((o) => isPurchase(o, city.id));
  const buying = purchase?.type === 'BuyItem' ? purchase : undefined;
  const local = client?.kind === 'remote' ? null : client;
  const pendingErrors = local?.validate(view.me, orders) ?? errors;
  const disbanded = client?.kind === 'remote' ? validDisbands(view, orders, pendingErrors) : new Set<number>();
  const price = (option: ProductionOption, before: number) => quotedBuyCost(option, orders, pendingErrors, before);
  const spend = pendingSpend(view, orders, pendingErrors);
  const options = new Map((city.detail?.options ?? []).map((o) => {
    const selected = buying?.type === 'BuyItem' && sameItem(buying.item, o.item);
    const candidate: Order[] = selected ? orders : [...orders.filter((ord) => !isPurchase(ord, city.id)), { type: 'BuyItem', city: city.id, item: o.item }];
    const index = selected ? orders.indexOf(buying!) : candidate.length - 1;
    const before = selected ? index : orders.length;
    const cost = price(o, before);
    const viewBlocker = o.buy_blocker?.startsWith('Not enough Gold:')
      || (o.buy_blocker === SPAWN_BLOCKER && disbandFreesSpawn(city, view, o.item, disbanded)) ? null : o.buy_blocker;
    const gold = spend.available(selected ? buying! : { type: 'BuyItem', city: city.id, item: o.item }, before).gold;
    const reason = purchase && !selected ? ONE_PURCHASE
      : cost !== null && cost > gold ? `Not enough Gold: costs ${cost} Gold; you have ${gold}`
      : local ? local.validate(view.me, candidate).find((e) => e.index === index)?.message ?? null
      : (selected ? pendingErrors.find((e) => e.index === index)?.message : null) ?? viewBlocker
        ?? (cost === null ? (o.item.kind === 'Unit' && o.item.unit === 'Heir' ? 'The Heir to the Throne cannot be bought: he is only trained' : 'World wonders cannot be bought') : cost > gold ? `Not enough Gold: costs ${cost} Gold; you have ${gold}` : null);
    const quote = o.buy_quote;
    const extra = quote ? quote.additional_settler_production * earlierSettlers(orders, pendingErrors, before) : 0;
    const breakdown = quote ? `${quote.gold_percent}% × (${quote.production.parts.map(([label, amount]) => `${amount} ${label}`).join(' + ')}${extra ? ` + ${extra} pending Settlers` : ''}) Production` : '';
    // `gold`: the treasury this purchase is paid from (after earlier queued purchases and upgrades).
    return [itemKey(o.item), { cost, reason, selected, breakdown, gold }] as const;
  }));
  // Byzantium's shop: a row per seller and kind, priced and refused by the same rules (core `company::blocker`).
  const hires = new Map<string, HirePreview>((view.company_offers ?? []).map((offer) => {
    const mine = pendingHire(orders, city.id);
    const selected = mine?.unit === offer.unit && mine.seller === offer.seller;
    const hire: Order = { type: 'HireCompany', city: city.id, unit: offer.unit, seller: offer.seller };
    const candidate: Order[] = selected ? orders : [...orders.filter((ord) => !isPurchase(ord, city.id)), hire];
    const index = selected ? orders.indexOf(mine!) : candidate.length - 1;
    const gold = spend.available(selected ? mine! : hire, selected ? index : orders.length).gold;
    // What the view knows about this city's purchase: a unit already bought here blocks the day; a full ring of tiles blocks the spawn.
    const block = (city.detail?.options ?? []).find((o) => o.item.kind === 'Unit' && (o.buy_block === 'already_bought' || o.buy_block === 'resistance' || (o.buy_block === 'no_spawn_tile' && o.item.unit === offer.unit)))?.buy_blocker ?? null;
    const reason = offer.unavailable ? offer.unavailable
      : purchase && !selected ? ONE_PURCHASE
      : offer.price > gold ? `Not enough Gold: the company costs ${offer.price} Gold; you have ${gold} (need ${offer.price - gold} more)`
      : local ? local.validate(view.me, candidate).find((e) => e.index === index)?.message ?? null
      : (selected ? pendingErrors.find((e) => e.index === index)?.message : null) ?? block
        ?? (offer.price > gold ? `Not enough Gold: the company costs ${offer.price} Gold; you have ${gold} (need ${offer.price - gold} more)` : null);
    return [`${offer.seller}:${offer.unit}`, { price: offer.price, reason, selected, gold } as HirePreview] as const;
  }));
  return { options, hires, gold: spend.gold, buying, hiring: pendingHire(orders, city.id) };
}

/**
 * The map's price tags and the city's "Buy a tile" row: each buyable tile with its price, the queued tile, and why this city
 * can buy none. A tag is payable from the treasury less every other queued spend; the city's own queued tile would be
 * replaced, so its Gold counts as free. An item or hire queued here blocks the control instead (`other`).
 */
export function tilePurchasePreview(city: CityView, view: PlayerView, orders: Order[], errors: OrderError[]) {
  const d = city.detail;
  const queued = pendingTile(orders, city.id);
  const spend = pendingSpend(view, orders, errors);
  const own = spend.entries.find((entry) => isPurchase(entry.order, city.id));
  // A queued tile replaces itself, but never an item or company purchase (the control refuses until that is cancelled).
  const funds = spend.gold + (own?.order.type === 'BuyTile' ? own.gold : 0);
  const blocker = d?.buy_tile_blocker || null;
  const tags = d && !blocker ? tilePriceTags(d.buyable_tiles, funds, queued?.tile ?? null) : [];
  const price = queued ? tags.find((t) => t.queued)?.price ?? null : null;
  const other = orders.find((o) => isPurchase(o, city.id) && o.type !== 'BuyTile');
  const error = queued ? errors.find((e) => orders[e.index] === queued)?.message ?? null : null;
  const cheapest = tags.length ? Math.min(...tags.map((t) => t.price)) : null;
  const goldBefore = queued ? spend.before(queued, orders.indexOf(queued)).gold : funds;
  return { tags, queued, price, funds, goldBefore, blocker, other, error, cheapest };
}
