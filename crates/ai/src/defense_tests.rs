use super::*;
use crate::Knobs;
use crate::fixture::{ENEMY, ME, ViewBuilder};
use crate::tactics::CityShield;
use legatum_core::{AiDifficulty, PlayerView, UnitId};

const CITY: Hex = Hex::new(8, 8);
const SUPPORT: Hex = Hex::new(7, 8);
const ENEMY_POS: Hex = Hex::new(11, 8);

fn view(kind: UnitKind, pos: Hex) -> PlayerView {
    let mut view = ViewBuilder::new(24, 16)
        .city(1, ME, CITY, 2)
        .unit(1, ME, kind, pos)
        .build();
    view.players[ENEMY.0 as usize].faction = Faction::Mongols;
    // Build after setting the faction so the visible stats include its three moves.
    ViewBuilder { view }
        .unit(50, ENEMY, UnitKind::Spearman, ENEMY_POS)
        .build()
}

fn planner(view: &PlayerView) -> Planner<'_> {
    Planner::new(
        view,
        Knobs::for_faction(view.faction_of(ME)),
        view.faction_of(ME),
        7,
    )
}

#[test]
fn a_healthy_archer_compares_a_bait_shot_with_survival() {
    let view = ViewBuilder::new(24, 16)
        .city(1, ME, Hex::new(6, 8), 3)
        .unit(1, ME, UnitKind::Archer, Hex::new(8, 8))
        .unit_hp(50, ENEMY, UnitKind::Spearman, Hex::new(10, 8), 1)
        .unit(51, ENEMY, UnitKind::Horseman, Hex::new(9, 7))
        .unit(52, ENEMY, UnitKind::Horseman, Hex::new(9, 9))
        .build();
    let mut p = planner(&view);
    let army = p.ctx.own_units.clone();
    p.fire_units(&army, false);
    p.avoid_free_damage(&army);
    assert!(
        matches!(
            p.orders.iter().find(|o| matches!(
                o,
                Order::MoveUnit {
                    unit: UnitId(1),
                    ..
                } | Order::RangedAttack {
                    unit: UnitId(1),
                    ..
                }
            )),
            Some(Order::MoveUnit { .. })
        ),
        "{:?}",
        p.orders
    );
}

#[test]
fn wounded_garrison_of_a_breached_city_shoots_the_adjacent_attacker_instead_of_resting() {
    let mut view = ViewBuilder::new(24, 16)
        .city(1, ME, CITY, 3)
        .unit_hp(1, ME, UnitKind::Archer, CITY, 20)
        .unit(50, ENEMY, UnitKind::Spearman, Hex::new(9, 8))
        .build();
    view.cities[0].hp = 0;
    let mut p = planner(&view);
    p.orders.push(Order::SkipUnit { unit: UnitId(1) });
    p.fire_from_holds();
    // An attacker next to a 0 HP city is the threat that decides the city, so the last
    // garrison fires rather than resting (valued city defence, 2026-11-08).
    assert_eq!(
        p.orders,
        vec![Order::RangedAttack {
            unit: UnitId(1),
            target: Hex::new(9, 8)
        }]
    );
}

/// The retreat-before-worst-case forecast still rejects this exchange, so a field unit
/// (including this unit arriving from outside) would not take it; the unit already
/// garrisoning the city holds, because it is not doomed for nothing. Only a fallen city
/// (0 HP) exposes its garrison: with HP left the city absorbs every attack (Civ 6).
fn field_retreats_but_garrison_holds(view: &PlayerView, unit: &legatum_core::view::UnitView) {
    assert_eq!(unit.pos, CITY);
    assert_ne!(planner(view).city_shield(CITY), CityShield::Full);
    let p = planner(view);
    assert!(!p.exchange_is_sustainable(unit, CITY));
    assert!(!p.garrison_doomed(unit));
    assert!(p.can_defend_at(unit, CITY));
    let mut arriving = unit.clone();
    arriving.pos = SUPPORT;
    assert!(!p.can_defend_at(&arriving, CITY));
}

