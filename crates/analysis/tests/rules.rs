use legatum_analysis::{Behavior, Config, Report, analyze_recording, scenario};
use legatum_core::{
    game::{blank_game, spawn_unit},
    rules::cities::found_city,
    state::Map,
    *,
};
use legatum_wasm::{history::Recording, session::OrderMap};

fn h(x: i32, y: i32) -> Hex {
    Hex::from_offset(x, y)
}
fn fixture() -> GameState {
    let mut setup = GameSetup::all_ai(123);
    setup.players.truncate(2);
    for p in &mut setup.players {
        p.faction = Faction::Switzerland;
    }
    setup.players[1].is_human = true;
    setup.width = 20;
    setup.height = 14;
    let mut s = blank_game(&setup, Map::new(20, 14, Terrain::Grassland));
    // These attack/response fixtures require combat, not the new-game Peace default.
    for relation in &mut s.diplomacy.as_mut().unwrap().relations {
        relation.state = diplomacy::RelationState::War;
    }
    for (p, pos) in [(PlayerId(0), h(4, 4)), (PlayerId(1), h(17, 10))] {
        let u = spawn_unit(&mut s, p, UnitKind::Settler, pos);
        found_city(&mut s, u, &mut vec![]).unwrap();
        s.player_mut(p).gold = 5000;
    }
    s
}
fn tape(mut s: GameState, turns: Vec<OrderMap>) -> Recording {
    legatum_core::visibility::update_explored(&mut s);
    let mut rec = Recording::new(&s);
    rec.metadata.game_id = "test-fixture".into();
    for orders in turns {
        let turn = s.turn;
        s = resolve_turn(&s, &orders).0;
        rec.observe(turn, &orders, &s);
    }
    rec.stop("stalled");
    rec
}
fn mine(orders: Vec<Order>) -> OrderMap {
    OrderMap::from([(PlayerId(0), orders)])
}
fn report(rec: &Recording) -> Report {
    analyze_recording(rec, Config { settler_turns: 2 }).unwrap()
}
fn count(report: &Report, rule: Behavior) -> usize {
    report
        .incidents
        .iter()
        .filter(|i| i.behavior == rule)
        .count()
}

#[test]
fn ranger_reproduction_detects_all_three_reported_failures_deterministically() {
    let tape = scenario::ranger_repro();
    tape.replay().unwrap();
    let a = report(&tape);
    for rule in [
        Behavior::UnansweredAttack,
        Behavior::ScoutNotFleeing,
        Behavior::GarrisonUnderRangedFire,
    ] {
        assert!(count(&a, rule) > 0, "missing {rule:?}");
    }
    assert!(count(&a, Behavior::UnansweredAttack) >= 2);
    assert!(a.turns.iter().any(|t| t.events.contains("units killed")));
    // The garrison is flagged while a favorable sortie on the Archer was available. Since
    // cities shield their garrisons (core `combat::city_tile_defender`, 2026-10-03), the
    // bombardment hits the city, the garrison stays healthy and its sortie stays favorable
    // through the last turn, so every bombarded turn is flagged.
    let garrison: Vec<_> = a
        .incidents
        .iter()
        .filter(|i| i.behavior == Behavior::GarrisonUnderRangedFire)
        .map(|i| i.turn)
        .collect();
    assert_eq!(garrison, [2, 3, 4]);
    assert!(count(&a, Behavior::GarrisonUnderRangedFire) >= 1);
    assert_eq!(
        serde_json::to_value(&a).unwrap(),
        serde_json::to_value(report(&tape)).unwrap()
    );
    assert!(a.incidents.iter().all(|i| i.player == 0));
}

#[test]
fn responses_reset_attack_streak_even_when_the_movement_is_blocked() {
    let mut s = fixture();
    let guard = spawn_unit(&mut s, PlayerId(0), UnitKind::Spearman, h(4, 4));
    let archer = spawn_unit(&mut s, PlayerId(1), UnitKind::Archer, h(6, 4));
    spawn_unit(&mut s, PlayerId(0), UnitKind::Spearman, h(3, 4));
    let shot = Order::RangedAttack {
        unit: archer,
        target: h(4, 4),
    };
    let turns = vec![
        OrderMap::from([(PlayerId(1), vec![shot.clone()])]),
        OrderMap::from([
            (PlayerId(1), vec![shot]),
            (
                PlayerId(0),
                vec![Order::MoveUnit {
                    unit: guard,
                    to: h(3, 4),
                }],
            ),
        ]),
    ];
    let rec = tape(s, turns);
    assert_eq!(
        rec.replay().unwrap().units[&guard].pos,
        h(4, 4),
        "the ordered response was blocked by a parked unit"
    );
    let r = report(&rec);
    assert_eq!(count(&r, Behavior::UnansweredAttack), 0);
    assert_eq!(count(&r, Behavior::GarrisonUnderRangedFire), 0);
}

