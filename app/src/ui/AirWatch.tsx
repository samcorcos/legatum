import { useGame } from '../store';
import { feel } from '../feel';
import { fightersOnGuard, guardRange } from '../airWing';
import { itemName, unitIconName } from '../data';
import type { PlayerView } from '../types';
import { Icon } from './index';

/** Rows shown in full; the rest fold into one "+N more" line (the dock is short at 1000x600). */
const SHOWN = 2;

/**
 * Fighters that are active without any order (user, 2026-10-17: "it's not clear to me that anything
 * happened"): This Turn names them on guard, so the player knows they are working, and one click opens
 * the city's Air wing.
 */
export function AirWatch({ view }: { view: PlayerView }) {
  const orders = useGame((s) => s.orders[s.seat]) ?? [];
  const held = useGame((s) => s.airHeld[view.me]);
  const select = useGame((s) => s.select);
  if (view.finished) return null;
  const guards = fightersOnGuard(view, orders, held);
  if (!guards.length) return null;
  const faction = view.players.find((p) => p.id === view.me)?.faction;
  return <section className="air-watch" aria-label="Aircraft on guard" data-testid="air-watch">
    <ul>
      {guards.slice(0, SHOWN).map(({ plane, city }) => <li key={plane.id}>
        <button onClick={() => { select({ kind: 'city', id: city.id }, true); feel.select(); }}>
          <Icon name={unitIconName(plane.kind, faction)} size={16} />
          <span><b>{itemName({ kind: 'Unit', unit: plane.kind }, faction)} on guard</b> · {city.name} · covers {guardRange()} tiles</span>
          <span className="air-watch-go" aria-hidden="true">›</span>
        </button>
      </li>)}
      {guards.length > SHOWN && <li className="muted air-watch-more">+{guards.length - SHOWN} more on guard</li>}
    </ul>
  </section>;
}