#[test]
fn a_healthy_city_shields_a_ranged_garrison_so_defense_sends_it_in() {
    let view = view(UnitKind::Archer, SUPPORT);
    let mut p = planner(&view);
    let u = p.ctx.own_units[0];
    let city = p.ctx.own_cities[0];
    assert_eq!(p.city_shield(CITY), CityShield::Full);
    assert!(p.can_defend_at(u, CITY));
    assert!(p.defend(u, city));
    assert_eq!(
        p.orders,
        vec![Order::MoveUnit {
            unit: u.id,
            to: CITY
        }]
    );
}

#[test]
fn routine_defense_and_fallback_garrison_reject_the_same_bad_assignment() {
    // A fallen city exposes its garrison: two Spearmen can kill an arriving Archer.
    let mut view = ViewBuilder {
        view: view(UnitKind::Archer, SUPPORT),
    }
    .unit(51, ENEMY, UnitKind::Spearman, Hex::new(10, 9))
    .build();
    view.cities[0].hp = 0;
    let mut p = planner(&view);
    let u = p.ctx.own_units[0];
    let city = p.ctx.own_cities[0];
    assert!(p.can_defend_at(u, SUPPORT));
    assert!(!p.can_defend_at(u, CITY));
    assert!(p.defend(u, city));
    assert_eq!(p.orders, vec![Order::SkipUnit { unit: u.id }]);

    let mut p = planner(&view);
    p.garrison_cities(&[u]);
    assert!(p.orders.is_empty(), "fallback must not send it back");
}

#[test]
fn garrison_selects_a_suitable_defender_over_a_closer_ranger() {
    let mut view = ViewBuilder {
        view: view(UnitKind::Archer, SUPPORT),
    }
    .unit(2, ME, UnitKind::Pikeman, Hex::new(6, 8))
    .build();
    // A healthy city shields its garrison: the threatened city takes the shooter.
    let mut p = planner(&view);
    p.garrison_cities(&p.ctx.own_units.clone());
    assert_eq!(
        p.orders,
        vec![Order::MoveUnit {
            unit: UnitId(1),
            to: CITY
        }]
    );
    // A fallen city exposes its garrison to melee: only the Pikeman can hold it.
    view.cities[0].hp = 0;
    let mut p = planner(&view);
    p.garrison_cities(&p.ctx.own_units.clone());
    assert_eq!(
        p.orders,
        vec![Order::MoveUnit {
            unit: UnitId(2),
            to: CITY
        }]
    );
}

#[test]
fn healthy_sustainable_city_defense_is_not_an_unanswered_field_exposure() {
    let view = view(UnitKind::Pikeman, CITY);
    let mut p = planner(&view);
    let u = p.ctx.own_units[0];
    assert!(p.exposure_at(CITY) > 0);
    assert!(p.can_defend_at(u, CITY));
    assert!(!p.can_defend_at(u, CITY.neighbor(0)));
    p.avoid_free_damage(&[u]);
    assert!(p.orders.is_empty());
    p.garrison_cities(&[u]);
    assert_eq!(p.orders, vec![Order::SkipUnit { unit: u.id }]);
}

#[test]
fn holding_preserves_fortification_but_arrival_cannot_borrow_it() {
    // A fallen city exposes its garrison to a Crossbowman that cannot capture: the exchange
    // decides. Fortified (20) the holder trades evenly with its counterfire; moving resets
    // the fortification, leaving only 14 strength at the destination.
    let mut view = view(UnitKind::Crossbowman, CITY);
    view.units.retain(|u| u.owner == ME);
    let mut view = ViewBuilder { view }
        .unit(50, ENEMY, UnitKind::Crossbowman, Hex::new(10, 8))
        .build();
    view.units[1].fortify = 5;
    view.cities[0].hp = 0;
    view.units[0].fortify = 6;
    let p = planner(&view);
    assert!(p.can_defend_at(&view.units[0], CITY));
    view.units[0].pos = SUPPORT;
    assert!(!planner(&view).can_defend_at(&view.units[0], CITY));
}

