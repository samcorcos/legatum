"""Turn legatum-sim report JSON into compact game records, candidate metrics and fitness.

Every number the run log shows is computed here from simulator reports, so it can be traced
back to a game (pillar 5). Archetype names are the classifier's (crates/sim/src/diversity.rs),
title-cased for the run log: Rush, Wide, Science, Turtle, Economic, Generalist.
"""

from __future__ import annotations

import math
import statistics

ARCHETYPES = ("Rush", "Wide", "Science", "Turtle", "Economic", "Generalist")
DETECTOR_BEHAVIORS = (
    "unanswered_attack", "scout_not_fleeing", "garrison_under_ranged_fire",
    "idle_military_threatened_city", "repeated_bounce", "movement_oscillation",
    "gold_hoarded_under_attack", "settler_wandering", "research_idle", "production_idle",
)

DEFINITIONS = {
    "candidate": "one genome (40 faction personality knobs plus 29 shared decision genes), played by every scripted seat it holds at Hard",
    "fitness": "w_proxy*(1 - proxy_win_rate) + w_survival*ai_survival_t150 + w_league*league_win_rate - penalties (faction ratio over 1.5, archetype entropy under the floor, a dominant archetype, stalls/panics, median length outside the window); weights in config.fitness",
    "best.proxy_win_rate": "share of the candidate's Proxy games (Proxy v2 rotated seat/faction vs 7 candidate AIs at Hard, Standard) that the Proxy won",
    "best.league_win_rate": "in head-to-head league games (candidate holds 4 alternating seats, a league member the other 4), share of decided games a candidate seat won; 0.5 is parity",
    "best.faction_ratio": "best / worst faction win rate over the candidate's seats in all its games, each shrunk toward the pooled rate ((wins + 10*pooled)/(seats + 10)); pillar 8 target <= 1.5 (the penalty is capped, since training samples are small)",
    "ai_survival_t150": "share of AI seats not eliminated before turn 150 (games that end earlier count survivors at the end)",
    "archetype_share": "classifier archetypes of the generation's candidate seats (all candidates, Proxy and league games)",
    "winner_entropy": "normalized Shannon entropy (K=7) of the archetypes of AI seats that won the generation's games",
    "matchup_matrix.ahead_rate": "ahead_rate[i][j]: over pairs of the candidate's seats in the same game with archetypes i and j, how often i finished ahead of j (winners, then survivors by VP, then later elimination; ties count half)",
    "gene_spread": "CMA-ES sampling standard deviation per gene, in its native units",
}


def title(arch: str | None) -> str | None:
    return arch[:1].upper() + arch[1:] if arch else None


def compact_games(report: dict) -> list[dict]:
    """The few fields the harness needs from each game record."""
    out = []
    for g in report.get("games", []):
        elim = {e["player"]: e["turn"] for e in g.get("eliminations", [])}
        proxy = ((g.get("telemetry") or {}).get("proxy")) or {}
        incidents = proxy.get("ai_incidents")
        diag = proxy.get("diagnostics") or {}
        players = [{
            "id": p["id"], "faction": p["faction"], "brain": p["brain"],
            "arch": title(p.get("archetype")), "alive": p["alive"], "vp": p["vp"],
            "cities": p.get("cities"), "elim": elim.get(p["id"]),
        } for p in g.get("players", [])]
        out.append({
            "i": g["index"], "seed": str(g["seed"]), "turns": g["turns"], "outcome": g["outcome"],
            "winners": list(g.get("winner_players", [])),
            "players": players,
            "proxy_seat": next((p["id"] for p in players if p["brain"] == "proxy"), None),
            "incidents": sum(incidents.values()) if isinstance(incidents, dict) else None,
            "incident_counts": incidents if isinstance(incidents, dict) else None,
            "first_capture": diag.get("first_capture"),
        })
    return out


