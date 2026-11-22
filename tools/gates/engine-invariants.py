#!/usr/bin/env python3
"""Engine invariants: the Rust core stays deterministic, integer-only and free of rendering code.

Checks the non-test Rust in crates/core and crates/ai (tests, examples and *_tests.rs are skipped) for:
  float      f32/f64 types and float literals (integer maths only)
  hash-order HashMap/HashSet/RandomState: iteration order is random per process. Use BTreeMap/BTreeSet or a Vec;
             a map used only for lookups (get/insert/contains, never iterated) may stay with a gate-allow reason
  rng        unseeded or global randomness (thread_rng, rand::random, OsRng, from_entropy, getrandom)
  clock      wall-clock time in rules (std::time, SystemTime, Instant::now, chrono)
  render     rendering, UI or browser crates, in code or in the crate's Cargo.toml [dependencies]
  save-shape GameState, Player, Unit or City (the saved state) changed without a RULES_VERSION bump
  fixtures   RULES_VERSION changed but a `.current.` save fixture still carries another save_version
             (refresh with tools/sim/refresh-save-version-fixtures.py OLD NEW)

Escape for one line: `// gate-allow: <reason>` on the line or the line above.

  tools/gates/engine-invariants.py [--repo DIR] BASE TIP     added lines of BASE..TIP (the merge gate)
  tools/gates/engine-invariants.py [--repo DIR] --all [REF]  every line at REF (default HEAD)
Exit 1 when anything is found.
"""
import argparse
import gzip
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from common import added_lines, allowed, changed_files, git, show, strip_rust, test_regions  # noqa: E402

SCOPE = re.compile(r"^crates/(core|ai)/src/.*\.rs$")
SKIP = re.compile(r"(_tests\.rs$|/tests\.rs$|/tests/|/testdata/|/benches/|/examples/)")
CARGO = re.compile(r"^crates/(core|ai)/Cargo\.toml$")
RENDER_CRATES = ("wgpu winit web_sys js_sys wasm_bindgen tauri egui eframe bevy macroquad sdl2 glium glutin ggez "
                 "minifb iced druid pixels image tiny_skia resvg usvg crossterm ratatui piston gtk").split()
RULES = [
    ("float", re.compile(r"\bf(?:32|64)\b|(?<![\w.])\d[\d_]*\.\d[\d_]*(?:[eE][+-]?\d+)?(?![\w.])|\b\d[\d_]*[eE][+-]?\d+\b"),
     "floating point in the engine; use integer maths (fixed point in basis points or per-mille)"),
    ("hash-order", re.compile(r"\b(?:HashMap|HashSet|FxHash\w*|AHash\w*|RandomState)\b"),
     "hash containers iterate in random order; use BTreeMap/BTreeSet or a Vec, or mark lookup-only use with "
     "`// gate-allow: lookup only, never iterated`"),
    ("rng", re.compile(r"\b(?:thread_rng|rand::random|rand::rng|OsRng|from_entropy|getrandom)\b"),
     "unseeded randomness; draw from the game's seeded per-purpose stream (crates/core/src/rng.rs)"),
    ("clock", re.compile(r"\bstd::time\b|\bSystemTime\b|\bInstant::now\b|\bUNIX_EPOCH\b|\bchrono::|\b(?:Utc|Local)::now\b"),
     "wall-clock time in the engine; results must depend only on state, orders and seed"),
    ("render", re.compile(r"\b(?:%s)::" % "|".join(RENDER_CRATES)),
     "rendering or UI code in the engine; the app and renderer drive the engine, not the other way round"),
]
SAVED = ("GameState", "Player", "Unit", "City")
FIXTURE_ROOTS = ("crates/wasm/tests/fixtures/", "app/e2e/saves/", "app/src/test/fixtures/")
SAVE_VERSION = re.compile(rb'\\?"save_version\\?"\s*:\s*(\d+)')


def in_scope(path):
    return bool(SCOPE.match(path)) and not SKIP.search(path)


def scan_file(text, only=None):
    """[(line_no, rule, message, source_line, allow_reason)] for one Rust file."""
    raw = text.split("\n")
    code = strip_rust(text)
    tests = test_regions(code)
    hits = []
    for i, line in enumerate(code):
        if (only is not None and i + 1 not in only) or (i < len(tests) and tests[i]):
            continue
        for rule, pat, msg in RULES:
            if pat.search(line):
                hits.append((i + 1, rule, msg, raw[i].strip() if i < len(raw) else "", allowed(raw, i)))
    return hits


def cargo_render_deps(text):
    """Rendering crates named in [dependencies] (and target-specific dependency tables)."""
    deps, section = [], ""
    for line in (text or "").splitlines():
        s = line.strip()
        if s.startswith("["):
            section = s
            continue
        if re.match(r"\[(target\.[^\]]*\.)?dependencies", section):
            m = re.match(r"([A-Za-z0-9_-]+)\s*=", s)
            if m and m.group(1).replace("-", "_") in RENDER_CRATES:
                deps.append(m.group(1))
    return deps


