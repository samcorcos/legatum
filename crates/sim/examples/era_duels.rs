//! The era duel table: one unit attacks another on open ground, both at full health, no
//! promotions or fortification, every roll at 100% (the middle of the 80-120% band), using the
//! real strength and damage functions of the core (`combat::unit_*_total`, `fight_class_bonus`,
//! `damage`). The attacker keeps attacking until a side dies or `ROUNDS` rounds pass.
//!
//!   cargo run --release -p legatum-sim --example era_duels
//!
//! Prints markdown: same-era duels per class pair, then each class's units against the units 1, 2
//! and 3 eras behind (any class), then the Medieval against the Ancient.
use legatum_core::{
    Faction, Terrain, UnitClass, UnitKind,
    combat::{self, damage},
    data,
};

const ROUNDS: u32 = 6;
const LINES: [[UnitKind; 7]; 3] = [
    [
        UnitKind::Soldier,
        UnitKind::Phalangite,
        UnitKind::Trooper,
        UnitKind::AssaultTrooper,
        UnitKind::Rifleman,
        UnitKind::Infantry,
        UnitKind::HunterKiller,
    ],
    [
        UnitKind::Ranger,
        UnitKind::CompositeArcher,
        UnitKind::Marksman,
        UnitKind::Arquebusier,
        UnitKind::FieldCannon,
        UnitKind::Artillery,
        UnitKind::Drone,
    ],
    [
        UnitKind::Horseman,
        UnitKind::Cataphract,
        UnitKind::Knight,
        UnitKind::Cuirassier,
        UnitKind::Cavalry,
        UnitKind::Tank,
        UnitKind::AutonomousArmour,
    ],
];
const CLASS_NAMES: [&str; 3] = ["melee", "ranged", "mounted"];

/// (rounds the attacker needs to kill, attacker HP left, defender HP left) with a mean roll.
fn duel(a: UnitKind, d: UnitKind) -> (Option<u32>, i32, i32) {
    let shoots = data::unit_stats(a).ranged > 0 && data::unit_class(a) == Some(UnitClass::Ranged);
    let (mut ha, mut hd) = (100, 100);
    for round in 1..=ROUNDS {
        let atk = if shoots {
            combat::unit_ranged_total(Faction::India, a, ha, 0)
        } else {
            combat::unit_attack_total(Faction::India, a, ha, 0)
        } + combat::fight_class_bonus(a, d, true, shoots).1;
        let def = combat::unit_defense_total(Faction::India, d, hd, Terrain::Plains, 0, 0)
            + combat::fight_class_bonus(d, a, false, shoots).1;
        let to_d = damage(atk, def, 100);
        let to_a = if shoots { 0 } else { damage(def, atk, 100) };
        hd -= to_d;
        if hd <= 0 {
            return (Some(round), ha, 0);
        }
        ha -= to_a;
        if ha <= 0 {
            return (None, 0, hd);
        }
    }
    (None, ha, hd)
}

fn cell(a: UnitKind, d: UnitKind) -> String {
    match duel(a, d) {
        (Some(r), ha, _) => format!("kills in {r} ({ha} HP left)"),
        (None, 0, hd) => format!("dies ({hd} HP dealt back)"),
        (None, ha, hd) => format!("stalls ({ha}/{hd})"),
    }
}

fn main() {
    println!("## Same era (attacker vs defender, attacker's view)\n");
    println!("| era | unit | vs melee | vs ranged | vs mounted |\n|---|---|---|---|---|");
    for era in 0..7 {
        for (ci, line) in LINES.iter().enumerate() {
            let a = line[era];
            let cells: Vec<String> = LINES.iter().map(|l| cell(a, l[era])).collect();
            println!(
                "| {era} | {:?} ({}) | {} |",
                a,
                CLASS_NAMES[ci],
                cells.join(" | ")
            );
        }
    }
    for behind in 1..=3usize {
        println!("\n## Attacker {behind} era(s) ahead of the defender\n");
        println!("| attacker | vs melee | vs ranged | vs mounted |\n|---|---|---|---|");
        for era in behind..7 {
            for line in LINES.iter() {
                let a = line[era];
                let cells: Vec<String> = LINES.iter().map(|l| cell(a, l[era - behind])).collect();
                println!("| {:?} (era {era}) | {} |", a, cells.join(" | "));
            }
        }
    }
    println!("\n## Strengths\n");
    println!("| era | melee | ranged str/att | mounted |\n|---|---|---|---|");
    for era in 0..7 {
        let s: Vec<_> = LINES.iter().map(|l| data::unit_stats(l[era])).collect();
        println!(
            "| {era} | {} | {}/{} | {} |",
            s[0].strength, s[1].strength, s[1].ranged, s[2].strength
        );
    }
}
