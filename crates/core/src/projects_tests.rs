use crate::{
    data, economy, projects, research,
    rules::{
        battle, cities,
        test_support::{city, game, hex},
    },
    scoring, *,
};
const P: PlayerId = PlayerId(0);
fn item(building: BuildingKind) -> ProductionItem {
    ProductionItem::Building { building }
}
fn wonder(s: &GameState, city: CityId, building: BuildingKind) -> ProductionItem {
    crate::landmarks::candidates(s, &s.cities[&city], building)[0]
}
fn setup(building: BuildingKind) -> (GameState, CityId, CityId) {
    let mut s = game(&[0, 1, 0]);
    for p in &mut s.players {
        p.techs.extend(TechId::ALL);
    }
    let a = city(&mut s, 0, hex(2, 2), 8);
    let b = city(&mut s, 0, hex(7, 2), 8);
    for id in [a, b] {
        let c = s.cities.get_mut(&id).unwrap();
        if let Some((required, _)) = data::great_work_requirement(building) {
            c.buildings.insert(required);
        }
        c.production_stored = 0;
    }
    (s, a, b)
}
#[test]
fn seven_world_wonders_charge_the_full_cost_and_transfer_the_held_award() {
    for building in data::GREAT_WORKS {
        for faction in Faction::ALL {
            let (mut s, a, b) = setup(building);
            s.player_mut(P).faction = faction;
            let cost = economy::production_cost(&s, P, item(building));
            let quote = research::tech_unlocks(&s, P)
                .into_iter()
                .find(|u| u.item == item(building))
                .unwrap();
            assert_eq!(quote.cost, cost);
            assert_eq!(
                quote.cost_breakdown.parts.iter().map(|p| p.1).sum::<i32>(),
                cost
            );
            assert_eq!(quote.cost_breakdown.total, cost);
            assert!(quote.cities.iter().all(|c| c.blocker.is_none()));
            assert_eq!(quote.buy_cost, None);
            let work_a = wonder(&s, a, building);
            let work_b = wonder(&s, b, building);
            let orders = [
                Order::SetProduction {
                    city: a,
                    item: work_a,
                },
                Order::SetProduction {
                    city: b,
                    item: work_b,
                },
                Order::BuyItem {
                    city: a,
                    item: work_a,
                },
            ];
            assert_eq!(
                cities::validate_city_orders(&s, P, &orders)
                    .iter()
                    .map(|e| e.index)
                    .collect::<Vec<_>>(),
                [1, 2]
            );
            let mut events = Vec::new();
            cities::apply_city_orders(&mut s, P, &orders[..1], &mut events);
            assert!(matches!(events[0],Event::MegaprojectStarted { building:b,.. } if b==building));
            cities::produce(&mut s, a, cost - 1, &mut events);
            assert_eq!(s.player(P).great_work_completed, None);
            cities::produce(&mut s, a, 1, &mut events);
            scoring::update(&mut s, &mut events);
            assert_eq!(s.player(P).great_work_completed, Some(building));
            assert_eq!(s.player(P).great_work_holder, Some(P));
            assert_eq!(
                events
                    .iter()
                    .filter(|e| matches!(e, Event::GreatWorkCompleted { of, player, .. } if *of == P && *player == P))
                    .count(),
                1
            );
            for other in data::GREAT_WORKS {
                assert!(!economy::can_produce(
                    &s,
                    &s.cities[&b],
                    wonder(&s, b, other)
                ));
            }
            battle::capture_city(&mut s, a, PlayerId(1), &mut events);
            scoring::update(&mut s, &mut events);
            assert_eq!(s.player(P).great_work_completed, Some(building));
            assert_eq!(s.player(P).great_work_holder, Some(PlayerId(1)));
            assert_eq!(s.player(PlayerId(1)).great_work_completed, None);
            assert!(scoring::vp(&s, PlayerId(1)).contains(&(VpSource::GreatWork { of: P }, 2)));
        }
    }
}

