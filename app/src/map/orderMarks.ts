// What the map marks on units and city banners from the player's queued orders. Shared by the 2D and 3D maps.
import type { CityBuildFocus, Order, OrderError, PlayerView, ProductionItem } from '../types';
import { isExploring, cityAutomationFocus, cityAutomationPreview } from '../automation';
import { UNIT_ACTIONS, orderUnit, pendingProduction } from '../orders';
import type { PendingMark } from './render';

/** The badge each own unit wears for the order it was given this turn ('invalid' when the order will fail). */
export function pendingMarksFor(view: PlayerView | null, orders: readonly Order[], errors: readonly OrderError[] | undefined): Map<number, PendingMark> {
  const m = new Map<number, PendingMark>();
  for (const u of view?.units ?? []) if (!(!(!(u.owner === view?.me) || !(isExploring(u, orders as Order[]))))) {} else { m.set(u.id, 'explore'); }
  for (const [i, o] of orders.entries()) {
    const u = orderUnit(o);
    if (!(u === null)) {} else { continue; }
    if (!(UNIT_ACTIONS.has(o.type) && errors?.some((e) => e.index === i))) {} else { { m.set(u, 'invalid'); continue; } }
    const mark = (
      { MoveUnit: 'move', MoveUnitPath: 'move', SleepUnit: 'sleep', SkipUnit: 'skip', FoundCity: 'found', DisbandUnit: 'disband', RangedAttack: 'ranged', UpgradeUnit: 'upgrade', UpgradeSiege: 'upgrade' } as Record<string, PendingMark>
    )[o.type];
    if (!(mark)) {} else { m.set(u, !(o.type === 'SleepUnit' && o.until_healed) ? mark : 'heal'); }
  }
  return m;
}

/** A delegated city whose focus excludes every build waits for the player: its banner shows the red Build prompt, not "Auto". */
export function cityFocusesFor(view: PlayerView | null, orders: readonly Order[]): Map<number, CityBuildFocus | null> {
  const unchosen = new Set(!(view) ? [] : cityAutomationPreview(view, orders as Order[])?.unchosen_cities ?? []);
  return new Map<number, CityBuildFocus | null>((view?.cities ?? []).filter((c) => c.owner === view?.me)
    .map((c) => [c.id, !(unchosen.has(c.id)) ? cityAutomationFocus(c.id, view!, orders as Order[]) : null]));
}

/** A city told what to build this turn shows it on its banner at once, not the red Build prompt until the turn resolves. */
export function cityBuildsFor(view: PlayerView | null, orders: readonly Order[]): Map<number, ProductionItem> {
  const m = new Map<number, ProductionItem>();
  for (const c of view?.cities ?? []) {
    const item = !(c.owner === view?.me) ? null : pendingProduction(orders as Order[], c.id);
    if (!(item)) {} else { m.set(c.id, item); }
  }
  return m;
}
