use super::*;
use crate::game::{blank_game, spawn_unit};
use crate::new_game;
use crate::rules::cities::found_city;
use crate::state::{GameSetup, Map};

/// Two empires on open grassland, each with one city and a Soldier beside it.
fn duel() -> GameState {
    let mut setup = GameSetup::all_ai(7);
    setup.players.truncate(2);
    setup.players[0].faction = Faction::Sparta;
    setup.players[1].faction = Faction::India;
    setup.width = 20;
    setup.height = 12;
    setup.barbarians = false;
    let mut state = blank_game(&setup, Map::new(20, 12, Terrain::Grassland));
    for (owner, col) in [(PlayerId(0), 4), (PlayerId(1), 14)] {
        let settler = spawn_unit(
            &mut state,
            owner,
            UnitKind::Settler,
            Hex::from_offset(col, 5),
        );
        found_city(&mut state, settler, &mut vec![]).expect("city");
        spawn_unit(
            &mut state,
            owner,
            UnitKind::Soldier,
            Hex::from_offset(col, 6),
        );
    }
    state
}

fn hold() -> Objective {
    Objective {
        summary: "Hold your city with a Soldier".into(),
        turns: 3,
        win: vec![Condition::MinCombatUnits(1)],
        end_on_win: false,
        keep: vec![Condition::HoldStartingCities],
        score: vec![
            ScoreTerm {
                metric: Metric::StartingCitiesHeld,
                weight: 100,
            },
            ScoreTerm {
                metric: Metric::CombatUnits,
                weight: 10,
            },
        ],
    }
}

fn city_of(state: &GameState, p: PlayerId) -> CityId {
    state.cities_of(p).next().unwrap().id
}

#[test]
fn a_held_city_plays_on_until_the_deadline_then_wins() {
    let start = duel();
    let mut now = start.clone();
    let s = evaluate(&hold(), PlayerId(0), &start, &now);
    assert_eq!(s.outcome, Outcome::Playing);
    assert_eq!((s.start_turn, s.end_turn, s.turns_left), (1, 4, 3));
    assert_eq!(s.score, 110);
    assert_eq!(s.conditions.len(), 2);
    assert!(s.conditions[0].keep && s.conditions[0].met);
    assert_eq!(s.conditions[0].progress, "1 of 1");
    now.turn = 4;
    let s = evaluate(&hold(), PlayerId(0), &start, &now);
    assert_eq!(s.outcome, Outcome::Won);
    assert_eq!(s.turns_left, 0);
}

#[test]
fn losing_a_starting_city_ends_the_scenario_at_once() {
    let start = duel();
    let mut now = start.clone();
    let id = city_of(&start, PlayerId(0));
    now.cities.get_mut(&id).unwrap().owner = PlayerId(1);
    let s = evaluate(&hold(), PlayerId(0), &start, &now);
    assert_eq!(s.outcome, Outcome::Lost);
    assert!(s.reason.contains("Hold your 1 city"), "{}", s.reason);
    assert_eq!(s.score, 10, "only the Soldier still scores");
}

#[test]
fn an_unmet_win_condition_at_the_deadline_loses() {
    let start = duel();
    let mut now = start.clone();
    now.units.retain(|_, u| u.owner != PlayerId(0));
    now.turn = 4;
    let s = evaluate(&hold(), PlayerId(0), &start, &now);
    assert_eq!(s.outcome, Outcome::Lost);
    assert!(s.reason.starts_with("Time ran out"), "{}", s.reason);
}

