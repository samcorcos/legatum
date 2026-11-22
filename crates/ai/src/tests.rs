//! Behavior tests on hand-built views. Every brain output is checked with `assert_valid`.

use crate::fixture::*;
use crate::{Brain, Knobs, RandomBrain, ScriptedBrain};
use legatum_core::*;

fn run(view: &PlayerView, knobs: Knobs) -> Vec<Order> {
    let mut b = ScriptedBrain::with_knobs(Faction::Switzerland, 7, view.me, knobs);
    // These tests predate the capitals victory (gene `capitals`).
    b.decisions.capitals = 0;
    let orders = b.orders(view);
    assert_valid(view, &orders);
    orders
}

fn scripted(view: &PlayerView) -> Vec<Order> {
    run(view, Knobs::for_faction(Faction::Switzerland))
}

fn order_for(orders: &[Order], unit: u32) -> Option<&Order> {
    let id = UnitId(unit);
    orders.iter().find(|o| match o {
        Order::MoveUnit { unit, .. }
        | Order::MoveUnitPath { unit, .. }
        | Order::SkipUnit { unit }
        | Order::FoundCity { unit }
        | Order::DisbandUnit { unit }
        | Order::RangedAttack { unit, .. }
        | Order::UpgradeUnit { unit } => *unit == id,
        _ => false,
    })
}

fn move_target(orders: &[Order], unit: u32) -> Option<Hex> {
    match order_for(orders, unit) {
        Some(Order::MoveUnit { to, .. }) => Some(*to),
        _ => None,
    }
}

#[test]
fn every_difficulty_picks_opening_capital_production_without_founding() {
    let state = new_game(&GameSetup::all_ai(4545));
    let view = player_view(&state, PlayerId(0));
    let capital = state.capital_of(PlayerId(0)).unwrap().id;
    for difficulty in [AiDifficulty::Easy, AiDifficulty::Normal, AiDifficulty::Hard] {
        let mut brain = ScriptedBrain::with_difficulty(
            state.player(PlayerId(0)).faction,
            4545,
            PlayerId(0),
            difficulty,
        );
        let orders = brain.orders(&view);
        assert_valid(&view, &orders);
        assert!(
            orders.iter().any(|order| matches!(order,
            Order::SetProduction { city, .. } if *city == capital)),
            "{difficulty:?}"
        );
        assert!(
            !orders
                .iter()
                .any(|order| matches!(order, Order::FoundCity { .. }))
        );
    }
}

#[test]
fn idle_spearman_collects_known_ruin_inside_own_borders() {
    let ruin = off(8, 5);
    let mut view = ViewBuilder::new(12, 12)
        .city(1, ME, off(5, 5), 3)
        .owner(ruin, ME)
        .ruin(ruin)
        .unit(1, ME, UnitKind::Spearman, off(6, 5))
        .build();
    for _ in 0..3 {
        let orders = scripted(&view);
        let next = move_target(&orders, 1).expect("idle Spearman should travel toward ruin");
        assert!(next.distance(ruin) < view.units[0].pos.distance(ruin));
        view.units[0].pos = next;
        if next == ruin {
            return;
        }
        view.turn += 1;
    }
    panic!("Spearman did not reach the ruin within three planned turns");
}

#[test]
fn threatened_city_garrison_stays_put_for_nearby_ruin() {
    let home = off(5, 5);
    let ruin = off(7, 5);
    let view = ViewBuilder::new(14, 12)
        .city(1, ME, home, 3)
        .unit(2, ENEMY, UnitKind::Spearman, off(4, 7))
        .owner(ruin, ME)
        .ruin(ruin)
        .unit(1, ME, UnitKind::Spearman, home)
        .build();
    let orders = scripted(&view);
    assert_ne!(move_target(&orders, 1), Some(ruin));
}

#[test]
fn quiet_city_garrison_can_collect_nearby_ruin() {
    let home = off(5, 5);
    let ruin = off(7, 5);
    let view = ViewBuilder::new(12, 12)
        .city(1, ME, home, 3)
        .owner(ruin, ME)
        .ruin(ruin)
        .unit(1, ME, UnitKind::Spearman, home)
        .build();
    let orders = scripted(&view);
    assert!(move_target(&orders, 1).is_some_and(|to| to.distance(ruin) < home.distance(ruin)));
}

#[test]
fn military_does_not_duplicate_a_scouts_ruin_order() {
    let ruin = off(8, 5);
    let view = ViewBuilder::new(12, 12)
        .city(1, ME, off(5, 5), 3)
        .owner(ruin, ME)
        .ruin(ruin)
        .unit(1, ME, UnitKind::Scout, off(7, 5))
        .unit(2, ME, UnitKind::Spearman, off(6, 5))
        .build();
    let orders = scripted(&view);
    assert_eq!(move_target(&orders, 1), Some(ruin));
    assert_ne!(move_target(&orders, 2), Some(ruin));
}

/// A mid-game view on the full 48x32 map: 6 own cities, ~30 own units, enemies and fog.
fn midgame_view() -> PlayerView {
    let mut b = ViewBuilder::new(48, 32)
        .unexplored_from_col(34)
        .patch(off(10, 10), 2, Terrain::Forest)
        .patch(off(20, 20), 1, Terrain::Hills)
        .patch(off(15, 5), 1, Terrain::Mountains)
        .patch(off(28, 14), 2, Terrain::Grassland)
        .ruin(off(30, 3))
        .known_techs(&[TechId::Irrigation, TechId::BronzeWorking])
        .gold(250);
    let cities = [(5, 5), (5, 12), (5, 20), (12, 8), (12, 16), (18, 12)];
    for (i, (c, r)) in cities.iter().enumerate() {
        b = b.city(i as u32 + 1, ME, off(*c, *r), 3 + i as i32);
    }
    b = b
        .producing(
            1,
            ProductionItem::Unit {
                unit: UnitKind::Spearman,
            },
            30,
        )
        .city(20, ENEMY, off(26, 22), 4)
        .city(21, PlayerId(3), off(30, 8), 3)
        .city(22, MATE, off(5, 28), 2);
    let mut id = 100;
    for (i, (c, r)) in cities.iter().enumerate() {
        let kinds = [UnitKind::Spearman, UnitKind::Archer, UnitKind::Spearman];
        for (k, kind) in kinds.iter().enumerate() {
            let pos = if k == 0 {
                off(*c, *r)
            } else {
                off(c + k as i32, *r + (i % 2) as i32)
            };
            b = b.unit(id, ME, *kind, pos);
            id += 1;
        }
    }
    for i in 0..6 {
        b = b.unit(id, ME, UnitKind::Scout, off(20 + i, 2 + 4 * i));
        id += 1;
    }
    for i in 0..4 {
        b = b.unit(id, ME, UnitKind::Settler, off(14 + i, 24 + i));
        id += 1;
    }
    for i in 0..10 {
        let kind = if i % 3 == 0 {
            UnitKind::Archer
        } else {
            UnitKind::Spearman
        };
        b = b.unit_hp(id, ENEMY, kind, off(22 + i % 5, 18 + i / 2), 60 + 4 * i);
        id += 1;
    }
    b = b.unit(id, MATE, UnitKind::Spearman, off(5, 27));
    b.build()
}

#[test]
fn research_gets_set_when_idle_and_left_alone_otherwise() {
    let view = ViewBuilder::new(12, 12).build();
    let orders = scripted(&view);
    let research: Vec<_> = orders
        .iter()
        .filter(|o| matches!(o, Order::SetResearchQueue { .. }))
        .collect();
    assert_eq!(research.len(), 1);

    // With that plan running, the next turn leaves it alone.
    let mut busy = ViewBuilder::new(12, 12).build();
    let queue = crate::fixture::research_queue(&orders).unwrap();
    busy.research.current = queue.first().copied();
    busy.research.queue = queue;
    assert!(!scripted(&busy).iter().any(|o| matches!(
        o,
        Order::SetResearch { .. } | Order::SetResearchQueue { .. }
    )));
}

#[test]
fn science_knob_pulls_research_toward_science_techs() {
    let view = ViewBuilder::new(12, 12)
        .known_techs(&[TechId::BronzeWorking, TechId::Irrigation])
        .build();
    let nerd = run(&view, Knobs::new(0, 0, 100, 0, 0));
    assert_eq!(crate::fixture::research_head(&nerd), Some(TechId::Writing));
}

#[test]
fn idle_city_gets_production_busy_city_does_not() {
    let view = ViewBuilder::new(16, 16)
        .city(1, ME, off(4, 4), 3)
        .city(2, ME, off(11, 11), 3)
        .producing(
            2,
            ProductionItem::Building {
                building: BuildingKind::Granary,
            },
            60,
        )
        .build();
    let orders = scripted(&view);
    assert!(
        orders
            .iter()
            .any(|o| matches!(o, Order::SetProduction { city, .. } if *city == CityId(1)))
    );
    assert!(
        !orders
            .iter()
            .any(|o| matches!(o, Order::SetProduction { city, .. } if *city == CityId(2)))
    );
}

