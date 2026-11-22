"""Tests for the evolution harness: python3 -m unittest discover -s tools/evolve -p 'test_*.py'

The end-to-end tests drive the real harness against a fake simulator and a fake Claude CLI
(no Rust build, no model call) and check the run log against docs/archive/ai-evolution-log.md, then
load it with the film/dashboard reader (tools/film/lib/runlog.mjs) when node is available.
"""

from __future__ import annotations

import gzip
import json
import math
import os
import re
import shutil
import subprocess
import sys
import tempfile
import textwrap
import unittest
from unittest.mock import patch
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
sys.path.insert(0, str(HERE))

import cmaes  # noqa: E402
import coach  # noqa: E402
import evolve  # noqa: E402
import genome  # noqa: E402
import metrics  # noqa: E402
import uncertainty  # noqa: E402
import runlog  # noqa: E402

KNOBS = {"expansion": 50, "military": 50, "science": 50, "economy": 50, "aggression": 50}
BUILTIN = {"schema": 1, "name": "builtin", "source": "test", "factions": {"Japan": dict(KNOBS), "Celts": dict(KNOBS, military=60)}}

FAKE_SIM = r'''#!/usr/bin/env python3
import json, sys, os, zlib
from pathlib import Path
a = sys.argv[1:]
def opt(name, default=None):
    return a[a.index(name) + 1] if name in a else default
if "--print-ai-genes" in a:
    print("[]"); sys.exit(0)
if "--print-ai-params" in a:
    print(json.dumps(BUILTIN)); sys.exit(0)
if "--replay" in a:
    out = Path(opt("--out")); out.mkdir(parents=True, exist_ok=True)
    gid = Path(opt("--replay")).name.split(".")[0]
    inc = {"incidents": [{"id": "I000001", "behavior": "research_idle", "turn": 12, "player": 1, "faction": "Celts",
                          "evidence": "research empty", "consecutive_turns": 1}]}
    (out / f"{gid}.incidents.json").write_text(json.dumps(inc))
    print(json.dumps({"game_id": gid, "incidents": str(out / f"{gid}.incidents.json"), "incident_count": 1}))
    sys.exit(0)
games, seed, label = int(opt("--games")), int(opt("--seed")), opt("--label")
params = json.loads(Path(opt("--ai-params")).read_text())
opp = json.loads(Path(opt("--opponent-params")).read_text()) if "--opponent-params" in a else None
proxy = "--proxy" in a
knob = lambda p: sum(k["aggression"] + k["military"] for k in p.get("factions", {}).values())
strength = knob(params)
archs = ["rush", "wide", "science", "turtle", "economic", "generalist"]
out = []
for i in range(games):
    s = seed + i
    ps = []
    pseat = i % 8 if proxy else None
    for seat in range(8):
        if seat == pseat:
            brain = "proxy"
        elif opp and (seat + i) % 2 == 1:
            brain = "scripted@" + opp["name"]
        else:
            brain = "scripted@" + params["name"]
        h = zlib.crc32(f"{s}-{seat}-{brain}".encode())
        ps.append({"id": seat, "faction": ["Japan", "Celts"][seat % 2], "team": seat, "brain": brain,
                   "archetype": archs[(h + strength // 40) % len(archs)], "difficulty": "Hard", "alive": seat != 7,
                   "vp": h % 10, "cities": 3, "techs": 5, "population": 9})
    roll = zlib.crc32(f"w{s}".encode()) % 1000
    if proxy:
        winner = pseat if roll > min(900, strength * 2) else (pseat + 1) % 8
    else:
        winner = roll % 8
    out.append({"index": i, "seed": str(s), "outcome": "vp", "winner_team": winner, "winner_players": [winner],
                "winner_factions": [], "turns": 90 + roll % 40, "players": ps,
                "eliminations": [{"player": 7, "turn": 60}], "wall_ms": 1, "panic": None, "determinism": None,
                "telemetry": {"proxy": {"ai_incidents": {"research_idle": roll % 5}, "diagnostics": {"first_capture": 40}} if proxy else None}})
    if "--archive" in a:
        arch = Path(opt("--archive-dir")); d = arch / "2026" / "09"; d.mkdir(parents=True, exist_ok=True)
        gid = f"native-{s}-{label}"
        (d / f"{gid}.tape.json.zst").write_bytes(b"x")
        with open(arch / "index.jsonl", "a") as f:
            f.write(json.dumps({"game_id": gid, "seed": str(s), "created_at": "2026-09-28T00:00:00Z"}) + "\n")
wins = sum(1 for g in out if proxy and g["winner_players"] == [g["index"] % 8])
report = {"meta": {"proxy_version": 2 if proxy else None, "proxy": {"start": opt("--proxy-start")} if proxy else None, "first_setup": {"legacy_all_war": proxy}, "source": {"commit": "test"}},
          "summary": {"games": games, "proxy": {"win_rate": wins / games, "ai_cities_alive": {"100": {"mean": 12.5, "observations": games}}} if proxy else None},
          "games": out,
          "ai_analysis": {"rates": [{"faction": "Celts", "behavior": "research_idle", "attacker_faction": None, "incidents": strength % 7, "opportunities": 100}]} if proxy else None}
od = Path(opt("--out-dir")); od.mkdir(parents=True, exist_ok=True)
(od / f"2026-09-28-{label}.json").write_text(json.dumps(report))
(od / f"2026-09-28-{label}.md").write_text("# fake")
'''