#[test]
fn a_capture_ends_early_and_scores_turns_to_spare() {
    let start = duel();
    let target = city_of(&start, PlayerId(1));
    let objective = Objective {
        summary: "Take their city".into(),
        turns: 10,
        win: vec![Condition::OwnCity(target)],
        end_on_win: true,
        keep: vec![],
        score: vec![
            ScoreTerm {
                metric: Metric::ObjectiveMet,
                weight: 100,
            },
            ScoreTerm {
                metric: Metric::TurnsLeft,
                weight: 10,
            },
            ScoreTerm {
                metric: Metric::CityDamage(target),
                weight: 1,
            },
            ScoreTerm {
                metric: Metric::CitiesCaptured,
                weight: 5,
            },
        ],
    };
    let mut now = start.clone();
    now.cities.get_mut(&target).unwrap().hp = 120;
    now.turn = 3;
    let s = evaluate(&objective, PlayerId(0), &start, &now);
    assert_eq!(s.outcome, Outcome::Playing);
    assert_eq!(s.score, 80, "partial credit for damage only");
    assert_eq!(s.conditions[0].progress, "Held by India");
    now.cities.get_mut(&target).unwrap().owner = PlayerId(0);
    let s = evaluate(&objective, PlayerId(0), &start, &now);
    assert_eq!(s.outcome, Outcome::Won);
    assert_eq!(s.turns_left, 8);
    assert_eq!(s.score, 100 + 80 + 5);
    assert!(s.reason.contains("8 turns to spare"));
}

#[test]
fn capturing_the_rivals_city_counts_but_founding_or_taking_another_does_not() {
    let start = duel();
    let objective = Objective {
        summary: "Capture any of their cities".into(),
        turns: 10,
        win: vec![Condition::CapturedFrom {
            rival: PlayerId(1),
            min: 1,
        }],
        end_on_win: true,
        keep: vec![],
        score: vec![ScoreTerm {
            metric: Metric::CitiesCaptured,
            weight: 10,
        }],
    };
    let mut now = start.clone();
    now.turn = 3;
    let settler = spawn_unit(
        &mut now,
        PlayerId(0),
        UnitKind::Settler,
        Hex::from_offset(9, 9),
    );
    found_city(&mut now, settler, &mut vec![]).unwrap();
    let s = evaluate(&objective, PlayerId(0), &start, &now);
    assert_eq!(
        s.outcome,
        Outcome::Playing,
        "a founded city is not a capture"
    );
    assert_eq!(s.conditions[0].progress, "0 of 1");
    // A city founded by someone else (a third empire) is not the rival's.
    let settler = spawn_unit(
        &mut now,
        PlayerId(1),
        UnitKind::Settler,
        Hex::from_offset(2, 2),
    );
    let third = found_city(&mut now, settler, &mut vec![]).unwrap();
    now.cities.get_mut(&third).unwrap().owner = PlayerId(0);
    now.cities.get_mut(&third).unwrap().founder = PlayerId(1);
    let s = evaluate(&objective, PlayerId(0), &start, &now);
    assert_eq!(
        s.outcome,
        Outcome::Won,
        "a city the rival founded counts once the seat owns it"
    );
    now.cities.get_mut(&third).unwrap().founder = PlayerId(0);
    let s = evaluate(&objective, PlayerId(0), &start, &now);
    assert_eq!(s.outcome, Outcome::Playing);
    let target = city_of(&start, PlayerId(1));
    now.cities.get_mut(&target).unwrap().owner = PlayerId(0);
    let s = evaluate(&objective, PlayerId(0), &start, &now);
    assert_eq!(s.outcome, Outcome::Won);
    assert_eq!(s.conditions[0].progress, "1 of 1");
}

#[test]
fn a_founded_city_does_not_replace_a_lost_starting_city() {
    let start = duel();
    let objective = Objective {
        summary: "Keep your starting city".into(),
        turns: 5,
        win: vec![Condition::MinStartingCities(1)],
        end_on_win: false,
        keep: vec![Condition::MinStartingCities(1)],
        score: vec![ScoreTerm {
            metric: Metric::StartingCitiesHeld,
            weight: 1,
        }],
    };
    let mut now = start.clone();
    now.turn = 2;
    let settler = spawn_unit(
        &mut now,
        PlayerId(0),
        UnitKind::Settler,
        Hex::from_offset(9, 9),
    );
    found_city(&mut now, settler, &mut vec![]).unwrap();
    assert_eq!(
        evaluate(&objective, PlayerId(0), &start, &now).outcome,
        Outcome::Playing
    );
    let home = city_of(&start, PlayerId(0));
    now.cities.get_mut(&home).unwrap().owner = PlayerId(1);
    assert!(
        now.cities_of(PlayerId(0)).count() >= 1,
        "the new city is still ours"
    );
    let s = evaluate(&objective, PlayerId(0), &start, &now);
    assert_eq!(s.outcome, Outcome::Lost, "{}", s.reason);
    assert_eq!(s.conditions[0].progress, "0 of 1");
}

