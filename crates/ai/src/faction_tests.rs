//! Decisions that make the eight factions' rules useful, through the public brain.
use crate::{Brain, Knobs, ScriptedBrain, fixture::*};
use legatum_core::*;
fn orders(v: &PlayerView) -> Vec<Order> {
    let out = ScriptedBrain::new(v.faction_of(ME), 7, ME).orders(v);
    assert_valid(v, &out);
    out
}
fn pending(v: &mut PlayerView) {
    v.players[0].faction_goal_done = false;
}

#[test]
fn all_eight_personalities_are_distinct_and_difficulty_preserves_determinism() {
    let mut seen = std::collections::BTreeSet::new();
    for f in Faction::ALL {
        let k = Knobs::for_faction(f);
        assert!(seen.insert((k.expansion, k.military, k.science, k.economy, k.aggression)));
        let v = ViewBuilder::new(16, 12)
            .faction(f)
            .city(1, ME, off(4, 4), 3)
            .unit(1, ME, UnitKind::Spearman, off(4, 4))
            .build();
        for d in [AiDifficulty::Easy, AiDifficulty::Normal, AiDifficulty::Hard] {
            let a = ScriptedBrain::with_difficulty(f, 3, ME, d).orders(&v);
            assert_eq!(a, ScriptedBrain::with_difficulty(f, 3, ME, d).orders(&v));
            assert_valid(&v, &a);
        }
    }
}

#[test]
fn hard_military_personality_keeps_funding_its_field_force() {
    let mut v = ViewBuilder::new(24, 16)
        .faction(Faction::Sparta)
        .known_techs(&[TechId::Currency])
        .gold(0)
        .city(1, ME, off(5, 8), 3)
        .unit(1, ME, UnitKind::Spearman, off(5, 8))
        .build();
    v.turn = 60;
    let soldier = ProductionItem::Unit {
        unit: UnitKind::Spearman,
    };
    let market = ProductionItem::Building {
        building: BuildingKind::Market,
    };
    let options = &mut v.cities[0].detail.as_mut().unwrap().options;
    options.retain(|o| o.item == soldier || o.item == market);
    for option in options {
        option.turns = if option.item == soldier { 70 } else { 1 };
    }
    let mut hard = ScriptedBrain::with_difficulty(Faction::Sparta, 7, ME, AiDifficulty::Hard);
    let orders = hard.orders(&v);
    assert_valid(&v, &orders);
    assert!(orders.contains(&Order::SetProduction {
        city: CityId(1),
        item: soldier
    }));
    let quiet =
        crate::difficulty::scale_knobs(Knobs::for_faction(Faction::Sparta), AiDifficulty::Easy);
    let orders = ScriptedBrain::with_knobs(Faction::Sparta, 7, ME, quiet).orders(&v);
    assert!(
        orders.contains(&Order::SetProduction {
            city: CityId(1),
            item: soldier
        }),
        "even the cautious personality must rebuild a depleted Sparta army"
    );
}

#[test]
fn industry_payback_does_not_delay_stalled_growth() {
    let mut v = ViewBuilder::new(20, 16)
        .faction(Faction::Switzerland)
        .known_techs(&TechId::ALL)
        .city(1, ME, off(4, 4), 4)
        .build();
    v.turn = 200;
    v.turn_limit = None;
    let d = &mut **v.cities[0].detail.as_mut().unwrap();
    d.buildings = vec![BuildingKind::Workshop, BuildingKind::Granary];
    d.yields.total.food = 8;
    d.yields.total.production = 16;
    d.options.retain(|o| {
        matches!(
            o.item,
            ProductionItem::Building {
                building: BuildingKind::Forge | BuildingKind::Mill
            }
        )
    });
    for opt in &mut d.options {
        // Twice the fixture's quote: the payback must stay well past fifty turns.
        opt.cost *= 2;
        opt.turns = (opt.cost + 15) / 16;
        opt.yields = d.yields.clone();
    }
    // Both are useful, but the Machine Shop takes over fifty Standard turns to
    // repay its price. Food immediately resumes growth in the existing city.
    assert!(orders(&v).contains(&Order::SetProduction {
        city: CityId(1),
        item: ProductionItem::Building {
            building: BuildingKind::Mill
        },
    }));
}

