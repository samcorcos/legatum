#!/usr/bin/env python3
"""Evolve the scripted AI's decision parameters with CMA-ES against the Proxy and a league.

    python3 tools/evolve/evolve.py [--hours 7] [--population 6] ...   # new run
    python3 tools/evolve/evolve.py --resume docs/sim-results/evolve-<run-id>

See docs/archive/ai-evolution.md for the design, the overnight command and the budget, and
docs/archive/ai-evolution-log.md for the run log this writes. The genome combines faction personalities
with bounded tactical and macro decisions (pillar 6); fitness is the Proxy benchmark plus league games, with
guardrails for faction balance (pillar 8) and strategic diversity (pillar 10).
"""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import os
import signal
import resource
import shutil
import threading
import subprocess
import sys
import time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import cmaes  # noqa: E402
import coach  # noqa: E402
import genome  # noqa: E402
import metrics  # noqa: E402
import uncertainty  # noqa: E402
import runlog  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]
SEEDS = {"proxy": 100_001, "league": 500_001, "bench_proxy": 900_001, "bench_league": 920_001,
         "held_out_proxy": 950_001, "held_out_league": 970_001, "held_out_balance": 990_001}


class Stop(Exception):
    pass


def utc() -> str:
    return dt.datetime.now(dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def r3(x):
    return None if x is None else round(x, 3)


def apply_nice(args):
    error = None
    if args.nice and os.getpriority(os.PRIO_PROCESS, 0) < 10:
        try:
            os.setpriority(os.PRIO_PROCESS, 0, 10)
        except OSError as exc:
            # Restricted macOS sessions may deny setpriority even when lowering it.
            # Keep interactive headroom and record the actual policy, never pretend
            # that the requested OS priority was applied.
            error = str(exc)
            args.threads = min(args.threads, max(1, (os.cpu_count() or 2) // 2))
            args.parallel = min(args.parallel, args.threads)
            print(f"[evolve] niceness unavailable: {exc}; limiting sims to {args.threads} threads", file=sys.stderr)
    args.niceness = {"requested": args.nice, "effective": os.getpriority(os.PRIO_PROCESS, 0),
                     "error": error, "thread_cap": args.threads}


def parse_args(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--resume", type=Path, help="continue the run in this directory")
    ap.add_argument("--run-id", help="YYYY-MM-DD-HHMM (default: now, local time)")
    ap.add_argument("--label", default="", help="free text recorded in run.json (e.g. smoke)")
    ap.add_argument("--out", type=Path, default=ROOT / "docs/sim-results")
    ap.add_argument("--data", type=Path, default=Path.home() / "LegatumData/evolve")
    ap.add_argument("--sim", default=os.environ.get("LEGATUM_SIM", str(ROOT / "target/release/legatum-sim")))
    ap.add_argument("--population", type=int, default=6)
    ap.add_argument("--proxy-games", type=int, default=512, help="Proxy games per candidate per generation")
    ap.add_argument("--league-games", type=int, default=128, help="league games per candidate per generation")
    ap.add_argument("--league-sample", type=int, default=4, help="league members played per generation")
    ap.add_argument("--league-every", type=int, default=10, help="add the generation's best to the league every N")
    ap.add_argument("--generations", type=int, default=10_000, help="maximum generations")
    ap.add_argument("--hours", type=float, help="wall-clock budget; stops starting generations in time to validate")
    ap.add_argument("--coach-every", type=int, default=10, help="checkpoint + coach cycle every K generations")
    ap.add_argument("--no-coach", action="store_true", help="skip the LLM (checkpoints still use the rule-based coach)")
    ap.add_argument("--coach-provider", choices=["claude", "codex", "rules"], default="claude")
    ap.add_argument("--coach-model", default="claude-opus-5-5")
    ap.add_argument("--coach-timeout", type=int, default=1800)
    ap.add_argument("--coach-games", type=int, default=20)
    ap.add_argument("--bench-proxy-games", type=int, default=512)
    ap.add_argument("--bench-league-games", type=int, default=128)
    ap.add_argument("--validate-games", type=int, default=512, help="held-out Proxy and league games (0 skips validation)")
    ap.add_argument("--balance-games", type=int, default=2000, help="Peace self-play games per AI for pillars 8/10")
    ap.add_argument("--bootstrap", type=int, default=256, help="whole-game bootstrap replicates for fitness SE")
    ap.add_argument("--nice", action=argparse.BooleanOptionalAction, default=True,
                    help="lower scheduling priority to niceness 10 (inherited by sims and coach; default on)")
    ap.add_argument("--sigma", type=float, default=0.1, help="initial step size, as a fraction of each gene range")
    ap.add_argument("--seed", type=int, default=1, help="CMA-ES sampling seed")
    ap.add_argument("--threads", type=int, default=os.cpu_count() or 8)
    ap.add_argument("--parallel", type=int, default=2, help="simulator processes at once (threads are split)")
    ap.add_argument("--no-film", action="store_true", help="don't start tools/film/make-film.sh nightly and at the end")
    ap.add_argument("--no-queue", action="store_true", help="don't turn high-confidence proposals into Astra tasks")
    args = ap.parse_args(argv)
    for key in ("population", "proxy_games", "league_games", "league_sample", "generations", "coach_every",
                "bench_proxy_games", "bench_league_games", "threads", "parallel", "balance_games"):
        if getattr(args, key) <= 0:
            ap.error(f"--{key.replace('_', '-')} must be positive")
    if args.population < 4 or args.bootstrap < 32:
        ap.error("population must be >=4 and bootstrap replicates >=32")
    if args.league_games < 2 * args.league_sample or args.league_games % 2:
        ap.error("league games must be even and at least twice league-sample")
    if args.validate_games < 0 or args.validate_games >= 20000 or args.balance_games >= 20000:
        ap.error("validation game counts must fit their disjoint 20,000-seed blocks")
    if args.proxy_games >= 400000 or args.league_games >= 400000:
        ap.error("training batch exceeds its seed block")
    if not 0 < args.sigma <= 1 or (args.hours is not None and args.hours <= 0):
        ap.error("sigma must be in (0,1] and hours must be positive")
    return args


class Run:
    def __init__(self, args):
        self.args = args
        args.sim = str(Path(shutil.which(args.sim) or args.sim).resolve())
        self.t0 = time.time()
        self.lock = threading.RLock()
        self.coach_thread = None
        simulator_bytes = Path(args.sim).read_bytes()
        simulator_hash = hashlib.sha256(simulator_bytes).hexdigest()
        if args.resume:
            self.run_dir = args.resume.resolve()
            self.run_json = json.loads((self.run_dir / "run.json").read_text())
            self.data = Path(self.run_json["data_dir"])
            self.cfg = self.run_json["config"]
            if self.cfg.get("harness_version") != 2:
                raise SystemExit("v1 runs cannot resume under v2's changed genome/selection contract; start a new run")
            if self.cfg["simulator_sha256"] != simulator_hash:
                raise SystemExit("simulator changed; start a new run so cached and new games use identical rules/AI")
            self.freeze_simulator(simulator_bytes)
            self.state = json.loads((self.data / "state.json").read_text())
            for key in ("validate_games", "balance_games", "bootstrap"):
                setattr(args, key, self.cfg[key])
            self.run_json.pop("final_validation", None)
            self.genes = self.cfg["genes"]
            self.run_json["status"] = "running"
            self.run_json.pop("finished_at", None)
            keep = lambda r: r.get("gen", 0) < self.state["gen"]
            runlog.truncate_jsonl(self.run_dir / "generations.jsonl", keep)
            runlog.truncate_jsonl(self.run_dir / "events.jsonl", keep)
            self.log(f"resuming {self.run_id} at generation {self.state['gen']}")
        else:
            run_id = args.run_id or dt.datetime.now().strftime("%Y-%m-%d-%H%M")
            self.run_dir = (args.out / f"evolve-{run_id}").resolve()
            if (self.run_dir / "run.json").exists():
                raise SystemExit(f"{self.run_dir} already exists; use --resume")
            self.data = (args.data / run_id).resolve()
            for d in ("checkpoints", "coach"):
                (self.run_dir / d).mkdir(parents=True, exist_ok=True)
            # A second checkout can choose the same minute-based run id. Never let
            # its candidates/state reuse this run's caches, even with the same binary.
            self.data.parent.mkdir(parents=True, exist_ok=True)
            try:
                self.data.mkdir()
            except FileExistsError as exc:
                raise SystemExit(f"data directory already exists: {self.data}; use --resume or a new --run-id") from exc
            self.freeze_simulator(simulator_bytes)
            builtin = genome.builtin_params(args.sim)
            self.genes = genome.genes_from(builtin, genome.builtin_catalog(args.sim))
            self.cfg = self.new_config(args, run_id)
            self.cfg["simulator_sha256"] = simulator_hash
            self.run_json = {
                "run_id": run_id, "started_at": utc(), "status": "running", "config": self.cfg,
                "baseline": None, "data_dir": str(self.data), "definitions": metrics.DEFINITIONS,
                "source_revision": self.git_rev(), "label": args.label or None,
            }
            (self.run_dir / "generations.jsonl").touch()
            (self.run_dir / "events.jsonl").touch()
            self.write_params(builtin, self.data / "params/builtin.json")
            league = [{"name": "builtin", "path": str(self.data / "params/builtin.json"), "gen": None, "why": "the current Hard AI"}]
            for frozen in sorted((ROOT / "crates/ai/params").glob("*.json")):
                p = json.loads(frozen.read_text())
                genome.check_params(p)
                p["name"] = f"frozen-{frozen.stem}".replace("_", "-")
                dest = self.data / f"params/{p['name']}.json"
                self.write_params(p, dest)
                league.append({"name": p["name"], "path": str(dest), "gen": None, "why": f"frozen AI version {frozen.name}"})
            x0 = genome.encode({g["name"]: g["default"] for g in self.genes}, self.genes)
            es = cmaes.CMAES(x0, args.sigma, args.population, seed=args.seed)
            self.state = {
                "gen": 0, "es": es.to_dict(), "pending": None, "best": None, "elites": {}, "league": league,
                "trackers": {"leader": None, "pending": None, "streak": 0, "seen": [], "extinct": [], "zero": {}, "guardrail": None},
                "last_checkpoint": None, "last_coach_gen": None, "digest_date": dt.date.today().isoformat(),
                "queued": [], "proposals": [], "failures": [], "gen_seconds": [], "coach_calls": [],
            }
            runlog.write_json(self.run_dir / "run.json", self.run_json)
            (self.data / "README.txt").write_text(
                f"Evolution run {run_id}: simulator scratch, candidate params, benchmark reports and the archived\n"
                "showcase/evidence tapes under archive/. The run log in docs/sim-results cites these tapes;\n"
                "keep this directory (no cleanup deletes it).\n")
            self.log(f"new run {run_id}: {len(self.genes)} genes, population {args.population}, data in {self.data}")
        self.es = cmaes.CMAES.from_dict(self.state["es"])
        self.cfg["nice"] = getattr(args, "niceness", {"requested": args.nice, "effective": os.getpriority(os.PRIO_PROCESS, 0)})
        self.cfg["threads"], self.cfg["parallel"] = args.threads, args.parallel

    # ------------------------------------------------------------------ setup helpers

    @property
    def run_id(self) -> str:
        return self.run_json["run_id"]

    def freeze_simulator(self, contents):
        """A rebuild in the main checkout must not change the rules midway through a run."""
        binary = self.data / "bin/legatum-sim"
        binary.parent.mkdir(parents=True, exist_ok=True)
        if not binary.exists():
            binary.write_bytes(contents)
            binary.chmod(0o755)
        elif binary.read_bytes() != contents:
            raise SystemExit("run's frozen simulator differs from the requested binary; start a new run")
        self.args.sim = str(binary)

    def new_config(self, a, run_id) -> dict:
        return {
            "harness_version": 2,
            "population": a.population, "evaluations_per_generation": a.population + 1,
            "selection": "CMA-ES rank weights; incumbent re-evaluated on same new seeds/opponents; promotion requires paired lower CI > 0 (Bonferroni over population)",
            "proxy_start": "war", "league_start": "peace", "bootstrap": a.bootstrap,
            "balance_games": a.balance_games,
            "validation_speeds": ["standard"],
            "games_per_candidate": a.proxy_games + a.league_games,
            "proxy_games": a.proxy_games, "league_games": a.league_games, "league_sample": a.league_sample,
            "league_every": a.league_every,
            "speed": "standard",
            "difficulty": "hard",
            "seeds": {
                "rule": "generation g plays Proxy seeds proxy+g*proxy_games.. and league seeds league+g*league_games.., the same for every candidate in it (common random numbers); benchmarks use fixed bench seeds; held-out seeds are used only for the final validation",
                **{k: str(v) for k, v in SEEDS.items()},
            },
            "coach_every": a.coach_every,
            "coach": "off" if a.no_coach else a.coach_provider,
            "coach_model": a.coach_model if a.coach_provider == "claude" else None,
            "coach_games": a.coach_games,
            "bench": {"proxy_games": a.bench_proxy_games, "league_games": a.bench_league_games},
            "validate_games": a.validate_games,
            "sigma0": a.sigma, "cma_seed": a.seed, "optimizer": "tools/evolve/cmaes.py (pure-Python CMA-ES)",
            "threads": a.threads, "parallel": a.parallel,
            "nice": getattr(a, "niceness", {"requested": a.nice}),
            "fitness": metrics.DEFAULT_FITNESS,
            "genes": self.genes,
            "proxy_version": None,
            "hours": a.hours,
            "film": not a.no_film, "queue_proposals": not a.no_queue,
        }

    def git_rev(self) -> str | None:
        r = subprocess.run(["git", "-C", str(ROOT), "rev-parse", "--short", "HEAD"], capture_output=True, text=True)
        return r.stdout.strip() or None

    def log(self, *msg):
        line = f"[evolve {time.strftime('%H:%M:%S')}] " + " ".join(str(m) for m in msg)
        print(line, file=sys.stderr, flush=True)
        with (self.data / "evolve.log").open("a") as f:
            f.write(line + "\n")

    @staticmethod
    def write_params(p: dict, path: Path):
        genome.check_params(p)
        runlog.write_json(path, p)

    def save_state(self):
        with self.lock:
            self.state["es"] = self.es.to_dict()
            runlog.write_json(self.data / "state.json", self.state, indent=None)

    def save_run_json(self):
        runlog.write_json(self.run_dir / "run.json", self.run_json)

    def event(self, gen, kind, detail, game_id=None):
        e = {"gen": gen, "t": utc(), "kind": kind, "detail": detail}
        if game_id:
            e["game_id"] = game_id
        runlog.append_jsonl(self.run_dir / "events.jsonl", e)
        self.log(f"event {kind}: {detail}")

    # ------------------------------------------------------------------ simulator

    def sim(self, label, *, params, games, seed, opponent=None, proxy=False, archive=None, keep=None, threads=None):
        """Run one legatum-sim batch and return its parsed report (the big JSON is not kept)."""
        a = self.args
        out = self.data / "slots" / label
        out.mkdir(parents=True, exist_ok=True)
        cmd = [a.sim, "--games", str(games), "--seed", str(seed),
               "--threads", str(threads or a.threads), "--label", label, "--out-dir", str(out),
               "--ai-params", str(params)]
        cmd += ["--proxy", "0", "--rotate-proxy", "--proxy-start", "war", "--difficulty", "hard"] if proxy else ["--difficulty", "hard"]
        if opponent:
            cmd += ["--opponent-params", str(opponent)]
        if archive:
            Path(archive).mkdir(parents=True, exist_ok=True)
            cmd += ["--archive", "--archive-dir", str(archive)]
        err = ""
        for _attempt in range(2):
            started = time.time()
            r = subprocess.run(cmd, capture_output=True, text=True)
            reports = [p for p in out.glob(f"*-{label}.json") if p.stat().st_mtime >= started - 1 and p.stat().st_size]
            if reports:
                path = max(reports, key=lambda p: p.stat().st_mtime)
                report = json.loads(path.read_text())
                if r.returncode != 0 or len(report.get("games", [])) != games:
                    raise RuntimeError(f"incomplete/failed simulator batch {label}: exit {r.returncode}, expected {games} games")
                if [int(g["seed"]) for g in report["games"]] != list(range(seed, seed + games)):
                    raise RuntimeError(f"wrong seed block in simulator batch {label}")
                meta = report["meta"]
                self.cfg.setdefault("simulator_source", meta.get("source"))
                if proxy and (meta.get("proxy_version") != 2 or (meta.get("proxy") or {}).get("start") != "war"):
                    raise RuntimeError("Proxy report must record version 2 and all-war start")
                if not proxy and (meta.get("first_setup") or {}).get("legacy_all_war", False):
                    raise RuntimeError("league must use current Peace-start rules")
                if keep:
                    runlog.write_gz_json(Path(keep), report)
                path.write_text("")  # scratch: overwritten every generation, never accumulates
                return report
            err = f"exit {r.returncode}: {r.stderr[-800:]}"
        raise RuntimeError(f"legatum-sim {label} failed twice: {err}")

    def league_members(self, gen):
        league = self.state["league"]
        rest = league[1:]
        n = max(0, min(self.cfg["league_sample"], len(league)) - 1)
        if rest:
            k = gen % len(rest)
            rest = (rest[k:] + rest[:k])[:n]
        return [league[0]] + rest

    # ------------------------------------------------------------------ one generation

    def eval_candidate(self, gen, k, params_path, name, members, threads):
        cache = self.data / f"gens/g{gen}/c{k}.json.gz"
        if cache.exists():
            cached = runlog.read_gz_json(cache)
            if not cached["failed"]:
                return cached
        cfg = self.cfg
        label = f"scripted@{name}"
        try:
            rep = self.sim(f"c{k}-proxy", params=params_path, proxy=True, games=cfg["proxy_games"],
                           seed=SEEDS["proxy"] + gen * cfg["proxy_games"], threads=threads)
            if cfg["proxy_version"] is None:
                cfg["proxy_version"] = rep["meta"].get("proxy_version")
            proxy_games = metrics.compact_games(rep)
            psum = metrics.report_summary(rep)
            league_games = []
            offset = 0
            pairs, extra = divmod(cfg["league_games"] // 2, len(members))
            for j, m in enumerate(members):
                per = 2 * (pairs + (j < extra))
                rep = self.sim(f"c{k}-l{j}", params=params_path, opponent=m["path"], games=per,
                               seed=SEEDS["league"] + gen * cfg["league_games"] + offset, threads=threads)
                offset += per
                for g in metrics.compact_games(rep):
                    g["opponent"] = m["name"]
                    league_games.append(g)
            m = metrics.candidate_metrics(proxy_games, league_games, label, psum, cfg["fitness"])
            f, parts = metrics.fitness(m, cfg["fitness"])
            result = {"id": name, "k": k, "fitness": f, "parts": parts, "metrics": m, "psum": psum,
                      "games": proxy_games + league_games, "failed": None,
                      "uncertainty": uncertainty.bootstrap(proxy_games + league_games, label, cfg["fitness"], cfg["bootstrap"])}
        except Exception as e:  # noqa: BLE001 - a failed candidate gets the worst fitness, the run goes on
            result = {"id": name, "k": k, "fitness": -10.0, "parts": {}, "metrics": None, "psum": None,
                      "games": [], "failed": f"{type(e).__name__}: {str(e)[:500]}"}
        runlog.write_gz_json(cache, result)
        if not result["failed"]:
            self.log(f"evaluated {name}: fitness {result['fitness']:.4f}, SE {result['uncertainty']['fitness_se']:.4f}")
        return result

    def generation(self, gen):
        t0 = time.time()
        cpu0 = resource.getrusage(resource.RUSAGE_CHILDREN)
        st = self.state
        if (SEEDS["proxy"] + (gen + 1) * self.cfg["proxy_games"] > SEEDS["league"]
                or SEEDS["league"] + (gen + 1) * self.cfg["league_games"] > SEEDS["bench_proxy"]):
            raise RuntimeError("training seed range exhausted; refusing to use benchmark/held-out seeds")
        if st["pending"] and st["pending"]["gen"] == gen:
            xs = st["pending"]["xs"]
        else:
            xs = self.es.ask()
            st["pending"] = {"gen": gen, "xs": xs}
            self.save_state()
        values = [genome.decode(x, self.genes) for x in xs]
        names, paths = [], []
        for k, v in enumerate(values):
            name = f"cand-g{gen}-c{k}"
            path = self.data / f"params/g{gen}/c{k}.json"
            self.write_params(genome.params_file(v, name, f"{self.run_id} generation {gen} candidate {k}"), path)
            names.append(name)
            paths.append(path)
        incumbent_values = (st["best"] or {}).get("values", {g["name"]: g["default"] for g in self.genes})
        incumbent_name = f"incumbent-g{gen}"
        incumbent_path = self.data / f"params/g{gen}/incumbent.json"
        self.write_params(genome.params_file(incumbent_values, incumbent_name, "fresh incumbent evaluation"), incumbent_path)
        members = self.league_members(gen)
        par = max(1, min(self.args.parallel, len(xs) + 1, self.args.threads))
        threads = max(1, self.args.threads // par)
        with ThreadPoolExecutor(max_workers=par) as pool:
            incumbent_future = pool.submit(self.eval_candidate, gen, "incumbent", incumbent_path, incumbent_name, members, threads)
            results = list(pool.map(lambda k: self.eval_candidate(gen, k, paths[k], names[k], members, threads), range(len(xs))))
            incumbent = incumbent_future.result()
        for r in results:
            if r["failed"]:
                st["failures"].append({"gen": gen, "id": r["id"], "error": r["failed"]})
                self.log(f"candidate {r['id']} failed: {r['failed']}")
        if incumbent["failed"]:
            raise RuntimeError(f"incumbent evaluation failed: {incumbent['failed']}")
        if all(r["failed"] for r in results):
            raise RuntimeError("all candidates failed; refusing to update CMA-ES from failed simulations")
        fits = [r["fitness"] for r in results]
        # CMA-ES already uses ranks internally; feed explicit ordinal ranks so a
        # numerically lucky score can never receive disproportionate recombination weight.
        order = sorted(range(len(xs)), key=lambda k: (-fits[k] + genome.boundary_distance(xs[k]), k))
        ranks = [order.index(k) for k in range(len(xs))]
        self.es.tell(xs, ranks)
        comparisons = []
        for r in results:
            if not r["failed"]:
                d = uncertainty.paired(r, incumbent, alpha=0.05 / len(results))
                comparisons.append({"id": r["id"], **d})
        eligible = {c["id"] for c in comparisons if c["ci"][0] > 0}
        promoted = max((r for r in results if r["id"] in eligible), key=lambda r: r["fitness"], default=None)
        old_id = (st["best"] or {}).get("id", "builtin")
        st["best"] = {"id": promoted["id"] if promoted else old_id,
                      "fitness": (promoted or incumbent)["fitness"], "gen": gen,
                      "values": values[promoted["k"]] if promoted else incumbent_values}
        if promoted:
            self.event(gen, "new_best", f"{promoted['id']} beat re-evaluated {old_id} on paired seeds (family-wise alpha 0.05)")
        st["pending"] = None
        wall = time.time() - t0
        rec = self.record(gen, results, values, members, wall)
        cpu1 = resource.getrusage(resource.RUSAGE_CHILDREN)
        rec["cpu_s"] = round(cpu1.ru_utime + cpu1.ru_stime - cpu0.ru_utime - cpu0.ru_stime, 2)
        rec["incumbent"] = {"id_before": old_id, "id_after": st["best"]["id"],
                            "fresh_fitness": incumbent["fitness"], "fitness_se": incumbent["uncertainty"]["fitness_se"],
                            "promoted": promoted is not None, "comparisons": comparisons}
        rec["candidates"] = [{"id": r["id"], "rank": ranks[r["k"]], "fitness": r["fitness"],
                              **{k: v for k, v in r.get("uncertainty", {}).items() if k != "samples"}}
                             for r in results]
        runlog.append_jsonl(self.run_dir / "generations.jsonl", rec)
        self.after_generation(gen, rec, results, values, paths)
        st["gen_seconds"].append(round(wall, 1))
        self.log(f"gen {gen}: best {rec['best']['fitness']} ({rec['best']['id']}, Proxy {rec['best']['proxy_win_rate']}, "
                 f"league {rec['best']['league_win_rate']}, {rec['best']['archetype']}), mean {rec['mean_fitness']}, "
                 f"sigma {rec['sigma']}, {wall:.0f}s")
        return rec, results, values, paths

    def record(self, gen, results, values, members, wall):
        ok = [r for r in results if r["metrics"]]
        best = max(results, key=lambda r: r["fitness"])
        fits = [r["fitness"] for r in results]
        counts: dict[str, int] = {}
        winners: dict[str, int] = {}
        for r in ok:
            for a, c in r["metrics"]["archetype_counts"].items():
                counts[a] = counts.get(a, 0) + c
            for g in r["games"]:
                for p in g["players"]:
                    if p["id"] in g["winners"] and p["brain"].startswith("scripted") and p["arch"]:
                        winners[p["arch"]] = winners.get(p["arch"], 0) + 1
        total = sum(counts.values()) or 1
        bm = best["metrics"] or {}
        mean_x = genome.decode(self.es.mean, self.genes)
        spread = self.es.stddevs()
        return {
            "gen": gen, "t": utc(), "wall_s": round(wall, 1),
            "best": {"id": best["id"], "fitness": best["fitness"], "proxy_win_rate": r3(bm.get("proxy_win_rate")),
                     "league_win_rate": r3(bm.get("league_win_rate")), "faction_ratio": r3(bm.get("faction_ratio")),
                     "archetype": bm.get("archetype"), "parts": best["parts"],
                     "fitness_se": best.get("uncertainty", {}).get("fitness_se"),
                     "ai_survival_t150": r3(bm.get("ai_survival_t150")), "archetype_entropy": r3(bm.get("archetype_entropy"))},
            "mean_fitness": r3(sum(fits) / len(fits)),
            "fitness_p10": r3(metrics.quantile(fits, 0.1)), "fitness_p90": r3(metrics.quantile(fits, 0.9)),
            "sigma": r3(self.es.sigma),
            "archetype_share": {a: round(counts.get(a, 0) / total, 3) for a in metrics.ARCHETYPES},
            "winner_entropy": r3(metrics.entropy(winners)),
            "elites": {a: {"id": e["id"], "fitness": e["fitness"]} for a, e in self.updated_elites(gen, results, values).items()},
            "genome_best": values[best["k"]],
            "gene_mean": {g["name"]: mean_x[g["name"]] for g in self.genes},
            "gene_spread": {g["name"]: round(s * (g["max"] - g["min"]), 1) for g, s in zip(self.genes, spread)},
            "league": [m["name"] for m in members],
            "failures": sum(1 for r in results if r["failed"]),
        }

    def updated_elites(self, gen, results, values):
        """Refresh each observed niche from this generation; never compare stale scores."""
        elites = self.state["elites"]
        for r in results:
            a = (r["metrics"] or {}).get("archetype")
            if a and (a not in elites or elites[a]["gen"] < gen or r["fitness"] > elites[a]["fitness"]):
                elites[a] = {"id": r["id"], "fitness": r["fitness"], "gen": gen, "values": values[r["k"]]}
        return elites

    def after_generation(self, gen, rec, results, values, paths):
        st, tr = self.state, self.state["trackers"]
        best = max(results, key=lambda r: r["fitness"])
        share = rec["archetype_share"]
        top = max(share, key=share.get)
        tr["streak"] = tr["streak"] + 1 if tr["pending"] == top else 1
        tr["pending"] = top
        if tr["leader"] is None:
            tr["leader"] = top
        elif top != tr["leader"] and tr["streak"] >= 2 and share[top] >= 0.25:
            self.event(gen, "archetype_takeover", f"{top} became the most common strategy ({round(share[top] * 100)}% of candidate seats in generation {gen}), taking over from {tr['leader']}")
            tr["leader"] = top
        for a, s in share.items():
            if s >= 0.05 and a not in tr["seen"]:
                tr["seen"].append(a)
            if s >= 0.05 and a in tr["extinct"]:
                tr["extinct"].remove(a)
            tr["zero"][a] = tr["zero"].get(a, 0) + 1 if s == 0 else 0
            if a in tr["seen"] and a not in tr["extinct"] and tr["zero"][a] >= 2:
                self.event(gen, "archetype_extinct", f"No candidate seat played {a} in generations {gen - 1} and {gen}")
                tr["extinct"].append(a)
        stalls = sum((r["metrics"] or {}).get("stalls", 0) + (r["metrics"] or {}).get("panics", 0) for r in results)
        if stalls:
            total = sum(len(r["games"]) for r in results)
            self.event(gen, "stall", f"{stalls} of {total} games stalled or panicked in generation {gen}")
        bm = best["metrics"] or {}
        issues = []
        if bm.get("dominant"):
            issues.append(f"{bm['dominant']} is ahead of every other archetype in the best candidate's games")
        if bm.get("archetype_entropy") is not None and bm["archetype_entropy"] < self.cfg["fitness"]["min_entropy"]:
            issues.append(f"archetype entropy {round(bm['archetype_entropy'], 2)} is under the {self.cfg['fitness']['min_entropy']} floor")
        key = "; ".join(issues) or None
        if key != tr["guardrail"]:
            if key:
                self.event(gen, "guardrail", f"Best candidate {best['id']}: {key}")
            tr["guardrail"] = key
        if gen > 0 and self.cfg["league_every"] and gen % self.cfg["league_every"] == 0 and not best["failed"]:
            self.add_league(gen, f"league-g{gen}", values[best["k"]], f"generation {gen}'s best ({best['id']}, fitness {best['fitness']})")

    def add_league(self, gen, name, values, why, replace_prefix=None):
        league = self.state["league"]
        if replace_prefix:
            league[:] = [m for m in league if not m["name"].startswith(replace_prefix)]
        path = self.data / f"params/{name}.json"
        self.write_params(genome.params_file(values, name, f"{self.run_id}: {why}"), path)
        league.append({"name": name, "path": str(path), "gen": gen, "why": why})
        while len(league) > 10:
            old = next((m for m in league if m["name"].startswith("league-g")), None)
            if not old:
                break
            league.remove(old)
        self.event(gen, "league_add", f"{name} joined the league: {why}")

    # ------------------------------------------------------------------ checkpoints and coach

    def benchmark(self, gen, values, name):
        cfg = self.cfg["bench"]
        path = self.data / f"params/{name}.json"
        self.write_params(genome.params_file(values, name, f"{self.run_id} checkpoint {gen}"), path)
        arch = self.data / f"archive/g{gen}"
        prep = self.sim(f"bench-g{gen}-proxy", params=path, proxy=True, games=cfg["proxy_games"], seed=SEEDS["bench_proxy"],
                        archive=arch, keep=self.data / f"bench/g{gen}-proxy.json.gz")
        lrep = self.sim(f"bench-g{gen}-league", params=path, opponent=self.state["league"][0]["path"], games=cfg["league_games"],
                        seed=SEEDS["bench_league"], archive=arch, keep=self.data / f"bench/g{gen}-league.json.gz")
        index = {}
        for line in (arch / "index.jsonl").read_text().splitlines() if (arch / "index.jsonl").exists() else []:
            try:
                meta = json.loads(line)
            except json.JSONDecodeError:
                continue
            index.setdefault(str(meta["seed"]), []).append(meta["game_id"])
        def attach(games, kind):
            for g in games:
                ids = index.get(g["seed"], [])
                found = [i for i in ids if next(arch.rglob(f"{i}.tape.json*"), None)]
                g["game_id"] = found[-1] if found else f"unarchived-{kind}-{g['seed']}"
                path = next(arch.rglob(f"{g['game_id']}.tape.json*"), None)
                g["archive_path"] = str(path) if path else None
                g["kind"] = kind
            return games
        # Proxy and league benchmarks use different seed ranges, so a seed names one game per checkpoint.
        pg = attach(metrics.compact_games(prep), "proxy")
        lg = attach(metrics.compact_games(lrep), "league")
        psum = metrics.report_summary(prep)
        label = f"scripted@{name}"
        m = metrics.candidate_metrics(pg, lg, label, psum, self.cfg["fitness"])
        m["incident_rates"] = psum["incident_rates"]
        wins: dict[str, list[int]] = {}
        for g in pg + lg:
            for p in g["players"]:
                if p["brain"] == label and p["arch"]:
                    w = wins.setdefault(p["arch"], [0, 0])
                    w[0] += p["id"] in g["winners"]
                    w[1] += 1
        bench = {
            "proxy_win_rate": r3(m["proxy_win_rate"]), "ai_survival_t150": r3(m["ai_survival_t150"]),
            "ai_cities_t100": r3(m["ai_cities_t100"]), "ai_cities_t150": r3(m["ai_cities_t150"]),
            "league_win_rate_vs_builtin": r3(m["league_win_rate"]),
            "faction_win_rates": m["faction_win_rates"], "faction_ratio": r3(m["faction_ratio"]),
            "archetype_win_rates": {a: round(w / n, 3) for a, (w, n) in sorted(wins.items())},
            "archetype_share": m["archetype_share"], "archetype_entropy": r3(m["archetype_entropy"]),
            "matchup_matrix": m["matchup"], "dominant_archetype": m["dominant"],
            "games": {"proxy": len(pg), "league_vs_builtin": len(lg)},
            "seeds": {"proxy": str(SEEDS["bench_proxy"]), "league": str(SEEDS["bench_league"])},
            "incident_rates": psum["incident_rates"],
            "proxy_version": prep["meta"].get("proxy_version"),
            "proxy_start": "war", "league_start": "peace",
            "uncertainty": {k: v for k, v in uncertainty.bootstrap(pg + lg, label, self.cfg["fitness"], self.cfg["bootstrap"]).items() if k != "samples"},
            "sample": {"game_ids": [g["game_id"] for g in pg + lg]},
        }
        return bench, m, pg + lg

    def showcase(self, games, label, why_best, coach_doc):
        proxy = [g for g in games if g["kind"] == "proxy" and g.get("archive_path")]
        out = []
        if not proxy:
            return out
        proxy_won = sum(1 for g in proxy if g["proxy_seat"] in g["winners"]) * 2 >= len(proxy)
        typical_pool = [g for g in proxy if (g["proxy_seat"] in g["winners"]) == proxy_won] or proxy
        med = sorted(g["turns"] for g in typical_pool)[len(typical_pool) // 2]
        typical = min(typical_pool, key=lambda g: (abs(g["turns"] - med), g["seed"]))
        out.append(self.show(typical, label, f"Typical benchmark game of {why_best} against the Proxy ({'a Proxy win' if proxy_won else 'an AI win'}, {typical['turns']} turns, the median length)"))
        ai_wins = [g for g in proxy if g["winners"] and g["proxy_seat"] not in g["winners"]]
        if ai_wins:
            g = min(ai_wins, key=lambda g: g["turns"])
            out.append(self.show(g, label, f"The fastest AI win against the Proxy in this checkpoint ({g['turns']} turns)"))
        for o in (coach_doc or {}).get("observations", []):
            if o["kind"] in ("emerged", "dominant", "countered") and o["evidence"]:
                ev = o["evidence"][0]
                g = next((x for x in games if x["game_id"] == ev["game_id"] and x.get("archive_path")), None)
                if g and all(s["game_id"] != g["game_id"] for s in out):
                    s = self.show(g, label, f"Shows {o['behavior']} ({o['kind']}), cited by the coach")
                    s["highlight_turns"].append({"turn": ev["turn"], "player": ev["player"], "note": ev["detail"][:200]})
                    s["highlight_turns"].sort(key=lambda h: h["turn"])
                    out.append(s)
                    break
        return out

    @staticmethod
    def show(g, label, why):
        turns = []
        ps = {p["id"]: p for p in g["players"]}
        if g.get("first_capture") and g["proxy_seat"] is not None:
            turns.append({"turn": g["first_capture"], "player": g["proxy_seat"], "note": f"The Proxy ({ps[g['proxy_seat']]['faction']}) captures its first city"})
        for p in sorted((p for p in g["players"] if p["elim"]), key=lambda p: p["elim"])[:2]:
            who = "the Proxy" if p["id"] == g["proxy_seat"] else ("a tuned AI" if p["brain"] == label else "an AI")
            turns.append({"turn": p["elim"], "player": p["id"], "note": f"{p['faction']} ({who}, seat {p['id'] + 1}) is eliminated"})
        for w in g["winners"][:1]:
            p = ps[w]
            who = "the Proxy" if w == g["proxy_seat"] else ("a tuned AI" if p["brain"] == label else "an AI")
            turns.append({"turn": g["turns"], "player": w, "note": f"{p['faction']} ({who}, seat {w + 1}) wins with {p['vp']} VP"})
        turns.sort(key=lambda h: h["turn"])
        return {"game_id": g["game_id"], "archive_path": g["archive_path"], "seed": g["seed"], "why": why, "highlight_turns": turns}

    def checkpoint(self, gen, values, name, why_best, run_coach=True, background=False):
        """Benchmark now; the coach (detector replays + one LLM call) can finish in the
        background while the next generations run. The checkpoint file is written when the
        coach is done, because its showcase includes a game the coach cites."""
        self.log(f"checkpoint {gen}: benchmarking {name}")
        bench, m, games = self.benchmark(gen, values, name)
        label = f"scripted@{name}"
        prev = self.state["last_checkpoint"]
        before_m = {k: m.get(k) for k in ("proxy_win_rate", "ai_survival_t150", "ai_cities_t100", "league_win_rate",
                                          "archetype_entropy", "archetype_share", "incident_rates")}
        self.state["last_checkpoint"] = {"gen": gen, "metrics": before_m, "genome": values}
        if gen > 0 and run_coach:
            for a, e in self.state["elites"].items():
                elite = f"elite-{a.lower()}"
                if not any(x["name"] == elite and x.get("id") == e["id"] for x in self.state["league"]):
                    self.add_league(gen, elite, e["values"], f"latest {a} representative ({e['id']}, generation {e['gen']}, fitness {e['fitness']})", replace_prefix=elite)
                    self.state["league"][-1]["id"] = e["id"]

        def finish():
            coach_doc = None
            if run_coach and prev is not None:
                try:
                    coach_doc = self.coach_cycle(gen, prev, m, games, values)
                except Exception as e:  # noqa: BLE001 - the coach never stops the run
                    self.log(f"coach cycle {gen} failed: {type(e).__name__}: {e}")
                    with self.lock:
                        self.state["coach_calls"].append({"gen": gen, "provider": "none", "error": f"{type(e).__name__}: {str(e)[:300]}"})
            cp = {"gen": gen, "genome": values, "genome_name": name, "benchmark": bench,
                  "showcase": self.showcase(games, label, why_best, coach_doc)}
            if prev is None:
                cp["baseline"] = True
            runlog.write_json(self.run_dir / f"checkpoints/{gen}.json", cp)
            return cp

        self.join_coach()
        if background and run_coach and prev is not None:
            self.coach_thread = threading.Thread(target=finish, name=f"coach-{gen}", daemon=True)
            self.coach_thread.start()
            return None
        return finish()

    def join_coach(self):
        t = getattr(self, "coach_thread", None)
        if t is not None:
            t.join()
            self.coach_thread = None

    def coach_cycle(self, gen, prev, after, games, values):
        a = self.args
        sampled = coach.sample_games(games, a.coach_games)
        det_dir = self.data / f"coach/detector/g{gen}"
        digests = {}
        for g, _why in sampled:
            if g.get("archive_path"):
                digests[g["game_id"]] = coach.detector_digest(coach.run_detector(a.sim, g["archive_path"], det_dir), g["proxy_seat"])
        change = {k: [prev["genome"].get(k), v] for k, v in values.items() if prev["genome"].get(k) != v}
        change = dict(sorted(change.items(), key=lambda kv: -abs((kv[1][0] or 0) - kv[1][1]))[:12])
        facts = coach.build_facts(self.run_id, gen, prev["gen"], prev["metrics"], after, sampled, digests, change)
        provider = "rules" if a.no_coach else a.coach_provider
        with self.lock:
            self.state["last_coach_gen"] = gen
        doc = coach.coach(facts, provider, ROOT, a.coach_timeout, a.coach_model, self.data / "coach")
        if doc["coach"]["error"]:
            self.log(f"coach fell back to rules: {doc['coach']['error']}")
        runlog.write_json(self.run_dir / f"coach/{gen}.json", doc)
        self.log(f"coach {gen} ({doc['coach']['provider']}): {doc['summary']}")
        with self.lock:
            self.state["coach_calls"].append({"gen": gen, **doc["coach"]})
            for p in doc["proposals"]:
                entry = {"gen": gen, **p, "task": None}
                if p["kind"] == "new_behavior" and p["confidence"] == "high" and not a.no_queue:
                    entry["task"] = self.queue_task(gen, p)
                self.state["proposals"].append(entry)
        return doc

    def queue_task(self, gen, p):
        digest = hashlib.sha256(p["description"].encode()).hexdigest()[:10]
        if digest in self.state["queued"]:
            return None
        self.state["queued"].append(digest)
        pending = ROOT / ".progress/astra-queue/pending"
        if not pending.parent.exists():
            return None
        pending.mkdir(exist_ok=True)
        name = f"evolve-{self.run_id}-g{gen}-{digest}"
        best = self.run_dir / f"checkpoints/{gen}.json"
        body = (f"mode: write\nname: {name}\n\n"
                f"From the ai-evolution coach (run {self.run_id}, generation {gen}, {p['confidence']} confidence):\n\n"
                f"NEW BEHAVIOR: {p['description']}\n\nExpected effect: {p['expected_effect']}\n\n"
                f"Evidence (archived benchmark tapes under {self.data}/archive/g{gen}): {json.dumps(p['evidence'])}\n\n"
                "Implement it in crates/ai only (pillar 6: decisions, never rules). Validate with an A/B Proxy benchmark on "
                "held-out seeds 950001+ (legatum-sim --proxy 0 --rotate-proxy --difficulty hard --games 256 "
                f"--seed 950001, with and without the change, both with --ai-params <the genome in {best}>). Adopt only if the "
                "Proxy win rate drops without breaking the guardrails (faction ratio <= 1.5, no stalls, archetype diversity). "
                "Checks: cargo fmt --all; cargo clippy --workspace --all-targets -- -D warnings; cargo test --workspace. "
                "Keep large artifacts in ~/LegatumData. Never use rm (use trash). Never pkill -f.\n")
        path = pending / f"{name}.md"
        runlog.write_text(path, body)
        return str(path)

    # ------------------------------------------------------------------ digest, film, final

    def digest(self, date: str, final=False):
        with self.lock:
            self._digest(date, final)

    def _digest(self, date: str, final=False):
        gens = runlog.read_jsonl(self.run_dir / "generations.jsonl")
        events = runlog.read_jsonl(self.run_dir / "events.jsonl")
        cps = sorted((json.loads(p.read_text()) for p in (self.run_dir / "checkpoints").glob("*.json")), key=lambda c: c["gen"])
        coaches = sorted((json.loads(p.read_text()) for p in (self.run_dir / "coach").glob("*.json")), key=lambda c: c["gen"])
        pct = lambda x: "n/a" if x is None else f"{x * 100:.1f}%"
        L = [f"# Evolution digest {self.run_id} ({date}{', final' if final else ''})", ""]
        c = self.cfg
        L.append(f"Status: {self.run_json['status']}. {len(gens)} generations of {c['population']} candidates, "
                 f"{c['proxy_games']} Proxy + {c['league_games']} league games each (Proxy v{c.get('proxy_version')}, Hard). "
                 f"Genome: {len(self.genes)} personality and decision genes (pillar 6). Proxy starts at War; league starts at Peace.")
        if gens:
            first, last = gens[0], gens[-1]
            bestg = max(gens, key=lambda g: g["best"]["fitness"])
            L += ["", "## Progress", "",
                  f"- Best fitness: {first['best']['fitness']} at generation {first['gen']}, {last['best']['fitness']} at generation {last['gen']}, peak {bestg['best']['fitness']} at generation {bestg['gen']} ({bestg['best']['id']}).",
                  f"- Mean fitness: {first['mean_fitness']} -> {last['mean_fitness']}; sigma {first['sigma']} -> {last['sigma']}.",
                  f"- Best candidate's Proxy win rate (training seeds): {pct(first['best']['proxy_win_rate'])} -> {pct(last['best']['proxy_win_rate'])}; league win rate {pct(first['best']['league_win_rate'])} -> {pct(last['best']['league_win_rate'])}; faction ratio {first['best']['faction_ratio']} -> {last['best']['faction_ratio']}.",
                  f"- Archetype share of candidate seats: {self.fmt_share(first['archetype_share'])} -> {self.fmt_share(last['archetype_share'])}; winner entropy {first['winner_entropy']} -> {last['winner_entropy']}.",
                  f"- Elites (best genome per archetype): {', '.join(f'{a} {e['id']} ({e['fitness']})' for a, e in last['elites'].items()) or 'none'}."]
        if cps:
            b0, b1 = cps[0]["benchmark"], cps[-1]["benchmark"]
            L += ["", f"## Benchmark (fixed bench seeds): checkpoint {cps[0]['gen']} vs {cps[-1]['gen']}", "",
                  "| metric | before | after |", "| --- | --- | --- |"]
            for k in ("proxy_win_rate", "ai_survival_t150", "ai_cities_t100", "league_win_rate_vs_builtin", "faction_ratio", "archetype_entropy", "dominant_archetype"):
                L.append(f"| {k} | {b0.get(k)} | {b1.get(k)} |")
            L += ["", "Archetype win rates (candidate seats): " + json.dumps(b0.get("archetype_win_rates")) + " -> " + json.dumps(b1.get("archetype_win_rates")), ""]
            for tag, b in (("before", b0), ("after", b1)):
                mm = b.get("matchup_matrix") or {}
                if mm.get("archetypes"):
                    L += [f"Matchup matrix {tag} (row ahead of column):", "", "| | " + " | ".join(mm["archetypes"]) + " |", "|" + " --- |" * (len(mm["archetypes"]) + 1)]
                    for a, row in zip(mm["archetypes"], mm["ahead_rate"]):
                        L.append(f"| {a} | " + " | ".join("-" if v is None else f"{v:.2f}" for v in row) + " |")
                    L.append("")
        fv = self.run_json.get("final_validation")
        L += ["## Adoption verdict", ""]
        verdict = (fv or {}).get("verdict", {"decision": "don't-adopt", "reasons": ["Held-out validation missing or skipped"]})
        L.append(f"**{verdict['decision']}**")
        L += [f"- {reason}" for reason in verdict["reasons"]]
        if fv:
            L += ["", "## Held-out validation (paired seeds 950001+; 95% confidence intervals)", "",
                  "| speed | stock Proxy win rate | evolved Proxy win rate | reduction (CI) | Peace league evolved win rate (CI) |", "| --- | --- | --- | --- | --- |"]
            for speed, row in fv.get("speeds", {}).items():
                pr, le = row["proxy"], row["league"]
                L.append(f"| {speed} | {pr['builtin']['rate']:.3f} {pr['builtin']['ci95']} | {pr['evolved']['rate']:.3f} {pr['evolved']['ci95']} | {pr['improvement']:.3f} {pr['ci95']} | {le['rate']:.3f} {le['ci95']} |")
            L += ["", "Faction balance and strategy diversity use separate homogeneous Peace self-play (raw ratios; simultaneous faction intervals).", ""]
            for speed, row in fv.get("speeds", {}).items():
                for tag, b in row["balance"].items():
                    L.append(f"- {speed} {tag}: {b['games']} games; faction ratio {b['faction_ratio']} (95% upper {b['faction_ratio_upper95']}); winner entropy {b['winner_entropy']} {b['winner_entropy_ci95']}; dominant {b['dominant']}; faction rates {json.dumps(b['factions'])}.")
        if gens:
            ses = [x['fitness_se'] for g in gens for x in g.get('candidates', []) if x.get('fitness_se') is not None]
            walls = [g['wall_s'] for g in gens]
            if ses:
                L += ["", f"Measured candidate fitness SE: mean {sum(ses)/len(ses):.4f}, range {min(ses):.4f}–{max(ses):.4f}. "
                      f"Observed generation rate: {3600*len(walls)/sum(walls):.2f}/hour (current load and niceness). "
                      "Training peaks across fresh seed blocks are not evidence of improvement; use the paired incumbent comparisons.", ""]
        if coaches:
            L += ["## Coach", ""]
            for cd in coaches:
                L.append(f"- Generation {cd['gen']} ({cd.get('coach', {}).get('provider')}): {cd['summary']}")
                if cd.get("dominant_strategy"):
                    d = cd["dominant_strategy"]
                    L.append(f"  - Dominant: {d['archetype']} ({d.get('share')}); counter: {d.get('counter_proposal') or 'none proposed'}")
            L.append("")
        props = self.state["proposals"]
        if props:
            L += ["## Proposals", ""]
            for kind, head in (("new_behavior", "New behaviors (high confidence become Astra tasks, adopted only after an A/B on held-out seeds)"),
                               ("tunable_parameter", "New parameters for the lead to expose as genes"),
                               ("design_lever", "Design levers and counters that need a rule change (lead only, never genes)"),
                               ("exploit", "Exploits to guard against")):
                rows = [p for p in props if p["kind"] == kind]
                if rows:
                    L.append(f"{head}:")
                    L += [f"- g{p['gen']} [{p['confidence']}] {p['description']}" + (f" (queued: {Path(p['task']).name})" if p.get("task") else "") for p in rows]
                    L.append("")
        fails = self.state["failures"]
        errs = [x for x in self.state["coach_calls"] if x.get("error")]
        L += ["## What failed", ""]
        L += [f"- g{f['gen']} {f['id']}: {f['error']}" for f in fails[-10:]] or []
        L += [f"- coach g{x['gen']}: {x['error']}" for x in errs]
        if not fails and not errs:
            L.append("- Nothing: no failed evaluations or coach calls.")
        L += ["", "## Events", ""] + [f"- g{e['gen']} {e['kind']}: {e['detail']}" for e in events[-30:]]
        text = "\n".join(L) + "\n"
        runlog.write_text(self.run_dir / f"digest-{date}.md", text)
        reviews = ROOT / ".reviews"
        if reviews.exists() or (ROOT / ".progress").exists():
            runlog.write_text(reviews / f"ai-coach-digest-{date}.md", text)

    @staticmethod
    def fmt_share(share):
        return ", ".join(f"{a} {round(v * 100)}%" for a, v in sorted(share.items(), key=lambda kv: -kv[1]) if v >= 0.05)

    def film(self):
        if self.args.no_film:
            return
        films = Path.home() / "LegatumData/films"
        films.mkdir(parents=True, exist_ok=True)
        log = open(films / f"{self.run_id}.log", "w")
        subprocess.Popen(["tools/film/make-film.sh", str(self.run_dir)], cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
        self.log(f"film hook started: tools/film/make-film.sh {self.run_dir} (log {films / (self.run_id + '.log')})")

    def validate_final(self, values):
        n = self.args.validate_games
        if n <= 0:
            return {"speeds": {}, "verdict": uncertainty.verdict({})}
        out = {}
        stock = Path(self.state["league"][0]["path"])
        evolved = self.data / "params/final.json"
        self.write_params(genome.params_file(values, "final", f"{self.run_id} selected incumbent"), evolved)
        speed = "standard"
        self.log("held-out validation: Standard")
        pg, balances = {}, {}
        failures = 0
        for tag, params in (("builtin", stock), ("evolved", evolved)):
            name = json.loads(params.read_text())["name"]
            rep = self.sim(f"validate-{speed}-{tag}-proxy", params=params, proxy=True, games=n,
                           seed=SEEDS["held_out_proxy"],
                           keep=self.data / f"bench/validate-{speed}-{tag}-proxy.json.gz")
            pg[tag] = metrics.compact_games(rep)
            failures += sum(g["outcome"] in ("stalled", "panic") for g in pg[tag])
            rep = self.sim(f"validate-{speed}-{tag}-balance", params=params, games=self.args.balance_games,
                           seed=SEEDS["held_out_balance"],
                           keep=self.data / f"bench/validate-{speed}-{tag}-balance.json.gz")
            balances[tag] = uncertainty.balance(metrics.compact_games(rep), f"scripted@{name}", self.cfg["bootstrap"])
        rep = self.sim(f"validate-{speed}-league", params=evolved, opponent=stock, games=n,
                       seed=SEEDS["held_out_league"],
                       keep=self.data / f"bench/validate-{speed}-league.json.gz")
        lg = metrics.compact_games(rep)
        wins = sum(any(p["brain"] == "scripted@final" and p["id"] in g["winners"] for p in g["players"]) for g in lg)
        failures += sum(g["outcome"] in ("stalled", "panic") for g in lg)
        out[speed] = {"proxy": uncertainty.proxy_comparison(pg["builtin"], pg["evolved"]),
                      "league": {"games": n, "wins": wins, "rate": wins / n, "ci95": uncertainty.wilson(wins, n)},
                      "balance": balances, "stalls": failures}
        return {"speeds": out, "seeds": {k: str(v) for k, v in SEEDS.items() if k.startswith("held_out")},
                "proxy_start": "war", "league_start": "peace", "verdict": uncertainty.verdict(out)}

    def budget_left(self) -> bool:
        next_gen = self.state["gen"] + 1
        if (SEEDS["proxy"] + next_gen * self.cfg["proxy_games"] > SEEDS["league"]
                or SEEDS["league"] + next_gen * self.cfg["league_games"] > SEEDS["bench_proxy"]):
            self.log("training seed partition exhausted; proceeding to held-out validation")
            return False
        if not self.args.hours:
            return True
        secs = self.state["gen_seconds"][-5:]
        per = sum(secs) / len(secs) if secs else 0
        # Standard games and 2,000-game self-play gates dominate closing time.
        # Use measured generation cost, and a conservative 2x Standard multiplier.
        train_games = (self.cfg["population"] + 1) * self.cfg["games_per_candidate"]
        closing_games = 3 * (3 * self.args.validate_games + 2 * self.args.balance_games)
        reserve = per * (1 + closing_games / train_games) + 600
        return time.time() - self.t0 + per + reserve < self.args.hours * 3600

    def loop(self):
        st = self.state
        if st["last_checkpoint"] is None:
            base = {g["name"]: g["default"] for g in self.genes}
            cp = self.checkpoint(0, base, "bench-builtin", "the built-in Hard AI", run_coach=False)
            b = cp["benchmark"]
            self.run_json["baseline"] = {"proxy_win_rate": b["proxy_win_rate"], "ai_cities_t150": b["ai_cities_t150"],
                                         "ai_cities_t100": b["ai_cities_t100"], "ai_survival_t150": b["ai_survival_t150"],
                                         "archetype_entropy": b["archetype_entropy"], "checkpoint": 0}
            self.save_run_json()
            self.save_state()
        last = None
        while st["gen"] < self.args.generations and self.budget_left():
            gen = st["gen"]
            rec, results, values, _ = self.generation(gen)
            last = (gen, rec, results, values)
            self.save_run_json()  # proxy_version is learned from the first report
            if gen > 0 and gen % self.cfg["coach_every"] == 0:
                self.checkpoint(gen, st["best"]["values"], f"bench-g{gen}", f"paired-selected incumbent ({st['best']['id']})", background=True)
            st["gen"] = gen + 1
            self.save_state()
            today = dt.date.today().isoformat()
            if today != st["digest_date"]:
                self.digest(st["digest_date"])
                st["digest_date"] = today
                self.save_state()
                self.film()
        self.finish(last)

    def finish(self, last):
        self.join_coach()
        st = self.state
        gen = st["gen"] - 1
        selected = (st["best"] or {}).get("values", {g["name"]: g["default"] for g in self.genes})
        if gen >= 0 and not (self.run_dir / f"checkpoints/{gen}.json").exists():
            due = st["last_coach_gen"] is None or gen - st["last_coach_gen"] >= max(1, self.cfg["coach_every"] // 2)
            self.checkpoint(gen, selected, f"bench-final-g{gen}", f"the paired-selected incumbent (generation {gen})", run_coach=due)
        runlog.write_json(self.run_dir / "best-params.json", genome.params_file(selected, f"evolve-{self.run_id}", f"{self.run_id}: paired-selected incumbent after generation {gen}"))
        try:
            fv = self.validate_final(selected)
            if fv:
                self.run_json["final_validation"] = fv
        except Exception as e:  # noqa: BLE001
            self.log(f"held-out validation failed: {e}")
            st["failures"].append({"gen": gen, "id": "validation", "error": str(e)})
            self.run_json["final_validation"] = {"speeds": {}, "error": str(e),
                "verdict": uncertainty.verdict({}, [f"validation failed: {e}"])}
        self.run_json["status"] = "finished"
        self.run_json["finished_at"] = utc()
        self.run_json["generations_run"] = st["gen"]
        self.save_run_json()
        self.save_state()
        self.digest(dt.date.today().isoformat(), final=True)
        self.film()
        self.log(f"finished: {self.run_dir}")

    def stopped(self):
        self.run_json["status"] = "stopped"
        self.run_json["finished_at"] = utc()
        self.save_run_json()
        self.digest(dt.date.today().isoformat())
        self.log(f"stopped; resume with: python3 tools/evolve/evolve.py --resume {self.run_dir}")


def main(argv=None):
    args = parse_args(argv)
    apply_nice(args)
    run = Run(args)

    def on_term(signum, frame):
        raise Stop()
    signal.signal(signal.SIGTERM, on_term)
    try:
        run.loop()
    except (KeyboardInterrupt, Stop):
        # state.json is only written at consistent points (after ask, after a generation),
        # so a resume redoes at most the interrupted generation from its cached candidates.
        run.stopped()
        return 130
    except Exception as exc:  # a broken evaluation must not leave a running/successful log
        run.state["failures"].append({"gen": run.state["gen"], "id": "harness", "error": str(exc)})
        run.run_json["final_validation"] = {"speeds": {}, "error": str(exc),
            "verdict": uncertainty.verdict({}, [f"run failed: {exc}"])}
        run.stopped()
        return 1
    print(run.run_dir)
    return 0


if __name__ == "__main__":
    sys.exit(main())
