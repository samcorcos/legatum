//! The three-class unit lineage (era overhaul, phase A2): data, the class bonus, mounted moves,
//! upgrades along the lineage and the faction rules that now read the class.
use crate::combat;
use crate::data::{self, MOUNTED_MOVES};
use crate::rules::battle::{FightKind, Target, plan_fight};
use crate::rules::test_support::{city, game, hex, unit};
use crate::{forecast_attack, player_view, *};

const LINES: [(UnitClass, [UnitKind; 9]); 3] = [
    (
        UnitClass::Melee,
        [
            UnitKind::Soldier,
            UnitKind::Phalangite,
            UnitKind::Trooper,
            UnitKind::AssaultTrooper,
            UnitKind::Rifleman,
            UnitKind::TrenchInfantry,
            UnitKind::Infantry,
            UnitKind::MechanisedInfantry,
            UnitKind::HunterKiller,
        ],
    ),
    (
        UnitClass::Ranged,
        [
            UnitKind::Ranger,
            UnitKind::CompositeArcher,
            UnitKind::Marksman,
            UnitKind::Arquebusier,
            UnitKind::FieldCannon,
            UnitKind::Howitzer,
            UnitKind::Artillery,
            UnitKind::RocketArtillery,
            UnitKind::Drone,
        ],
    ),
    (
        UnitClass::Mounted,
        [
            UnitKind::Horseman,
            UnitKind::Cataphract,
            UnitKind::Knight,
            UnitKind::Cuirassier,
            UnitKind::Cavalry,
            UnitKind::Landship,
            UnitKind::Tank,
            UnitKind::MainBattleTank,
            UnitKind::AutonomousArmour,
        ],
    ),
];

#[test]
fn every_unit_has_a_class_except_scouts_and_civilians() {
    for kind in UnitKind::ALL {
        let class = data::unit_class(kind);
        let combat_unit = !matches!(kind, UnitKind::Scout | UnitKind::Settler | UnitKind::Heir);
        assert_eq!(class.is_some(), combat_unit, "{kind:?}");
    }
    for (class, line) in LINES {
        for kind in line {
            assert_eq!(data::unit_class(kind), Some(class), "{kind:?}");
        }
    }
}

#[test]
fn each_line_upgrades_one_step_at_a_time_to_a_stronger_dearer_unit_of_a_later_era() {
    for (class, line) in LINES {
        for pair in line.windows(2) {
            let (from, to) = (data::unit_stats(pair[0]), data::unit_stats(pair[1]));
            assert_eq!(from.upgrades_to, Some(pair[1]), "{:?}", pair[0]);
            assert_eq!(data::unit_class(pair[1]), Some(class));
            let power = |s: &data::UnitStats| s.strength.max(s.ranged);
            assert!(power(&to) > power(&from), "{:?}", pair[1]);
            assert!(to.cost > from.cost, "{:?}", pair[1]);
            assert!(
                data::unit_era_tier(pair[1]) > data::unit_era_tier(pair[0]),
                "{:?} must come in a later era",
                pair[1]
            );
        }
        assert_eq!(
            data::unit_stats(*line.last().unwrap()).upgrades_to,
            None,
            "{class:?} ends its line"
        );
    }
}

#[test]
fn every_class_unit_unlocks_with_a_technology_of_its_own_era_or_from_the_start() {
    let from_the_start = [UnitKind::Soldier, UnitKind::Ranger, UnitKind::Horseman];
    for kind in UnitKind::ALL {
        if data::unit_class(kind).is_none() {
            continue;
        }
        let requires = data::unit_stats(kind).requires;
        assert_eq!(
            requires.is_none(),
            from_the_start.contains(&kind),
            "{kind:?}"
        );
    }
    // Spearman, Archer and Horseman are the triangle from turn one (era overhaul proposal).
    assert_eq!(data::unit_era_tier(UnitKind::Horseman), 0);
    assert_eq!(data::unit_era_tier(UnitKind::Knight), 2);
    assert_eq!(data::unit_era_tier(UnitKind::AssaultTrooper), 3);
}

#[test]
fn mounted_units_have_four_moves_the_armour_five_and_six_and_nobody_else_does() {
    for kind in UnitKind::ALL {
        let moves = data::unit_stats(kind).moves;
        if kind == UnitKind::Tank || kind == UnitKind::MainBattleTank {
            assert_eq!(moves, data::TANK_MOVES, "{kind:?}");
        } else if kind == UnitKind::Landship {
            assert_eq!(moves, data::LANDSHIP_MOVES, "{kind:?}");
        } else if kind == UnitKind::AutonomousArmour {
            assert_eq!(moves, data::AUTONOMOUS_ARMOUR_MOVES, "{kind:?}");
        } else if data::unit_class(kind) == Some(UnitClass::Mounted) {
            assert_eq!(moves, MOUNTED_MOVES, "{kind:?}");
        } else {
            assert!(moves < MOUNTED_MOVES, "{kind:?}");
        }
    }
}

