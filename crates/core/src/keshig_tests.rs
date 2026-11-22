//! The Keshig, the Mongol horse archer (user, 2026-10-17): the Mongols' Classical mounted unit
//! (the Cataphract's slot), a mounted unit that can also shoot. One order a turn: a charge (a move onto the enemy) or a shot (a
//! `RangedAttack`), never both, and a shot never moves it.
use crate::combat;
use crate::data::{self, KESHIG_KIND, KESHIG_RANGE, KESHIG_RANGED};
use crate::events::Event;
use crate::orders::Order;
use crate::rules::battle::{FightKind, Target, plan_fight};
use crate::rules::test_support::{city, game, hex, orders, resolve_units, unit};
use crate::*;

fn mongols(s: &mut GameState, player: u8) {
    s.player_mut(PlayerId(player)).faction = Faction::Mongols;
}

fn parts(s: &GameState, a: UnitId, d: UnitId, kind: FightKind) -> (Vec<String>, Vec<String>) {
    let f = plan_fight(s, 1, a, Target::Unit(d), kind);
    let labels = |b: &Breakdown| b.parts.iter().map(|(l, _)| l.clone()).collect();
    (labels(&f.attacker_strength), labels(&f.defender_strength))
}

#[test]
fn the_keshig_is_a_classical_cataphract_with_a_bow_and_the_mongol_horseman_is_ordinary() {
    let k = data::unit_stats_for(Faction::Mongols, KESHIG_KIND);
    let c = data::unit_stats(UnitKind::Cataphract);
    assert_eq!(
        KESHIG_KIND,
        UnitKind::Cataphract,
        "the Classical mounted slot"
    );
    assert_eq!(
        (k.strength, k.ranged, k.range),
        (
            c.strength,
            data::era_scaled(KESHIG_RANGED, data::unit_era_tier(KESHIG_KIND)),
            KESHIG_RANGE
        )
    );
    assert_eq!(k.requires, Some(TechId::IronWorking));
    assert_eq!(k.moves, data::MOUNTED_MOVES + 1, "the Mongol +1 move");
    assert!(k.cost > c.cost, "the bow costs Production");
    assert_eq!(k.upgrades_to, Some(UnitKind::Knight));
    assert_eq!(
        data::unit_stats(UnitKind::Horseman).upgrades_to,
        Some(KESHIG_KIND),
        "Mongol Horsemen upgrade into Keshigs"
    );
    assert_eq!(data::unit_name_for(Faction::Mongols, KESHIG_KIND), "Keshig");
    assert_eq!(data::unit_class(KESHIG_KIND), Some(UnitClass::Mounted));
    // The Ancient Mongol Horseman is ordinary (plus the Mongol move), the Knight the Keshig
    // upgrades to has no bow, and nobody else's Cataphract shoots.
    let h = data::unit_stats_for(Faction::Mongols, UnitKind::Horseman);
    let base = data::unit_stats(UnitKind::Horseman);
    assert_eq!(
        (h.cost, h.strength, h.ranged),
        (base.cost, base.strength, 0)
    );
    assert_eq!(
        data::unit_name_for(Faction::Mongols, UnitKind::Horseman),
        "Horseman"
    );
    assert_eq!(
        data::unit_stats_for(Faction::Mongols, UnitKind::Knight).ranged,
        0
    );
    for f in Faction::ALL {
        if f != Faction::Mongols {
            assert_eq!(data::unit_stats_for(f, KESHIG_KIND).ranged, 0);
        }
    }
    let scout = data::unit_stats_for(Faction::Mongols, UnitKind::Scout);
    assert_eq!((scout.cost, scout.strength), (60, 5));
    assert_eq!(
        data::unit_name_for(Faction::Mongols, UnitKind::Scout),
        "Scout"
    );
    assert!(data::attacks_in_melee(KESHIG_KIND));
    assert!(!data::attacks_in_melee(UnitKind::Ranger));
    assert!(data::attacks_in_melee(UnitKind::Soldier));
}

#[test]
fn a_keshig_shot_fights_as_ranged_and_its_charge_as_mounted() {
    let mut s = game(&[0, 1]);
    mongols(&mut s, 0);
    let keshig = unit(&mut s, 0, KESHIG_KIND, hex(4, 4));
    let spear = unit(&mut s, 1, UnitKind::Soldier, hex(5, 4));
    // Arrows: the spear line gets no bonus, and the shot takes no retaliation.
    let (att, def) = parts(&s, keshig, spear, FightKind::Ranged);
    assert!(att.iter().any(|l| l == "Keshig (ranged)"), "{att:?}");
    assert!(def.iter().all(|l| !l.contains("against")), "{def:?}");
    assert_eq!(
        plan_fight(&s, 1, keshig, Target::Unit(spear), FightKind::Ranged).damage_to_attacker,
        0
    );
    // The charge: the spear line meets it with the class bonus.
    let (_, def) = parts(&s, keshig, spear, FightKind::Melee);
    assert!(
        def.iter().any(|l| l == combat::MELEE_VS_MOUNTED_LABEL),
        "{def:?}"
    );

    let mut s = game(&[0, 1]);
    mongols(&mut s, 0);
    let keshig = unit(&mut s, 0, KESHIG_KIND, hex(4, 4));
    let archer = unit(&mut s, 1, UnitKind::Ranger, hex(5, 4));
    let horse = unit(&mut s, 1, UnitKind::Horseman, hex(4, 5));
    // Charging an Archer it is cavalry; shooting it, archer against archer.
    let (att, _) = parts(&s, keshig, archer, FightKind::Melee);
    assert!(
        att.iter().any(|l| l == combat::MOUNTED_VS_RANGED_LABEL),
        "{att:?}"
    );
    let (att, def) = parts(&s, keshig, archer, FightKind::Ranged);
    assert!(att.iter().chain(&def).all(|l| !l.contains("against")));
    // A rider gets no bonus against its arrows, as against an Archer's (designer, 2026-11-10).
    let (_, def) = parts(&s, keshig, horse, FightKind::Ranged);
    assert!(def.iter().all(|l| !l.contains("against")), "{def:?}");
    let (_, def) = parts(&s, archer, keshig, FightKind::Ranged);
    assert!(def.iter().all(|l| !l.contains("against")), "{def:?}");
}

