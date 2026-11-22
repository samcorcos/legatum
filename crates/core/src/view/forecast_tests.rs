use super::tests::{P0, P1, P2, add_city, at, test_state};
use super::*;
use crate::events::Event;
use crate::game::spawn_unit;
use crate::rules::cities::process_cities;

fn detail(state: &GameState, id: CityId) -> CityDetail {
    player_view(state, P0)
        .cities
        .into_iter()
        .find(|c| c.id == id)
        .unwrap()
        .detail
        .map(|d| (*d).clone())
        .unwrap()
}

fn queue(state: &mut GameState, id: CityId, unit: UnitKind) {
    let c = state.cities.get_mut(&id).unwrap();
    c.producing = Some(ProductionItem::Unit { unit });
    c.production_stored = 1_000;
    state.player_mut(P0).techs.extend(TechId::ALL);
}

#[test]
fn concurrent_settler_forecasts_use_costs_in_city_resolution_order() {
    let mut s = test_state(24, 16);
    spawn_unit(&mut s, P2, UnitKind::Settler, at(20, 12));
    let first = add_city(&mut s, P0, at(5, 5), 2);
    let second = add_city(&mut s, P0, at(12, 5), 2);
    s.turn = 100;
    for id in [first, second] {
        let c = s.cities.get_mut(&id).unwrap();
        c.producing = Some(ProductionItem::Unit {
            unit: UnitKind::Settler,
        });
        c.production_stored = 97;
        assert_eq!(
            detail(&s, id).yields.total.production,
            if id == first { 5 } else { 3 }
        );
    }
    assert_eq!(s.player(P0).settlers_trained, 0);
    let hash = s.hash();
    let first_forecast = detail(&s, first);
    let second_forecast = detail(&s, second);
    assert_eq!(s.hash(), hash, "forecasting must not mutate the game");
    // The first Settler costs FIRST_SETTLER_PERCENT (100%) of 70.
    assert_eq!(first_forecast.production_cost, Some(70));
    assert_eq!(first_forecast.turns_to_complete, Some(1));
    assert_eq!(second_forecast.production_cost, Some(120));
    assert_eq!(second_forecast.turns_to_complete, Some(8));
    let second_completion_turn = second_forecast.turns_to_complete.unwrap();

    let (next, events) = crate::rules::resolve_turn(&s, &BTreeMap::new());
    assert_eq!(next.player(P0).settlers_trained, 1);
    for (id, forecast) in [(first, first_forecast), (second, second_forecast)] {
        let completed = events.iter().any(|e| {
            matches!(e,
            Event::ProductionCompleted { city, .. } if *city == id)
        });
        assert_eq!(completed, forecast.turns_to_complete == Some(1));
        assert_eq!(next.cities[&id].producing.is_none(), completed);
        assert_eq!(
            next.cities[&id].production_stored,
            97 + forecast.yields.total.production
                - if completed {
                    forecast.production_cost.unwrap()
                } else {
                    0
                }
        );
    }
    assert_eq!(detail(&next, second).production_cost, Some(120));
    let mut next = next;
    for turn in 2..=second_completion_turn {
        let (resolved, events) = crate::rules::resolve_turn(&next, &BTreeMap::new());
        assert_eq!(
            events.iter().any(|e| matches!(e,
            Event::ProductionCompleted { city, .. } if *city == second)),
            turn == second_completion_turn,
            "turn {turn}"
        );
        next = resolved;
    }
    assert_eq!(next.player(P0).settlers_trained, 2);
}