#[test]
fn historical_names() {
    let names: Vec<_> = [
        UnitKind::Soldier,
        UnitKind::Ranger,
        UnitKind::Horseman,
        UnitKind::Trooper,
        UnitKind::Marksman,
        UnitKind::Knight,
        UnitKind::AssaultTrooper,
        UnitKind::Rifleman,
        UnitKind::FieldCannon,
        UnitKind::Cavalry,
        UnitKind::Infantry,
        UnitKind::Artillery,
        UnitKind::Tank,
        UnitKind::HunterKiller,
        UnitKind::Drone,
        UnitKind::AutonomousArmour,
    ]
    .map(data::unit_name)
    .to_vec();
    assert_eq!(
        names,
        [
            "Spearman",
            "Archer",
            "Horseman",
            "Pikeman",
            "Crossbowman",
            "Knight",
            "Musketman",
            "Rifleman",
            "Field Cannon",
            "Cavalry",
            "Infantry",
            "Artillery",
            "Tank",
            "Hunter-Killer",
            "Combat Drone",
            "Autonomous Armour",
        ]
    );
}

#[test]
fn the_class_bonus_is_the_triangle_and_nothing_else() {
    use UnitKind::*;
    let bonus = |a, b| combat::class_bonus(a, b).1;
    // A percentage of the unit's own base strength (user, 2026-10-31), so it bites the same in
    // every era.
    let pct = |k: UnitKind| data::unit_stats(k).strength * combat::CLASS_BONUS_PERCENT / 100;
    // Before the Modern era (user, 2026-10-20: the triangle ends there); Infantry, Tank and
    // Artillery onward have none (`the_triangle_ends_at_the_modern_era`).
    let melee = [Soldier, Trooper, AssaultTrooper, Rifleman];
    let ranged = [Ranger, Marksman, FieldCannon];
    let mounted = [Horseman, Knight, Cavalry];
    for m in melee {
        for h in mounted {
            assert_eq!(bonus(m, h), pct(m), "{m:?} vs {h:?}");
            assert_eq!(bonus(h, m), 0, "{h:?} vs {m:?}");
        }
        for r in ranged {
            assert_eq!(bonus(m, r), 0);
            assert_eq!(bonus(r, m), 0);
        }
    }
    for h in mounted {
        for r in ranged {
            assert_eq!(bonus(h, r), pct(h), "{h:?} vs {r:?}");
            assert_eq!(bonus(r, h), 0);
        }
        assert_eq!(bonus(h, Scout), 0);
        assert_eq!(bonus(h, h), 0);
    }
    for modern in [
        Infantry,
        HunterKiller,
        Tank,
        AutonomousArmour,
        Artillery,
        Drone,
    ] {
        for other in [
            Soldier,
            Horseman,
            Cavalry,
            Ranger,
            FieldCannon,
            Tank,
            Infantry,
        ] {
            assert_eq!(bonus(modern, other), 0, "{modern:?} vs {other:?}");
            assert_eq!(bonus(other, modern), 0, "{other:?} vs {modern:?}");
        }
    }
    assert_eq!(bonus(Soldier, Soldier), 0);
    assert_eq!(bonus(Scout, Horseman), 0);
}

#[test]
fn the_class_bonus_lines_state_the_percentage() {
    let pct = format!("(+{}%)", combat::CLASS_BONUS_PERCENT);
    assert!(combat::MELEE_VS_MOUNTED_LABEL.ends_with(&pct));
    assert!(combat::MOUNTED_VS_RANGED_LABEL.ends_with(&pct));
}