#[test]
fn expansionist_builds_settlers() {
    let view = ViewBuilder::new(20, 20).city(1, ME, off(5, 5), 2).build();
    let orders = run(&view, Knobs::new(100, 0, 0, 0, 0));
    assert!(orders.contains(&Order::SetProduction {
        city: CityId(1),
        item: ProductionItem::Unit {
            unit: UnitKind::Settler
        }
    }));
}

fn production_for(orders: &[Order], city: u32) -> Option<ProductionItem> {
    orders.iter().find_map(|o| match o {
        Order::SetProduction { city: id, item } if *id == CityId(city) => Some(*item),
        _ => None,
    })
}

#[test]
fn exposed_openings_reinforce_the_starting_spearman_for_every_faction() {
    for faction in Faction::ALL {
        for fog in [false, true] {
            let mut b = ViewBuilder::new(24, 16)
                .faction(faction)
                .city(1, ME, off(5, 8), 1)
                .unit(1, ME, UnitKind::Spearman, off(5, 8));
            if !fog {
                // Outside the immediate three-tile military threat radius.
                b = b.city(2, ENEMY, off(10, 8), 1);
            }
            let mut view = b.build();
            if fog {
                // A broad open perimeter, with only the capital's neighborhood explored.
                for (i, tile) in view.tiles.iter_mut().enumerate() {
                    let h = off(i as i32 % view.width, i as i32 / view.width);
                    if h.distance(off(5, 8)) > 3 {
                        tile.explored = false;
                        tile.visible = false;
                        tile.terrain = None;
                    }
                }
            }
            view.turn = 2;
            // As in core, a population-1 capital can't choose its Settler yet: it is listed
            // as unavailable, and the exposed opening builds its defender first.
            let d = view.cities[0].detail.as_mut().unwrap();
            let settler = ProductionItem::Unit {
                unit: UnitKind::Settler,
            };
            let o = d.options.iter().position(|o| o.item == settler).unwrap();
            let o = d.options.remove(o);
            d.unavailable.push(legatum_core::view::UnavailableOption {
                item: settler,
                cost: o.cost,
                reason: "Settler needs population 2 (currently 1)".into(),
                prerequisite_building: None,
                production_banked: 0,
            });
            let orders = ScriptedBrain::new(faction, 7, ME).orders(&view);
            assert_valid(&view, &orders);
            assert_eq!(
                production_for(&orders, 1),
                Some(ProductionItem::Unit {
                    unit: UnitKind::Spearman
                }),
                "{faction:?}, fog={fog}"
            );
        }
    }
}

#[test]
fn safe_opening_expands_then_reinforces_even_after_the_settler_is_gone() {
    let mut view = ViewBuilder::new(24, 16)
        .unexplored_from_col(9)
        .city(1, ME, off(5, 8), 2)
        .unit(1, ME, UnitKind::Spearman, off(5, 8))
        .ruin(off(8, 8))
        .build();
    assert_eq!(
        production_for(&scripted(&view), 1),
        Some(ProductionItem::Unit {
            unit: UnitKind::Settler
        })
    );
    // The first trained Settler has founded or been lost; its surcharge survives either.
    for o in &mut view.cities[0].detail.as_mut().unwrap().options {
        if o.item
            == (ProductionItem::Unit {
                unit: UnitKind::Settler,
            })
        {
            o.cost += 20;
        }
    }
    view.turn = 25;
    assert_eq!(
        production_for(&scripted(&view), 1),
        Some(ProductionItem::Unit {
            unit: UnitKind::Spearman
        })
    );
}

#[test]
fn hard_reinforces_a_known_rival() {
    let capital = off(5, 8);
    let site = off(5, 2);
    let mut view = ViewBuilder::new(24, 16)
        .city(1, ME, capital, 2)
        .unit(1, ME, UnitKind::Spearman, capital)
        .patch(site, 2, Terrain::Forest)
        .build();
    view.turn = 4;
    // The empire's third Settler (170) costs twice a Spearman (80), the relation the test's
    // Spearman-first opening was written for.
    settlers_trained(&mut view, 2);
    for (i, tile) in view.tiles.iter_mut().enumerate() {
        let h = off(i as i32 % view.width, i as i32 / view.width);
        if h.distance(capital) > 3 && h.distance(site) > 2 {
            tile.explored = false;
            tile.visible = false;
            tile.terrain = None;
        }
    }
    let decide = |view: &PlayerView, difficulty, modernization| {
        let mut brain = ScriptedBrain::with_difficulty(Faction::Switzerland, 7, ME, difficulty);
        brain.decisions.modernization_value = modernization;
        // The reinforcement policy without the priced colony: with `colony_compound` on, a
        // one-Spearman capital at peace trains its Settler first (`colony_compound_tests`).
        brain.decisions.colony_compound = 0;
        // The plain Hard reinforcement, not the Redoubt garrison (`swiss_playbook`,
        // `walls_garrison`).
        brain.decisions.swiss_playbook = 0;
        brain.decisions.walls_garrison = 0;
        let orders = brain.orders(view);
        assert_valid(view, &orders);
        production_for(&orders, 1)
    };
    assert_eq!(
        decide(&view, AiDifficulty::Normal, 100),
        Some(ProductionItem::Unit {
            unit: UnitKind::Spearman
        })
    );
    view.cities.push(
        ViewBuilder::new(24, 16)
            .city(2, ENEMY, off(8, 8), 1)
            .build()
            .cities
            .remove(0),
    );
    for modernization in [0, 100] {
        // Both policies reinforce. Squared strength favors the Spearman's 20 points
        // over the Archer's 11 here; zero preserves the old ranged-composition bonus.
        assert_eq!(
            decide(&view, AiDifficulty::Hard, modernization),
            Some(ProductionItem::Unit {
                unit: if modernization == 0 {
                    UnitKind::Archer
                } else {
                    UnitKind::Spearman
                }
            })
        );
    }
}

#[test]
fn disconnected_rivals_and_teammates_do_not_trigger_opening_defense() {
    let mut b = ViewBuilder::new(24, 16)
        .city(1, ME, off(5, 8), 2)
        .city(2, MATE, off(5, 13), 1)
        .city(3, ENEMY, off(10, 8), 1)
        .unit(1, ME, UnitKind::Spearman, off(5, 8))
        .unit(2, MATE, UnitKind::Spearman, off(6, 8));
    for row in 0..16 {
        b = b.terrain(off(7, row), Terrain::Mountains);
    }
    assert_eq!(
        production_for(&scripted(&b.build()), 1),
        Some(ProductionItem::Unit {
            unit: UnitKind::Settler
        })
    );
}

#[test]
fn opening_defender_in_production_counts_toward_the_reinforcement() {
    let view = ViewBuilder::new(24, 16)
        .city(1, ME, off(5, 8), 2)
        .city(2, ME, off(5, 13), 2)
        .city(3, ENEMY, off(10, 8), 1)
        .unit(1, ME, UnitKind::Spearman, off(5, 8))
        .producing(
            1,
            ProductionItem::Unit {
                unit: UnitKind::Spearman,
            },
            80,
        )
        .build();
    let orders = scripted(&view);
    // The capital keeps an armed build: its Spearman or, with no enemy in sight, a Horseman
    // (riders stand above the melee line since 2026-10-31).
    assert!(
        production_for(&orders, 1).is_none_or(|item| matches!(item,
            ProductionItem::Unit { unit } if legatum_core::data::unit_stats(unit).military)),
        "{orders:?}"
    );
    assert_eq!(
        production_for(&orders, 2),
        Some(ProductionItem::Unit {
            unit: UnitKind::Settler
        })
    );
}