def report_summary(report: dict) -> dict:
    """Run-level numbers: Proxy summary snapshots, incident rates, versions."""
    s = report.get("summary", {})
    proxy = s.get("proxy") or {}
    snap = lambda key, turn: (proxy.get(key) or {}).get(str(turn)) or {}
    rates: dict[str, dict[str, int]] = {}
    for r in (report.get("ai_analysis") or {}).get("rates", []):
        if r.get("attacker_faction") is not None:
            continue
        x = rates.setdefault(r["behavior"], {"incidents": 0, "opportunities": 0})
        x["incidents"] += r["incidents"]
        x["opportunities"] += r["opportunities"]
    meta = report.get("meta", {})
    return {
        "games": s.get("games"),
        "proxy_win_rate": proxy.get("win_rate"),
        "ai_cities_t100": snap("ai_cities_alive", 100).get("mean"),
        "ai_cities_t100_obs": snap("ai_cities_alive", 100).get("observations"),
        "ai_cities_t150": snap("ai_cities_alive", 150).get("mean"),
        "ai_cities_t150_obs": snap("ai_cities_alive", 150).get("observations"),
        "incident_rates": rates,
        "proxy_version": meta.get("proxy_version"),
        "source": meta.get("source"),
    }


def rank_key(p: dict, winners: list[int]) -> tuple:
    if p["id"] in winners:
        return (2, 0)
    if p["alive"]:
        return (1, p["vp"])
    return (0, p["elim"] or 0)


def entropy(counts: dict[str, int], k: int = len(ARCHETYPES)) -> float | None:
    total = sum(counts.values())
    if not total:
        return None
    h = -sum((c / total) * math.log(c / total) for c in counts.values() if c)
    return h / math.log(k)


def matchup(games: list[dict], is_ours) -> dict:
    """Pairwise ahead rates between archetypes of our seats that shared a game."""
    ahead: dict[tuple[str, str], float] = {}
    pairs: dict[tuple[str, str], int] = {}
    for g in games:
        seats = [p for p in g["players"] if is_ours(g, p) and p["arch"]]
        for a in seats:
            for b in seats:
                if a is b or a["arch"] == b["arch"]:
                    continue
                key = (a["arch"], b["arch"])
                ka, kb = rank_key(a, g["winners"]), rank_key(b, g["winners"])
                pairs[key] = pairs.get(key, 0) + 1
                ahead[key] = ahead.get(key, 0.0) + (1.0 if ka > kb else 0.5 if ka == kb else 0.0)
    names = [a for a in ARCHETYPES if any(a in k for k in pairs)]
    return {
        "archetypes": names,
        "ahead_rate": [[round(ahead[(a, b)] / pairs[(a, b)], 3) if pairs.get((a, b)) else None for b in names] for a in names],
        "pairs": [[pairs.get((a, b), 0) for b in names] for a in names],
        "definition": DEFINITIONS["matchup_matrix.ahead_rate"],
    }


def niche(counts: dict[str, int]) -> str | None:
    """The candidate's quality-diversity niche: its most common real strategy. Generalist (the
    classifier's residual) only when no seat played anything else."""
    real = {a: c for a, c in counts.items() if a != "Generalist" and c}
    pool = real or counts
    return max(sorted(pool), key=lambda a: pool[a]) if pool else None


def dominant_archetype(m: dict, min_pairs: int) -> str | None:
    """A strategy ahead (>50%) of every other sufficiently sampled archetype (at least two).
    Generalist is a residual label, not a strategy, so it can be a rival but never dominant."""
    names = m["archetypes"]
    for i, a in enumerate(names):
        if a == "Generalist":
            continue
        rivals = [j for j in range(len(names)) if j != i and m["pairs"][i][j] >= min_pairs]
        if len(rivals) >= 2 and all(m["ahead_rate"][i][j] > 0.5 for j in rivals):
            return a
    return None


