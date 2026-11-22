//! Combat math shared by resolution and forecasts, on primitive inputs so a PlayerView can use
//! it too. Civ 6-style: damage = 30 * e^(strength difference / 25), as an integer table.

use crate::data::{self, CITY_BASE_STRENGTH, CITY_HP, CITY_STRENGTH_PER_POP, UNIT_HP};
use crate::experiments::FactionRuleSet;
use crate::types::*;

/// round(30 * e^(d / 25)) for d in -50..=50.
const DAMAGE_TABLE: [i32; 101] = [
    4, 4, 4, 5, 5, 5, 5, 5, 6, 6, 6, 6, 7, 7, 7, 7, 8, 8, 8, 9, 9, 9, 10, 10, 11, 11, 11, 12, 12,
    13, 13, 14, 15, 15, 16, 16, 17, 18, 19, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32,
    34, 35, 37, 38, 40, 41, 43, 45, 47, 48, 50, 53, 55, 57, 59, 62, 64, 67, 69, 72, 75, 78, 82, 85,
    88, 92, 96, 100, 104, 108, 112, 117, 122, 127, 132, 137, 143, 149, 155, 161, 168, 174, 181,
    189, 197, 205, 213, 222,
];

pub const ROLL_MIN: i32 = 80;
pub const ROLL_MAX: i32 = 120;

/// The class bonus (user, 2026-10-10: "melee beats horse, ranged beats melee, horse beats ranged"):
/// a side in a fight against the class it beats gains this percent of its unit's base strength,
/// whether it attacks or defends, in melee or a meeting. The melee line (the spear line) beats
/// mounted; mounted beats ranged, but only in melee: a charge reaches them, while a mounted unit
/// that is the target of a shot gets no bonus (designer, 2026-11-10). Ranged has no bonus: its range, which lets it strike unanswered, is its edge over
/// the melee line. A percentage (user, 2026-10-31, replacing the flat +18), so the counter bites
/// the same in every era, as every era bonus must; rounded down like a promotion's.
pub const CLASS_BONUS_PERCENT: i32 = 40;
/// The breakdown line of the melee line's bonus against mounted (states `CLASS_BONUS_PERCENT`).
pub const MELEE_VS_MOUNTED_LABEL: &str = "Melee line against mounted (+40%)";
/// The breakdown line of the mounted bonus against ranged (states `CLASS_BONUS_PERCENT`).
pub const MOUNTED_VS_RANGED_LABEL: &str = "Mounted against ranged (+40%)";

/// The class bonus of a unit with base strength `base`: `CLASS_BONUS_PERCENT` of it, rounded down.
pub fn class_bonus_strength(base: i32) -> i32 {
    base * CLASS_BONUS_PERCENT / 100
}

/// The unit triangle ends at the World War era, the era first called Modern (user, 2026-10-20, replacing the open-ground rule of
/// 2026-10-19): from this era tier (the Infantry's, the Tank's and the Artillery's) units have no
/// class bonuses. A class bonus applies only when both units are from before it, so an Infantry
/// meeting a Cavalry or a Tank gets nothing and neither does the Tank against an Artillery: from
/// then on units differ by strength, cost and moves. The Machine Age (era tier 5, 2026-11-14)
/// sits before it, so its units still fight in the triangle; the Modern era (2026-11-14) is after it.
pub const TRIANGLE_END_ERA_TIER: u8 = 6;

/// Whether the triangle still applies between these two units: both from before the World War era.
pub fn triangle_applies(own: UnitKind, other: UnitKind) -> bool {
    data::unit_era_tier(own) < TRIANGLE_END_ERA_TIER
        && data::unit_era_tier(other) < TRIANGLE_END_ERA_TIER
}

/// A horse can escape a melee attack begun from beyond adjacency by moving away. Armour (the
/// EarlyTank, the Tank, the Autonomous Armour) cannot. This identifies that case; it does not
/// prohibit the attack order or a hit on a horse that stays.
pub fn outrider_blocks(attacker: UnitKind, defender: UnitKind, start_distance: i32) -> bool {
    data::unit_class(attacker) == Some(UnitClass::Melee)
        && data::is_horse_mounted(defender)
        && start_distance > 1
}

/// Strength `own` gains against `other`, and the breakdown line that says why, from `own`'s
/// base strength without faction edges (`data::unit_stats`; the AI's estimates). A fight uses
/// `class_bonus_for`. Nothing from the Modern era on (`triangle_applies`).
pub fn class_bonus(own: UnitKind, other: UnitKind) -> (&'static str, i32) {
    bonus_of(
        data::unit_stats(own).strength,
        triangle_applies(own, other),
        data::unit_class(own),
        data::unit_class(other),
    )
}

/// `class_bonus_for` with every faction rule on.
pub fn class_bonus_for(faction: Faction, own: UnitKind, other: UnitKind) -> (&'static str, i32) {
    class_bonus_for_rules(FactionRuleSet::EMPTY, faction, own, other)
}