#[test]
fn units_gone_needs_every_named_unit_dead() {
    let start = duel();
    let enemy: Vec<UnitId> = start.units_of(PlayerId(1)).map(|u| u.id).collect();
    assert_eq!(enemy.len(), 1);
    let objective = Objective {
        summary: "Kill their Soldier".into(),
        turns: 5,
        win: vec![Condition::UnitsGone(vec![enemy[0], UnitId(9999)])],
        end_on_win: false,
        keep: vec![],
        score: vec![ScoreTerm {
            metric: Metric::CombatUnits,
            weight: 1,
        }],
    };
    let mut now = start.clone();
    now.turn = 6;
    let s = evaluate(&objective, PlayerId(0), &start, &now);
    assert_eq!(s.outcome, Outcome::Lost);
    assert_eq!(s.conditions[0].progress, "1 of 2");
    now.units.remove(&enemy[0]);
    let s = evaluate(&objective, PlayerId(0), &start, &now);
    assert_eq!(s.outcome, Outcome::Won);
    assert_eq!(s.conditions[0].progress, "2 of 2");
}

#[test]
fn stop_win_fails_only_when_that_rival_wins() {
    let start = duel();
    let rival = PlayerId(1);
    let objective = Objective {
        summary: "Stop them winning".into(),
        turns: 5,
        win: vec![Condition::StopWin { rival }],
        end_on_win: false,
        keep: vec![Condition::StopWin { rival }],
        score: vec![ScoreTerm {
            metric: Metric::ObjectiveMet,
            weight: 100,
        }],
    };
    let mut now = start.clone();
    now.turn = start.turn + 2;
    let team = now.team_of(rival);
    now.vp_holds.insert(team, 3);
    let s = evaluate(&objective, PlayerId(0), &start, &now);
    assert_eq!(s.outcome, Outcome::Playing, "{}", s.reason);
    assert_eq!(
        s.conditions[0].progress,
        "Holding its target, 3 of 10 turns"
    );
    // The rival wins before the window ends: a loss, as soon as it happens.
    let mut won = now.clone();
    won.finished = true;
    won.winner = Some(team);
    let s = evaluate(&objective, PlayerId(0), &start, &won);
    assert_eq!(s.outcome, Outcome::Lost, "{}", s.reason);
    assert!(s.reason.starts_with("Failed: Stop"), "{}", s.reason);
    // The window runs out with the rival still short: met.
    let mut over = start.clone();
    over.turn = start.turn + 5;
    let s = evaluate(&objective, PlayerId(0), &start, &over);
    assert_eq!(s.outcome, Outcome::Won, "{}", s.reason);
    assert_eq!(s.conditions[0].progress, "Not holding its target");
}

#[test]
fn score_only_windows_complete_and_count_founded_cities() {
    let start = duel();
    let objective = Objective {
        summary: "Expand".into(),
        turns: 2,
        win: vec![],
        end_on_win: false,
        keep: vec![],
        score: vec![
            ScoreTerm {
                metric: Metric::Cities,
                weight: 100,
            },
            ScoreTerm {
                metric: Metric::CitiesFounded,
                weight: 1,
            },
        ],
    };
    let mut now = start.clone();
    let settler = spawn_unit(
        &mut now,
        PlayerId(0),
        UnitKind::Settler,
        Hex::from_offset(9, 9),
    );
    found_city(&mut now, settler, &mut vec![]).unwrap();
    let s = evaluate(&objective, PlayerId(0), &start, &now);
    assert_eq!((s.outcome, s.score), (Outcome::Playing, 201));
    now.turn = 3;
    assert_eq!(
        evaluate(&objective, PlayerId(0), &start, &now).outcome,
        Outcome::Complete
    );
}

