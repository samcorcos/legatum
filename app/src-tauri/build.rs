use std::process::Command;

/// App commands. Declaring them gives each an `allow-<command>` permission, so a capability can
/// grant a command to one origin only (the host page on http://127.0.0.1 gets two of them,
/// capabilities/online-remote.json). Every command must also be allowed in
/// capabilities/default.json, or the bundled UI can no longer call it.
const COMMANDS: &[&str] = &[
    "archive_stage",
    "archive_commit",
    "scenario_save_run",
    "scenario_dropins",
    "training_start",
    "training_poll",
    "training_log_append",
    "why_log_append",
    "online_host_start",
    "online_host_stop",
    "online_info",
];

fn git(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn main() {
    // The build label (src/online.rs::build_label) mirrors app/src/buildLabel.ts: the release
    // label from LEGATUM_BUILD_LABEL if set, else `git describe` against the last playtest tag,
    // the same command app/vite.config.ts runs.
    println!("cargo:rerun-if-env-changed=LEGATUM_BUILD_LABEL");
    println!("cargo:rerun-if-env-changed=LEGATUM_RELAY_URL");
    let describe =
        git(&["describe", "--tags", "--match", "playtest-*", "--always"]).unwrap_or_default();
    println!("cargo:rustc-env=LEGATUM_BUILD_DESCRIBE={describe}");
    // Rerun when HEAD moves or a tag appears, so a dev build's label does not go stale.
    let mut watched = vec!["HEAD".to_string(), "packed-refs".into(), "refs/tags".into()];
    if let Some(branch) = git(&["symbolic-ref", "-q", "HEAD"]) {
        watched.push(branch);
    }
    for path in watched {
        if let Some(p) = git(&["rev-parse", "--git-path", &path]) {
            if std::path::Path::new(&p).exists() {
                println!("cargo:rerun-if-changed={p}");
            }
        }
    }

    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(COMMANDS)),
    )
    .expect("tauri build script failed");
}