#[test]
fn pre_modern_riders_stand_well_above_their_eras_melee_line() {
    // User, 2026-10-31 (the Civ 6 way): a rider is about 30% stronger than the same era's spear
    // line and costs more; the spear's class bonus evens the fight rather than making it a
    // massacre. The Tank and the Autonomous Armour, past the triangle, keep their own numbers.
    use UnitKind::*;
    for (melee, rider) in [
        (Soldier, Horseman),
        (Phalangite, Cataphract),
        (Trooper, Knight),
        (AssaultTrooper, Cuirassier),
        (Rifleman, Cavalry),
    ] {
        let (m, h) = (data::unit_stats(melee), data::unit_stats(rider));
        let ratio = h.strength * 100 / m.strength;
        assert!(
            (data::MOUNTED_STRENGTH_PERCENT - 3..=data::MOUNTED_STRENGTH_PERCENT + 3)
                .contains(&ratio),
            "{rider:?} {} against {melee:?} {}",
            h.strength,
            m.strength
        );
        assert!(h.cost > m.cost, "{rider:?} costs more than {melee:?}");
        // The spear line's bonus (a percentage of its own strength) leaves it ahead, not crushing.
        let spear = m.strength + combat::class_bonus(melee, rider).1;
        assert!(
            spear > h.strength && spear - h.strength <= m.strength / 5,
            "{melee:?} {spear} vs {rider:?} {}",
            h.strength
        );
    }
    assert_eq!(data::design_stats(Tank).strength, 53);
    assert_eq!(data::design_stats(AutonomousArmour).strength, 60);
}

#[test]
fn a_rider_storms_a_city_a_little_below_the_melee_lines_strength() {
    let mut s = game(&[0, 1]);
    let c = city(&mut s, 1, hex(5, 4), 3);
    let horse = unit(&mut s, 0, UnitKind::Horseman, hex(4, 4));
    let fight = plan_fight(&s, 1, horse, Target::City(c), FightKind::Melee);
    let penalty =
        data::unit_stats(UnitKind::Horseman).strength * combat::MOUNTED_CITY_PENALTY_PERCENT / 100;
    assert!(
        has(
            &fight.attacker_strength.parts,
            combat::MOUNTED_CITY_LABEL,
            -penalty
        ),
        "{:?}",
        fight.attacker_strength.parts
    );
    let spear = data::unit_stats(UnitKind::Soldier).strength;
    assert!(
        fight.attacker_strength.total < spear && fight.attacker_strength.total >= spear - 3,
        "{}",
        fight.attacker_strength.total
    );
    assert!(
        combat::MOUNTED_CITY_LABEL
            .ends_with(&format!("(-{}%)", combat::MOUNTED_CITY_PENALTY_PERCENT))
    );
    // The forecast shows the same line.
    let f = forecast_attack(&player_view(&s, PlayerId(0)), horse, hex(5, 4)).unwrap();
    assert_eq!(f.attacker_strength.parts, fight.attacker_strength.parts);
    // A spear line, a shot and the Tank onward pay nothing.
    assert_eq!(
        combat::mounted_city_penalty(Faction::India, UnitKind::Soldier, false),
        0
    );
    assert_eq!(
        combat::mounted_city_penalty(Faction::Mongols, data::KESHIG_KIND, true),
        0
    );
    assert_eq!(
        combat::mounted_city_penalty(Faction::India, UnitKind::Tank, false),
        0
    );
}

#[test]
fn india_rides_the_generic_mounted_rules() {
    // India has no mounted exemption or modifier: its riders are the base line, can be built,
    // and pay the same city-charge penalty as everyone's. The Knight is the exception: it is the
    // War Elephant (`tests/special_units.rs`).
    use UnitKind::*;
    for kind in [Horseman, Cataphract, Cuirassier, Cavalry] {
        let (own, base) = (
            data::unit_stats_for(Faction::India, kind),
            data::unit_stats(kind),
        );
        assert_eq!(
            (own.strength, own.moves, own.cost),
            (base.strength, base.moves, base.cost),
            "{kind:?}"
        );
        assert_eq!(
            data::faction_unavailable_reason(ProductionItem::Unit { unit: kind }, Faction::India),
            None,
            "{kind:?}"
        );
        let penalty = combat::mounted_city_penalty(Faction::India, kind, false);
        assert_eq!(
            penalty,
            combat::mounted_city_penalty(Faction::Athens, kind, false),
            "{kind:?}"
        );
        assert!(penalty > 0, "{kind:?}");
    }
}

type Parts = Vec<(String, i32)>;

fn strengths(s: &GameState, a: UnitId, d: UnitId, kind: FightKind) -> (Parts, Parts) {
    let fight = plan_fight(s, 1, a, Target::Unit(d), kind);
    (fight.attacker_strength.parts, fight.defender_strength.parts)
}

fn has(parts: &[(String, i32)], label: &str, value: i32) -> bool {
    parts.iter().any(|(l, v)| l == label && *v == value)
}