/// `class_bonus` for a unit of `faction`, from its faction's base strength.
pub fn class_bonus_for_rules(
    off: FactionRuleSet,
    faction: Faction,
    own: UnitKind,
    other: UnitKind,
) -> (&'static str, i32) {
    bonus_of(
        data::unit_stats_for_rules(off, faction, own).strength,
        triangle_applies(own, other),
        data::unit_class(own),
        data::unit_class(other),
    )
}

/// The breakdown line of the class bonus between two classes, or None when `own` does not beat
/// `other` (`class_bonus` on kinds, `fight_class_bonus` in a fight).
pub fn class_bonus_between(
    own: Option<UnitClass>,
    other: Option<UnitClass>,
) -> Option<&'static str> {
    match (own, other) {
        (Some(UnitClass::Melee), Some(UnitClass::Mounted)) => Some(MELEE_VS_MOUNTED_LABEL),
        (Some(UnitClass::Mounted), Some(UnitClass::Ranged)) => Some(MOUNTED_VS_RANGED_LABEL),
        _ => None,
    }
}

fn bonus_of(
    base: i32,
    applies: bool,
    own: Option<UnitClass>,
    other: Option<UnitClass>,
) -> (&'static str, i32) {
    match class_bonus_between(own, other) {
        Some(label) if applies => (label, class_bonus_strength(base)),
        _ => ("", 0),
    }
}

/// The class a unit fights as in one fight. A shot is a ranged attack: a mounted unit firing its
/// bow (the Mongol Keshig) fights that shot as a ranged unit, so a spear line gets no bonus
/// against arrows and a mounted target gets no bonus against them, as against an Archer's. Charging, and
/// defending, it is mounted. Every other unit fights as its own class.
pub fn fight_class(kind: UnitKind, shooting: bool) -> Option<UnitClass> {
    match data::unit_class(kind) {
        Some(UnitClass::Mounted) if shooting => Some(UnitClass::Ranged),
        class => class,
    }
}

/// The class bonus `own` gets against `other` in a fight where the attacker `shot` (a ranged
/// attack) or not: `attacker_side` says which of the two is the attacker. From `own`'s base
/// strength without faction edges (the AI's estimates); a fight uses `fight_class_bonus_for`.
/// None from the Modern era on (`triangle_applies`).
pub fn fight_class_bonus(
    own: UnitKind,
    other: UnitKind,
    attacker_side: bool,
    shot: bool,
) -> (&'static str, i32) {
    fight_bonus_of(
        data::unit_stats(own).strength,
        own,
        other,
        attacker_side,
        shot,
    )
}

/// `fight_class_bonus_for` with every faction rule on.
pub fn fight_class_bonus_for(
    faction: Faction,
    own: UnitKind,
    other: UnitKind,
    attacker_side: bool,
    shot: bool,
) -> (&'static str, i32) {
    fight_class_bonus_for_rules(
        FactionRuleSet::EMPTY,
        faction,
        own,
        other,
        attacker_side,
        shot,
    )
}

/// `fight_class_bonus` for a unit of `faction`, from its faction's base strength.
pub fn fight_class_bonus_for_rules(
    off: FactionRuleSet,
    faction: Faction,
    own: UnitKind,
    other: UnitKind,
    attacker_side: bool,
    shot: bool,
) -> (&'static str, i32) {
    fight_bonus_of(
        data::unit_stats_for_rules(off, faction, own).strength,
        own,
        other,
        attacker_side,
        shot,
    )
}

fn fight_bonus_of(
    base: i32,
    own: UnitKind,
    other: UnitKind,
    attacker_side: bool,
    shot: bool,
) -> (&'static str, i32) {
    let (own_shoots, other_shoots) = if attacker_side {
        (shot, false)
    } else {
        (false, shot)
    };
    let own_class = fight_class(own, own_shoots);
    // A mounted unit that is the target of a shot gets no bonus: its bonus is for charging
    // (designer, 2026-11-10).
    let under_fire = shot && own_class == Some(UnitClass::Mounted);
    bonus_of(
        base,
        triangle_applies(own, other) && !under_fire,
        own_class,
        fight_class(other, other_shoots),
    )
}

/// Riders are poor at storming a city (2026-10-31, with the stronger mounted line): a pre-Modern
/// mounted unit charging a city loses this percent of its base strength, which takes it a little
/// below its era's melee line (Horseman 26 - 8 = 18 against the Spearman's 20). So it can still
/// finish a broken city, but taking cities stays the melee line's job, and the mounted premium
/// buys open-field strength and moves, not sieges. Chosen in the combat lab (2026-10-31): at 0%
/// an all-mounted army captured 65-73% of cities against the melee line's 45-59% at equal
/// Production; at 33% it is level in the open and below the melee line against Walls. Shots (the
/// Keshig's bow) and the Tank onward are unaffected.
pub const MOUNTED_CITY_PENALTY_PERCENT: i32 = 33;
/// The breakdown line of the mounted penalty against a city (states the percentage).
pub const MOUNTED_CITY_LABEL: &str = "Mounted against a city (-33%)";

