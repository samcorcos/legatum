// Byzantium's Buy a city and the fall of Constantinople (core `buy_city`, rules 65): the words and
// numbers every surface shares (city panel, tile card, banner tooltip and chip, turn report, advisor).
// The core owns every rule and number; this file only reads `PlayerView.city_buys`, `city_offers` and
// `byzantine_capitals` and says them in the player's terms.
import { RULES, isWallsLine } from './data';
import { empireOfPlayer } from './empires';
import type { BidView, Breakdown, CityBuyView, CityOfferView, CityView, CapitalFallView, Order, PlayerView } from './types';

/** Core `buy_city` constants (exported with the rules). */
export const BUY_CITY = RULES.buyCity;
/** Turn ends between an offer and the sale. */
export const OFFER_TURNS: number = BUY_CITY.offerTurns;
/** Percent of the held Gold lost when the owner raises the price above it. */
export const FAIL_FEE_PERCENT: number = BUY_CITY.failFeePercent;

type Seats = Pick<PlayerView, 'players'>;
const turns = (n: number) => `${n} turn${n === 1 ? '' : 's'}`;

/** Core reasons name players by seat ("You are at War with AI 2"): name them by their empire, as everywhere else. */
export function renameSeats(view: Seats, text: string): string {
  return [...view.players].sort((a, b) => b.name.length - a.name.length).reduce((out, p) => {
    const escaped = p.name.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    return out.replace(new RegExp(`(^|[^\\w])${escaped}(?![\\w])`, 'g'), (_, before: string) => `${before}${empireOfPlayer(view, p.id)}`);
  }, text);
}

/** The viewer's own price for a city, as Byzantium (undefined for any other viewer or city). */
export function myCityBuy(view: Pick<PlayerView, 'me'> & Partial<Pick<PlayerView, 'city_buys'>>, city: number): CityBuyView | undefined {
  return view.city_buys?.find((b) => b.city === city && b.buyer === view.me);
}

/** What a Byzantium would pay for a city of someone else's, as public knowledge: the first Byzantium that can buy it now. */
export function publicCityBuy(view: Pick<PlayerView, 'me'> & Partial<Pick<PlayerView, 'city_buys'>>, city: number): CityBuyView | undefined {
  return view.city_buys?.find((b) => b.city === city && b.buyer !== view.me && !b.blocker);
}

/** The offer running on a city, if the viewer knows of one. */
export function offerOn(view: Partial<Pick<PlayerView, 'city_offers'>>, city: number): CityOfferView | undefined {
  return view.city_offers?.find((o) => o.city === city);
}

/** The viewer's own running offer (Byzantium has one at a time). */
export function myOffer(view: Pick<PlayerView, 'me'> & Partial<Pick<PlayerView, 'city_offers'>>): CityOfferView | undefined {
  return view.city_offers?.find((o) => o.buyer === view.me);
}

/** The BuyCity order queued this turn, if any (one at a time). */
export const queuedBuy = (orders: readonly Order[]): Extract<Order, { type: 'BuyCity' }> | undefined =>
  orders.find((o): o is Extract<Order, { type: 'BuyCity' }> => o.type === 'BuyCity');

/** Queue an offer of `gold` (absent: the price) for `city`, replacing any other queued offer (core takes one at a time). */
export const withBuy = (orders: readonly Order[], city: number, gold?: number): Order[] =>
  [...orders.filter((o) => o.type !== 'BuyCity'), gold ? { type: 'BuyCity', city, gold } : { type: 'BuyCity', city }];

/** The Gold a queued offer holds: its bid, or the city's price when it names none. */
export function offerGold(view: Pick<PlayerView, 'me'> & Partial<Pick<PlayerView, 'city_buys'>>, o: Extract<Order, { type: 'BuyCity' }>): number {
  return o.gold || (myCityBuy(view, o.city)?.price ?? 0);
}

/** A bid choice's name: "Price", "+25%". */
export const bidLabel = (percent: number): string => (percent === 100 ? 'Price' : `+${percent - 100}%`);

/** What breaks an offer, in one run: "2 more military units or 1 Walls level". */
export function stopParts(guards: number, walls?: number | null): string {
  const units = `${guards} more military unit${guards === 1 ? '' : 's'}`;
  return walls ? `${units} or ${walls} Walls level${walls === 1 ? '' : 's'}` : units;
}

/** One line under a bid choice: "Stopped by 2 more military units or 1 Walls level". */
export const stopLine = (b: Pick<BidView, 'guards_to_stop' | 'walls_to_stop'>): string => `Stopped by ${stopParts(b.guards_to_stop, b.walls_to_stop)}`;

/** The buyer's bid choices for a city (core `CityBuyView::bids`); just the price when the view has none. */
export function bidChoices(buy: CityBuyView): BidView[] {
  return buy.bids?.length ? buy.bids : [{ percent: 100, gold: buy.price, guards_to_stop: 1 }];
}
/** Drop the queued offer. */
export const withoutBuy = (orders: readonly Order[]): Order[] => orders.filter((o) => o.type !== 'BuyCity');

