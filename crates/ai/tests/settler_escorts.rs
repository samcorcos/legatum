//! Arrived escorts are assigned from the current PlayerView, including after a reload.

use legatum_ai::{Brain, ScriptedBrain};
use legatum_core::game::{blank_game, spawn_unit};
use legatum_core::rules::cities::found_city;
use legatum_core::rules::units::order_unit;
use legatum_core::state::Map;
use legatum_core::visibility::update_explored;
use legatum_core::*;
use std::collections::BTreeMap;

const ME: PlayerId = PlayerId(0);
const OTHER: PlayerId = PlayerId(1);
const HOME: Hex = Hex::new(3, 8);
const SITE: Hex = Hex::new(7, 8);

struct Scenario {
    state: GameState,
    settler: UnitId,
    escort: UnitId,
    garrison: UnitId,
    spare: UnitId,
}

fn scenario() -> Scenario {
    let mut setup = GameSetup::all_ai(7);
    setup.width = 24;
    setup.height = 16;
    setup.players.truncate(2);
    setup.players[0].faction = Faction::Mongols;
    setup.players[1].faction = Faction::Byzantium;
    let mut state = blank_game(&setup, Map::new(24, 16, Terrain::Plains));
    for (owner, pos) in [(ME, HOME), (OTHER, Hex::new(16, 8))] {
        let founder = spawn_unit(&mut state, owner, UnitKind::Settler, pos);
        found_city(&mut state, founder, &mut Vec::new()).unwrap();
    }
    let garrison = spawn_unit(&mut state, ME, UnitKind::Spearman, HOME);
    let settler = spawn_unit(&mut state, ME, UnitKind::Settler, HOME);
    let escort = spawn_unit(&mut state, ME, UnitKind::Spearman, Hex::new(4, 8));
    let spare = spawn_unit(&mut state, ME, UnitKind::Spearman, Hex::new(2, 8));
    state.units.get_mut(&settler).unwrap().goto = Some(SITE);
    // A live journey has spent movement; full moves plus a goto means it was blocked.
    state.units.get_mut(&settler).unwrap().moves_left = 0;
    // Isolate assignment from site scoring: the settler's public failed-site memory
    // leaves just this legal destination, four hexes from its separately garrisoned home.
    for i in 0..state.map.len() {
        let pos = state.map.hex_at(i);
        state.player_mut(ME).explored.set(i);
        if pos != SITE {
            state.player_mut(ME).failed_city_sites.insert(pos);
        }
    }
    state.turn = 20;
    update_explored(&mut state);
    assert_eq!(HOME.distance(SITE), 4);
    assert!(settlement::known_legal_site(&player_view(&state, ME), SITE));
    assert_eq!(
        data::unit_stats_for(Faction::Mongols, UnitKind::Spearman).moves,
        3
    );
    assert_eq!(
        data::unit_stats_for(Faction::Mongols, UnitKind::Settler).moves,
        2
    );
    Scenario {
        state,
        settler,
        escort,
        garrison,
        spare,
    }
}

fn orders(state: &GameState) -> Vec<Order> {
    let decide = |view: &PlayerView| {
        ScriptedBrain::new(view.faction_of(ME), state.setup.seed, ME).orders(view)
    };
    let generated = decide(&player_view(state, ME));
    let loaded: GameState = serde_json::from_str(&serde_json::to_string(state).unwrap()).unwrap();
    assert_eq!(generated, decide(&player_view(&loaded, ME)));
    assert!(
        validate_orders(state, ME, &generated).is_empty(),
        "{generated:?}"
    );
    generated
}

fn unit_order(orders: &[Order], unit: UnitId) -> &Order {
    let actions: Vec<_> = orders
        .iter()
        .filter(|o| order_unit(o) == Some(unit))
        .collect();
    assert_eq!(actions.len(), 1, "unit {unit:?}: {orders:?}");
    actions[0]
}

fn assert_holds(orders: &[Order], unit: UnitId) {
    assert!(
        matches!(
            unit_order(orders, unit),
            Order::SkipUnit { .. } | Order::UpgradeUnit { .. }
        ),
        "unit {unit:?} abandoned its site: {orders:?}"
    );
}

fn play(state: &GameState, orders: Vec<Order>) -> GameState {
    let (next, events) = resolve_turn(state, &BTreeMap::from([(ME, orders)]));
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::OrderFailed { .. })),
        "{events:?}"
    );
    next
}