#[test]
fn opening_defense_prices_reinforcements_without_speculative_buying() {
    let mut view = ViewBuilder::new(24, 16)
        .city(1, ME, off(5, 8), 2)
        .city(2, ENEMY, off(10, 8), 1)
        .unit(1, ME, UnitKind::Spearman, off(5, 8))
        .build();
    view.turn = 8;
    // The third Settler costs twice a Spearman: the relation this test was written for.
    settlers_trained(&mut view, 2);
    let f = Faction::Switzerland;
    let orders = ScriptedBrain::with_difficulty(f, 7, ME, AiDifficulty::Easy).orders(&view);
    assert_valid(&view, &orders);
    assert_eq!(
        production_for(&orders, 1),
        Some(ProductionItem::Unit {
            unit: UnitKind::Spearman
        })
    );
    for modernization in [0, 100] {
        for difficulty in [AiDifficulty::Normal, AiDifficulty::Hard] {
            let mut brain = ScriptedBrain::with_difficulty(f, 7, ME, difficulty);
            brain.decisions.modernization_value = modernization;
            // The opening-defense policy without the priced colony (`colony_compound`).
            brain.decisions.colony_compound = 0;
            // The plain Hard reinforcement, not the Redoubt garrison (`swiss_playbook`,
            // `walls_garrison`).
            brain.decisions.swiss_playbook = 0;
            brain.decisions.walls_garrison = 0;
            let orders = brain.orders(&view);
            assert_valid(&view, &orders);
            assert_eq!(
                production_for(&orders, 1),
                Some(ProductionItem::Unit {
                    unit: if difficulty == AiDifficulty::Hard && modernization == 0 {
                        UnitKind::Archer
                    } else {
                        UnitKind::Spearman
                    }
                })
            );
        }
    }

    let mut v = view.clone();
    v.turn = 16;
    v.economy.gold = 60;
    let d = &mut **v.cities[0].detail.as_mut().unwrap();
    d.producing = Some(ProductionItem::Unit {
        unit: UnitKind::Spearman,
    });
    d.turns_to_complete = Some(8);
    d.buy_cost = Some(40);
    d.options
        .iter_mut()
        .find(|o| o.item == d.producing.unwrap())
        .unwrap()
        .buy_cost = d.buy_cost;
    let buy = Order::BuyItem {
        city: CityId(1),
        item: ProductionItem::Unit {
            unit: UnitKind::Spearman,
        },
    };
    let normal = ScriptedBrain::with_difficulty(f, 7, ME, AiDifficulty::Normal).orders(&v);
    let hard = ScriptedBrain::with_difficulty(f, 7, ME, AiDifficulty::Hard).orders(&v);
    assert_valid(&v, &normal);
    assert_valid(&v, &hard);
    assert!(!normal.contains(&buy));
    assert!(!hard.contains(&buy), "no forecast emergency");
    v.economy.gold = 38;
    assert!(
        !ScriptedBrain::with_difficulty(f, 7, ME, AiDifficulty::Hard)
            .orders(&v)
            .contains(&buy),
        "cannot buy without enough earned gold"
    );
}

#[test]
fn first_settler_purchase_preserves_the_swiss_goal_and_deficit_reserve() {
    let mut view = ViewBuilder::new(24, 16)
        .city(1, ME, off(5, 8), 2)
        .gold(40)
        .producing(
            1,
            ProductionItem::Unit {
                unit: UnitKind::Settler,
            },
            70,
        )
        .build();
    let buy = Order::BuyItem {
        city: CityId(1),
        item: ProductionItem::Unit {
            unit: UnitKind::Settler,
        },
    };
    assert!(!scripted(&view).contains(&buy));
    view.economy.gold = 1150;
    // Buying is now independent: do not duplicate the settler already in the queue.
    assert!(!scripted(&view).contains(&buy));
    view.cities[0].detail.as_mut().unwrap().producing = None;
    assert!(scripted(&view).contains(&buy));
    view.economy.gold = 600;
    view.economy.gold_per_turn.total = -100;
    assert!(!scripted(&view).contains(&buy));
}

#[test]
fn athens_build_a_workshop_then_the_lab_network() {
    let mut view = ViewBuilder::new(20, 16)
        .faction(Faction::Athens)
        .known_techs(&TechId::ALL)
        .city(1, ME, off(5, 8), 3)
        .build();
    view.turn = 60;
    let workshop = ProductionItem::Building {
        building: BuildingKind::Workshop,
    };
    let relay = ProductionItem::Building {
        building: BuildingKind::GreatLibrary,
    };
    let lab = ProductionItem::Building {
        building: BuildingKind::Library,
    };
    view.cities[0]
        .detail
        .as_mut()
        .unwrap()
        .options
        .retain(|o| o.item == workshop || o.item == lab || o.item == relay);
    let orders = ScriptedBrain::new(Faction::Athens, 7, ME).orders(&view);
    assert_valid(&view, &orders);
    assert_eq!(production_for(&orders, 1), Some(workshop));
    view.cities[0]
        .detail
        .as_mut()
        .unwrap()
        .buildings
        .push(BuildingKind::Workshop);
    let orders = ScriptedBrain::new(Faction::Athens, 7, ME).orders(&view);
    assert_valid(&view, &orders);
    assert_eq!(production_for(&orders, 1), Some(lab));
}

#[test]
fn threatened_idle_city_builds_a_defender() {
    let view = ViewBuilder::new(16, 16)
        .city(1, ME, off(5, 5), 3)
        .unit(50, ENEMY, UnitKind::Spearman, off(7, 5))
        .unit(51, ENEMY, UnitKind::Spearman, off(7, 6))
        .build();
    let orders = scripted(&view);
    let item = orders.iter().find_map(|o| match o {
        Order::SetProduction { item, .. } => Some(*item),
        _ => None,
    });
    assert!(matches!(
        item,
        Some(ProductionItem::Unit {
            unit: UnitKind::Spearman | UnitKind::Archer
        })
    ));
}

#[test]
fn buys_with_plentiful_gold_but_never_the_megaproject() {
    let base = || {
        ViewBuilder::new(16, 16)
            .faction(Faction::India)
            .city(1, ME, off(4, 4), 3)
            .gold(1000)
    };
    let view = base()
        .producing(
            1,
            ProductionItem::Building {
                building: BuildingKind::Granary,
            },
            40,
        )
        .build();
    let mut b = ScriptedBrain::new(Faction::India, 1, ME);
    let orders = b.orders(&view);
    assert_valid(&view, &orders);
    assert!(orders.contains(&Order::BuyItem {
        city: CityId(1),
        item: ProductionItem::Building {
            building: BuildingKind::Granary
        }
    }));

    let relay = base()
        .producing(
            1,
            ProductionItem::Building {
                building: BuildingKind::GreatLibrary,
            },
            500,
        )
        .build();
    let orders = b.orders(&relay);
    assert!(!orders.iter().any(|o| matches!(o, Order::BuyItem { .. })));
}

#[test]
fn scout_heads_to_frontier() {
    let view = ViewBuilder::new(20, 12)
        .unexplored_from_col(12)
        .unit(1, ME, UnitKind::Scout, off(3, 6))
        .build();
    let orders = scripted(&view);
    let to = move_target(&orders, 1).expect("scout moves");
    let (col, _) = to.to_offset();
    assert_eq!(col, 11, "frontier is the last explored column, got {to:?}");
}

#[test]
fn scout_prefers_nearby_ruin_and_avoids_enemy_army() {
    let view = ViewBuilder::new(24, 12)
        .unexplored_from_col(20)
        .ruin(off(6, 6))
        .unit(1, ME, UnitKind::Scout, off(3, 6))
        .build();
    assert_eq!(move_target(&scripted(&view), 1), Some(off(6, 6)));

    let guarded = ViewBuilder::new(24, 12)
        .unexplored_from_col(20)
        .ruin(off(6, 6))
        .unit(1, ME, UnitKind::Scout, off(3, 6))
        .unit(9, ENEMY, UnitKind::Spearman, off(7, 6))
        .build();
    let to = move_target(&scripted(&guarded), 1).expect("scout moves");
    assert!(
        to.distance(off(7, 6)) > 2,
        "scout walked into danger: {to:?}"
    );
}

/// Plains everywhere with a richer but more distant forest patch.
fn two_sites() -> (ViewBuilder, Hex) {
    let forest = off(12, 8);
    let b = ViewBuilder::new(24, 18)
        .city(1, ME, off(6, 8), 3)
        .patch(forest, 2, Terrain::Forest);
    (b, forest)
}

#[test]
fn settler_values_early_arrival_and_founds_on_arrival() {
    let (b, forest) = two_sites();
    let view = b.unit(1, ME, UnitKind::Settler, off(6, 8)).build();
    let sites = legatum_core::settlement::settlement_sites(&view, &view.units[0], &[]);
    let near = sites.iter().find(|s| s.rank == Some(1)).unwrap().pos;
    assert!(near.distance(view.units[0].pos) < forest.distance(view.units[0].pos));
    assert_eq!(move_target(&scripted(&view), 1), Some(near));

    let (b, _) = two_sites();
    // Just arrived: it walked last turn.
    let view = b.unit(1, ME, UnitKind::Settler, near).goto(1, near).build();
    assert_eq!(
        order_for(&scripted(&view), 1),
        Some(&Order::FoundCity { unit: UnitId(1) })
    );
}

#[test]
fn first_settler_founds_quickly() {
    let mut view = ViewBuilder::new(16, 16)
        .unit(1, ME, UnitKind::Settler, off(8, 8))
        .build();
    view.turn = 1;
    // All plains: the start tile is as good as any, so found right away.
    assert_eq!(
        order_for(&scripted(&view), 1),
        Some(&Order::FoundCity { unit: UnitId(1) })
    );
}