#[test]
fn switzerland_invest_until_the_treasury_goal_wins_then_save_except_to_prevent_a_fall() {
    let mut v = ViewBuilder::new(12, 12)
        .faction(Faction::Switzerland)
        .gold(798)
        .city(1, ME, off(4, 4), 3)
        .producing(
            1,
            ProductionItem::Unit {
                unit: UnitKind::Spearman,
            },
            300,
        )
        .build();
    pending(&mut v);
    v.economy.gold_per_turn.total = 0;
    assert!(
        orders(&v)
            .iter()
            .any(|o| matches!(o, Order::BuyItem { .. }))
    );
    v.economy.gold_per_turn.total = 10;
    v.standings.push(legatum_core::view::TeamStanding {
        team: v.team,
        members: vec![ME],
        vp: 8,
        target: 10,
        victory_hold: None,
    });
    assert!(
        !orders(&v)
            .iter()
            .any(|o| matches!(o, Order::BuyItem { .. }))
    );
    let enemy = ViewBuilder::new(12, 12)
        .unit(9, ENEMY, UnitKind::Scout, off(5, 4))
        .build()
        .units
        .pop()
        .unwrap();
    v.units.push(enemy);
    assert!(
        !orders(&v)
            .iter()
            .any(|o| matches!(o, Order::BuyItem { .. }))
    );
    v.economy.gold = 550;
    assert!(
        !orders(&v)
            .iter()
            .any(|o| matches!(o, Order::BuyItem { .. }))
    );
    v.cities[0].hp = 1;
    assert!(orders(&v).iter().any(|o| matches!(
        o,
        Order::BuyItem {
            city: CityId(1),
            ..
        }
    )));
}

#[test]
fn india_pursues_six_cities_and_avoids_far_settlement() {
    assert!(Knobs::for_faction(Faction::India).target_cities() >= 6);
    let v = ViewBuilder::new(30, 16)
        .faction(Faction::India)
        .city(1, ME, off(4, 8), 4)
        .unit(1, ME, UnitKind::Settler, off(4, 8))
        .build();
    let out = orders(&v);
    let h = out
        .iter()
        .find_map(|o| match o {
            Order::MoveUnit {
                unit: UnitId(1),
                to,
            } => Some(*to),
            _ => None,
        })
        .unwrap();
    assert!(h.distance(off(4, 8)) <= 8);
}

#[test]
fn india_settlement_search_includes_sites_three_hexes_from_a_city() {
    let v = ViewBuilder::new(24, 16)
        .faction(Faction::India)
        .city(1, ME, off(4, 8), 4)
        .unit(1, ME, UnitKind::Settler, off(6, 8))
        .build();
    let candidates = legatum_core::settlement::settlement_candidates(&v, &v.units[0], &[]);
    assert!(
        candidates
            .iter()
            .any(|site| site.pos.distance(off(4, 8)) == 3),
        "India's planner should consider the 3-hex ring legal"
    );
}

#[test]
fn byzantium_scout_explores_when_no_camp_is_near() {
    let mut v = ViewBuilder::new(24, 16)
        .faction(Faction::Byzantium)
        .known_techs(&[
            TechId::Irrigation,
            TechId::BronzeWorking,
            TechId::Masonry,
            TechId::Currency,
            TechId::Writing,
            TechId::Feudalism,
            TechId::CodeOfLaws,
        ])
        .unexplored_from_col(14)
        .city(1, ME, off(4, 4), 3)
        .unit(1, ME, UnitKind::Scout, off(10, 4))
        .build();
    pending(&mut v);
    // The old goal's requirement no longer pulls the research queue, and the scout keeps
    // exploring.
    assert_ne!(
        crate::fixture::research_head(&orders(&v)),
        Some(TechId::PrintingPress)
    );
    assert!(orders(&v).iter().any(|o| matches!(
        o,
        Order::MoveUnit {
            unit: UnitId(1),
            ..
        }
    )));
}

#[test]
fn celts_settler_chooses_a_legal_site_without_hill_requirement() {
    let mut v = ViewBuilder::new(30, 18)
        .faction(Faction::Celts)
        .city(1, ME, off(4, 8), 4)
        .patch(off(10, 8), 1, Terrain::Hills)
        .unit(1, ME, UnitKind::Settler, off(6, 8))
        .build();
    pending(&mut v);
    let out = orders(&v);
    let h = out
        .iter()
        .find_map(|o| match o {
            Order::MoveUnit {
                unit: UnitId(1),
                to,
            } => Some(*to),
            _ => None,
        })
        .unwrap();
    assert!(legatum_core::settlement::known_legal_site(&v, h));
}

