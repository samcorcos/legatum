use std::{fs, process::Command};

#[test]
fn batch_analysis_is_opt_in_and_thread_independent() {
    let dir = std::env::temp_dir().join(format!("legatum-batch-analysis-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let run = |label: &str, threads: &str, analyze: bool| {
        let mut command = Command::new(env!("CARGO_BIN_EXE_legatum-sim"));
        command
            .args([
                "--games",
                "2",
                "--seed",
                "99",
                "--turn-limit",
                "3",
                "--label",
                label,
                "--threads",
                threads,
                "--out-dir",
            ])
            .arg(&dir);
        if analyze {
            command.arg("--analyze");
        }
        let output = command.output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let path = fs::read_dir(&dir)
            .unwrap()
            .map(|e| e.unwrap().path())
            .find(|p| {
                p.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .ends_with(&format!("-{label}.json"))
            })
            .unwrap();
        serde_json::from_slice::<serde_json::Value>(&fs::read(path).unwrap()).unwrap()
    };
    let one = run("one", "1", true);
    let two = run("two", "2", true);
    assert_eq!(one["ai_analysis"], two["ai_analysis"]);
    assert_eq!(one["ai_analysis"]["games"], 2);
    assert_eq!(
        one["ai_analysis"]["seats_by_faction"]
            .as_object()
            .unwrap()
            .len(),
        9, // Seeded analysis retains the established nine-faction roster.
    );
    assert!(!one["ai_analysis"]["rates"].as_array().unwrap().is_empty());
    assert!(run("off", "1", false)["ai_analysis"].is_null());
}
