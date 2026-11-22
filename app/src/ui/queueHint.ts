import { useSyncExternalStore } from 'react';

/** The research queue's first-time hint ("Shift+click a technology to queue it next.") shows until the player has queued once. */
const KEY = 'legatum.research-queue-hint.v1';
const listeners = new Set<() => void>();
/** Fallback when storage is blocked or unavailable: the hint still goes away for the rest of the session. */
let queuedThisSession = false;

export function queueHintSeen(): boolean {
  if (queuedThisSession) return true;
  try { return localStorage.getItem(KEY) === '1'; } catch { return false; }
}

export function markQueueHintSeen() {
  queuedThisSession = true;
  try { localStorage.setItem(KEY, '1'); } catch { /* the in-memory flag carries it */ }
  listeners.forEach((cb) => cb());
}

/** Test hook: forget that the player has queued. */
export function resetQueueHint() {
  queuedThisSession = false;
  try { localStorage.removeItem(KEY); } catch { /* nothing stored */ }
  listeners.forEach((cb) => cb());
}

const subscribe = (cb: () => void) => { listeners.add(cb); return () => { listeners.delete(cb); }; };
export const useQueueHintSeen = () => useSyncExternalStore(subscribe, queueHintSeen, () => false);
