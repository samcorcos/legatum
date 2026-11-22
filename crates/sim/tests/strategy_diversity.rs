use legatum_core::{CityId, Event, Faction, PlayerId, ProductionItem, UnitKind};
use legatum_sim::diversity::*;

#[test]
fn classifier_boundaries_precedence_and_diplomacy_hook() {
    let mut e = Evidence::default();
    assert_eq!(classify(&e), Archetype::Generalist);
    e.markets = 2;
    e.cities = 2;
    e.gold = 12;
    assert_eq!(classify(&e), Archetype::Economic);
    e.population = 12;
    e.production = 12;
    assert_eq!(classify(&e), Archetype::Economic);
    e.population = 11;
    assert_eq!(classify(&e), Archetype::Economic);
    e.walls = 2;
    e.great_works = 1;
    assert_eq!(classify(&e), Archetype::Turtle);
    e.great_works = 0;
    e.defense_cost = 10;
    e.development_total_cost = 100;
    assert_eq!(classify(&e), Archetype::Turtle);
    e.defense_cost = 9;
    assert_eq!(classify(&e), Archetype::Economic);
    e.laboratories = 2;
    e.science_cost = 25;
    assert_eq!(classify(&e), Archetype::Science);
    e.science_cost = 24;
    assert_eq!(classify(&e), Archetype::Economic);
    e.early_founded = 4;
    assert_eq!(classify(&e), Archetype::Wide);
    e.early_attacks = 3;
    e.early_military_cost = 120;
    e.early_total_cost = 240;
    assert_eq!(classify(&e), Archetype::Rush);
    e.early_total_cost = 241;
    assert_eq!(classify(&e), Archetype::Wide);
    e.early_total_cost = 240;
    e.early_attacks = 2;
    assert_eq!(classify(&e), Archetype::Wide);
    e.diplomacy = Some(DiplomacyEvidence {
        allied_turns: 20,
        coalitions_led: 1,
    });
    assert_eq!(classify(&e), Archetype::Diplomatic);
    // Capturing four cities is not evidence of a wide settlement strategy.
    assert_eq!(
        classify(&Evidence {
            early_peak_cities: 4,
            ..Default::default()
        }),
        Archetype::Generalist
    );
}

#[test]
fn observations_stop_at_the_horizon_and_do_not_count_late_or_captured_production() {
    let completion = Event::ProductionCompleted {
        city: CityId(1),
        owner: PlayerId(0),
        item: ProductionItem::Unit {
            unit: UnitKind::Soldier,
        },
    };
    let cost = legatum_core::data::unit_stats(UnitKind::Soldier).cost as u64;
    let mut c = Collector::new();
    c.observe_event(60, &completion);
    c.observe_event(61, &completion);
    c.observe_event(120, &completion);
    c.observe_event(121, &completion);
    c.observe_event(
        20,
        &Event::CityCaptured {
            city: CityId(1),
            from: PlayerId(1),
            to: PlayerId(0),
            bounty: None,
        },
    );
    c.observe_event(
        20,
        &Event::ItemBought {
            city: CityId(1),
            owner: PlayerId(0),
            item: ProductionItem::Unit {
                unit: UnitKind::Soldier,
            },
            gold: 100,
        },
    );
    let e = &c.telemetry.empires[&0];
    assert_eq!(e.early_military_cost, 3 * cost);
    assert_eq!(e.early_total_cost, 3 * cost);
    assert_eq!(e.development_total_cost, 4 * cost);
    assert_eq!(e.early_founded, 0);
    assert!(e.diplomacy.is_none());
    let mut standard = Collector::new();
    standard.observe_event(120, &completion);
    assert_eq!(standard.telemetry.empires[&0].early_military_cost, cost);
    assert_eq!(standard.telemetry.development_end_turn, 240);
}

