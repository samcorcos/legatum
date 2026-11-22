// A plate before the game on a phone-sized touch screen: Legatum is built for a laptop or desktop
// (legatum-ui: phone is not a target). "Continue anyway" opens the game and is remembered, so the plate
// shows once per browser. The phone layout code stays; this only says what to expect.
// Not shown under automation (navigator.webdriver), so the
// e2e specs that emulate phones keep testing the layout behind it.
import { useEffect, useRef, useState, type ReactNode } from 'react';
import './crash.css';

export const NARROW_PLATE_KEY = 'legatum.narrow-plate.continued.v1';
export const NARROW_QUERY = '(max-width: 699px) and (pointer: coarse)';

function remembered(): boolean {
  try { return window.localStorage.getItem(NARROW_PLATE_KEY) === '1'; } catch { return false; }
}

function remember() {
  try { window.localStorage.setItem(NARROW_PLATE_KEY, '1'); } catch { /* Storage blocked: the plate returns next visit. */ }
}

/** Whether this window should see the plate before the game. */
export function wantsNarrowPlate(win: Window = window): boolean {
  try {
    if (win.navigator.webdriver) return false;
    return typeof win.matchMedia === 'function' && win.matchMedia(NARROW_QUERY).matches && !remembered();
  } catch {
    return false;
  }
}

export function NarrowPlate({ onContinue }: { onContinue: () => void }) {
  const button = useRef<HTMLButtonElement>(null);
  useEffect(() => { button.current?.focus(); }, []);
  return (
    <main className="crash-screen" data-testid="narrow-plate">
      <div className="crash-card" role="dialog" aria-modal="true" aria-labelledby="narrow-title">
        <p className="crash-kicker">Legatum</p>
        <h1 id="narrow-title">Made for a bigger screen</h1>
        <p className="crash-text">The map and panels are cramped on a phone. Open Legatum on a laptop or desktop for the full map.</p>
        <div className="crash-actions">
          <button ref={button} type="button" className="btn primary crash-reload" onClick={() => { remember(); onContinue(); }}>
            Continue anyway
          </button>
        </div>
      </div>
    </main>
  );
}

/** Shows the plate instead of its children until the player continues. */
export function NarrowGate({ children }: { children: ReactNode }) {
  const [plate, setPlate] = useState(wantsNarrowPlate);
  return plate ? <NarrowPlate onContinue={() => setPlate(false)} /> : <>{children}</>;
}