#[test]
fn faster_escort_waits_for_its_settler_through_founding_without_a_second_escort() {
    let Scenario {
        mut state,
        settler,
        escort,
        garrison,
        spare,
    } = scenario();
    let mut arrived = false;
    let mut founded = false;
    for _ in 0..6 {
        let generated = orders(&state);
        if state.units[&escort].pos == SITE {
            arrived = true;
            assert_holds(&generated, escort);
        }
        assert_holds(&generated, garrison);
        assert_ne!(
            unit_order(&generated, spare),
            &Order::MoveUnit {
                unit: spare,
                to: SITE
            }
        );
        if let Some(s) = state.units.get(&settler) {
            if s.pos != SITE {
                // Crossing the commitment deadline can confirm the same route
                // explicitly; both orders must still arrive and found below.
                assert!(matches!(
                    unit_order(&generated, settler),
                    Order::MoveUnit { to, .. } | Order::MoveUnitPath { to, .. }
                        if *to == SITE
                ));
            }
        } else {
            founded = true;
            break;
        }
        state = play(&state, generated);
        if arrived {
            assert_eq!(state.units[&escort].pos, SITE);
        } else {
            assert_eq!(state.units[&escort].pos, SITE, "soldier must arrive first");
            assert_ne!(
                state.units[&settler].pos, SITE,
                "settler must still be approaching"
            );
        }
        assert_eq!(state.units[&garrison].pos, HOME);
        assert_eq!(
            state
                .units_of(ME)
                .filter(|u| data::unit_stats(u.kind).military && u.pos == SITE)
                .count(),
            1
        );
    }
    assert!(arrived && founded);
    assert!(state.cities_of(ME).any(|c| c.pos == SITE));
}

#[test]
fn arrived_escort_leaves_when_the_settler_disappears_or_changes_destination() {
    for changed_destination in [false, true] {
        let Scenario {
            mut state,
            settler,
            escort,
            ..
        } = scenario();
        state.units.get_mut(&escort).unwrap().pos = SITE;
        if changed_destination {
            let new_site = Hex::new(3, 12);
            state.player_mut(ME).failed_city_sites.insert(SITE);
            state.player_mut(ME).failed_city_sites.remove(&new_site);
            state.units.get_mut(&settler).unwrap().goto = Some(new_site);
        } else {
            state.units.remove(&settler);
        }
        update_explored(&mut state);
        let generated = orders(&state);
        assert!(
            matches!(unit_order(&generated, escort), Order::MoveUnit { to, .. } if *to != SITE)
        );
        assert_ne!(play(&state, generated).units[&escort].pos, SITE);
    }
}

#[test]
fn another_friendly_soldier_at_the_site_becomes_the_only_escort() {
    let Scenario {
        mut state,
        settler,
        escort,
        spare,
        ..
    } = scenario();
    state.units.get_mut(&spare).unwrap().pos = SITE;
    update_explored(&mut state);
    let generated = orders(&state);
    assert_eq!(
        unit_order(&generated, settler),
        &Order::MoveUnit {
            unit: settler,
            to: SITE
        }
    );
    assert_holds(&generated, spare);
    assert_ne!(
        unit_order(&generated, escort),
        &Order::MoveUnit {
            unit: escort,
            to: SITE
        }
    );
    let next = play(&state, generated);
    assert_eq!(next.units[&spare].pos, SITE);
    assert_ne!(next.units[&escort].pos, SITE);
}

#[test]
fn an_arrived_escort_still_retreats_from_an_overwhelming_threat() {
    let Scenario {
        mut state, escort, ..
    } = scenario();
    let generated = orders(&state);
    state = play(&state, generated);
    assert_eq!(state.units[&escort].pos, SITE);
    state.units.get_mut(&escort).unwrap().hp = 20;
    for relation in &mut state.diplomacy.as_mut().unwrap().relations {
        relation.state = diplomacy::RelationState::War;
    }
    let threat = spawn_unit(&mut state, OTHER, UnitKind::Pikeman, Hex::new(9, 8));
    update_explored(&mut state);
    assert!(player_view(&state, ME).units.iter().any(|u| u.id == threat));
    let generated = orders(&state);
    assert!(matches!(unit_order(&generated, escort), Order::MoveUnit { to, .. } if *to != SITE));
    state = play(&state, generated);
    assert!(state.units[&escort].pos.distance(state.units[&threat].pos) > 2);
    assert!(state.units[&escort].hp >= 20);
}

#[test]
fn another_friendly_scout_blocks_recruitment() {
    let Scenario {
        mut state,
        escort,
        spare,
        ..
    } = scenario();
    spawn_unit(&mut state, ME, UnitKind::Scout, SITE);
    update_explored(&mut state);
    let generated = orders(&state);
    for soldier in [escort, spare] {
        assert_ne!(
            unit_order(&generated, soldier),
            &Order::MoveUnit {
                unit: soldier,
                to: SITE
            }
        );
    }
    let next = play(&state, generated);
    // Scouting remains an independent assignment; soldiers respect current occupancy
    // even if the scout is about to leave in the simultaneous movement phase.
    assert_ne!(next.units[&escort].pos, SITE);
    assert_ne!(next.units[&spare].pos, SITE);
}

#[test]
fn a_settler_founding_without_an_escort_does_not_recruit_one() {
    let Scenario {
        mut state,
        settler,
        escort,
        spare,
        ..
    } = scenario();
    state.units.get_mut(&settler).unwrap().pos = SITE;
    update_explored(&mut state);
    let generated = orders(&state);
    assert_eq!(
        unit_order(&generated, settler),
        &Order::FoundCity { unit: settler }
    );
    for soldier in [escort, spare] {
        assert_ne!(
            unit_order(&generated, soldier),
            &Order::MoveUnit {
                unit: soldier,
                to: SITE
            }
        );
    }
    assert!(play(&state, generated).cities_of(ME).any(|c| c.pos == SITE));
}