FAKE_CLAUDE = r'''#!/usr/bin/env python3
import json, sys
text = sys.stdin.read()
if "FAIL_COACH" in text:
    print("boom", file=sys.stderr); sys.exit(3)
facts = json.loads(text.split("FACTS:\n", 1)[1])
g = facts["games"][0]
seat = next(p["id"] for p in g["players"] if p["brain"] != "proxy")
out = {
  "observations": [
    {"kind": "weakness", "behavior": "research_idle", "archetype": None,
     "evidence": [{"game_id": g["game_id"], "turn": 12, "player": seat, "detail": "idle research"},
                  {"game_id": "invented-game", "turn": 5, "player": 1, "detail": "not real"}],
     "metric_change": {"name": "proxy_win_rate", "before": 0.123, "after": 0.456}, "confidence": "medium"},
    {"kind": "emerged", "behavior": "Moonshot play", "archetype": None, "evidence": [], "metric_change": None, "confidence": "high"}],
  "dominant_strategy": None,
  "proposals": [{"kind": "new_behavior", "description": "Buy a defender when the Proxy's army is adjacent.",
                 "expected_effect": "Fewer captures.", "confidence": "high",
                 "evidence": [{"game_id": g["game_id"], "player": seat, "turns": [12, 99999]}]}],
  "summary": "The tuned AI researches less idly. It still loses most games to the Proxy."}
print(json.dumps({"type": "result", "is_error": False, "num_turns": 2, "total_cost_usd": 0.01,
                  "result": json.dumps(out), "structured_output": out}))
'''


def write_exe(path: Path, body: str) -> str:
    path.write_text(body)
    path.chmod(0o755)
    return str(path)


def fake_game(i, winners, proxy_seat=None, label="scripted@c", archs=None, turns=100):
    archs = archs or ["Rush", "Wide", "Wide", "Science", "Turtle", "Economic", "Generalist", "Rush"]
    return {"i": i, "seed": str(1000 + i), "turns": turns, "outcome": "vp", "winners": winners,
            "players": [{"id": s, "faction": ["Japan", "Celts"][s % 2], "brain": "proxy" if s == proxy_seat else label,
                         "arch": archs[s], "alive": True, "vp": s, "cities": 3, "elim": None} for s in range(8)],
            "proxy_seat": proxy_seat, "incidents": i, "incident_counts": None, "first_capture": None}


