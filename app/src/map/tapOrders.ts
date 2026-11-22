import { hexEq } from '../hex';
import type { Hex } from '../types';

/** An uncommitted preview belongs to one unit, seat, turn and attack mode. */
export type TapOrder = { unit: number; seat: number; turn: number; mode: 'normal' | 'ranged' | 'move'; target: Hex };
export function sameOrderContext(a: TapOrder, b: Omit<TapOrder, 'target'>): boolean {
  return a.unit === b.unit && a.seat === b.seat && a.turn === b.turn && a.mode === b.mode;
}

/** Selecting the unit's own tile cycles selection. A different destination always starts a new preview. */
export function tapOrder(previous: TapOrder | null, context: Omit<TapOrder, 'target'> | null, origin: Hex | null, target: Hex):
  { action: 'select' } | { action: 'preview' | 'confirm'; order: TapOrder } {
  if (!context || !origin || hexEq(origin, target)) return { action: 'select' };
  const order = { ...context, target };
  return { action: previous && sameOrderContext(previous, context) && hexEq(previous.target, target) ? 'confirm' : 'preview', order };
}