#[test]
fn scout_requires_visible_hostile_and_a_known_safe_step_and_excludes_humans() {
    let mut s = fixture();
    let scout = spawn_unit(&mut s, PlayerId(0), UnitKind::Scout, h(8, 6));
    spawn_unit(&mut s, PlayerId(1), UnitKind::Spearman, h(9, 6));
    let r = report(&tape(
        s.clone(),
        vec![mine(vec![Order::MoveUnit {
            unit: scout,
            to: h(6, 6),
        }])],
    ));
    assert_eq!(count(&r, Behavior::ScoutNotFleeing), 0);
    // Allied adjacent soldiers are not threats.
    s.players[1].team = s.players[0].team;
    s.setup.players[1].team = s.setup.players[0].team;
    assert_eq!(
        count(
            &report(&tape(s.clone(), vec![OrderMap::new()])),
            Behavior::ScoutNotFleeing
        ),
        0
    );
    s.players[1].team = TeamId(1);
    s.setup.players[1].team = TeamId(1);
    for pos in h(8, 6).neighbors() {
        if pos != h(9, 6) {
            s.map.get_mut(pos).unwrap().terrain = Terrain::Mountains;
        }
    }
    assert_eq!(
        count(
            &report(&tape(s, vec![OrderMap::new()])),
            Behavior::ScoutNotFleeing
        ),
        0
    );
}

#[test]
fn healthy_idle_military_near_threatened_city_but_not_healing_or_garrison() {
    let mut s = fixture();
    spawn_unit(&mut s, PlayerId(0), UnitKind::Spearman, h(3, 5));
    spawn_unit(&mut s, PlayerId(0), UnitKind::Spearman, h(4, 4));
    let wounded = spawn_unit(&mut s, PlayerId(0), UnitKind::Spearman, h(3, 3));
    s.units.get_mut(&wounded).unwrap().hp = 10;
    spawn_unit(&mut s, PlayerId(1), UnitKind::Archer, h(6, 4));
    let r = report(&tape(s, vec![OrderMap::new(); 2]));
    assert_eq!(count(&r, Behavior::IdleMilitaryThreatenedCity), 1);
}

#[test]
fn repeated_engine_bounces_not_just_stationary_units() {
    let mut s = fixture();
    // Different owners on one team still bounce on ordinary ground. Own traffic
    // now takes turns, so it cannot serve as a repeated-Bounce detector fixture.
    let mut ally = s.players[0].clone();
    ally.id = PlayerId(2);
    ally.name = "Ally".into();
    s.players.push(ally);
    s.setup.players.push(s.setup.players[0].clone());
    spawn_unit(&mut s, PlayerId(2), UnitKind::Settler, h(17, 1));
    let a = spawn_unit(&mut s, PlayerId(0), UnitKind::Spearman, h(8, 4));
    let b = spawn_unit(&mut s, PlayerId(2), UnitKind::Spearman, h(10, 4));
    let orders = OrderMap::from([
        (
            PlayerId(0),
            vec![Order::MoveUnit {
                unit: a,
                to: h(9, 4),
            }],
        ),
        (
            PlayerId(2),
            vec![Order::MoveUnit {
                unit: b,
                to: h(9, 4),
            }],
        ),
    ]);
    let r = report(&tape(s, vec![orders.clone(), orders]));
    assert_eq!(count(&r, Behavior::RepeatedBounce), 2);
    assert!(
        r.incidents
            .iter()
            .filter(|i| i.behavior == Behavior::RepeatedBounce)
            .all(|i| i.turn == 2)
    );
}

#[test]
fn movement_oscillation_requires_a_b_a_b() {
    let mut s = fixture();
    let u = spawn_unit(&mut s, PlayerId(0), UnitKind::Spearman, h(8, 4));
    let turns = [h(9, 4), h(8, 4), h(9, 4)]
        .into_iter()
        .map(|to| mine(vec![Order::MoveUnit { unit: u, to }]))
        .collect();
    let r = report(&tape(s, turns));
    assert_eq!(count(&r, Behavior::MovementOscillation), 1);
}