#[test]
fn never_founds_within_three_of_a_city() {
    // Settler right next to an enemy city: every nearby tile is too close.
    for dist in 1..=3 {
        let view = ViewBuilder::new(16, 16)
            .city(1, ENEMY, off(8, 8), 3)
            .unit(1, ME, UnitKind::Settler, off(8 + dist, 8))
            .build();
        let orders = scripted(&view);
        assert!(
            !orders.iter().any(|o| matches!(o, Order::FoundCity { .. })),
            "founded at distance {dist}"
        );
        // A wartime route may be explicitly confirmed so it cannot be recomputed through
        // remembered enemy land; peaceful orders retain their destination-only form.
        let to = orders
            .iter()
            .find_map(|o| match o {
                Order::MoveUnit {
                    unit: UnitId(1),
                    to,
                }
                | Order::MoveUnitPath {
                    unit: UnitId(1),
                    to,
                    ..
                } => Some(*to),
                _ => None,
            })
            .expect("settler moves to a legal site");
        assert!(to.distance(off(8, 8)) > 3);
    }
    // Two settlers near each other never pick sites within 3 of each other.
    let view = ViewBuilder::new(24, 18)
        .city(1, ME, off(4, 8), 3)
        .unit(1, ME, UnitKind::Settler, off(4, 8))
        .unit(2, ME, UnitKind::Settler, off(4, 8))
        .build();
    let orders = scripted(&view);
    let a = move_target(&orders, 1).expect("settler 1 moves");
    let b = move_target(&orders, 2).expect("settler 2 moves");
    assert!(a.distance(b) > 3, "{a:?} vs {b:?}");
}

#[test]
fn settler_retreats_from_enemy_army() {
    let view = ViewBuilder::new(20, 16)
        .city(1, ME, off(4, 8), 3)
        .unit(1, ME, UnitKind::Settler, off(9, 8))
        .unit(9, ENEMY, UnitKind::Spearman, off(10, 8))
        .build();
    let to = move_target(&scripted(&view), 1).expect("settler flees");
    assert!(to.distance(off(9, 8)) <= 2, "reach safety this turn");
    assert!(to.distance(off(10, 8)) > 2, "outside enemy attack reach");
    assert!(to.distance(off(4, 8)) < 5, "toward home");
}

#[test]
fn military_defends_a_threatened_city() {
    let city = off(6, 6);
    let view = ViewBuilder::new(20, 16)
        .city(1, ME, city, 2)
        .unit(1, ME, UnitKind::Spearman, off(10, 6))
        .unit(50, ENEMY, UnitKind::Spearman, off(4, 5))
        .unit(51, ENEMY, UnitKind::Spearman, off(4, 7))
        .build();
    let orders = scripted(&view);
    let to = move_target(&orders, 1).expect("defender moves");
    assert!(to.distance(city) <= 1, "defender went to {to:?}");
}

#[test]
fn garrison_holds_in_its_city() {
    let city = off(6, 6);
    let view = ViewBuilder::new(20, 16)
        .city(1, ME, city, 2)
        .unit(1, ME, UnitKind::Spearman, city)
        .build();
    assert_eq!(
        order_for(&scripted(&view), 1),
        Some(&Order::SkipUnit { unit: UnitId(1) })
    );
}

/// Our full-health Spearman (20) next to an enemy Spearman at 90 HP (19 x 0.9 = 17): a ratio of
/// about 117%, between the aggression-100 threshold (90%) and the aggression-0 one (130%).
fn weak_neighbor_view() -> PlayerView {
    ViewBuilder::new(24, 16)
        .city(1, ME, off(3, 8), 3)
        .unit(2, ME, UnitKind::Spearman, off(3, 8))
        .unit(1, ME, UnitKind::Spearman, off(14, 8))
        .unit_hp(50, ENEMY, UnitKind::Spearman, off(15, 8), 90)
        .build()
}

#[test]
fn attacks_a_weak_neighbor_only_when_aggressive() {
    let view = weak_neighbor_view();
    let bold = run(&view, Knobs::new(50, 50, 50, 50, 100));
    assert_eq!(move_target(&bold, 1), Some(off(15, 8)));

    let timid = run(&view, Knobs::new(50, 50, 50, 50, 0));
    assert_ne!(move_target(&timid, 1), Some(off(15, 8)));
}

#[test]
fn does_not_attack_a_strong_city() {
    let view = ViewBuilder::new(24, 16)
        .city(1, ME, off(3, 8), 3)
        .unit(2, ME, UnitKind::Spearman, off(3, 8))
        .unit(1, ME, UnitKind::Spearman, off(13, 8))
        .city(9, ENEMY, off(15, 8), 8)
        .unit(50, ENEMY, UnitKind::Spearman, off(15, 8))
        .build();
    let orders = run(&view, Knobs::new(50, 50, 50, 50, 100));
    assert_ne!(move_target(&orders, 1), Some(off(15, 8)));
}

#[test]
fn captures_an_unescorted_enemy_settler() {
    let view = ViewBuilder::new(24, 16)
        .city(1, ME, off(3, 8), 3)
        .unit(2, ME, UnitKind::Spearman, off(3, 8))
        .unit(1, ME, UnitKind::Spearman, off(12, 8))
        .unit(50, ENEMY, UnitKind::Settler, off(14, 8))
        .build();
    let orders = run(&view, Knobs::new(50, 50, 50, 50, 0));
    assert_eq!(move_target(&orders, 1), Some(off(14, 8)));
}

#[test]
fn archer_shoots_enemy_in_range() {
    let view = ViewBuilder::new(20, 16)
        .city(1, ME, off(6, 6), 2)
        .unit(1, ME, UnitKind::Archer, off(6, 6))
        .unit(50, ENEMY, UnitKind::Spearman, off(8, 6))
        .build();
    assert_eq!(
        order_for(&scripted(&view), 1),
        Some(&Order::RangedAttack {
            unit: UnitId(1),
            target: off(8, 6)
        })
    );
}

#[test]
fn never_orders_a_teammates_unit_or_city() {
    let view = ViewBuilder::new(20, 16)
        .city(1, ME, off(4, 4), 3)
        .city_with_detail(2, MATE, off(12, 10), 3)
        .unit(1, ME, UnitKind::Spearman, off(4, 4))
        .unit(2, MATE, UnitKind::Settler, off(14, 12))
        .unit(3, MATE, UnitKind::Scout, off(10, 4))
        .unit(4, MATE, UnitKind::Spearman, off(12, 10))
        .unit(5, MATE, UnitKind::Archer, off(13, 10))
        .unit(50, ENEMY, UnitKind::Spearman, off(15, 10))
        .build();
    // assert_valid (inside scripted) rejects any order for a unit or city that isn't ours.
    let orders = scripted(&view);
    assert!(order_for(&orders, 1).is_some());
    for u in 2..=5 {
        assert!(order_for(&orders, u).is_none());
    }
}

#[test]
fn deterministic_for_same_view_and_seed() {
    let view = midgame_view();
    for faction in Faction::ALL {
        let a = ScriptedBrain::new(faction, 99, ME).orders(&view);
        let b = ScriptedBrain::new(faction, 99, ME).orders(&view);
        assert_eq!(a, b);
        assert_valid(&view, &a);
    }
    let r1 = RandomBrain::new(5, ME).orders(&view);
    let r2 = RandomBrain::new(5, ME).orders(&view);
    assert_eq!(r1, r2);
}

#[test]
fn midgame_orders_every_own_unit_and_idle_city() {
    let view = midgame_view();
    let orders = scripted(&view);
    for u in view.units.iter().filter(|u| u.owner == ME) {
        assert!(order_for(&orders, u.id.0).is_some(), "unit {:?} idle", u.id);
    }
    let idle = view
        .cities
        .iter()
        .filter(|c| c.owner == ME && c.detail.as_ref().is_some_and(|d| d.producing.is_none()));
    for c in idle {
        assert!(
            orders
                .iter()
                .any(|o| matches!(o, Order::SetProduction { city, .. } if *city == c.id))
        );
    }
}

#[test]
fn random_brain_only_references_own_units_and_cities() {
    let view = midgame_view();
    for seed in 0..20 {
        let orders = RandomBrain::new(seed, ME).orders(&view);
        assert_valid(&view, &orders);
        assert!(!orders.is_empty());
    }
}

#[test]
fn finished_game_gives_no_orders() {
    let mut view = midgame_view();
    view.finished = true;
    assert!(scripted(&view).is_empty());
}

#[test]
fn scout_uses_the_only_pass_through_a_mountain_wall() {
    let mut b =
        ViewBuilder::new(20, 12)
            .unexplored_from_col(12)
            .unit(1, ME, UnitKind::Scout, off(3, 2));
    for row in 0..12 {
        if row != 9 {
            b = b.terrain(off(11, row), Terrain::Mountains);
        }
    }
    let orders = scripted(&b.build());
    assert_eq!(move_target(&orders, 1), Some(off(11, 9)));
}

#[test]
fn settler_sticks_to_its_chosen_site() {
    let north = off(10, 3);
    let south = off(10, 13);
    let base = || {
        ViewBuilder::new(24, 18)
            .city(1, ME, off(6, 8), 3)
            .patch(north, 2, Terrain::Forest)
            .patch(south, 2, Terrain::Forest)
            .unit(1, ME, UnitKind::Settler, off(6, 8))
    };
    for site in [north, south] {
        let view = base().goto(1, site).build();
        assert_eq!(move_target(&scripted(&view), 1), Some(site));
    }
}

