// Byzantium's Buy a city and the fall of Constantinople: the shared words (city panel, tile card, banner, report).
import { describe, expect, it } from 'vitest';
import { decodeOrders } from './orderDecoding';
import { bidChoices, bidLabel, capitalFallLine, cityBuyTileLines, offerChip, offerLines, priceLines, priceText, renameSeats, stopLine, withBuy, withoutBuy } from './buyCity';
import { contextFromViews, formatEvents } from './format';
import { promisedGold } from './deals';
import type { CityView, Event, PlayerView } from './types';

const players = [
  { id: 0, name: 'AI 1', faction: 'Byzantium', team: 0 },
  { id: 1, name: 'AI 2', faction: 'Sparta', team: 1 },
  { id: 2, name: 'AI 3', faction: 'India', team: 2 },
];
const cityOf = (id: number, owner: number, name: string, extra: Partial<CityView> = {}) => ({ id, owner, name, pos: { q: id, r: 0 }, is_capital: false, visible: true, population: 4, ...extra } as unknown as CityView);
const breakdown = { total: 600, parts: [['Starting price', 150], ['Population 4 x 60', 240], ['Buildings: 50% of their Gold price', 90], ['Walls level 0: +0%', 0], ['1 military unit in or next to it: +25%', 120]] as [string, number][] };
const view = (me: number, over: Partial<PlayerView> = {}) => ({
  me, team: me, turn: 12, players, economy: { gold: 1000 }, diplomacy: { relations: [], proposals: [], actions: [] },
  cities: [cityOf(1, 0, 'Constantinople', { is_capital: true }), cityOf(2, 0, 'Nicaea'), cityOf(3, 1, 'Pylos'), cityOf(4, 1, 'Sparta', { is_capital: true })],
  units: [], tiles: [], width: 1, height: 1,
  ...over,
}) as unknown as PlayerView;

describe('the price', () => {
  it('lists the lines that add something, in the player\'s figures', () => {
    expect(priceLines(breakdown)).toEqual([['Starting price', 150], ['Population 4 × 60', 240], ['Buildings: 50% of their Gold price', 90], ['1 military unit in or next to it: +25%', 120]]);
    expect(priceText(breakdown)).toBe('600 Gold (Starting price 150, Population 4 × 60 +240, Buildings: 50% of their Gold price +90, 1 military unit in or next to it: +25% +120)');
  });
  it('names players by their empire, never their seat', () => {
    expect(renameSeats(view(0), 'You are at War with AI 2')).toBe('You are at War with Sparta');
    expect(renameSeats(view(0), 'War between AI 1 and AI 2')).toBe('War between Byzantium and Sparta');
  });
  it('queues one offer at a time, decodes it from storage and counts its Gold as promised', () => {
    const orders = withBuy(withBuy([{ type: 'SkipUnit', unit: 4 }], 3), 2);
    expect(orders).toEqual([{ type: 'SkipUnit', unit: 4 }, { type: 'BuyCity', city: 2 }]);
    expect(withoutBuy(orders)).toEqual([{ type: 'SkipUnit', unit: 4 }]);
    expect(decodeOrders([{ type: 'BuyCity', city: 3 }])).toEqual([{ type: 'BuyCity', city: 3 }]);
    const v = view(0, { city_buys: [{ city: 3, buyer: 0, price: 600, breakdown }] });
    expect(promisedGold(v, [{ type: 'BuyCity', city: 3 }])).toBe(600);
  });
});

