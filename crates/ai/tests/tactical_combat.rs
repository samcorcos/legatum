//! C1 regressions resolve real turns from small, fogged tactical situations.

use legatum_ai::{Brain, ScriptedBrain};
use legatum_core::events::Defender;
use legatum_core::game::{blank_game, spawn_unit};
use legatum_core::rules::cities::found_city;
use legatum_core::state::Map;
use legatum_core::visibility::update_explored;
use legatum_core::*;
use std::collections::BTreeMap;

const ME: PlayerId = PlayerId(0);
const ENEMY: PlayerId = PlayerId(1);

fn at(col: i32, row: i32) -> Hex {
    Hex::from_offset(col, row)
}

fn scenario() -> GameState {
    let mut setup = GameSetup::standard(7, [Faction::Athens, Faction::India]);
    setup.width = 24;
    setup.height = 16;
    // Authored before cities fired back; city strike has its own tests in core.
    setup.turn_limit = None;
    setup.players.truncate(2);
    for (i, p) in setup.players.iter_mut().enumerate() {
        p.team = TeamId(i as u8);
    }
    let mut state = blank_game(&setup, Map::new(24, 16, Terrain::Grassland));
    // These fixtures exercise active combat. New games start at Peace, so make
    // the hostility explicit instead of weakening any tactical assertions.
    for relation in &mut state.diplomacy.as_mut().unwrap().relations {
        relation.state = diplomacy::RelationState::War;
    }
    for (owner, pos) in [(ME, at(8, 8)), (ENEMY, at(20, 8))] {
        let settler = spawn_unit(&mut state, owner, UnitKind::Settler, pos);
        found_city(&mut state, settler, &mut Vec::new()).unwrap();
    }
    state
        .cities
        .values_mut()
        .find(|c| c.owner == ME)
        .unwrap()
        .name = "Shardfall".into();
    state.player_mut(ME).gold = 0;
    state.turn = 30;
    state
}

fn decide(state: &mut GameState, difficulty: AiDifficulty) -> Vec<Order> {
    update_explored(state);
    let view = player_view(state, ME);
    let mut brain =
        ScriptedBrain::with_difficulty(view.faction_of(ME), state.setup.seed, ME, difficulty);
    let orders = brain.orders(&view);
    // Recreating the brain or serializing a save cannot erase the tactical response.
    let loaded: GameState = serde_json::from_str(&serde_json::to_string(state).unwrap()).unwrap();
    assert_eq!(orders, brain.orders(&player_view(&loaded, ME)));
    assert!(validate_orders(state, ME, &orders).is_empty(), "{orders:?}");
    orders
}

fn play(state: &GameState, ours: Vec<Order>, theirs: Vec<Order>) -> (GameState, Vec<Event>) {
    assert!(validate_orders(state, ENEMY, &theirs).is_empty());
    let (next, events) = resolve_turn(state, &BTreeMap::from([(ME, ours), (ENEMY, theirs)]));
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::OrderFailed { .. })),
        "{events:?}"
    );
    (next, events)
}

#[test]
fn scouts_flee_known_attack_reach_toward_home_within_one_turn() {
    for difficulty in [AiDifficulty::Normal, AiDifficulty::Hard] {
        for (enemy_kind, enemy_faction, distance) in [
            (UnitKind::Spearman, Faction::India, 1),
            (UnitKind::Archer, Faction::India, 2),
        ] {
            for fully_explored in [false, true] {
                let mut state = scenario();
                state.player_mut(ENEMY).faction = enemy_faction;
                let scout = spawn_unit(&mut state, ME, UnitKind::Scout, at(10, 8));
                state.units.get_mut(&scout).unwrap().moves_left = 0;
                let enemy = spawn_unit(&mut state, ENEMY, enemy_kind, at(10 + distance, 8));
                if fully_explored {
                    for i in 0..state.map.len() {
                        state.player_mut(ME).explored.set(i);
                    }
                }
                let orders = decide(&mut state, difficulty);
                let view = player_view(&state, ME);
                let threat = view.units.iter().find(|u| u.id == enemy).unwrap();
                let reach = threat.range.max(threat.max_moves);
                let (next, _) = play(&state, orders, vec![]);
                let escaped = &next.units[&scout];
                assert!(escaped.pos.distance(threat.pos) > reach);
                assert_ne!(next.map.get(escaped.pos).unwrap().owner, Some(ENEMY));
                assert!(
                    escaped.pos.distance(at(8, 8)) <= at(10, 8).distance(at(8, 8)),
                    "{difficulty:?} {enemy_kind:?} {distance} explored={fully_explored}: {:?}",
                    escaped.pos
                );
            }
        }
    }
}