#[test]
fn growth_forecast_uses_food_retained_after_settler_completion() {
    let mut s = test_state(16, 12);
    spawn_unit(&mut s, P2, UnitKind::Settler, at(14, 10));
    let pos = at(5, 5);
    let id = add_city(&mut s, P0, pos, 3);
    // Grassland center plus desert ring one: four food. The Settler's pop 3 to 2 loses nothing
    // here (ring one isn't grown), and the territory phase claims one grassland tile: six food.
    for h in pos.ring(1) {
        s.map.get_mut(h).unwrap().terrain = Terrain::Desert;
    }
    let c = s.cities.get_mut(&id).unwrap();
    c.food_stored = 60;
    c.producing = Some(ProductionItem::Unit {
        unit: UnitKind::Settler,
    });
    c.production_stored = 147;
    let (s, events) = crate::rules::resolve_turn(&s, &BTreeMap::new());
    assert!(events.iter().any(|e| matches!(e,
        Event::ProductionCompleted { city, .. } if *city == id)));
    assert_eq!(s.cities[&id].population, 2);
    assert_eq!(s.cities[&id].food_stored, 58);
    let forecast = detail(&s, id);
    assert_eq!(forecast.growth_threshold, 46);
    assert_eq!(forecast.yields.total.food, 6);
    assert_eq!(forecast.turns_to_grow, Some(1));

    let (next, events) = crate::rules::resolve_turn(&s, &BTreeMap::new());
    assert_eq!(next.cities[&id].population, 3);
    assert_eq!(next.cities[&id].food_stored, 14);
    assert!(events.iter().any(|e| matches!(e,
        Event::CityGrew { city, population: 3, .. } if *city == id)));
}

#[test]
fn stored_food_growth_forecast_respects_consumption_and_resistance() {
    for (food, stored, resistance, grows) in [
        (4, 45, 0, false),
        (4, 46, 0, true),
        (2, 46, 0, false),
        (2, 48, 0, true),
        (4, 58, 1, false),
    ] {
        let mut s = test_state(16, 12);
        let pos = at(5, 5);
        let id = add_city(&mut s, P0, pos, 2);
        for h in pos.within(1) {
            s.map.get_mut(h).unwrap().terrain = Terrain::Desert;
        }
        if food == 4 {
            s.map.get_mut(pos).unwrap().terrain = Terrain::Grassland;
        }
        let c = s.cities.get_mut(&id).unwrap();
        c.food_stored = stored;
        c.resistance = resistance;
        assert_eq!(
            detail(&s, id).turns_to_grow,
            grows.then_some(1),
            "food={food}, stored={stored}, resistance={resistance}"
        );
        let (next, _) = crate::rules::resolve_turn(&s, &BTreeMap::new());
        assert_eq!(next.cities[&id].population, if grows { 3 } else { 2 });
    }
}

#[test]
fn bankruptcy_forecast_matches_disband_and_report_without_mutating_state() {
    let mut s = test_state(16, 12);
    for kind in [UnitKind::Spearman, UnitKind::Pikeman, UnitKind::Pikeman] {
        spawn_unit(&mut s, P0, kind, at(2, 2));
    }
    let victim = UnitId(s.next_unit_id);
    spawn_unit(&mut s, P0, UnitKind::Settler, at(2, 2));
    spawn_unit(&mut s, P1, UnitKind::Pikeman, at(3, 2));
    spawn_unit(&mut s, P2, UnitKind::Pikeman, at(4, 2));
    s.player_mut(P0).gold = 1;
    let hash = s.hash();
    let view = player_view(&s, P0);
    assert_eq!(hash, s.hash());
    let forecast = view.economy.bankruptcy.clone().unwrap();
    assert_eq!(forecast.gold.total, -2);
    assert_eq!(
        forecast.gold.parts,
        vec![("Treasury".into(), 1), ("Unit upkeep".into(), -3)]
    );
    let disband = forecast.disband.unwrap();
    assert_eq!(
        disband.unit,
        Some(victim),
        "highest id breaks equal-cost ties"
    );
    assert_eq!(disband.kind, UnitKind::Pikeman);
    assert_eq!(disband.city, None);

    let mut events = Vec::new();
    process_cities(&mut s, &mut events);
    assert_eq!(s.player(P0).gold, 0);
    assert!(!s.units.contains_key(&victim));
    assert!(events.iter().any(
        |e| matches!(e, Event::UnitDisbanded { unit, owner, reason, .. }
        if *unit == victim && *owner == P0 && reason.contains("-2 Gold"))
    ));
}