#[test]
fn observations_freeze_at_horizon_or_elimination() {
    let mut state = legatum_core::new_game(&legatum_core::GameSetup::all_ai(1));
    let mut c = Collector::new();
    c.observe_resolved_turn(70, &state);
    state.player_mut(PlayerId(0)).alive = false;
    c.observe_resolved_turn(240, &state);
    c.observe_resolved_turn(241, &state);
    assert_eq!(c.telemetry.empires[&0].development_turn, 70);
    assert_eq!(c.telemetry.empires[&1].development_turn, 240);
    assert_eq!(c.telemetry.empires.len(), 8);
}

fn player(team: u8, a: Archetype, vp: u32, won: bool) -> EmpireResult {
    EmpireResult {
        team,
        faction: Faction::India,
        archetype: Some(a),
        won,
        alive: true,
        vp,
        eliminated_turn: None,
    }
}

#[test]
fn summary_uses_equal_game_weight_reports_ties_and_excludes_censored_samples() {
    use Archetype::*;
    let mut s = DiversitySummary::new(vec![Rush, Wide, Economic]);
    let mut rows = vec![
        player(0, Rush, 10, true),
        player(1, Wide, 10, false),
        player(2, Wide, 3, false),
    ];
    s.observe(true, &rows); // Rush is the actual winner even with equal VP.
    rows = vec![player(0, Rush, 4, false), player(1, Wide, 4, false)];
    s.observe(true, &rows); // draw, all tied
    s.observe(false, &rows); // cap stop, not losses
    rows[0].archetype = None;
    s.observe(true, &rows); // legacy data, not Generalist
    s.finish();
    assert_eq!((s.games, s.seats, s.winners), (2, 5, 1));
    assert_eq!(
        (s.excluded_unfinished_games, s.excluded_missing_labels),
        (1, 1)
    );
    let m = &s.matchups[&Rush][&Wide];
    assert_eq!(
        (m.games, m.player_pairs, m.ahead_pairs, m.tied_pairs),
        (2, 3, 2, 1)
    );
    assert_eq!(m.ahead_rate, Some(0.5)); // not 2/3: every game has equal weight
    assert_eq!(m.tie_rate, Some(0.5));
    assert_eq!(m.score, Some(0.75));
    assert_eq!(s.matchups[&Wide][&Rush].score, Some(0.25));
    assert_eq!(s.matchups[&Economic][&Rush].ahead_rate, None);
    assert_eq!(s.by_archetype[&Rush].win_rate, Some(0.5));
    assert_eq!(s.by_archetype[&Wide].player_frequency, Some(0.6));
    assert_eq!(s.by_archetype[&Rush].winner_frequency, Some(1.0));
    assert_eq!(s.by_faction[&Faction::India][&Rush].wins, 1);
    assert_eq!(s.winner_entropy, Some(0.0));
    let json = serde_json::to_string(&s).unwrap();
    assert_eq!(serde_json::from_str::<DiversitySummary>(&json).unwrap(), s);
    let check = check_diversity(
        &s,
        GuardrailConfig {
            min_entropy: 0.8,
            min_matchup_games: 2,
        },
    );
    assert_eq!(check.status, CheckStatus::Fail);
    assert!(check.dominant.is_empty()); // missing Economic comparisons cannot be assumed wins
    assert_eq!(check.unobserved, vec![Economic]);
    assert_eq!(check.never_won, vec![Wide]);
    let md = render_markdown(&s, &check, "test");
    assert!(md.contains("50.0% / 50.0% (2)"));
    assert!(md.contains("| India | 1 / 2 | 0 / 3 | 0 / 0 |"));
}

#[test]
fn ranking_respects_survival_elimination_and_teammates() {
    use Archetype::*;
    let mut a = player(0, Rush, 100, false);
    let mut b = player(1, Wide, 1, false);
    a.alive = false;
    a.eliminated_turn = Some(50);
    let mut s = DiversitySummary::new(vec![Rush, Wide]);
    s.observe(true, &[a.clone(), b.clone()]);
    b.alive = false;
    b.eliminated_turn = Some(49);
    s.observe(true, &[a.clone(), b.clone()]);
    b.eliminated_turn = Some(50);
    s.observe(true, &[a.clone(), b.clone()]);
    b.team = a.team;
    s.observe(true, &[a, b]); // same-team pairs never count
    let m = &s.matchups[&Rush][&Wide];
    assert_eq!(
        (m.games, m.ahead_pairs, m.tied_pairs, m.behind_pairs),
        (3, 1, 1, 1)
    );
}

