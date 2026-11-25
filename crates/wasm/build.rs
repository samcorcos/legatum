use sha2::{Digest, Sha256};
use std::{fs, path::Path, process::Command};

fn sources(dir: &Path, paths: &mut Vec<std::path::PathBuf>) {
    for entry in fs::read_dir(dir).expect("engine source directory") {
        let path = entry.expect("source entry").path();
        if path.is_dir() {
            sources(&path, paths);
        } else if path.extension().is_some_and(|e| e == "rs") {
            paths.push(path);
        }
    }
}

fn main() {
    // Read at run time: a compile-time env!() path can come from another worktree via the sccache cache.
    let manifest = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let root = Path::new(&manifest).join("../..");
    let mut paths = Vec::new();
    sources(&root.join("crates/core/src"), &mut paths);
    sources(&root.join("crates/wasm/src"), &mut paths);
    paths.sort();
    let mut hash = Sha256::new();
    for path in paths {
        println!("cargo:rerun-if-changed={}", path.display());
        hash.update(
            path.strip_prefix(&root)
                .unwrap()
                .to_string_lossy()
                .as_bytes(),
        );
        hash.update([0]);
        hash.update(fs::read(path).expect("read engine source"));
    }
    // sha2 0.11 digests no longer implement LowerHex; the same lowercase hex, byte by byte.
    let fp: String = hash.finalize().iter().map(|b| format!("{b:02x}")).collect();
    println!("cargo:rustc-env=LEGATUM_RULES_FINGERPRINT={fp}");
    let git = |args: &[&str]| {
        Command::new("git")
            .args(args)
            .current_dir(&root)
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
    };
    // Resolve worktree HEAD and its symbolic ref so provenance refreshes after a commit.
    for name in [
        "HEAD".to_owned(),
        git(&["symbolic-ref", "HEAD"]).unwrap_or_default(),
    ] {
        if !name.is_empty()
            && let Some(path) = git(&["rev-parse", "--git-path", &name])
        {
            println!("cargo:rerun-if-changed={}", root.join(path).display());
        }
    }
    println!(
        "cargo:rustc-env=LEGATUM_ENGINE_COMMIT={}",
        git(&["rev-parse", "HEAD"]).unwrap_or_else(|| "unknown".into())
    );
}