#[test]
fn a_world_wonders_yield_replaces_its_tile_yield() {
    let (mut s, a, _) = setup(BuildingKind::HagiaSophia);
    let work = wonder(&s, a, BuildingKind::HagiaSophia);
    let ProductionItem::Wonder { tile, .. } = work else {
        unreachable!()
    };
    let before = economy::city_yield_totals(&s, &s.cities[&a], None, None);
    let lost = economy::owner_tile_yields(&s, P, s.map.get(tile).unwrap(), None);
    s.cities.get_mut(&a).unwrap().producing = Some(work);
    let cost = economy::production_cost(&s, P, work);
    cities::produce(&mut s, a, cost, &mut Vec::new());
    let after = economy::city_yield_totals(&s, &s.cities[&a], None, None);
    assert_eq!(
        after,
        before - lost + data::building_stats(BuildingKind::HagiaSophia).yields
    );
}

#[test]
fn first_completion_cancels_rival_orders_and_refunds_half_of_each_city_bank() {
    let (mut s, winner_city, _) = setup(BuildingKind::Pyramids);
    let rival_city = city(&mut s, 1, hex(2, 7), 8);
    let winner_work = wonder(&s, winner_city, BuildingKind::Pyramids);
    let rival_work = wonder(&s, rival_city, BuildingKind::Pyramids);
    let before_gold = s.player(PlayerId(1)).gold;
    s.cities.get_mut(&winner_city).unwrap().producing = Some(winner_work);
    {
        let c = s.cities.get_mut(&rival_city).unwrap();
        c.producing = Some(rival_work);
        c.production_stored = 140;
        c.bank(rival_work, 20);
        c.production_queue.push(rival_work);
    }
    let mut events = Vec::new();
    cities::produce(&mut s, winner_city, 360, &mut events);
    assert_eq!(
        s.player(P).great_work_completed,
        Some(BuildingKind::Pyramids)
    );
    assert_eq!(s.cities[&rival_city].producing, None);
    assert!(s.cities[&rival_city].production_queue.is_empty());
    assert_eq!(s.cities[&rival_city].banked(rival_work), 0);
    assert_eq!(s.player(PlayerId(1)).gold, before_gold + 80);
    assert!(events.iter().any(|e| matches!(e, Event::WonderRaceLost {
        city, owner, winner, building: BuildingKind::Pyramids,
        production: 160, gold: 80, active: true,
    } if *city == rival_city && *owner == PlayerId(1) && *winner == P)));
    assert!(
        economy::building_blocker(&s, &s.cities[&rival_city], BuildingKind::Pyramids, true)
            .unwrap()
            .contains("only one")
    );
    // Even destroying the tile later does not reopen the global construction slot.
    let site = match winner_work {
        ProductionItem::Wonder { tile, .. } => tile,
        _ => unreachable!(),
    };
    s.map.get_mut(site).unwrap().landmark = None;
    assert!(
        economy::building_blocker(&s, &s.cities[&rival_city], BuildingKind::Pyramids, true)
            .is_some()
    );
}
/// Thun, 8.9.26 T92: Thun switched to the Hagia Sophia with its Mill queued, and India finished
/// it that turn. Whichever city resolves first, the loser stops and asks its owner (the reason is
/// in its city view), its queue waits one phase with no false blocker, and then resumes.
#[test]
fn a_lost_wonder_race_stops_the_city_for_a_choice_in_either_resolution_order() {
    for loser_first in [true, false] {
        let (mut s, winner_city, _) = setup(BuildingKind::Pyramids);
        let rival_city = city(&mut s, 1, hex(2, 7), 8);
        let winner_work = wonder(&s, winner_city, BuildingKind::Pyramids);
        let rival_work = wonder(&s, rival_city, BuildingKind::Pyramids);
        // His Mill (200), so the resumed build does not complete within the test.
        let next = item(BuildingKind::Mill);
        s.cities.get_mut(&winner_city).unwrap().producing = Some(winner_work);
        {
            let c = s.cities.get_mut(&rival_city).unwrap();
            c.buildings.insert(BuildingKind::Granary);
            c.producing = Some(rival_work);
            c.production_stored = 10;
            c.bank(next, 79);
            c.production_queue.push(next);
        }
        assert_eq!(
            economy::production_blocker(&s, &s.cities[&rival_city], next, true),
            None
        );
        let mut events = Vec::new();
        if loser_first {
            cities::produce(&mut s, rival_city, 30, &mut events);
        }
        cities::produce(&mut s, winner_city, 360, &mut events);
        if !loser_first {
            cities::produce(&mut s, rival_city, 30, &mut events);
        }
        let c = &s.cities[&rival_city];
        let (production, held) = if loser_first { (40, 0) } else { (10, 30) };
        assert_eq!(c.producing, None, "loser_first {loser_first}");
        assert_eq!(c.production_queue, vec![next]);
        assert_eq!(c.banked(next), 79);
        assert_eq!(c.production_stored, held);
        assert_eq!(
            c.wonder_lost,
            Some(crate::state::WonderLoss {
                turn: s.turn,
                winner: P,
                building: BuildingKind::Pyramids,
                production,
                gold: production / 2,
            })
        );
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, Event::OrderFailed { .. }))
        );
        let view = player_view(&s, PlayerId(1));
        let detail = view.cities.iter().find(|v| v.id == rival_city).unwrap();
        let detail = detail.detail.as_ref().unwrap();
        assert_eq!(detail.wonder_lost, c.wonder_lost);
        assert_eq!(detail.producing, None);
        // No choice by the next production phase: the queue resumes with its own bank.
        s.turn += 1;
        cities::produce(&mut s, rival_city, 5, &mut events);
        let c = &s.cities[&rival_city];
        assert_eq!(c.wonder_lost, None);
        assert_eq!(c.producing, Some(next), "loser_first {loser_first}");
        assert_eq!(c.production_stored, 79 + held + 5);
    }
}

