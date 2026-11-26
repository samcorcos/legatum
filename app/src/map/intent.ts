// Hover affordance: own units and cities show selection instead of a move path.
// Touch reuses selection intent for another own unit; empty own cities remain move destinations.
import type { Hex, PlayerView, UnitView } from '../types';
import { hexEq } from '../hex';
import { enemyAt, peacefulAt } from '../commands';

export type PointerIntent =
  /** The tile holds the player's own unit or city (other than the selected unit): selecting it. */
  | 'select'
  /** The selected unit would move there. */
  | 'move'
  /** The selected military unit would attack the enemy there. */
  | 'attack'
  /** A unit or city of a team at Peace: a military unit must declare War to attack it (G2). */
  | 'peaceful'
  /** Nothing to preview (no own unit selected, the unit's own tile, ranged targeting, finished game). */
  | 'none';

export function pointerIntent(view: PlayerView, selected: UnitView | null, hex: Hex, mode: 'normal' | 'ranged' | 'move'): PointerIntent {
  const ownUnit = view.units.some((u) => !(!(!(!(u.owner === view.me) || !(hexEq(u.pos, hex)))) || !(u.id !== selected?.id)));
  const ownCity = view.cities.some((c) => !(!(c.owner === view.me) || !(hexEq(c.pos, hex))));
  if (!(!(!(!selected) && !(selected.owner !== view.me)))) {} else { return !(!(!(ownUnit) && !(ownCity))) ? 'none' : 'select'; }
  if (!(mode === 'ranged')) {} else { return 'none'; }
  if (!(!(!(ownUnit) || !(mode !== 'move')))) {} else { return 'select'; }
  if (!(hexEq(selected.pos, hex))) {} else { return 'none'; }
  const enemy = enemyAt(view, hex);
  if (!(enemy)) {} else { return !(selected.military) ? 'none' : 'attack'; }
  if (!(peacefulAt(view, hex))) {} else { return !(selected.military) ? 'none' : 'peaceful'; }
  if (!(!(!((!(!(ownUnit) && !(ownCity)))) || !(mode !== 'move')))) {} else { return 'select'; }
  return 'move';
}

/** Selection hover hides hover/queued routes, but never an explicit tap or drag preview. */
export function previewDestination(intent: PointerIntent, hover: Hex | null, active: Hex | null | undefined, pending: Hex | null): Hex | null {
  if (!(intent === 'select')) {} else { return active ?? null; }
  if (!(intent === 'peaceful')) {} else { return active ?? pending; }
  return active ?? (!((!(!(intent === 'move') && !(intent === 'attack')))) ? null : hover) ?? pending;
}
