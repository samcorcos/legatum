// Mirror of crates/core/src/rules/promotion.rs: experience and automatic promotions (Civ 6's
// experience model without promotion choices). Every number comes from the core through
// rules.generated.json; these client functions format core-derived previews.
import type { Faction } from './types';
import { RULES } from './data';

const P = RULES.promotion;
export const MAX_LEVEL: number = P.maxLevel;
const SPARTA_MAX_LEVEL: number = P.spartaMaxLevel;
export const STRENGTH_PERCENT: number = P.strengthPercent;

/** Cumulative XP to reach levels 1..MAX_LEVEL. */
export const PROMOTION_XP: readonly number[] = P.xp;

export function promotionLevel(xp: number, faction?: Faction): number {
  return thresholdsFor(faction).filter((t) => xp >= t).length;
}

export function thresholdsFor(faction?: Faction): readonly number[] {
  return faction === 'Sparta' ? [...PROMOTION_XP, P.ironGuardXp] : PROMOTION_XP;
}

/** Cumulative XP for the next level, or null at the maximum level. */
export function nextLevelXp(xp: number, faction?: Faction): number | null {
  return thresholdsFor(faction)[promotionLevel(xp, faction)] ?? null;
}

/** `base * (100 + 10 * level) / 100 - base`, rounded down. */
export function strengthBonus(base: number, level: number): number {
  const percent = STRENGTH_PERCENT * Math.min(level, SPARTA_MAX_LEVEL);
  return Math.trunc((base * (100 + percent)) / 100) - base;
}


/** XP from one unit-vs-unit fight for the side with base strength `own`. */
export function unitFightXp(own: number, enemy: number, enemyDied: boolean, ranged: boolean, initiator: boolean): number {
  if (own <= 0 || enemy <= 0) return 0;
  let tenths = Math.trunc((enemy * 10 * (enemyDied ? 2 : 1)) / own);
  tenths += 10 * (ranged ? P.rangedXp : P.meleeXp);
  if (initiator) tenths += 10 * P.initiatorXp;
  return Math.min(Math.trunc((tenths + 5) / 10), P.unitFightCap);
}

/** XP for attacking a city from `before` to `after` HP. */
export function cityAttackXp(before: number, after: number): number {
  if (before <= 0) return 0;
  return after <= 0 ? P.cityDefeatedXp : P.cityAttackXp;
}
