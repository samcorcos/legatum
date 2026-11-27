// Byzantium's Buy a city on a city's panel (core `buy_city`, rules 65). One block per city panel:
// - Byzantium, on a rival's city: the price with every line (pillar 5), the bid choices (Price, +25%, +50%, +100%)
//   each with what stops it, one primary "Make an offer (N Gold)" that queues `Order::BuyCity` (Undo before the turn
//   ends), or the reason it can't, in place;
// - anyone, on a city under offer: the countdown with the Gold held, and for the seller what they can do;
// - the owner of a city a Byzantium could buy now: its price, so the counter is visible.
import { useState } from 'react';
import { useGame } from '../store';
import type { CityView, Order, PlayerView } from '../types';
import { empireOfPlayer } from '../empires';
import { freeGold } from '../deals';
import { BreakdownTable, Icon } from './bits';
import { Amount, Amounts } from './Amounts';
import { ProgressBar } from './ProgressBar';
import { Tip } from './Tip';
import {
  FAIL_FEE_PERCENT, OFFER_TURNS, bidChoices, bidLabel, myCityBuy, offerGold, offerLines, offerOn, offerTone, priceLines, publicCityBuy, queuedBuy,
  renameSeats, stopLine, withBuy, withoutBuy,
} from '../buyCity';
import './CityBuy.css';

const NO_ORDERS: Order[] = [];

/**
 * What the offer button can do for the viewer now: make an offer of `gold` (absent: the queued bid, else the price),
 * or the reason it can't (said once, in place). `free` is the Gold left after this turn's other orders.
 */
export function buyChoice(view: PlayerView, city: CityView, orders: readonly Order[], gold?: number): { price: number; gold: number; free: number; reason: string | null; queued: boolean; never: boolean } | null {
  const buy = myCityBuy(view, city.id);
  if (!buy || offerOn(view, city.id)) return null;
  const mine = queuedBuy(orders);
  const queued = mine?.city === city.id;
  const bid = gold ?? (queued && mine ? offerGold(view, mine) : buy.price);
  // A capital is never for sale: the panel says so in one quiet line, with no price or button.
  const never = city.is_capital || !!city.original_capital;
  const free = freeGold(view, withoutBuy(orders));
  let reason = buy.blocker ? renameSeats(view, buy.blocker) : null;
  if (!reason && !queued) {
    if (mine) reason = `You are already making an offer for ${view.cities.find((c) => c.id === mine.city)?.name ?? 'another city'}: one offer at a time`;
    else if (free < bid) reason = `Needs ${bid} Gold: you have ${Math.max(0, free)} after this turn's other orders`;
  }
  return { price: buy.price, gold: bid, free, reason, queued, never };
}

/** Whether the selected city's offer is the screen's primary action (the turn's call to action steps back). */
export const buyIsPrimary = (view: PlayerView, city: CityView, orders: readonly Order[]): boolean => {
  const c = buyChoice(view, city, orders);
  return !!c && !c.never && !c.reason && !c.queued;
};

