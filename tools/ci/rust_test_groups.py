#!/usr/bin/env python3
"""Split the workspace's Rust test targets into CI groups that build and run on separate runners.

Usage:
  tools/ci/rust_test_groups.py names          # JSON list of group names (the CI matrix)
  tools/ci/rust_test_groups.py args <group>   # cargo target flags for one group
  tools/ci/rust_test_groups.py check          # prove every test target is in exactly one group

Every target that `cargo test` runs (lib, bins, integration tests, and examples marked `test = true`) lands in
exactly one group, read from `cargo metadata`, so a new test file or crate is picked up without editing this file.
Examples without `test = true` are not built here: clippy --all-targets already compiles them.

Why groups: a GitHub-hosted runner for a private repo has 2 cores, and one release build of every test target took
about 50 minutes there. Each group rebuilds the core and AI libraries it needs, but the groups run side by side.
"""
import json
import subprocess
import sys

AI = "legatum-ai"
AI_TEST_SPLIT = 2  # the AI crate's integration tests are the largest share of the build


def test_targets():
    meta = json.loads(subprocess.check_output(["cargo", "metadata", "--no-deps", "--format-version", "1"]))
    out = []
    for pkg in meta["packages"]:
        for t in pkg["targets"]:
            if not t.get("test"):
                continue
            kind = t["kind"][0]
            if kind in ("lib", "rlib", "cdylib", "staticlib", "dylib", "proc-macro"):
                kind = "lib"
            out.append((pkg["name"], kind, t["name"]))
    return sorted(out)


def group_of(target, ai_tests):
    pkg, kind, name = target
    if pkg == "legatum-core":
        return "core"
    if pkg == AI:
        if kind == "lib":
            return "ai-lib"
        if kind == "test":
            return f"ai-tests-{ai_tests.index(name) % AI_TEST_SPLIT + 1}"
        return "ai-tests-1"  # examples with tests, bins
    if pkg == "legatum-sim":
        return "sim"
    return "platform"  # wasm, server, archive, analysis, and any new crate


def groups():
    targets = test_targets()
    ai_tests = sorted(n for p, k, n in targets if p == AI and k == "test")
    out = {}
    for t in targets:
        out.setdefault(group_of(t, ai_tests), []).append(t)
    return out


def args(members):
    """Cargo flags that select exactly these targets."""
    by_pkg = {}
    for pkg, kind, name in members:
        by_pkg.setdefault(pkg, []).append((kind, name))
    flags = []
    for pkg in sorted(by_pkg):
        flags += ["-p", pkg]
    # A whole package (or set of whole packages) is selected with --tests; partial packages name their targets.
    all_targets = test_targets()
    whole = all(sorted(by_pkg[p]) == sorted((k, n) for q, k, n in all_targets if q == p) for p in by_pkg)
    if whole:
        return flags + ["--tests"]
    if len(by_pkg) != 1:
        sys.exit("a partial group must stay inside one package so --test names are unambiguous")
    for kind, name in sorted(by_pkg[next(iter(by_pkg))]):
        flags += ["--lib"] if kind == "lib" else [f"--{kind}", name]
    return flags


def main():
    cmd = sys.argv[1] if len(sys.argv) > 1 else ""
    g = groups()
    if cmd == "names":
        print(json.dumps(sorted(g)))
    elif cmd == "args" and len(sys.argv) == 3:
        if sys.argv[2] not in g:
            sys.exit(f"unknown group {sys.argv[2]!r}; groups: {', '.join(sorted(g))}")
        print(" ".join(args(g[sys.argv[2]])))
    elif cmd == "check":
        seen = [t for members in g.values() for t in members]
        assert sorted(seen) == test_targets() and len(seen) == len(set(seen)), "groups must cover each target once"
        for name in sorted(g):
            print(f"{name}: {len(g[name])} targets")
    else:
        sys.exit(__doc__)


if __name__ == "__main__":
    main()