/// `mounted_city_penalty` with every faction rule on.
pub fn mounted_city_penalty(faction: Faction, kind: UnitKind, shot: bool) -> i32 {
    mounted_city_penalty_rules(FactionRuleSet::EMPTY, faction, kind, shot)
}

/// The strength a unit of `faction` and `kind` loses charging a city (`MOUNTED_CITY_PENALTY_PERCENT`
/// of its base strength, rounded down), 0 for a shot or anything but a horse (armour, from the
/// EarlyTank on, pays nothing, and neither does the Indian War Elephant).
pub fn mounted_city_penalty_rules(
    off: FactionRuleSet,
    faction: Faction,
    kind: UnitKind,
    shot: bool,
) -> i32 {
    if shot || !data::is_horse_mounted(kind) || data::is_war_elephant_rules(off, faction, kind) {
        return 0;
    }
    data::unit_stats_for_rules(off, faction, kind).strength * MOUNTED_CITY_PENALTY_PERCENT / 100
}

/// `add_mounted_city_penalty` with every faction rule on.
pub fn add_mounted_city_penalty(b: &mut Breakdown, faction: Faction, kind: UnitKind, shot: bool) {
    add_mounted_city_penalty_rules(b, FactionRuleSet::EMPTY, faction, kind, shot)
}

/// Add the mounted penalty against a city to an attacker's breakdown (nothing when it has none).
pub fn add_mounted_city_penalty_rules(
    b: &mut Breakdown,
    off: FactionRuleSet,
    faction: Faction,
    kind: UnitKind,
    shot: bool,
) {
    let penalty = mounted_city_penalty_rules(off, faction, kind, shot);
    if penalty != 0 {
        b.add(MOUNTED_CITY_LABEL, -penalty);
    }
}

/// Add `own`'s class bonus against `other` in a fight to a strength breakdown (nothing when it
/// has none): `own` is of `faction`, the attacker's side when `attacker_side`, a ranged attack
/// when `shot`.
pub fn add_fight_class_bonus_rules(
    b: &mut Breakdown,
    off: FactionRuleSet,
    faction: Faction,
    own: UnitKind,
    other: UnitKind,
    attacker_side: bool,
    shot: bool,
) {
    let (label, bonus) = fight_class_bonus_for_rules(off, faction, own, other, attacker_side, shot);
    if bonus != 0 {
        b.add(label, bonus);
    }
    let (label, extra) =
        fight_reislaufer_bonus_for_rules(off, faction, own, other, attacker_side, shot);
    if extra != 0 {
        b.add(label, extra);
    }
}

/// The breakdown lines of the Gaesatae's charge and missing shield wall.
pub const GAESATAE_ATTACK_LABEL: &str = "Gaesatae: furious charge";
pub const GAESATAE_DEFENSE_LABEL: &str = "Gaesatae: no shield wall";
/// The breakdown line of the Reisläufer's extra bonus against mounted (states the percentage).
pub const REISLAUFER_LABEL: &str = "Reisläufer pikes against mounted (+50%)";

/// Strength a Celtic Gaesatae gains attacking (0 for any other unit).
pub fn gaesatae_attack(off: FactionRuleSet, made_by: Faction, kind: UnitKind) -> i32 {
    if data::is_gaesatae_rules(off, made_by, kind) {
        data::era_scaled(data::GAESATAE_ATTACK_STRENGTH, data::unit_era_tier(kind))
    } else {
        0
    }
}

/// Strength a Celtic Gaesatae loses defending (a negative number; 0 for any other unit).
pub fn gaesatae_defense(off: FactionRuleSet, made_by: Faction, kind: UnitKind) -> i32 {
    if data::is_gaesatae_rules(off, made_by, kind) {
        data::era_scaled(data::GAESATAE_DEFENSE_STRENGTH, data::unit_era_tier(kind))
    } else {
        0
    }
}

/// The extra a Swiss Reisläufer gets against mounted, on top of the melee line's bonus
/// (`REISLAUFER_MOUNTED_PERCENT` of its base strength, rounded down): only where the melee line's
/// own bonus applies (`fight_class_bonus_for_rules`), so it ends with the triangle and never
/// helps a unit under fire.
pub fn fight_reislaufer_bonus_for_rules(
    off: FactionRuleSet,
    made_by: Faction,
    own: UnitKind,
    other: UnitKind,
    attacker_side: bool,
    shot: bool,
) -> (&'static str, i32) {
    if data::is_reislaufer_rules(off, made_by, own)
        && fight_class_bonus_for_rules(off, made_by, own, other, attacker_side, shot).0
            == MELEE_VS_MOUNTED_LABEL
    {
        let base = data::unit_stats_for_rules(off, made_by, own).strength;
        (
            REISLAUFER_LABEL,
            base * data::REISLAUFER_MOUNTED_PERCENT / 100,
        )
    } else {
        ("", 0)
    }
}