#[test]
fn an_eliminated_seat_loses() {
    let start = duel();
    let mut now = start.clone();
    now.players[0].alive = false;
    assert_eq!(
        evaluate(&hold(), PlayerId(0), &start, &now).outcome,
        Outcome::Lost
    );
}

#[test]
fn objective_json_is_readable_and_round_trips() {
    let json = r#"{"summary":"s","turns":5,"win":[{"OwnCity":3},{"RivalMaxCities":{"rival":2,"max":4}}],
        "end_on_win":true,"keep":["HoldStartingCities",{"MinCombatUnits":2}],
        "score":[{"metric":"Cities","weight":100},{"metric":{"RivalCities":2},"weight":-50}]}"#;
    let o: Objective = serde_json::from_str(json).unwrap();
    assert_eq!(o.keep[0], Condition::HoldStartingCities);
    assert_eq!(o.win[0], Condition::OwnCity(CityId(3)));
    assert_eq!(o.score[1].metric, Metric::RivalCities(PlayerId(2)));
    let back: Objective = serde_json::from_str(&serde_json::to_string(&o).unwrap()).unwrap();
    assert_eq!(back, o);
}

#[test]
fn objectives_must_name_real_cities_and_rivals() {
    let start = duel();
    let mut o = hold();
    assert!(check_objective(&o, PlayerId(0), &start).is_empty());
    o.win.push(Condition::OwnCity(CityId(99)));
    o.score.push(ScoreTerm {
        metric: Metric::RivalCities(PlayerId(0)),
        weight: 1,
    });
    let errors = check_objective(&o, PlayerId(0), &start);
    assert_eq!(errors.len(), 2, "{errors:?}");
    assert!(!check_objective(&o, PlayerId(5), &start).is_empty());
}

#[test]
fn generated_games_are_legal() {
    for seed in [1, 2, 3] {
        let state = new_game(&GameSetup::all_ai(seed));
        assert_eq!(validate_state(&state), Vec::<String>::new());
    }
    assert_eq!(validate_state(&duel()), Vec::<String>::new());
}

#[test]
fn illegal_edits_are_reported() {
    let base = duel();
    let check = |edit: &dyn Fn(&mut GameState), needle: &str| {
        let mut s = base.clone();
        edit(&mut s);
        let errors = validate_state(&s);
        assert!(
            errors.iter().any(|e| e.contains(needle)),
            "expected {needle:?} in {errors:?}"
        );
    };
    // Two Soldiers on one tile.
    check(
        &|s| {
            spawn_unit(s, PlayerId(0), UnitKind::Soldier, Hex::from_offset(4, 6));
        },
        "stack",
    );
    // A Soldier inside a peaceful neighbour's borders.
    check(
        &|s| {
            spawn_unit(s, PlayerId(0), UnitKind::Soldier, Hex::from_offset(13, 5));
        },
        "territory without being at war",
    );
    // A unit on water.
    check(
        &|s| {
            let h = Hex::from_offset(10, 1);
            s.map.get_mut(h).unwrap().terrain = Terrain::Ocean;
            spawn_unit(s, PlayerId(0), UnitKind::Scout, h);
        },
        "impassable",
    );
    // A second city three hexes away.
    check(
        &|s| {
            let settler = spawn_unit(s, PlayerId(0), UnitKind::Settler, Hex::from_offset(9, 9));
            let id = found_city(s, settler, &mut vec![]).unwrap();
            let c = s.cities.get_mut(&id).unwrap();
            let old = c.pos;
            c.pos = Hex::from_offset(7, 5);
            s.map.get_mut(old).unwrap().city = None;
            let t = s.map.get_mut(Hex::from_offset(7, 5)).unwrap();
            t.owner = Some(PlayerId(0));
            t.city = Some(id);
        },
        "within 3 hexes",
    );
    // Hit points out of range, and a unit that outlives its empire.
    check(
        &|s| {
            s.units.values_mut().next().unwrap().hp = 0;
        },
        "0 HP",
    );
    check(
        &|s| {
            s.players[1].alive = false;
        },
        "eliminated empire",
    );
}