#[test]
fn a_keshig_shoots_from_where_it_stands_or_charges_but_never_both() {
    let mut s = game(&[0, 1]);
    mongols(&mut s, 0);
    let keshig = unit(&mut s, 0, KESHIG_KIND, hex(4, 4));
    let spear = unit(&mut s, 1, UnitKind::Soldier, hex(5, 4));
    let shot = vec![Order::RangedAttack {
        unit: keshig,
        target: hex(5, 4),
    }];
    let (next, events) = resolve_units(&s, &orders(0, shot.clone()));
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::OrderFailed { .. })),
        "{events:?}"
    );
    assert_eq!(next.units[&keshig].pos, hex(4, 4), "a shot never moves it");
    assert_eq!(next.units[&keshig].hp, 100, "no retaliation");
    assert!(next.units[&spear].hp < 100);
    // A second order for the same unit in the same turn is refused: one order a turn.
    let mut both = shot;
    both.push(Order::MoveUnit {
        unit: keshig,
        to: hex(3, 4),
    });
    let (next, events) = resolve_units(&s, &orders(0, both));
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::OrderFailed { .. })),
        "{events:?}"
    );
    assert_eq!(next.units[&keshig].pos, hex(4, 4), "no shoot-and-ride-away");
    // Out of range: the shot is refused with the range in the reason.
    let far = unit(&mut s, 1, UnitKind::Soldier, hex(4, 7));
    let (_, events) = resolve_units(
        &s,
        &orders(
            0,
            vec![Order::RangedAttack {
                unit: keshig,
                target: s.units[&far].pos,
            }],
        ),
    );
    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::OrderFailed { reason, .. } if reason.contains("out of range")
        )),
        "{events:?}"
    );
}

#[test]
fn a_keshig_charge_can_take_a_city_and_its_shot_cannot() {
    let mut s = game(&[0, 1]);
    mongols(&mut s, 0);
    let c = city(&mut s, 1, hex(5, 4), 1);
    s.cities.get_mut(&c).unwrap().hp = 0;
    let keshig = unit(&mut s, 0, KESHIG_KIND, hex(4, 4));
    let (next, _) = resolve_units(
        &s,
        &orders(
            0,
            vec![Order::MoveUnit {
                unit: keshig,
                to: hex(5, 4),
            }],
        ),
    );
    assert_eq!(next.cities[&c].owner, PlayerId(0), "the charge captures");
    let (next, _) = resolve_units(
        &s,
        &orders(
            0,
            vec![Order::RangedAttack {
                unit: keshig,
                target: hex(5, 4),
            }],
        ),
    );
    assert_eq!(next.cities[&c].owner, PlayerId(1), "a shot never captures");
}

#[test]
fn the_forecast_offers_both_attacks_with_their_own_class_lines() {
    let mut s = game(&[0, 1]);
    mongols(&mut s, 0);
    let keshig = unit(&mut s, 0, KESHIG_KIND, hex(4, 4));
    unit(&mut s, 1, UnitKind::Soldier, hex(5, 4));
    crate::visibility::update_explored(&mut s);
    let v = player_view(&s, PlayerId(0));
    let shot = forecast::forecast_attack_mode(&v, keshig, hex(5, 4), forecast::AttackMode::Ranged)
        .expect("a shot in range");
    let charge = forecast::forecast_attack_mode(&v, keshig, hex(5, 4), forecast::AttackMode::Melee)
        .expect("a charge");
    assert!(shot.ranged && !charge.ranged);
    assert_eq!(shot.expected_damage_to_attacker, 0);
    assert!(charge.expected_damage_to_attacker > 0);
    let lines = |b: &Breakdown| b.parts.iter().map(|(l, _)| l.clone()).collect::<Vec<_>>();
    assert!(
        lines(&shot.defender_strength)
            .iter()
            .all(|l| !l.contains("against"))
    );
    assert!(
        lines(&charge.defender_strength)
            .iter()
            .any(|l| l == combat::MELEE_VS_MOUNTED_LABEL)
    );
    assert!(shot.expected_damage_to_defender > 0);
}