def candidate_metrics(proxy_games: list[dict], league_games: list[dict], label: str,
                      proxy_summary: dict | None, cfg: dict) -> dict:
    ours = lambda g, p: p["brain"] == label
    seats = [(g, p) for g in proxy_games + league_games for p in g["players"] if ours(g, p)]
    ai_proxy = [p for g in proxy_games for p in g["players"] if ours(g, p)]
    proxy_wins = sum(1 for g in proxy_games if g["proxy_seat"] in g["winners"])
    decided = [g for g in league_games if g["winners"]]
    league_wins = sum(1 for g in decided if any(ours(g, p) and p["id"] in g["winners"] for p in g["players"]))
    fac: dict[str, list[int]] = {}
    for g, p in seats:
        f = fac.setdefault(p["faction"], [0, 0])
        f[0] += p["id"] in g["winners"]
        f[1] += 1
    # Shrink each faction toward the pooled rate (a prior worth 10 seats): at a few hundred
    # seats a faction has only a handful of wins, and raw ratios swing by 3x on noise alone.
    pooled = sum(w for w, _ in fac.values()) / max(1, sum(n for _, n in fac.values()))
    smoothed = {k: (w + 10 * pooled) / (n + 10) for k, (w, n) in fac.items()}
    counts: dict[str, int] = {}
    for _, p in seats:
        if p["arch"]:
            counts[p["arch"]] = counts.get(p["arch"], 0) + 1
    total = sum(counts.values())
    mm = matchup(proxy_games + league_games, ours)
    games = proxy_games + league_games
    lengths = [g["turns"] for g in league_games] or [g["turns"] for g in proxy_games]
    return {
        "proxy_games": len(proxy_games),
        "league_games": len(league_games),
        "proxy_win_rate": proxy_wins / len(proxy_games) if proxy_games else None,
        "ai_survival_t150": (sum(1 for p in ai_proxy if p["elim"] is None or p["elim"] >= 150) / len(ai_proxy)) if ai_proxy else None,
        "ai_cities_t100": (proxy_summary or {}).get("ai_cities_t100"),
        "ai_cities_t150": (proxy_summary or {}).get("ai_cities_t150"),
        "league_win_rate": league_wins / len(decided) if decided else None,
        "faction_win_rates": {k: round(w / n, 4) for k, (w, n) in sorted(fac.items())},
        "faction_ratio": (max(smoothed.values()) / min(smoothed.values())) if smoothed and min(smoothed.values()) > 0 else None,
        "archetype_counts": counts,
        "archetype_share": {a: round(counts.get(a, 0) / total, 4) for a in ARCHETYPES} if total else {},
        "archetype_entropy": entropy(counts),
        "archetype": niche(counts),
        "dominant": dominant_archetype(mm, cfg.get("dominance_min_pairs", 20)),
        "matchup": mm,
        "stalls": sum(1 for g in games if g["outcome"] == "stalled"),
        "panics": sum(1 for g in games if g["outcome"] == "panic"),
        "median_turns": statistics.median(lengths) if lengths else None,
    }


DEFAULT_FITNESS = {
    "w_proxy": 1.0, "w_survival": 0.3, "w_league": 0.5,
    "w_faction": 0.05, "faction_max": 1.5, "faction_penalty_cap": 0.15,
    "w_entropy": 0.5, "min_entropy": 0.55,
    "w_dominance": 0.05, "dominance_min_pairs": 20,
    "w_stall": 2.0,
    "w_length": 0.1, "length_min": 60, "length_max": 300,
}


def fitness(m: dict, cfg: dict) -> tuple[float, dict]:
    c = {**DEFAULT_FITNESS, **cfg}
    pwr = m["proxy_win_rate"] if m["proxy_win_rate"] is not None else 1.0
    surv = m["ai_survival_t150"] or 0.0
    lwr = m["league_win_rate"] if m["league_win_rate"] is not None else 0.5
    games = max(1, m["proxy_games"] + m["league_games"])
    parts = {
        "proxy": c["w_proxy"] * (1 - pwr),
        "survival": c["w_survival"] * surv,
        "league": c["w_league"] * lwr,
        "pen_faction": -min(c["faction_penalty_cap"], c["w_faction"] * max(0.0, (m["faction_ratio"] or 1.0) - c["faction_max"])),
        "pen_entropy": -c["w_entropy"] * max(0.0, c["min_entropy"] - (m["archetype_entropy"] if m["archetype_entropy"] is not None else 0.0)),
        "pen_dominance": -c["w_dominance"] if m["dominant"] else 0.0,
        "pen_stall": -c["w_stall"] * (m["stalls"] + m["panics"]) / games,
        "pen_length": -c["w_length"] if m["median_turns"] is not None and not c["length_min"] <= m["median_turns"] <= c["length_max"] else 0.0,
    }
    return round(sum(parts.values()), 4), {k: round(v, 4) for k, v in parts.items()}


def quantile(xs: list[float], q: float) -> float | None:
    if not xs:
        return None
    s = sorted(xs)
    i = (len(s) - 1) * q
    lo = math.floor(i)
    hi = min(lo + 1, len(s) - 1)
    return s[lo] + (s[hi] - s[lo]) * (i - lo)