/// `add_phalanx` with every faction rule on.
pub fn add_phalanx(b: &mut Breakdown, faction: Faction, kind: UnitKind, adjacent_hoplite: bool) {
    add_phalanx_rules(b, FactionRuleSet::EMPTY, faction, kind, adjacent_hoplite)
}

/// The Spartan Hoplite's phalanx line: `data::HOPLITE_PHALANX_STRENGTH` while another Hoplite of
/// its empire stands next to it (`adjacent_hoplite`, judged by the caller at the fight's stage).
/// Nothing for any other unit.
pub fn add_phalanx_rules(
    b: &mut Breakdown,
    off: FactionRuleSet,
    faction: Faction,
    kind: UnitKind,
    adjacent_hoplite: bool,
) {
    if adjacent_hoplite && data::is_hoplite_rules(off, faction, kind) {
        b.add(PHALANX_LABEL, data::HOPLITE_PHALANX_STRENGTH);
    }
}

/// The breakdown line the phalanx adds.
pub const PHALANX_LABEL: &str = "Phalanx: next to another Hoplite";

/// Build an attacker's strength in the same order for resolution and both forecasts.
/// `meeting` is only meaningful for a melee fight; `city` adds the city-specific modifiers.
/// `faction` is the owner's (its faction-wide rules); `made_by` the faction whose version of the
/// unit it is (a hired Company is its seller's: the unit's own stat line and rules follow it).
#[allow(clippy::too_many_arguments)]
pub fn attack_strength_rules(
    off: FactionRuleSet,
    faction: Faction,
    made_by: Faction,
    kind: UnitKind,
    hp: i32,
    level: u8,
    ranged: bool,
    meeting: bool,
    phalanx: bool,
    unsupplied: Option<crate::resources::Resource>,
    city: bool,
) -> Breakdown {
    let mut strength = if ranged {
        unit_ranged_strength_for_rules(off, faction, made_by, kind, hp, level)
    } else if meeting {
        unit_meeting_strength_for_rules(off, faction, made_by, kind, hp, level)
    } else {
        unit_attack_strength_for_rules(off, faction, made_by, kind, hp, level)
    };
    add_phalanx_rules(&mut strength, off, made_by, kind, phalanx);
    crate::resources::add_unsupplied(&mut strength, made_by, kind, ranged, unsupplied);
    if city {
        if ranged {
            strength.add("Siege against city HP", data::siege_city_bonus(kind));
        }
        add_mounted_city_penalty_rules(&mut strength, off, made_by, kind, ranged);
    }
    strength
}

/// Damage a side with strength `own` deals to a side with strength `other`, before the roll.
pub fn base_damage(own: i32, other: i32) -> i32 {
    let d = (own - other).clamp(-50, 50);
    DAMAGE_TABLE[(d + 50) as usize]
}

/// Damage after a roll in `ROLL_MIN..=ROLL_MAX` percent. Always at least 1.
pub fn damage(own: i32, other: i32, roll_pct: i32) -> i32 {
    (base_damage(own, other) * roll_pct / 100).max(1)
}

/// Percent of its normal damage an attack deals to a walled city's HP (user, 2026-10-18: walls
/// make a city slower and costlier to take, never impossible). `wall_tier` is the city's
/// (`data::wall_tier`); `covered` is the siege tier of Siege of the attacker,
/// 0 when none does. A covered attacker whose Siege unit reaches the city's highest tier fights as
/// if there were no Walls: full damage. A lower tier takes back only part of it, and an
/// uncovered attacker deals half. Aircraft strikes are not halved. Only a hit aimed at city HP is
/// reduced; direct hits on a 0-HP city's garrison are not.
pub fn wall_damage_percent(wall_tier: u8, covered: u8, air: bool) -> i32 {
    if wall_tier == 0 || air || covered >= wall_tier {
        100
    } else if covered == 0 {
        WALLS_DAMAGE_PERCENT
    } else {
        SIEGE_PARTIAL_PERCENT
    }
}

/// Percent of its normal damage an attack by `kind` deals to a city's HP: the Walls
/// (`wall_damage_percent`) times the kind's own share (`data::city_hp_damage_percent`, half for a
/// Biplane).
pub fn city_damage_percent(kind: UnitKind, wall_tier: u8, covered: u8) -> i32 {
    wall_damage_percent(wall_tier, covered, data::is_aircraft(kind))
        * data::city_hp_damage_percent(kind)
        / 100
}

/// Damage to a walled city's HP from an attack with `normal_damage` (`wall_damage_percent`).
pub fn damage_to_city_hp(percent: i32, normal_damage: i32) -> i32 {
    normal_damage * percent / 100
}

/// Walls halve the damage of an attack the attacker has no Siege to cancel.
pub const WALLS_DAMAGE_PERCENT: i32 = 50;
/// A Siege unit whose tier is below the city's walls cancels the tiers it reaches
/// and half the halving: its attacks deal this percent (siege study 2026-10-18; at 50 a lower
/// tier barely beat having none, at 100 the tiers stopped mattering).
pub const SIEGE_PARTIAL_PERCENT: i32 = 75;

