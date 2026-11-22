use super::*;
use crate::fixture::{ME, ViewBuilder, off};
use legatum_core::view::TeamStanding;
use legatum_core::{AiDifficulty, Faction};

fn view(vp: u32) -> PlayerView {
    let mut v = ViewBuilder::new(18, 12).city(1, ME, off(4, 4), 8).build();
    v.turn = legatum_core::data::EXPECTED_GAME_TURNS;
    v.turn_limit = None;
    v.standings = vec![TeamStanding {
        team: v.team,
        members: vec![ME],
        vp,
        target: 10,
        victory_hold: None,
    }]
    .into();
    v
}

fn brain(opening: Opening, difficulty: AiDifficulty) -> OpeningBrain {
    OpeningBrain::new(
        ScriptedBrain::with_difficulty(Faction::Switzerland, 7, ME, difficulty),
        OpeningPolicy::builtin(opening),
    )
}

#[test]
fn closing_preserves_development_then_reuses_the_shared_planner() {
    for opening in Opening::ALL.into_iter().filter(|o| *o != Opening::Default) {
        for difficulty in [AiDifficulty::Easy, AiDifficulty::Normal, AiDifficulty::Hard] {
            let mut b = brain(opening, difficulty);
            let mut v = view(8);
            v.turn -= 1;
            b.plan(&v);
            assert_ne!(b.phase, Phase::Close, "{opening:?}");
            v.turn += 1;
            let expected = b.base.plan(&v);
            assert_eq!(b.plan(&v), expected, "{opening:?}");
            assert_eq!(b.phase, Phase::Close);
            assert_eq!(b.plan(&v), expected, "repeated planning is stable");
            assert_eq!(b.switches.last().unwrap().turn, v.turn);
            let switches = b.switches.len();
            v.turn += 1;
            v.standings[0].vp = 5;
            assert_eq!(b.plan(&v), b.base.plan(&v));
            assert_eq!(
                b.phase,
                Phase::Close,
                "lost titles cannot restart an opening"
            );
            assert_eq!(b.switches.len(), switches);
        }
    }
}

#[test]
fn closing_stall_window_counts_new_best_team_vp_not_title_churn() {
    let mut b = brain(Opening::Economic, AiDifficulty::Hard);
    let mut v = view(5);
    let start = v.turn;
    let window = 120_u32;
    b.plan(&v);
    assert_ne!(
        b.phase,
        Phase::Close,
        "a late first view is not evidence of a stall"
    );
    for (elapsed, vp) in [(1, 4), (window - 1, 5), (window, 5)] {
        v.turn = start + elapsed;
        v.standings[0].vp = vp;
        b.plan(&v);
        assert_eq!(b.phase == Phase::Close, elapsed == window);
    }
}

#[test]
fn closing_stall_window_restarts_when_the_team_makes_progress() {
    let mut b = brain(Opening::Economic, AiDifficulty::Hard);
    let mut v = view(5);
    b.plan(&v);
    v.turn += 119;
    v.standings[0].vp = 6;
    b.plan(&v);
    v.turn += 119;
    b.plan(&v);
    assert_ne!(b.phase, Phase::Close);
    v.turn += 1;
    b.plan(&v);
    assert_eq!(b.phase, Phase::Close);
}

#[test]
fn default_never_enters_closing_even_after_a_stalled_endgame() {
    for primitives in [false, true] {
        let mut b = brain(Opening::Default, AiDifficulty::Hard);
        b.primitives_enabled = primitives;
        let mut v = view(9);
        for turn in [1, 240, 300, 600, 1200] {
            v.turn = turn;
            let orders = b.plan(&v);
            if !primitives {
                assert_eq!(orders, b.base.plan(&v));
            }
            assert_ne!(b.phase, Phase::Close);
            assert!(b.vp_progress.is_none());
        }
    }
}
