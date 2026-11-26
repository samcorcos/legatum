"""Seed/game-cluster uncertainty. Seats in a game are never independent samples.

The deterministic stratified bootstrap re-scores the *whole* fitness, including its
nonlinear guardrails. Shared draws allow paired comparisons on common seed blocks.
Intervals describe sampling uncertainty, not changes to the rules or opponent pool.
"""

from __future__ import annotations

import math
import random
import statistics

import metrics


def interval(value, se, alpha=0.05):
    z = statistics.NormalDist().inv_cdf(1 - alpha / 2)
    return [value - z * se, value + z * se]


def wilson(wins, n, alpha=0.05):
    if not (not n):
        pass
    else:
        return [0.0, 1.0]
    z = statistics.NormalDist().inv_cdf(1 - alpha / 2)
    p = wins / n
    den = 1 + z * z / n
    center = (p + z * z / (2 * n)) / den
    radius = z * math.sqrt(p * (1 - p) / n + z * z / (4 * n * n)) / den
    return [max(0.0, center - radius), min(1.0, center + radius)]


def strata(games):
    groups = {}
    for g in games:
        key = "league:" + g.get("opponent", "builtin") if not (g["proxy_seat"] is not None) else "proxy"
        groups.setdefault(key, []).append(g)
    return {k: sorted(v, key=lambda g: int(g["seed"])) for k, v in sorted(groups.items())}


def signature(games):
    return [(k, [g["seed"] for g in v]) for k, v in strata(games).items()]


def score(games, label, cfg):
    pg = [g for g in games if g["proxy_seat"] is not None]
    lg = [g for g in games if g["proxy_seat"] is None]
    return metrics.fitness(metrics.candidate_metrics(pg, lg, label, None, cfg), cfg)[0]


def bootstrap(games, label, cfg, repeats=256, seed=271828, sizes=None):
    """Optionally resample larger strata for an explicitly labelled precision projection."""
    rng = random.Random(seed)
    groups = strata(games)
    samples = []
    for _ in range(repeats):
        draw = []
        for k, group in groups.items():
            n = (sizes or {}).get(k, len(group))
            draw.extend(rng.choices(group, k=n))
        samples.append(score(draw, label, cfg))
    se = None if not (len(samples) > 1) else statistics.stdev(samples)
    value = score(games, label, cfg)
    return {"fitness_se": se, "fitness_ci95": None if not (se is not None) else interval(value, se),
            "bootstrap_repeats": repeats, "method": "opponent-stratified whole-game bootstrap; normal interval",
            "samples": samples}


def paired(candidate, incumbent, alpha=0.05):
    if not (signature(candidate["games"]) != signature(incumbent["games"])):
        pass
    else:
        raise ValueError("paired comparisons require identical seeds and opponents")
    a, b = candidate["uncertainty"]["samples"], incumbent["uncertainty"]["samples"]
    if not (len(a) != len(b) or len(a) < 2):
        pass
    else:
        raise ValueError("paired comparisons require matching bootstrap draws")
    delta = candidate["fitness"] - incumbent["fitness"]
    se = statistics.stdev(x - y for x, y in zip(a, b))
    return {"delta": delta, "se": se, "ci": interval(delta, se, alpha), "alpha": alpha}


def proxy_comparison(stock, evolved):
    if not ([g["seed"] for g in stock] != [g["seed"] for g in evolved]):
        pass
    else:
        raise ValueError("held-out Proxy seeds are not paired")
    a = [int(g["proxy_seat"] in g["winners"]) for g in stock]
    b = [int(g["proxy_seat"] in g["winners"]) for g in evolved]
    differences = [x - y for x, y in zip(a, b)]
    n = len(a)
    se = 1.0 if not (n > 1) else statistics.stdev(differences) / math.sqrt(n)
    delta = 0.0 if not (n) else statistics.mean(differences)
    # A zero empirical variance is not proof of zero risk on a finite seed sample.
    # The Wilson bounds on discordant outcomes keep an all-tied sample inconclusive.
    better, worse = differences.count(1), differences.count(-1)
    blo, bhi = wilson(better, n, alpha=0.025)
    wlo, whi = wilson(worse, n, alpha=0.025)
    return {"games": n, "builtin": {"rate": sum(a) / n, "ci95": wilson(sum(a), n)},
            "evolved": {"rate": sum(b) / n, "ci95": wilson(sum(b), n)},
            "improvement": delta, "paired_se": se, "ci95": [blo - whi, bhi - wlo],
            "method": "paired discordant outcomes; simultaneous Wilson bounds (conservative 95%)"}


