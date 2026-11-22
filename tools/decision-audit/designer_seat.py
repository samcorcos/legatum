#!/usr/bin/env python3
"""Designer-seat score: how often Hard wins from the designer's own positions.

    designer_seat.py [--commit REF] [--where local|box] [--salts 48] [--turns 2,20,40,60,80]
                     [--tapes KEY,...] [--threads N] [--jobs N] [--out FILE] [--no-history]
                     [--seat-params FILE]
    designer_seat.py chart          # redraw docs/status/designer-seat.png from the history

For each tape in TAPES, Hard plays the designer's seat from each start turn to the end of the game,
48 reseeded runs per start turn (the takeover curve). A tape's score is its mean win rate over the
start turns (the area under its takeover curve); the training and held-out scores are the means of
their tapes. Higher means Hard converts his positions more often.

Method (as `takeover_gap.py`): each archived tape is copied out of ~/LegatumData/archive, replayed
on its own recording engine (`audit_states`, built from the tag with `patch_recording_engine.py`)
to export hash-verified states, and `seat_run` from the measured commit continues from those
states with the current rules and AI. Older states load through serde defaults (additive fields);
`seat_run` reports how many states changed on load. Prepared tapes and recording engines are
cached under ~/LegatumRuns/designer-seat/, so a rerun only builds the measured commit.

--where local builds and runs on this Mac under `nice` (about 25 minutes on 16 cores);
--where box runs the measured checkout on the cloud build box (no spot cost; HEAD only).
Writes one JSON per run to docs/sim-results/designer-seat/ and a row to its history.jsonl.

--seat-params FILE (an AiParams JSON, e.g. one gene on) plays both the commit's Hard and Hard with
those parameters in the designer's seat on the same branch turns, salts and dice (`seat_run
--continue hard,params:FILE`; every other seat keeps the defaults). It writes two JSONs (the
control and `<out>-params.json`), prints the paired difference per tape and set, and never writes
history. --jobs caps how many tapes run at once (default all); each gets --threads / jobs threads.
"""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import glob
import hashlib
import json
import math
import os
from pathlib import Path
import re
import shlex
import shutil
import subprocess
import sys
import tempfile
import time

ROOT = Path(__file__).resolve().parents[2]
CACHE = Path(os.environ.get("LEGATUM_RUNS", Path.home() / "LegatumRuns")) / "designer-seat"
ARCHIVE = Path(os.environ.get("LEGATUM_ARCHIVE", Path.home() / "LegatumData" / "archive"))
RESULTS = ROOT / "docs/sim-results/designer-seat"
HISTORY = RESULTS / "history.jsonl"
CHART = ROOT / "docs/status/designer-seat.png"

# The designer's finished (or decided) games. Training tapes may be searched and diffed; held-out
# tapes never are. To rotate a tape, change this table and rerun the control on one commit.
TAPES = [
    dict(key="aztecs-8929", label="Aztecs 8.9.29", set="training",
         game="web-1794185723703-7e9a939d1bbeb000-40fd636faa9e7800"),
    dict(key="germany-8927", label="Germany 8.9.27", set="training",
         game="web-1794091560504-8ec026e92a999800-bfb5c3002d509000"),
    dict(key="sparta-8926", label="Sparta 8.9.26", set="training",
         game="web-1794077507992-ddcf737af418c000-906860538f220000"),
    dict(key="swiss-8926", label="Swiss 8.9.26", set="training",
         game="web-1794064787118-af4b44bc68bdf800-e65233f918e50800"),
    dict(key="india-8925", label="India 8.9.25", set="training",
         game="web-1793923917961-a54bbab40de43800-552fc98fac6de000"),
    dict(key="mongols-8925", label="Mongols 8.9.25", set="held_out",
         game="web-1794014395550-273aacc182459000-21dcb6ca9099f000"),
    dict(key="celts-8926", label="Celts 8.9.26", set="held_out",
         game="web-1794080775290-e38668a51b505800-0081f975f7c2f800"),
    # Retired from the held-out set on 2026-11-07 (replaced by Celts 8.9.26); still measured, in no set score.
    dict(key="celts-8925", label="Celts 8.9.25", set="retired",
         game="web-1793751268951-a199efcee0e0e800-4c0b5285be7c7000"),
    dict(key="swiss-8924", label="Swiss 8.9.24", set="held_out",
         game="web-1793669249959-c93b556936917800-bf98470506532800"),
]
TURNS = [2, 20, 40, 60, 80]
Z = 1.959964