describe('a running offer', () => {
  const offer = { city: 3, buyer: 0, seller: 1, escrow: 600, ends_turn: 14, turns_left: 3, price_now: 600 };
  it('shows a public countdown chip', () => {
    expect(offerChip({ city_offers: [offer] }, 3)).toBe('Being bought: 3 turns');
    expect(offerChip({ city_offers: [{ ...offer, turns_left: 1 }] }, 3)).toBe('Being bought: 1 turn');
    expect(offerChip({ city_offers: [offer] }, 2)).toBeNull();
  });
  it('tells the buyer, the seller and everyone else what holds it', () => {
    expect(offerLines(view(0), offer).head).toBe('Buying Pylos: yours at the end of turn 14 (3 turns)');
    const seller = offerLines(view(1), offer);
    expect(seller.head).toBe('Byzantium is buying Pylos: it changes hands at the end of turn 14 (3 turns)');
    expect(seller.options).toBe('Move military units in or build Walls to raise the price, or declare War.');
    expect(offerLines(view(2), offer).head).toBe('Byzantium is buying Pylos from Sparta: it changes hands at the end of turn 14 (3 turns)');
    expect(offerLines(view(0), { ...offer, price_now: 700 }).detail).toContain('the offer fails at the end of this turn and you lose 60 Gold');
    // An overbid: the seller is told what it takes to stop it.
    expect(offerLines(view(1), { ...offer, escrow: 900, guards_to_stop: 3, walls_to_stop: 2 }).detail)
      .toBe('You get the 900 Gold Byzantium offered. If its price rises above 900 Gold (now 600), the offer fails. 3 more military units or 2 Walls levels in or next to it would do that.');
  });
  it('lists the bid choices with what stops each, and queues the chosen bid', () => {
    expect(stopLine({ guards_to_stop: 1, walls_to_stop: 1 })).toBe('Stopped by 1 more military unit or 1 Walls level');
    expect(stopLine({ guards_to_stop: 5 })).toBe('Stopped by 5 more military units');
    expect(bidLabel(100)).toBe('Price');
    expect(bidLabel(150)).toBe('+50%');
    expect(withBuy([], 3, 900)).toEqual([{ type: 'BuyCity', city: 3, gold: 900 }]);
    expect(bidChoices({ city: 3, buyer: 0, price: 600, breakdown })).toEqual([{ percent: 100, gold: 600, guards_to_stop: 1 }]);
    const v = view(0, { city_buys: [{ city: 3, buyer: 0, price: 600, breakdown }] });
    expect(promisedGold(v, [{ type: 'BuyCity', city: 3, gold: 900 }])).toBe(900);
  });
  it('puts the offer, not the price, on the tile card', () => {
    const v = view(1, { city_offers: [offer], city_buys: [{ city: 3, buyer: 0, price: 600, breakdown }] });
    const lines = cityBuyTileLines(v, v.cities[2]);
    expect(lines.map((l) => l.key)).toEqual(['city-offer', 'city-offer-options']);
    expect(lines[0].tone).toBe('bad');
  });
});