#[test]
fn settler_escapes_when_the_enemy_blocks_the_route_home() {
    let mut state = scenario();
    let settler = spawn_unit(&mut state, ME, UnitKind::Settler, at(11, 8));
    spawn_unit(&mut state, ENEMY, UnitKind::Spearman, at(10, 8));
    let orders = decide(&mut state, AiDifficulty::Normal);
    let (next, _) = play(&state, orders, vec![]);
    assert!(next.units[&settler].pos.distance(at(10, 8)) > 2);
}

#[test]
fn settler_can_retreat_into_a_safe_city_with_a_friendly_garrison() {
    let mut state = scenario();
    let settler = spawn_unit(&mut state, ME, UnitKind::Settler, at(10, 8));
    spawn_unit(&mut state, ME, UnitKind::Spearman, at(8, 8));
    spawn_unit(&mut state, ENEMY, UnitKind::Musketman, at(11, 8));
    let orders = decide(&mut state, AiDifficulty::Normal);
    let (next, _) = play(&state, orders, vec![]);
    assert_eq!(next.units[&settler].pos, at(8, 8));
}

#[test]
fn a_scout_moves_away_even_when_rough_ground_prevents_a_complete_escape() {
    let mut state = scenario();
    let pos = at(11, 8);
    let scout = spawn_unit(&mut state, ME, UnitKind::Scout, pos);
    state.player_mut(ENEMY).faction = Faction::Mongols;
    let enemy = spawn_unit(&mut state, ENEMY, UnitKind::Spearman, at(12, 8));
    for h in pos.within(3) {
        state.map.get_mut(h).unwrap().terrain = Terrain::Hills;
    }
    let orders = decide(&mut state, AiDifficulty::Normal);
    let (next, _) = play(&state, orders, vec![]);
    // Only one costly step fits. Get closer to safety instead of freezing because
    // every reachable arrival remains inside the fast enemy's three-move reach.
    assert_eq!(next.units[&scout].pos.distance(pos), 1);
    assert_eq!(next.units[&scout].pos.distance(next.units[&enemy].pos), 2);
}

#[test]
fn garrison_sorties_against_an_adjacent_ranger_in_one_turn_even_when_wounded() {
    for difficulty in [AiDifficulty::Normal, AiDifficulty::Hard] {
        for (soldier_hp, ranger_hp) in [(100, 100), (55, 30)] {
            let mut state = scenario();
            let soldier = spawn_unit(&mut state, ME, UnitKind::Spearman, at(8, 8));
            state.units.get_mut(&soldier).unwrap().hp = soldier_hp;
            state.units.get_mut(&soldier).unwrap().moves_left = 0;
            let ranger = spawn_unit(&mut state, ENEMY, UnitKind::Archer, at(9, 8));
            state.units.get_mut(&ranger).unwrap().hp = ranger_hp;
            let orders = decide(&mut state, difficulty);
            let forecast = forecast_attack_mode(
                &player_view(&state, ME),
                soldier,
                at(9, 8),
                AttackMode::Melee,
            )
            .unwrap();
            assert!(forecast.loss_chance <= 25);
            assert!(orders.contains(&Order::MoveUnit {
                unit: soldier,
                to: at(9, 8)
            }));
            let (_, events) = play(
                &state,
                orders,
                vec![Order::RangedAttack {
                    unit: ranger,
                    target: at(8, 8),
                }],
            );
            assert!(events.iter().any(|e| matches!(e,
                Event::Battle { attacker, defender: Defender::Unit { unit, .. },
                    damage_to_defender, ranged: false, .. }
                    if *attacker == soldier && *unit == ranger && *damage_to_defender > 0
            )));
        }
    }
}