class CmaesTest(unittest.TestCase):
    def test_minimizes_a_shifted_sphere(self):
        es = cmaes.CMAES([0.0] * 6, 0.5, 10, seed=3)
        for _ in range(250):
            xs = es.ask()
            es.tell(xs, [sum((v - 1.5) ** 2 for v in x) for x in xs])
        self.assertLess(sum((v - 1.5) ** 2 for v in es.mean), 1e-6)

    def test_state_round_trips_and_asks_the_same_candidates(self):
        es = cmaes.CMAES([0.2] * 5, 0.3, 8, seed=9)
        xs = es.ask()
        es.tell(xs, [sum(x) for x in xs])
        clone = cmaes.CMAES.from_dict(json.loads(json.dumps(es.to_dict())))
        self.assertEqual(clone.ask(), es.ask())

    def test_eigendecomposition(self):
        vals, vecs = cmaes._jacobi_eigh([[4.0, 1.0], [1.0, 3.0]])
        for k, lam in enumerate(vals):
            v = [vecs[0][k], vecs[1][k]]
            av = [4 * v[0] + v[1], v[0] + 3 * v[1]]
            self.assertAlmostEqual(av[0], lam * v[0], places=9)
            self.assertAlmostEqual(av[1], lam * v[1], places=9)


class GenomeTest(unittest.TestCase):
    def test_decision_ranges_match_rust_schema(self):
        source = (ROOT / "crates/ai/src/params.rs").read_text()
        block = source.split("decision_params! {", 1)[1].split("\n}", 1)[0]
        entries = re.findall(
            r'^\s*(\w+):\s*-?\d+,\s*(-?\d+),\s*(-?\d+),\s*".*?";\s*$',
            block,
            re.MULTILINE,
        )
        rust_ranges = {name: (int(minimum), int(maximum)) for name, minimum, maximum in entries}
        missing = sorted(set(rust_ranges) - set(genome.DECISION_RANGES))
        self.assertEqual(
            genome.DECISION_RANGES,
            rust_ranges,
            f"decision schema mismatch; missing from DECISION_RANGES: {missing}",
        )

    def test_genes_decode_to_knob_only_params(self):
        genes = genome.genes_from(BUILTIN)
        self.assertEqual([g["name"] for g in genes][:2], ["Celts.expansion", "Celts.military"])
        self.assertEqual(len(genes), 10)
        x = genome.encode({g["name"]: g["default"] for g in genes}, genes)
        values = genome.decode(x, genes)
        self.assertEqual(values["Celts.military"], 60)
        p = genome.params_file(values, "t", "test")
        self.assertEqual(set(p), {"schema", "name", "source", "factions"})
        self.assertEqual(genome.decode([2.0] + [-1.0] * 9, genes)["Celts.expansion"], 100)
        self.assertGreater(genome.boundary_distance([1.5, 0.5]), 0)

    def test_pillar6_guard_rejects_anything_but_knobs(self):
        with self.assertRaises(ValueError):
            genome.params_file({"Japan.food_yield": 3}, "t", "s")
        for bad in ({**BUILTIN, "yields": {}},
                    {**BUILTIN, "factions": {"Japan": {**KNOBS, "vision": 3}}},
                    {**BUILTIN, "factions": {"Japan": {**KNOBS, "military": 101}}}):
            with self.assertRaises(ValueError):
                genome.check_params(bad)

    def test_decision_genes_can_be_scoped_to_one_faction(self):
        params = genome.params_file({"decision.siege_window": 6}, "t", "test", faction_target="Sparta")
        self.assertNotIn("decisions", params)
        self.assertEqual(params["faction_overrides"], {"Sparta": {"siege_window": 6}})
        genome.check_params(params)
        with self.assertRaises(ValueError):
            genome.check_params({**params, "faction_overrides": {"Sparta": {"siege_window": 99}}})


