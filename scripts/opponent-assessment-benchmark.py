#!/usr/bin/env python3
"""Prepare observer-only baseline; run and compare matched opponent-assessment suites.

prepare-baseline --revision master --destination /tmp/legatum-opponent-baseline
run --binary /path/to/legatum-sim --output /tmp/opponent/base --label baseline
compare /tmp/opponent/base/*-baseline-hard.json /tmp/opponent/branch/*-branch-hard.json

Run suites sequentially, each with nice -n 15 and four worker threads. Build each
revision once with CARGO_BUILD_JOBS=4 cargo build --release -p legatum-sim, preserving
its binary before rebuilding the other revision. No worktree or branch is modified
by baseline preparation. Its source manifest records the exact archived revision.
"""
import argparse
import collections
import io
import json
from pathlib import Path
import shutil
import statistics
import subprocess
import tarfile

ROOT = Path(__file__).resolve().parents[1]


def replace_once(path, old, new):
    text = path.read_text()
    if text.count(old) != 1:
        raise ValueError(f"Expected one instrumentation anchor in {path}: {old!r}")
    path.write_text(text.replace(old, new, 1))


def prepare(args):
    revision = subprocess.check_output(
        ["git", "rev-parse", args.revision], cwd=ROOT, text=True).strip()
    destination = args.destination.resolve()
    destination.mkdir(parents=True, exist_ok=False)
    archive = subprocess.check_output(["git", "archive", revision, "Cargo.toml", "Cargo.lock", "crates", "scripts"], cwd=ROOT)
    with tarfile.open(fileobj=io.BytesIO(archive)) as source:
        source.extractall(destination, filter="data")
    # Refresh source mtimes so Cargo inspects files when reusing a target directory.
    for path in (destination / "crates").rglob("*"):
        if path.is_file() and (path.suffix == ".rs" or path.name == "Cargo.toml"):
            path.touch()
    sim = destination / "crates/sim/src"
    shutil.copyfile(ROOT / "crates/sim/src/opponent_telemetry.rs", sim / "opponent_telemetry.rs")
    with (sim / "lib.rs").open("a") as output:
        output.write("\npub mod opponent_telemetry;\n")
    replace_once(sim / "telemetry.rs", "    pub strategy: &'static str,",
                 "    pub strategy: &'static str,\n    pub opponent_assessment: &'static str,")
    replace_once(sim / "telemetry.rs", "pub const DEFINITIONS: Definitions = Definitions {",
                 "pub const DEFINITIONS: Definitions = Definitions {\n    opponent_assessment: legatum_sim::opponent_telemetry::DEFINITION,")
    replace_once(sim / "telemetry.rs", "pub struct Collector {",
                 "pub struct Collector {\n    pub opponents: legatum_sim::opponent_telemetry::Observer,")
    replace_once(sim / "telemetry.rs", "        GameTelemetry {\n",
                 "        GameTelemetry {\n            opponent_assessment: Some(self.opponents.seats.clone()),\n")
    replace_once(sim / "telemetry.rs", "pub struct GameTelemetry {",
                 'pub struct GameTelemetry {\n    #[serde(default, skip_serializing_if = "Option::is_none")]\n'
                 '    pub opponent_assessment: Option<BTreeMap<u8, legatum_sim::opponent_telemetry::Counters>>,')
    replace_once(sim / "runner.rs", "        let events = match caught(|| s.step(&orders)) {",
                 "        telemetry.opponents.before(&s.state);\n        let events = match caught(|| s.step(&orders)) {")
    replace_once(sim / "runner.rs", "        observe_events(\n            turn,",
                 "        telemetry.opponents.after(&events);\n        observe_events(\n            turn,")
    replace_once(sim / "openings.rs", "    pub phase_switches: Vec<PhaseSwitch>,",
                 "    pub phase_switches: Vec<PhaseSwitch>,\n    pub first_walls_turn: Option<u32>,")
    replace_once(sim / "openings.rs", "            phase_switches: Vec::new(),",
                 "            phase_switches: Vec::new(),\n            first_walls_turn: None,")
    replace_once(sim / "openings.rs", "                Event::WarDeclared { relation }",
                 "                Event::ProductionCompleted { owner, item: legatum_core::ProductionItem::Building { building }, .. }\n"
                 "                    if *owner == seat && data::building_family(*building) == legatum_core::BuildingKind::Walls => {\n"
                 "                    self.first_walls_turn.get_or_insert(turn);\n                }\n"
                 "                Event::WarDeclared { relation }")
    manifest = {"baseline_revision": revision, "observer_source": str(ROOT),
                "modified_files": ["crates/sim/src/" + name for name in
                                   ("opponent_telemetry.rs", "lib.rs", "telemetry.rs", "runner.rs", "openings.rs")],
                "behavior_changes": False}
    (destination / "opponent-baseline-manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    print(json.dumps(manifest, indent=2))


def run(args):
    args.output.mkdir(parents=True, exist_ok=True)
    binary = str(args.binary.resolve())
    common = ["nice", "-n", "15", binary, "--games", str(args.games), "--threads", "4",
              "--players", "8", "--difficulty", "hard",
              "--out-dir", str(args.output.resolve())]
    suites = [("hard", 5400001, []),
              ("v6", 3610001, ["--proxy", "0", "--rotate-proxy", "--proxy-version", "6"])]
    for name, seed, extra in suites:
        command = common + ["--seed", str(seed), "--label", f"{args.label}-{name}"] + extra
        print(json.dumps({"command": command}), flush=True)
        subprocess.run(command, check=True)


def summarize(report):
    games = report["games"]
    observed = [game for game in games if (game.get("telemetry") or {}).get("opponent_assessment") is not None]
    counters = collections.Counter()
    reasons = collections.Counter()
    wins = collections.Counter()
    exposures = collections.Counter()
    proxy_wins = proxy_games = 0
    for game in games:
        winners = set(game["winner_players"])
        for seat in game["players"]:
            faction = seat["faction"]
            exposures[faction] += 1
            wins[faction] += seat["id"] in winners
            for switch in seat.get("strategy_switches", []):
                reasons[switch["reason"]] += 1
            if seat["brain"] == "proxy":
                proxy_games += 1
                proxy_wins += seat["id"] in winners
    for game in observed:
        for seat in game["telemetry"]["opponent_assessment"].values():
            counters.update(seat)
    rates = {faction: wins[faction] / n for faction, n in sorted(exposures.items())}
    return {"games": len(games), "observed_games": len(observed),
            "panics": sum(game["outcome"] == "panic" for game in games),
            "stalls": sum(game["outcome"] == "stalled" for game in games),
            "mean_turns": statistics.mean(game["turns"] for game in games),
            "faction_win_rates": rates,
            "faction_ratio": max(rates.values()) / min(rates.values()) if rates and min(rates.values()) else None,
            "assault_metrics": dict(sorted(counters.items())),
            "strategy_switches": sum(reasons.values()), "switch_reasons": dict(reasons.most_common()),
            "v6_games": proxy_games, "v6_wins": proxy_wins,
            "v6_win_rate": proxy_wins / proxy_games if proxy_games else None}


def compare(args):
    baseline, candidate = [json.loads(path.read_text()) for path in (args.baseline, args.candidate)]
    # Matching seed alone is insufficient if seats/factions/difficulties drifted.
    def identities(report):
        return [(g["seed"], [(p["id"], p["faction"], p["brain"], p["difficulty"]) for p in g["players"]])
                for g in report["games"]]
    if identities(baseline) != identities(candidate):
        raise ValueError("Reports do not contain identical ordered seeds, factions, brains and difficulties")
    if baseline.get("metric_definitions", {}).get("opponent_assessment") != candidate.get("metric_definitions", {}).get("opponent_assessment"):
        raise ValueError("Assault metric definitions differ")
    base, branch = summarize(baseline), summarize(candidate)
    metrics = ["failed_walled_assault_turns", "failed_celts_assault_turns", "soft_target_captures"]
    changes = {key: branch["assault_metrics"].get(key, 0) - base["assault_metrics"].get(key, 0) for key in metrics}
    changes["mean_turns_percent"] = 100 * (branch["mean_turns"] / base["mean_turns"] - 1)
    changes["stalls"] = branch["stalls"] - base["stalls"]
    changes["faction_ratio"] = (branch["faction_ratio"] - base["faction_ratio"]
                                if base["faction_ratio"] is not None and branch["faction_ratio"] is not None else None)
    changes["v6_win_rate"] = (branch["v6_win_rate"] - base["v6_win_rate"]
                              if base["v6_win_rate"] is not None and branch["v6_win_rate"] is not None else None)
    print(json.dumps({"baseline": base, "branch": branch, "changes": changes}, indent=2))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(required=True)
    prepare_parser = sub.add_parser("prepare-baseline")
    prepare_parser.add_argument("--revision", default="master")
    prepare_parser.add_argument("--destination", required=True, type=Path)
    prepare_parser.set_defaults(action=prepare)
    run_parser = sub.add_parser("run")
    run_parser.add_argument("--binary", required=True, type=Path)
    run_parser.add_argument("--output", required=True, type=Path)
    run_parser.add_argument("--label", required=True)
    run_parser.add_argument("--games", type=int, default=400)
    run_parser.set_defaults(action=run)
    compare_parser = sub.add_parser("compare")
    compare_parser.add_argument("baseline", type=Path)
    compare_parser.add_argument("candidate", type=Path)
    compare_parser.set_defaults(action=compare)
    args = parser.parse_args()
    args.action(args)


if __name__ == "__main__":
    main()