/// Timing check: `cargo test --release -p legatum-ai midgame_speed -- --ignored --nocapture`.
#[test]
#[ignore]
fn midgame_speed() {
    let view = midgame_view();
    let mut brain = ScriptedBrain::new(Faction::Byzantium, 1, ME);
    let runs = 200;
    let start = std::time::Instant::now();
    for _ in 0..runs {
        std::hint::black_box(brain.orders(std::hint::black_box(&view)));
    }
    let per_call = start.elapsed() / runs;
    println!("ScriptedBrain::orders on the mid-game view: {per_call:?} per call");
    if !cfg!(debug_assertions) {
        assert!(per_call < std::time::Duration::from_millis(5));
    }
}

/// An ungarrisoned pop-2 city (strength 14, 200 HP: weight 28).
fn open_city_view(spearmen: &[Hex]) -> PlayerView {
    let mut b = ViewBuilder::new(24, 16)
        .city(1, ME, off(3, 8), 3)
        .unit(90, ME, UnitKind::Spearman, off(3, 8))
        .city(9, ENEMY, off(15, 8), 2);
    for (i, h) in spearmen.iter().enumerate() {
        b = b.unit(1 + i as u32, ME, UnitKind::Spearman, *h);
    }
    b.build()
}

#[test]
fn lone_spearman_waits_but_a_pair_finishes_a_suppressed_city() {
    let city = off(15, 8);
    let alone = open_city_view(&[off(14, 8)]);
    let orders = run(&alone, Knobs::new(50, 50, 50, 50, 0));
    assert_ne!(move_target(&orders, 1), Some(city));

    let mut pair = open_city_view(&[off(14, 8), off(14, 7)]);
    // The pair commits when it can finish through city fire; full-HP cities
    // require the larger combined army covered by city_assault's staging test.
    pair.cities
        .iter_mut()
        .find(|c| c.owner == ENEMY)
        .unwrap()
        .hp = 80;
    let orders = run(&pair, Knobs::new(50, 50, 50, 50, 100));
    assert_eq!(move_target(&orders, 1), Some(city));
    assert_eq!(move_target(&orders, 2), Some(city));
}

#[test]
fn aggressive_army_gathers_on_distinct_tiles_near_a_winnable_city() {
    let target = off(14, 8);
    let base = |defenders: u32| {
        let mut b = ViewBuilder::new(30, 16)
            .city(1, ME, off(3, 8), 3)
            .unit(90, ME, UnitKind::Spearman, off(3, 8))
            .unit(1, ME, UnitKind::Spearman, off(4, 8))
            .unit(2, ME, UnitKind::Spearman, off(4, 9))
            .city(9, ENEMY, target, 6);
        for i in 0..defenders {
            b = b.unit(50 + i, ENEMY, UnitKind::Spearman, off(14 - i as i32, 8));
        }
        b.build()
    };
    // Garrison 20 + a quarter of the pop-6 city (44/4 = 11): 31 against our 40.
    let view = base(1);
    let bold = run(&view, Knobs::new(50, 50, 50, 50, 80));
    let a = move_target(&bold, 1).expect("unit 1 marches");
    let b = move_target(&bold, 2).expect("unit 2 marches");
    assert_eq!(a.distance(target), 3);
    assert_eq!(b.distance(target), 3);
    assert_ne!(a, b, "gathering tiles must differ");

    let timid = run(&view, Knobs::new(50, 50, 50, 50, 10));
    assert!(move_target(&timid, 1).is_none_or(|t| t.distance(target) > 5));

    // Three defenders: not winnable with two soldiers, so no campaign.
    let strong = base(3);
    let bold = run(&strong, Knobs::new(50, 50, 50, 50, 80));
    assert!(move_target(&bold, 1).is_none_or(|t| t.distance(target) > 5));
}

#[test]
fn two_units_never_step_into_the_same_tile() {
    // Two soldiers on non-adjacent sides of a gap tile G, with a one-tile-wide corridor out of
    // G; everything else is mountains, so both first steps are G although their destinations
    // differ.
    let g = off(6, 6);
    let (a, b) = (g.neighbor(0), g.neighbor(2));
    let (da, db) = (g.neighbor(4), g.neighbor(4).neighbor(4));
    let mut builder = ViewBuilder::new(12, 12).patch(g, 8, Terrain::Mountains);
    for h in [g, a, b, da, db] {
        builder = builder.terrain(h, Terrain::Plains);
    }
    let view = builder
        .unit(1, ME, UnitKind::Spearman, a)
        .unit(2, ME, UnitKind::Spearman, b)
        .build();
    let mut p =
        crate::scripted::Planner::new(&view, Knobs::new(50, 50, 50, 50, 50), Faction::India, 1);
    p.orders = vec![
        Order::MoveUnit {
            unit: UnitId(1),
            to: da,
        },
        Order::MoveUnit {
            unit: UnitId(2),
            to: db,
        },
    ];
    let orders = p.orders.clone();
    p.dedupe_first_steps();
    // The resolver gives the gap tile to unit 1 and unit 2 waits a round with its moves, so
    // neither order is cancelled; with the gene `movement_wait` at 0 unit 2 loses its turn.
    assert_eq!(p.orders, orders);
    p.orders = orders;
    p.decisions.movement_wait = 0;
    p.dedupe_first_steps();
    assert_eq!(
        p.orders,
        vec![
            Order::MoveUnit {
                unit: UnitId(1),
                to: da
            },
            Order::SkipUnit { unit: UnitId(2) },
        ]
    );
}

#[test]
fn two_units_with_different_first_steps_do_not_collide_on_the_second() {
    let middle = off(6, 6);
    let left = middle.neighbor(3).neighbor(3);
    let right = middle.neighbor(0).neighbor(0);
    let mut b = ViewBuilder::new(14, 14).patch(middle, 14, Terrain::Mountains);
    for h in [left, middle.neighbor(3), middle, middle.neighbor(0), right] {
        b = b.terrain(h, Terrain::Plains);
    }
    let view = b
        .unit(1, ME, UnitKind::Settler, left)
        .unit(2, ME, UnitKind::Settler, right)
        .build();
    let mut p = planner(&view);
    p.orders = vec![
        Order::MoveUnit {
            unit: UnitId(1),
            to: right,
        },
        Order::MoveUnit {
            unit: UnitId(2),
            to: left,
        },
    ];
    let orders = p.orders.clone();
    p.dedupe_first_steps();
    // Unit 1 enters the middle first and unit 2 waits (the resolver's turn-taking).
    assert_eq!(p.orders, orders);
    p.orders = orders;
    p.decisions.movement_wait = 0;
    p.dedupe_first_steps();
    assert_eq!(
        order_for(&p.orders, 2),
        Some(&Order::SkipUnit { unit: UnitId(2) })
    );
}

#[test]
fn movement_repaths_when_a_friend_arrives_before_the_next_round() {
    let view = ViewBuilder::new(14, 14)
        .unit(1, ME, UnitKind::Spearman, Hex::new(3, 5))
        .unit(2, ME, UnitKind::Spearman, Hex::new(4, 5))
        .build();
    let mut p = planner(&view);
    let orders = vec![
        Order::MoveUnit {
            unit: UnitId(1),
            to: Hex::new(6, 5),
        },
        Order::MoveUnit {
            unit: UnitId(2),
            to: Hex::new(5, 5),
        },
    ];
    p.orders = orders.clone();
    p.dedupe_first_steps();
    // The rear soldier follows on round one, then routes around the now parked leader.
    // Reserving both complete paths from the initial snapshot unnecessarily cancels a move.
    assert_eq!(p.orders, orders);
}

#[test]
fn yielding_rechecks_the_occupancy_of_the_yielding_units_start() {
    let gap = off(6, 6);
    let (a, b, exit) = (gap.neighbor(0), gap.neighbor(2), gap.neighbor(4));
    let mut builder = ViewBuilder::new(12, 12).patch(gap, 8, Terrain::Mountains);
    for h in [gap, a, b, exit] {
        builder = builder.terrain(h, Terrain::Plains);
    }
    let view = builder
        .unit(1, ME, UnitKind::Spearman, a)
        .unit(2, ME, UnitKind::Spearman, b)
        .build();
    let mut p = planner(&view);
    p.orders = vec![
        Order::MoveUnit {
            unit: UnitId(1),
            to: b,
        },
        Order::MoveUnit {
            unit: UnitId(2),
            to: exit,
        },
    ];
    let orders = p.orders.clone();
    p.dedupe_first_steps();
    // Both enter the gap, unit 1 first, and swap places on the next round: no cancellation.
    assert_eq!(p.orders, orders);
    p.orders = orders;
    p.decisions.movement_wait = 0;
    p.dedupe_first_steps();
    assert_eq!(
        p.orders,
        vec![
            Order::SkipUnit { unit: UnitId(1) },
            Order::SkipUnit { unit: UnitId(2) },
        ]
    );
}

