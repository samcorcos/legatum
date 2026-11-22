// Combat card projections call the same Rust functions as resolution and forecasts.
import type { Breakdown, Faction, TechId, Terrain, UnitKind } from './types';
import { RULES, siegeBonusOf } from './data';
import { Game } from './wasm-pkg/legatum.js';

const projection = <T>(query: Record<string, unknown>): T => JSON.parse(Game.combat_projection(JSON.stringify(query))) as T;

export const CITY_HP = RULES.cityHp;
/** HP a city regains at the end of a turn unless it took damage that turn or an at-war military unit is adjacent (core `data::CITY_HEAL`). */
export const CITY_HEAL = RULES.cityHeal;

/** Integer division truncating toward zero, like Rust. */
export const idiv = (a: number, b: number) => Math.trunc(a / b);

export function baseDamage(own: number, other: number): number {
  return projection({ op: 'baseDamage', own, other });
}

export function damage(own: number, other: number, rollPct: number): number {
  return projection({ op: 'damage', own, other, roll: rollPct });
}

/**
 * Core `combat::carried_share`: a hit that kills a layer of a city's defence carries only its
 * unused share (`leftover / damage`) of what the same attack would deal to the next layer.
 */
export function carriedShare(direct: number, leftover: number, damage: number): number {
  return projection({ op: 'carriedShare', direct, leftover, damage });
}

export function breakdown(): Breakdown {
  return { total: 0, parts: [] };
}

export function addPart(b: Breakdown, label: string, amount: number) {
  if (amount !== 0) {
    b.parts.push([label, amount]);
    b.total += amount;
  }
}


/** Core `combat::class_bonus`; `faction` selects the unit's own strength in faction-sensitive rules. */
export function classBonus(own: UnitKind, other: UnitKind, faction?: Faction): { label: string; bonus: number } {
  return projection({ op: 'classBonus', own, other, faction });
}

export const MELEE_VS_MOUNTED_LABEL = RULES.meleeVsMountedLabel;
export const MOUNTED_VS_RANGED_LABEL = RULES.mountedVsRangedLabel;
/** Core `combat::class_bonus_strength`, used by card copy and tests. */
export const classBonusStrength = (base: number): number => projection({ op: 'classBonusStrength', base });
/** Core's pre-Modern mounted penalty when charging a city; shots pay none. */
export const mountedCityPenalty = (faction: Faction, kind: UnitKind, shot: boolean): number =>
  projection({ op: 'mountedCityPenalty', faction, kind, shot });

/** Core `combat::outrider_blocks`: a horse outruns a melee attacker that was not adjacent at turn start. */
export const outriderBlocks = (attacker: UnitKind, defender: UnitKind, startDistance: number): boolean =>
  projection({ op: 'outriderBlocks', attacker, defender, distance: startDistance });

/** Core `combat::fight_class_bonus`, including the Keshig's shot class. */
export function fightClassBonus(own: UnitKind, other: UnitKind, attackerSide: boolean, shot: boolean, faction?: Faction): { label: string; bonus: number } {
  return projection({ op: 'fightBonus', own, other, attacker_side: attackerSide, shot, faction });
}

/** The breakdown line the Spartan phalanx adds (core `combat::PHALANX_LABEL`). */
export const PHALANX_LABEL = RULES.phalanxLabel;

/** Core `data::siege_city_bonus`: strength a Bomber adds against city HP. */
export function siegeCityBonus(kind: UnitKind): number {
  return siegeBonusOf(kind);
}

/** Core `combat::add_fight_class_bonus`: add `own`'s class bonus against `other` in a fight to a strength breakdown (nothing when it has none). */
export function addFightClassBonus(b: Breakdown, own: UnitKind, other: UnitKind, attackerSide: boolean, shot: boolean, faction?: Faction) {
  const { label, bonus } = fightClassBonus(own, other, attackerSide, shot, faction);
  addPart(b, label, bonus);
}

export function unitAttackStrength(kind: UnitKind, hp: number, faction: Faction = 'India', level = 0): Breakdown {
  return projection({ op: 'unitAttack', faction, kind, hp, level });
}