#[test]
fn an_unreachable_attack_does_not_displace_a_healthy_garrison() {
    let mut view = view(UnitKind::Archer, CITY);
    // Low enough that the Spearman could break the city if it reached it.
    view.cities[0].hp = 30;
    for h in ENEMY_POS.neighbors() {
        let i = view.index(h).unwrap();
        view.tiles[i].terrain = Some(Terrain::Hills);
    }
    let p = planner(&view);
    assert!(p.exposure_at(CITY) > 0);
    assert_eq!(p.city_shield(CITY), CityShield::Full);
    assert!(p.can_defend_at(&view.units[0], CITY));
    // Unknown terrain is uncertainty, not proof that the enemy cannot reach us.
    for h in ENEMY_POS.neighbors() {
        let i = view.index(h).unwrap();
        view.tiles[i].terrain = None;
        view.tiles[i].explored = false;
    }
    // Leave one known, long detour to keep the coarse exposure estimate connected.
    let i = view.index(ENEMY_POS.neighbor(0)).unwrap();
    view.tiles[i].terrain = Some(Terrain::Hills);
    view.tiles[i].explored = true;
    let p = planner(&view);
    assert_eq!(p.city_shield(CITY), CityShield::MeleeOnly);
    // It may then fight the garrison: it holds, since surviving prevents a capture.
    assert!(!p.garrison_doomed(&view.units[0]));
    assert!(p.can_defend_at(&view.units[0], CITY));
}

#[test]
fn a_surviving_garrison_can_shield_an_immediately_capturable_city() {
    let mut view = view(UnitKind::Archer, CITY);
    // With HP left the city takes the attack: its garrison is safe and holds.
    let p = planner(&view);
    assert!(p.exchange_is_sustainable(&view.units[0], CITY));
    assert!(!p.garrison_doomed(&view.units[0]));
    // At 0 HP the garrison is exposed; surviving the attack prevents an immediate capture.
    view.cities[0].hp = 0;
    let p = planner(&view);
    assert_eq!(p.city_shield(CITY), CityShield::None);
    assert!(!p.garrison_doomed(&view.units[0]));
    assert!(p.can_defend_at(&view.units[0], CITY));
    // Certain death with a melee attacker in reach: the city falls either way, so
    // even the garrison leaves rather than die for nothing.
    view.units[0].hp = 20;
    assert!(planner(&view).garrison_doomed(&view.units[0]));
    assert!(!planner(&view).can_defend_at(&view.units[0], CITY));
}

#[test]
fn a_ruined_city_does_not_justify_unanswered_fire_from_an_enemy_that_cannot_capture() {
    let mut view = view(UnitKind::Spearman, CITY);
    view.cities[0].hp = 0;
    view.units.retain(|u| u.owner == ME);
    let view = ViewBuilder { view }
        .unit(50, ENEMY, UnitKind::Archer, Hex::new(10, 8))
        .build();
    field_retreats_but_garrison_holds(&view, &view.units[0]);
}

#[test]
fn counterfire_is_one_volley_and_must_not_hide_overwhelming_threats() {
    let mut view = view(UnitKind::Crossbowman, CITY);
    view.cities[0].hp = 0;
    view.units.retain(|u| u.owner == ME);
    let mut view = ViewBuilder { view }
        .unit(50, ENEMY, UnitKind::Archer, Hex::new(10, 8))
        .build();
    assert!(planner(&view).can_defend_at(&view.units[0], CITY));
    view = ViewBuilder { view }
        .unit(51, ENEMY, UnitKind::Crossbowman, Hex::new(8, 6))
        .unit(52, ENEMY, UnitKind::Crossbowman, Hex::new(6, 10))
        .build();
    // Ranged fire alone cannot capture: the garrison holds even against three shooters.
    field_retreats_but_garrison_holds(&view, &view.units[0]);
}

