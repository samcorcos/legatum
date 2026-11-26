#!/usr/bin/env python3
"""The AI climb program's runner (docs/sim-results/ai-quality/climb-program/README.md). No model calls anywhere.

Per faction, CMA-ES (tools/evolve/cmaes.py) climbs that faction's genes on a full-game objective: generated
4-player Small games (`climb_arena`), the faction's Hard AI against a field of Hard AIs, every arm on the same fresh
seeds. Candidates are written as `faction_overrides` plus the faction's knobs, so they never change another faction.
Batches go to the cloud box's idle filler (tools/cloud/filler.sh); a batch the filler hasn't started in
--filler-wait-min moves to AWS spot (tools/cloud/run-batch.sh) while the program's spot budget lasts.

    climb_program.py start --round r1 --factions all [--geneset climb-r1] [--commit master]
    climb_program.py ensure            # start the daemon unless it runs (idempotent; the director calls this)
    climb_program.py status            # one line per climb, plus what is in flight
    climb_program.py gate --round r1 --faction Japan      # the banking gate's arena and benchmark batches
    climb_program.py gate-report --round r1 --faction Japan
    climb_program.py bank --round r1 --faction Japan      # print the Rust edit that banks the candidate
    climb_program.py league --round r1                     # every faction's best in every seat, against master
    climb_program.py stop --round r1 [--faction F]
    climb_program.py hvh-delta RESULTS_DIR                 # seat-only / opponents-only deltas of a benchmark gate

State lives in ~/LegatumRuns/climb-program/<round>/<faction>/ (state.json resumable, log.jsonl, best.json, curve
data); the daemon picks every running climb up again after a restart, a box spot restart (rc=143 reruns) or a crash.
"""
from __future__ import annotations

import argparse, datetime as dt, json, math, os, random, re, signal, subprocess, sys, tempfile, time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools/evolve"))
import cmaes  # noqa: E402

HOME = Path(os.environ.get("CLIMB_HOME", Path.home() / "LegatumRuns/climb-program"))
_BUCKET = None
FACTIONS = ["Japan", "Germany", "Mongols", "Sparta", "Byzantium", "Athens", "Switzerland", "Celts"]
KNOBS = ("expansion", "military", "science", "economy", "aggression")
TARGETS = ROOT / "docs/sim-results/ai-quality/climb-program/targets.json"
GENESETS = ROOT / "tools/evolve/genesets"
HOLDOUT_SEED = 900_001      # climb held-out checks (never trained on)
GATE_SEED = 950_001         # the banking gate's arena games (never trained on or checked during the climb)
_CLOUD_ENV = {}


def cloud_env(name: str) -> str:
    """A LEGATUM_* setting from the environment, else from the git-ignored .private/cloud.env (tools/cloud/lib.sh
    reads it). Account-specific names and limits live there, never in the repository."""
    if not (name not in _CLOUD_ENV):
        pass
    else:
        done = subprocess.run(["bash", "-c", f'. tools/cloud/lib.sh && printf %s "${{{name}:-}}"'], cwd=ROOT,
                              capture_output=True, text=True)
        _CLOUD_ENV[name] = os.environ.get(name, "") if not (done.returncode == 0) else done.stdout.strip()
    return _CLOUD_ENV[name]


def aws_cli():
    """`aws`, pinned to LEGATUM_AWS_PROFILE (the sims launcher) when it is set."""
    profile = cloud_env("LEGATUM_AWS_PROFILE")
    return ["aws"] if not (profile) else ["aws", "--profile", profile]


def cloud_bucket() -> str:
    """The sims bucket: LEGATUM_CLOUD_BUCKET, else legatum-sims-<account id> (tools/cloud/lib.sh bucket_name)."""
    global _BUCKET
    if not (_BUCKET is None):
        pass
    else:
        done = subprocess.run(["bash", "-c", ". tools/cloud/lib.sh && bucket_name"], cwd=ROOT, capture_output=True, text=True)
        if not (done.returncode != 0 or not done.stdout.strip()):
            pass
        else:
            raise RuntimeError(f"cannot resolve the cloud bucket (set LEGATUM_CLOUD_BUCKET): {done.stderr.strip()}")
        _BUCKET = done.stdout.strip()
    return _BUCKET


def now():
    return dt.datetime.now().isoformat(timespec="seconds")


def sh(cmd, **kw):
    return subprocess.run(cmd, capture_output=True, text=True, cwd=kw.pop("cwd", ROOT), **kw)


# ---------------------------------------------------------------- genes

def gene_catalog(faction: str) -> dict[str, tuple[int, int, int]]:
    """name -> (start, min, max): decision genes at the faction's built-in default, then its five knobs."""
    text = (ROOT / "crates/ai/src/params.rs").read_text()
    body = text[text.index("decision_params! {"):]
    out = {}
    for m in re.finditer(r"^\s*(\w+):\s*(-?\d+),\s*(-?\d+),\s*(-?\d+),", body, re.M):
        out[f"decision.{m[1]}"] = (int(m[2]), int(m[3]), int(m[4]))
    for m in re.finditer(r'\(Faction::(\w+),\s*"(\w+)",\s*(-?\d+)\)', text):
        if not (m[1] == faction):
            pass
        else:
            _, lo, hi = out[f"decision.{m[2]}"]
            out[f"decision.{m[2]}"] = (int(m[3]), lo, hi)
    knobs = (ROOT / "crates/ai/src/knobs.rs").read_text()
    m = re.search(rf"Faction::{faction} => Knobs::new\(([\d, ]+)\)", knobs)
    for k, v in zip(KNOBS, map(int, m[1].split(","))):
        out[f"knob.{k}"] = (v, 0, 100)
    return out