#[test]
fn a_bombarded_city_buys_ranged_relief_when_its_garrison_cannot_answer() {
    let mut state = scenario();
    let city = state.cities_of(ME).next().unwrap().id;
    let soldier = spawn_unit(&mut state, ME, UnitKind::Spearman, at(8, 8));
    state.units.get_mut(&soldier).unwrap().fortify = 6;
    let ranger = spawn_unit(&mut state, ENEMY, UnitKind::Archer, at(10, 8));
    state.map.get_mut(at(9, 8)).unwrap().terrain = Terrain::Ocean;
    state.player_mut(ME).gold = 1000;
    let orders = decide(&mut state, AiDifficulty::Normal);
    assert!(
        orders.contains(&Order::BuyItem {
            city,
            item: ProductionItem::Unit {
                unit: UnitKind::Archer
            }
        }),
        "{orders:?}"
    );
    let (_, events) = play(
        &state,
        orders,
        vec![Order::RangedAttack {
            unit: ranger,
            target: at(8, 8),
        }],
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::ItemBought { city: id, item: ProductionItem::Unit { .. }, .. } if *id == city))
    );
}

#[test]
fn three_healthy_soldiers_attack_a_lightly_defended_city_together_before_retreating() {
    let target = at(20, 8);
    for difficulty in [AiDifficulty::Normal, AiDifficulty::Hard] {
        for (faction, positions) in [
            (Faction::Byzantium, [at(18, 8), at(19, 7), at(19, 9)]),
            // Separate straight approaches: the fast units must spend their third
            // move attacking, rather than merely staging next to the target.
            (
                Faction::Mongols,
                [
                    target.neighbor(0).neighbor(0),
                    target.neighbor(2).neighbor(2).neighbor(2),
                    target.neighbor(4).neighbor(4).neighbor(4),
                ],
            ),
        ] {
            let mut state = scenario();
            state.player_mut(ME).faction = faction;
            let garrison = spawn_unit(&mut state, ENEMY, UnitKind::Spearman, target);
            state.units.get_mut(&garrison).unwrap().hp = 70;
            let soldiers = positions.map(|pos| spawn_unit(&mut state, ME, UnitKind::Spearman, pos));
            let orders = decide(&mut state, difficulty);
            for soldier in soldiers {
                assert!(
                    orders.contains(&Order::MoveUnit {
                        unit: soldier,
                        to: target
                    }),
                    "{faction:?} {difficulty:?} {orders:?}"
                );
            }
            let (_, events) = play(&state, orders, vec![]);
            for soldier in soldiers {
                assert!(
                    events.iter().any(|e| matches!(e,
                        Event::Battle { attacker, at, damage_to_defender, ranged: false, .. }
                            if *attacker == soldier && *at == target && *damage_to_defender > 0
                    )),
                    "{events:?}"
                );
            }
        }
    }
}

#[test]
fn two_healthy_soldiers_do_not_charge_a_full_garrison_without_a_favorable_group() {
    let mut state = scenario();
    let target = at(20, 8);
    let garrison = spawn_unit(&mut state, ENEMY, UnitKind::Spearman, target);
    let soldiers =
        [at(17, 8), at(17, 9)].map(|pos| spawn_unit(&mut state, ME, UnitKind::Spearman, pos));
    // Continued reconnaissance isolates staging exposure from the lost-sight regression.
    let scout = spawn_unit(&mut state, ME, UnitKind::Scout, target.neighbor(0));
    for id in soldiers {
        assert_eq!(state.units[&id].pos.distance(target), 3);
        assert_eq!(state.units[&id].hp, 100);
    }
    let mut orders = decide(&mut state, AiDifficulty::Normal);
    orders.retain(|o| legatum_core::rules::units::order_unit(o) != Some(scout));
    orders.push(Order::SkipUnit { unit: scout });
    assert!(player_view(&state, ME).tile(target).unwrap().visible);
    let (next, events) = play(&state, orders, vec![Order::SkipUnit { unit: garrison }]);
    assert_eq!(next.units[&garrison].pos, target, "stationary defender");
    for id in soldiers {
        assert_eq!(next.units[&id].hp, 100);
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, Event::Battle { attacker, .. } if *attacker == id))
        );
    }
}