#[test]
fn cumulative_damage_preserves_survivable_garrisons_and_wound_immunity() {
    for (faction, count, survives) in [
        (Faction::Switzerland, 1, true),
        (Faction::Switzerland, 2, true),
        (Faction::Switzerland, 3, false),
        (Faction::Sparta, 3, true),
    ] {
        // Pikemen have the same base strength for both factions; only Sparta
        // ignores wounds, so three equal-strength attacks remain survivable.
        let mut view = ViewBuilder::new(24, 16)
            .faction(faction)
            .city(1, ME, CITY, 2)
            .unit(1, ME, UnitKind::Pikeman, CITY)
            .build();
        view.players[ENEMY.0 as usize].faction = Faction::Mongols;
        view.units[0].fortify = 3;
        for (id, pos) in [
            (50, ENEMY_POS),
            (51, Hex::new(10, 6)),
            (52, Hex::new(11, 5)),
        ]
        .into_iter()
        .take(count)
        {
            view = ViewBuilder { view }
                .unit(id, ENEMY, UnitKind::Pikeman, pos)
                .build();
        }
        let p = planner(&view);
        assert_eq!(p.exchange_is_sustainable(&view.units[0], CITY), survives);
        if survives {
            assert!(p.can_defend_at(&view.units[0], CITY));
        } else {
            field_retreats_but_garrison_holds(&view, &view.units[0]);
        }
    }
}

#[test]
fn enemy_iteration_order_cannot_hide_lethal_ranged_then_melee_damage() {
    use legatum_core::combat;

    let mut view = ViewBuilder::new(24, 16)
        .city(1, ME, CITY, 2)
        .unit_hp(1, ME, UnitKind::Spearman, CITY, 60)
        .unit(50, ENEMY, UnitKind::Scout, Hex::new(10, 8))
        .unit(51, ENEMY, UnitKind::Archer, Hex::new(8, 10))
        .build();
    view.cities[0].hp = 0;
    let defender = view.units[0].clone();
    let scout =
        combat::unit_attack_strength_for(view.faction_of(ENEMY), UnitKind::Scout, 100, 0).total;
    let ranger =
        combat::unit_ranged_strength_for(view.faction_of(ENEMY), UnitKind::Archer, 100, 0).total;
    let damage_after = |attacks: [i32; 2]| {
        attacks.into_iter().fold(0, |damage, attack| {
            let defense = combat::unit_defense_strength_for(
                view.faction_of(ME),
                defender.kind,
                defender.hp - damage,
                Terrain::Plains,
                0,
                defender.level,
            )
            .total;
            damage + combat::damage(attack, defense, combat::ROLL_MAX)
        })
    };
    // Ranged fire happens before movement. Following unit IDs instead misses the
    // lethal order by one HP because the engine rounds both damage and wounds.
    assert_eq!(damage_after([scout, ranger]), 59);
    assert_eq!(damage_after([ranger, scout]), 60);
    field_retreats_but_garrison_holds(&view, &defender);
    view.units.reverse();
    field_retreats_but_garrison_holds(&view, &defender);
}

#[test]
fn a_new_city_does_not_strip_the_capitals_only_garrison() {
    // A playtest-style loss: the capital's lone Spearman walked to a fresh colony and the
    // empty capital was captured. Only a free unit outside every city fills the colony.
    let colony = Hex::new(4, 8);
    let mut view = ViewBuilder::new(24, 16)
        .city(1, ME, CITY, 2)
        .city(2, ME, colony, 1)
        .unit(1, ME, UnitKind::Spearman, CITY)
        .build();
    // The damaged colony is garrisoned first.
    view.cities.iter_mut().find(|c| c.pos == colony).unwrap().hp = 100;
    let mut p = planner(&view);
    p.garrison_cities(&p.ctx.own_units.clone());
    assert_eq!(p.orders, vec![Order::SkipUnit { unit: UnitId(1) }]);
}