export function CityBuy({ view, city }: { view: PlayerView; city: CityView }) {
  const orders = useGame((s) => s.orders[s.seat]) ?? NO_ORDERS;
  // The bid the player picked, by percent of the price; the queued offer's own bid otherwise.
  const [picked, setPicked] = useState<{ city: number; percent: number } | null>(null);
  const offer = offerOn(view, city.id);
  if (offer) {
    const lines = offerLines(view, offer);
    const tone = offerTone(view, offer);
    return <section className={`city-buy offer ${tone}`} data-testid="city-offer">
      <ProgressBar className="city-buy-countdown" name="City offer countdown" label={<Amounts text={lines.bar} />}
        color="var(--gold, #b8891c)" tone={tone === 'bad' ? 'bad' : 'neutral'}
        value={OFFER_TURNS - offer.turns_left + 1} max={OFFER_TURNS} turns={`${offer.turns_left} turn${offer.turns_left === 1 ? '' : 's'}`} />
      <p className="city-buy-detail"><Amounts text={lines.detail} /></p>
      {lines.options && <p className="city-buy-options" data-testid="city-offer-options"><Amounts text={lines.options} /></p>}
    </section>;
  }
  const buy = myCityBuy(view, city.id);
  const bids = buy ? bidChoices(buy) : [];
  const queuedOrder = queuedBuy(orders);
  const queuedHere = queuedOrder?.city === city.id ? offerGold(view, queuedOrder) : undefined;
  const pick = picked?.city === city.id ? bids.find((b) => b.percent === picked.percent) : undefined;
  const chosen = pick ?? bids.find((b) => b.gold === queuedHere) ?? bids[0];
  const choice = buyChoice(view, city, orders, pick?.gold ?? queuedHere ?? chosen?.gold);
  if (choice && buy) {
    if (choice.never) return <section className="city-buy" data-testid="city-buy"><p className="muted city-buy-never">Not for sale: {renameSeats(view, buy.blocker ?? `${city.name} is a capital`)}.</p></section>;
    const edit = useGame.getState().editOrders;
    const offerNow = () => edit((os) => withBuy(os, city.id, choice.gold === buy.price ? undefined : choice.gold));
    const undo = () => edit((os) => withoutBuy(os));
    const choose = (percent: number, gold: number) => {
      setPicked({ city: city.id, percent });
      // A queued offer follows the new bid.
      if (choice.queued) edit((os) => withBuy(os, city.id, gold === buy.price ? undefined : gold));
    };
    const label = `Make an offer (${choice.gold} Gold)`;
    const closed = !!buy.blocker;
    return <section className="city-buy" data-testid="city-buy">
      <div className="city-buy-title"><Icon name="yield-gold" size={16} /> Buy for <Amount n={choice.price} unit="Gold" /></div>
      <BreakdownTable b={{ total: buy.price, parts: priceLines(buy.breakdown) }} unit="Gold" />
      <p className="city-buy-rule">The owner gets all the Gold you offer and {OFFER_TURNS} turns' warning. War calls it off. If the owner raises the price above your offer, it fails and you lose {FAIL_FEE_PERCENT}% of it. Offer more and it takes more to stop.</p>
      {bids.length > 1 && !closed && <div className="city-buy-bids" role="radiogroup" aria-label="Your offer" data-testid="buy-city-bids">
        {bids.map((b) => {
          const on = b.gold === choice.gold;
          const short = b.gold > choice.free;
          return <button key={b.percent} type="button" role="radio" aria-checked={on} className={`city-buy-bid${on ? ' on' : ''}`}
            data-testid={`buy-city-bid-${b.percent}`} disabled={short && !on} onClick={() => choose(b.percent, b.gold)}>
            <span className="city-buy-bid-head"><span>{bidLabel(b.percent)}</span><Amount n={b.gold} unit="Gold" /></span>
            <span className="city-buy-bid-stop">{short && !on ? `Needs ${b.gold} Gold` : stopLine(b)}</span>
          </button>;
        })}
      </div>}
      <div className="foreign-city-buttons">
        <button className={`btn${choice.reason || choice.queued ? '' : ' primary'}`} data-testid="buy-city-offer"
          aria-pressed={choice.queued} disabled={!!choice.reason && !choice.queued} onClick={choice.queued ? undo : offerNow}>
          {choice.queued ? <><Icon name="ui-check" size={13} className="ui-mark" label="Queued" /> {label}</> : label}
        </button>
      </div>
      {choice.queued && <p className="city-buy-queued" data-testid="buy-city-queued">Queued: your offer for {city.name} starts when the turn ends. <button className="link" onClick={undo}>Undo</button></p>}
      {choice.reason && !choice.queued && <p className="city-buy-why" role="note" data-testid="buy-city-why">{choice.reason}.</p>}
    </section>;
  }
  // The owner's own city that a Byzantium could buy now: its price, and how to raise it.
  const pub = city.owner === view.me ? publicCityBuy(view, city.id) : undefined;
  if (pub) {
    const buyer = empireOfPlayer(view, pub.buyer);
    return <section className="city-buy owner" data-testid="city-buy-owner">
      <Tip as="div" content={<BreakdownTable b={{ total: pub.price, parts: priceLines(pub.breakdown) }} title={`${buyer}'s price for ${city.name}`} unit="Gold" />}>
        <p className="city-buy-detail"><Icon name="yield-gold" size={14} /> <Amounts text={`${buyer} can buy this city for ${pub.price} Gold.`} /></p>
      </Tip>
      <p className="city-buy-options">Military units in or next to it and Walls raise the price.</p>
    </section>;
  }
  return null;
}