#[test]
fn a_production_order_clears_a_lost_wonder_race() {
    let (mut s, winner_city, _) = setup(BuildingKind::Pyramids);
    let rival_city = city(&mut s, 1, hex(2, 7), 8);
    let winner_work = wonder(&s, winner_city, BuildingKind::Pyramids);
    let rival_work = wonder(&s, rival_city, BuildingKind::Pyramids);
    s.cities.get_mut(&winner_city).unwrap().producing = Some(winner_work);
    s.cities.get_mut(&rival_city).unwrap().producing = Some(rival_work);
    cities::produce(&mut s, winner_city, 360, &mut Vec::new());
    assert!(s.cities[&rival_city].wonder_lost.is_some());
    let next = item(BuildingKind::Granary);
    let mut orders = std::collections::BTreeMap::new();
    orders.insert(
        PlayerId(1),
        vec![Order::SetProduction {
            city: rival_city,
            item: next,
        }],
    );
    let (s, _) = resolve_turn(&s, &orders);
    assert_eq!(s.cities[&rival_city].wonder_lost, None);
    assert_eq!(s.cities[&rival_city].producing, Some(next));
}

#[test]
fn public_progress_does_not_reveal_hidden_city_details_and_another_empire_can_build_same_route() {
    let (mut s, a, _) = setup(BuildingKind::GrandBazaar);
    let work = wonder(&s, a, BuildingKind::GrandBazaar);
    s.cities.get_mut(&a).unwrap().producing = Some(work);
    s.cities.get_mut(&a).unwrap().production_stored = 123;
    let rival = player_view(&s, PlayerId(1));
    assert!(rival.cities.is_empty());
    assert_eq!(rival.great_works[0].progress, 123);
    assert_eq!(
        rival.great_works[0].building,
        Some(BuildingKind::GrandBazaar)
    );
    assert_eq!(rival.great_works[0].turns, None);
    assert_eq!(rival.great_works[0].blocker, None);
    for id in [city(&mut s, 2, hex(2, 7), 8), city(&mut s, 2, hex(7, 7), 8)] {
        s.cities
            .get_mut(&id)
            .unwrap()
            .buildings
            .insert(BuildingKind::Bank);
        assert!(
            economy::can_produce(
                &s,
                &s.cities[&id],
                wonder(&s, id, BuildingKind::GrandBazaar)
            ) == (s
                .cities_of(PlayerId(2))
                .filter(|c| c.buildings.contains(&BuildingKind::Bank))
                .count()
                == 2)
        );
    }
    let c = s.cities_of(PlayerId(2)).next().unwrap().id;
    assert!(
        economy::can_produce(&s, &s.cities[&c], wonder(&s, c, BuildingKind::GrandBazaar)),
        "same-team empires have separate slots"
    );
}
#[test]
fn loss_of_support_cancels_before_more_production_and_banks_earned_progress() {
    let (mut s, a, b) = setup(BuildingKind::GrandBazaar);
    let work_a = wonder(&s, a, BuildingKind::GrandBazaar);
    let work_b = wonder(&s, b, BuildingKind::GrandBazaar);
    s.cities.get_mut(&a).unwrap().producing = Some(work_a);
    s.cities.get_mut(&a).unwrap().production_stored = 100;
    s.cities.get_mut(&b).unwrap().owner = PlayerId(1);
    assert!(
        economy::building_blocker(&s, &s.cities[&a], BuildingKind::GrandBazaar, false)
            .unwrap()
            .contains("2 owned")
    );
    let mut events = Vec::new();
    cities::produce(&mut s, a, 10, &mut events);
    // The Great Work keeps its 100 banked; this turn's 10 is unassigned idle Production.
    assert_eq!(s.cities[&a].banked(work_a), 100);
    assert_eq!(s.cities[&a].production_stored, 10);
    assert_eq!(s.cities[&a].producing, None);
    assert!(matches!(events[0], Event::OrderFailed { .. }));
    s.cities.get_mut(&b).unwrap().owner = P;
    cities::apply_city_orders(
        &mut s,
        P,
        &[
            Order::SetProduction {
                city: a,
                item: work_a,
            },
            Order::SetProduction {
                city: a,
                item: ProductionItem::Unit {
                    unit: UnitKind::Spearman,
                },
            },
            Order::SetProduction {
                city: b,
                item: work_b,
            },
        ],
        &mut events,
    );
    // Resuming the Great Work took its bank and the idle 10; switching to a Spearman
    // banked all 110 with the Great Work again.
    assert_eq!(s.cities[&a].production_stored, 0);
    assert_eq!(
        s.cities[&a].banked(work_a),
        110,
        "earned progress stays in original city"
    );
    assert_eq!(s.cities[&b].production_stored, 0);
    battle::capture_city(&mut s, b, PlayerId(1), &mut events);
    assert_eq!(s.cities[&b].producing, None);
    assert_eq!(s.cities[&b].production_stored, 0);
}
/// City b's Food surplus at population 7 (Mill, grassland around it).
const B_SURPLUS: i32 = 7;