#[test]
fn gold_hoarding_requires_attacks_and_affordable_defense_and_no_spending() {
    let mut s = fixture();
    spawn_unit(&mut s, PlayerId(0), UnitKind::Spearman, h(4, 4));
    let archer = spawn_unit(&mut s, PlayerId(1), UnitKind::Archer, h(6, 4));
    let shot = OrderMap::from([(
        PlayerId(1),
        vec![Order::RangedAttack {
            unit: archer,
            target: h(4, 4),
        }],
    )]);
    let r = report(&tape(s.clone(), vec![shot.clone(); 2]));
    assert_eq!(count(&r, Behavior::GoldHoardedUnderAttack), 1);
    // Buying a defender is a response even while unrelated construction continues.
    let city = s
        .cities
        .values()
        .find(|c| c.owner == PlayerId(0))
        .unwrap()
        .id;
    let mut building = s.clone();
    building
        .player_mut(PlayerId(0))
        .techs
        .extend([TechId::Irrigation, TechId::Currency]);
    building.cities.get_mut(&city).unwrap().producing = Some(ProductionItem::Building {
        building: BuildingKind::Market,
    });
    let mut bought = shot.clone();
    bought.insert(
        PlayerId(0),
        vec![Order::BuyItem {
            city,
            item: ProductionItem::Unit {
                unit: UnitKind::Spearman,
            },
        }],
    );
    let r = report(&tape(building, vec![shot.clone(), bought]));
    assert_eq!(count(&r, Behavior::GoldHoardedUnderAttack), 0);
    assert_eq!(
        count(
            &report(&tape(s.clone(), vec![OrderMap::new(); 2])),
            Behavior::GoldHoardedUnderAttack
        ),
        0
    );
    s.players[0].gold = 0;
    assert_eq!(
        count(
            &report(&tape(s, vec![shot; 2])),
            Behavior::GoldHoardedUnderAttack
        ),
        0
    );
}

#[test]
fn settler_age_is_observed_and_founding_ends_it() {
    let mut s = fixture();
    let u = spawn_unit(&mut s, PlayerId(0), UnitKind::Settler, h(10, 5));
    let r = report(&tape(s.clone(), vec![OrderMap::new(); 3]));
    assert_eq!(count(&r, Behavior::SettlerWandering), 1);
    let r = report(&tape(
        s,
        vec![
            OrderMap::new(),
            OrderMap::new(),
            mine(vec![Order::FoundCity { unit: u }]),
        ],
    ));
    assert_eq!(count(&r, Behavior::SettlerWandering), 0);
}

#[test]
fn idle_research_and_production_do_not_flag_completion_turns() {
    let mut s = fixture();
    s.cities
        .values_mut()
        .find(|c| c.owner == PlayerId(0))
        .unwrap()
        .buildings
        .insert(BuildingKind::Library);
    let r = report(&tape(s.clone(), vec![OrderMap::new()]));
    assert_eq!(count(&r, Behavior::ResearchIdle), 1);
    assert_eq!(count(&r, Behavior::ProductionIdle), 1);
    let city = *s.cities.keys().next().unwrap();
    let view = player_view(&s, PlayerId(0));
    let tech = view.research.available[0].tech;
    let item = ProductionItem::Unit {
        unit: UnitKind::Spearman,
    };
    let r = report(&tape(
        s,
        vec![mine(vec![
            Order::SetResearch { tech },
            Order::BuyItem { city, item },
        ])],
    ));
    assert_eq!(count(&r, Behavior::ResearchIdle), 0);
    assert_eq!(count(&r, Behavior::ProductionIdle), 0);
    assert_eq!(r.turns[0].players.len(), 1, "human seats are excluded");
    let player = &r.turns[0].players[0];
    assert_eq!(player.player, 0);
    assert_eq!(player.cities, 1);
    assert!(player.alive);
    assert!(
        player
            .outcomes
            .iter()
            .any(|o| o.contains("bought") && o.contains("Spearman")),
        "incident-free spending still has attributed evidence"
    );
    assert!(r.narrative(64_000).contains("bought"));
    assert_eq!(r.map_size, [20, 14]);
    assert!(!r.finished, "a stopped tape is not a lost game");
    assert!(!r.has_winner);
    assert_eq!(r.seats[0].difficulty, AiDifficulty::Normal);
}

