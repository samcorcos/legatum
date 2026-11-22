import { useGame } from '../store';
import { empireOfPlayer } from '../empires';
import type { PlayerView } from '../types';
import { itemName } from '../data';
import { effectiveProduction } from '../orders';
import { PlayerBadge } from './PlayerBadge';
import { feel } from '../feel';

/** Every known city is reachable without the canvas, including cities already producing. */
export function CityList({ view }: { view: PlayerView }) {
  const select = useGame((s) => s.select);
  const selection = useGame((s) => s.selection);
  const orders = useGame((s) => s.orders[s.seat]) ?? [];
  const cities = [...view.cities].sort((a, b) => Number(b.owner === view.me) - Number(a.owner === view.me) || a.name.localeCompare(b.name));
  // The count is yours; known foreign cities are listed too, and counted separately.
  const mine = cities.filter((c) => c.owner === view.me).length;
  const foreign = cities.length - mine;
  // An eliminated empire can have no known cities to list.
  if (!cities.length) return null;
  return (
    <details className="city-list">
      <summary>Cities seen <b>{mine}</b>{foreign > 0 && <span className="muted"> · {foreign} foreign</span>}</summary>
      <div className="city-list-items" role="group" aria-label="Cities seen">
        {cities.map((city) => {
          const owner = view.players.find((p) => p.id === city.owner);
          return (
            <button key={city.id} aria-pressed={selection?.kind === 'city' && selection.id === city.id}
              onClick={() => { select({ kind: 'city', id: city.id }, true); feel.select(); }}>
              <PlayerBadge id={city.owner} name={empireOfPlayer(view, city.owner)} />
              <span>{city.name}</span>
              {city.detail && (() => {
                // Pending choices count, and a blocked queue is waiting rather than empty.
                const current = effectiveProduction(orders, city.id, city.detail.producing);
                const waiting = city.detail.production_queue?.[0];
                return <span className="muted city-list-production">{current ? itemName(current, owner?.faction) : waiting ? `Waiting: ${itemName(waiting, owner?.faction)}` : 'No production'}</span>;
              })()}
            </button>
          );
        })}
      </div>
    </details>
  );
}
