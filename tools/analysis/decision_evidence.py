#!/usr/bin/env python3
"""Read decision-study reports, or apply/remove its observer-only instrumentation.

Usage: decision_evidence.py instrument|restore
       decision_evidence.py summarize REPORT.json[.gz]
       decision_evidence.py compare BASE.json[.gz] CANDIDATE.json[.gz]
No third-party packages. Instrumentation only records the first resolved turn
owning four cities; the development milestone can instead mean two tall cities.
"""
import argparse
import collections
import gzip
import hashlib
import json
import math
from pathlib import Path
import random
import statistics


Z = 1.959963984540054
PATCHES = [
    ("    peak_cities: BTreeMap<u8, usize>,\n",
     "    peak_cities: BTreeMap<u8, usize>,\n    four_city_turns: BTreeMap<u8, u32>,\n"),
    ("            let cities = state.cities_of(p.id).count();\n",
     "            let cities = state.cities_of(p.id).count();\n"
     "            if cities >= 4 {\n"
     "                self.four_city_turns.entry(p.id.0).or_insert(resolved_turn);\n"
     "            }\n"),
    ("            peak_cities: self.peak_cities.clone(),\n",
     "            peak_cities: self.peak_cities.clone(),\n"
     "            four_city_turns: self.four_city_turns.clone(),\n"),
    ("    pub peak_cities: BTreeMap<u8, usize>,\n",
     "    pub peak_cities: BTreeMap<u8, usize>,\n"
     "    #[serde(default)]\n    pub four_city_turns: BTreeMap<u8, u32>,\n"),
]


def instrument(restore=False):
    path = Path(__file__).resolve().parents[2] / "crates/sim/src/telemetry.rs"
    source = path.read_text()
    for old, new in PATCHES:
        before, after = (new, old) if restore else (old, new)
        assert source.count(before) == 1, f"unexpected source: {before!r}"
        assert restore or new not in source, "observer already present"
        source = source.replace(before, after)
    path.write_text(source)


def read(path):
    raw = Path(path).read_bytes()
    return json.loads(gzip.decompress(raw) if raw[:2] == b"\x1f\x8b" else raw)


def pct(xs, p):
    """Nearest-rank percentile; median is separately the central-pair average."""
    return sorted(xs)[max(0, math.ceil(len(xs) * p) - 1)] if xs else None


def dist(xs):
    xs = list(xs)
    return dict(n=len(xs), mean=statistics.mean(xs) if xs else None,
                median=statistics.median(xs) if xs else None, p90=pct(xs, .9))


def ci_mean(xs):
    xs = list(xs)
    if len(xs) < 2:
        return dict(n=len(xs), mean=statistics.mean(xs) if xs else None, ci95=None)
    mean = statistics.mean(xs)
    half = Z * statistics.stdev(xs) / math.sqrt(len(xs))
    return dict(n=len(xs), mean=mean, ci95=[mean-half, mean+half])


def wilson(k, n):
    p = k / n
    center = (p + Z*Z/(2*n)) / (1 + Z*Z/n)
    half = Z * math.sqrt(p*(1-p)/n + Z*Z/(4*n*n)) / (1 + Z*Z/n)
    return [center-half, center+half]


def four(g, seat):
    return g["telemetry"]["four_city_turns"].get(str(seat))


def reached(turns):
    turns = list(turns)
    ts = [t for t in turns if t is not None]
    return dict(total=len(turns), reached=len(ts), fraction=len(ts)/len(turns),
                turns=dist(ts), by_turn={t: sum(x <= t for x in ts)/len(turns)
                                      for t in [50, 75, 100, 136, 150, 200]})


def proxy(g):
    return g["telemetry"].get("proxy")


def won(g):
    return proxy(g)["seat"] in g["winner_players"]


def snapshot(g, turn):
    s = proxy(g)["snapshots"].get(str(turn))
    if s is None:
        if turn not in [25, 50, 75] or any(p["cities_at_turns"].get(str(turn)) is None for p in g["players"]):
            return None
        cities = {p["id"]: p["cities_at_turns"][str(turn)] for p in g["players"]}
        pc = cities[proxy(g)["seat"]]
        ai = (sum(cities.values())-pc)/7
        return dict(proxy=pc, ai=ai, gap=pc-ai,
                    captures=sum(t <= turn for t in proxy(g)["diagnostics"]["capture_turns"]))
    return dict(proxy=s["proxy_cities"], ai=s["ai_cities_alive"]/7,
                gap=s["proxy_cities"]-s["ai_cities_alive"]/7,
                captures=s["diagnostics"]["proxy_captures"])