/// Per-hit carry (designer, 2026-11-11): a hit of `damage` against a layer (the city's HP, a
/// garrison, a relief unit) that had `damage - leftover` HP left kills it with `leftover` to
/// spare. Only that unused share of the hit carries on: `direct`, the damage the same attack
/// (same roll) would deal to the next layer directly, times `leftover / damage`, rounded down.
/// Used by resolution (`rules::battle::carried_damage`) and the assault forecast.
pub fn carried_share(direct: i32, leftover: i32, damage: i32) -> i32 {
    if leftover <= 0 || damage <= 0 || direct <= 0 {
        return 0;
    }
    (i64::from(direct) * i64::from(leftover.min(damage)) / i64::from(damage)) as i32
}

/// -1 strength per 10 HP lost.
fn wounded(b: &mut Breakdown, hp: i32, max_hp: i32, per: i32) {
    b.add("Wounded", -((max_hp - hp) / per));
}

/// Promotion: +`PROMOTION_STRENGTH_PERCENT`% of the unit's base strength per level, as its own
/// line right after the base strength.
fn promoted(b: &mut Breakdown, base: i32, level: u8) {
    // `add` drops a zero line, so only a real bonus needs its label formatted.
    let bonus = crate::rules::promotion::strength_bonus(base, level);
    if bonus != 0 {
        b.add(crate::rules::promotion::strength_label(level), bonus);
    }
}

pub fn unit_attack_strength(kind: UnitKind, hp: i32) -> Breakdown {
    unit_attack_strength_for_rules(
        FactionRuleSet::EMPTY,
        Faction::India,
        Faction::India,
        kind,
        hp,
        0,
    )
}

/// `unit_attack_strength_for` with every faction rule on.
pub fn unit_attack_strength_for(faction: Faction, kind: UnitKind, hp: i32, level: u8) -> Breakdown {
    unit_attack_strength_for_rules(FactionRuleSet::EMPTY, faction, faction, kind, hp, level)
}

pub fn unit_attack_strength_for_rules(
    off: FactionRuleSet,
    faction: Faction,
    made_by: Faction,
    kind: UnitKind,
    hp: i32,
    level: u8,
) -> Breakdown {
    let mut b = Breakdown::default();
    let base = data::unit_stats_made_by_rules(off, faction, made_by, kind).strength;
    b.add(data::unit_name_for(made_by, kind), base);
    promoted(&mut b, base, level);
    let charge = gaesatae_attack(off, made_by, kind);
    if charge != 0 {
        b.add(GAESATAE_ATTACK_LABEL, charge);
    }
    if !data::ignores_wounds_rules(off, faction, kind) {
        wounded(&mut b, hp, UNIT_HP, 10);
    }
    b
}

pub fn unit_defense_strength(kind: UnitKind, hp: i32, terrain: Terrain, fortify: i32) -> Breakdown {
    unit_defense_strength_for_rules(
        FactionRuleSet::EMPTY,
        Faction::India,
        Faction::India,
        kind,
        hp,
        terrain,
        fortify,
        0,
    )
}