#[test]
fn archived_arrivals_hold_instead_of_repeating_the_site_home_site_cycle() {
    // Both fogged views come from a hash-verified replay, not a reconstructed world.
    // Provenance and baseline reproduction are in fixtures/escort-t168-t170.md.
    let fixture: serde_json::Value = serde_json::from_reader(flate2::read::GzDecoder::new(
        &include_bytes!("fixtures/escort-t168-t170.json.gz")[..],
    ))
    .unwrap();
    for case in fixture["cases"].as_array().unwrap() {
        // The archive predates typed VP sources: its labels become `VpSource`s (only the faction
        // goal matters to the AI; title and milestone labels are dropped).
        let mut raw = case["view"].clone();
        for player in raw["players"].as_array_mut().unwrap() {
            let id = player["id"].clone();
            let typed: Vec<serde_json::Value> = player["vp_sources"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|entry| entry[0] == "Faction goal")
                .map(|entry| serde_json::json!([{"kind": "FactionGoal", "of": id}, entry[1]]))
                .collect();
            player["vp_sources"] = typed.into();
        }
        // The Military Engineer retired with the Siege upgrade: the archived view's Engineers are gone.
        raw["units"].as_array_mut().unwrap().retain(|unit| {
            !matches!(
                unit["kind"].as_str(),
                Some("MilitaryEngineer" | "SiegeEngine" | "Bombard")
            )
        });
        // It also predates eras: give every technology option the new fields.
        raw["research"]["era_name"] = "Ancient".into();
        raw["research"]["unlocks"]
            .as_array_mut()
            .unwrap()
            .retain(|unlock| {
                !matches!(
                    unlock["item"]["building"].as_str(),
                    Some("AidStation" | "Reactor" | "Embassy" | "Archive")
                ) && !matches!(
                    unlock["item"]["unit"].as_str(),
                    Some("MilitaryEngineer" | "SiegeEngine" | "Bombard")
                )
            });
        for list in ["available", "catalog"] {
            for option in raw["research"][list].as_array_mut().into_iter().flatten() {
                option["era"] = 0.into();
                option["researchable"] = true.into();
            }
        }
        for city in raw["cities"].as_array_mut().unwrap() {
            if city["detail"].is_null() {
                continue;
            }
            if let Some(buildings) = city["detail"]["buildings"].as_array_mut() {
                buildings
                    .retain(|building| !matches!(building.as_str(), Some("Embassy" | "Archive")));
            }
            if let Some(sources) = city["detail"]["yields_after"]["sources"].as_array_mut() {
                sources.retain(|source| {
                    !matches!(source["label"].as_str(), Some("Embassy" | "Archive"))
                });
            }
        }
        let view: PlayerView = serde_json::from_value(raw).unwrap();
        let seed: u64 = case["seed"].as_str().unwrap().parse().unwrap();
        let mut expected: Vec<Order> =
            serde_json::from_value(case["archived_orders"].clone()).unwrap();
        let escort = UnitId(29);
        let settler = UnitId(37);
        let site = Hex::new(38, 12);
        assert!(
            view.units
                .iter()
                .any(|u| u.id == escort && u.pos == site && u.hp == 100)
        );
        assert!(
            view.units
                .iter()
                .any(|u| u.id == settler && u.goto == Some(site) && u.hp == 100)
        );
        assert!(view.players.iter().all(|p| !view.at_war(view.me, p.id)));
        assert_eq!(
            unit_order(&expected, escort),
            &Order::MoveUnit {
                unit: escort,
                to: Hex::new(35, 15)
            }
        );
        assert_eq!(
            unit_order(&expected, settler),
            &Order::MoveUnit {
                unit: settler,
                to: site
            }
        );
        *expected
            .iter_mut()
            .find(|o| order_unit(o) == Some(escort))
            .unwrap() = Order::SkipUnit { unit: escort };
        let generated = ScriptedBrain::with_difficulty(
            view.faction_of(view.me),
            seed,
            view.me,
            AiDifficulty::Normal,
        )
        .orders(&view);
        // Mongol distant-scout policy may change unrelated orders.
        // The arrived escort must still hold while the settler keeps its site commitment.
        assert_eq!(
            unit_order(&generated, escort),
            &Order::SkipUnit { unit: escort },
            "archived turn {}",
            view.turn
        );
        assert_eq!(
            unit_order(&generated, settler),
            &Order::MoveUnit {
                unit: settler,
                to: site
            },
            "archived turn {}",
            view.turn
        );
        // The era-gated research queue and Alliance offers (free since 2026-10-22) are new since
        // the archive; everything else must match.
        let not_research = |orders: &[Order]| {
            orders
                .iter()
                .filter(|o| {
                    !matches!(
                        o,
                        Order::SetResearch { .. }
                            | Order::SetResearchQueue { .. }
                            | Order::ProposeTreaty { .. }
                    )
                })
                .count()
        };
        assert_eq!(
            not_research(&generated),
            not_research(&expected),
            "no extra escort recruited"
        );
    }
}