#[test]
fn routine_arrival_yields_to_a_later_emergency_retreat_step() {
    let goal = off(6, 6);
    let retreat_from = goal.neighbor(3).neighbor(3);
    let view = ViewBuilder::new(14, 14)
        .unit(90, ME, UnitKind::Pikeman, retreat_from)
        .unit(1, ME, UnitKind::Spearman, goal.neighbor(1))
        .unit(100, ENEMY, UnitKind::Archer, retreat_from.neighbor(3))
        .build();
    let mut p = planner(&view);
    p.orders = vec![
        Order::MoveUnit {
            unit: UnitId(1),
            to: goal,
        },
        Order::MoveUnit {
            unit: UnitId(90),
            to: goal,
        },
    ];
    p.retreats.insert(UnitId(90));
    p.dedupe_first_steps();
    assert_eq!(
        order_for(&p.orders, 1),
        Some(&Order::SkipUnit { unit: UnitId(1) })
    );
    assert_eq!(move_target(&p.orders, 90), Some(goal));
}

#[test]
fn stuck_units_take_turns() {
    // Both had gotos and full moves: they bounced last turn. The higher id yields.
    let view = ViewBuilder::new(24, 12)
        .unit(1, ME, UnitKind::Spearman, off(10, 5))
        .unit(2, ME, UnitKind::Spearman, off(10, 7))
        .build();
    let mut view = view;
    for u in view.units.iter_mut() {
        u.goto = Some(off(20, 6));
    }
    let orders = scripted(&view);
    assert_eq!(
        order_for(&orders, 2),
        Some(&Order::SkipUnit { unit: UnitId(2) })
    );
}

#[test]
fn settler_discounts_foreign_neighbors_but_can_found_beside_them() {
    let view = ViewBuilder::new(16, 16)
        .owner(off(10, 8), ENEMY)
        .unit(1, ME, UnitKind::Settler, off(8, 8))
        .build();
    let orders = scripted(&view);
    let ctx = crate::analysis::Ctx::new(&view);
    let score = ctx.site_score[view.index(off(8, 8)).unwrap()].unwrap();
    assert!(score < ctx.site_score[view.index(off(4, 4)).unwrap()].unwrap());
    assert!(orders.contains(&Order::FoundCity { unit: UnitId(1) }));
    assert!(ctx.site_score[view.index(off(10, 8)).unwrap()].is_none());
}

#[test]
fn archer_never_shoots_a_civilian() {
    let view = ViewBuilder::new(20, 16)
        .city(1, ME, off(6, 6), 2)
        .unit(1, ME, UnitKind::Archer, off(6, 6))
        .unit(50, ENEMY, UnitKind::Settler, off(8, 6))
        .build();
    let orders = scripted(&view);
    assert!(
        !orders
            .iter()
            .any(|o| matches!(o, Order::RangedAttack { .. }))
    );
}

#[test]
fn review_archer_shoots_city_with_only_a_civilian_inside() {
    let target = off(8, 6);
    let view = ViewBuilder::new(20, 16)
        .city(1, ME, off(3, 6), 2)
        .unit(90, ME, UnitKind::Spearman, off(3, 6))
        .city(9, ENEMY, target, 2)
        .unit(1, ME, UnitKind::Archer, off(6, 6))
        .unit(50, ENEMY, UnitKind::Settler, target)
        .build();
    assert_eq!(
        order_for(&scripted(&view), 1),
        Some(&Order::RangedAttack {
            unit: UnitId(1),
            target
        })
    );
}

#[test]
fn review_campaign_gathers_on_reachable_land_not_the_nearest_island() {
    let target = off(14, 8);
    let start = off(4, 8);
    let island = target
        .ring(3)
        .into_iter()
        .min_by_key(|h| (h.distance(start), *h))
        .unwrap();
    let mut b = ViewBuilder::new(30, 16)
        .city(1, ME, off(3, 8), 3)
        .unit(90, ME, UnitKind::Spearman, off(3, 8))
        .unit(1, ME, UnitKind::Spearman, start)
        .unit(2, ME, UnitKind::Spearman, off(4, 9))
        .city(9, ENEMY, target, 2);
    for h in island.neighbors() {
        b = b.terrain(h, Terrain::Ocean);
    }
    let view = b.build();
    let orders = run(&view, Knobs::new(50, 50, 50, 50, 80));
    let ctx = crate::analysis::Ctx::new(&view);
    for id in [1, 2] {
        let to = move_target(&orders, id).expect("army marches");
        assert_eq!(to.distance(target), 3);
        let from = view.units.iter().find(|u| u.id == UnitId(id)).unwrap().pos;
        assert_ne!(
            ctx.distance_field(from, false)[view.index(to).unwrap()],
            crate::analysis::UNREACHABLE,
            "unit {id} gathered on an island"
        );
    }
}

#[test]
fn settler_that_stood_still_on_its_site_sometimes_waits() {
    // Full moves, no goto, turn > 1: its found order probably clashed with an unseen settler.
    // Different seeds must not all make the same choice, or two empires would mirror forever.
    let view = ViewBuilder::new(16, 16)
        .unit(1, ME, UnitKind::Settler, off(8, 8))
        .build();
    let mut founds = 0;
    for seed in 0..20 {
        let orders = ScriptedBrain::new(Faction::India, seed, ME).orders(&view);
        assert_valid(&view, &orders);
        match order_for(&orders, 1) {
            Some(Order::FoundCity { .. }) => founds += 1,
            Some(Order::SkipUnit { .. }) => {}
            other => panic!("unexpected {other:?}"),
        }
    }
    assert!(founds > 0 && founds < 20, "{founds}/20 founded");
}

fn planner(view: &PlayerView) -> crate::scripted::Planner<'_> {
    let mut p =
        crate::scripted::Planner::new(view, Knobs::new(50, 50, 50, 50, 60), Faction::India, 7);
    p.decisions.capitals = 0;
    p
}

#[test]
fn confirmed_route_stops_before_a_newly_seen_enemy() {
    let start = Hex::new(3, 3);
    let clear = Hex::new(4, 3);
    let enemy = Hex::new(5, 3);
    let goal = Hex::new(6, 3);
    let view = ViewBuilder::new(12, 10)
        .unit(1, ME, UnitKind::Spearman, start)
        .unit(2, ENEMY, UnitKind::Spearman, enemy)
        .build();
    let mut p = planner(&view);
    assert!(p.unit_order(
        UnitId(1),
        Order::MoveUnitPath {
            unit: UnitId(1),
            to: goal,
            path: vec![clear, enemy, goal],
            stop_on_block: true,
        }
    ));
    assert_eq!(
        p.orders,
        vec![Order::MoveUnitPath {
            unit: UnitId(1),
            to: clear,
            path: vec![clear],
            stop_on_block: true,
        }]
    );

    let mut p = planner(&view);
    assert!(p.unit_order(
        UnitId(1),
        Order::MoveUnitPath {
            unit: UnitId(1),
            to: goal,
            path: vec![enemy, goal],
            stop_on_block: true,
        }
    ));
    assert_eq!(p.orders, vec![Order::SkipUnit { unit: UnitId(1) }]);
}

#[test]
fn uncapped_ai_uses_the_expected_game_length_for_pressure_without_a_build_or_research_deadline() {
    let mut view = ViewBuilder::new(24, 16).city(1, ME, off(8, 8), 3).build();
    // This is about pressure, not about food buildings (now buildable from turn 1).
    view.cities[0].detail.as_mut().unwrap().options.retain(|o| {
        !matches!(
            o.item,
            ProductionItem::Building {
                building: BuildingKind::Granary
            }
        )
    });
    view.turn_limit = None;
    view.turn = legatum_core::data::EXPECTED_GAME_TURNS / 2;
    assert_eq!(planner(&view).conquest_pressure(), 0);
    view.turn = legatum_core::data::EXPECTED_GAME_TURNS;
    assert_eq!(planner(&view).conquest_pressure(), 100);
    view.turn *= 5;
    assert_eq!(planner(&view).turns_remaining(), i32::MAX);
    let orders = run(&view, Knobs::new(100, 0, 0, 0, 0));
    assert!(orders.iter().any(|o| matches!(
        o,
        Order::SetProduction {
            item: ProductionItem::Unit {
                unit: UnitKind::Settler
            },
            ..
        }
    )));
    assert!(crate::fixture::research_head(&orders).is_some());
    view.turn_limit = Some(view.turn + 4);
    assert_eq!(planner(&view).turns_remaining(), 5);
    assert!(
        !run(&view, Knobs::new(100, 0, 0, 0, 0))
            .iter()
            .any(|o| matches!(
                o,
                Order::SetProduction {
                    item: ProductionItem::Unit {
                        unit: UnitKind::Settler
                    },
                    ..
                }
            ))
    );
}