#[test]
fn bankruptcy_requires_negative_treasury_not_just_negative_income() {
    let mut s = test_state(16, 12);
    for _ in 0..3 {
        spawn_unit(&mut s, P0, UnitKind::Spearman, at(2, 2));
    }
    for gold in [3, 4, 100] {
        s.player_mut(P0).gold = gold;
        assert!(player_view(&s, P0).economy.bankruptcy.is_none());
    }
    s.units.clear();
    s.player_mut(P0).gold = -1;
    let forecast = player_view(&s, P0).economy.bankruptcy.clone().unwrap();
    assert_eq!(forecast.gold.total, -1);
    assert!(forecast.disband.is_none());
    s.player_mut(P0).alive = false;
    assert!(player_view(&s, P0).economy.bankruptcy.is_none());
}

#[test]
fn bankruptcy_includes_completing_units_and_identifies_their_city() {
    let mut s = test_state(16, 12);
    let id = add_city(&mut s, P0, at(5, 5), 2);
    for _ in 0..4 {
        spawn_unit(&mut s, P0, UnitKind::Spearman, at(2, 2));
    }
    queue(&mut s, id, UnitKind::Pikeman);
    s.player_mut(P0).gold = -economy::player_income(&s, P0).gold.total;
    assert!(s.player(P0).gold >= 0);
    let hash = s.hash();
    let view = player_view(&s, P0);
    assert_eq!(
        s.hash(),
        hash,
        "projection cannot allocate live ids or spend progress"
    );
    let forecast = view.economy.bankruptcy.clone().unwrap();
    // Five military units, two free: three Gold (no technology halves it any more).
    assert_eq!(forecast.gold.total, -3);
    assert!(
        forecast
            .gold
            .parts
            .contains(&("Upkeep of completing units".into(), -3))
    );
    let disband = forecast.disband.unwrap();
    assert_eq!(disband.unit, None, "never promise a future global id");
    assert_eq!(disband.city, Some(id));
    assert_eq!(disband.kind, UnitKind::Pikeman);
    let produced_id = UnitId(s.next_unit_id + 1);
    let mut events = Vec::new();
    process_cities(&mut s, &mut events);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::UnitDisbanded { unit, .. } if *unit == produced_id))
    );
}

#[test]
fn idle_waste_uses_the_idle_cap_and_preserves_existing_overflow() {
    let mut s = test_state(16, 12);
    let pos = at(5, 5);
    let id = add_city(&mut s, P0, pos, 2);
    for h in pos.ring(1).into_iter().take(2) {
        s.map.get_mut(h).unwrap().terrain = Terrain::Forest;
    }
    let cap = crate::rules::cities::IDLE_PRODUCTION_CAP;
    for (stored, wasted) in [(cap - 7, 0), (cap - 2, 5), (cap, 7), (cap + 20, 7)] {
        s.cities.get_mut(&id).unwrap().production_stored = stored;
        let d = detail(&s, id);
        assert_eq!(d.yields.total.production, 7);
        assert_eq!(d.idle_production_cap, cap);
        assert_eq!(d.idle_production_waste, wasted);
        let mut next = s.clone();
        process_cities(&mut next, &mut Vec::new());
        assert_eq!(next.cities[&id].production_stored, stored + 7 - wasted);
    }
    s.cities.get_mut(&id).unwrap().resistance = 1;
    assert_eq!(detail(&s, id).idle_production_waste, 0);
    s.cities.get_mut(&id).unwrap().resistance = 0;
    queue(&mut s, id, UnitKind::Spearman);
    assert_eq!(detail(&s, id).idle_production_waste, 0);
}