#[test]
fn idle_surplus_defender_takes_cover_even_when_city_power_covers_threat() {
    // Coach g20 native-1790594683761282000-70283-8, P5 T28-29.
    let city = Hex::new(8, 8);
    let cover = Hex::new(7, 8);
    let mut view = ViewBuilder::new(24, 16)
        .city(1, ME, city, 3)
        .unit(1, ME, UnitKind::Pikeman, city)
        .unit(2, ME, UnitKind::Archer, Hex::new(6, 8))
        .unit(50, ENEMY, UnitKind::Pikeman, Hex::new(11, 8))
        .terrain(cover, Terrain::Hills)
        .build();
    view.units[1].fortify = 2 * data::FORTIFY_PER_TURN;
    let mut p = planner(&view);
    p.plan_military();
    assert!(
        p.orders.contains(&Order::MoveUnit {
            unit: UnitId(2),
            to: cover
        }),
        "{:?}",
        p.orders
    );
    assert!(
        !p.orders
            .iter()
            .any(|o| matches!(o, Order::MoveUnit { unit, .. } if *unit == UnitId(1)))
    );
    // One idle turn is not the timeout, and no enemy means ordinary resting.
    view.units[1].fortify = data::FORTIFY_PER_TURN;
    let mut p = planner(&view);
    p.plan_military();
    assert!(p.orders.contains(&Order::SkipUnit { unit: UnitId(2) }));
    view.units[1].fortify = 2 * data::FORTIFY_PER_TURN;
    view.units.retain(|u| u.owner == ME);
    let mut p = planner(&view);
    p.plan_military();
    assert!(p.orders.contains(&Order::SkipUnit { unit: UnitId(2) }));
}

#[test]
fn a_city_that_can_be_broken_this_turn_exposes_its_garrison_to_melee_only() {
    // Three Pikemen can break a pop-2 city this turn: ranged fire still only hits the
    // city, but melee after it reaches the garrison.
    let mut view = ViewBuilder::new(24, 16)
        .city(1, ME, CITY, 2)
        .unit(1, ME, UnitKind::Archer, CITY)
        .build();
    view.players[ENEMY.0 as usize].faction = Faction::Mongols;
    let mut view = ViewBuilder { view }
        .unit(50, ENEMY, UnitKind::Archer, Hex::new(10, 8))
        .build();
    assert_eq!(planner(&view).city_shield(CITY), CityShield::Full);
    assert!(planner(&view).can_defend_at(&view.units[0], CITY));
    view.cities[0].hp = 30;
    let p = planner(&view);
    assert_eq!(p.city_shield(CITY), CityShield::MeleeOnly);
    assert!(
        p.exchange_is_sustainable(&view.units[0], CITY),
        "ranged fire alone never reaches the garrison of a city with HP"
    );
    for (id, pos) in [(51, ENEMY_POS), (52, Hex::new(10, 6))] {
        view = ViewBuilder { view }
            .unit(id, ENEMY, UnitKind::Pikeman, pos)
            .build();
    }
    assert!(!planner(&view).exchange_is_sustainable(&view.units[0], CITY));
}

#[test]
fn rome_keeps_the_last_defender_of_a_threatened_city() {
    let mut view = ViewBuilder::new(24, 16)
        .faction(Faction::Rome)
        .city(1, ME, CITY, 3)
        .unit(1, ME, UnitKind::Spearman, CITY)
        .unit(2, ME, UnitKind::Spearman, Hex::new(2, 8))
        .unit(50, ENEMY, UnitKind::Spearman, Hex::new(9, 8))
        .gold(500)
        .build();
    view.units
        .iter_mut()
        .find(|u| u.id == UnitId(1))
        .unwrap()
        .level = 1;
    let mut p = planner(&view);
    p.difficulty = AiDifficulty::Hard;
    assert!(!p.roman_unit_can_leave(p.ctx.own_units[0]));
    p.plan_settlers();
    assert!(
        !p.orders
            .iter()
            .any(|order| matches!(order, Order::FoundCity { unit } if *unit == UnitId(1)))
    );
}

