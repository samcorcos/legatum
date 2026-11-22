//! A Standard two-player Hard run through the CLI: VP games end by a completed hold (or an
//! explicitly checked other ending), never stall, and the hold telemetry is self-consistent.
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

struct TempDir(PathBuf);
impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn run(args: &[&str]) -> (Value, String, TempDir) {
    let dir = std::env::temp_dir().join(format!("legatum-sim-holds-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir(&dir).unwrap();
    let temp = TempDir(dir.clone());
    let output = Command::new(env!("CARGO_BIN_EXE_legatum-sim"))
        .args(args)
        .args(["--label", "test-holds", "--out-dir"])
        .arg(&dir)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let path = fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| p.extension().is_some_and(|e| e == "json"))
        .unwrap();
    let json = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    (
        json,
        fs::read_to_string(path.with_extension("md")).unwrap(),
        temp,
    )
}

#[test]
fn standard_hard_duel_holds_are_consistent_and_never_stall() {
    // 16 games: on the rules-50 tree a Hard duel ends by capitals in about 11 of 12 games
    // (22 of 24 in a 2026-11-22 run), so 8 games held no VP hold win; game 15 of this seed range has one.
    let (report, md, _temp) = run(&[
        "--games",
        "16",
        "--players",
        "2",
        "--difficulty",
        "hard",
        "--threads",
        "2",
    ]);
    let needed = 10; // Standard hold length
    let games = report["games"].as_array().unwrap();
    let mut hold_wins = 0;
    for g in games {
        let outcome = g["outcome"].as_str().unwrap();
        // Explicit stall check: the longer hold games still end well inside the cap, and a
        // stall would never carry a winner or a victory type.
        let cap = report["meta"]["safety_cap"].as_u64().unwrap();
        assert!(g["turns"].as_u64().unwrap() < cap, "game hit the cap: {g}");
        assert_ne!(outcome, "stalled", "{g}");
        assert_ne!(outcome, "panic", "{g}");
        let h = &g["telemetry"]["victory_holds"];
        assert_eq!(h["turns_needed"], needed);
        let n = |k: &str| h[k].as_u64().unwrap();
        let (started, broken, completed, unfinished) =
            (n("started"), n("broken"), n("completed"), n("unfinished"));
        assert!(started >= broken + completed, "{h}");
        assert_eq!(started, broken + completed + unfinished, "{h}");
        assert_eq!(h["holds"].as_array().unwrap().len() as u64, started);
        match outcome {
            "vp" => {
                hold_wins += 1;
                let w = &h["hold_win"];
                let (first, start, win) = (
                    w["first_reached_turn"].as_u64().unwrap(),
                    w["final_hold_start_turn"].as_u64().unwrap(),
                    w["win_turn"].as_u64().unwrap(),
                );
                assert!(first <= start && start < win, "{w}");
                assert_eq!(win - start, needed - 1, "{w}");
                assert_eq!(win, g["turns"].as_u64().unwrap(), "{w}");
                assert_eq!(w["team"], g["winner_team"]);
                assert!(completed >= 1);
                assert_eq!(
                    w["earlier_broken_holds"].as_u64().unwrap() > 0,
                    first < start,
                    "a winner that broke a hold started its final hold later: {w}"
                );
            }
            _ => assert!(h.get("hold_win").is_none(), "{h}"),
        }
    }
    assert!(hold_wins > 0, "no VP hold win in the sample");
    let s = &report["summary"]["victory_holds"];
    assert_eq!(s["hold_wins"].as_u64().unwrap(), hold_wins);
    assert_eq!(s["observed_games"].as_u64().unwrap(), games.len() as u64);
    assert!(s["first_reach_to_win"]["min"].as_u64().unwrap() >= needed - 1);
    assert_eq!(
        report["summary"]["length_by_outcome"]["vp"]["n"]
            .as_u64()
            .unwrap(),
        hold_wins
    );
    let share: f64 = report["summary"]["win_share_by_faction"]
        .as_object()
        .unwrap()
        .values()
        .map(|v| v.as_f64().unwrap())
        .sum();
    assert!((share - 1.0).abs() < 1e-9, "{share}");
    for needle in [
        "## Overview",
        "| Holds started / broken / completed / unfinished |",
        "| Win reason mix |",
        "Winner share by faction",
    ] {
        assert!(md.contains(needle), "{needle}");
    }
}
