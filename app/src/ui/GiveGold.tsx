// Co-op Gold gifts (core `Order::GiveGold`, rules 50): a "Give Gold" control on each living teammate's
// row in the diplomacy panel, with the forecast of both treasuries (pillar 5). The core moves the Gold
// after this turn's purchases, capped at what the giver has left then, so the teammate spends it next turn.
import { useState } from 'react';
import { useGame } from '../store';
import { empireOfPlayer } from '../empires';
import { withThe } from '../resources';
import { playerColor } from '../theme';
import type { Order, PlayerId, PlayerView } from '../types';
import { Amount } from './Amounts';
import { PortraitRoundel } from './premium';
import { GoldStepper } from './GoldStepper';
import { GIFT_ARRIVES, freeGold } from '../deals';
import { QueuedDiplomacyLine, useJustQueued } from './QueuedDiplomacyLine';

/** The short rule, shown under the control. */
export const GIVE_GOLD_RULE = `Give Gold to a teammate. ${GIFT_ARRIVES}`;
/** Deals: the same rule for another empire, in the Guide's words. */
export const GIFT_RIVAL_RULE = GIFT_ARRIVES;

/** Gold already queued as gifts this turn, to anyone. */
export function queuedGifts(orders: Order[]): number {
  return orders.reduce((n, o) => n + (o.type === 'GiveGold' ? o.amount : 0), 0);
}

/**
 * The forecast for a gift: what you keep and what the teammate has next turn (before either side's
 * income). Gold already promised in queued orders (gifts, offers you pay for) is not yours to keep.
 */
export function giftForecast(view: PlayerView, to: PlayerId, amount: number, orders: Order[]): { keep: number; theirs: number | null } {
  const keep = freeGold(view, orders) - amount;
  const mate = view.players.find((p) => p.id === to);
  const theirs = typeof mate?.gold === 'number' ? mate.gold + amount : null;
  return { keep, theirs };
}

export function TeamGifts({ view }: { view: PlayerView }) {
  const orders = useGame((s) => s.orders[s.seat] ?? []);
  const mates = view.players.filter((p) => p.team === view.team && p.id !== view.me && p.alive);
  if (!mates.length || view.finished) return null;
  return <section className="team-gifts" aria-label="Your team" data-testid="team-gifts">
    <h4 className="diplomacy-section">Your team</h4>
    <ul className="empire-list">
      {mates.map((p) => <GiftRow key={p.id} view={view} to={p.id} orders={orders} />)}
    </ul>
    <p className="muted">{GIVE_GOLD_RULE}</p>
  </section>;
}

function GiftRow({ view, to, orders }: { view: PlayerView; to: PlayerId; orders: Order[] }) {
  const mate = view.players.find((p) => p.id === to)!;
  return <li className="gift-row" data-testid="gift-row" style={{ ['--empire' as string]: playerColor(to) }}>
    <div className="gift-who">
      <PortraitRoundel faction={mate.faction} size={24} />
      <strong className="empire-pick-name">{empireOfPlayer(view, to)}</strong>
    </div>
    <GiftControl view={view} to={to} orders={orders} />
  </li>;
}

/**
 * The amount field, Give Gold button and forecast for one recipient: a teammate's row, or (deals,
 * rules 61) a met empire not at War in its empire detail. Queued, it shows a ready line.
 */
export function GiftControl({ view, to, orders, defaultAmount = 50 }: { view: PlayerView; to: PlayerId; orders: Order[]; defaultAmount?: number }) {
  // Named as the empire rows and the report name it: "Aztecs" on the row, "the Aztecs" in a sentence.
  const empire = empireOfPlayer(view, to), inSentence = withThe(empire);
  const Cap = inSentence.charAt(0).toUpperCase() + inSentence.slice(1);
  const queued = orders.filter((o): o is Extract<Order, { type: 'GiveGold' }> => o.type === 'GiveGold' && o.to === to);
  const justQueued = useJustQueued(queued.length > 0);
  const free = freeGold(view, orders);
  // Text, so clearing the field leaves it empty instead of snapping to 0.
  const [text, setText] = useState(() => String(Math.min(defaultAmount, Math.max(0, free))));
  const amount = text.trim() === '' ? NaN : Number(text);
  const valid = Number.isInteger(amount) && amount >= 1 && amount <= free;
  const f = giftForecast(view, to, valid ? amount : 0, orders);
  const give = () => { if (valid) useGame.getState().editOrders((os) => [...os, { type: 'GiveGold', to, amount }]); };
  const cancel = () => useGame.getState().editOrders((os) => os.filter((o) => !(o.type === 'GiveGold' && o.to === to)));
  return queued.length > 0
    ? <QueuedDiplomacyLine name="Gift" onCancel={cancel} testId="gift-queued" focus={justQueued}>you give <Amount n={queued.reduce((n, o) => n + o.amount, 0)} unit="Gold" /> to {inSentence} when you end the turn.</QueuedDiplomacyLine>
    : <div className="gift-control">
        <GoldStepper value={text} onChange={setText} max={Math.max(1, free)} label={`Gold to give ${inSentence}`} />
        <button className="btn" disabled={!valid} onClick={give} data-testid="give-gold">Give Gold</button>
        {valid
          ? <p className="gift-forecast" data-testid="gift-forecast">You keep <Amount n={f.keep} unit="Gold" />.{f.theirs !== null && <> {Cap} will have <Amount n={f.theirs} unit="Gold" />.</>}</p>
          : <p className="order-error gift-forecast" role="alert" data-testid="gift-forecast">
              {free < 1 ? 'You have no Gold to give.' : <>Give between 1 and <Amount n={free} unit="Gold" />.</>}
            </p>}
      </div>;
}