def sh(*argv, cwd=ROOT, **kw):
    return subprocess.run([str(a) for a in argv], cwd=cwd, check=True, **kw)


def git(*argv) -> str:
    return subprocess.check_output(["git", *argv], cwd=ROOT, text=True).strip()


def rules_version(tree: Path) -> int:
    text = (tree / "crates/core/src/lib.rs").read_text()
    return int(re.search(r"pub const RULES_VERSION: u32 = (\d+);", text).group(1))


def export(commit: str, dest: Path) -> Path:
    """Unpack COMMIT's tree into DEST once (a frozen source, never a live worktree)."""
    if not (dest / "Cargo.toml").exists():
        tmp = dest.with_name(dest.name + ".tmp")
        shutil.rmtree(tmp, ignore_errors=True)
        tmp.mkdir(parents=True)
        archive = subprocess.Popen(["git", "archive", commit], cwd=ROOT, stdout=subprocess.PIPE)
        sh("tar", "-x", "-C", tmp, stdin=archive.stdout)
        if archive.wait():
            raise SystemExit(f"git archive {commit} failed")
        tmp.rename(dest)
    return dest


def cargo_example(tree: Path, example: str, target: Path) -> Path:
    # One target dir per source tree: `git archive` stamps files with the commit time, so a target dir
    # shared across trees can keep a stale example binary (cargo sees old mtimes as fresh).
    env = dict(os.environ, CARGO_TARGET_DIR=str(target))
    sh(ROOT / "tools/sim/build-slot.sh", "--", "nice", "-n", "10", "cargo", "build", "--release", "-q",
       "-p", "legatum-sim", "--example", example, cwd=tree, env=env)
    return target / "release/examples" / example


def wilson(wins: int, n: int) -> list[float]:
    if n == 0:
        return [0.0, 1.0]
    p = wins / n
    d = 1 + Z * Z / n
    c = (p + Z * Z / (2 * n)) / d
    h = Z * math.sqrt(p * (1 - p) / n + Z * Z / (4 * n * n)) / d
    return [round(max(0.0, c - h), 4), round(min(1.0, c + h), 4)]


def mean_ci(cells: list[dict]) -> tuple[float, list[float]]:
    """Mean of independent binomial rates; normal interval, each p kept half a win from 0 and 1."""
    k = len(cells)
    mean = sum(c["wins"] / c["runs"] for c in cells) / k
    var = 0.0
    for c in cells:
        p = min(max(c["wins"], 0.5), c["runs"] - 0.5) / c["runs"]
        var += p * (1 - p) / c["runs"]
    var /= k * k
    h = Z * math.sqrt(var)
    return round(mean, 4), [round(max(0.0, mean - h), 4), round(min(1.0, mean + h), 4)]


def find_tape(game: str) -> Path:
    hits = sorted(glob.glob(str(ARCHIVE / "*" / "*" / f"{game}*.tape.json.zst")) +
                  glob.glob(str(ARCHIVE / "in-progress" / "*" / "*" / f"{game}*.tape.json.zst")))
    if not hits:
        raise SystemExit(f"no archived tape for {game} under {ARCHIVE}")
    finished = [h for h in hits if "/in-progress/" not in h]
    return Path((finished or hits)[-1])