#[test]
fn late_campaign_prioritizes_reachable_vp_leader_and_keeps_garrison() {
    use legatum_core::view::TeamStanding;
    let home = off(3, 8);
    let weak = off(13, 8);
    let leader = off(17, 8);
    let mut view = ViewBuilder::new(28, 16)
        .city(1, ME, home, 3)
        .unit(90, ME, UnitKind::Spearman, home)
        .unit(1, ME, UnitKind::Spearman, off(4, 8))
        .unit(2, ME, UnitKind::Spearman, off(4, 9))
        .city(9, ENEMY, weak, 2)
        .city(10, PlayerId(3), leader, 2)
        .build();
    view.standings = vec![
        TeamStanding {
            team: TeamId(2),
            members: vec![ENEMY],
            vp: 2,
            target: 10,
            victory_hold: None,
        },
        TeamStanding {
            team: TeamId(3),
            members: vec![PlayerId(3)],
            vp: 4,
            target: 10,
            victory_hold: None,
        },
    ]
    .into();
    let target = |v: &PlayerView| {
        let p = planner(v);
        let army = p
            .ctx
            .own_units
            .iter()
            .copied()
            .filter(|u| u.id.0 < 90)
            .collect::<Vec<_>>();
        p.campaign_target(&army)
    };
    assert_eq!(target(&view), Some(weak));
    view.turn = 260;
    assert_eq!(target(&view), Some(leader));
    assert_eq!(
        order_for(&scripted(&view), 90),
        Some(&Order::SkipUnit { unit: UnitId(90) })
    );
    // Public race pressure works before the late turn threshold too.
    view.turn = 40;
    view.standings[1].vp = 9;
    assert_eq!(target(&view), Some(leader));
    // A leader on disconnected land is not a campaign, no matter its VP.
    for h in leader.neighbors() {
        let i = view.index(h).unwrap();
        view.tiles[i].terrain = Some(Terrain::Ocean);
    }
    assert_eq!(target(&view), Some(weak));
}

#[test]
fn late_pressure_does_not_attack_unseen_or_overwhelming_city_or_abandon_defense() {
    let target = off(14, 8);
    let mut view = open_city_view(&[off(4, 8), off(4, 9)]);
    view.turn = 140;
    view.cities[1].pos = target;
    view.cities[1].visible = false;
    let i = view.index(target).unwrap();
    view.tiles[i].visible = false;
    let p = planner(&view);
    assert_eq!(p.campaign_target(&p.ctx.own_units), Some(target));
    assert!(p.too_risky(p.ctx.own_units[0], target));
    let orders = scripted(&view);
    assert!(!orders.iter().any(|o| matches!(o, Order::MoveUnit { to, .. } | Order::RangedAttack { target: to, .. } if *to == target)));
    view.cities[1].visible = true;
    view.tiles[i].visible = true;
    view.cities[1].population = 30;
    view.cities[1].walls = true;
    let p = planner(&view);
    assert_eq!(p.campaign_target(&p.ctx.own_units), None);

    let mut under_attack = ViewBuilder::new(24, 16)
        .city(1, ME, off(3, 8), 3)
        .unit(90, ME, UnitKind::Spearman, off(3, 8))
        .unit(1, ME, UnitKind::Spearman, off(5, 8))
        .unit(2, ME, UnitKind::Spearman, off(5, 9))
        .unit(51, ENEMY, UnitKind::Pikeman, off(1, 8))
        .unit(52, ENEMY, UnitKind::Pikeman, off(1, 9))
        .unit(53, ENEMY, UnitKind::Pikeman, off(2, 6))
        .city(9, ENEMY, target, 1)
        .build();
    under_attack.turn = 140;
    let orders = scripted(&under_attack);
    for id in [1, 2, 90] {
        assert!(move_target(&orders, id).is_none_or(|h| h.distance(off(3, 8)) <= 2));
    }
}

#[test]
fn disconnected_troops_cannot_make_a_campaign_look_winnable() {
    let mut view = open_city_view(&[off(4, 8), off(4, 9)]);
    view.turn = 130;
    let island = view.units.iter().find(|u| u.id == UnitId(2)).unwrap().pos;
    for h in island.neighbors() {
        let i = view.index(h).unwrap();
        view.tiles[i].terrain = Some(Terrain::Ocean);
    }
    let p = planner(&view);
    let army = p
        .ctx
        .own_units
        .iter()
        .copied()
        .filter(|u| u.id.0 < 90)
        .collect::<Vec<_>>();
    assert_eq!(p.campaign_target(&army), None);
}

#[test]
fn research_responds_to_food_production_gold_and_army_bottlenecks() {
    let research = |v: &PlayerView| {
        let mut p = planner(v);
        p.plan_research();
        crate::fixture::research_head(&p.orders)
    };
    let mut view = ViewBuilder::new(20, 16).city(1, ME, off(5, 8), 3).build();
    view.cities[0].detail.as_mut().unwrap().yields.total.food = 1;
    assert_eq!(research(&view), Some(TechId::Irrigation));
    let d = view.cities[0].detail.as_mut().unwrap();
    d.yields.total.food = 20;
    d.yields.total.production = 1;
    assert_eq!(research(&view), Some(TechId::BronzeWorking));

    let mut view = ViewBuilder::new(20, 16)
        .known_techs(&[TechId::Irrigation, TechId::BronzeWorking, TechId::Masonry])
        .city(1, ME, off(5, 8), 3)
        .build();
    view.economy.gold_per_turn.total = -4;
    assert_eq!(research(&view), Some(TechId::Currency));
    view.economy.gold_per_turn.total = 10;
    view.economy.science_per_turn.total = 1;
    assert_eq!(research(&view), Some(TechId::Writing));

    let view = ViewBuilder::new(20, 16)
        .known_techs(&TechId::ALL[..5])
        .city(1, ME, off(5, 8), 3)
        .unit(1, ME, UnitKind::Spearman, off(5, 8))
        .unit(2, ME, UnitKind::Spearman, off(6, 8))
        .unit(50, ENEMY, UnitKind::Pikeman, off(7, 8))
        .build();
    // The Ancient era is done: an enemy at the gates puts the Classical first tier first: Iron
    // Working (it reveals the Iron every Classical unit needs; gene `reveal_tech_first`), then
    // Mathematics' siege.
    assert_eq!(research(&view), Some(TechId::IronWorking));
    let mut p = planner(&view);
    p.decisions.reveal_tech_first = 0;
    p.plan_research();
    assert_eq!(
        crate::fixture::research_head(&p.orders),
        Some(TechId::Mathematics)
    );
}

#[test]
fn expansion_ignores_unreachable_islands_and_unfinishable_settlers() {
    let center = off(8, 8);
    let mut b = ViewBuilder::new(24, 16).city(1, ME, center, 3);
    // A small starting island, with plentiful good land beyond its ocean moat.
    for h in center.ring(2) {
        b = b.terrain(h, Terrain::Ocean);
    }
    let view = b.build();
    assert_eq!(planner(&view).count_good_sites(4), 0);
    let settler = |o: &Order| {
        matches!(
            o,
            Order::SetProduction {
                item: ProductionItem::Unit {
                    unit: UnitKind::Settler
                },
                ..
            }
        )
    };
    assert!(!scripted(&view).iter().any(settler));
    let mut view = ViewBuilder::new(24, 16).city(1, ME, center, 3).build();
    assert!(planner(&view).count_good_sites(4) > 0);
    view.turn = 145;
    // Late in the game the empire has trained a few Settlers: the next one is dear (170).
    settlers_trained(&mut view, 2);
    assert!(!run(&view, Knobs::new(100, 0, 0, 0, 0)).iter().any(settler));
}

#[test]
fn melee_risk_for_adjacent_archer_is_not_its_safe_ranged_forecast() {
    let target = off(9, 8);
    let view = ViewBuilder::new(20, 16)
        .unit_hp(1, ME, UnitKind::Archer, off(8, 8), 25)
        .unit(50, ENEMY, UnitKind::Pikeman, target)
        .build();
    let p = planner(&view);
    let u = p.ctx.own_units[0];
    let ranged = forecast_attack_mode(&view, u.id, target, AttackMode::Ranged).unwrap();
    let melee = forecast_attack_mode(&view, u.id, target, AttackMode::Melee).unwrap();
    assert_eq!(ranged.loss_chance, 0);
    assert_eq!(melee.loss_chance, 100);
    assert!(p.too_risky(u, target));
    // A free shot does not make standing next to a lethal melee attacker safe.
    let orders = scripted(&view);
    let escape = move_target(&orders, 1).expect("wounded Archer can escape");
    assert!(escape.distance(target) > 2);
}

#[test]
fn one_unit_occupies_a_zero_hp_city_instead_of_bouncing_with_its_army() {
    let target = off(15, 8);
    let mut view = open_city_view(&[off(14, 8), off(14, 7)]);
    view.cities
        .iter_mut()
        .find(|c| c.owner == ENEMY)
        .unwrap()
        .hp = 0;
    let orders = scripted(&view);
    assert_eq!(
        orders
            .iter()
            .filter(|o| matches!(o, Order::MoveUnit { to, .. } if *to == target))
            .count(),
        1
    );
}

#[test]
fn an_archer_uses_melee_to_occupy_an_empty_zero_hp_city() {
    let target = off(15, 8);
    let mut view = ViewBuilder::new(24, 16)
        .city(1, ME, off(3, 8), 3)
        .unit(90, ME, UnitKind::Spearman, off(3, 8))
        .city(9, ENEMY, target, 2)
        .unit(1, ME, UnitKind::Archer, off(14, 8))
        .build();
    view.cities[1].hp = 0;
    let orders = scripted(&view);
    assert_eq!(move_target(&orders, 1), Some(target));
    let forecast = forecast_attack_mode(&view, UnitId(1), target, AttackMode::Melee).unwrap();
    assert!(!forecast.ranged);
    assert_eq!(forecast.loss_chance, 0);
    assert_valid(&view, &orders);
}