#[test]
fn a_spear_line_unit_gets_the_bonus_attacking_and_defending_against_a_horseman() {
    let mut s = game(&[0, 1]);
    let spear = unit(&mut s, 0, UnitKind::Soldier, hex(4, 4));
    let horse = unit(&mut s, 1, UnitKind::Horseman, hex(5, 4));
    let spear_bonus = combat::class_bonus_strength(data::unit_stats(UnitKind::Soldier).strength);
    assert_eq!(spear_bonus, 8, "40% of the Spearman's 20");
    let (att, def) = strengths(&s, spear, horse, FightKind::Melee);
    assert!(
        has(&att, combat::MELEE_VS_MOUNTED_LABEL, spear_bonus),
        "{att:?}"
    );
    assert!(
        def.iter()
            .all(|(l, _)| l != combat::MOUNTED_VS_RANGED_LABEL)
    );
    // The charge: the horseman attacks, the spear line defends with the same bonus.
    let (att, def) = strengths(&s, horse, spear, FightKind::Melee);
    assert!(
        has(&def, combat::MELEE_VS_MOUNTED_LABEL, spear_bonus),
        "{def:?}"
    );
    assert!(att.iter().all(|(l, _)| !l.contains("against")), "{att:?}");
    // A meeting engagement gives each side its own bonus.
    let (att, def) = strengths(&s, spear, horse, FightKind::Meeting);
    assert!(has(&att, combat::MELEE_VS_MOUNTED_LABEL, spear_bonus));
    assert!(def.iter().all(|(l, _)| !l.contains("against")));
}

#[test]
fn a_horseman_gets_the_bonus_charging_archers_and_none_under_their_fire() {
    let mut s = game(&[0, 1]);
    let horse = unit(&mut s, 0, UnitKind::Horseman, hex(4, 4));
    let archer = unit(&mut s, 1, UnitKind::Ranger, hex(5, 4));
    let horse_bonus = combat::class_bonus_strength(data::unit_stats(UnitKind::Horseman).strength);
    assert_eq!(horse_bonus, 10, "40% of the Horseman's 26");
    let (att, _) = strengths(&s, horse, archer, FightKind::Melee);
    assert!(
        has(&att, combat::MOUNTED_VS_RANGED_LABEL, horse_bonus),
        "{att:?}"
    );
    // Under fire from an archer the horseman defends without it (designer, 2026-11-10).
    let (att, def) = strengths(&s, archer, horse, FightKind::Ranged);
    assert!(
        def.iter()
            .all(|(l, _)| l != combat::MOUNTED_VS_RANGED_LABEL),
        "{def:?}"
    );
    assert!(att.iter().all(|(l, _)| !l.contains("against")), "{att:?}");
}

#[test]
fn ranged_has_no_class_bonus_against_the_melee_line_and_melee_has_none_against_ranged() {
    let mut s = game(&[0, 1]);
    let archer = unit(&mut s, 0, UnitKind::Ranger, hex(3, 4));
    let spear = unit(&mut s, 1, UnitKind::Soldier, hex(5, 4));
    let (att, def) = strengths(&s, archer, spear, FightKind::Ranged);
    assert!(att.iter().chain(&def).all(|(l, _)| !l.contains("against")));
    let (att, def) = strengths(&s, spear, archer, FightKind::Melee);
    assert!(att.iter().chain(&def).all(|(l, _)| !l.contains("against")));
}

#[test]
fn a_mounted_unit_under_fire_gets_no_bonus_but_keeps_it_charging() {
    let mut s = game(&[0, 1]);
    let cannon = unit(&mut s, 0, UnitKind::FieldCannon, hex(4, 4));
    let cavalry = unit(&mut s, 1, UnitKind::Cavalry, hex(5, 4));
    let bonus = combat::class_bonus_strength(data::unit_stats(UnitKind::Cavalry).strength);
    // Under fire the Cavalry defends without the bonus.
    let (att, def) = strengths(&s, cannon, cavalry, FightKind::Ranged);
    assert!(
        def.iter()
            .all(|(l, _)| l != combat::MOUNTED_VS_RANGED_LABEL),
        "{def:?}"
    );
    assert!(att.iter().all(|(l, _)| !l.contains("against")), "{att:?}");
    let view = player_view(&s, PlayerId(0));
    let f = forecast_attack(&view, cannon, hex(5, 4)).unwrap();
    let fight = plan_fight(&s, 1, cannon, Target::Unit(cavalry), FightKind::Ranged);
    assert_eq!(f.attacker_strength.parts, fight.attacker_strength.parts);
    assert_eq!(f.defender_strength.parts, fight.defender_strength.parts);
    assert_eq!(f.defender_strength.total, fight.defender_strength.total);
    // The charge keeps the bonus, in an assault and in a meeting engagement.
    let (att, _) = strengths(&s, cavalry, cannon, FightKind::Melee);
    assert!(has(&att, combat::MOUNTED_VS_RANGED_LABEL, bonus), "{att:?}");
    let (_, def) = strengths(&s, cannon, cavalry, FightKind::Meeting);
    assert!(has(&def, combat::MOUNTED_VS_RANGED_LABEL, bonus), "{def:?}");
}