def prepare(tape: dict, max_turn: int) -> dict:
    """Copy the archived tape, build its bare tape and export hash-verified states (cached)."""
    source = find_tape(tape["game"])
    digest = hashlib.sha256(source.read_bytes()).hexdigest()[:16]
    work = CACHE / "tapes" / f"{tape['key']}-{digest}"
    info_path = work / "prepared.json"
    if info_path.exists():
        info = json.loads(info_path.read_text())
        if info["states_through"] >= max_turn:
            return info
    work.mkdir(parents=True, exist_ok=True)
    copy = work / source.name
    shutil.copy2(source, copy)  # never read or write the live archive in place
    sh("python3", ROOT / "tools/decision-audit/build_tape.py", work, copy, stdout=subprocess.DEVNULL)
    meta = json.loads((work / "tape-info.json").read_text())
    engine = meta["saves"][0]["engine_commit"]
    if not engine:
        raise SystemExit(f"{tape['key']}: archive has no engine commit")
    tree = export(engine, CACHE / "engines" / engine)
    sh("python3", ROOT / "tools/decision-audit/patch_recording_engine.py", tree)
    binary = cargo_example(tree, "audit_states", CACHE / "target-recording" / engine)
    full = work / "states-full.jsonl"
    sh("nice", "-n", "10", binary, work / "tape.json", full, stdout=subprocess.DEVNULL)
    keep = max_turn - meta["first"] + 2  # states for turns first..max_turn+1
    with open(full) as src, open(work / "states.jsonl", "w") as dst:
        for i, line in enumerate(src):
            if i >= keep:
                break
            dst.write(line)
    full.unlink()
    seat = [s["player"] for s in meta["human"]]
    if len(seat) != 1:
        raise SystemExit(f"{tape['key']}: expected one human seat, found {seat}")
    info = dict(source=str(source), source_sha256_16=digest, dir=str(work), engine=engine,
                recording_rules_version=rules_version(tree), seat=seat[0], first=meta["first"],
                last=meta["last"], faction=meta["human"][0]["faction"], states_through=max_turn)
    info_path.write_text(json.dumps(info, indent=1) + "\n")
    return info


def continue_arg(args) -> list[str]:
    """`seat_run --continue` for --seat-params: the control and the parameters in one run."""
    if not args.seat_params:
        return []
    return ["--continue", f"hard,params:{args.seat_params.resolve()}"]