#[test]
fn starvation_forecast_matches_the_turn_a_population_is_lost() {
    for stored in [0, 3, 7] {
        let mut s = test_state(16, 12);
        spawn_unit(&mut s, P2, UnitKind::Settler, at(14, 10));
        let id = add_city(&mut s, P0, at(5, 5), 3);
        for t in &mut s.map.tiles {
            t.terrain = Terrain::Desert;
        }
        s.cities.get_mut(&id).unwrap().food_stored = stored;
        let d = detail(&s, id);
        let surplus = d.yields.total.food - FOOD_PER_POP * 3;
        assert!(surplus < 0, "the fixture must starve (surplus {surplus})");
        let forecast = d
            .turns_to_starve
            .expect("a starving city forecasts its loss");
        let mut state = s.clone();
        let mut turns = 0;
        while state.cities[&id].population == 3 {
            state = crate::rules::resolve_turn(&state, &BTreeMap::new()).0;
            turns += 1;
            assert!(turns <= 50, "never starved");
        }
        assert_eq!(turns, forecast, "stored={stored}, surplus={surplus}");
    }
    let mut s = test_state(16, 12);
    let id = add_city(&mut s, P0, at(5, 5), 1);
    for t in &mut s.map.tiles {
        t.terrain = Terrain::Desert;
    }
    assert_eq!(
        detail(&s, id).turns_to_starve,
        None,
        "population 1 cannot shrink"
    );
    let c = s.cities.get_mut(&id).unwrap();
    c.population = 3;
    c.resistance = 2;
    assert_eq!(
        detail(&s, id).turns_to_starve,
        None,
        "resistance pauses growth"
    );
}

#[test]
fn heal_forecast_counts_turns_at_twenty_a_turn_and_flags_an_adjacent_enemy() {
    let mut s = test_state(16, 12);
    let id = add_city(&mut s, P0, at(5, 5), 2);
    let full = detail(&s, id);
    assert_eq!((full.turns_to_heal, full.heal_blocked), (None, false));
    for (hp, turns) in [(0, 10), (1, 10), (160, 2), (180, 1), (199, 1)] {
        s.cities.get_mut(&id).unwrap().hp = hp;
        assert_eq!(detail(&s, id).turns_to_heal, Some(turns), "{hp} HP");
    }
    s.cities.get_mut(&id).unwrap().hp = 100;
    assert!(!detail(&s, id).heal_blocked);
    spawn_unit(&mut s, P1, UnitKind::Spearman, at(6, 5));
    assert!(!detail(&s, id).heal_blocked, "a teammate is no threat");
    spawn_unit(&mut s, P2, UnitKind::Spearman, at(4, 5));
    assert!(detail(&s, id).heal_blocked);
}

#[test]
fn a_city_hit_on_the_turn_that_just_resolved_says_so() {
    let mut s = test_state(16, 12);
    let id = add_city(&mut s, P0, at(5, 5), 2);
    s.turn = 7;
    assert!(!detail(&s, id).attacked_last_turn, "never attacked");
    s.cities.get_mut(&id).unwrap().last_attacked_turn = Some(6);
    assert!(
        detail(&s, id).attacked_last_turn,
        "hit on turn 6, now turn 7"
    );
    s.cities.get_mut(&id).unwrap().last_attacked_turn = Some(5);
    assert!(
        !detail(&s, id).attacked_last_turn,
        "an older hit is not last turn's"
    );
    s.cities.get_mut(&id).unwrap().last_attacked_turn = Some(7);
    assert!(
        !detail(&s, id).attacked_last_turn,
        "this turn has not resolved yet"
    );
}

#[test]
fn population_unlocks_name_the_settler_and_gated_upgrades() {
    let mut s = test_state(16, 12);
    let id = add_city(&mut s, P0, at(5, 5), 1);
    let settler = ProductionItem::Unit {
        unit: UnitKind::Settler,
    };
    assert_eq!(
        detail(&s, id).population_unlocks,
        vec![PopulationUnlock {
            population: data::SETTLER_MIN_POP,
            item: settler
        }]
    );
    s.player_mut(P0).techs.extend(TechId::ALL);
    s.cities
        .get_mut(&id)
        .unwrap()
        .buildings
        .insert(BuildingKind::Library);
    let unlocks = detail(&s, id).population_unlocks;
    assert!(unlocks.contains(&PopulationUnlock {
        population: 5,
        item: ProductionItem::Building {
            building: BuildingKind::University
        }
    }));
    assert!(
        unlocks
            .windows(2)
            .all(|w| w[0].population <= w[1].population)
    );
    assert!(unlocks.iter().all(|u| u.population > 1));
}