class MetricsTest(unittest.TestCase):
    def test_candidate_metrics_and_fitness(self):
        proxy = [fake_game(0, [0], proxy_seat=0), fake_game(1, [3], proxy_seat=1)]
        league = [fake_game(2, [2]), fake_game(3, [])]
        league[0]["players"][2]["brain"] = "scripted@c"
        m = metrics.candidate_metrics(proxy, league, "scripted@c", {"ai_cities_t100": 9.0}, {})
        self.assertEqual(m["proxy_win_rate"], 0.5)
        self.assertEqual(m["league_win_rate"], 1.0)  # one decided game, won by our seat
        self.assertEqual(m["archetype"], "Rush")  # most common real strategy, not Generalist
        self.assertTrue(0 < m["archetype_entropy"] <= 1)
        f, parts = metrics.fitness(m, {})
        self.assertAlmostEqual(f, sum(parts.values()), places=3)
        collapsed = dict(m, archetype_entropy=0.1, dominant="Rush", stalls=1)
        f2, parts2 = metrics.fitness(collapsed, {})
        self.assertLess(f2, f)
        self.assertLess(parts2["pen_entropy"], 0)
        self.assertLess(parts2["pen_dominance"], 0)

    def test_dominance_ignores_generalist_and_needs_samples(self):
        m = {"archetypes": ["Rush", "Wide", "Generalist"],
             "ahead_rate": [[None, 0.8, 0.7], [0.2, None, 0.4], [0.3, 0.6, None]],
             "pairs": [[0, 30, 30], [30, 0, 30], [30, 30, 0]]}
        self.assertEqual(metrics.dominant_archetype(m, 20), "Rush")
        self.assertIsNone(metrics.dominant_archetype(m, 31))
        m["archetypes"] = ["Generalist", "Wide", "Rush"]
        self.assertIsNone(metrics.dominant_archetype(m, 20))

    def test_entropy(self):
        self.assertIsNone(metrics.entropy({}))
        self.assertEqual(metrics.entropy({"Rush": 5}), 0)
        self.assertAlmostEqual(metrics.entropy({a: 1 for a in metrics.ARCHETYPES}), 1.0)
        self.assertEqual(metrics.quantile([1, 2, 3, 4], 0.5), 2.5)


class UncertaintyTest(unittest.TestCase):
    def test_full_fitness_bootstrap_clusters_seats_and_pairs_draws(self):
        games = [fake_game(i, [0 if i % 2 else 1], proxy_seat=0) for i in range(100)]
        cfg = {k: 0 for k in metrics.DEFAULT_FITNESS if k.startswith("w_")}
        cfg["w_proxy"] = 1
        u = uncertainty.bootstrap(games, "scripted@c", cfg)
        self.assertAlmostEqual(u["fitness_se"], 0.05, delta=0.008)
        candidate = {"games": games, "fitness": 0.5, "uncertainty": u}
        paired = uncertainty.paired(candidate, candidate)
        self.assertEqual(paired["ci"], [0, 0])
        with self.assertRaises(ValueError):
            uncertainty.paired(candidate, {**candidate, "games": games[1:]})
        after = uncertainty.bootstrap(games, "scripted@c", cfg, sizes={"proxy": 400})
        self.assertLess(after["fitness_se"], u["fitness_se"] * 0.6)

    def test_proxy_identical_results_are_inconclusive(self):
        games = [fake_game(i, [0], proxy_seat=0) for i in range(32)]
        result = uncertainty.proxy_comparison(games, games)
        self.assertEqual(result["improvement"], 0)
        self.assertLess(result["ci95"][0], 0)
        self.assertGreater(result["ci95"][1], 0)

    def test_balance_does_not_hide_zero_win_factions_or_missing_data(self):
        games = [fake_game(i, [0]) for i in range(16)]
        b = uncertainty.balance(games, "scripted@c", repeats=32)
        self.assertIsNone(b["faction_ratio"])
        self.assertIsNone(b["faction_ratio_upper95"])
        self.assertEqual(b["winner_entropy"], 0)
        self.assertEqual(uncertainty.verdict({})["decision"], "don't-adopt")

    def test_adoption_requires_standard_validation_and_all_guardrails(self):
        names = metrics.ARCHETYPES[:-1]
        b = {"games": 2000, "factions": dict.fromkeys(genome.FACTIONS, {}),
             "faction_ratio_upper95": 1.4, "winner_entropy_ci95": [0.6, 0.8],
             "dominant": None, "stalls": 0,
             "matchup_matrix": {"archetypes": names, "pairs": [[40]*6 for _ in names],
                                "ahead_rate": [[0.5]*6 for _ in names]}}
        row = {"proxy": {"ci95": [0.02, 0.08]}, "league": {"ci95": [0.52, 0.62]},
               "balance": {"evolved": b}}
        self.assertEqual(uncertainty.verdict({"standard": row})["decision"], "adopt")
        self.assertEqual(uncertainty.verdict({})["decision"], "don't-adopt")
        b["dominant"] = "Wide"
        self.assertEqual(uncertainty.verdict({"standard": row})["decision"], "don't-adopt")