#[test]
fn athens_research_the_receiving_network_branch() {
    let mut v = ViewBuilder::new(12, 12)
        .faction(Faction::Athens)
        .known_techs(&[
            TechId::Irrigation,
            TechId::BronzeWorking,
            TechId::Masonry,
            TechId::Currency,
            TechId::Writing,
        ])
        .city(1, ME, off(4, 4), 4)
        .build();
    pending(&mut v);
    let p =
        crate::scripted::Planner::new(&v, Knobs::for_faction(Faction::Athens), Faction::Athens, 7);
    // Its network branch (Library, Great Library) outranks the army technologies.
    assert!(p.tech_value(TechId::Philosophy) > p.tech_value(TechId::Mathematics));
    assert!(p.tech_value(TechId::CodeOfLaws) > p.tech_value(TechId::Mathematics));
}

#[test]
fn athens_capture_prize_uses_the_free_pick_research_value() {
    let mut v = ViewBuilder::new(16, 12)
        .faction(Faction::Sparta)
        .city(1, ME, off(4, 4), 3)
        .city(2, PlayerId(2), off(7, 4), 3)
        .build();
    v.players[2].faction = Faction::Athens;
    v.cities[1].greece_captive_tech = true;
    let mut p =
        crate::scripted::Planner::new(&v, Knobs::for_faction(Faction::Sparta), Faction::Sparta, 7);
    let best = crate::goal_actions::pick_free_techs(&v, 1, |t| p.tech_value(t))[0];
    assert_eq!(
        p.greece_captive_tech_value(&v.cities[1]),
        p.tech_value(best)
    );
    p.decisions.greece_captive_attack = 0;
    assert_eq!(p.greece_captive_tech_value(&v.cities[1]), 0);
}