#[test]
fn a_mounted_archers_shot_gives_a_mounted_target_no_bonus_and_the_spear_line_keeps_its_own() {
    let mut s = game(&[0, 1]);
    let keshig = unit(&mut s, 0, data::KESHIG_KIND, hex(3, 4));
    let horse = unit(&mut s, 1, UnitKind::Horseman, hex(5, 4));
    let spear = unit(&mut s, 1, UnitKind::Soldier, hex(5, 5));
    let (_, def) = strengths(&s, keshig, horse, FightKind::Ranged);
    assert!(
        def.iter()
            .all(|(l, _)| l != combat::MOUNTED_VS_RANGED_LABEL),
        "{def:?}"
    );
    let spear_bonus = combat::class_bonus_strength(data::unit_stats(UnitKind::Soldier).strength);
    let (_, def) = strengths(&s, keshig, spear, FightKind::Melee);
    assert!(
        has(&def, combat::MELEE_VS_MOUNTED_LABEL, spear_bonus),
        "{def:?}"
    );
}

#[test]
fn the_ai_estimates_give_a_mounted_target_of_a_shot_no_bonus() {
    let (horse, cannon) = (UnitKind::Horseman, UnitKind::FieldCannon);
    let bonus = combat::class_bonus_strength(data::unit_stats(horse).strength);
    assert_eq!(combat::fight_class_bonus(horse, cannon, false, true).1, 0);
    assert_eq!(
        combat::fight_class_bonus(horse, cannon, true, false).1,
        bonus
    );
}

#[test]
fn the_forecast_shows_the_same_class_bonus_as_the_fight() {
    let mut s = game(&[0, 1]);
    let spear = unit(&mut s, 0, UnitKind::Soldier, hex(4, 4));
    let horse = unit(&mut s, 1, UnitKind::Horseman, hex(5, 4));
    let view = player_view(&s, PlayerId(0));
    let f = forecast_attack(&view, spear, hex(5, 4)).unwrap();
    let fight = plan_fight(&s, 1, spear, Target::Unit(horse), FightKind::Melee);
    assert_eq!(f.attacker_strength.parts, fight.attacker_strength.parts);
    assert_eq!(f.defender_strength.parts, fight.defender_strength.parts);
    assert!(has(
        &f.attacker_strength.parts,
        combat::MELEE_VS_MOUNTED_LABEL,
        combat::class_bonus_strength(data::unit_stats(UnitKind::Soldier).strength)
    ));
}

#[test]
fn faction_rules_follow_the_class_through_the_lineage() {
    // Sparta: the whole melee line ignores wound penalties; mounted and ranged do not.
    for kind in UnitKind::ALL {
        let melee = data::unit_class(kind) == Some(UnitClass::Melee);
        assert_eq!(
            data::ignores_wounds(Faction::Sparta, kind),
            melee,
            "{kind:?}"
        );
        assert!(!data::ignores_wounds(Faction::Switzerland, kind));
    }
    // Celts: the melee line is ordinary (the -2 design strength was removed 2026-11-02);
    // Byzantium: sturdier veterans, not Spearmen. The +4 is a design number: it rides the era
    // curve with the unit.
    for kind in [UnitKind::Soldier, UnitKind::Trooper, UnitKind::Infantry] {
        let (c, o) = (
            data::unit_stats_for(Faction::Celts, kind),
            data::unit_stats_for(Faction::Switzerland, kind),
        );
        assert_eq!((c.strength, c.cost), (o.strength, o.cost), "{kind:?}");
    }
    assert_eq!(
        data::unit_stats_for(Faction::Byzantium, UnitKind::Soldier).strength,
        data::unit_stats(UnitKind::Soldier).strength
    );
    assert_eq!(
        data::unit_stats_for(Faction::Byzantium, UnitKind::Rifleman).strength,
        data::era_scaled(44 + 4, 4)
    );
    // Germany moves normally; Mongols move one faster.
    assert_eq!(
        data::unit_stats_for(Faction::Germany, UnitKind::Horseman).moves,
        MOUNTED_MOVES
    );
    assert_eq!(
        data::unit_stats_for(Faction::Mongols, UnitKind::Horseman).moves,
        MOUNTED_MOVES + 1
    );
    // Mongols builds no Walls before the Medieval era (user, 2026-10-19) and buys no Siege
    // before it; from then on both.
    let walls = ProductionItem::Building {
        building: BuildingKind::Walls,
    };
    assert!(!data::available_to_player(Faction::Mongols, 1, walls));
    assert!(data::available_to_player(
        Faction::Mongols,
        data::MONGOL_WALLS_ERA,
        walls
    ));
    assert!(!data::siege_available(Faction::Mongols, 1));
    assert!(data::siege_available(
        Faction::Mongols,
        data::MONGOL_WALLS_ERA
    ));
    assert!(data::siege_available(Faction::Switzerland, 0));
    assert_eq!(
        data::player_unavailable_reason(walls, Faction::Mongols, 1).as_deref(),
        Some("The Mongols raise Walls from the Medieval era")
    );
    assert!(data::available_to_faction(
        Faction::Mongols,
        ProductionItem::Unit {
            unit: UnitKind::Horseman
        }
    ));
}

