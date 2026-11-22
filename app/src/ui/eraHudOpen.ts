import { useSyncExternalStore } from 'react';

/** Whether the race panel's "How you win" block is expanded to every objective. Compact (one line) by default, remembered per viewer. */
const KEY = 'legatum.era-hud-open.v1';
const listeners = new Set<() => void>();
/** Fallback when storage is blocked or unavailable: the choice still holds for the rest of the session. */
let session: boolean | null = null;

function eraHudOpen(): boolean {
  if (session !== null) return session;
  try { return localStorage.getItem(KEY) === '1'; } catch { return false; }
}

export function setEraHudOpen(open: boolean) {
  session = open;
  try { localStorage.setItem(KEY, open ? '1' : '0'); } catch { /* the in-memory flag carries it */ }
  listeners.forEach((cb) => cb());
}

const subscribe = (cb: () => void) => { listeners.add(cb); return () => { listeners.delete(cb); }; };
export const useEraHudOpen = () => useSyncExternalStore(subscribe, eraHudOpen, () => false);

/** Dev flag for specs about the rest of the race panel (Nobody row, tokens): `localStorage['legatum.dev.no-era-hud'] = '1'` leaves the block out. */
export function noEraHud(): boolean {
  try { return localStorage.getItem('legatum.dev.no-era-hud') === '1'; } catch { return false; }
}