def run_local(commit: str, prepared: dict, args) -> dict[str, tuple[list[dict], str]]:
    tree = export(commit, CACHE / "builds" / commit)
    binary = cargo_example(tree, "seat_run", CACHE / "target-measured" / commit)
    runs = CACHE / "runs" / f"{commit[:12]}-{int(time.time())}"
    runs.mkdir(parents=True)
    procs = {}
    jobs = max(1, min(args.jobs or len(prepared), len(prepared)))
    threads = max(1, args.threads // jobs) if args.jobs else max(2, args.threads // 2)
    pending = list(prepared.items())
    while pending or procs:
        while pending and len(procs) < jobs:
            key, (tape, info, turns) = pending.pop(0)
            log = open(runs / f"{key}.log", "w")
            argv = ["nice", "-n", "10", binary, "--tape", Path(info["dir"]) / "tape.json",
                    "--states", Path(info["dir"]) / "states.jsonl", "--out", runs / f"{key}.jsonl",
                    "--turns", ",".join(map(str, turns)), "--salts", args.salts, "--seat", info["seat"],
                    "--end-turn", args.end_turn, "--threads", threads, "--game", key]
            argv += continue_arg(args)
            procs[key] = (subprocess.Popen([str(a) for a in argv], cwd=ROOT, stdout=log,
                                           stderr=subprocess.STDOUT), log)
        key = next(iter(procs))
        proc, log = procs.pop(key)
        if proc.wait():
            raise SystemExit(f"seat_run failed for {key}; see {runs / (key + '.log')}")
        log.close()
    return {key: ([json.loads(x) for x in (runs / f"{key}.jsonl").read_text().splitlines()],
                  (runs / f"{key}.log").read_text()) for key in prepared}


def run_box(commit: str, prepared: dict, args) -> dict[str, tuple[list[dict], str]]:
    if commit != git("rev-parse", "HEAD"):
        raise SystemExit("--where box measures this checkout's HEAD; check out the commit first")
    with tempfile.TemporaryDirectory(prefix=".designer-seat-", dir=ROOT) as name:
        work = Path(name)
        rel = work.relative_to(ROOT)
        lines = ["tools/sim/build-slot.sh -- cargo build --release -q -p legatum-sim --example seat_run",
                 "mkdir -p designer-seat-out"]
        for key, (tape, info, turns) in prepared.items():
            (work / key).mkdir()
            for f in ("tape.json", "states.jsonl"):
                shutil.copy2(Path(info["dir"]) / f, work / key / f)
            lines.append(
                f"target/release/examples/seat_run --tape {rel}/{key}/tape.json --states {rel}/{key}/states.jsonl "
                f"--out designer-seat-out/{key}.jsonl --turns {','.join(map(str, turns))} --salts {args.salts} "
                f"--seat {info['seat']} --end-turn {args.end_turn} --game {key} "
                f"{' '.join(shlex.quote(a) for a in continue_arg(args))} > designer-seat-out/{key}.log 2>&1")
        out = work / "art"
        sh(ROOT / "tools/cloud/remote.sh", ".", "--with-uncommitted", "--artifact", "designer-seat-out",
           "--artifact-dir", out, "--label", "designer-seat", "--", "bash", "-euo", "pipefail", "-c",
           "; ".join(lines))
        res = out / "designer-seat-out"
        return {key: ([json.loads(x) for x in (res / f"{key}.jsonl").read_text().splitlines()],
                      (res / f"{key}.log").read_text()) for key in prepared}


def summarise(commit: str, prepared: dict, results: dict, args, elapsed: float) -> dict:
    tapes = []
    for key, (tape, info, turns) in prepared.items():
        rows, log = results[key]
        cells = []
        for t in TURNS:
            sel = [r for r in rows if r["branch_turn"] == t]
            if t not in turns:
                cells.append(dict(turn=t, wins=None, runs=0, rate=None, ci=None,
                                  note=f"tape ends at T{info['last']}"))
                continue
            wins = sum(bool(r["won"]) for r in sel)
            cells.append(dict(turn=t, wins=wins, runs=len(sel), rate=round(wins / len(sel), 4),
                              ci=wilson(wins, len(sel)),
                              stalled=sum(bool(r.get("stalled")) for r in sel),
                              median_end_turn=sorted(r["end_turn"] for r in sel)[len(sel) // 2]))
        measured = [c for c in cells if c["runs"]]
        score, ci = mean_ci(measured)
        migrated = [int(m) for m in re.findall(r"(\d+) migrated", log)]
        tapes.append(dict(key=key, label=tape["label"], set=tape["set"], game=tape["game"],
                          faction=info["faction"], seat=info["seat"], tape_turns=[info["first"], info["last"]],
                          recording_engine=info["engine"], recording_rules_version=info["recording_rules_version"],
                          source_sha256_16=info["source_sha256_16"],
                          states_changed_on_load=migrated[0] if migrated else None,
                          score=score, ci=ci, cells=cells))
    sets = {}
    for name in ("training", "held_out"):
        cells = [c for t in tapes if t["set"] == name for c in t["cells"] if c["runs"]]
        members = [t for t in tapes if t["set"] == name]
        if not members:
            continue
        # The set score weights tapes equally (mean of tape scores); its interval combines their variances.
        mean = sum(t["score"] for t in members) / len(members)
        var = sum(((t["ci"][1] - t["ci"][0]) / (2 * Z)) ** 2 for t in members) / len(members) ** 2
        h = Z * math.sqrt(var)
        sets[name] = dict(score=round(mean, 4), ci=[round(max(0, mean - h), 4), round(min(1, mean + h), 4)],
                          tapes=[t["key"] for t in members], runs=sum(c["runs"] for c in cells))
    tree_rules = int(re.search(r"pub const RULES_VERSION: u32 = (\d+);",
                               git("show", f"{commit}:crates/core/src/lib.rs")).group(1))
    defaults = hashlib.sha256(git("show", f"{commit}:crates/ai/src/params.rs").encode() + b"\n").hexdigest()
    return dict(schema=1, commit=commit, rules_version=tree_rules, ai_defaults_sha256=defaults,
                created=datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"), where=args.where,
                salts=args.salts, start_turns=TURNS, end_turn=args.end_turn,
                elapsed_minutes=round(elapsed / 60, 1), label=args.label, sets=sets, tapes=tapes)


# ---- chart -----------------------------------------------------------------------------------------

def font(size, bold=False):
    from PIL import ImageFont
    paths = (["/System/Library/Fonts/Supplemental/Arial Bold.ttf", "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf"]
             if bold else ["/System/Library/Fonts/Supplemental/Arial.ttf", "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"])
    for path in paths:
        if Path(path).exists():
            return ImageFont.truetype(path, size)
    return ImageFont.load_default()


INK, MUTED, GRID, BG = "#1f2328", "#59636e", "#e3e6ea", "#ffffff"
SET_COLOR = {"held_out": "#1f6feb", "training": "#bc4c00"}
TAPE_COLOR = ["#bc4c00", "#d4a72c", "#8250df", "#1f6feb", "#1a7f37", "#0598bc", "#cf222e"]


def tape_color(tapes: list[dict], i: int) -> str:
    """Retired tapes are grey; the rest take the palette in order."""
    if tapes[i]["set"] == "retired":
        return "#8c959f"
    return TAPE_COLOR[sum(t["set"] != "retired" for t in tapes[:i]) % len(TAPE_COLOR)]


def chart(path: Path = CHART) -> Path:
    from PIL import Image, ImageDraw
    rows = [json.loads(x) for x in HISTORY.read_text().splitlines() if x.strip()]
    latest = json.loads((ROOT / rows[-1]["file"]).read_text())
    W, H = 1080, 1920
    img = Image.new("RGB", (W, H), BG)
    d = ImageDraw.Draw(img)
    L, R = 120, W - 60
    d.text((L - 60, 50), "Designer-seat score", font=font(56, True), fill=INK)
    d.text((L - 60, 125), "Hard in his seat from T2-T80 of his games; higher is better",
           font=font(30), fill=MUTED)
    held, train = latest["sets"]["held_out"], latest["sets"]["training"]
    for i, (name, s) in enumerate((("Held out", held), ("Training", train))):
        x = L - 60 + i * 480
        color = SET_COLOR["held_out" if i == 0 else "training"]
        d.text((x, 190), name, font=font(32, True), fill=color)
        d.text((x, 230), f"{100 * s['score']:.1f}%", font=font(96, True), fill=color)
        d.text((x, 345), f"95% CI {100 * s['ci'][0]:.0f}-{100 * s['ci'][1]:.0f}%", font=font(28), fill=MUTED)
    d.text((L - 60, 395), f"{latest['commit'][:9]}  rules v{latest['rules_version']}  "
           f"{latest['salts']} runs per start  {latest['created'][:10]}", font=font(26), fill=MUTED)

    def axes(y0, h, xs, xlabels, title, ymax):
        d.text((L - 60, y0 - 70), title, font=font(36, True), fill=INK)
        for v in range(0, ymax + 1, 25 if ymax > 50 else 10):
            y = y0 + h - v / ymax * h
            d.line([(L, y), (R, y)], fill=GRID, width=2)
            d.text((L - 14, y), f"{v}%", font=font(24), fill=MUTED, anchor="rm")
        for x, lab in zip(xs, xlabels):
            d.text((x, y0 + h + 18), lab, font=font(24), fill=MUTED, anchor="mt")
        return lambda v: y0 + h - min(v * 100, ymax) / ymax * h

    # Panel 1: the headline over landings.
    y0, h = 540, 480
    n = len(rows)
    xs = [L + 40 + (R - L - 80) * (i / max(1, n - 1)) for i in range(n)] if n > 1 else [(L + R) / 2]
    ymax = 100
    Y = axes(y0, h, xs, [r["commit"][:7] if n <= 6 else "" for r in rows], "Score per landing", ymax)
    for name in ("training", "held_out"):
        pts = [(x, Y(r["sets"][name]["score"])) for x, r in zip(xs, rows)]
        for x, r in zip(xs, rows):
            lo, hi = r["sets"][name]["ci"]
            d.line([(x, Y(lo)), (x, Y(hi))], fill=SET_COLOR[name], width=3)
        if len(pts) > 1:
            d.line(pts, fill=SET_COLOR[name], width=6)
        for x, y in pts:
            d.ellipse([x - 11, y - 11, x + 11, y + 11], fill=SET_COLOR[name])
    lx = L
    for name, lab in (("held_out", "held out"), ("training", "training")):
        d.rectangle([lx, y0 + h + 62, lx + 26, y0 + h + 88], fill=SET_COLOR[name])
        d.text((lx + 36, y0 + h + 75), lab, font=font(26), fill=INK, anchor="lm")
        lx += 220

    # Panel 2: the latest takeover curves per tape.
    y0, h = 1250, 440
    xs = [L + 40 + (R - L - 80) * i / (len(TURNS) - 1) for i in range(len(TURNS))]
    Y = axes(y0, h, xs, [f"T{t}" for t in TURNS], "Win rate by start turn (latest)", 100)
    for i, t in enumerate(latest["tapes"]):
        color = tape_color(latest["tapes"], i)
        pts = [(xs[j], Y(c["rate"])) for j, c in enumerate(t["cells"]) if c["runs"]]
        width = {"held_out": 6, "retired": 2}.get(t["set"], 3)
        if len(pts) > 1:
            d.line(pts, fill=color, width=width)
        for x, y in pts:
            d.ellipse([x - 8, y - 8, x + 8, y + 8], fill=color)
    for i, t in enumerate(latest["tapes"]):
        color = tape_color(latest["tapes"], i)
        x = L - 60 + (i % 2) * 480
        y = y0 + h + 70 + (i // 2) * 40
        d.rectangle([x, y - 12, x + 26, y + 12], fill=color)
        tag = {"held_out": " (held out)", "retired": " (retired)"}.get(t["set"], "")
        d.text((x + 36, y), f"{t['label']}{tag} {100 * t['score']:.0f}%", font=font(26), fill=INK, anchor="lm")
    path.parent.mkdir(parents=True, exist_ok=True)
    img.save(path, optimize=True)
    return path


# ---- main ------------------------------------------------------------------------------------------

def main() -> None:
    if sys.argv[1:2] == ["chart"]:
        print(chart())
        return
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--commit", default="HEAD", help="commit whose engine and AI are measured (default HEAD)")
    p.add_argument("--where", choices=["local", "box"], default="local")
    p.add_argument("--salts", type=int, default=48)
    p.add_argument("--end-turn", type=int, default=260)
    p.add_argument("--tapes", default="", help="comma list of tape keys (default all)")
    p.add_argument("--threads", type=int, default=os.cpu_count() or 8)
    p.add_argument("--label", default="", help="short note for the history row, e.g. 'control'")
    p.add_argument("--out", type=Path, help="JSON path (default docs/sim-results/designer-seat/<date>-<sha>.json)")
    p.add_argument("--no-history", action="store_true", help="don't append to history.jsonl or redraw the chart")
    p.add_argument("--jobs", type=int, default=0, help="tapes run at once (default all)")
    p.add_argument("--seat-params", type=Path,
                   help="AiParams JSON for the seat: also play Hard with it, paired with the control")
    args = p.parse_args()
    commit = git("rev-parse", args.commit)
    keys = set(filter(None, args.tapes.split(",")))
    chosen = [t for t in TAPES if not keys or t["key"] in keys]
    start = time.time()
    prepared = {}
    for tape in chosen:
        info = prepare(tape, max(TURNS))
        turns = [t for t in TURNS if info["first"] <= t <= info["last"]]
        prepared[tape["key"]] = (tape, info, turns)
        print(f"{tape['key']}: T{info['first']}-T{info['last']} recorded on {info['engine'][:9]} "
              f"(rules v{info['recording_rules_version']}), start turns {turns}", flush=True)
    results = (run_local if args.where == "local" else run_box)(commit, prepared, args)
    if args.seat_params:
        paired(commit, prepared, results, args, time.time() - start)
        return
    summary = summarise(commit, prepared, results, args, time.time() - start)
    out = args.out or RESULTS / f"{summary['created'][:10]}-{commit[:9]}.json"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(summary, indent=1) + "\n")
    if not args.no_history and not keys:
        row = dict(commit=commit, rules_version=summary["rules_version"], created=summary["created"],
                   label=args.label, file=str(out.resolve().relative_to(ROOT)),
                   sets=summary["sets"], tapes={t["key"]: t["score"] for t in summary["tapes"]})
        with open(HISTORY, "a") as f:
            f.write(json.dumps(row) + "\n")
        chart()
    for name, s in summary["sets"].items():
        print(f"{name}: {100 * s['score']:.1f}% [{100 * s['ci'][0]:.1f}, {100 * s['ci'][1]:.1f}]")
    for t in summary["tapes"]:
        cells = " ".join(f"T{c['turn']}:{c['wins']}/{c['runs']}" if c["runs"] else f"T{c['turn']}:-" for c in t["cells"])
        print(f"  {t['label']:<16} {t['set']:<9} {100 * t['score']:5.1f}%  {cells}")
    print(f"wrote {out} in {summary['elapsed_minutes']} min")


def paired(commit: str, prepared: dict, results: dict, args, elapsed: float) -> None:
    """--seat-params: summarise the control and the parameters arm from one paired run."""
    arms = {"hard": "control", f"params:{args.seat_params.resolve()}": "params"}
    summaries = {}
    for brain, name in arms.items():
        rows = {key: ([r for r in rows if r.get("brain") == brain], log)
                for key, (rows, log) in results.items()}
        summary = summarise(commit, prepared, rows, args, elapsed)
        summary["seat_params"] = json.loads(args.seat_params.read_text()) if name == "params" else None
        summaries[name] = summary
    base = args.out or RESULTS / f"{summaries['control']['created'][:10]}-{commit[:9]}-paired.json"
    base.parent.mkdir(parents=True, exist_ok=True)
    base.write_text(json.dumps(summaries["control"], indent=1) + "\n")
    other = base.with_name(base.stem + "-params.json")
    other.write_text(json.dumps(summaries["params"], indent=1) + "\n")
    for name, s in summaries["control"]["sets"].items():
        a, b = s["score"], summaries["params"]["sets"][name]["score"]
        print(f"{name}: control {100 * a:.1f}%  params {100 * b:.1f}%  delta {100 * (b - a):+.1f} pp")
    for ta, tb in zip(summaries["control"]["tapes"], summaries["params"]["tapes"]):
        print(f"  {ta['label']:<16} {ta['set']:<9} control {100 * ta['score']:5.1f}%  "
              f"params {100 * tb['score']:5.1f}%  delta {100 * (tb['score'] - ta['score']):+.1f} pp")
    print(f"wrote {base} and {other} in {round(elapsed / 60, 1)} min")


if __name__ == "__main__":
    main()