#[test]
fn rome_keeps_a_second_defender_when_local_threat_exceeds_the_remaining_force() {
    let view = ViewBuilder::new(24, 16)
        .faction(Faction::Rome)
        .city(1, ME, CITY, 3)
        .unit(1, ME, UnitKind::Spearman, SUPPORT)
        .unit(2, ME, UnitKind::Spearman, CITY)
        .unit(3, ME, UnitKind::Spearman, Hex::new(2, 8))
        .unit(50, ENEMY, UnitKind::Horseman, Hex::new(9, 8))
        .unit(51, ENEMY, UnitKind::Horseman, Hex::new(9, 9))
        .unit(52, ENEMY, UnitKind::Horseman, Hex::new(10, 8))
        .build();
    let p = planner(&view);
    assert!(p.ctx.threat[0] > 0);
    assert!(!p.roman_unit_can_leave(p.ctx.own_units[0]));
}

#[test]
fn rome_values_a_safe_site_for_a_spare_veteran_below_the_settler_floor() {
    let mut view = ViewBuilder::new(28, 16)
        .faction(Faction::Rome)
        .city(1, ME, Hex::new(6, 8), 3)
        .unit(1, ME, UnitKind::Spearman, Hex::new(13, 8))
        .gold(500)
        .build();
    view.units[0].level = 1;
    let mut p = planner(&view);
    p.difficulty = AiDifficulty::Hard;
    p.decisions.settler_min_site_score = 1000;
    let site = p
        .choose_site(&view.units[0])
        .expect("safe Roman foundation site");
    assert!(legatum_core::settlement::known_legal_site(&view, site));
}

#[test]
fn rome_founds_at_once_with_a_spare_veteran_on_a_safe_site() {
    let site = Hex::new(13, 8);
    let mut view = ViewBuilder::new(28, 16)
        .faction(Faction::Rome)
        .city(1, ME, Hex::new(6, 8), 3)
        .unit(1, ME, UnitKind::Spearman, site)
        .unit(2, ME, UnitKind::Spearman, Hex::new(6, 8))
        .gold(0)
        .build();
    view.units[0].level = 1;
    let mut p = planner(&view);
    p.difficulty = AiDifficulty::Hard;
    p.decisions.settler_min_site_score = 1000;
    assert!(legatum_core::settlement::known_legal_site(&view, site));
    p.plan_settlers();
    assert!(
        p.orders.contains(&Order::FoundCity { unit: UnitId(1) }),
        "{:?}",
        p.orders
    );
}

/// The Roman genes this stream added, set explicitly so a test reads one at a time.
fn rome_planner(view: &PlayerView) -> Planner<'_> {
    let mut p = planner(view);
    p.difficulty = AiDifficulty::Hard;
    p.decisions.roman_founders = 1;
    p.decisions.roman_hunt_radius = 8;
    p.decisions.roman_soldier_route = 1;
    p.decisions.roman_veteran_retreat = 1;
    p.decisions.roman_promotion_value = 100;
    p
}