#[test]
fn distant_support_does_not_commit_a_lone_soldier_to_an_unfavorable_city_attack() {
    let mut state = scenario();
    let target = at(20, 8);
    spawn_unit(&mut state, ENEMY, UnitKind::Spearman, target);
    let exposed = spawn_unit(&mut state, ME, UnitKind::Spearman, at(19, 8));
    spawn_unit(&mut state, ME, UnitKind::Spearman, at(17, 7));
    spawn_unit(&mut state, ME, UnitKind::Spearman, at(17, 9));
    let orders = decide(&mut state, AiDifficulty::Normal);
    assert!(
        !orders.contains(&Order::MoveUnit {
            unit: exposed,
            to: target
        }),
        "{orders:?}"
    );
    let (next, _) = play(&state, orders, vec![]);
    assert!(next.units[&exposed].pos.distance(target) > 2);
}

#[test]
fn shardfall_soldier_counters_a_ranger_in_or_near_the_city_within_two_turns() {
    for difficulty in [AiDifficulty::Normal, AiDifficulty::Hard] {
        for soldier_pos in [at(8, 8), at(7, 8)] {
            let mut state = scenario();
            let soldier = spawn_unit(&mut state, ME, UnitKind::Spearman, soldier_pos);
            // Moves were spent last turn; resolution gives the defender a fresh budget.
            state.units.get_mut(&soldier).unwrap().moves_left = 0;
            let ranger = spawn_unit(&mut state, ENEMY, UnitKind::Archer, at(9, 8));
            let mut countered = false;
            for _ in 0..2 {
                let ours = decide(&mut state, difficulty);
                let target = state.units[&soldier].pos;
                let theirs = vec![Order::RangedAttack {
                    unit: ranger,
                    target,
                }];
                let (next, events) = play(&state, ours, theirs);
                countered |= events.iter().any(|e| {
                    matches!(e,
                        Event::Battle { attacker, defender: Defender::Unit { unit, .. },
                            damage_to_defender, ranged: false, .. }
                            if *attacker == soldier && *unit == ranger && *damage_to_defender > 0
                    )
                });
                state = next;
                if countered {
                    break;
                }
            }
            assert!(countered, "{difficulty:?}, Soldier at {soldier_pos:?}");
        }
    }
}

#[test]
fn wounded_soldier_retreats_out_of_ranger_range_then_heals_and_fortifies() {
    let mut state = scenario();
    let soldier = spawn_unit(&mut state, ME, UnitKind::Spearman, at(10, 8));
    state.units.get_mut(&soldier).unwrap().hp = 45;
    let ranger = spawn_unit(&mut state, ENEMY, UnitKind::Archer, at(12, 8));
    let orders = decide(&mut state, AiDifficulty::Normal);
    let (mut next, _) = play(
        &state,
        orders,
        vec![Order::RangedAttack {
            unit: ranger,
            target: at(10, 8),
        }],
    );
    assert_eq!(next.units[&soldier].pos, at(8, 8), "retreat into Shardfall");
    assert!(next.units[&soldier].pos.distance(next.units[&ranger].pos) > 2);
    let hp = next.units[&soldier].hp;
    let orders = decide(&mut next, AiDifficulty::Normal);
    assert!(orders.contains(&Order::SkipUnit { unit: soldier }));
    let (healed, _) = play(&next, orders, vec![]);
    assert!(healed.units[&soldier].hp > hp);
    assert!(healed.units[&soldier].fortify > 0);
}