/** The price's lines worth reading: the base always, the others only when they add something. */
export function priceLines(b: Breakdown): [string, number][] {
  return b.parts.filter(([, v], i) => i === 0 || v !== 0).map(([label, v]) => [label.replace(/ x (\d)/, ' × $1').replace(/: x(\d)/, ': ×$1'), v]);
}

/** One line for a hover: "275 Gold (Starting price 75, Population 4 × 25 +100, …)". */
export function priceText(b: Breakdown): string {
  const parts = priceLines(b).map(([l, v], i) => (i === 0 ? `${l} ${v}` : `${l} +${v}`));
  return `${b.total} Gold${parts.length > 1 ? ` (${parts.join(', ')})` : ''}`;
}

/** The public chip on a city's banner while an offer runs: "Being bought: 2 turns". */
export function offerChip(view: Partial<Pick<PlayerView, 'city_offers'>>, city: number): string | null {
  const o = offerOn(view, city);
  return o ? `Being bought: ${turns(o.turns_left)}` : null;
}

/** The chip's tone from the viewer's side: green for the buyer's team, red for the seller's, ink for everyone else. */
export function offerTone(view: Pick<PlayerView, 'team' | 'players'>, o: CityOfferView): 'good' | 'bad' | 'neutral' {
  const team = (id: number) => view.players.find((p) => p.id === id)?.team;
  return team(o.buyer) === view.team ? 'good' : team(o.seller) === view.team ? 'bad' : 'neutral';
}

/** "1 more military unit" / "2 more military units". */
const moreUnits = (n: number) => `${n} more military unit${n === 1 ? '' : 's'}`;
const wallsLevels = (n: number) => `${n} Walls level${n === 1 ? '' : 's'}`;

/**
 * The Walls levels that would break an offer and can be finished before it ends, or null. Only the seller's own
 * city shows its build times (its `detail`): the levels' turns, one after another, must fit the turns left.
 */
export function wallsInTime(view: Pick<PlayerView, 'me' | 'cities'>, o: CityOfferView): number | null {
  const need = o.walls_to_stop;
  if (!need) return null;
  const option = view.cities.find((c) => c.id === o.city)?.detail?.options.find((x) => x.item.kind === 'Building' && isWallsLine(x.item.building));
  return option && option.turns > 0 && option.turns * need <= o.turns_left ? need : null;
}

/** The seller's counters in one phrase: "move 1 more military unit in or next to it, build 1 Walls level, or declare War". */
export function keepPhrase(guards: number, walls: number | null): string {
  const move = guards ? `move ${moreUnits(guards)} in or next to it` : 'move military units in or next to it';
  return `${move}${walls ? `, build ${wallsLevels(walls)}` : ''}, or declare War`;
}

/** The lines about a running offer, from the viewer's side: the countdown and what holds it. */
/** `head` counts the turns left in words; `bar` leaves them to the countdown bar beside it; `tile` is the tile card's whole text. */
export function offerLines(view: Pick<PlayerView, 'me' | 'team' | 'players' | 'cities'>, o: CityOfferView): { head: string; bar: string; detail: string; tile: string } {
  const lines = offerText(view, o);
  const head = `${lines.bar} (${turns(o.turns_left)})`;
  return { ...lines, head, tile: lines.own ? lines.detail : `${head}. ${lines.detail}` };
}

function offerText(view: Pick<PlayerView, 'me' | 'team' | 'players' | 'cities'>, o: CityOfferView): { bar: string; detail: string; own: boolean } {
  const city = view.cities.find((c) => c.id === o.city)?.name ?? 'this city';
  const when = `at the end of turn ${o.ends_turn}`;
  const breaking = o.price_now > o.escrow;
  const fee = Math.floor(o.escrow * FAIL_FEE_PERCENT / 100);
  if (o.buyer === view.me) {
    const walls = o.walls_to_stop ? `, or ${wallsLevels(o.walls_to_stop)} ${o.walls_to_stop === 1 ? 'is' : 'are'} built` : '';
    const g = o.guards_to_stop ?? 0;
    return {
      bar: `Buying ${city}: yours ${when}`,
      own: true,
      detail: breaking
        ? `Its price rose to ${o.price_now} Gold, above your ${o.escrow} Gold offer: the offer fails at the end of this turn and you lose ${fee} Gold.`
        : `You pay ${o.escrow} Gold for ${city} ${when}. It fails (you lose ${fee} Gold) if ${g ? `${moreUnits(g)} come${g === 1 ? 's' : ''} in or next to it${walls}` : 'its price rises above your offer'}.`,
    };
  }
  const buyer = empireOfPlayer(view, o.buyer);
  if (o.seller === view.me) {
    return {
      bar: `${buyer} is buying ${city}: it changes hands ${when}`,
      own: true,
      detail: breaking
        ? `You raised its price to ${o.price_now} Gold, above ${buyer}'s ${o.escrow} Gold offer: the offer fails at the end of this turn.`
        : `${buyer} pays you ${o.escrow} Gold for ${city} ${when}. To keep it, ${keepPhrase(o.guards_to_stop ?? 0, wallsInTime(view, o))}.`,
    };
  }
  return {
    bar: `${buyer} is buying ${city} from ${empireOfPlayer(view, o.seller)}: it changes hands ${when}`,
    own: false,
    detail: `${buyer} offered ${o.escrow} Gold. If its price rises above that (now ${o.price_now}), the offer fails.`,
  };
}