fn tall_growth_setup() -> (GameState, CityId, CityId) {
    let mut s = game(&[0, 1]);
    s.setup.turn_limit = None;
    // Border expansion cannot add food, keeping the forecast's yield assumptions true.
    for tile in &mut s.map.tiles {
        tile.terrain = Terrain::Mountains;
    }
    let a = city(&mut s, 0, hex(2, 2), 8);
    let b = city(&mut s, 0, hex(7, 2), 7);
    city(&mut s, 1, hex(2, 7), 1);
    for (id, terrain) in [(a, Terrain::Plains), (b, Terrain::Grassland)] {
        for h in s.cities[&id].pos.within(1) {
            s.map.get_mut(h).unwrap().terrain = terrain;
        }
    }
    let c = s.cities.get_mut(&a).unwrap();
    c.buildings.insert(BuildingKind::Mill);
    c.resistance = 1;
    c.food_stored = 0;
    let c = s.cities.get_mut(&b).unwrap();
    c.buildings.insert(BuildingKind::Mill);
    c.food_stored = data::growth_threshold(7) - 4 * B_SURPLUS;
    (s, a, b)
}

#[test]
fn tall_forecast_does_not_count_a_city_that_starves_after_resistance() {
    let (mut s, a, b) = tall_growth_setup();

    let hash = s.hash();
    let forecast = projects::development_forecast(&s, P);
    assert_eq!(forecast.growth[0].turns_to_eight, Some(0));
    assert_eq!(forecast.growth[1].turns_to_eight, Some(4));
    assert_eq!(forecast.tall_turns, None);
    assert_eq!(s.hash(), hash, "forecasting must not mutate the game");

    for turn in 1..=4 {
        let (next, _events) = rules::resolve_turn(&s, &Default::default());
        s = next;
        assert_eq!(s.cities[&a].population, if turn == 1 { 8 } else { 7 });
        assert_eq!(s.cities[&b].population, if turn == 4 { 8 } else { 7 });
        assert_eq!(economy::city_yields(&s, &s.cities[&a]).total.food, 14);
    }
}