class SettingsTest(unittest.TestCase):
    def test_nice_default_inheritance_and_denied_fallback(self):
        a = evolve.parse_args([])
        self.assertTrue(a.nice)
        with patch("evolve.os.getpriority", return_value=0), patch("evolve.os.setpriority") as change:
            evolve.apply_nice(a)
            change.assert_called_once_with(os.PRIO_PROCESS, 0, 10)
        with patch("evolve.os.getpriority", return_value=0), patch("evolve.os.setpriority", side_effect=PermissionError("denied")), patch("evolve.os.cpu_count", return_value=16):
            a.threads = 16
            evolve.apply_nice(a)
            self.assertEqual(a.threads, 8)
            self.assertEqual(a.niceness["error"], "denied")
        with patch("evolve.os.setpriority") as change:
            evolve.apply_nice(evolve.parse_args(["--no-nice"]))
            change.assert_not_called()

    def test_mixed_gene_ranges_round_trip_and_reject_rule_fields(self):
        p = {**BUILTIN, "decisions": {"recover_hp": 50}}
        catalog = [{"name": "decision.recover_hp", "min": 25, "max": 90, "default": 50, "description": "test"}]
        genes = genome.genes_from(p, catalog)
        values = {g["name"]: g["default"] for g in genes}
        self.assertEqual(genome.decode(genome.encode(values, genes), genes), values)
        self.assertEqual(genome.params_file(values, "x", "test")["decisions"], p["decisions"])
        for bad in ({"vision": 8}, {"cost": 1}, {"recover_hp": 24}, {"recover_hp": True}):
            with self.assertRaises(ValueError):
                genome.check_params({**BUILTIN, "decisions": bad})