describe('the price on the tile card', () => {
  it('Byzantium reads the price and the refusal; a capital shows no price', () => {
    const v = view(0, { city_buys: [{ city: 3, buyer: 0, price: 600, breakdown }, { city: 4, buyer: 0, price: 900, breakdown, blocker: 'Sparta is a capital' }] });
    expect(cityBuyTileLines(v, v.cities[2])[0].text).toMatch(/^Buy for 600 Gold \(Starting price 150/);
    expect(cityBuyTileLines(v, v.cities[3])).toEqual([]);
  });
  it('the owner reads what Byzantium would pay and how to raise it', () => {
    const v = view(1, { city_buys: [{ city: 3, buyer: 0, price: 600, breakdown }] });
    expect(cityBuyTileLines(v, v.cities[2])[0].text).toMatch(/^Byzantium can buy this city for 600 Gold .*Military units and Walls raise the price\.$/);
  });
});

describe('the fall of Constantinople', () => {
  it('is public on the capital, and the refuge says what it saves', () => {
    expect(capitalFallLine(view(1, { byzantine_capitals: [{ player: 0, capital: 1 }] }), 1)).toBe('Take it and you take every Byzantine city.');
    expect(capitalFallLine(view(0, { byzantine_capitals: [{ player: 0, capital: 1 }] }), 1)).toBe('Lose this city and you lose every city, unless the Heir shelters in another one.');
    const refuge = { byzantine_capitals: [{ player: 0, capital: 1, refuge: 2 }] };
    expect(capitalFallLine(view(1, refuge), 1)).toBe("Byzantium's capital. Its Heir shelters in Nicaea: take this city and only this city falls.");
    expect(capitalFallLine(view(1, refuge), 2)).toBe("Byzantium's Heir shelters here: if Constantinople falls, this city becomes the capital and nothing else falls.");
    expect(capitalFallLine(view(1, refuge), 3)).toBeNull();
  });
});

describe('report lines', () => {
  const lines = (me: number, events: Event[]) => formatEvents(events, contextFromViews(null, view(me))).map((l) => l.text);
  it('the seller is told what it can do', () => {
    expect(lines(1, [{ type: 'CityOfferMade', city: 3, buyer: 0, seller: 1, escrow: 600, ends_turn: 14 }]))
      .toEqual(['Byzantium is buying Pylos for 600 Gold, at the end of turn 14. Move military units in or build Walls to raise the price, or declare War.']);
    const notice = formatEvents([{ type: 'CityOfferMade', city: 3, buyer: 0, seller: 1, escrow: 900, price: 600, guards_to_stop: 3, ends_turn: 14 }], contextFromViews(null, view(1)));
    expect(notice[0].details).toEqual(['If nothing changes, the city goes to Byzantium and you get the 900 Gold. If its price rises above 900 Gold, the offer fails: 3 more military units in or next to it would do that. War calls it off.']);
    expect(lines(1, [{ type: 'CityOfferMade', city: 3, buyer: 0, seller: 1, escrow: 900, price: 600, ends_turn: 14 }]))
      .toEqual(['Byzantium is buying Pylos for 900 Gold (its price is 600), at the end of turn 14. Move military units in or build Walls to raise the price, or declare War.']);
    expect(lines(0, [{ type: 'CityOfferMade', city: 3, buyer: 0, seller: 1, escrow: 600, ends_turn: 14 }]))
      .toEqual(['You are buying Pylos for 600 Gold: it changes hands at the end of turn 14']);
  });
  it('every outcome names its reason in empire names', () => {
    expect(lines(1, [{ type: 'CityOfferEnded', city: 3, buyer: 0, seller: 1, outcome: 'Cancelled', refund: 600, reason: 'Offer for Pylos called off: War between AI 1 and AI 2. The Gold went back.' }]))
      .toEqual(["Byzantium's offer for Pylos was called off: War between Byzantium and Sparta"]);
    expect(lines(0, [{ type: 'CityOfferEnded', city: 3, buyer: 0, seller: 1, outcome: 'Failed', refund: 540, fee: 60, reason: 'Offer for Pylos failed: the price rose to 750 Gold. 60 Gold was lost.' }]))
      .toEqual(['Your offer for Pylos failed: its price rose to 750 Gold. 60 Gold was lost']);
    expect(lines(1, [{ type: 'CityOfferEnded', city: 3, buyer: 0, seller: 1, outcome: 'Sold', paid: 600, reason: 'AI 1 bought Pylos from AI 2 for 600 Gold.' }]))
      .toEqual(['Byzantium bought Pylos from you: you get 600 Gold']);
  });
  it('the fall and the refuge', () => {
    expect(lines(1, [{ type: 'CapitalFell', player: 0, captor: 1, capital: 1, cities: [2] }]))
      .toEqual(['You took Constantinople and with it every city of Byzantium: 1 more city']);
    expect(lines(2, [{ type: 'HeirRefuge', player: 0, captor: 1, capital: 1, refuge: 2 }]))
      .toEqual(["Sparta took Constantinople, but Byzantium's Heir shelters in Nicaea: it is the capital now, and no other city fell"]);
  });
});
