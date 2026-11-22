import { useGame } from './store';
import type { Order, PlayerView } from './types';

/** How long the confirmation stays over the map (G17): a few seconds, or until the next action. */
export const NOTICE_MS = 6000;
let seq = 0;
type Feedback = Parameters<ReturnType<typeof useGame.getState>['editOrders']>[1];

/**
 * G17: queue a decision's orders and confirm it over the map, with Undo. `reopenTeam` reopens the
 * diplomacy inbox at that empire on Undo, so the restored offer can be answered again. Returns
 * false when the edit changed nothing (the answer was already queued).
 */
export function answerDecision(text: string, edit: (orders: Order[], view: PlayerView) => Order[], reopenTeam?: number, feedback?: Feedback): boolean {
  const s = useGame.getState();
  const before = s.orders[s.seat];
  s.editOrders(edit, feedback);
  const after = useGame.getState().orders[s.seat];
  if (after === before) return false;
  useGame.getState().set({ notice: { id: ++seq, text, seat: s.seat, before, after, reopenTeam } });
  return true;
}

/** Undo the confirmed decision: restore the orders from before it, and reopen its offer. */
export function undoNotice(): void {
  const s = useGame.getState();
  const n = s.notice;
  if (!n) return;
  if (n.seat !== s.seat || s.orders[s.seat] !== n.after) { s.set({ notice: null }); return; }
  s.editOrders(() => n.before);
  s.set({ notice: null, ...(n.reopenTeam !== undefined ? { diplomacyOpen: true, diplomacyTeam: n.reopenTeam } : {}) });
}
