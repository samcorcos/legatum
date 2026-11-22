// Co-op Gold gifts (core `Order::GiveGold`, rules 50): a "Give Gold" control on each living teammate's
// row in the diplomacy panel, with the forecast of both treasuries (pillar 5). The core moves the Gold
// after this turn's purchases, capped at what the giver has left then, so the teammate spends it next turn.
import { useState } from 'react';
import { useGame } from '../store';
import type { Order, PlayerId, PlayerView } from '../types';
import { Amount } from './Amounts';

/** The short rule, shown under the control. */
export const GIVE_GOLD_RULE = 'Give Gold to a teammate. They get it next turn, after your purchases this turn.';

/** Gold already queued as gifts this turn, to anyone. */
export function queuedGifts(orders: Order[]): number {
  return orders.reduce((n, o) => n + (o.type === 'GiveGold' ? o.amount : 0), 0);
}

/** The forecast for a gift: what you keep and what the teammate has next turn (before either side's income). */
export function giftForecast(view: PlayerView, to: PlayerId, amount: number, orders: Order[]): { keep: number; theirs: number | null } {
  const keep = view.economy.gold - queuedGifts(orders) - amount;
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
      {mates.map((p) => <GiftRow key={p.id} view={view} to={p.id} name={p.name} orders={orders} />)}
    </ul>
    <p className="muted">{GIVE_GOLD_RULE}</p>
  </section>;
}

function GiftRow({ view, to, name, orders }: { view: PlayerView; to: PlayerId; name: string; orders: Order[] }) {
  const queued = orders.filter((o): o is Extract<Order, { type: 'GiveGold' }> => o.type === 'GiveGold' && o.to === to);
  const free = view.economy.gold - queuedGifts(orders);
  const [amount, setAmount] = useState(() => Math.min(50, Math.max(0, free)));
  const valid = Number.isInteger(amount) && amount >= 1 && amount <= free;
  const f = giftForecast(view, to, valid ? amount : 0, orders);
  const give = () => useGame.getState().editOrders((os) => [...os, { type: 'GiveGold', to, amount }]);
  const undo = () => useGame.getState().editOrders((os) => os.filter((o) => !(o.type === 'GiveGold' && o.to === to)));
  return <li className="gift-row" data-testid="gift-row">
    <strong>{name}</strong>
    {queued.length > 0
      ? <p data-testid="gift-queued">Queued: give <Amount n={queued.reduce((n, o) => n + o.amount, 0)} unit="Gold" /> to {name}. <button className="link" onClick={undo}>Undo</button></p>
      : <div className="gift-control">
          <label>Amount <input type="number" min={1} max={Math.max(1, free)} step={1} value={amount} aria-label={`Gold to give ${name}`}
            onChange={(e) => setAmount(Math.floor(Number(e.target.value)))} /></label>
          <button className="btn" disabled={!valid} onClick={give} data-testid="give-gold">Give Gold</button>
          <p className="gift-forecast" data-testid="gift-forecast">
            {valid
              ? <>You keep <Amount n={f.keep} unit="Gold" />{f.theirs !== null && <>; {name} has <Amount n={f.theirs} unit="Gold" /></>} next turn, before income and purchases.</>
              : free < 1 ? 'You have no Gold to give.' : <>Give between 1 and <Amount n={free} unit="Gold" />.</>}
          </p>
        </div>}
  </li>;
}