#[test]
fn threatened_city_switches_build_and_buys_a_defender_at_the_quoted_price() {
    for producing in [
        None,
        Some(ProductionItem::Unit {
            unit: UnitKind::Settler,
        }),
    ] {
        let mut state = scenario();
        let city = state.cities_of(ME).next().unwrap().id;
        state.cities.get_mut(&city).unwrap().producing = producing;
        // A Settler needs population 2 to be built at all.
        state.cities.get_mut(&city).unwrap().population = 2;
        let ranger = spawn_unit(&mut state, ENEMY, UnitKind::Archer, at(9, 8));
        update_explored(&mut state);
        // The healthy city absorbs the bombardment (Civ 6), so the AI buys a shooter
        // that answers it from safety.
        let shooter = ProductionItem::Unit {
            unit: UnitKind::Archer,
        };
        let quote = player_view(&state, ME)
            .cities
            .iter()
            .find(|c| c.id == city)
            .unwrap()
            .detail
            .as_ref()
            .unwrap()
            .options
            .iter()
            .find(|o| o.item == shooter)
            .unwrap()
            .buy_cost
            .unwrap();
        state.player_mut(ME).gold = quote + 100;
        let orders = decide(&mut state, AiDifficulty::Normal);
        if producing.is_some() {
            assert!(
                !orders
                    .iter()
                    .any(|o| matches!(o, Order::SetProduction { city: id, .. } if *id == city)),
                "purchase preserves construction"
            );
        }
        assert!(orders.contains(&Order::BuyItem {
            city,
            item: shooter
        }));
        let (mut next, events) = play(
            &state,
            orders,
            vec![Order::RangedAttack {
                unit: ranger,
                target: at(8, 8),
            }],
        );
        assert!(events.iter().any(
            |e| matches!(e, Event::ItemBought { city: id, item, gold, .. }
            if *id == city && *item == shooter && *gold == quote)
        ));
        assert!(next.units_of(ME).any(|u| u.kind == UnitKind::Archer));
        let orders = decide(&mut next, AiDifficulty::Normal);
        let (_, events) = play(&next, orders, vec![]);
        assert!(events.iter().any(
            |e| matches!(e, Event::Battle { attacker_owner, at: pos, .. }
            if *attacker_owner == ME && *pos == at(9, 8))
        ));

        state.player_mut(ME).gold = quote - 1;
        let orders = decide(&mut state, AiDifficulty::Normal);
        assert!(
            !orders.contains(&Order::BuyItem {
                city,
                item: shooter
            }),
            "cannot invent purchase gold"
        );
    }
}

#[test]
fn ranged_defender_returns_fire_and_prefers_a_damaged_harasser() {
    let mut state = scenario();
    let ranger = spawn_unit(&mut state, ME, UnitKind::Archer, at(8, 8));
    spawn_unit(&mut state, ENEMY, UnitKind::Archer, at(9, 8));
    let wounded = spawn_unit(&mut state, ENEMY, UnitKind::Archer, at(8, 9));
    state.units.get_mut(&wounded).unwrap().hp = 30;
    let orders = decide(&mut state, AiDifficulty::Normal);
    assert!(orders.contains(&Order::RangedAttack {
        unit: ranger,
        target: at(8, 9)
    }));
    let (_, events) = play(&state, orders, vec![]);
    assert!(events.iter().any(|e| matches!(e,
        Event::Battle { attacker, damage_to_defender, ranged: true, .. }
            if *attacker == ranger && *damage_to_defender > 0)));
}

#[test]
fn threatened_city_calls_in_a_reinforcement() {
    let mut state = scenario();
    let soldier = spawn_unit(&mut state, ME, UnitKind::Spearman, at(4, 8));
    spawn_unit(&mut state, ENEMY, UnitKind::Archer, at(9, 8));
    spawn_unit(&mut state, ENEMY, UnitKind::Spearman, at(10, 8));
    let orders = decide(&mut state, AiDifficulty::Normal);
    let (next, _) = play(&state, orders, vec![]);
    assert!(next.units[&soldier].pos.distance(at(8, 8)) < 4);
}