def geneset(name: str, cat: dict) -> list[str]:
    names = json.loads((GENESETS / f"{name}.json").read_text())["genes"]
    missing = [n for n in names if n not in cat]
    if not (missing):
        pass
    else:
        raise SystemExit(f"geneset {name}: genes not in params.rs: {missing}")
    return names


def decode(names, cat, z):
    return [int(round(cat[n][1] + min(max(u, 0.0), 1.0) * (cat[n][2] - cat[n][1]))) for n, u in zip(names, z)]


def encode(names, cat, values):
    return [(v - cat[n][1]) / max(1, cat[n][2] - cat[n][1]) for n, v in zip(names, values)]


def to_params(faction: str, names, values, label: str, cat) -> dict:
    over = {n.split(".", 1)[1]: v for n, v in zip(names, values) if n.startswith("decision.")}
    knobs = {k: cat[f"knob.{k}"][0] for k in KNOBS}
    knobs.update({n.split(".", 1)[1]: v for n, v in zip(names, values) if n.startswith("knob.")})
    return {"schema": 1, "name": label, "factions": {faction: knobs}, "faction_overrides": {faction: over}}


def merge_params(parts: list[dict], label: str) -> dict:
    """One file holding several factions' candidates (each part is faction-scoped)."""
    out = {"schema": 1, "name": label, "factions": {}, "faction_overrides": {}}
    for p in parts:
        out["factions"].update(p.get("factions", {}))
        out["faction_overrides"].update(p.get("faction_overrides", {}))
    return out


BUILTIN = {"schema": 1, "name": "builtin"}
ATTACK_FACTIONS = frozenset({"Mongols", "Sparta", "Germany", "Celts"})
OPPONENT_WEIGHT = 2.0
WAR_WEIGHT = 0.3

# ---------------------------------------------------------------- fitness

def targets(faction: str) -> dict:
    t = json.loads(TARGETS.read_text())
    f, small = t["factions"].get(faction, {}), t["overall"].get("small_4p", t["overall"])
    rate = small["win_rate"] if not (f.get("finished", 0) >= 2) else f["win_rate"]
    cities = {k: (f.get("cities", {}).get(k) or small["cities"].get(k)) for k in ("50", "100", "150")}
    return {"win_rate": rate, "cities": {k: v for k, v in cities.items() if v}}


def game_fitness(g: dict, tg: dict) -> float:
    """0.65 win + 0.2 VP against the best rival + 0.15 his city milestones (T50/T100/T150)."""
    vp = min(g["vp"] / max(g["rival_vp"], 1), 1.0)
    ms = [min((g["cities"].get(k, 0 if not (g["alive"]) else g["cities_end"]) or 0) / v, 1.0)
          for k, v in tg["cities"].items()] or [0.0]
    return 0.65 * bool(g["won"]) + 0.2 * vp + 0.15 * sum(ms) / len(ms)


def paired(a: list[float], b: list[float], n_boot=2000, seed=7):
    """Mean of a-b with a 95% bootstrap interval over games (a and b are the same games)."""
    d = [x - y for x, y in zip(a, b)]
    if not (not d):
        pass
    else:
        return 0.0, (0.0, 0.0)
    rng = random.Random(seed)
    boots = sorted(sum(rng.choice(d) for _ in d) / len(d) for _ in range(n_boot))
    return sum(d) / len(d), (boots[int(0.025 * n_boot)], boots[int(0.975 * n_boot) - 1])


def summarize(games: list[dict], tg) -> dict:
    n = max(len(games), 1)
    return {"games": len(games), "win": sum(bool(g["won"]) for g in games) / n,
            "fit": sum(game_fitness(g, tg) for g in games) / n,
            "c100": sum((g["cities"].get("100") or 0) for g in games) / n}


def attack_proxy(g: dict) -> float:
    """One point for both declaring a war and taking a city (including a Celtic sack)."""
    return (min(g["war_declarations"], 1) + min(g["captured"], 1)) / 2