#[test]
fn rome_prices_the_first_promotions_a_siege_earns() {
    let view = ViewBuilder::new(24, 16)
        .faction(Faction::Rome)
        .city(1, ME, CITY, 3)
        .unit(1, ME, UnitKind::Spearman, Hex::new(12, 8))
        .unit(2, ME, UnitKind::Spearman, Hex::new(12, 9))
        .unit(3, ME, UnitKind::Spearman, Hex::new(12, 10))
        .unit(4, ME, UnitKind::Archer, Hex::new(12, 11))
        .build();
    let mut view = view;
    for u in view.units.iter_mut() {
        u.xp = Some(match u.id.0 {
            2 => 10,
            _ => 0,
        });
    }
    let force = [UnitId(1), UnitId(2), UnitId(3), UnitId(4)];
    let mut p = rome_planner(&view);
    // Two turns of city attacks pay 6 XP each; the most experienced (10 XP) also takes the
    // capture: 26 >= 16. The others reach 6. The Archer is not a founder.
    assert_eq!(p.roman_siege_promotions(&force, 2), 1);
    // Four turns pay 12: the 10-XP unit promotes on attacks alone, and the capture falls
    // to it, so the 0-XP units stay at 12.
    assert_eq!(p.roman_siege_promotions(&force, 4), 1);
    // Six turns pay 18: every melee unit promotes.
    assert_eq!(p.roman_siege_promotions(&force, 6), 3);
    p.decisions.roman_promotion_value = 0;
    assert_eq!(p.roman_siege_promotions(&force, 6), 0);
}

#[test]
fn rome_pulls_a_wounded_veteran_out_of_reach_to_heal() {
    let vet = Hex::new(12, 8);
    let mut view = ViewBuilder::new(24, 16)
        .faction(Faction::Rome)
        .city(1, ME, Hex::new(4, 8), 3)
        .unit(2, ME, UnitKind::Spearman, Hex::new(4, 8))
        .unit(3, ME, UnitKind::Spearman, Hex::new(5, 8))
        .unit_hp(1, ME, UnitKind::Spearman, vet, 50)
        .unit(50, ENEMY, UnitKind::Spearman, Hex::new(13, 8))
        .build();
    view.units
        .iter_mut()
        .find(|u| u.id == UnitId(1))
        .unwrap()
        .level = 1;
    let mut p = rome_planner(&view);
    assert!(p.exposure_at(vet) > 0);
    p.plan_settlers();
    let moved = p.orders.iter().any(|o| {
        matches!(o, Order::MoveUnit { unit, to } if *unit == UnitId(1) && to.distance(Hex::new(13, 8)) > 1)
    });
    assert!(moved, "{:?}", p.orders);
    let mut p = rome_planner(&view);
    p.decisions.roman_veteran_retreat = 0;
    p.plan_settlers();
    assert!(
        !p.orders
            .iter()
            .any(|o| format!("{o:?}").contains("UnitId(1)")),
        "{:?}",
        p.orders
    );
}

/// Gene `roman_finish_siege` (owner's game, Laurion T32-34): three Roman units promoted by
/// their city attacks and, now wounded veterans beside a nearly fallen city, the veteran
/// retreat walked all three home. While the capture forecast holds, the siege keeps them.
#[test]
fn rome_finishes_a_siege_its_first_promotions_were_earned_in() {
    let target = Hex::new(14, 8);
    let mut view = ViewBuilder::new(24, 16)
        .faction(Faction::Rome)
        .city(1, ME, Hex::new(4, 8), 3)
        .city(9, ENEMY, target, 1)
        .unit(4, ME, UnitKind::Spearman, Hex::new(4, 8))
        .unit_hp(1, ME, UnitKind::Spearman, Hex::new(13, 8), 60)
        .unit_hp(2, ME, UnitKind::Spearman, Hex::new(13, 9), 60)
        .unit_hp(3, ME, UnitKind::Spearman, Hex::new(15, 7), 60)
        // A relief column in reach: what makes the wounded veterans exposed.
        .unit(50, ENEMY, UnitKind::Spearman, Hex::new(12, 8))
        .build();
    for u in view.units.iter_mut().filter(|u| u.id.0 <= 3) {
        u.level = 1;
        u.xp = Some(16);
    }
    let city = view.cities.iter_mut().find(|c| c.pos == target).unwrap();
    city.hp = 10;
    let siege = |view: &PlayerView, finish: i32| {
        let mut p = rome_planner(view);
        p.decisions.roman_finish_siege = finish;
        p.campaigns.active = Some(crate::campaign::Campaign {
            target: legatum_core::CityId(9),
            start_turn: 1,
            committed: vec![UnitId(1), UnitId(2), UnitId(3)],
            deadline: None,
            expected_value: 100,
            target_was_unwalled: true,
        });
        p.plan_settlers();
        // Orders given here pre-empt the siege: plan_military never sees these units.
        let left = (1..=3)
            .filter(|id| {
                p.orders
                    .iter()
                    .any(|o| legatum_core::rules::units::order_unit(o) == Some(UnitId(*id)))
            })
            .count();
        (p.roman_live_siege().is_some(), left)
    };
    assert_eq!(
        siege(&view, 1),
        (true, 0),
        "the capture forecast holds; nobody leaves"
    );
    view.cities.iter_mut().find(|c| c.pos == target).unwrap().hp = 0;
    assert_eq!(
        siege(&view, 1),
        (true, 0),
        "shot down to 0 HP, it falls to the next melee step"
    );
    let (live, left) = siege(&view, 0);
    assert!(!live);
    assert!(
        left > 0,
        "the control arm pulls the veterans out of the siege"
    );
}