#[test]
fn entropy_keeps_zero_win_categories_and_handles_empty_data() {
    assert_eq!(normalized_entropy(&[]), None);
    assert_eq!(normalized_entropy(&[0, 0]), None);
    assert_eq!(normalized_entropy(&[7]), Some(0.0));
    assert_eq!(normalized_entropy(&[7, 0, 0]), Some(0.0));
    assert!((normalized_entropy(&[7, 7, 7]).unwrap() - 1.0).abs() < 1e-12);
    assert!((normalized_entropy(&[7, 7, 0, 0]).unwrap() - 0.5).abs() < 1e-12);
}

#[test]
fn guardrail_detects_dominance_even_with_high_entropy_and_passes_a_cycle() {
    use Archetype::*;
    let config = GuardrailConfig {
        min_entropy: 0.8,
        min_matchup_games: 2,
    };
    let make = |cycle: bool| {
        let mut s = DiversitySummary::new(vec![Rush, Wide, Economic]);
        for (a, b) in [(Rush, Wide), (Wide, Economic), (Economic, Rush)] {
            for _ in 0..2 {
                s.observe(true, &[player(0, a, 10, true), player(1, b, 0, false)]);
            }
        }
        if !cycle {
            // Add nonwinning seat comparisons. Keep an even winner distribution while
            // giving Rush an overall matchup advantage against Economic.
            for _ in 0..3 {
                s.observe(
                    true,
                    &[player(0, Rush, 9, false), player(1, Economic, 0, false)],
                );
            }
        }
        s.finish();
        s
    };
    let cycle = make(true);
    let c = check_diversity(&cycle, config);
    assert_eq!(c.status, CheckStatus::Pass);
    assert!(c.dominant.is_empty());
    assert!(c.without_observed_counter.is_empty());
    let dominant = make(false);
    assert!((dominant.winner_entropy.unwrap() - 1.0).abs() < 1e-12);
    let c = check_diversity(&dominant, config);
    assert_eq!(c.status, CheckStatus::Fail);
    assert_eq!(c.dominant, vec![Rush]);
    let c = check_diversity(
        &cycle,
        GuardrailConfig {
            min_matchup_games: 3,
            ..config
        },
    );
    assert_eq!(c.status, CheckStatus::InsufficientData);
    assert_eq!(c.missing_matchups.len(), 3);
    let empty = DiversitySummary::new(vec![Rush, Wide]);
    assert_eq!(
        check_diversity(&empty, config).status,
        CheckStatus::InsufficientData
    );
    assert_eq!(
        check_diversity(
            &cycle,
            GuardrailConfig {
                min_entropy: f64::NAN,
                ..config
            }
        )
        .status,
        CheckStatus::Fail
    );
}

#[test]
fn generalist_is_checked_for_dominance_and_can_counter_a_named_strategy() {
    use Archetype::*;
    let mut s = DiversitySummary::new(vec![Rush, Wide, Generalist]);
    for (a, b) in [(Rush, Wide), (Generalist, Rush), (Generalist, Wide)] {
        s.observe(true, &[player(0, a, 3, false), player(1, b, 1, false)]);
    }
    s.finish();
    let check = check_diversity(
        &s,
        GuardrailConfig {
            min_entropy: 0.0,
            min_matchup_games: 1,
        },
    );
    assert_eq!(check.dominant, vec![Generalist]);
    assert_eq!(check.without_observed_counter, vec![Generalist]);
    assert_eq!(check.status, CheckStatus::Fail);
}