#[test]
fn healthy_unit_avoids_an_unfavorable_exposed_position() {
    let mut state = scenario();
    let soldier = spawn_unit(&mut state, ME, UnitKind::Spearman, at(11, 8));
    spawn_unit(&mut state, ENEMY, UnitKind::Musketman, at(12, 8));
    let orders = decide(&mut state, AiDifficulty::Normal);
    assert!(!orders.contains(&Order::SkipUnit { unit: soldier }));
    assert!(!orders.contains(&Order::MoveUnit {
        unit: soldier,
        to: at(12, 8)
    }));
    let (next, _) = play(&state, orders, vec![]);
    assert!(next.units[&soldier].pos.distance(at(12, 8)) > 2);
}

#[test]
fn a_melee_forecast_is_not_proof_of_reaching_the_ranger_this_turn() {
    let mut state = scenario();
    let soldier = spawn_unit(&mut state, ME, UnitKind::Spearman, at(8, 8));
    spawn_unit(&mut state, ENEMY, UnitKind::Archer, at(10, 8));
    state.map.get_mut(at(9, 8)).unwrap().terrain = Terrain::Mountains;
    let orders = decide(&mut state, AiDifficulty::Normal);
    assert!(!orders.contains(&Order::MoveUnit {
        unit: soldier,
        to: at(10, 8)
    }));
    // The Spearman garrisons the city: it holds against the bombardment instead of
    // retreating (a field unit retreats: see the exposed-position test above).
    let (next, _) = play(&state, orders, vec![]);
    assert_eq!(next.units[&soldier].pos, at(8, 8));
}

#[test]
fn a_trapped_wounded_ranger_still_returns_fire() {
    let mut state = scenario();
    let pos = at(8, 8);
    let ranger = spawn_unit(&mut state, ME, UnitKind::Archer, pos);
    state.units.get_mut(&ranger).unwrap().hp = 25;
    for h in pos.neighbors() {
        state.map.get_mut(h).unwrap().terrain = Terrain::Mountains;
    }
    // Keep the Archer trapped while allowing its shot through a water gap.
    state.map.get_mut(at(9, 8)).unwrap().terrain = Terrain::Ocean;
    spawn_unit(&mut state, ENEMY, UnitKind::Archer, at(10, 8));
    let orders = decide(&mut state, AiDifficulty::Normal);
    assert!(orders.contains(&Order::RangedAttack {
        unit: ranger,
        target: at(10, 8)
    }));
}

#[test]
fn two_rangers_concentrate_fire_on_a_wounded_unit() {
    let mut state = scenario();
    let a = spawn_unit(&mut state, ME, UnitKind::Archer, at(8, 8));
    let b = spawn_unit(&mut state, ME, UnitKind::Archer, at(8, 9));
    let target = at(9, 8);
    let wounded = spawn_unit(&mut state, ENEMY, UnitKind::Spearman, target);
    state.units.get_mut(&wounded).unwrap().hp = 55;
    spawn_unit(&mut state, ENEMY, UnitKind::Spearman, at(9, 9));
    let orders = decide(&mut state, AiDifficulty::Normal);
    for unit in [a, b] {
        assert!(orders.contains(&Order::RangedAttack { unit, target }));
    }
    let (_, events) = play(&state, orders, vec![]);
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(e, Event::Battle { at, attacker_owner, .. }
        if *at == target && *attacker_owner == ME))
            .count(),
        2
    );
}