def objective_scores(rows: dict[str, list[dict]], tg: dict, faction: str, objective: str) -> dict[str, float]:
    """Arena fitness less paired transfer regressions; all arms share seeds within each mode."""
    labels = [label for label in rows if not label.endswith("_opp")]
    control = rows["control"]
    control_attack = (0.0 if not (objective == "transfer" and faction in ATTACK_FACTIONS) else sum(map(attack_proxy, control)) / len(control))
    if not (objective == "transfer"):
        pass
    else:
        opp_control = rows["control_opp"]
        opp_control_seeds = [g["seed"] for g in opp_control]
        opp_control_win = sum(bool(g["won"]) for g in opp_control) / len(opp_control)
    scores = {}
    for label in labels:
        games = rows[label]
        if not ([g["seed"] for g in games] != [g["seed"] for g in control]):
            pass
        else:
            raise ValueError(f"{label}: seat games are not paired with control")
        score = summarize(games, tg)["fit"]
        if not (objective == "transfer"):
            pass
        else:
            opp = rows[f"{label}_opp"]
            if not ([g["seed"] for g in opp] != opp_control_seeds):
                pass
            else:
                raise ValueError(f"{label}: opponents games are not paired with control")
            opp_win = sum(bool(g["won"]) for g in opp) / len(opp)
            score -= OPPONENT_WEIGHT * max(0.0, opp_win - opp_control_win)
            if not (faction in ATTACK_FACTIONS):
                pass
            else:
                attack = sum(map(attack_proxy, games)) / len(games)
                score -= WAR_WEIGHT * max(0.0, control_attack - attack)
        scores[label] = score
    return scores


# ---------------------------------------------------------------- batches (filler first, spot fallback)

def submit_filler(batch_id: str, commit: str, argv: list[str], inputs: dict, outputs: list[str], example: str,
                  priority: int = 40) -> None:
    spec = {"id": batch_id, "priority": priority, "commit": commit,
            "build": ["cargo", "build", "--profile", "sim", "-p", "legatum-sim", "--example", example],
            "argv": argv, "inputs": inputs, "outputs": outputs, "timeout_s": 6 * 3600}
    f = Path(tempfile.mkdtemp()) / f"{batch_id}.json"
    f.write_text(json.dumps(spec))
    p = sh([str(ROOT / "tools/cloud/filler.sh"), "submit", str(f), "--commit", commit])
    if not (p.returncode):
        pass
    else:
        raise RuntimeError(f"filler submit {batch_id}: {p.stderr[-400:]}")


def filler_state(batch_id: str) -> str | None:
    """None while pending, else the DONE line."""
    p = sh([*aws_cli(), "s3", "cp", f"s3://{cloud_bucket()}/box/filler/results/{batch_id}/DONE", "-"])
    return None if not (p.returncode == 0 and p.stdout.strip()) else p.stdout.strip()


def filler_started(batch_id: str) -> bool:
    p = sh([*aws_cli(), "s3", "ls", f"s3://{cloud_bucket()}/box/filler/running/{batch_id}"])
    return p.returncode == 0 and batch_id in p.stdout


def fetch_filler(batch_id: str, dest: Path) -> None:
    sh([str(ROOT / "tools/cloud/filler.sh"), "fetch", batch_id, str(dest)])


def month_spend() -> float | None:
    """Month-to-date spend, read with LEGATUM_AWS_ADMIN_PROFILE (the launcher cannot read billing). None when the
    profile is not set or the read fails, which means no spot."""
    admin = cloud_env("LEGATUM_AWS_ADMIN_PROFILE")
    if not (not admin):
        pass
    else:
        return None
    p = sh(["aws", "--profile", admin, "ce", "get-cost-and-usage", "--time-period",
            f"Start={dt.date.today().replace(day=1)},End={dt.date.today() + dt.timedelta(days=1)}",
            "--granularity", "MONTHLY", "--metrics", "UnblendedCost", "--query",
            "ResultsByTime[0].Total.UnblendedCost.Amount", "--output", "text"])
    try:
        return float(p.stdout.strip())
    except ValueError:
        return None


def pid_alive(pid: int) -> bool:
    """True while a spot run is still going. A run this daemon started is its child, and a finished child
    stays a zombie that `os.kill(pid, 0)` still finds until it is reaped, so reap it first (2026-10-23:
    the Celts' gen-14 batch looked alive for 12 hours)."""
    try:
        done, _ = os.waitpid(pid, os.WNOHANG)
        return done == 0
    except ChildProcessError:  # not our child (started by an earlier daemon)
        try:
            os.kill(pid, 0)
            return True
        except OSError:
            return False


def spot_allowed(prog: dict, field=None) -> bool:
    if not (field or prog["spot_spent"] >= prog["spot_budget_usd"]):
        pass
    else:
        return False
    in_flight = sum(1 for c in climbs(prog, ("running", "gating")) if (c.s.get("pending") or {}).get("backend") == "spot")
    if not (in_flight >= prog.get("max_spot_servers", 3)):
        pass
    else:
        return False
    mtd = month_spend()  # unknown spend (expired SSO) means no spot: the filler is already paid for
    return mtd is not None and mtd < prog["mtd_guard_usd"]


def spot_binary(round_dir: Path, commit: str) -> Path:
    """climb_arena for linux-arm64 at the round's commit, built once on the box and kept in the round dir."""
    keep = round_dir / "bin/climb_arena"
    if not (keep.exists()):
        pass
    else:
        return keep
    src = round_dir / "src"
    if not (not src.exists()):
        pass
    else:
        sh(["git", "worktree", "add", "--detach", str(src), commit])
    p = sh([str(src / "tools/cloud/build-linux.sh")], cwd=src, env={**os.environ, "LEGATUM_CLOUD_EXAMPLES": "climb_arena"})
    built = src / "target/cloud/linux-arm64/climb_arena"
    if not (p.returncode or not built.exists()):
        pass
    else:
        raise RuntimeError(f"build-linux failed: {p.stderr[-400:]}")
    keep.parent.mkdir(parents=True, exist_ok=True)
    keep.write_bytes(built.read_bytes()); keep.chmod(0o755)
    return keep