#[test]
fn a_horseman_covers_four_tiles_in_a_turn_and_a_spearman_two() {
    use crate::rules::test_support::{no_orders, resolve_units};
    let mut s = game(&[0, 1]);
    let horse = unit(&mut s, 0, UnitKind::Horseman, hex(1, 4));
    let spear = unit(&mut s, 0, UnitKind::Soldier, hex(1, 6));
    crate::rules::test_support::goto(&mut s, horse, hex(9, 4));
    crate::rules::test_support::goto(&mut s, spear, hex(9, 6));
    let (next, _) = resolve_units(&s, &no_orders());
    assert_eq!(next.units[&horse].pos.distance(hex(1, 4)), 4);
    assert_eq!(next.units[&spear].pos.distance(hex(1, 6)), 2);
}

#[test]
fn every_class_has_exactly_one_unit_in_every_era_with_growing_steps() {
    // The era-by-era lineage (user, 2026-10-10): the unit of era N is unlocked by a technology
    // of era N. Strength rises by a percentage every era (user, 2026-10-18), so the steps start
    // small and each one is bigger than the last.
    for (class, line) in LINES {
        let tiers: Vec<u8> = line.iter().map(|k| data::unit_era_tier(*k)).collect();
        assert_eq!(tiers, vec![0, 1, 2, 3, 4, 5, 6, 7, 8], "{class:?}");
        let power = |k: UnitKind| {
            let s = data::unit_stats(k);
            if s.ranged > 0 { s.ranged } else { s.strength }
        };
        let steps: Vec<i32> = line.windows(2).map(|w| power(w[1]) - power(w[0])).collect();
        assert!(steps.iter().all(|s| *s > 0), "{class:?} {steps:?}");
        // The steps grow up to the Industrial era. The Machine Age (2026-11-14) splits the old
        // Industrial-to-Modern step in two, so those two halves are each smaller; pre-Modern
        // riders stand well above their era's melee line (2026-10-31) while the armour keeps its
        // own numbers, so the mounted line is checked only up to the Cavalry.
        assert!(
            steps[..4].windows(2).all(|w| w[1] >= w[0]),
            "{class:?} strength steps do not grow: {steps:?}"
        );
        // Industrial to Modern, through the Machine Age, is still a big step (the armour's own
        // numbers make the mounted one small: Cavalry 79, Landship 86, Tank 94).
        assert!(
            class == UnitClass::Mounted || steps[4] + steps[5] >= steps[3],
            "{class:?} the Machine Age halves are too small: {steps:?}"
        );
        assert!(
            steps[6] >= 2 * steps[0] && steps[7] >= steps[6],
            "{class:?} late steps are not big: {steps:?}"
        );
    }
}

#[test]
fn a_unit_costs_about_the_same_more_each_era() {
    for (class, line) in LINES {
        for w in line.windows(3) {
            let (a, b, c) = (
                data::unit_stats(w[0]).cost as f64,
                data::unit_stats(w[1]).cost as f64,
                data::unit_stats(w[2]).cost as f64,
            );
            // The Machine Age sits between Industrial and Modern, so the steps on either side of
            // it are about the square root of the old ratio (1.17 for the melee line).
            let (r1, r2) = (b / a, c / b);
            assert!(
                (1.1..1.6).contains(&r1) && (1.1..1.6).contains(&r2),
                "{class:?} {:?}: cost ratios {r1:.2}, {r2:.2}",
                w[1]
            );
        }
    }
}