#[test]
fn unseen_reinforcements_cannot_change_the_tactical_decision() {
    let mut state = scenario();
    spawn_unit(&mut state, ME, UnitKind::Spearman, at(8, 8));
    spawn_unit(&mut state, ENEMY, UnitKind::Archer, at(9, 8));
    let hidden = spawn_unit(&mut state, ENEMY, UnitKind::Spearman, at(20, 9));
    update_explored(&mut state);
    let before = player_view(&state, ME);
    assert!(!before.units.iter().any(|u| u.id == hidden));
    // HP contributes to the public Army title, so vary private position/orders instead.
    state.units.get_mut(&hidden).unwrap().fortify = 6;
    state.units.get_mut(&hidden).unwrap().goto = Some(at(19, 9));
    state.units.get_mut(&hidden).unwrap().pos = at(21, 9);
    assert_eq!(before, player_view(&state, ME));
    for difficulty in [AiDifficulty::Easy, AiDifficulty::Normal, AiDifficulty::Hard] {
        let orders = decide(&mut state, difficulty);
        assert_eq!(
            orders,
            ScriptedBrain::with_difficulty(before.faction_of(ME), state.setup.seed, ME, difficulty)
                .orders(&before)
        );
    }
}

#[test]
fn purchase_uses_cheapest_adequate_defender_and_saves_a_falling_city_after_reload() {
    let mut state = scenario();
    let city = state.cities_of(ME).next().unwrap().id;
    state.cities.get_mut(&city).unwrap().hp = 1;
    state.player_mut(ME).techs.insert(TechId::Feudalism);
    // Pikeman is over reserve, Spearman is affordable and survives this forecast.
    state.player_mut(ME).gold = 230;
    let enemy = spawn_unit(&mut state, ENEMY, UnitKind::Spearman, at(9, 8));
    let orders = decide(&mut state, AiDifficulty::Normal);
    let buy = Order::BuyItem {
        city,
        item: ProductionItem::Unit {
            unit: UnitKind::Spearman,
        },
    };
    assert!(orders.contains(&buy), "{orders:?}");
    let (saved, _) = play(
        &state,
        orders,
        vec![Order::MoveUnit {
            unit: enemy,
            to: at(8, 8),
        }],
    );
    assert_eq!(saved.player(ME).military_purchases, 1);
    assert_eq!(
        saved.cities[&city].owner, ME,
        "the purchased garrison prevents the capture"
    );
    state.player_mut(ME).gold = 2000;
    state.player_mut(ME).military_purchases = 3;
    assert!(decide(&mut state, AiDifficulty::Normal).contains(&buy));
    state.player_mut(ME).military_purchases = 4;
    let mut restored: GameState =
        serde_json::from_str(&serde_json::to_string(&state).unwrap()).unwrap();
    // The lifetime allowance limits optional spending, never rescue from a forecast capture.
    assert!(decide(&mut restored, AiDifficulty::Normal).contains(&buy));
}

#[test]
fn local_defense_and_unreachable_attackers_do_not_trigger_purchases() {
    let mut state = scenario();
    let city = state.cities_of(ME).next().unwrap().id;
    state.cities.get_mut(&city).unwrap().hp = 1;
    state.player_mut(ME).gold = 2000;
    // Two local defenders: the relief forecast answers the lone Spearman, and the
    // preemptive floor (a second defender beside visible enemy troops) is already met.
    spawn_unit(&mut state, ME, UnitKind::Spearman, at(8, 7));
    spawn_unit(&mut state, ME, UnitKind::Spearman, at(7, 7));
    spawn_unit(&mut state, ENEMY, UnitKind::Spearman, at(9, 8));
    assert!(
        !decide(&mut state, AiDifficulty::Normal)
            .iter()
            .any(|o| matches!(
                o,
                Order::BuyItem {
                    item: ProductionItem::Unit { .. },
                    ..
                }
            ))
    );
    state.units.retain(|_, u| u.owner != ME);
    state
        .units
        .values_mut()
        .find(|u| u.owner == ENEMY)
        .unwrap()
        .pos = at(10, 8);
    // Enemy within nominal move range, but no path on its movement budget.
    for h in at(8, 8).neighbors() {
        state.map.get_mut(h).unwrap().terrain = Terrain::Mountains;
    }
    assert!(
        !decide(&mut state, AiDifficulty::Normal)
            .iter()
            .any(|o| matches!(
                o,
                Order::BuyItem {
                    item: ProductionItem::Unit { .. },
                    ..
                }
            ))
    );
}