#[test]
fn every_faction_builds_or_trains_its_unique_piece_when_useful() {
    let cases = [
        (
            Faction::Switzerland,
            ProductionItem::Building {
                building: BuildingKind::Walls,
            },
        ),
        (
            Faction::India,
            ProductionItem::Building {
                building: BuildingKind::Market,
            },
        ),
        (
            Faction::Byzantium,
            ProductionItem::Unit {
                unit: UnitKind::Pikeman,
            },
        ),
        (
            Faction::Germany,
            ProductionItem::Building {
                building: BuildingKind::Workshop,
            },
        ),
        (
            Faction::Celts,
            ProductionItem::Unit {
                unit: UnitKind::Archer,
            },
        ),
        (
            Faction::Mongols,
            ProductionItem::Unit {
                unit: UnitKind::Scout,
            },
        ),
        (
            Faction::Sparta,
            ProductionItem::Unit {
                unit: UnitKind::Spearman,
            },
        ),
        (
            Faction::Athens,
            ProductionItem::Building {
                building: BuildingKind::Library,
            },
        ),
    ];
    for (f, item) in cases {
        // A developed island with no room to expand. Buildings compete with each other;
        // military replacements are chosen in response to an approaching army.
        let mut b = ViewBuilder::new(16, 12)
            .faction(f)
            .known_techs(&if f == Faction::Sparta {
                Vec::new()
            } else {
                // The Ancient era and Feudalism (the Pikeman).
                let mut known = TechId::ALL[..5].to_vec();
                known.push(TechId::Feudalism);
                known
            });
        for i in 0..b.view.tiles.len() {
            b.view.tiles[i].terrain = Some(Terrain::Ocean);
        }
        b = b
            .patch(off(4, 4), 2, Terrain::Plains)
            .city(1, ME, off(4, 4), 4);
        if matches!(
            item,
            ProductionItem::Unit {
                unit: UnitKind::Pikeman | UnitKind::Archer | UnitKind::Spearman
            }
        ) {
            b = b.unit(9, ENEMY, UnitKind::Spearman, off(6, 4));
            if f == Faction::Celts {
                // V9 Spearmen have 18 melee: cover the garrison before asking for ranged support.
                b = b.unit(5, ME, UnitKind::Pikeman, off(4, 4));
            }
            if f == Faction::Byzantium {
                b = b.unit(5, ME, UnitKind::Spearman, off(4, 4)).unit(
                    6,
                    ME,
                    UnitKind::Archer,
                    off(4, 5),
                );
            }
        } else if f == Faction::Mongols {
            b = b.unexplored_from_col(6);
            for i in 0..5 {
                b = b.unit(i + 10, ME, UnitKind::Spearman, off(4, 4));
            }
        } else {
            for i in 0..8 {
                b = b.unit(
                    i + 10,
                    ME,
                    UnitKind::Spearman,
                    off(4 + (i as i32) % 2, 4 + (i as i32) / 2),
                );
            }
            if f == Faction::Switzerland {
                b = b.city(2, ENEMY, off(11, 4), 3);
            }
        }
        let mut v = b.build();
        pending(&mut v);
        v.turn = 70;
        // This check isolates each faction piece after that empire used its wonder slot.
        v.great_works.push(legatum_core::projects::GreatWorkView {
            player: ME,
            completed: Some(BuildingKind::Pyramids),
            building: None,
            city: None,
            progress: 0,
            cost: 0,
            production_per_turn: 0,
            turns: None,
            blocker: None,
        });
        if let ProductionItem::Building { building } = item {
            // Keep the unique piece and one ordinary alternative unbuilt.
            let d = &mut **v.cities[0].detail.as_mut().unwrap();
            // The Monument that used to serve as the unbuilt ordinary alternative is gone, and
            // every other ordinary building outvalues Walls for some faction: leave only the
            // unique piece unbuilt. Feudalism now also unlocks the Mill; this test is about
            // each faction's piece, so remove that competing ordinary building option.
            d.buildings = crate::fixture::ORDINARY_BUILDINGS
                .into_iter()
                .filter(|b| *b != building)
                .collect();
            d.options.retain(|o| !matches!(o.item, ProductionItem::Building { building: b }
                if d.buildings.contains(&b) || b == BuildingKind::Mill || (f == Faction::India && b == BuildingKind::ImperialDiwan)));
        }
        if matches!(item, ProductionItem::Unit { .. }) {
            let d = &mut **v.cities[0].detail.as_mut().unwrap();
            d.buildings = crate::fixture::ORDINARY_BUILDINGS.to_vec();
            d.options.retain(|o| {
                !matches!(
                    o.item,
                    ProductionItem::Building {
                        building: BuildingKind::Mill
                    }
                )
            });
            // A developed city builds an army piece in under ten turns: fourteen Production a
            // turn against Game pace (the fixture's own rate is lower; the era lineage made
            // the Pikeman a little dearer).
            for o in &mut d.options {
                o.turns = (o.cost + 13) / 14;
            }
            v.cities[0].walls = true;
        }
        let out = orders(&v);
        assert!(
            out.contains(&Order::SetProduction {
                city: CityId(1),
                item
            }),
            "{f:?}: {out:?}"
        );
    }
}

#[test]
fn india_does_not_queue_an_army_that_its_remitted_gold_cannot_fund() {
    let mut v = ViewBuilder::new(16, 12)
        .faction(Faction::India)
        .city_with_detail(1, ME, off(4, 4), 4)
        .unit(1, ME, UnitKind::Spearman, off(4, 4))
        .unit(2, ME, UnitKind::Spearman, off(5, 4))
        .build();
    v.economy.gold = 0;
    v.economy.gold_per_turn.total = -1;
    v.players[0].faction_goal_done = false;
    let p =
        crate::scripted::Planner::new(&v, Knobs::for_faction(Faction::India), Faction::India, 7);
    assert!(p.troop_unfunded(0, p.ctx.own_cities[0], UnitKind::Spearman));
    let planned = orders(&v);
    assert!(
        planned.iter().any(|order| matches!(
            order,
            Order::SetProduction {
                city: CityId(1),
                ..
            }
        )),
        "{planned:?}"
    );
    assert!(!planned.iter().any(|order| matches!(order,
        Order::SetProduction { item: ProductionItem::Unit { unit }, .. }
        | Order::BuyItem { item: ProductionItem::Unit { unit }, .. }
        if legatum_core::data::pays_upkeep(*unit)
    )));
    v.economy.gold_per_turn.total = 3;
    let funded =
        crate::scripted::Planner::new(&v, Knobs::for_faction(Faction::India), Faction::India, 7);
    assert!(!funded.troop_unfunded(0, funded.ctx.own_cities[0], UnitKind::Spearman));
    assert!(funded.goal_techs().contains(&TechId::Currency));
}