def struct_bodies(repo, rev):
    """{name: normalised struct text with its attributes} for the saved-state types at REV."""
    hits = git(repo, "grep", "-l", "-E", r"struct (%s)([^A-Za-z0-9_]|$)" % "|".join(SAVED), rev, "--", "crates/core/src/*.rs") or ""
    found = {}
    for path in sorted({ln.split(":", 1)[1] for ln in hits.splitlines() if ":" in ln}):
        text = show(repo, rev, path) or ""
        code = strip_rust(text)
        for i, line in enumerate(code):
            m = re.match(r"\s*(?:pub(?:\([^)]*\))?\s+)?struct (\w+)\b", line)
            if not m or m.group(1) not in SAVED:
                continue
            start = i
            while start > 0 and code[start - 1].strip().startswith("#["):
                start -= 1
            depth, j, body = 0, i, []
            while j < len(code):
                body.append(code[j])
                depth += code[j].count("{") - code[j].count("}")
                if depth <= 0 and "{" in "".join(body):
                    break
                j += 1
            found[m.group(1)] = re.sub(r"\s+", " ", " ".join(code[start:i] + body)).strip()
    return found


def rules_version(repo, rev):
    m = re.search(r"pub const RULES_VERSION: u32 = (\d+);", show(repo, rev, "crates/core/src/lib.rs") or "")
    return int(m.group(1)) if m else None


def stale_fixtures(repo, rev, version):
    out = []
    for path in (git(repo, "ls-tree", "-r", "--name-only", rev, *FIXTURE_ROOTS) or "").split():
        if ".current." not in os.path.basename(path) or path.endswith(".zst"):
            continue
        raw = show(repo, rev, path, binary=True) or b""
        if path.endswith(".gz"):
            try:
                raw = gzip.decompress(raw)
            except OSError:
                continue
        versions = {int(v) for v in SAVE_VERSION.findall(raw)}
        if versions and versions != {version}:
            out.append(f"{path} has save_version {','.join(map(str, sorted(versions)))}")
    return out


def check(repo, base, tip, all_mode=False):
    """(findings, allowed) as printable lines."""
    findings, allows = [], []
    if all_mode:
        paths = [p for p in (git(repo, "ls-tree", "-r", "--name-only", tip, "crates/core", "crates/ai") or "").split()]
        targets = [(p, None) for p in paths if in_scope(p)]
        cargos = [p for p in paths if CARGO.match(p)]
    else:
        rows = changed_files(repo, base, tip)
        targets = [(p, added_lines(repo, base, tip, p)) for s, p in rows if s != "D" and in_scope(p)]
        cargos = [p for s, p in rows if s != "D" and CARGO.match(p)]
    for path, only in targets:
        for line_no, rule, msg, src, reason in scan_file(show(repo, tip, path) or "", only):
            where = f"{path}:{line_no}: [{rule}]"
            if reason:
                allows.append(f"{where} allowed ({reason})")
            else:
                findings.append(f"{where} {msg}\n    {src[:160]}")
    for path in cargos:
        for dep in cargo_render_deps(show(repo, tip, path)):
            findings.append(f"{path}: [render] depends on `{dep}`; the engine crates must not pull in rendering or UI crates")
    version = rules_version(repo, tip)
    if not all_mode and any(in_scope(p) or p.startswith("crates/core/") for _, p in changed_files(repo, base, tip)):
        before, after = struct_bodies(repo, base), struct_bodies(repo, tip)
        changed = sorted(n for n in SAVED if n in before and n in after and before[n] != after[n])
        old = rules_version(repo, base)
        if changed and version is not None and old == version:
            findings.append(f"crates/core/src/lib.rs: [save-shape] the saved-state type{'s' if len(changed) > 1 else ''} "
                            f"{', '.join(changed)} changed but RULES_VERSION is still {version}; bump it and run "
                            f"`python3 tools/sim/refresh-save-version-fixtures.py {version} {version + 1 if version else 'NEW'}`")
    if version is not None and (all_mode or rules_version(repo, base) != version):
        for s in stale_fixtures(repo, tip, version):
            findings.append(f"{s.split(' ')[0]}: [fixtures] {s.split(' ', 1)[1]} but RULES_VERSION is {version}; run "
                            f"`python3 tools/sim/refresh-save-version-fixtures.py OLD {version}`")
    return findings, allows


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--repo", default=".")
    ap.add_argument("--all", action="store_true", help="scan every line at one commit")
    ap.add_argument("--verbose", "-v", action="store_true", help="also list the lines a gate-allow covers")
    ap.add_argument("revs", nargs="*")
    a = ap.parse_args(argv)
    if a.all:
        base, tip = None, (a.revs[0] if a.revs else "HEAD")
    elif len(a.revs) == 2:
        base, tip = a.revs
    else:
        ap.error("give BASE TIP, or --all [REF]")
    findings, allows = check(a.repo, base, tip, a.all)
    if a.verbose:
        for line in allows:
            print("allow  " + line)
    for line in findings:
        print("FAIL   " + line)
    print(f"engine-invariants: {len(findings)} finding(s), {len(allows)} allowed")
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