def summarize(report):
    gs = report["games"]
    n = len(gs)
    assert n and len({g["seed"] for g in gs}) == n
    assert all(len(g["players"]) == 8 and len(g["winner_factions"]) <= 1 for g in gs)
    assert all(g["outcome"] != "panic" and "four_city_turns" in g["telemetry"] for g in gs)
    factions = sorted(p["faction"] for p in gs[0]["players"])
    assert len(set(factions)) == 8
    for g in gs:
        assert sorted(p["faction"] for p in g["players"]) == factions
        for p in g["players"]:
            seat = str(p["id"])
            assert (four(g, p["id"]) is not None) == (g["telemetry"]["peak_cities"][seat] >= 4)
            if four(g, p["id"]) is not None:
                assert 1 <= four(g, p["id"]) <= g["turns"]
    counts = collections.Counter(f for g in gs for f in g["winner_factions"])
    wins = {f: counts[f] for f in factions}
    rng = random.Random(20261001)
    outcomes = [g["winner_factions"][0] if g["winner_factions"] else None for g in gs]
    ratios = []
    for _ in range(2000):
        c = collections.Counter(rng.choices(outcomes, k=n))
        ratios.append(max(c[f] for f in factions)/min(c[f] for f in factions)
                      if min(c[f] for f in factions) else float("inf"))
    result = dict(games=n, outcomes=dict(collections.Counter(g["outcome"] for g in gs)),
                  turns=dist(g["turns"] for g in gs),
                  wars=dist(g["telemetry"]["diplomacy"]["war_starts"] for g in gs),
                  captures=dist(g["city_captures"] for g in gs),
                  settler_captures=dist(g["telemetry"]["settler_captures"] for g in gs),
                  factions={f: dict(wins=wins[f], rate=wins[f]/n, ci95=wilson(wins[f], n))
                            for f in factions},
                  ratio=max(wins.values())/min(wins.values()) if min(wins.values()) else None,
                  ratio_bootstrap95=[pct(ratios, .025), pct(ratios, .975)])
    result["four_cities"] = {f: reached(four(g, p["id"]) for g in gs
        for p in g["players"] if p["faction"] == f) for f in factions}
    result["all_eight_four_cities"] = reached(max(g["telemetry"]["four_city_turns"].values())
        if len(g["telemetry"]["four_city_turns"]) == 8 else None for g in gs)
    if proxy(gs[0]):
        w = sum(won(g) for g in gs)
        result["proxy"] = dict(wins=w, rate=w/n, ci95=wilson(w, n), snapshots={},
            four_cities=reached(four(g, proxy(g)["seat"]) for g in gs),
            ai_four_cities=reached(four(g, p["id"]) for g in gs
                                  for p in g["players"] if p["id"] != proxy(g)["seat"]),
            captures=dist(proxy(g)["proxy_captures"] for g in gs),
            settlers_captured=dist(proxy(g)["diagnostics"]["settlers_captured"] for g in gs))
        for t in [25, 50, 75, 100, 136, 150, 200]:
            ss = [s for g in gs if (s := snapshot(g, t)) is not None]
            result["proxy"]["snapshots"][t] = {key: ci_mean(s[key] for s in ss)
                                                  for key in ["proxy", "ai", "gap", "captures"]}
        result["proxy"]["factions"] = {f: dict(games=sum(proxy(g)["faction"] == f for g in gs),
            wins=sum(proxy(g)["faction"] == f and won(g) for g in gs)) for f in factions}
    return result


def compare(a, b):
    ga, gb = (sorted(r["games"], key=lambda g: int(g["seed"])) for r in [a, b])
    assert len(ga) == len(gb)
    for x, y in zip(ga, gb):
        assert x["seed"] == y["seed"]
        assert [[p[k] for k in ["id", "team", "faction", "difficulty", "brain"]]
                for p in x["players"]] == [[p[k] for k in ["id", "team", "faction", "difficulty", "brain"]]
                                          for p in y["players"]]
    result = dict(games=len(ga), turns=ci_mean(y["turns"]-x["turns"] for x, y in zip(ga, gb)),
        wars=ci_mean(y["telemetry"]["diplomacy"]["war_starts"]-x["telemetry"]["diplomacy"]["war_starts"]
                     for x, y in zip(ga, gb)),
        captures=ci_mean(y["city_captures"]-x["city_captures"] for x, y in zip(ga, gb)))
    result["faction_win_changes"] = {f: ci_mean(int(f in y["winner_factions"])-int(f in x["winner_factions"])
        for x, y in zip(ga, gb)) for f in sorted(p["faction"] for p in ga[0]["players"])}
    if proxy(ga[0]):
        assert all(proxy(x)["seat"] == proxy(y)["seat"] and proxy(x)["faction"] == proxy(y)["faction"]
                   for x, y in zip(ga, gb))
        result["proxy_wins"] = ci_mean(int(won(y))-int(won(x)) for x, y in zip(ga, gb))
        result["proxy_discordant"] = dict(collections.Counter(f"{int(won(x))}{int(won(y))}"
                                                              for x, y in zip(ga, gb)))
        result["snapshots"] = {}
        for t in [25, 50, 75, 100, 136, 150, 200]:
            pairs = [(sx, sy) for x, y in zip(ga, gb)
                     if (sx := snapshot(x, t)) is not None and (sy := snapshot(y, t)) is not None]
            result["snapshots"][t] = {key: ci_mean(y[key]-x[key] for x, y in pairs)
                                      for key in ["proxy", "ai", "gap", "captures"]}
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=["instrument", "restore", "summarize", "compare"])
    parser.add_argument("reports", nargs="*")
    args = parser.parse_args()
    expected = {"instrument": 0, "restore": 0, "summarize": 1, "compare": 2}[args.mode]
    if len(args.reports) != expected:
        parser.error(f"{args.mode} requires {expected} report paths")
    if args.mode in ["instrument", "restore"]:
        instrument(args.mode == "restore")
    else:
        result = (summarize(read(args.reports[0])) if args.mode == "summarize"
                  else compare(*(read(p) for p in args.reports)))
        result["input_sha256"] = {p: hashlib.sha256(Path(p).read_bytes()).hexdigest() for p in args.reports}
        print(json.dumps(result, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
