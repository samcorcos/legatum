//! Public-report wiring, plus unchanged order tapes pinning instrumentation neutrality.
use legatum_core::{Event, ProductionItem, UnitKind, new_game, resolve_turn, scoring};
use legatum_wasm::session::{BrainKind, Session};
use legatum_wasm::tape::Tape;
use serde_json::{Value, json};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

struct TempDir(PathBuf);
impl TempDir {
    fn new(name: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("legatum-sim-metrics-{}-{name}", std::process::id()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn report(temp: &TempDir, args: &[&str]) -> (Value, String) {
    let reports = temp.0.join("reports");
    let output = Command::new(env!("CARGO_BIN_EXE_legatum-sim"))
        .args(args)
        .args(["--label", "test-sim-metrics", "--out-dir"])
        .arg(&reports)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let path = fs::read_dir(reports)
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| p.extension().is_some_and(|e| e == "json"))
        .unwrap();
    let json = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    let markdown = fs::read_to_string(path.with_extension("md")).unwrap();
    (json, markdown)
}

#[test]
fn instrumented_metrics_match_replayed_tapes() {
    let temp = TempDir::new("tapes");
    let tapes = temp.0.join("tapes");
    let (report, md) = report(
        &temp,
        &[
            "--games",
            "3",
            "--seed",
            "18446744073709551613",
            "--threads",
            "2",
            "--brain",
            "mixed",
            "--turn-limit",
            "40",
            "--determinism",
            "--tape-out",
            tapes.to_str().unwrap(),
        ],
    );
    let mut totals = [0u64; 4];
    let mut opening_turns: [Vec<u32>; 4] = Default::default();
    for (index, seed) in (u64::MAX - 2..=u64::MAX).enumerate() {
        let filename = format!("game-{index:05}-seed-{seed}.json");
        // Validate the report against the tapes this run generated. (The pinned determinism
        // fixtures are AI-independent replays; comparing regenerated games to them would break
        // on every AI change.)
        let generated: Value =
            serde_json::from_str(&fs::read_to_string(tapes.join(filename)).unwrap()).unwrap();
        let tape = Tape::from_json(&generated.to_string()).unwrap();
        let mut state = new_game(&tape.setup);
        let mut counts = [0u64; 4];
        let mut eliminations = Vec::new();
        let mut production_completions = Vec::new();
        let mut city_foundings = Vec::new();
        let mut first_sources =
            std::collections::BTreeMap::<(u8, legatum_core::Resource), u32>::new();
        let mut research_turns = std::collections::BTreeMap::new();
        let mut peak_cities = std::collections::BTreeMap::<u8, usize>::new();
        for turn in &tape.turns {
            let (next, events) = resolve_turn(&state, &turn.orders);
            for (index, event) in events.iter().enumerate() {
                match event {
                    Event::TechResearched { player, tech } => {
                        research_turns
                            .entry(player.0)
                            .or_insert_with(std::collections::BTreeMap::new)
                            .entry(format!("{tech:?}"))
                            .or_insert(turn.turn);
                    }
                    Event::Battle { .. } => counts[0] += 1,
                    Event::CityCaptured { .. } => counts[1] += 1,
                    Event::SettlerCaptured { .. } => counts[2] += 1,
                    Event::ProductionCompleted { owner, item, .. } => {
                        production_completions.push((*owner, *item, turn.turn));
                    }
                    Event::CityFounded { owner, .. } => city_foundings.push((*owner, turn.turn)),
                    Event::ResourcesRevealed {
                        player,
                        resource,
                        city: Some(_),
                        ..
                    } => {
                        first_sources
                            .entry((player.0, *resource))
                            .or_insert(turn.turn);
                    }
                    Event::PlayerEliminated { player } => {
                        counts[3] += 1;
                        // Matches runner.rs's observe_events: the elimination's `by` names
                        // whoever took or razed the player's last city, found by scanning
                        // this turn's earlier events backwards.
                        let by = events[..index].iter().rev().find_map(|event| match event {
                            Event::CityCaptured { from, to, .. } if from == player => Some(to.0),
                            Event::CityRazed { from, by, .. } if from == player => Some(by.0),
                            _ => None,
                        });
                        let mut entry = json!({"player": player.0, "turn": turn.turn});
                        if let Some(by) = by {
                            entry["by"] = json!(by);
                        }
                        eliminations.push(entry);
                    }
                    _ => {}
                }
            }
            for player in &next.players {
                let owned = next.cities_of(player.id).count();
                let peak = peak_cities.entry(player.id.0).or_default();
                *peak = (*peak).max(owned);
            }
            assert_eq!(next.hash().to_string(), turn.hash);
            state = next;
        }
        let game = &report["games"][index];
        assert_eq!(game["telemetry"]["research_turns"], json!(research_turns));
        assert_eq!(game["telemetry"]["peak_cities"], json!(peak_cities));
        assert_eq!(game["telemetry"]["battles"], counts[0]);
        assert_eq!(game["city_captures"], counts[1]);
        assert_eq!(game["telemetry"]["settler_captures"], counts[2]);
        assert_eq!(game["eliminations"], json!(eliminations));
        assert_eq!(game["telemetry"]["elimination_count"], counts[3]);
        assert!(game["telemetry"].get("city_captures").is_none());
        assert!(game["telemetry"]["turn_50"].is_null());
        assert_eq!(game["determinism"]["ok"], true);
        assert_eq!(
            game["telemetry"]["openings"].as_object().unwrap().len(),
            state.players.len()
        );
        for p in &state.players {
            let first_produced = |kind| {
                production_completions
                    .iter()
                    .find(|(owner, item, _)| {
                        *owner == p.id && *item == ProductionItem::Unit { unit: kind }
                    })
                    .map(|(_, _, turn)| *turn)
            };
            let first_soldier_turn = first_produced(UnitKind::Soldier);
            let first_settler_turn = first_produced(UnitKind::Settler);
            let first_city_turn = Some(1);
            let second_city_turn = city_foundings
                .iter()
                .find(|(owner, _)| *owner == p.id)
                .map(|(_, turn)| *turn);
            let mut expected = json!({"first_soldier_turn": first_soldier_turn,
                "first_settler_turn": first_settler_turn, "first_city_turn": first_city_turn,
                "second_city_turn": second_city_turn});
            for (resource, key) in [
                (legatum_core::Resource::Iron, "first_iron_source_turn"),
                (legatum_core::Resource::Coal, "first_coal_source_turn"),
            ] {
                if let Some(turn) = first_sources.get(&(p.id.0, resource)) {
                    expected[key] = json!(turn);
                }
            }
            // The reveal and use turns of each strategic resource are checked by the resource tests.
            let mut actual = game["telemetry"]["openings"][p.id.0.to_string()].clone();
            if let Some(map) = actual.as_object_mut() {
                map.remove("first_reveal_turn");
                map.remove("first_use_turn");
            }
            assert_eq!(actual, expected);
            for (samples, turn) in opening_turns.iter_mut().zip([
                first_soldier_turn,
                first_settler_turn,
                first_city_turn,
                second_city_turn,
            ]) {
                samples.extend(turn);
            }
            assert_eq!(
                game["players"][p.id.0 as usize]["vp"],
                scoring::vp_total(&state, p.id)
            );
            assert_eq!(
                game["telemetry"]["final_scores"]["players"][p.id.0.to_string()]["total"],
                scoring::vp_total(&state, p.id)
            );
        }
        for (total, count) in totals.iter_mut().zip(counts) {
            *total += count;
        }
    }
    let summary = &report["summary"]["telemetry"];
    for (metric, total) in [
        "battles",
        "city_captures",
        "settler_captures",
        "eliminations",
    ]
    .into_iter()
    .zip(totals)
    {
        assert_eq!(summary["events"][metric]["total"], total);
        assert_eq!(summary["events"][metric]["observed_games"], 3);
    }
    assert_eq!(summary["turn_50"]["ended_before_turn_50"], 3);
    assert_eq!(summary["turn_50"]["any_leader"]["eligible_games"], 0);
    let openings = &report["summary"]["openings"];
    assert_eq!(openings["observed_games"], 3);
    assert_eq!(openings["observed_empires"], 24);
    assert_eq!(openings["excluded_games_without_openings"], 0);
    for ((metric, label), mut samples) in [
        ("first_soldier", "First Soldier completed"),
        ("first_settler", "First Settler completed"),
        ("first_city", "First city founded"),
        ("second_city", "Second city founded"),
    ]
    .into_iter()
    .zip(opening_turns)
    {
        assert!(!samples.is_empty(), "exercise observed {metric} turns");
        samples.sort_unstable();
        let percentile = |p| samples[(p * (samples.len() - 1) + 50) / 100];
        assert_eq!(openings[metric]["reached_empires"], samples.len());
        assert_eq!(openings[metric]["never_empires"], 24 - samples.len());
        assert_eq!(
            openings[metric]["turns"],
            json!({"min": samples.first(), "p10": percentile(10),
                "median": percentile(50), "p90": percentile(90), "max": samples.last()})
        );
        assert!(md.contains(&format!(
            "| {label} (turn, per empire) | {} | {} | {} | {} | {} |",
            samples[0],
            percentile(10),
            percentile(50),
            percentile(90),
            samples.last().unwrap()
        )));
        assert!(md.contains(&format!(
            "| {label} | {} | {} |",
            samples.len(),
            24 - samples.len()
        )));
    }
    assert_eq!(
        report["meta"]["first_setup"]["seed"],
        "18446744073709551613"
    );
    assert!(md.contains("## Per-game events"));
    assert!(md.contains("3 games ended before turn 50"));
}

#[test]
fn openings_exclude_starting_units_and_keep_unreached_empires_at_the_safety_cap() {
    let temp = TempDir::new("openings-cap");
    let (report, md) = report(
        &temp,
        &["--games", "1", "--seed", "700001", "--safety-cap", "1"],
    );
    assert_eq!(report["games"][0]["outcome"], "stalled");
    let empires = report["games"][0]["telemetry"]["openings"]
        .as_object()
        .unwrap();
    assert_eq!(empires.len(), 8);
    for opening in empires.values() {
        assert_eq!(
            *opening,
            json!({
                "first_soldier_turn": null, "first_settler_turn": null,
                "first_city_turn": 1, "second_city_turn": null
            })
        );
    }
    let openings = &report["summary"]["openings"];
    assert_eq!(openings["observed_games"], 1);
    assert_eq!(openings["observed_empires"], 8);
    assert_eq!(openings["first_city"]["reached_empires"], 8);
    assert_eq!(openings["first_city"]["turns"]["median"], 1);
    for metric in ["first_soldier", "first_settler", "second_city"] {
        assert_eq!(
            openings[metric],
            json!({
                "reached_empires": 0, "never_empires": 8, "turns": null
            })
        );
    }
    assert!(md.contains("| First Soldier completed (turn, per empire) | - | - | - | - | - |"));
    assert!(md.contains("| Second city founded | 0 | 8 |"));
    assert!(
        report["metric_definitions"]["openings"]
            .as_str()
            .unwrap()
            .contains("Starting units")
    );
}

#[test]
fn normal_run_reports_turn_50_and_preserves_turn_75_replay_data() {
    let temp = TempDir::new("normal");
    let (report, _) = report(
        &temp,
        &[
            "--games",
            "1",
            "--seed",
            "20001",
            "--turn-limit",
            "75",
            "--threads",
            "1",
        ],
    );
    let game = &report["games"][0];
    assert!(game.get("hashes").is_none()); // Telemetry works without tape or determinism flags.
    assert_eq!(game["turns"], 75);
    let setup =
        legatum_wasm::json::setup_from_value(report["meta"]["first_setup"].clone()).unwrap();
    assert_eq!(setup.seed, 20001);
    assert_eq!(setup.turn_limit, Some(75));
    assert_eq!(
        setup
            .players
            .iter()
            .map(|p| p.faction)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        8
    );
    let mut session = Session::new(&setup, |_| BrainKind::Scripted);
    for turn in 1..=75 {
        let orders = session.ai_orders();
        session.step(&orders);
        if turn == 50 || turn == 75 {
            let expected: std::collections::BTreeMap<_, _> = session
                .state
                .teams()
                .into_iter()
                .map(|t| (t.0, scoring::team_vp(&session.state, t)))
                .collect();
            let actual = if turn == 50 {
                &game["telemetry"]["turn_50"]["team_vp"]
            } else {
                &game["team_vp_turn_75"]
            };
            assert_eq!(*actual, json!(expected));
        }
    }
    let snapshot = &game["telemetry"]["turn_50"];
    assert_eq!(snapshot["resolved_turn"], 50);
    assert_eq!(snapshot["next_turn"], 51);
    assert_eq!(
        report["summary"]["telemetry"]["turn_50"]["any_leader"]["eligible_games"],
        1
    );
    assert!(
        report["metric_definitions"]["turn_50"]
            .as_str()
            .unwrap()
            .contains("GameState.turn is 51")
    );
}

#[test]
fn empty_cli_report_has_no_fake_rates_or_observations() {
    let temp = TempDir::new("empty");
    let (report, md) = report(&temp, &["--games", "0", "--threads", "1"]);
    assert_eq!(report["games"], json!([]));
    assert_eq!(report["summary"]["telemetry"]["sample_games"], 0);
    assert_eq!(report["summary"]["openings"]["observed_empires"], 0);
    for metric in ["first_soldier", "first_settler", "second_city"] {
        assert_eq!(
            report["summary"]["openings"][metric],
            json!({
                "reached_empires": 0, "never_empires": 0, "turns": null
            })
        );
    }
    assert_eq!(
        report["summary"]["telemetry"]["events"]["battles"]["observed_games"],
        0
    );
    assert!(md.contains("0/0 (n/a)"));
    assert!(!md.contains("NaN"));
    // The curated public export has no .git directory, so provenance is unknown there.
    // A checkout must still report its actual commit instead of silently omitting it.
    let expected_commit = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8(output.stdout).unwrap().trim().to_owned());
    assert_eq!(
        report["meta"]["source"]["commit"].as_str(),
        expected_commit.as_deref()
    );
}

#[test]
fn difficulty_rotation_reaches_brains_tapes_and_report_denominators() {
    let mut totals = std::collections::BTreeMap::<String, (u64, u64)>::new();
    let mut winners = 0;
    for offset in 0..3 {
        let temp = TempDir::new(&format!("difficulty-{offset}"));
        let tapes = temp.0.join("tapes");
        let (data, md) = report(
            &temp,
            &[
                "--games",
                "2",
                "--seed",
                "930001",
                "--difficulty-rotation",
                &offset.to_string(),
                "--determinism",
                // Enough time for a VP leader even after the founded-capital opening
                // shifted the early races on these fixed seeds.
                "--turn-limit",
                "120",
                "--threads",
                "1",
                "--tape-out",
                tapes.to_str().unwrap(),
            ],
        );
        assert_eq!(data["meta"]["difficulty_rotation"], offset);
        assert_eq!(data["summary"]["determinism_checked"], 2);
        assert_eq!(data["summary"]["determinism_failures"], 0);
        let mut expected = std::collections::BTreeMap::<String, (u64, u64)>::new();
        let mut per_faction = std::collections::BTreeMap::<(String, String), (u64, u64)>::new();
        for (i, game) in data["games"].as_array().unwrap().iter().enumerate() {
            winners += u64::from(!game["winner_team"].is_null());
            let seed = 930001 + i;
            let tape = Tape::from_json(
                &fs::read_to_string(tapes.join(format!("game-{i:05}-seed-{seed}.json"))).unwrap(),
            )
            .unwrap();
            let mut session = Session::new(&tape.setup, |_| BrainKind::Scripted);
            for turn in &tape.turns {
                assert_eq!(turn.orders, session.ai_orders());
                session.step(&turn.orders);
                assert_eq!(session.state.hash().to_string(), turn.hash);
            }
            for (seat, player) in game["players"].as_array().unwrap().iter().enumerate() {
                let difficulty = ["Easy", "Normal", "Hard"][(seat + offset) % 3];
                assert_eq!(player["difficulty"], difficulty);
                assert_eq!(
                    format!("{:?}", tape.setup.players[seat].difficulty),
                    difficulty
                );
                let won = u64::from(player["team"] == game["winner_team"]);
                let d = expected.entry(difficulty.into()).or_default();
                d.0 += 1;
                d.1 += won;
                let f = per_faction
                    .entry((
                        player["faction"].as_str().unwrap().into(),
                        difficulty.into(),
                    ))
                    .or_default();
                f.0 += 1;
                f.1 += won;
            }
        }
        for (d, (seats, wins)) in expected {
            assert_eq!(
                data["summary"]["by_difficulty"][&d],
                json!({"seats": seats, "wins": wins, "rate": wins as f64 / seats as f64})
            );
            let total = totals.entry(d).or_default();
            total.0 += seats;
            total.1 += wins;
        }
        for ((f, d), (seats, wins)) in per_faction {
            assert_eq!(
                data["summary"]["by_faction_difficulty"][&f][&d],
                json!({"seats": seats, "wins": wins, "rate": wins as f64 / seats as f64})
            );
        }
        assert!(md.contains("## Win rate by difficulty"));
        assert!(md.contains("## Win rate by faction and difficulty"));
    }
    assert!(totals.values().all(|(seats, _)| *seats == 16));
    assert!(winners > 0);
    assert_eq!(totals.values().map(|(_, wins)| wins).sum::<u64>(), winners);
}

#[test]
fn forced_strategies_reach_orders_tapes_classification_and_both_report_views() {
    use legatum_ai::{Brain, ScriptedBrain, Strategy};
    use legatum_sim::diversity::{Evidence, classify};
    let temp = TempDir::new("strategies");
    let tapes = temp.0.join("tapes");
    let (data, md) = report(
        &temp,
        &[
            "--games",
            "1",
            "--seed",
            "930001",
            "--turn-limit",
            "12",
            "--strategies",
            "rush,wide,economic,science,turtle,economic,rush,wide",
            "--determinism",
            "--tape-out",
            tapes.to_str().unwrap(),
        ],
    );
    let strategies = [
        Strategy::Rush,
        Strategy::Wide,
        Strategy::Economic,
        Strategy::Science,
        Strategy::Turtle,
        Strategy::Economic,
        Strategy::Rush,
        Strategy::Wide,
    ];
    let tape =
        Tape::from_json(&fs::read_to_string(tapes.join("game-00000-seed-930001.json")).unwrap())
            .unwrap();
    let mut state = new_game(&tape.setup);
    // Rival trends and assessment cadence depend on each brain's observed history.
    let mut brains: Vec<_> = state
        .players
        .iter()
        .map(|p| {
            ScriptedBrain::with_strategy(
                p.faction,
                legatum_wasm::session::brain_seed(state.setup.seed, p.id),
                p.id,
                strategies[p.id.0 as usize],
            )
        })
        .collect();
    for turn in &tape.turns {
        for p in state.players.iter().filter(|p| p.alive) {
            let view = legatum_core::player_view(&state, p.id);
            assert_eq!(turn.orders[&p.id], brains[p.id.0 as usize].orders(&view));
        }
        state = resolve_turn(&state, &turn.orders).0;
        assert_eq!(state.hash().to_string(), turn.hash);
    }
    for (seat, strategy) in strategies.iter().enumerate() {
        let p = &data["games"][0]["players"][seat];
        assert_eq!(p["strategy"], strategy.name());
        let evidence: Evidence = serde_json::from_value(
            data["games"][0]["telemetry"]["strategy"]["empires"][seat.to_string()].clone(),
        )
        .unwrap();
        assert_eq!(p["archetype"], json!(classify(&evidence)));
    }
    assert_eq!(data["summary"]["diversity"]["observed"]["seats"], 8);
    assert_eq!(data["summary"]["diversity"]["assigned"]["seats"], 8);
    assert_eq!(data["summary"]["determinism_failures"], 0);
    assert!(md.contains("## Strategy diversity: observed behavior"));
    assert!(md.contains("## Strategy diversity: assigned scripted biases"));
    assert!(md.contains("### Assigned bias versus observed behavior"));
    assert!(md.contains("### Faction wins by archetype"));
}

#[test]
fn diversity_cli_exits_fail_or_insufficient_after_writing_reports_and_rejects_invalid_inputs() {
    let temp = TempDir::new("diversity-exits");
    let run = |args: &[&str], label: &str| {
        Command::new(env!("CARGO_BIN_EXE_legatum-sim"))
            .args(args)
            .args(["--threads", "1", "--out-dir"])
            .arg(&temp.0)
            .args(["--label", label])
            .output()
            .unwrap()
    };
    let empty = run(&["--games", "0", "--diversity-check"], "empty");
    assert_eq!(
        empty.status.code(),
        Some(2),
        "{}",
        String::from_utf8_lossy(&empty.stderr)
    );
    let concentrated = run(
        &["--games", "1", "--seed", "930001", "--diversity-check"],
        "concentrated",
    );
    assert_eq!(
        concentrated.status.code(),
        Some(1),
        "{}",
        String::from_utf8_lossy(&concentrated.stderr)
    );
    let files: Vec<_> = fs::read_dir(&temp.0)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    assert_eq!(
        files
            .iter()
            .filter(|p| p.extension().is_some_and(|x| x == "json"))
            .count(),
        2
    );
    assert_eq!(
        files
            .iter()
            .filter(|p| p.extension().is_some_and(|x| x == "md"))
            .count(),
        2
    );
    for args in [
        vec!["--games", "1", "--strategy-matchups"],
        vec!["--games", "0", "--strategies", "rush,wide,tall"],
        vec!["--games", "0", "--strategies", "diplomatic"],
        vec!["--games", "0", "--strategies", "rush", "--brain", "random"],
        vec![
            "--games",
            "0",
            "--strategies",
            "rush",
            "--difficulty-rotation",
            "1",
        ],
        vec!["--games", "0", "--diversity-min-entropy", "NaN"],
        vec!["--games", "0", "--diversity-min-entropy", "1.01"],
        vec![
            "--games",
            "0",
            "--diversity-check",
            "--diversity-basis",
            "assigned",
        ],
    ] {
        assert!(!run(&args, "invalid").status.success(), "{args:?}");
    }
}