#[test]
fn every_era_makes_the_melee_line_a_growing_percentage_stronger() {
    // User, 2026-10-18 (playtest 8.9.4): "each era should be thought of as a percentage increase";
    // 2026-10-19: the step grows, 25, 30, 35, 40, 45, 50 percent.
    let melee = LINES[0].1.map(|k| data::unit_stats(k).strength);
    assert_eq!(melee, data::MELEE_LINE);
    // 2026-11-14: the Machine Age splits the 45% Industrial-to-World War step into 21% and 20%,
    // and the Modern era adds a 20% step before the Future's 50%.
    assert_eq!(melee, [20, 25, 32, 44, 61, 74, 89, 107, 161]);
    assert_eq!(data::ERA_STEP_PERCENT, [25, 30, 35, 40, 21, 20, 20, 50]);
    for (w, step) in melee.windows(2).zip(data::ERA_STEP_PERCENT) {
        let grown = i64::from(w[0]) * i64::from(100 + step);
        assert!(
            (i64::from(w[1]) * 100 - grown).abs() <= 100,
            "{} -> {} is not +{step}%",
            w[0],
            w[1],
        );
    }
    // Same era: a class keeps its place against the melee line of its day.
    for (_, line) in LINES {
        assert_eq!(data::unit_era_tier(line[0]), 0);
    }
}

#[test]
fn later_eras_jump_harder_and_two_eras_ahead_is_decisive() {
    // Small steps early, big steps late (user, 2026-10-18): from the Renaissance on, one era
    // ahead deals at least twice what it takes back, two eras ahead at least four times, three
    // eras ahead at least ten times. The first steps stay small (the rush balance).
    let melee = data::MELEE_LINE;
    let ratio = |new: usize, old: usize| {
        f64::from(combat::base_damage(melee[new], melee[old]))
            / f64::from(combat::base_damage(melee[old], melee[new]))
    };
    assert!(
        ratio(1, 0) < 2.0,
        "the first step is small: {}",
        ratio(1, 0)
    );
    for era in 3..7 {
        assert!(ratio(era, era - 1) >= 2.0, "era {era} over {}", era - 1);
        assert!(ratio(era, era - 2) >= 4.0, "era {era} over {}", era - 2);
    }
    for era in 4..7 {
        assert!(ratio(era, era - 3) >= 10.0, "era {era} over {}", era - 3);
    }
}

#[test]
fn a_citys_lines_ride_the_same_curve() {
    let strength =
        |era| combat::city_defense_with_fort(Faction::India, 4, true, 2, data::CITY_HP, era).total;
    assert!(strength(0) < strength(3) && strength(3) < strength(5));
    // Era 0 is the design numbers: 10 + 2 * 4 + 10 + 5 + 5.
    assert_eq!(strength(0), 38);
    assert_eq!(
        strength(5),
        data::era_scaled(10, 5)
            + data::era_scaled(8, 5)
            + data::era_scaled(10, 5)
            + data::era_scaled(5, 5) * 2
    );
}

#[test]
fn the_triangle_ends_at_the_modern_era() {
    // User, 2026-10-20: a class bonus applies only when both units are from before the Modern
    // era; from then on units differ by strength, cost and moves.
    let pct = |k: UnitKind| combat::class_bonus_strength(data::unit_stats(k).strength);
    let melee = combat::MELEE_VS_MOUNTED_LABEL;
    let mounted = combat::MOUNTED_VS_RANGED_LABEL;
    let mut s = game(&[0, 1]);
    let infantry = unit(&mut s, 0, UnitKind::Infantry, hex(4, 4));
    let tank = unit(&mut s, 1, UnitKind::Tank, hex(5, 4));
    let rifle = unit(&mut s, 0, UnitKind::Rifleman, hex(4, 6));
    let cavalry = unit(&mut s, 1, UnitKind::Cavalry, hex(5, 6));
    let artillery = unit(&mut s, 1, UnitKind::Artillery, hex(5, 8));
    let cannon = unit(&mut s, 1, UnitKind::FieldCannon, hex(5, 9));
    let charger = unit(&mut s, 0, UnitKind::Cavalry, hex(4, 8));
    let tank_a = unit(&mut s, 0, UnitKind::Tank, hex(4, 9));
    let none = |parts: &[(String, i32)]| parts.iter().all(|(l, _)| !l.contains("against"));

    // Modern or later on either side: no bonus, attacking or defending, and no forecast line.
    for (a, d) in [(infantry, tank), (infantry, cavalry), (rifle, tank)] {
        let (att, def) = strengths(&s, a, d, FightKind::Melee);
        assert!(none(&att) && none(&def), "{att:?} {def:?}");
        let (att, def) = strengths(&s, d, a, FightKind::Melee);
        assert!(none(&att) && none(&def), "{att:?} {def:?}");
    }
    for (a, d) in [(tank_a, artillery), (charger, artillery)] {
        let (att, def) = strengths(&s, a, d, FightKind::Melee);
        assert!(none(&att) && none(&def), "{att:?} {def:?}");
    }
    // Before the Modern era the triangle is whole, in a fight and in the forecast.
    let (att, _) = strengths(&s, rifle, cavalry, FightKind::Melee);
    assert!(has(&att, melee, pct(UnitKind::Rifleman)), "{att:?}");
    let (_, def) = strengths(&s, cavalry, rifle, FightKind::Melee);
    assert!(has(&def, melee, pct(UnitKind::Rifleman)), "{def:?}");
    let (att, _) = strengths(&s, charger, cannon, FightKind::Melee);
    assert!(has(&att, mounted, pct(UnitKind::Cavalry)), "{att:?}");
    let f = forecast_attack(&player_view(&s, PlayerId(0)), rifle, hex(5, 6)).unwrap();
    let fight = plan_fight(&s, 1, rifle, Target::Unit(cavalry), FightKind::Melee);
    assert_eq!(f.attacker_strength.parts, fight.attacker_strength.parts);
    let f = forecast_attack(&player_view(&s, PlayerId(0)), infantry, hex(5, 4)).unwrap();
    assert!(none(&f.attacker_strength.parts) && none(&f.defender_strength.parts));
    // The kind-level bonus agrees.
    assert_eq!(combat::class_bonus(UnitKind::Infantry, UnitKind::Tank).1, 0);
    assert_eq!(
        combat::class_bonus(UnitKind::Tank, UnitKind::Artillery).1,
        0
    );
    assert_eq!(
        combat::class_bonus(UnitKind::Rifleman, UnitKind::Cavalry).1,
        pct(UnitKind::Rifleman)
    );
}

