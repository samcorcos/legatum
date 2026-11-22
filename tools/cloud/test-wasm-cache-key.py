#!/usr/bin/env python3
"""Exercise build-wasm's exact-input cache key against isolated source changes."""
import os
import pathlib
import shutil
import subprocess
import tempfile


ROOT = pathlib.Path(__file__).resolve().parents[2]


def copy_inputs(destination):
    for name in ("Cargo.toml", "Cargo.lock"):
        shutil.copy2(ROOT / name, destination / name)
    for name in ("crates", ".cargo"):
        source = ROOT / name
        if source.exists():
            shutil.copytree(source, destination / name)
    (destination / "scripts").mkdir()
    shutil.copy2(ROOT / "scripts/build-wasm.sh", destination / "scripts/build-wasm.sh")
    for name in ("rust-toolchain", "rust-toolchain.toml"):
        source = ROOT / name
        if source.exists():
            shutil.copy2(source, destination / name)


def current_key(cache):
    entries = [path.name for path in cache.iterdir() if path.is_dir() and len(path.name) == 64]
    if not entries:
        raise AssertionError("build did not create a cache entry")
    return max((cache / name for name in entries), key=lambda path: path.stat().st_mtime_ns).name


def build(project, cache, fake_bin, mutate=None):
    if mutate is not None:
        with mutate.open("a") as file:
            file.write("\n# cache-key regression input\n")
    subprocess.run(
        [str(project / "scripts/build-wasm.sh")],
        cwd=project,
        env={**os.environ, "PATH": f"{fake_bin}:{os.environ['PATH']}",
             "LEGATUM_WASM_CACHE_DIR": str(cache), "LEGATUM_TIMED_BUILD_WASM": "1"},
        check=True,
        stdout=subprocess.DEVNULL,
    )
    return current_key(cache)


def main():
    with tempfile.TemporaryDirectory(prefix="wasm-cache-key-") as temporary:
        base = pathlib.Path(temporary)
        project = base / "project"
        cache = base / "cache"
        fake_bin = base / "bin"
        project.mkdir()
        fake_bin.mkdir()
        copy_inputs(project)

        # Keep the test on the real cache/key path without invoking a WASM build.
        wasm_pack = fake_bin / "wasm-pack"
        wasm_pack.write_text(
            "#!/usr/bin/env python3\n"
            "import pathlib, sys\n"
            "if sys.argv[1:2] == ['--version']:\n"
            " print('wasm-pack test stub 1')\n"
            "elif sys.argv[1:2] == ['build']:\n"
            " out = pathlib.Path(sys.argv[sys.argv.index('--out-dir') + 1])\n"
            " out.mkdir(parents=True, exist_ok=True)\n"
            " (out / 'stub.txt').write_text('package')\n"
            "else: raise SystemExit(2)\n"
        )
        wasm_pack.chmod(0o755)
        flock = fake_bin / "flock"
        flock.write_text("#!/bin/sh\nexit 0\n")
        flock.chmod(0o755)
        env = {**os.environ, "PATH": f"{fake_bin}:{os.environ['PATH']}"}
        subprocess.run(["git", "init", "-q"], cwd=project, env=env, check=True)
        subprocess.run(["git", "add", "-A"], cwd=project, env=env, check=True)

        # The first invocation populates a cache entry; repeat it through build().
        initial = build(project, cache, fake_bin)
        sim = build(project, cache, fake_bin, project / "crates/sim/Cargo.toml")
        if sim != initial:
            raise AssertionError("changing crates/sim changed the WASM cache key")

        for relative in (
            "crates/core/Cargo.toml",
            "crates/ai/Cargo.toml",
            "crates/wasm/Cargo.toml",
            "Cargo.lock",
            "scripts/build-wasm.sh",
        ):
            path = project / relative
            original = path.read_bytes()
            key = build(project, cache, fake_bin, path)
            if key == initial:
                raise AssertionError(f"changing {relative} did not change the WASM cache key")
            path.write_bytes(original)

        # When metadata cannot be derived, retain the old all-crates key scope.
        failing_cargo = fake_bin / "cargo"
        failing_cargo.write_text("#!/bin/sh\nexit 1\n")
        failing_cargo.chmod(0o755)
        baseline = build(project, cache, fake_bin)
        sim_manifest = project / "crates/sim/Cargo.toml"
        original = sim_manifest.read_bytes()
        fallback_key = build(project, cache, fake_bin, sim_manifest)
        if fallback_key == baseline:
            raise AssertionError("metadata failure did not fall back to hashing all crates")
        sim_manifest.write_bytes(original)

    print("WASM cache key regression checks passed")


if __name__ == "__main__":
    main()