#[test]
fn ranged_target_uses_exact_kill_odds_instead_of_uncapped_base_damage() {
    // Both base-damage estimates suggest a kill. Two equally threatening Spearmen; only the
    // one in the open is a guaranteed kill across all rolls, so exact odds must decide.
    let view = ViewBuilder::new(20, 16)
        .terrain(off(8, 9), Terrain::Hills)
        .unit(1, ME, UnitKind::Archer, off(8, 8))
        .unit_hp(50, ENEMY, UnitKind::Spearman, off(9, 8), 22)
        .unit_hp(51, ENEMY, UnitKind::Spearman, off(8, 9), 22)
        .build();
    let a = forecast_attack_mode(&view, UnitId(1), off(9, 8), AttackMode::Ranged).unwrap();
    let b = forecast_attack_mode(&view, UnitId(1), off(8, 9), AttackMode::Ranged).unwrap();
    assert!(
        a.win_chance > b.win_chance,
        "{} {}",
        a.win_chance,
        b.win_chance
    );
    assert_eq!(
        order_for(&scripted(&view), 1),
        Some(&Order::RangedAttack {
            unit: UnitId(1),
            target: off(9, 8)
        })
    );
}

#[test]
fn ranged_target_weighs_kill_odds_by_the_threat_removed() {
    // Not kill-first: an 85% kill on a Pikeman, which would hit our Archer far harder, removes
    // more expected threat than a certain kill on a Spearman with the same HP.
    let view = ViewBuilder::new(20, 16)
        .unit(1, ME, UnitKind::Archer, off(8, 8))
        .unit_hp(50, ENEMY, UnitKind::Spearman, off(9, 8), 20)
        .unit_hp(51, ENEMY, UnitKind::Pikeman, off(8, 9), 18)
        .build();
    let a = forecast_attack_mode(&view, UnitId(1), off(9, 8), AttackMode::Ranged).unwrap();
    let b = forecast_attack_mode(&view, UnitId(1), off(8, 9), AttackMode::Ranged).unwrap();
    assert_eq!((a.win_chance, b.win_chance), (100, 85));
    assert_eq!(
        order_for(&scripted(&view), 1),
        Some(&Order::RangedAttack {
            unit: UnitId(1),
            target: off(8, 9)
        })
    );
}

#[test]
fn private_enemy_state_cannot_change_the_view_or_decision() {
    let state = new_game(&GameSetup::all_ai(711));
    let before = player_view(&state, ME);
    let mut changed = state.clone();
    for u in changed.units.values_mut().filter(|u| u.owner != ME) {
        u.goto = Some(off(20, 20));
        if !before.units.iter().any(|seen| seen.id == u.id) {
            u.fortify = 6;
        }
    }
    // Unobserved orders and research progress are not part of the public score or view.
    for p in changed.players.iter_mut().filter(|p| p.id != ME) {
        p.science_stored += 7;
    }
    let after = player_view(&changed, ME);
    assert_eq!(before, after);
    for difficulty in [AiDifficulty::Easy, AiDifficulty::Normal, AiDifficulty::Hard] {
        let mut brain =
            ScriptedBrain::with_difficulty(state.players[0].faction, 711, ME, difficulty);
        let orders = brain.orders(&before);
        assert_eq!(orders, brain.orders(&after));
        assert_eq!(
            orders,
            ScriptedBrain::with_difficulty(state.players[0].faction, 711, ME, difficulty)
                .orders(&before)
        );
    }
}

#[test]
fn bicycle_ranged_units_fire_in_campaigns_and_keep_melee_follow_through() {
    for kind in [UnitKind::Crossbowman, UnitKind::CompositeArcher] {
        let target = off(15, 8);
        let mut view = ViewBuilder::new(24, 16)
            .city(1, ME, off(3, 8), 4)
            .unit(90, ME, UnitKind::Pikeman, off(3, 8))
            .city(9, ENEMY, target, 4)
            .unit(1, ME, kind, off(13, 8))
            .unit(2, ME, UnitKind::Pikeman, off(14, 8))
            .build();
        let orders = scripted(&view);
        assert_eq!(
            order_for(&orders, 1),
            Some(&Order::RangedAttack {
                unit: UnitId(1),
                target
            })
        );
        assert_valid(&view, &orders);
        view.cities[1].hp = 0;
        assert_eq!(move_target(&scripted(&view), 2), Some(target));
    }
}

#[test]
fn settler_and_ui_rank_the_same_safe_site() {
    let (b, _) = two_sites();
    let view = b.unit(1, ME, UnitKind::Settler, off(6, 8)).build();
    let settler = view.units.iter().find(|u| u.id == UnitId(1)).unwrap();
    let sites = legatum_core::settlement::settlement_sites(&view, settler, &[]);
    let recommended = sites.iter().find(|s| s.rank == Some(1)).unwrap();
    assert_eq!(move_target(&scripted(&view), 1), Some(recommended.pos));
}

#[test]
fn buys_a_quoted_defender_without_replacing_a_market() {
    let market = ProductionItem::Building {
        building: BuildingKind::Market,
    };
    let spearman = ProductionItem::Unit {
        unit: UnitKind::Spearman,
    };
    let mut view = ViewBuilder::new(16, 16)
        .faction(Faction::India)
        .city(1, ME, off(5, 5), 3)
        .gold(1000)
        .producing(1, market, 1000)
        .unit(50, ENEMY, UnitKind::Pikeman, off(7, 5))
        .build();
    let d = view.cities[0].detail.as_mut().unwrap();
    d.options
        .iter_mut()
        .find(|o| o.item == spearman)
        .unwrap()
        .buy_cost = Some(80);
    let orders = ScriptedBrain::new(Faction::India, 7, ME).orders(&view);
    assert!(orders.contains(&Order::BuyItem {
        city: CityId(1),
        item: spearman
    }));
    assert!(!orders.iter().any(|o| matches!(
        o,
        Order::SetProduction {
            city: CityId(1),
            ..
        }
    )));
    view.cities[0]
        .detail
        .as_mut()
        .unwrap()
        .options
        .iter_mut()
        .find(|o| o.item == spearman)
        .unwrap()
        .buy_block = Some(legatum_core::economy::BuyBlock::NoSpawnTile);
    assert!(
        !ScriptedBrain::new(Faction::India, 7, ME)
            .orders(&view)
            .iter()
            .any(|o| matches!(o, Order::BuyItem { .. }))
    );
}

#[test]
fn a_threatened_city_buys_a_ranged_garrison_that_the_city_shields() {
    // Civ 6: the city absorbs the Pikeman's attack, so a bought Archer is safe while it
    // shoots; it is preferred over a cheaper Spearman that holds equally well.
    let market = ProductionItem::Building {
        building: BuildingKind::Market,
    };
    let mut view = ViewBuilder::new(16, 16)
        .faction(Faction::India)
        .city(1, ME, off(5, 5), 3)
        .gold(1000)
        .producing(1, market, 1000)
        .unit(50, ENEMY, UnitKind::Pikeman, off(7, 5))
        .build();
    let d = view.cities[0].detail.as_mut().unwrap();
    for (unit, cost) in [(UnitKind::Spearman, 80), (UnitKind::Archer, 100)] {
        d.options
            .iter_mut()
            .find(|o| o.item == ProductionItem::Unit { unit })
            .unwrap()
            .buy_cost = Some(cost);
    }
    let orders = ScriptedBrain::new(Faction::India, 7, ME).orders(&view);
    assert!(
        orders.contains(&Order::BuyItem {
            city: CityId(1),
            item: ProductionItem::Unit {
                unit: UnitKind::Archer
            }
        }),
        "{orders:?}"
    );
}

mod colony_compound_tests {
    use super::*;

    /// A one-Spearman capital at peace, a known rival town five hexes off and good land in
    /// reach: the old army gate wants a second troop first; the priced colony outbids it.
    #[test]
    fn a_priced_colony_outbids_a_second_troop_in_a_quiet_opening() {
        let mut view = ViewBuilder::new(24, 16)
            .city(1, ME, off(5, 8), 2)
            .city(2, ENEMY, off(10, 8), 1)
            .unit(1, ME, UnitKind::Spearman, off(5, 8))
            .build();
        view.turn = 8;
        settlers_trained(&mut view, 1);
        let pick = |gene: i32| {
            let mut brain =
                ScriptedBrain::with_difficulty(Faction::Switzerland, 7, ME, AiDifficulty::Hard);
            brain.decisions.colony_compound = gene;
            let orders = brain.orders(&view);
            assert_valid(&view, &orders);
            production_for(&orders, 1)
        };
        let settler = Some(ProductionItem::Unit {
            unit: UnitKind::Settler,
        });
        assert_ne!(pick(0), settler);
        assert_eq!(pick(100), settler);
    }
}