/** The fall of Constantinople, for a city that is a Byzantine capital or the Heir's refuge. */
export function capitalFallOf(view: Partial<Pick<PlayerView, 'byzantine_capitals'>>, city: number): { fall: CapitalFallView; role: 'capital' | 'refuge' } | undefined {
  for (const fall of view.byzantine_capitals ?? []) {
    if (fall.capital === city) return { fall, role: 'capital' };
    if (fall.refuge === city) return { fall, role: 'refuge' };
  }
  return undefined;
}

/** The public line on a Byzantine capital or refuge, from the viewer's side. */
export function capitalFallLine(view: Pick<PlayerView, 'me' | 'players' | 'cities'> & Partial<Pick<PlayerView, 'byzantine_capitals'>>, city: CityView | number): string | null {
  const id = typeof city === 'number' ? city : city.id;
  const hit = capitalFallOf(view, id);
  if (!hit) return null;
  const { fall, role } = hit;
  const own = fall.player === view.me;
  const name = (c: number | null | undefined) => view.cities.find((x) => x.id === c)?.name ?? 'another city';
  const of = empireOfPlayer(view, fall.player);
  if (role === 'refuge') {
    return own
      ? `Your Heir shelters here: if ${name(fall.capital)} falls, this city becomes your capital and nothing else falls.`
      : `${of}'s Heir shelters here: if ${name(fall.capital)} falls, this city becomes the capital and nothing else falls.`;
  }
  if (fall.refuge != null) {
    return own
      ? `Your capital. Your Heir shelters in ${name(fall.refuge)}: if this city falls, only this city falls.`
      : `${of}'s capital. Its Heir shelters in ${name(fall.refuge)}: take this city and only this city falls.`;
  }
  return own
    ? 'Lose this city and you lose every city, unless your Heir is in another of your cities.'
    : 'Take it and you take every Byzantine city.';
}

/**
 * Why Byzantium can't buy a city now, as one line: at War it names the way out ("Make Peace with Sparta to buy it", the
 * War not repeated); every other core reason is said with the empires' names.
 */
export function buyBlocker(view: Pick<PlayerView, 'players' | 'me' | 'team'>, city: Pick<CityView, 'owner'>, buy: Pick<CityBuyView, 'blocker'>): string {
  const reason = renameSeats(view, buy.blocker ?? '');
  return /at War with/.test(reason) ? `Make Peace with ${empireOfPlayer(view, city.owner)} to buy it` : reason.replace(/\.$/, '');
}

/** The tile card's lines about a city (each `key` becomes a `tile-<key>` test id): the fall of Constantinople, a running offer, and the price. */
export function cityBuyTileLines(view: PlayerView, city: CityView): { key: string; text: string; tone?: 'good' | 'bad'; icon?: string }[] {
  const out: { key: string; text: string; tone?: 'good' | 'bad'; icon?: string }[] = [];
  const fall = capitalFallLine(view, city);
  if (fall) out.push({ key: 'capital-fall', text: fall, icon: 'unit-heirtothethrone' });
  const offer = offerOn(view, city.id);
  if (offer) {
    const lines = offerLines(view, offer);
    const tone = offerTone(view, offer);
    out.push({ key: 'city-offer', text: lines.tile, icon: 'yield-gold', ...(tone === 'neutral' ? {} : { tone }) });
    return out;
  }
  const mine = myCityBuy(view, city.id);
  if (mine) {
    // A capital is never for sale: its line is the fall above, not a price.
    if (!city.is_capital && !city.original_capital) {
      out.push({ key: 'city-buy', text: mine.blocker ? `${/^Make Peace/.test(buyBlocker(view, city, mine)) ? '' : 'Not now: '}${buyBlocker(view, city, mine)}.` : `Buy for ${mine.price} Gold. Click the city to make an offer.`, icon: 'yield-gold' });
    }
    return out;
  }
  const pub = publicCityBuy(view, city.id);
  if (pub) {
    const buyer = empireOfPlayer(view, pub.buyer);
    out.push({ key: 'city-buy', text: city.owner === view.me
      ? `${buyer} can buy this city for ${pub.price} Gold. Military units and Walls raise the price.`
      : `${buyer} can buy it for ${pub.price} Gold.`, icon: 'yield-gold' });
  }
  return out;
}