export function unitDefenseStrength(kind: UnitKind, hp: number, terrain: Terrain, fortify: number, faction: Faction = 'India', level = 0): Breakdown {
  return projection({ op: 'unitDefense', faction, kind, hp, level, terrain, fortify });
}

export function unitRangedStrength(kind: UnitKind, hp: number, faction: Faction = 'India', level = 0): Breakdown {
  return projection({ op: 'unitRanged', faction, kind, hp, level });
}

/**
 * Core `combat::city_can_fire`: only a walled city fires, as in Civ 6 (any Walls tier; the
 * Switzerland' Alpine Redoubt is their Walls), and only while it has HP left.
 */
export function cityCanFire(walls: boolean, hp: number): boolean {
  return projection({ op: 'cityCanFire', walls, hp });
}

/**
 * Core `combat::city_tile_defender` (Civ 6): while a city has HP it absorbs every attack on its
 * tile and its garrison is safe; at 0 HP its garrison defends, and a melee win against it
 * captures the city. Each hit is aimed from the stage's snapshot, so damage past 0 HP never
 * carries into the garrison.
 */
export function cityTileDefender(cityHp: number, garrisoned: boolean): 'City' | 'Garrison' {
  return projection({ op: 'cityTileDefender', hp: cityHp, garrisoned });
}

/** Core `data::wall_tier`: 0 no Walls, 1 Walls, 2 a Castle, 3 a Star Fort. `fort` is the tier above plain Walls (0 none, 1 a Castle, 2 a Star Fort). */
export const wallTier = (walls: boolean, fort = 0): number => RULES.wallTiers[Number(walls)][fort];

/** Core `data::siege_tier`: the siege tier of an empire's Siege units, the highest siege technology it knows (Mathematics 1, Machinery 2, Metallurgy 3; 0 ignores nothing). */
export function siegeTier(techs: readonly TechId[]): number {
  return projection({ op: 'siegeTier', techs });
}

/**
 * Core `data::siege_cover`: the Walls tier an attack by a unit of `kind` ignores. An Industrial or later land unit has Siege built in and
 * ignores every tier (3, a Star Fort); one that bought Siege (`siege`) reaches its owner's siege tier (`techs`); aircraft and the rest 0.
 */
export function siegeCover(kind: UnitKind, siege: boolean, techs: readonly TechId[]): number {
  return projection({ op: 'siegeCover', kind, siege, techs });
}

/** Core `data::MAX_WALL_TIER`: a Star Fort. */
export const MAX_WALL_TIER = RULES.maxWallTier;

/**
 * Core `combat::wall_damage_percent`: the percent of its normal damage an attack deals to a walled city's HP. `covered` is the
 * siege tier of the attacker (0 when it has no Siege): at the city's tier or above the Walls do nothing,
 * below it the attack takes back part, and uncovered it deals half. Air strikes are not halved.
 */
export function wallDamagePercent(tier: number, covered: number, air = false): number {
  return projection({ op: 'wallDamagePercent', tier, covered, air });
}

/** Core `combat::city_defense_covered`: the city's strength against an attacker with Siege of tier `covered`: Walls tiers up to it are left out. Every line but the wounds rides the owner's era curve (core `data::era_scaled`). */
export function cityDefenseCovered(population: number, tier: number, covered: number, hp: number, faction: Faction = 'India', era = 0): Breakdown {
  return projection({ op: 'cityDefense', faction, population, tier, covered, hp, era });
}


/**
 * Exact odds over every pair of integer rolls in 80..=120 (41 × 41 outcomes), the same way the
 * core's forecast is specified. `ranged` means the attacker takes no damage.
 */
export function exactOdds(
  atk: number,
  def: number,
  atkHp: number,
  defHp: number,
  ranged: boolean,
  cityPercent = 100,
): { win: number; loss: number; expToAtk: number; expToDef: number } {
  return projection({ op: 'exactOdds', atk, def, atk_hp: atkHp, def_hp: defHp, ranged, city_percent: cityPercent });
}

/** Chance of losing to one share of a pooled melee counterattack, computed with core's rolls. */
export const splitCounterattackLoss = (defender: number, attacker: number, attackerHp: number, attackers: number): number =>
  projection({ op: 'splitCounterattackLoss', defender, attacker, attacker_hp: attackerHp, attackers });