#[test]
fn swiss_discount_saves_a_falling_city_without_hoarding_the_goal_reserve() {
    let mut state = scenario();
    state.player_mut(ME).faction = Faction::Switzerland;
    let city = state.cities_of(ME).next().unwrap().id;
    let soldier = ProductionItem::Unit {
        unit: UnitKind::Spearman,
    };
    let buy_cost =
        legatum_core::economy::buy_cost_for(&state, &state.cities[&city], soldier).unwrap();
    assert_eq!(buy_cost, 140);
    state.player_mut(ME).gold = 1000;
    spawn_unit(&mut state, ENEMY, UnitKind::Spearman, at(9, 8));
    // A nearly fallen city needs a garrison that survives melee: the Spearman.
    state.cities.get_mut(&city).unwrap().hp = 1;
    // This is the first military unit, so V9 budgets its free slot at zero upkeep.
    let upkeep_buffer = (0 - player_view(&state, ME).economy.gold_per_turn.total).max(0);
    state.player_mut(ME).gold = buy_cost + upkeep_buffer - 1;
    assert!(
        !decide(&mut state, AiDifficulty::Normal)
            .iter()
            .any(|o| matches!(
                o,
                Order::BuyItem {
                    item: ProductionItem::Unit { .. },
                    ..
                }
            ))
    );
    state.player_mut(ME).gold = buy_cost + upkeep_buffer;
    let orders = decide(&mut state, AiDifficulty::Normal);
    assert!(orders.contains(&Order::BuyItem {
        city,
        item: soldier
    }));
    let (next, events) = play(&state, orders, vec![]);
    assert!(next.player(ME).gold < buy_cost);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::ItemBought { gold, .. } if *gold == buy_cost))
    );
}

#[test]
fn a_bombarded_garrison_holds_its_city_instead_of_leaving_it_to_the_infantry() {
    // Traced Proxy loss (seed 840003, turn 53): the capital's only Spearman stepped out
    // of reach of an Archer's bombardment and the Proxy's Spearman walked into the city.
    let mut state = scenario();
    let city = at(8, 8);
    let city_id = state.cities_of(ME).next().unwrap().id;
    // Earlier bombardment already wore the city down.
    state.cities.get_mut(&city_id).unwrap().hp = 25;
    let garrison = spawn_unit(&mut state, ME, UnitKind::Spearman, city);
    // Wounded enough that maximum rolls could kill it, but not certain to.
    state.units.get_mut(&garrison).unwrap().hp = 70;
    let ranger = spawn_unit(&mut state, ENEMY, UnitKind::Archer, at(10, 8));
    // A mountain keeps the Archer out of sortie reach, as bombardment usually is.
    state.map.get_mut(at(9, 8)).unwrap().terrain = Terrain::Mountains;
    let infantry = spawn_unit(&mut state, ENEMY, UnitKind::Spearman, at(9, 7));
    let attacks = vec![
        Order::MoveUnit {
            unit: ranger,
            to: city,
        },
        Order::MoveUnit {
            unit: infantry,
            to: city,
        },
    ];
    // Leaving the city would hand it over this turn.
    let (abandoned, _) = play(
        &state,
        vec![Order::MoveUnit {
            unit: garrison,
            to: at(5, 8),
        }],
        attacks.clone(),
    );
    assert_eq!(abandoned.cities[&city_id].owner, ENEMY);
    for difficulty in [AiDifficulty::Normal, AiDifficulty::Hard] {
        let mut state = state.clone();
        let orders = decide(&mut state, difficulty);
        assert!(
            !orders.iter().any(|o| matches!(o,
                Order::MoveUnit { unit, to } if *unit == garrison && *to != city
                    && state.units.values().all(|e| e.owner == ME || e.pos != *to))),
            "the garrison must hold (or strike), not walk away: {orders:?}"
        );
        let (next, _) = play(&state, orders, attacks.clone());
        assert_eq!(next.cities[&city_id].owner, ME, "{difficulty:?}");
    }
}