#[test]
fn tall_forecast_requires_overlap_after_all_cities_grow_or_starve() {
    for (food, resistance, expected) in [(4, 1, None), (6, 1, Some(4)), (0, 4, Some(4))] {
        let (mut s, a, b) = tall_growth_setup();
        let c = s.cities.get_mut(&a).unwrap();
        c.food_stored = food;
        c.resistance = resistance;
        assert_eq!(projects::development_forecast(&s, P).tall_turns, expected);
        for turn in 1..=4 {
            let (next, _) = rules::resolve_turn(&s, &Default::default());
            s = next;
            let reached = expected == Some(turn);
            assert_eq!(
                s.cities[&a].population >= 8 && s.cities[&b].population >= 8,
                reached
            );
        }
    }
}

#[test]
fn tall_forecast_finds_a_later_overlap_after_starvation_and_regrowth() {
    let (mut s, a, _) = tall_growth_setup();
    let pos = s.cities[&a].pos;
    // 15 Food: starve at population 8, then recover at +1 Food at population 7.
    s.map.get_mut(pos).unwrap().terrain = Terrain::Grassland;
    let expected = 2 + data::growth_threshold(7);
    assert_eq!(
        projects::development_forecast(&s, P).tall_turns,
        Some(expected)
    );
    for _ in 1..=expected {
        let (next, _) = rules::resolve_turn(&s, &Default::default());
        s = next;
    }
}

#[test]
fn tall_forecast_includes_the_horizon_and_an_existing_pair_but_not_other_owners() {
    let (mut s, a, b) = tall_growth_setup();
    for h in s.cities[&a].pos.within(1) {
        s.map.get_mut(h).unwrap().terrain = Terrain::Grassland;
    }
    let c = s.cities.get_mut(&a).unwrap();
    c.buildings = [BuildingKind::Mill].into();
    c.resistance = 0;
    let horizon = crate::data::EXPECTED_GAME_TURNS as i32;
    let c = s.cities.get_mut(&b).unwrap();
    c.food_stored = data::growth_threshold(7) - B_SURPLUS;
    c.resistance = horizon - 1;
    assert_eq!(
        projects::development_forecast(&s, P).tall_turns,
        Some(horizon)
    );
    s.cities.get_mut(&b).unwrap().resistance = horizon;
    assert_eq!(projects::development_forecast(&s, P).tall_turns, None);
    s.cities.get_mut(&b).unwrap().population = 8;
    assert_eq!(projects::development_forecast(&s, P).tall_turns, Some(0));
    s.cities.get_mut(&b).unwrap().owner = PlayerId(1);
    assert_eq!(projects::development_forecast(&s, P).tall_turns, None);
}

/// Plains around the city, with only the first `grassland` tiles of its ring (and itself) green:
/// at population 6 a Mill can lift it to population 8 and nothing less can.
fn green_ring(s: &mut GameState, id: CityId, grassland: usize) {
    for (i, h) in s.cities[&id].pos.within(1).into_iter().enumerate() {
        if let Some(t) = s.map.get_mut(h) {
            t.terrain = if i < grassland {
                Terrain::Grassland
            } else {
                Terrain::Plains
            };
        }
    }
}

#[test]
fn growth_forecast_reassigns_population_and_respects_resistance_and_food_limits() {
    let mut s = game(&[0, 1]);
    let id = city(&mut s, 0, hex(3, 3), 6);
    green_ring(&mut s, id, 3);
    assert_eq!(projects::turns_to_population(&s, &s.cities[&id], 8), None);
    s.cities
        .get_mut(&id)
        .unwrap()
        .buildings
        .insert(BuildingKind::Mill);
    s.cities.get_mut(&id).unwrap().resistance = 3;
    let eta = projects::turns_to_population(&s, &s.cities[&id], 8).unwrap();
    s.cities.get_mut(&id).unwrap().resistance = 0;
    for turn in 1..=eta - 3 {
        let food = economy::city_yields(&s, &s.cities[&id]).total.food;
        cities::grow(&mut s, id, food, &mut Vec::new());
        if turn < eta - 3 {
            assert!(s.cities[&id].population < 8);
        }
    }
    assert_eq!(s.cities[&id].population, 8);
}