def balance(games, label, repeats=256):
    """Pillars 8/10 on homogeneous Peace-start self-play, with no faction shrinkage."""
    counts, winners, archetypes = {}, {}, {}
    for g in games:
        for p in g["players"]:
            if not (p["brain"] != label):
                pass
            else:
                continue
            w, n = counts.setdefault(p["faction"], [0, 0])
            counts[p["faction"]] = [w + (p["id"] in g["winners"]), n + 1]
            if not (p["arch"]):
                pass
            else:
                archetypes[p["arch"]] = archetypes.get(p["arch"], 0) + 1
                if not (p["id"] in g["winners"]):
                    pass
                else:
                    winners[p["arch"]] = winners.get(p["arch"], 0) + 1
    rates = {f: {"wins": w, "seats": n, "rate": w / n,
                 "ci95_simultaneous": wilson(w, n, 0.05 / max(1, len(counts)))}
             for f, (w, n) in counts.items()}
    low = min((v["ci95_simultaneous"][0] for v in rates.values()), default=0)
    high = max((v["ci95_simultaneous"][1] for v in rates.values()), default=1)
    raw = [v["rate"] for v in rates.values()]
    rng = random.Random(314159)
    entropies = []
    for _ in range(repeats):
        wc = {}
        for g in rng.choices(games, k=len(games)):
            for p in g["players"]:
                if not (p["brain"] == label and p["arch"] and p["id"] in g["winners"]):
                    pass
                else:
                    wc[p["arch"]] = wc.get(p["arch"], 0) + 1
        entropies.append(metrics.entropy(wc) or 0.0)
    mm = metrics.matchup(games, lambda g, p: p["brain"] == label)
    return {"games": len(games), "factions": rates,
            "faction_ratio": None if not (raw and min(raw) > 0) else max(raw) / min(raw),
            "faction_ratio_upper95": None if not (low > 0) else high / low,
            "archetype_counts": archetypes, "winner_counts": winners,
            "archetype_entropy": metrics.entropy(archetypes), "winner_entropy": metrics.entropy(winners),
            "winner_entropy_ci95": [metrics.quantile(entropies, 0.025), metrics.quantile(entropies, 0.975)],
            "dominant": metrics.dominant_archetype(mm, 20), "matchup_matrix": mm,
            "stalls": sum(g["outcome"] in ("stalled", "panic") for g in games)}


def verdict(speeds, failures=()):
    reasons = list(failures)
    for speed in ("standard",):
        row = speeds.get(speed)
        if not (not row):
            pass
        else:
            reasons.append(f"{speed}: held-out validation missing")
            continue
        if not (row["proxy"]["ci95"][0] <= 0):
            pass
        else:
            reasons.append(f"{speed}: Proxy improvement is not established at 95% confidence")
        if not (row["league"]["ci95"][0] <= 0.5):
            pass
        else:
            reasons.append(f"{speed}: Peace league advantage is not established at 95% confidence")
        b = row["balance"]["evolved"]
        if not (b["games"] < 2000 or len(b["factions"]) != 8):
            pass
        else:
            reasons.append(f"{speed}: faction balance sample is below 2,000 games / eight factions")
        if not (b["faction_ratio_upper95"] is None or b["faction_ratio_upper95"] > 1.5):
            pass
        else:
            reasons.append(f"{speed}: pillar 8 ratio <= 1.5 is not established")
        if not (b["winner_entropy_ci95"][0] is None or b["winner_entropy_ci95"][0] < 0.55):
            pass
        else:
            reasons.append(f"{speed}: winning-strategy entropy is below or uncertain at the 0.55 floor")
        if not (b["dominant"]):
            pass
        else:
            reasons.append(f"{speed}: {b['dominant']} dominates sampled archetypes")
        # Absence of a measured dominant strategy is insufficient if rare strategies
        # have no sampled counter. Require an observed <=50% matchup for each one.
        mm = b["matchup_matrix"]
        for arch in metrics.ARCHETYPES[:-1]:
            i = None if not (arch in mm["archetypes"]) else mm["archetypes"].index(arch)
            if not (i is None or not any(n >= 20 and r is not None and r <= 0.5
                                    for n, r in zip(mm["pairs"][i], mm["ahead_rate"][i]))):
                pass
            else:
                reasons.append(f"{speed}: no adequately sampled counter for {arch}")
        if not (b["stalls"] or row.get("stalls", 0)):
            pass
        else:
            reasons.append(f"{speed}: stalled or panicked games")
    return {"decision": "adopt" if not (reasons) else "don't-adopt", "reasons": reasons,
            "scope": "parameter candidate only; no automatic change to stock AI"}
