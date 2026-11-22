use std::{fs, process::Command};

#[test]
fn archive_cli_writes_verified_json_and_narrative_and_never_overwrites_on_divergence() {
    let temp = tempfile::tempdir().unwrap();
    let tape = temp.path().join("repro.json");
    let out = temp.path().join("out");
    let generated = Command::new(env!("CARGO_BIN_EXE_legatum-analyze"))
        .arg("--write-repro")
        .arg(&tape)
        .arg("--out")
        .arg(&out)
        .output()
        .unwrap();
    assert!(
        generated.status.success(),
        "{}",
        String::from_utf8_lossy(&generated.stderr)
    );
    let recording = legatum_archive::read(&tape).unwrap();
    let identity = Command::new(env!("CARGO_BIN_EXE_legatum-analyze"))
        .arg("--engine-info")
        .output()
        .unwrap();
    assert!(identity.status.success());
    let identity: serde_json::Value = serde_json::from_slice(&identity.stdout).unwrap();
    assert_eq!(
        identity["rules_fingerprint"],
        recording.metadata.rules_fingerprint
    );
    let compressed = temp.path().join("repro.tape.json.zst");
    fs::write(&compressed, legatum_archive::encode(&recording).unwrap()).unwrap();
    let run = |input: &std::path::Path| {
        Command::new(env!("CARGO_BIN_EXE_legatum-analyze"))
            .arg(input)
            .arg("--out")
            .arg(&out)
            .output()
            .unwrap()
    };
    let analyzed = run(&compressed);
    assert!(analyzed.status.success());
    let manifest: serde_json::Value = serde_json::from_slice(&analyzed.stdout).unwrap();
    assert!(manifest["incident_count"].as_u64().unwrap() >= 5);
    let report = out.join("ranger-passivity-repro.incidents.json");
    let original = fs::read(&report).unwrap();
    assert!(out.join("ranger-passivity-repro.narrative.md").exists());
    let mut recording = recording;
    recording.tape.turns[1].hash = "0".into();
    fs::write(&tape, serde_json::to_vec(&recording).unwrap()).unwrap();
    let failed = run(&tape);
    assert!(!failed.status.success());
    assert!(String::from_utf8_lossy(&failed.stderr).contains("first divergent turn: 2"));
    assert_eq!(fs::read(report).unwrap(), original);
}