class CoachTest(unittest.TestCase):
    def facts(self):
        g = fake_game(0, [0], proxy_seat=0)
        g["game_id"] = "native-1-a"
        before = {"proxy_win_rate": 0.8, "archetype_share": {"Rush": 0.1}, "incident_rates": {"research_idle": {"incidents": 5, "opportunities": 100}}}
        after = {"proxy_win_rate": 0.6, "archetype_share": {"Rush": 0.5, "Generalist": 0.5}, "incident_rates": {"research_idle": {"incidents": 20, "opportunities": 100}}}
        digest = {"native-1-a": {"available": True, "counts": {"research_idle": 1}, "examples": [
            {"id": "I1", "behavior": "research_idle", "turn": 12, "player": 1, "faction": "Celts", "evidence": "idle", "consecutive_turns": 1}]}}
        return coach.build_facts("r", 10, 0, before, after, [(g, "loss")], digest, {})

    def test_validation_keeps_only_supported_claims(self):
        facts = self.facts()
        self.assertEqual(facts["dominant_candidate"]["archetype"], "Rush")
        raw = {"observations": [
            {"kind": "weakness", "behavior": "research_idle", "evidence": [
                {"game_id": "native-1-a", "turn": 12, "player": 1, "detail": "ok"},
                {"game_id": "native-1-a", "turn": 500, "player": 1, "detail": "turn past the end"},
                {"game_id": "native-9-z", "turn": 3, "player": 1, "detail": "unknown game"}],
             "metric_change": {"name": "proxy_win_rate", "before": 9, "after": 9}, "confidence": "high"},
            {"kind": "weakness", "behavior": "made_up", "evidence": [], "confidence": "low"}],
            "proposals": [{"kind": "exploit", "description": "x", "expected_effect": "y", "confidence": "high", "evidence": []}],
            "dominant_strategy": {"archetype": "Rush", "counter_proposal": "Science economies outscale it."},
            "summary": "  Two   sentences. "}
        fields, dropped = coach.validate(raw, facts)
        self.assertEqual(len(fields["observations"]), 1)
        o = fields["observations"][0]
        self.assertEqual(len(o["evidence"]), 1)
        self.assertEqual(o["metric_change"], {"name": "proxy_win_rate", "before": 0.8, "after": 0.6})
        self.assertEqual(fields["proposals"][0]["confidence"], "medium")  # high needs cited games
        self.assertEqual(fields["dominant_strategy"]["share"], 0.5)
        self.assertEqual(fields["summary"], "Two sentences.")
        self.assertGreaterEqual(len(dropped), 3)

    def test_rule_based_fallback(self):
        doc = coach.rule_based(self.facts(), "timeout")
        self.assertIn("LLM unavailable: timeout", doc["summary"])
        behaviors = {o["behavior"] for o in doc["observations"]}
        self.assertIn("research_idle", behaviors)
        self.assertIn("Rush play", behaviors)
        for o in doc["observations"]:
            self.assertTrue(o["evidence"])

    def test_claude_provider_and_failure_fallback(self):
        with tempfile.TemporaryDirectory() as tmp:
            os.environ["LEGATUM_CLAUDE"] = write_exe(Path(tmp, "claude"), FAKE_CLAUDE)
            try:
                facts = self.facts()
                doc = coach.coach(facts, "claude", Path(tmp), 60, "m", Path(tmp, "log"))
                self.assertIsNone(doc["coach"]["error"])
                self.assertEqual(doc["coach"]["provider"], "claude")
                self.assertEqual(len(doc["observations"]), 1)
                self.assertEqual(doc["proposals"][0]["evidence"][0]["turns"], [12])
                facts["games"][0]["why"] = "FAIL_COACH"
                doc = coach.coach(facts, "claude", Path(tmp), 60, "m", Path(tmp, "log"))
                self.assertIn("exited 3", doc["coach"]["error"])
                self.assertTrue(doc["coach"]["provider"].startswith("rules"))
                self.assertTrue(doc["summary"])
            finally:
                del os.environ["LEGATUM_CLAUDE"]

    def test_sampling_is_bounded_and_unique(self):
        games = []
        for i in range(40):
            g = fake_game(i, [i % 8], proxy_seat=i % 3)
            g["game_id"] = f"g{i}"
            games.append(g)
        picked = coach.sample_games(games, 20)
        self.assertEqual(len(picked), 20)
        self.assertEqual(len({g["game_id"] for g, _ in picked}), 20)


class RunlogTest(unittest.TestCase):
    def test_append_read_and_truncate(self):
        with tempfile.TemporaryDirectory() as tmp:
            p = Path(tmp, "g.jsonl")
            for g in range(3):
                runlog.append_jsonl(p, {"gen": g})
            with p.open("a") as f:
                f.write('{"gen": 3, "t')  # torn line
            self.assertEqual([r["gen"] for r in runlog.read_jsonl(p)], [0, 1, 2])
            runlog.truncate_jsonl(p, lambda r: r["gen"] < 2)
            self.assertEqual(p.read_text().count("\n"), 2)
            runlog.write_gz_json(Path(tmp, "x.json.gz"), {"a": 1})
            self.assertEqual(runlog.read_gz_json(Path(tmp, "x.json.gz")), {"a": 1})