/// `unit_defense_strength_for` with every faction rule on.
pub fn unit_defense_strength_for(
    faction: Faction,
    kind: UnitKind,
    hp: i32,
    terrain: Terrain,
    fortify: i32,
    level: u8,
) -> Breakdown {
    unit_defense_strength_for_rules(
        FactionRuleSet::EMPTY,
        faction,
        faction,
        kind,
        hp,
        terrain,
        fortify,
        level,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn unit_defense_strength_for_rules(
    off: FactionRuleSet,
    faction: Faction,
    made_by: Faction,
    kind: UnitKind,
    hp: i32,
    terrain: Terrain,
    fortify: i32,
    level: u8,
) -> Breakdown {
    let mut b = Breakdown::default();
    let base = data::unit_stats_made_by_rules(off, faction, made_by, kind).strength;
    b.add(data::unit_name_for(made_by, kind), base);
    promoted(&mut b, base, level);
    let no_shield_wall = gaesatae_defense(off, made_by, kind);
    if no_shield_wall != 0 {
        b.add(GAESATAE_DEFENSE_LABEL, no_shield_wall);
    }
    let terrain_defense = data::terrain_defense_rules(off, faction, terrain);
    if terrain_defense != 0 {
        b.add(format!("{terrain:?} terrain"), terrain_defense);
    }
    b.add("Fortified", fortify);
    if !data::ignores_wounds_rules(off, faction, kind) {
        wounded(&mut b, hp, UNIT_HP, 10);
    }
    b
}

/// Ranged attack strength. Ranged attacks take no retaliation.
pub fn unit_ranged_strength(kind: UnitKind, hp: i32) -> Breakdown {
    unit_ranged_strength_for_rules(
        FactionRuleSet::EMPTY,
        Faction::India,
        Faction::India,
        kind,
        hp,
        0,
    )
}

/// `unit_ranged_strength_for` with every faction rule on.
pub fn unit_ranged_strength_for(faction: Faction, kind: UnitKind, hp: i32, level: u8) -> Breakdown {
    unit_ranged_strength_for_rules(FactionRuleSet::EMPTY, faction, faction, kind, hp, level)
}

pub fn unit_ranged_strength_for_rules(
    off: FactionRuleSet,
    faction: Faction,
    made_by: Faction,
    kind: UnitKind,
    hp: i32,
    level: u8,
) -> Breakdown {
    let mut b = Breakdown::default();
    let base = data::unit_stats_made_by_rules(off, faction, made_by, kind).ranged;
    b.add(
        format!("{} (ranged)", data::unit_name_for(made_by, kind)),
        base,
    );
    promoted(&mut b, base, level);
    if !data::ignores_wounds_rules(off, faction, kind) {
        wounded(&mut b, hp, UNIT_HP, 10);
    }
    b
}

/// The `Wounded` line of a unit's strengths (0 for units that ignore wounds).
fn wound_penalty(off: FactionRuleSet, faction: Faction, kind: UnitKind, hp: i32) -> i32 {
    if data::ignores_wounds_rules(off, faction, kind) {
        0
    } else {
        -((UNIT_HP - hp) / 10)
    }
}

/// `unit_attack_total` with every faction rule on.
pub fn unit_attack_total(faction: Faction, kind: UnitKind, hp: i32, level: u8) -> i32 {
    unit_attack_total_rules(FactionRuleSet::EMPTY, faction, faction, kind, hp, level)
}

/// `unit_attack_strength_for(..).total`, without building the labelled lines.
pub fn unit_attack_total_rules(
    off: FactionRuleSet,
    faction: Faction,
    made_by: Faction,
    kind: UnitKind,
    hp: i32,
    level: u8,
) -> i32 {
    let base = data::unit_stats_made_by_rules(off, faction, made_by, kind).strength;
    let total = base
        + crate::rules::promotion::strength_bonus(base, level)
        + gaesatae_attack(off, made_by, kind)
        + wound_penalty(off, faction, kind, hp);
    debug_assert_eq!(
        total,
        unit_attack_strength_for_rules(off, faction, made_by, kind, hp, level).total
    );
    total
}

/// `unit_defense_total` with every faction rule on.
pub fn unit_defense_total(
    faction: Faction,
    kind: UnitKind,
    hp: i32,
    terrain: Terrain,
    fortify: i32,
    level: u8,
) -> i32 {
    unit_defense_total_rules(
        FactionRuleSet::EMPTY,
        faction,
        faction,
        kind,
        hp,
        terrain,
        fortify,
        level,
    )
}

/// `unit_defense_strength_for(..).total`, without building the labelled lines.
#[allow(clippy::too_many_arguments)]
pub fn unit_defense_total_rules(
    off: FactionRuleSet,
    faction: Faction,
    made_by: Faction,
    kind: UnitKind,
    hp: i32,
    terrain: Terrain,
    fortify: i32,
    level: u8,
) -> i32 {
    let base = data::unit_stats_made_by_rules(off, faction, made_by, kind).strength;
    let total = base
        + crate::rules::promotion::strength_bonus(base, level)
        + gaesatae_defense(off, made_by, kind)
        + data::terrain_defense_rules(off, faction, terrain)
        + fortify
        + wound_penalty(off, faction, kind, hp);
    debug_assert_eq!(
        total,
        unit_defense_strength_for_rules(off, faction, made_by, kind, hp, terrain, fortify, level)
            .total
    );
    total
}

/// `unit_ranged_total` with every faction rule on.
pub fn unit_ranged_total(faction: Faction, kind: UnitKind, hp: i32, level: u8) -> i32 {
    unit_ranged_total_rules(FactionRuleSet::EMPTY, faction, faction, kind, hp, level)
}

/// `unit_ranged_strength_for(..).total`, without building the labelled lines.
pub fn unit_ranged_total_rules(
    off: FactionRuleSet,
    faction: Faction,
    made_by: Faction,
    kind: UnitKind,
    hp: i32,
    level: u8,
) -> i32 {
    let base = data::unit_stats_made_by_rules(off, faction, made_by, kind).ranged;
    let total = base
        + crate::rules::promotion::strength_bonus(base, level)
        + wound_penalty(off, faction, kind, hp);
    debug_assert_eq!(
        total,
        unit_ranged_strength_for_rules(off, faction, made_by, kind, hp, level).total
    );
    total
}

/// Meeting engagements (both sides moving into each other) use this for both sides:
/// no terrain or fortification bonuses for anyone.
pub fn unit_meeting_strength(kind: UnitKind, hp: i32) -> Breakdown {
    unit_meeting_strength_for_rules(
        FactionRuleSet::EMPTY,
        Faction::India,
        Faction::India,
        kind,
        hp,
        0,
    )
}

/// `unit_meeting_strength_for` with every faction rule on.
pub fn unit_meeting_strength_for(
    faction: Faction,
    kind: UnitKind,
    hp: i32,
    level: u8,
) -> Breakdown {
    unit_meeting_strength_for_rules(FactionRuleSet::EMPTY, faction, faction, kind, hp, level)
}

pub fn unit_meeting_strength_for_rules(
    off: FactionRuleSet,
    faction: Faction,
    made_by: Faction,
    kind: UnitKind,
    hp: i32,
    level: u8,
) -> Breakdown {
    unit_attack_strength_for_rules(off, faction, made_by, kind, hp, level)
}

pub fn city_defense_strength(population: i32, has_walls: bool, hp: i32) -> Breakdown {
    city_defense_strength_for_rules(
        FactionRuleSet::EMPTY,
        Faction::India,
        population,
        has_walls,
        hp,
        0,
    )
}

/// `city_defense_strength_for` with every faction rule on.
pub fn city_defense_strength_for(
    faction: Faction,
    population: i32,
    has_walls: bool,
    hp: i32,
    era: u8,
) -> Breakdown {
    city_defense_strength_for_rules(
        FactionRuleSet::EMPTY,
        faction,
        population,
        has_walls,
        hp,
        era,
    )
}

pub fn city_defense_strength_for_rules(
    off: FactionRuleSet,
    faction: Faction,
    population: i32,
    has_walls: bool,
    hp: i32,
    era: u8,
) -> Breakdown {
    city_defense_with_fort_rules(off, faction, population, has_walls, 0, hp, era)
}

/// `city_defense_with_fort` with every faction rule on.
pub fn city_defense_with_fort(
    faction: Faction,
    population: i32,
    has_walls: bool,
    fort: u8,
    hp: i32,
    era: u8,
) -> Breakdown {
    city_defense_with_fort_rules(
        FactionRuleSet::EMPTY,
        faction,
        population,
        has_walls,
        fort,
        hp,
        era,
    )
}

/// `fort` is the Walls tier above plain Walls: 0 none, 1 a Castle, 2 a Star Fort. Each adds its
/// own strength on top of the tier below. `era` is the owner's era (0 Ancient): every line
/// except the wounds rides the era curve (`data::era_scaled`), so a city keeps pace with the
/// units of its day (user, 2026-10-18: city strength and Walls tiers follow the same curve).
pub fn city_defense_with_fort_rules(
    off: FactionRuleSet,
    faction: Faction,
    population: i32,
    has_walls: bool,
    fort: u8,
    hp: i32,
    era: u8,
) -> Breakdown {
    city_defense_covered_rules(
        off,
        faction,
        population,
        u8::from(has_walls) + fort,
        0,
        hp,
        era,
    )
}

/// `city_defense_covered` with every faction rule on.
pub fn city_defense_covered(
    faction: Faction,
    population: i32,
    wall_tier: u8,
    covered: u8,
    hp: i32,
    era: u8,
) -> Breakdown {
    city_defense_covered_rules(
        FactionRuleSet::EMPTY,
        faction,
        population,
        wall_tier,
        covered,
        hp,
        era,
    )
}

/// A city's defence against an attacker a Siege of tier `covered` covers
/// (`wall_damage_percent`): the strength of the Walls tiers up to `covered` is left out, and
/// the higher tiers (a Castle, a Star Fort) still count. `wall_tier` as in `data::wall_tier`.
pub fn city_defense_covered_rules(
    off: FactionRuleSet,
    faction: Faction,
    population: i32,
    wall_tier: u8,
    covered: u8,
    hp: i32,
    era: u8,
) -> Breakdown {
    let mut b = Breakdown::default();
    b.add("City", data::era_scaled(CITY_BASE_STRENGTH, era));
    b.add(
        "Population",
        data::era_scaled(CITY_STRENGTH_PER_POP * population, era),
    );
    if wall_tier >= 1 && covered < 1 {
        b.add(
            data::building_name_for(faction, BuildingKind::Walls),
            data::era_scaled(
                data::building_stats_for_rules(off, faction, BuildingKind::Walls).city_strength,
                era,
            ),
        );
    }
    if wall_tier >= 2 && covered < 2 {
        b.add(
            "Castle upgrade",
            data::era_scaled(
                data::building_stats(BuildingKind::Castle).city_strength,
                era,
            ),
        );
    }
    if wall_tier >= 3 && covered < 3 {
        b.add(
            "Star Fort upgrade",
            data::era_scaled(
                data::building_stats(BuildingKind::StarFort).city_strength,
                era,
            ),
        );
    }
    wounded(&mut b, hp, CITY_HP, 20);
    b
}

/// City strike, as in Civ 6: only a walled city fires. Any Walls tier counts (a Castle upgrades
/// Walls, and the Switzerland' Customs Bastion is their Walls), and a city at 0 HP stops firing.
/// Unwalled cities rely on their garrison and HP alone. Resolution, forecasts and the AI all ask
/// this one function.
pub fn city_can_fire(walls: bool, hp: i32) -> bool {
    walls && hp > 0
}

/// Which defender an attack on a hostile city's tile damages, as in Civ 6.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CityTileDefender {
    /// The city's HP takes the attack (its defense strength retaliates against melee); damage
    /// beyond its remaining HP can reach the garrison after stage damage is pooled.
    City,
    /// The city is at 0 HP: its garrison defends with its own unit strength. Ranged fire hits
    /// it, and a melee attacker that destroys it captures the city.
    Garrison,
}

/// While a city has HP it is the target of every attack on its tile; a hit's unused share
/// past its HP carries into the garrison ([`carried_share`]). At 0 HP a garrison defends;
/// with no garrison the city itself is the target (a melee unit entering captures it; ranged
/// fire changes nothing). Each hit is decided from the stage's snapshot. Resolution (`rules::battle::ranged_target`,
/// `rules::movement`), forecasts (`forecast::forecast_attack_mode`) and the AI all ask this one
/// function; the app requests it through the WASM combat projection.
pub fn city_tile_defender(city_hp: i32, garrisoned: bool) -> CityTileDefender {
    if city_hp <= 0 && garrisoned {
        CityTileDefender::Garrison
    } else {
        CityTileDefender::City
    }
}

/// Identical math for resolution, fogged forecasts and AI target selection.
pub fn city_defense_for_rules(
    rules: crate::experiments::RuleExperiments,
    faction: Faction,
    population: i32,
    walls: bool,
    fort: u8,
    hp: i32,
    era: u8,
) -> Breakdown {
    city_defense_vs(
        rules,
        faction,
        population,
        u8::from(walls) + fort,
        0,
        hp,
        era,
    )
}

/// `city_defense_for_rules` against an attacker a Siege of tier `covered` covers
/// (`city_defense_covered`). The one defence breakdown resolution, forecasts and the AI use.
pub fn city_defense_vs(
    rules: crate::experiments::RuleExperiments,
    faction: Faction,
    population: i32,
    wall_tier: u8,
    covered: u8,
    hp: i32,
    era: u8,
) -> Breakdown {
    let mut b = city_defense_covered_rules(
        rules.faction_rules_off,
        faction,
        population,
        wall_tier,
        covered,
        hp,
        era,
    );
    if rules.frontier_cities && wall_tier == 0 {
        b.add(
            "Experimental small unwalled city",
            -4 * (4 - population).max(0),
        );
    }
    if rules.wall_masonry && wall_tier >= 1 && covered < 1 {
        b.add("Experimental wall masonry", 8);
    }
    b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outrider_applies_to_horses_but_not_vehicles_or_non_melee_attacks() {
        assert!(outrider_blocks(UnitKind::Spearman, UnitKind::Horseman, 2));
        assert!(outrider_blocks(UnitKind::Pikeman, UnitKind::Knight, 2));
        assert!(!outrider_blocks(UnitKind::Pikeman, UnitKind::Knight, 1));
        assert!(!outrider_blocks(UnitKind::Archer, UnitKind::Horseman, 2));
        assert!(!outrider_blocks(UnitKind::Horseman, UnitKind::Horseman, 2));
        assert!(!outrider_blocks(UnitKind::Infantry, UnitKind::Tank, 2));
    }

    #[test]
    fn experimental_city_levers_preserve_faction_bonuses_and_default_math() {
        use crate::experiments::RuleExperiments;
        for faction in Faction::ALL {
            for population in 1..=6 {
                for walls in [false, true] {
                    let base = city_defense_with_fort(faction, population, walls, 0, 140, 0);
                    assert_eq!(
                        city_defense_for_rules(
                            RuleExperiments::default(),
                            faction,
                            population,
                            walls,
                            0,
                            140,
                            0
                        ),
                        base
                    );
                    let rules = RuleExperiments {
                        frontier_cities: true,
                        wall_masonry: true,
                        ..Default::default()
                    };
                    let strength =
                        city_defense_for_rules(rules, faction, population, walls, 0, 140, 0);
                    let delta = if walls {
                        8
                    } else {
                        -4 * (4 - population).max(0)
                    };
                    assert_eq!(strength.total, base.total + delta);
                }
            }
        }
    }

    #[test]
    fn the_city_shields_its_garrison_until_it_falls() {
        use CityTileDefender::*;
        for hp in [1, 5, 200] {
            assert_eq!(city_tile_defender(hp, true), City);
            assert_eq!(city_tile_defender(hp, false), City);
        }
        assert_eq!(city_tile_defender(0, true), Garrison);
        assert_eq!(city_tile_defender(0, false), City);
    }

    #[test]
    fn equal_strength_deals_30() {
        assert_eq!(base_damage(20, 20), 30);
        assert_eq!(damage(20, 20, 100), 30);
    }

    #[test]
    fn stronger_deals_more_and_is_clamped() {
        assert!(base_damage(32, 20) > base_damage(20, 32));
        assert_eq!(base_damage(200, 0), 222);
        assert_eq!(base_damage(0, 200), 4);
    }

    #[test]
    fn wounds_reduce_strength() {
        assert_eq!(unit_attack_strength(UnitKind::Spearman, 100).total, 20);
        assert_eq!(unit_attack_strength(UnitKind::Spearman, 55).total, 16);
    }
}