#[test]
fn success_outcomes_use_verified_replay_and_never_include_free_form_names() {
    let mut s = fixture();
    let settler = spawn_unit(&mut s, PlayerId(0), UnitKind::Settler, h(10, 5));
    s.players[0].name = "untrusted-player-name".into();
    let mut recording = tape(s, vec![mine(vec![Order::FoundCity { unit: settler }])]);
    // Telemetry is stored in archives but is not a replay-verified source of behavioral claims.
    recording.telemetry[1].players[0].cities = 999;
    let r = report(&recording);
    let player = &r.turns[0].players[0];
    assert_eq!(player.cities, 2);
    assert!(player.outcomes.iter().any(|o| o.contains("founded city")));
    assert!(!r.narrative(64_000).contains("untrusted-player-name"));
}

#[test]
fn refuses_bad_hashes_and_rules_and_preserves_anchor_turn_numbers() {
    let mut s = fixture();
    s.turn = 41;
    let mut t = tape(s, vec![OrderMap::new()]);
    let r = report(&t);
    assert_eq!(r.first_turn, 41);
    assert!(!r.complete_history);
    t.tape.turns[0].hash = "0".into();
    assert!(
        analyze_recording(&t, Config::default())
            .unwrap_err()
            .contains("divergent turn: 41")
    );
    t.metadata.rules_fingerprint = "wrong".into();
    assert!(
        analyze_recording(&t, Config::default())
            .unwrap_err()
            .contains("incompatible engine")
    );
}

#[test]
fn rates_use_opportunities_and_attacker_factions_and_compact_output_discloses_omissions() {
    let r = report(&scenario::ranger_repro());
    let rates: Vec<_> = r
        .rates
        .iter()
        .filter(|r| r.behavior == Behavior::UnansweredAttack)
        .collect();
    assert_eq!(rates.len(), 2);
    // Four attacked unit-turns: shots at the garrisoned city hit the city, not the unit
    // (core `combat::city_tile_defender`), so only field units count as attacked.
    for rate in rates {
        assert_eq!((rate.counts.incidents, rate.counts.opportunities), (3, 4));
        assert_eq!(rate.incident_percent, Some(75.0));
    }
    let mut aggregate = legatum_analysis::Aggregate::default();
    aggregate.add(&r);
    aggregate.add(&r);
    assert_eq!(aggregate.games, 2);
    let rate = aggregate
        .rates
        .iter()
        .find(|r| r.behavior == Behavior::UnansweredAttack)
        .unwrap();
    let original = r
        .rates
        .iter()
        .find(|r| r.behavior == Behavior::UnansweredAttack)
        .unwrap();
    assert_eq!(rate.counts.incidents, original.counts.incidents * 2);
    assert_eq!(rate.counts.opportunities, original.counts.opportunities * 2);
    let narrative = r.narrative(2000);
    assert!(narrative.len() <= 2000);
    assert!(narrative.contains("Omitted turn lines:"));
}

#[test]
fn a_garrison_holding_under_bombardment_is_flagged_only_when_it_had_a_better_option() {
    // Garrison-hold rule: a city's garrison stays under bombardment by design. With the
    // Archer across an impassable water gap (no sortie reaches it) and no ranged reply, holding is not
    // an incident; a ranged garrison that could shoot back but held is still flagged.
    let run = |garrison_kind: UnitKind| {
        let mut s = fixture();
        let guard = spawn_unit(&mut s, PlayerId(0), garrison_kind, h(4, 4));
        let archer = spawn_unit(&mut s, PlayerId(1), UnitKind::Archer, h(6, 4));
        for n in h(6, 4).neighbors() {
            s.map.get_mut(n).unwrap().terrain = Terrain::Mountains;
        }
        // Water keeps the attack lane open without giving the garrison a land route.
        s.map.get_mut(h(5, 4)).unwrap().terrain = Terrain::Ocean;
        let shot = Order::RangedAttack {
            unit: archer,
            target: h(4, 4),
        };
        let turns = (0..3)
            .map(|_| {
                OrderMap::from([
                    (PlayerId(1), vec![shot.clone()]),
                    (PlayerId(0), vec![Order::SkipUnit { unit: guard }]),
                ])
            })
            .collect();
        report(&tape(s, turns))
    };
    let held = run(UnitKind::Spearman);
    assert_eq!(count(&held, Behavior::GarrisonUnderRangedFire), 0);
    assert_eq!(count(&held, Behavior::UnansweredAttack), 0);
    let passive_shooter = run(UnitKind::Archer);
    assert!(count(&passive_shooter, Behavior::GarrisonUnderRangedFire) > 0);
    // The city absorbs the shots, so the garrison itself is never the unit attacked.
    assert_eq!(count(&passive_shooter, Behavior::UnansweredAttack), 0);
}
