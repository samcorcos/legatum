//! The three-class unit lineage as the AI reads it (era overhaul, phase A2).
//!
//! The classes and their triangle live in the core (`data::unit_class`, `combat::class_bonus`).
//! Here the AI gets the lineage ladders (best unit first, so "the best melee we can build" is a
//! lookup), per-era quality, and the composition value: how much a class is worth next to
//! the enemy army we can see. The value is a score, not a rule: it is added to the unit's
//! production value like any other term, so a counter is built because it pays, not because a
//! rule says "answer horses with spears".

use legatum_core::{UnitClass, UnitKind, data};

/// Melee line, best first.
pub const MELEE_LADDER: [UnitKind; 9] = [
    UnitKind::HunterKiller,
    UnitKind::MechanisedInfantry,
    UnitKind::Infantry,
    UnitKind::TrenchInfantry,
    UnitKind::Rifleman,
    UnitKind::AssaultTrooper,
    UnitKind::Trooper,
    UnitKind::Phalangite,
    UnitKind::Soldier,
];
/// Ranged line, best first.
pub const RANGED_LADDER: [UnitKind; 9] = [
    UnitKind::Drone,
    UnitKind::RocketArtillery,
    UnitKind::Artillery,
    UnitKind::Howitzer,
    UnitKind::FieldCannon,
    UnitKind::Arquebusier,
    UnitKind::Marksman,
    UnitKind::CompositeArcher,
    UnitKind::Ranger,
];

/// Mounted line, best first.
pub const MOUNTED_LADDER: [UnitKind; 9] = [
    UnitKind::AutonomousArmour,
    UnitKind::MainBattleTank,
    UnitKind::Tank,
    UnitKind::Landship,
    UnitKind::Cavalry,
    UnitKind::Cuirassier,
    UnitKind::Knight,
    UnitKind::Cataphract,
    UnitKind::Horseman,
];

pub fn class(unit: UnitKind) -> Option<UnitClass> {
    data::unit_class(unit)
}

/// An armed unit of one of the land combat classes (not a Scout, a civilian or an aircraft: the
/// air layer lives in hangars and has its own orders).
pub fn is_combat(unit: UnitKind) -> bool {
    class(unit).is_some_and(|c| c != UnitClass::Air)
}

pub fn is_melee_line(unit: UnitKind) -> bool {
    class(unit) == Some(UnitClass::Melee)
}

/// A shooter of the ranged class.
pub fn is_ranged_line(unit: UnitKind) -> bool {
    class(unit) == Some(UnitClass::Ranged)
}

pub fn is_mounted(unit: UnitKind) -> bool {
    class(unit) == Some(UnitClass::Mounted)
}

/// Whether a unit closes in to fight (core `data::attacks_in_melee`): it charges, can take a
/// city, counts in an army's melee wing and threatens a charge. The Mongol Keshig, a horse
/// archer, is this and a shooter; for every other unit it means "has no ranged attack".
pub fn charges(unit: UnitKind) -> bool {
    data::attacks_in_melee(unit)
}

/// A shooter that never closes in (an Archer): it stays back, fires, and is never
/// the capturer. `ranged` is the unit's ranged strength for its faction. Not the Keshig.
pub fn shooter_only(unit: UnitKind, ranged: i32) -> bool {
    ranged > 0 && !charges(unit)
}

/// A unit with both attacks (the Keshig): each turn it charges or shoots, never both.
pub fn dual(unit: UnitKind, ranged: i32) -> bool {
    ranged > 0 && charges(unit)
}

/// A unit's fighting worth for comparing it with its upgrade: its stronger attack, plus half the
/// other when it has both (a second attack is an option each turn, not extra damage).
pub fn worth(unit: UnitKind, strength: i32, ranged: i32) -> i32 {
    if dual(unit, ranged) {
        strength.max(ranged) + strength.min(ranged) / 2
    } else {
        strength.max(ranged)
    }
}

/// The melee line, then the ranged line: what the Sparta and rush openings reach for.
pub fn melee_first() -> Vec<UnitKind> {
    MELEE_LADDER.into_iter().chain(RANGED_LADDER).collect()
}

/// The ranged line, then the melee line.
pub fn ranged_first() -> Vec<UnitKind> {
    RANGED_LADDER.into_iter().chain(MELEE_LADDER).collect()
}

/// The ranged line, then the cheapest melee unit.
pub fn ranged_then_soldier() -> Vec<UnitKind> {
    RANGED_LADDER
        .into_iter()
        .chain([UnitKind::Soldier])
        .collect()
}

/// Quality of a unit's generation, as an extra production value: the era of the technology that
/// unlocks it (Ancient 0, Classical 200, Medieval 250, Renaissance 400, then 150
/// more per era), so a newer generation of the class is preferred once it can be built.
pub fn quality(unit: UnitKind) -> i32 {
    match data::unit_era_tier(unit) {
        0 => 0,
        1 => 200,
        2 => 250,
        3 => 400,
        era => 400 + 150 * (i32::from(era) - 3),
    }
}

/// Whether a better unit of the same class replaces this one now: the unit's upgrade is unlocked
/// (`known`). Melee units stay buildable after their upgrade because the better one needs Iron.
pub fn obsolete(unit: UnitKind, known: impl Fn(legatum_core::TechId) -> bool) -> bool {
    if is_melee_line(unit) {
        return false;
    }
    data::unit_stats(unit)
        .upgrades_to
        .and_then(|to| data::unit_stats(to).requires)
        .is_some_and(known)
}