#[test]
fn rome_hunts_promotions_only_near_home() {
    let city = Hex::new(3, 8);
    let recruit = Hex::new(8, 8);
    let camp = Hex::new(17, 8);
    let mut view = ViewBuilder::new(28, 16)
        .faction(Faction::Rome)
        .city(1, ME, city, 3)
        .unit(2, ME, UnitKind::Spearman, city)
        .unit(1, ME, UnitKind::Spearman, recruit)
        .build();
    view.barbarian_camps
        .push(legatum_core::view::BarbarianCampView {
            pos: camp,
            visible: false,
        });
    let toward_camp = |orders: &[Order]| {
        orders.iter().any(|o| match o {
            Order::MoveUnit { unit, to } | Order::MoveUnitPath { unit, to, .. } => {
                *unit == UnitId(1) && to.distance(camp) < recruit.distance(camp)
            }
            _ => false,
        })
    };
    // The camp is 9 hexes from the recruit and 14 from home: beyond an 8-hex hunt.
    let mut p = rome_planner(&view);
    p.plan_military();
    assert!(!toward_camp(&p.orders), "{:?}", p.orders);
    let mut p = rome_planner(&view);
    p.decisions.roman_hunt_radius = 0;
    p.plan_military();
    assert!(toward_camp(&p.orders), "{:?}", p.orders);
}

#[test]
fn rome_routes_a_veteran_founder_as_a_soldier_not_a_settler() {
    let vet = Hex::new(14, 8);
    let mut view = ViewBuilder::new(28, 16)
        .faction(Faction::Rome)
        .city(1, ME, Hex::new(4, 8), 3)
        .unit(2, ME, UnitKind::Spearman, Hex::new(4, 8))
        .unit(1, ME, UnitKind::Spearman, vet)
        .unit(50, ENEMY, UnitKind::Horseman, Hex::new(14, 11))
        .unit(51, ENEMY, UnitKind::Horseman, Hex::new(14, 5))
        .build();
    view.units
        .iter_mut()
        .find(|u| u.id == UnitId(1))
        .unwrap()
        .level = 1;
    let p = rome_planner(&view);
    let soldier = p.choose_site(
        &view
            .units
            .iter()
            .find(|u| u.id == UnitId(1))
            .unwrap()
            .clone(),
    );
    let mut p = rome_planner(&view);
    p.decisions.roman_soldier_route = 0;
    let settler = p.choose_site(
        &view
            .units
            .iter()
            .find(|u| u.id == UnitId(1))
            .unwrap()
            .clone(),
    );
    assert!(soldier.is_some(), "a soldier walks past the riders' reach");
    assert!(
        settler.is_none(),
        "a Settler's route avoids it: {settler:?}"
    );
}