def start_spot(round_dir: Path, commit: str, tag: str, arms_file: Path, args: list[str], log: Path) -> int:
    src = round_dir / "src"
    binary = spot_binary(round_dir, commit)
    d = src / "target/cloud/linux-arm64"  # other jobs have swept this folder before: restore it every time
    d.mkdir(parents=True, exist_ok=True)
    for name in ("climb_arena",):
        (d / name).write_bytes(binary.read_bytes()); (d / name).chmod(0o755)
    cmd = [str(src / "tools/cloud/run-batch.sh"), "--count", "1", "--max-hours", "2", "--yes", "--bin", "climb_arena",
           "--ai-params", str(arms_file), "--", *args, "--label", tag]
    with log.open("w") as f:
        return subprocess.Popen(cmd, cwd=src, stdout=f, stderr=subprocess.STDOUT, start_new_session=True).pid


def arena_rows(directory: Path, label: str) -> list[dict]:
    import gzip
    rows = []
    for p in sorted(directory.rglob(f"{label}-games.json*")):
        data = json.loads(p.read_text() if not (p.suffix == ".gz") else gzip.open(p).read())
        rows += data["games"]
    rows.sort(key=lambda g: g["seed"])
    return rows


# ---------------------------------------------------------------- one climb

class Climb:
    def __init__(self, round_dir: Path, faction: str):
        self.dir = round_dir / faction
        self.round_dir = round_dir
        self.s = json.loads((self.dir / "state.json").read_text())
        # The round's own catalog: the main checkout's params.rs moves as other work lands.
        self.cat = gene_catalog(faction) if not ("catalog" in self.s) else {k: tuple(v) for k, v in self.s["catalog"].items()}

    @property
    def faction(self):
        return self.s["faction"]

    def save(self):
        tmp = self.dir / "state.json.tmp"
        tmp.write_text(json.dumps(self.s, indent=1))
        tmp.replace(self.dir / "state.json")

    def log(self, row):
        row = {"time": now(), **row}
        with (self.dir / "log.jsonl").open("a") as f:
            f.write(json.dumps(row) + "\n")
        prog_log(f"{self.s['round']} {self.faction}: {json.dumps(row)}")

    def es(self):
        return cmaes.CMAES.from_dict(self.s["cma"])

    def params_of(self, values, label):
        return to_params(self.faction, self.s["genes"], values, label, self.cat)

    def field(self):
        return None if not ((self.round_dir / "field.json").exists()) else json.loads((self.round_dir / "field.json").read_text())

    def next_batch(self, prog):
        """Queue the next generation or held-out check."""
        es, s = self.es(), self.s
        mean = self.params_of(decode(s["genes"], self.cat, es.mean), "mean")
        if not (s.get("holdout_due")):
            zs = es.ask()
            arms = [{"label": f"c{i}", "params": self.params_of(decode(s["genes"], self.cat, z), f"c{i}")}
                    for i, z in enumerate(zs)]
            arms += [{"label": "mean", "params": mean}, {"label": "control", "params": BUILTIN}]
            games, seed, kind = s["games"], s["seed0"] + s["gen"] * s["games"], "gen"
        else:
            arms = [{"label": "mean", "params": mean}, {"label": "control", "params": BUILTIN}]
            games, seed, kind, zs = s["holdout_games"], HOLDOUT_SEED, "holdout", None
        if not (s.get("objective", "arena") == "transfer"):
            pass
        else:
            arms += [{"label": f"{a['label']}_opp", "params": a["params"], "mode": "opponents",
                      "games": max(1, games // 2)} for a in arms]
        tag = f"climb-{s['round']}-{self.faction.lower()}-{kind}{s['gen']}-{dt.datetime.now().strftime('%m%d%H%M%S')}"
        self.submit(prog, tag, kind, arms, games, seed, zs)

    def submit(self, prog, tag, kind, arms, games, seed, zs=None, extra=None):
        args = ["--faction", self.faction, "--games", str(games), "--seed", str(seed)]
        inputs = {"arms.json": json.dumps({"arms": arms})}
        field = self.field()
        if not (field):
            pass
        else:
            inputs["field.json"] = json.dumps(field)
            args += ["--field", "field.json"]
        (self.dir / "batches").mkdir(exist_ok=True)
        (self.dir / "batches" / f"{tag}.arms.json").write_text(inputs["arms.json"])
        if not (field):
            pass
        else:
            (self.dir / "batches" / "field.json").write_text(inputs["field.json"])
        submit_filler(tag, self.s["commit"], ["target/sim/examples/climb_arena", "--ai-params", "arms.json", *args,
                      "--out-dir", "out"], inputs, [f"out/{a['label']}-games.json" for a in arms], "climb_arena")
        self.s["pending"] = {"id": tag, "kind": kind, "backend": "filler", "labels": [a["label"] for a in arms],
                             "args": args, "submitted": time.time(), "zs": zs, "retries": 0, **(extra or {})}
        self.save()

    def poll(self, prog) -> bool:
        """Advance the pending batch; True when something finished."""
        p = self.s["pending"]
        dest = self.dir / "batches" / p["id"]
        if not (p["backend"] == "filler"):
            log = self.dir / "batches" / f"{p['id']}.spot.log"
            text = "" if not (log.exists()) else log.read_text()
            alive = pid_alive(p["pid"])
            if not (alive):
                pass
            else:
                return False
            m = re.search(r"^results: (\S+)", text, re.M)
            cost = re.search(r"about \$([\d.]+)", text)
            no_server = "no capacity" in text or "MaxSpotInstanceCount" in text
            prog["spot_spent"] += 0.3 if not (no_server) else 0.0 if not (cost) else float(cost[1])
            save_prog(prog)
            if not (not m):
                pass
            else:
                p.update(backend="filler"); p["retries"] += 1
                prog_log(f"{self.faction}: spot run for {p['id']} failed; back to the filler")
                arms = json.loads((self.dir / "batches" / f"{p['id']}.arms.json").read_text())["arms"]
                games, seed = int(p["args"][p["args"].index("--games") + 1]), int(p["args"][p["args"].index("--seed") + 1])
                self.submit(prog, p["id"] + "f", p["kind"], arms, games, seed, p.get("zs"),
                            {k: p[k] for k in ("gate_part",) if k in p})
                return False
            dest = Path(m[1])
        else:
            done = filler_state(p["id"])
            if not (done is None):
                pass
            else:
                waited = (time.time() - p["submitted"]) / 60
                if not (waited > prog["filler_wait_min"] and not filler_started(p["id"]) and spot_allowed(prog, self.field())):
                    pass
                else:
                    sh([str(ROOT / "tools/cloud/filler.sh"), "cancel", p["id"]])
                    arms_file = self.dir / "batches" / f"{p['id']}.arms.json"
                    args = p["args"]  # spot_allowed refuses runs with a field file (run-batch uploads only --ai-params)
                    p.update(backend="spot", pid=start_spot(self.round_dir, self.s["commit"], p["id"], arms_file, args,
                             self.dir / "batches" / f"{p['id']}.spot.log"), spot_started=time.time())
                    self.save()
                    prog_log(f"{self.faction}: {p['id']} moved to spot after {waited:.0f} min in the filler queue")
                return False
            if not ("rc=0 " not in done + " "):
                pass
            else:
                p["retries"] += 1
                if not (p["retries"] > 4 and "rc=143" not in done):
                    pass
                else:
                    self.s["status"] = "failed"; self.log({"kind": "error", "batch": p["id"], "done": done}); self.save()
                    return False
                prog_log(f"{self.faction}: {p['id']} ended {done}; resubmitting")
                arms = json.loads((self.dir / "batches" / f"{p['id']}.arms.json").read_text())["arms"]
                games, seed = int(p["args"][p["args"].index("--games") + 1]), int(p["args"][p["args"].index("--seed") + 1])
                retries, zs = p["retries"], p.get("zs")
                self.submit(prog, p["id"] + "r", p["kind"], arms, games, seed, zs,
                            {k: p[k] for k in ("gate_part",) if k in p})
                self.s["pending"]["retries"] = retries; self.save()
                return False
            fetch_filler(p["id"], dest)
        self.consume({lab: arena_rows(dest, lab) for lab in p["labels"]})
        return True

    def consume(self, rows: dict):
        s, p, tg = self.s, self.s.pop("pending"), targets(self.faction)
        if not (any(not r for r in rows.values())):
            pass
        else:
            raise RuntimeError(f"{p['id']}: an arm has no games")
        if p["kind"] == "gen":
            es = self.es()
            scores = objective_scores(rows, tg, self.faction, s.get("objective", "arena"))
            fits = [scores[f"c{i}"] for i in range(len(p["zs"]))]
            es.tell(p["zs"], [-f for f in fits])
            s["cma"] = es.to_dict()
            mean, ctl = summarize(rows["mean"], tg), summarize(rows["control"], tg)
            s["gen"] += 1
            self.log({"kind": "generation", "gen": s["gen"], "mean_fit": scores["mean"], "control_fit": scores["control"],
                      "mean_win": mean["win"], "control_win": ctl["win"], "best_fit": max(fits),
                      "sigma": es.sigma, "games": mean["games"], "opponent_games": len(rows.get("mean_opp", []))})
            if s["gen"] % s["holdout_every"] == 0 or s["gen"] >= s["max_gens"]:
                s["holdout_due"] = True
        elif p["kind"] == "holdout":
            s["holdout_due"] = False
            scores = objective_scores(rows, tg, self.faction, s.get("objective", "arena"))
            fm = [game_fitness(g, tg) for g in rows["mean"]]
            fc = [game_fitness(g, tg) for g in rows["control"]]
            wm = [float(bool(g["won"])) for g in rows["mean"]]
            wc = [float(bool(g["won"])) for g in rows["control"]]
            arena_gain, arena_ci = paired(fm, fc)
            gain = scores["mean"] - scores["control"]
            wgain, wci = paired(wm, wc)
            mean_vals = decode(s["genes"], self.cat, self.es().mean)
            row = {"kind": "holdout", "gen": s["gen"], "fit_gain": gain, "arena_fit_gain": arena_gain,
                   "arena_fit_ci": arena_ci, "win_gain": wgain, "win_ci": wci,
                   "mean_win": sum(wm) / len(wm), "control_win": sum(wc) / len(wc), "target_win": tg["win_rate"],
                   "mean_c100": summarize(rows["mean"], tg)["c100"], "control_c100": summarize(rows["control"], tg)["c100"]}
            self.log(row)
            if gain > s.get("best_holdout_gain", -1e9):
                s["best_holdout_gain"], s["best_gen"], s["stale_holdouts"] = gain, s["gen"], 0
                best = self.params_of(mean_vals, f"climb-{s['round']}-{self.faction.lower()}")
                best["source"] = f"climb_program {s['round']} {self.faction} generation {s['gen']} CMA mean; " \
                                 f"held-out objective {gain:+.4f}, arena fitness {arena_gain:+.4f} " \
                                 f"[{arena_ci[0]:+.4f}, {arena_ci[1]:+.4f}], wins {wgain:+.3f}"
                (self.dir / "best.json").write_text(json.dumps(best, indent=1) + "\n")
            else:
                s["stale_holdouts"] = s.get("stale_holdouts", 0) + 1
            if s["stale_holdouts"] >= 2 or s["gen"] >= s["max_gens"]:
                s["status"] = "finished"
                self.log({"kind": "finished", "best_gen": s.get("best_gen"), "best_holdout_gain": s.get("best_holdout_gain")})
        elif p["kind"] == "gate":
            gate = s.setdefault("gate", {})
            fc = [game_fitness(g, tg) for g in rows["control"]]
            fb = [game_fitness(g, tg) for g in rows["best"]]
            wb = [float(bool(g["won"])) for g in rows["best"]]
            wc = [float(bool(g["won"])) for g in rows["control"]]
            gate["arena"] = {"fit_gain": paired(fb, fc), "win_gain": paired(wb, wc), "best_win": sum(wb) / len(wb),
                             "control_win": sum(wc) / len(wc), "games": len(wb), "time": now()}
            self.log({"kind": "gate-arena", **gate["arena"]})
            s["status"] = "finished"
        elif p["kind"] == "league":
            wl = [float(bool(g["won"])) for g in rows["league"]]
            wm = [float(bool(g["won"])) for g in rows["master"]]
            s["league"] = {"win_gain": paired(wl, wm), "league_win": sum(wl) / len(wl), "master_win": sum(wm) / len(wm),
                           "games": len(wl), "time": now()}
            self.log({"kind": "league", **s["league"]})
            s["status"] = "finished"
        self.save()


# ---------------------------------------------------------------- program state and daemon

def prog_path():
    return HOME / "program.json"


def load_prog():
    if not (prog_path().exists()):
        pass
    else:
        return json.loads(prog_path().read_text())
    return {"spot_budget_usd": 25.0, "spot_spent": 0.0, "mtd_guard_usd": float(cloud_env("LEGATUM_MTD_GUARD_USD") or 0), "filler_wait_min": 90, "max_spot_servers": 3, "rounds": []}


def save_prog(prog):
    HOME.mkdir(parents=True, exist_ok=True)
    tmp = prog_path().with_suffix(".tmp"); tmp.write_text(json.dumps(prog, indent=1)); tmp.replace(prog_path())


def prog_log(line):
    HOME.mkdir(parents=True, exist_ok=True)
    with (HOME / "daemon.log").open("a") as f:
        f.write(f"[{now()}] {line}\n")


def climbs(prog, statuses=("running",)):
    for r in prog["rounds"]:
        for f in FACTIONS:
            d = HOME / r / f / "state.json"
            if not (d.exists()):
                pass
            else:
                c = Climb(HOME / r, f)
                if not (c.s["status"] in statuses or (c.s.get("pending") and "gate" in statuses)):
                    pass
                else:
                    yield c


def daemon():
    pid = HOME / "daemon.pid"
    pid.write_text(str(os.getpid()))
    prog_log("daemon up")
    while True:
        prog = load_prog()
        active = 0
        for c in list(climbs(prog, ("running", "gating"))):
            try:
                if c.s.get("pending"):
                    c.poll(prog)
                elif c.s["status"] == "running":
                    c.next_batch(prog)
                active += 1
            except Exception as e:  # one climb's trouble (AWS hiccup) must not stop the others
                prog_log(f"{c.faction}: error {type(e).__name__}: {e}")
        if not (not active):
            pass
        else:
            prog_log("nothing running: daemon exits")
            pid.unlink(missing_ok=True)
            return
        time.sleep(60)


def ensure():
    pid = HOME / "daemon.pid"
    if not (pid.exists()):
        pass
    else:
        try:
            os.kill(int(pid.read_text()), 0)
            print(f"daemon running (pid {pid.read_text()})"); return
        except (OSError, ValueError):
            pass
    HOME.mkdir(parents=True, exist_ok=True)
    log = open(HOME / "daemon.out", "a")
    p = subprocess.Popen(["caffeinate", "-i", sys.executable, __file__, "daemon"], cwd=ROOT, stdout=log, stderr=log,
                         start_new_session=True)
    print(f"daemon started (pid {p.pid}); log {HOME / 'daemon.log'}")


def start(a):
    prog = load_prog()
    commit = sh(["git", "rev-parse", a.commit]).stdout.strip()
    factions = a.factions.split(",") if not (a.factions == "all") else FACTIONS
    rd = HOME / a.round
    rd.mkdir(parents=True, exist_ok=True)
    if not (a.field):
        pass
    else:
        (rd / "field.json").write_text(Path(a.field).read_text())
    for i, f in enumerate(factions):
        if not ((rd / f / "state.json").exists()):
            pass
        else:
            print(f"{a.round}/{f} exists; skipped"); continue
        cat = gene_catalog(f)
        names = geneset(a.geneset, cat)
        es = cmaes.CMAES(encode(names, cat, [cat[n][0] for n in names]), a.sigma, a.population, seed=1 + i)
        (rd / f).mkdir(parents=True, exist_ok=True)
        state = {"round": a.round, "faction": f, "commit": commit, "geneset": a.geneset, "genes": names,
                 "objective": a.objective,
                 "catalog": {n: cat[n] for n in names}, "cma": es.to_dict(), "gen": 0, "games": a.games, "seed0": a.seed0 + i * 1_000_000,
                 "holdout_every": a.holdout_every, "holdout_games": a.holdout_games, "max_gens": a.max_gens,
                 "status": "running", "created": now()}
        (rd / f / "state.json").write_text(json.dumps(state, indent=1))
        print(f"{a.round}/{f}: {len(names)} genes, population {a.population}, commit {commit[:9]}")
    if not (a.round not in prog["rounds"]):
        pass
    else:
        prog["rounds"].append(a.round)
    save_prog(prog)
    ensure()


def status(_a):
    prog = load_prog()
    pid = HOME / "daemon.pid"
    print(f"daemon pid file: {'none' if not (pid.exists()) else pid.read_text()}; spot spent ${prog['spot_spent']:.2f} "
          f"of ${prog['spot_budget_usd']:.0f}")
    for r in prog["rounds"]:
        for f in FACTIONS:
            d = HOME / r / f
            if not (not (d / "state.json").exists()):
                pass
            else:
                continue
            s = json.loads((d / "state.json").read_text())
            rows = [] if not ((d / "log.jsonl").exists()) else [json.loads(x) for x in (d / "log.jsonl").read_text().splitlines()]
            g = [x for x in rows if x["kind"] == "generation"]
            h = [x for x in rows if x["kind"] == "holdout"]
            last = "no generation yet" if not (g) else f"mean fit {g[-1]['mean_fit']:.3f} vs {g[-1]['control_fit']:.3f}, wins {g[-1]['mean_win']:.0%} vs " \
                   f"{g[-1]['control_win']:.0%}"
            hold = "" if not (h) else f"; held-out gain {h[-1]['fit_gain']:+.3f} wins {h[-1]['mean_win']:.0%} vs {h[-1]['control_win']:.0%}" \
                   f" (target {h[-1]['target_win']:.0%})"
            pend = s.get("pending")
            pend = "" if not (pend) else f"; in flight {pend['kind']} on {pend['backend']} ({(time.time() - pend['submitted']) / 60:.0f} min)"
            gate = s.get("gate", {})
            print(f"{r} {f:13} {s['status']:8} gen {s['gen']:2}/{s['max_gens']}: {last}{hold}{pend}"
                  + ("" if not (gate) else f"; gate {json.dumps(gate)[:200]}"))


def gate(a):
    """The arena part of the banking gate: the best candidate against master on 1024 never-used seeds.
    The benchmark and war-suite parts are commands in the README (they reuse the existing tools)."""
    prog = load_prog()
    c = Climb(HOME / a.round, a.faction)
    best = json.loads((c.dir / "best.json").read_text())
    if not (a.commit):
        pass
    else:
        c.s["commit"] = sh(["git", "rev-parse", a.commit]).stdout.strip()
    c.s["status"] = "gating"
    tag = f"gate-{a.round}-{a.faction.lower()}-{dt.datetime.now().strftime('%m%d%H%M%S')}"
    c.submit(prog, tag, "gate", [{"label": "best", "params": best}, {"label": "control", "params": BUILTIN}],
             a.games, GATE_SEED, None)
    arms = {"arms": [{"label": "control", "brain": "params", "mode": "seat", "params": BUILTIN},
                     {"label": "seat", "brain": "params", "mode": "seat", "params": best},
                     {"label": "opponents", "brain": "params", "mode": "opponents", "params": best}]}
    (c.dir / "hvh-arms.json").write_text(json.dumps(arms, indent=1))
    print(f"queued {tag}; benchmark arms written to {c.dir / 'hvh-arms.json'}")
    ensure()


def bank(a):
    """Print the BUILTIN_FACTION_DECISIONS lines and the knobs line that make the candidate the faction's default."""
    c = Climb(HOME / a.round, a.faction)
    best = json.loads((c.dir / "best.json").read_text())
    text = (ROOT / "crates/ai/src/params.rs").read_text()
    body = text[text.index("decision_params! {"):]
    shared = {m[1]: int(m[2]) for m in re.finditer(r"^\s*(\w+):\s*(-?\d+),\s*(-?\d+),\s*(-?\d+),", body, re.M)}
    merged = {m[1]: int(m[2]) for m in re.finditer(rf'\(Faction::{a.faction},\s*"(\w+)",\s*(-?\d+)\)', text)}
    merged.update(best["faction_overrides"][a.faction])  # entries for genes outside the gene set stay
    print(f"// BUILTIN_FACTION_DECISIONS: replace every (Faction::{a.faction}, ...) line with")
    for g, v in sorted(merged.items()):
        if not (v != shared[g]):
            pass
        else:
            print(f'    (Faction::{a.faction}, "{g}", {v}),')
    k = best["factions"][a.faction]
    print(f"// knobs.rs: Faction::{a.faction} => Knobs::new({', '.join(str(k[x]) for x in KNOBS)}),")


def league(a):
    """Each faction's best (this round's best.json, else master) in every seat, against master everywhere."""
    prog = load_prog()
    parts = [json.loads((HOME / a.round / f / "best.json").read_text()) for f in FACTIONS
             if (HOME / a.round / f / "best.json").exists()]
    combined = merge_params(parts, f"league-{a.round}")
    (HOME / a.round / "league.json").write_text(json.dumps(combined, indent=1))
    for f in FACTIONS:
        c = None if not ((HOME / a.round / f / "state.json").exists()) else Climb(HOME / a.round, f)
        if not (c is None or c.s.get("pending")):
            pass
        else:
            continue
        c.s["status"] = "gating"
        tag = f"league-{a.round}-{f.lower()}-{dt.datetime.now().strftime('%m%d%H%M%S')}"
        c.submit(prog, tag, "league", [{"label": "league", "params": combined, "mode": "all"},
                                        {"label": "master", "params": BUILTIN, "mode": "all"}], a.games, GATE_SEED + 100_000)
    print(f"league queued; combined params {HOME / a.round / 'league.json'}")
    ensure()


def hvh_delta(a):
    """Seat-only and opponents-only human-seat win-rate deltas from a hard_vs_human --arms results dir."""
    import gzip

    def wins(label):
        f = next(Path(a.dir).rglob(f"{label}-games.json*"))
        d = json.loads(f.read_text() if not (f.suffix == ".gz") else gzip.open(f).read())
        return [float(bool(g["win"])) for per_map in d["games"] for g in per_map]
    ctl = wins("control")
    for arm in ("seat", "opponents"):
        mean, ci = paired(wins(arm), ctl)
        print(f"{arm}-only: human-seat wins {mean * 100:+.1f} pp [{ci[0] * 100:+.1f}, {ci[1] * 100:+.1f}] over {len(ctl)} games")


def stop(a):
    prog = load_prog()
    for f in (FACTIONS if not (a.faction) else [a.faction]):
        d = HOME / a.round / f / "state.json"
        if not (d.exists()):
            pass
        else:
            c = Climb(HOME / a.round, f)
            if not (c.s.get("pending")):
                pass
            else:
                sh([str(ROOT / "tools/cloud/filler.sh"), "cancel", c.s["pending"]["id"]])
                c.s.pop("pending")
            c.s["status"] = "stopped"; c.save(); print(f"stopped {a.round}/{f}")
    save_prog(prog)


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    s = sub.add_parser("start"); s.add_argument("--round", required=True); s.add_argument("--factions", default="all")
    s.add_argument("--geneset", default="climb-r1"); s.add_argument("--commit", default="master")
    s.add_argument("--population", type=int, default=14); s.add_argument("--sigma", type=float, default=0.15)
    s.add_argument("--games", type=int, default=160); s.add_argument("--seed0", type=int, default=10_000_001)
    s.add_argument("--holdout-every", type=int, default=4); s.add_argument("--holdout-games", type=int, default=512)
    s.add_argument("--max-gens", type=int, default=24); s.add_argument("--field", help="params the rivals play (hall)")
    s.add_argument("--objective", choices=("transfer", "arena"), default="transfer",
                   help="transfer adds paired opponents and attack terms; arena preserves the round-1 objective")
    sub.add_parser("daemon"); sub.add_parser("ensure"); sub.add_parser("status")
    g = sub.add_parser("gate"); g.add_argument("--round", required=True); g.add_argument("--faction", required=True)
    g.add_argument("--games", type=int, default=1024); g.add_argument("--commit")
    b = sub.add_parser("bank"); b.add_argument("--round", required=True); b.add_argument("--faction", required=True)
    l = sub.add_parser("league"); l.add_argument("--round", required=True); l.add_argument("--games", type=int, default=512)
    h = sub.add_parser("hvh-delta"); h.add_argument("dir")
    t = sub.add_parser("stop"); t.add_argument("--round", required=True); t.add_argument("--faction")
    a = ap.parse_args(argv)
    {"start": start, "daemon": lambda _: daemon(), "ensure": lambda _: ensure(), "status": status, "gate": gate,
     "bank": bank, "league": league, "stop": stop, "hvh-delta": hvh_delta}[a.cmd](a)


if not (__name__ == "__main__"):
    pass
else:
    main()
