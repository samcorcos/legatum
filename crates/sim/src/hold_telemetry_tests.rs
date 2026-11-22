use super::*;
use crate::stats::{GameRecord, Outcome, RunMeta, SourceRevision, summarize};
use crate::telemetry::{Collector, GameTelemetry};
use legatum_core::{GameSetup, TeamId};

fn started(team: u8) -> Event {
    Event::VictoryHoldStarted {
        team: TeamId(team),
        vp: 10,
        target: 10,
        turns_needed: 10,
    }
}

fn broken(team: u8, turns_held: u32) -> Event {
    Event::VictoryHoldBroken {
        team: TeamId(team),
        turns_held,
        vp: 9,
        target: 10,
    }
}

fn won(team: u8) -> Event {
    Event::GameWon {
        team: TeamId(team),
        reason: "held".into(),
    }
}

/// A Standard game state whose running holds are `holds` (team, turns held).
fn standard_state(holds: &[(u8, u32)]) -> GameState {
    let mut state = legatum_core::new_game(&GameSetup::all_ai(1));
    for (team, held) in holds {
        state.vp_holds.insert(TeamId(*team), *held);
    }
    state
}

#[test]
fn started_broken_restarted_and_won_are_counted_with_turns() {
    let mut o = Observer::default();
    // Team 0 reaches on turn 10 and loses it on 13 (held 4); team 1 reaches on 15 and never
    // finishes; team 0 reaches again on 20 and wins on turn 29 (10 turn ends held).
    o.observe_event(10, &started(0));
    o.observe_event(13, &broken(0, 4));
    o.observe_event(15, &started(1));
    o.observe_event(20, &started(0));
    o.observe_event(29, &won(0));
    o.observe_resolved_turn(29);
    let t = o.finish(&standard_state(&[(0, 10), (1, 3)]));
    assert_eq!(t.turns_needed, 10);
    assert_eq!(
        (t.started, t.broken, t.completed, t.unfinished),
        (3, 1, 1, 1)
    );
    assert_eq!(t.started, t.broken + t.completed + t.unfinished);
    let spans: Vec<_> = t
        .holds
        .iter()
        .map(|h| (h.team, h.start_turn, h.end_turn, h.turns_held, h.result))
        .collect();
    assert_eq!(
        spans,
        vec![
            (0, 10, 13, 4, HoldResult::Broken),
            (1, 15, 29, 3, HoldResult::Unfinished),
            (0, 20, 29, 10, HoldResult::Completed),
        ]
    );
}

#[test]
fn hold_win_records_first_reach_final_hold_and_earlier_breaks() {
    let mut o = Observer::default();
    o.observe_event(10, &started(0));
    o.observe_event(13, &broken(0, 4));
    o.observe_event(20, &started(0));
    o.observe_event(29, &won(0));
    o.observe_resolved_turn(29);
    let t = o.finish(&standard_state(&[(0, 10)]));
    assert_eq!(
        t.hold_win,
        Some(HoldWin {
            team: 0,
            first_reached_turn: 10,
            final_hold_start_turn: 20,
            win_turn: 29,
            earlier_broken_holds: 1,
            first_to_reach: true,
        })
    );
    let w = t.hold_win.unwrap();
    assert!(w.first_reached_turn <= w.final_hold_start_turn);
    assert_eq!(w.win_turn - w.final_hold_start_turn, t.turns_needed - 1);
}

#[test]
fn a_win_by_the_second_team_to_reach_is_flagged() {
    let mut o = Observer::default();
    o.observe_event(8, &started(1));
    o.observe_event(12, &broken(1, 5));
    o.observe_event(9, &started(0));
    o.observe_event(18, &won(0));
    o.observe_resolved_turn(18);
    let t = o.finish(&standard_state(&[(0, 10)]));
    let w = t.hold_win.unwrap();
    assert_eq!((w.team, w.first_reached_turn), (0, 9));
    assert!(!w.first_to_reach);
    assert_eq!(w.earlier_broken_holds, 0);
}

#[test]
fn games_without_a_hold_win_have_no_hold_win_record() {
    // Conquest while a hold is still running: unfinished, not completed.
    let mut o = Observer::default();
    o.observe_event(30, &started(2));
    o.observe_event(33, &won(1));
    o.observe_resolved_turn(33);
    let t = o.finish(&standard_state(&[(2, 4)]));
    assert_eq!(t.hold_win, None);
    assert_eq!((t.started, t.completed, t.unfinished), (1, 0, 1));
    // A game that never reached a target has an empty record.
    let t = Observer::default().finish(&standard_state(&[]));
    assert_eq!((t.started, t.holds.len(), t.hold_win), (0, 0, None));
}

fn telemetry(holds: Option<VictoryHoldTelemetry>) -> GameTelemetry {
    let state = legatum_core::new_game(&GameSetup::all_ai(1));
    let mut t = Collector::default().finish(&state, 0);
    t.victory_holds = holds;
    t
}

fn record(index: usize, outcome: Outcome, turns: u32, holds: VictoryHoldTelemetry) -> GameRecord {
    let mut r = GameRecord::panicked(index, index as u64, String::new(), 0);
    r.outcome = outcome;
    r.panic = None;
    r.turns = turns;
    r.telemetry = Some(telemetry(Some(holds)));
    r
}

fn hold_record(
    started: u32,
    broken: u32,
    completed: u32,
    unfinished: u32,
    hold_win: Option<HoldWin>,
) -> VictoryHoldTelemetry {
    VictoryHoldTelemetry {
        turns_needed: 10,
        holds: vec![],
        started,
        broken,
        completed,
        unfinished,
        hold_win,
    }
}

