import { useGame } from '../store';
import { feel } from '../feel';
import { empireOfPlayer } from '../empires';
import { offerLines } from '../buyCity';
import { Icon } from './bits';
import { Amounts } from './Amounts';
import './siege.css';

/**
 * A city of yours that Byzantium is buying (core `buy_city`) is an alarm, like a city under attack: you may lose it
 * and can still stop it, so it sits at the top of This Turn, never behind routine work. One card per running offer
 * that still holds: who, the sentence (what it pays, when, what keeps the city) and a button that goes to the city.
 */
export function OfferAlerts() {
  const view = useGame((s) => s.views?.[s.seat]);
  const select = useGame((s) => s.select);
  const mine = (view?.city_offers ?? []).filter((o) => o.seller === view?.me && o.price_now <= o.escrow);
  if (!view || !mine.length) return null;
  return (
    <div className="siege-alerts offer-alerts" role="region" aria-label="Cities being bought" data-testid="offer-alerts">
      {mine.map((o) => {
        const city = view.cities.find((c) => c.id === o.city);
        if (!city) return null;
        const title = `${empireOfPlayer(view, o.buyer)} is buying ${city.name}`;
        const go = () => { select({ kind: 'city', id: o.city }, true); feel.select(); };
        return <section key={o.city} className="siege-card approach offer-card" aria-label={title} data-city={o.city} data-kind="offer">
          <header className="siege-head">
            <span className="siege-mark" aria-hidden="true"><Icon name="ui-alert" size={20} /></span>
            <h4 className="siege-title">{title}</h4>
          </header>
          <p className="offer-card-text"><Amounts text={offerLines(view, o).detail} /></p>
          <div className="siege-foot">
            <span className="siege-who">{o.turns_left} turn{o.turns_left === 1 ? '' : 's'} left</span>
            <button className="btn siege-go" onClick={go} aria-label={`Go to ${city.name}`}>Go to city</button>
          </div>
        </section>;
      })}
    </div>
  );
}
