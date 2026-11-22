use serde_json::Value;
use std::process::Command;

#[test]
fn proxy_cli_rotates_and_reports_only_the_seven_ais_incidents() {
    let out = std::env::temp_dir().join(format!("legatum-proxy-test-{}", std::process::id()));
    let result = Command::new(env!("CARGO_BIN_EXE_legatum-sim"))
        .args([
            "--proxy",
            "3",
            "--rotate-proxy",
            "--difficulty",
            "hard",
            "--games",
            "2",
            "--seed",
            "700001",
            "--turn-limit",
            "20",
            "--determinism",
            "--threads",
            "2",
            "--label",
            "proxy-cli",
        ])
        .arg("--out-dir")
        .arg(&out)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let path = std::fs::read_dir(&out)
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| p.extension().is_some_and(|e| e == "json"))
        .unwrap();
    let report: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(report["summary"]["proxy"]["incident_observed_games"], 2);
    assert_eq!(report["summary"]["determinism_failures"], 0);
    assert_eq!(report["meta"]["proxy"]["start"], "peace");
    assert_eq!(report["meta"]["proxy"]["version"], 4);
    assert_eq!(report["meta"]["proxy_version"], 4);
    for (i, g) in report["games"].as_array().unwrap().iter().enumerate() {
        let seat = i as u64 + 3;
        assert_eq!(g["telemetry"]["proxy"]["seat"], seat);
        assert_eq!(g["telemetry"]["proxy"]["proxy_version"], 4);
        let players = g["players"].as_array().unwrap();
        assert_eq!(players.iter().filter(|p| p["brain"] == "proxy").count(), 1);
        assert_eq!(players[seat as usize]["brain"], "proxy");
        assert!(players.iter().all(|p| p["difficulty"] == "Hard"));
        assert_eq!(g["telemetry"]["proxy"]["snapshots"], serde_json::json!({}));
        assert!(g["telemetry"]["proxy"]["detector_version"].is_number());
    }
    let ai_seats: u64 = report["ai_analysis"]["seats_by_faction"]
        .as_object()
        .unwrap()
        .values()
        .map(|v| v.as_u64().unwrap())
        .sum();
    assert_eq!(ai_seats, 14);
    for (kind, n) in report["summary"]["proxy"]["ai_incidents"]
        .as_object()
        .unwrap()
    {
        let aggregate: u64 = report["ai_analysis"]["rates"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["attacker_faction"].is_null() && r["behavior"] == *kind)
            .map(|r| r["incidents"].as_u64().unwrap())
            .sum();
        assert_eq!(n.as_u64().unwrap(), aggregate, "{kind}");
    }
}

#[test]
fn all_policy_versions_and_start_modes_are_reported_and_repeat() {
    for version in ["2", "4", "5", "6"] {
        for start in ["war", "peace"] {
            let out = std::env::temp_dir().join(format!(
                "legatum-proxy-matrix-{}-{version}-{start}",
                std::process::id()
            ));
            let result = Command::new(env!("CARGO_BIN_EXE_legatum-sim"))
                .args([
                    "--proxy",
                    "0",
                    "--proxy-version",
                    version,
                    "--proxy-start",
                    start,
                    "--games",
                    "1",
                    "--seed",
                    "830001",
                    "--turn-limit",
                    "25",
                    "--determinism",
                    "--threads",
                    "1",
                ])
                .arg("--out-dir")
                .arg(&out)
                .output()
                .unwrap();
            assert!(
                result.status.success(),
                "{}",
                String::from_utf8_lossy(&result.stderr)
            );
            let path = std::fs::read_dir(out)
                .unwrap()
                .map(|e| e.unwrap().path())
                .find(|p| p.extension().is_some_and(|x| x == "json"))
                .unwrap();
            let report: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
            let number: u32 = version.parse().unwrap();
            assert_eq!(report["meta"]["proxy_version"], number);
            assert_eq!(report["meta"]["proxy"]["version"], number);
            assert_eq!(report["meta"]["proxy"]["start"], start);
            assert_eq!(
                report["games"][0]["telemetry"]["proxy"]["proxy_version"],
                number
            );
            assert_eq!(report["summary"]["proxy"]["proxy_versions"][version], 1);
            assert_eq!(report["summary"]["determinism_checked"], 1);
            assert_eq!(report["summary"]["determinism_failures"], 0);
            assert_eq!(
                report["meta"]["first_setup"]["legacy_all_war"]
                    .as_bool()
                    .unwrap_or(false),
                start == "war"
            );
        }
    }
}

#[test]
fn opening_comparator_is_named_and_does_not_claim_a_proxy_version() {
    let out = std::env::temp_dir().join(format!("legatum-proxy-opening-{}", std::process::id()));
    let result = Command::new(env!("CARGO_BIN_EXE_legatum-sim"))
        .args([
            "--proxy",
            "0",
            "--proxy-opening",
            "GreedyExpand",
            "--rotate-proxy",
            "--legacy-hard",
            "all",
            "--difficulty",
            "hard",
            "--games",
            "2",
            "--seed",
            "3600001",
            "--turn-limit",
            "20",
            "--threads",
            "2",
            "--determinism",
        ])
        .arg("--out-dir")
        .arg(&out)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let path = std::fs::read_dir(out)
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| p.extension().is_some_and(|x| x == "json"))
        .unwrap();
    let report: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    assert!(report["meta"]["proxy_version"].is_null());
    assert_eq!(report["summary"]["determinism_failures"], 0);
    for (i, game) in report["games"].as_array().unwrap().iter().enumerate() {
        let proxy = &game["telemetry"]["proxy"];
        assert_eq!(proxy["seat"], i);
        assert!(proxy["proxy_version"].is_null());
        assert_eq!(proxy["benchmark_brain"], "opening:GreedyExpand");
        assert_eq!(game["players"][i]["brain"], "hard-v2:GreedyExpand");
    }
}