class EndToEndTest(unittest.TestCase):
    """The whole loop against a fake simulator: run log contract, resume, coach, readers."""

    def setUp(self):
        self.tmp = tempfile.mkdtemp(prefix="legatum-evolve-test-")
        self.sim = write_exe(Path(self.tmp, "fake-sim"), FAKE_SIM.replace("BUILTIN)", json.dumps(BUILTIN) + ")", 1))
        os.environ["LEGATUM_CLAUDE"] = write_exe(Path(self.tmp, "claude"), FAKE_CLAUDE)

    def tearDown(self):
        os.environ.pop("LEGATUM_CLAUDE", None)
        shutil.rmtree(self.tmp, ignore_errors=True)

    def args(self, *extra):
        return ["--sim", self.sim, "--out", str(Path(self.tmp, "results")), "--data", str(Path(self.tmp, "data")),
                "--population", "4", "--proxy-games", "8", "--league-games", "8", "--league-sample", "2",
                "--coach-every", "2", "--league-every", "2", "--bench-proxy-games", "8", "--bench-league-games", "4",
                "--validate-games", "8", "--balance-games", "8", "--bootstrap", "32", "--threads", "2", "--parallel", "2", "--no-film", "--no-queue", *extra]

    def test_incumbent_historic_score_is_discarded_and_league_count_is_exact(self):
        args = evolve.parse_args(self.args("--run-id", "incumbent", "--generations", "1", "--league-games", "14", "--league-sample", "3"))
        run = evolve.Run(args)
        values = {g["name"]: g["default"] for g in run.genes}
        run.state["best"] = {"id": "lucky-old-score", "fitness": 999, "values": values}
        run.add_league(0, "extra1", values, "test")
        run.add_league(0, "extra2", values, "test")
        rec, results, _, _ = run.generation(0)
        self.assertLess(run.state["best"]["fitness"], 2)
        self.assertEqual(rec["incumbent"]["id_before"], "lucky-old-score")
        self.assertTrue(all(r["metrics"]["league_games"] == 14 for r in results))
        self.assertTrue((run.data / "gens/g0/cincumbent.json.gz").exists())

    def test_rebuild_during_run_cannot_change_simulator_and_resume_rejects_it(self):
        args = evolve.parse_args(self.args("--run-id", "frozen", "--generations", "1"))
        run = evolve.Run(args)
        self.assertNotEqual(run.args.sim, self.sim)
        write_exe(Path(self.sim), "#!/usr/bin/env python3\nraise SystemExit(9)\n")
        report = run.sim("frozen-test", params=run.state["league"][0]["path"], games=2, seed=100001, proxy=True)
        self.assertEqual(len(report["games"]), 2)
        with self.assertRaisesRegex(SystemExit, "simulator changed"):
            evolve.Run(evolve.parse_args(self.args("--resume", str(run.run_dir))))

    def test_two_checkouts_cannot_share_candidate_caches(self):
        evolve.Run(evolve.parse_args(self.args("--run-id", "collision")))
        with self.assertRaisesRegex(SystemExit, "data directory already exists"):
            evolve.Run(evolve.parse_args(self.args("--run-id", "collision", "--out", str(Path(self.tmp, "other-results")))))

    def test_validation_failure_is_a_visible_dont_adopt_verdict(self):
        run = evolve.Run(evolve.parse_args(self.args("--run-id", "failed-validation")))
        with patch.object(run, "validate_final", side_effect=RuntimeError("forced validation failure")):
            run.finish(None)
        self.assertEqual(run.run_json["final_validation"]["verdict"]["decision"], "don't-adopt")
        digest = next(run.run_dir.glob("digest-*.md")).read_text()
        self.assertIn("forced validation failure", digest)
        self.assertNotIn("Nothing: no failed evaluations", digest)

    def test_run_log_contract_resume_and_readers(self):
        self.assertEqual(evolve.main(self.args("--run-id", "2026-01-02-0300", "--generations", "2")), 0)
        run = Path(self.tmp, "results/evolve-2026-01-02-0300")
        self.assertEqual(evolve.main(self.args("--resume", str(run), "--generations", "3")), 0)
        rj = json.loads((run / "run.json").read_text())
        self.assertEqual(rj["status"], "finished")
        for k in ("run_id", "started_at", "finished_at", "config", "baseline"):
            self.assertIn(k, rj)
        for k in ("population", "games_per_candidate", "speed", "seeds", "coach_every", "genes"):
            self.assertIn(k, rj["config"])
        self.assertEqual(rj["config"]["genes"][0].keys() >= {"name", "min", "max", "default", "description"}, True)
        self.assertEqual(set(rj["baseline"]) >= {"proxy_win_rate", "ai_cities_t150", "archetype_entropy"}, True)
        self.assertEqual(rj["config"]["proxy_version"], 2)
        gens = runlog.read_jsonl(run / "generations.jsonl")
        self.assertEqual([g["gen"] for g in gens], [0, 1, 2])  # resumed without duplicates
        need = {"gen", "t", "wall_s", "best", "mean_fitness", "fitness_p10", "fitness_p90", "sigma", "archetype_share",
                "winner_entropy", "elites", "genome_best", "gene_mean", "gene_spread"}
        for g in gens:
            self.assertTrue(need <= set(g), need - set(g))
            self.assertTrue({"id", "fitness", "proxy_win_rate", "league_win_rate", "faction_ratio", "archetype"} <= set(g["best"]))
            self.assertEqual(len(g["genome_best"]), 10)
            self.assertEqual(len(g["candidates"]), 4)
            self.assertTrue(all(c["fitness_se"] is not None for c in g["candidates"]))
            self.assertEqual(len(g["incumbent"]["comparisons"]), 4)
        # 0 is the built-in baseline; 1 closed the first (2-generation) run; 2 is a coach cycle.
        cps = sorted(int(p.stem) for p in (run / "checkpoints").glob("*.json"))
        self.assertEqual(cps, [0, 1, 2])
        cp = json.loads((run / "checkpoints/2.json").read_text())
        self.assertTrue({"gen", "genome", "benchmark", "showcase"} <= set(cp))
        self.assertTrue({"proxy_win_rate", "ai_survival_t150", "faction_win_rates", "archetype_win_rates", "matchup_matrix"} <= set(cp["benchmark"]))
        for s in cp["showcase"]:
            self.assertTrue(Path(s["archive_path"]).exists())
            self.assertTrue({"game_id", "archive_path", "seed", "why", "highlight_turns"} <= set(s))
        doc = json.loads((run / "coach/2.json").read_text())
        self.assertTrue({"gen", "observations", "proposals", "summary"} <= set(doc))
        self.assertEqual(doc["coach"]["provider"], "claude")
        self.assertEqual(len(doc["observations"]), 1)  # the invented behavior was dropped
        ev = doc["observations"][0]["evidence"]
        self.assertEqual(len(ev), 1)
        self.assertIn(ev[0]["game_id"], cp["benchmark"]["sample"]["game_ids"])
        events = runlog.read_jsonl(run / "events.jsonl")
        kinds = {e["kind"] for e in events}
        self.assertIn("league_add", kinds)
        self.assertTrue(kinds <= {"new_best", "archetype_takeover", "archetype_extinct", "league_add", "proposal_adopted", "proposal_rejected", "stall", "guardrail"})
        self.assertTrue(list(run.glob("digest-*.md")))
        best = json.loads((run / "best-params.json").read_text())
        genome.check_params(best)
        self.assertIn("final_validation", rj)
        self.assertEqual(set(rj["final_validation"]["speeds"]), {"standard"})
        self.assertEqual(rj["final_validation"]["verdict"]["decision"], "don't-adopt")
        self.assertEqual(rj["config"]["proxy_start"], "war")
        self.assertEqual(rj["config"]["league_start"], "peace")
        self.assertIn("**don't-adopt**", next(run.glob("digest-*.md")).read_text())
        state = json.loads(Path(rj["data_dir"], "state.json").read_text())
        self.assertEqual(best["factions"], genome.params_file(state["best"]["values"], "x", "x")["factions"])
        if shutil.which("node"):
            script = textwrap.dedent(f"""
                const {{ loadRun, facts }} = await import({json.dumps(str(ROOT / 'tools/film/lib/runlog.mjs'))});
                const f = facts(loadRun({json.dumps(str(run))}));
                console.log(JSON.stringify({{gens: f.generations.count, cps: f.checkpoints.length, coach: f.coach.length, ba: f.before_after}}));
            """)
            out = subprocess.run(["node", "--input-type=module", "-e", script], capture_output=True, text=True, check=True)
            got = json.loads(out.stdout)
            self.assertEqual((got["gens"], got["cps"], got["coach"]), (3, 3, 2))
            self.assertEqual(got["ba"]["before_gen"], 0)


if __name__ == "__main__":
    unittest.main()
