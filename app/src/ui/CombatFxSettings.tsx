import { useCombatFx } from '../feel';
import { useReducedMotion } from './useReducedMotion';
import './CombatFxSettings.css';

/**
 * Map animations (combat playback and the other map moments): On or Off. Off is the map without them: every change
 * is instant and units stand where they ended; the damage numbers still show. The system's reduced motion forces Off.
 */
export function CombatFxSettings() {
  const { enabled, update } = useCombatFx();
  const reduced = useReducedMotion();
  const on = enabled && !reduced;
  return <div className="hint-settings combat-fx-settings" onKeyDown={(e) => e.stopPropagation()}>
    <div className="combat-fx-row">
      <span id="map-animations-label" className="combat-fx-label">Map animations</span>
      <div className="seg small fx-speed" role="group" aria-labelledby="map-animations-label">
        <button type="button" className={on ? 'on' : ''} aria-pressed={on} disabled={reduced} onClick={() => update({ enabled: true })}>On</button>
        <button type="button" className={on ? '' : 'on'} aria-pressed={!on} onClick={() => update({ enabled: false })}>Off</button>
      </div>
    </div>
    <p className="combat-fx-note">Off: instant moves; damage numbers still show.</p>
    {reduced && <p className="combat-fx-note">Off while your system asks for reduced motion.</p>}
  </div>;
}