fn hold_win(first: u32, final_start: u32, win: u32, earlier: u32, first_to_reach: bool) -> HoldWin {
    HoldWin {
        team: 0,
        first_reached_turn: first,
        final_hold_start_turn: final_start,
        win_turn: win,
        earlier_broken_holds: earlier,
        first_to_reach,
    }
}

#[test]
fn run_summary_aggregates_holds_reach_to_win_and_comebacks() {
    let mut records = vec![
        record(
            0,
            Outcome::Vp,
            120,
            hold_record(1, 0, 1, 0, Some(hold_win(111, 111, 120, 0, true))),
        ),
        record(
            1,
            Outcome::Vp,
            200,
            hold_record(3, 2, 1, 0, Some(hold_win(150, 191, 200, 2, false))),
        ),
        record(2, Outcome::Conquest, 90, hold_record(1, 0, 0, 1, None)),
        record(3, Outcome::Stalled, 1200, hold_record(2, 2, 0, 0, None)),
    ];
    // A record from before holds existed and a panic are excluded, never counted as zero.
    let mut legacy = record(4, Outcome::Draw, 100, hold_record(0, 0, 0, 0, None));
    legacy.telemetry = Some(telemetry(None));
    records.push(legacy);
    records.push(GameRecord::panicked(5, 5, "boom".into(), 0));
    let s = summarize(&records);
    let h = &s.victory_holds;
    assert_eq!((h.observed_games, h.excluded_games_without_holds), (4, 2));
    assert_eq!(h.turns_needed, Some(10));
    assert_eq!(h.games_with_hold, 4);
    assert_eq!(
        (h.started, h.broken, h.completed, h.unfinished),
        (7, 4, 2, 1)
    );
    assert!((h.break_rate - 4.0 / 7.0).abs() < 1e-9);
    assert!((h.break_rate_resolved - 4.0 / 6.0).abs() < 1e-9);
    assert_eq!(
        (h.hold_wins, h.comebacks, h.wins_not_first_to_reach),
        (2, 1, 1)
    );
    let reach = h.first_reach_to_win.as_ref().unwrap();
    assert_eq!((reach.n, reach.min, reach.max), (2, 9, 50));
    assert!((reach.mean - 29.5).abs() < 1e-9);
    let last = h.final_hold_to_win.as_ref().unwrap();
    assert_eq!((last.min, last.max), (9, 9));
    assert!(h.first_reach_turn.as_ref().unwrap().max <= h.win_turn.as_ref().unwrap().max);
    // Game length per outcome kind: panics are left out, stalls are kept apart from wins.
    let by = &s.length_by_outcome;
    assert_eq!(by[&Outcome::Vp].n, 2);
    assert!((by[&Outcome::Vp].mean - 160.0).abs() < 1e-9);
    assert_eq!(by[&Outcome::Stalled].max, 1200);
    assert!(!by.contains_key(&Outcome::Panic));
}

fn meta() -> RunMeta {
    RunMeta {
        label: "t".into(),
        date: "2026-10-06".into(),
        started_at: "2026-10-06T00:00:00Z".into(),
        games: 1,
        first_seed: "0".into(),
        threads: 1,
        brain: "mixed".into(),
        strategies: vec![],
        strategy_matchups: false,
        difficulty_rotation: None,
        proxy: None,
        proxy_version: None,
        safety_cap: 600,
        turn_limit: None,
        determinism: false,
        wall_ms: 1000,
        source: SourceRevision::default(),
        command: vec![],
        first_setup: GameSetup::all_ai(0),
    }
}

#[test]
fn overview_markdown_prints_the_compact_table() {
    let records = vec![record(
        0,
        Outcome::Vp,
        120,
        hold_record(2, 1, 1, 0, Some(hold_win(100, 111, 120, 1, true))),
    )];
    let md = crate::stats::render_markdown(&meta(), &summarize(&records), &records);
    for needle in [
        "## Overview",
        "| Holds started / broken / completed / unfinished | 2 / 1 / 1 / 0 |",
        "| Hold break rate | 50.0% of started; 50.0% of resolved",
        "| First reach to win, turns (mean / median / min-max) | 20.0 / 20 / 20-20 |",
        "| Comebacks (winner broke an earlier hold of its own) | 1 |",
        "| VP hold | 1 | 100.0% |",
        "Winner share by faction",
    ] {
        assert!(md.contains(needle), "missing {needle:?} in\n{md}");
    }
}

#[test]
fn serialization_round_trips_and_old_reports_still_load() {
    let holds = hold_record(1, 0, 1, 0, Some(hold_win(5, 5, 14, 0, true)));
    let t = telemetry(Some(holds.clone()));
    let mut json = serde_json::to_value(&t).unwrap();
    assert_eq!(json["victory_holds"]["hold_win"]["win_turn"], 14);
    assert_eq!(json["victory_holds"]["completed"], 1);
    let back: GameTelemetry = serde_json::from_value(json.clone()).unwrap();
    assert_eq!(back.victory_holds, Some(holds));
    // A report from before holds carries no field: loads as None, never as zero holds.
    json.as_object_mut().unwrap().remove("victory_holds");
    let old: GameTelemetry = serde_json::from_value(json).unwrap();
    assert!(old.victory_holds.is_none());
    // Without a win the hold_win key is omitted.
    let none = serde_json::to_value(hold_record(0, 0, 0, 0, None)).unwrap();
    assert!(none.get("hold_win").is_none());
    let none: VictoryHoldTelemetry = serde_json::from_value(none).unwrap();
    assert_eq!(none.hold_win, None);
}