#[test]
fn settler_stall_accounts_for_growth_and_starvation_before_completion() {
    let mut s = test_state(16, 12);
    let id = add_city(&mut s, P0, at(5, 5), 1);
    // A population-1 city can't build a Settler at all: no stall forecast, no ETA; the city
    // banks it at resolution and asks for a new choice.
    queue(&mut s, id, UnitKind::Settler);
    assert_eq!(detail(&s, id).production_stall, None);
    assert_eq!(detail(&s, id).turns_to_complete, None);
    let mut next = s.clone();
    process_cities(&mut next, &mut Vec::new());
    assert!(next.cities[&id].producing.is_none());

    // A population-2 city that starves before completing stalls.
    for t in &mut s.map.tiles {
        t.terrain = Terrain::Desert;
    }
    let c = s.cities.get_mut(&id).unwrap();
    c.population = 2;
    c.food_stored = 0;
    assert_eq!(
        detail(&s, id).production_stall,
        Some(ProductionStall::Population {
            required: 2,
            available: 1
        })
    );
    s.cities.get_mut(&id).unwrap().production_stored = 0;
    assert_eq!(
        detail(&s, id).production_stall,
        None,
        "still accumulating production"
    );
}

#[test]
fn spawn_stall_matches_tile_rules_and_does_not_charge_future_upkeep() {
    let mut s = test_state(16, 12);
    let pos = at(5, 5);
    let id = add_city(&mut s, P0, pos, 2);
    queue(&mut s, id, UnitKind::Pikeman);
    for h in pos.within(1) {
        spawn_unit(&mut s, P0, UnitKind::Spearman, h);
    }
    s.player_mut(P0).gold = -economy::player_income(&s, P0).gold.total;
    let d = detail(&s, id);
    assert_eq!(d.production_stall, Some(ProductionStall::SpawnBlocked));
    assert_eq!(d.turns_to_complete, None);
    assert!(player_view(&s, P0).economy.bankruptcy.is_none());
    let mut next = s.clone();
    process_cities(&mut next, &mut Vec::new());
    assert!(next.cities[&id].producing.is_some());

    let center = s.units.values().find(|u| u.pos == pos).unwrap().id;
    s.units.get_mut(&center).unwrap().kind = UnitKind::Settler;
    assert!(
        detail(&s, id).production_stall.is_none(),
        "military can share with a friendly civilian"
    );
    s.units.get_mut(&center).unwrap().owner = P2;
    assert_eq!(
        detail(&s, id).production_stall,
        Some(ProductionStall::SpawnBlocked),
        "enemy civilians block spawning"
    );
    s.units.remove(&center);
    s.map.get_mut(pos).unwrap().terrain = Terrain::Mountains;
    assert_eq!(
        detail(&s, id).production_stall,
        Some(ProductionStall::SpawnBlocked)
    );
    s.cities.get_mut(&id).unwrap().resistance = 1;
    assert_eq!(
        detail(&s, id).production_stall,
        None,
        "resistance is already reported separately"
    );
}

#[test]
fn forecasts_serialize_and_do_not_expose_other_empires_private_queues() {
    let mut s = test_state(16, 12);
    // A population-2 city on desert starves to 1 before its Settler can complete.
    for t in &mut s.map.tiles {
        t.terrain = Terrain::Desert;
    }
    let own = add_city(&mut s, P0, at(5, 5), 2);
    let ally = add_city(&mut s, P1, at(8, 5), 1);
    queue(&mut s, own, UnitKind::Settler);
    let before = player_view(&s, P0);
    s.cities.get_mut(&ally).unwrap().producing = Some(ProductionItem::Unit {
        unit: UnitKind::Pikeman,
    });
    s.cities.get_mut(&ally).unwrap().production_stored = 1_000;
    let after = player_view(&s, P0);
    assert_eq!(
        before, after,
        "an ally's hidden build cannot change our forecast"
    );
    assert!(
        after
            .cities
            .iter()
            .find(|c| c.id == ally)
            .unwrap()
            .detail
            .is_none()
    );
    let json = serde_json::to_value(&after).unwrap();
    assert_eq!(
        json["cities"][0]["detail"]["production_stall"]["kind"],
        "Population"
    );
    assert_eq!(serde_json::from_value::<PlayerView>(json).unwrap(), after);
}
