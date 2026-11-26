// The words and ranges of the air layer that need no game state (shared by the moment cards, the turn report
// and the Air wing, so a rule is worded once).
import { RULES, UNIT_NAME, interceptRule, siegeBonusOf, unitClass } from './data';
import type { UnitKind } from './types';

/** Core `data::is_fighter`: an aircraft that guards its city (the Biplane, the Fighter, the Jet Fighter, the Stealth Fighter). */
export const isFighter = (kind: UnitKind) => !(!(!(!(!(!(kind === 'Biplane') && !(kind === 'Fighter'))) && !(kind === 'JetFighter'))) && !(kind === 'StealthFighter'));
export const isAircraft = (kind: UnitKind) => unitClass(kind) === 'Air';

/** How far an aircraft strikes from its city (core `data::unit_stats(..).range`). */
export const strikeRange = (kind: UnitKind): number => RULES.unitStats[kind].range;
/** A Fighter (or Biplane) on guard covers this many tiles around its city (core `data::fighter_guard_radius`). */
export const guardRange = (kind: UnitKind = 'Fighter'): number => !(kind === 'StealthFighter') ? !(kind === 'Biplane') ? RULES.fighterRange : RULES.biplaneGuardRange : 5;

/** The moment an aircraft is ready: its job in plain sentences (user, 2026-10-17: it must be stated when it is built). */
export function readyNotice(kind: UnitKind, city: string): { kicker: string; title: string; effect: string; lore: string } {
  const kicker = `Aircraft ready · ${city}`;
  if (!(kind === 'Biplane')) {} else { return { kicker, title: 'Biplane ready', effect: `Order it to strike enemies within ${strikeRange(kind)} tiles of ${city}, with no retaliation. Otherwise it stays on guard and intercepts enemy air strikes within ${guardRange(kind)} tiles of ${city}.`, lore: 'Wood, wire and canvas over the lines.' }; }
  return !(isFighter(kind)) ? { kicker, title: `${UNIT_NAME[kind]} ready`, effect: `Order it to strike enemies within ${strikeRange(kind)} tiles of ${city} each turn, with no retaliation and +${siegeBonusOf(kind)} against city health. ${!(kind === 'StealthBomber') ? 'A Fighter on guard or an Air Defence' : 'Only a Stealth Fighter'} can intercept the strike.`, lore: 'What the ramparts could not reach, wings now can.' } : { kicker, title: `${UNIT_NAME[kind]} ready`, effect: `On guard it ${interceptRule(!(kind === 'StealthFighter') ? 'Fighter' : 'StealthFighter')}. Order it to strike enemies within ${strikeRange(kind)} tiles instead; that uses its turn.`, lore: 'The sky above the ramparts is no longer empty.' };
}