#[test]
fn the_tank_has_five_moves_the_autonomous_armour_six_and_the_rest_of_the_mounted_class_four() {
    assert_eq!(data::unit_stats(UnitKind::Tank).moves, data::TANK_MOVES);
    assert_eq!(data::TANK_MOVES, 5);
    assert_eq!(
        data::unit_stats(UnitKind::AutonomousArmour).moves,
        data::AUTONOMOUS_ARMOUR_MOVES
    );
    const { assert!(data::AUTONOMOUS_ARMOUR_MOVES > data::TANK_MOVES) };
    assert_eq!(
        data::unit_stats(UnitKind::Landship).moves,
        data::LANDSHIP_MOVES
    );
    const { assert!(data::LANDSHIP_MOVES < MOUNTED_MOVES) };
    for kind in LINES[2].1 {
        if !matches!(
            kind,
            UnitKind::Landship
                | UnitKind::Tank
                | UnitKind::MainBattleTank
                | UnitKind::AutonomousArmour
        ) {
            assert_eq!(data::unit_stats(kind).moves, MOUNTED_MOVES, "{kind:?}");
        }
    }
}

#[test]
fn germany_uses_ordinary_movement_from_turn_one() {
    let mut s = game(&[0, 1]);
    s.player_mut(PlayerId(0)).faction = Faction::Germany;
    let mounted = crate::game::spawn_unit(&mut s, PlayerId(0), UnitKind::Horseman, hex(4, 4));
    let settler = crate::game::spawn_unit(&mut s, PlayerId(0), UnitKind::Settler, hex(5, 4));
    assert_eq!(
        s.unit_moves_for(PlayerId(0), UnitKind::Horseman),
        MOUNTED_MOVES
    );
    assert_eq!(s.unit_moves_for(PlayerId(0), UnitKind::Settler), 2);
    crate::rules::units::reset_turn(&mut s);
    assert_eq!(s.units[&mounted].moves_left, MOUNTED_MOVES);
    assert_eq!(s.units[&settler].moves_left, 2);
    let view = player_view(&s, PlayerId(0));
    assert_eq!(
        view.units
            .iter()
            .find(|u| u.id == mounted)
            .unwrap()
            .max_moves,
        MOUNTED_MOVES
    );
    assert_eq!(
        view.units
            .iter()
            .find(|u| u.id == settler)
            .unwrap()
            .max_moves,
        2
    );
}

#[test]
fn old_save_with_siege_absorbed_still_loads_without_unlocking_siege() {
    let mut s = game(&[0, 1]);
    s.player_mut(PlayerId(0)).faction = Faction::Mongols;
    let mut saved = serde_json::to_value(&s).unwrap();
    saved["players"][0]["siege_absorbed"] = serde_json::json!(true);
    let loaded: GameState = serde_json::from_value(saved).unwrap();
    assert_eq!(loaded.player(PlayerId(0)).faction, Faction::Mongols);
    assert!(!data::siege_available(
        Faction::Mongols,
        loaded.era_of(PlayerId(0))
    ));
}