#[test]
fn food_quotes_for_the_mill_match_actual_growth_after_it_is_built() {
    let mut s = game(&[0, 1]);
    let id = city(&mut s, 0, hex(3, 3), 4);
    for tile in &mut s.map.tiles {
        tile.terrain = Terrain::Plains;
    }
    green_ring(&mut s, id, 3);
    s.cities.get_mut(&id).unwrap().food_stored = 17;
    s.cities
        .get_mut(&id)
        .unwrap()
        .buildings
        .insert(BuildingKind::Granary);
    let before = s.hash();
    let growth = projects::development_forecast(&s, P);
    let unlocks = research::tech_unlocks(&s, P);
    let mill = unlocks
        .iter()
        .find(|u| u.item == item(BuildingKind::Mill))
        .unwrap();
    let quote = &mill.cities[0];
    assert_eq!(quote.blocker, None, "the Granary is in place");
    assert!(quote.build_turns.is_some());
    assert_eq!(
        s.hash(),
        before,
        "forecasting never installs or grows anything"
    );
    // Without the Mill this city never reaches eight; the quote says when it would with one,
    // and the Mill has no population gate, so the gate quote is the same number.
    assert_eq!(growth.growth[0].turns_to_eight, None);
    assert_eq!(mill.min_population, 0);
    let six = quote.turns_to_six_after.unwrap();
    let eight = quote.turns_to_eight_after.unwrap();
    assert!(six < eight);
    assert_eq!(quote.turns_to_eight_at_gate, Some(eight));

    // Check the quotes against actual growth with the Mill installed, keeping territory
    // and builds fixed as promised.
    economy::install_building(s.cities.get_mut(&id).unwrap(), BuildingKind::Mill);
    for turn in 1..=eight {
        let food = economy::city_yields(&s, &s.cities[&id]).total.food;
        cities::grow(&mut s, id, food, &mut Vec::new());
        let pop = s.cities[&id].population;
        assert_eq!(pop >= 6, turn >= six, "turn {turn}");
        assert_eq!(pop >= 8, turn == eight, "turn {turn}");
    }
}

#[test]
fn foundry_is_a_single_industrial_host_and_exchange_keeps_its_network_requirement() {
    for building in [BuildingKind::RoyalArsenal, BuildingKind::GrandBazaar] {
        let (mut s, a, b) = setup(building);
        let work = wonder(&s, a, building);
        s.cities.get_mut(&b).unwrap().owner = PlayerId(1);
        assert_eq!(
            economy::can_produce(&s, &s.cities[&a], work),
            building == BuildingKind::RoyalArsenal
        );
        if building == BuildingKind::RoyalArsenal {
            s.cities.get_mut(&a).unwrap().producing = Some(work);
            s.cities.get_mut(&a).unwrap().production_stored = 77;
            s.cities.get_mut(&a).unwrap().buildings.clear();
            cities::produce(&mut s, a, 9, &mut Vec::new());
            assert_eq!(s.cities[&a].producing, None);
            assert_eq!(s.cities[&a].banked(work), 77);
        }
    }
}

#[test]
fn switzerland_cannot_buy_a_great_work() {
    let building = BuildingKind::GrandBazaar;
    let (mut state, city, _) = setup(building);
    state.player_mut(P).faction = Faction::Switzerland;
    state.player_mut(P).gold = 2_000;
    let item = wonder(&state, city, building);
    assert_eq!(
        economy::buy_cost_for(&state, &state.cities[&city], item),
        None
    );
    assert!(!cities::validate_city_orders(&state, P, &[Order::BuyItem { city, item }]).is_empty());
    assert_eq!(state.player(P).gold, 2_000);
}
