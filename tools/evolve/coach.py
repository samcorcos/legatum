"""The LLM coach: every K generations, read ~20 benchmark games of the current best genome
(detector reports, metric deltas, archetype shares) and write structured observations.

Providers: `claude` (default; Claude CLI headless, read-only tools) and `codex` (Astra,
`codex exec -s read-only`, for when it has usage again). Whatever the model returns is
validated against the facts it was given: behavior names must come from the classifier or
detector vocabulary, evidence must cite a sampled game, a real seat and a turn inside that
game, and metric numbers are replaced by the logged values. If the call fails or nothing
valid comes back, a rule-based coach over the same facts writes the cycle instead, and the
failure is recorded, so evolution never stops for the coach.
"""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import tempfile
import time
from pathlib import Path

from metrics import ARCHETYPES, DETECTOR_BEHAVIORS

KINDS = ("emerged", "disappeared", "dominant", "countered", "weakness")
PROPOSAL_KINDS = ("new_behavior", "tunable_parameter", "exploit", "design_lever")
CONFIDENCE = ("high", "medium", "low")
BEHAVIORS = tuple(f"{a} play" for a in ARCHETYPES) + DETECTOR_BEHAVIORS

_nullable = lambda t: {"type": [t, "null"]}
_obj = lambda props: {"type": "object", "additionalProperties": False, "required": list(props), "properties": props}
SCHEMA = _obj({
    "observations": {"type": "array", "maxItems": 8, "items": _obj({
        "kind": {"type": "string", "enum": list(KINDS)},
        "behavior": {"type": "string", "enum": list(BEHAVIORS)},
        "archetype": {"type": ["string", "null"], "enum": list(ARCHETYPES) + [None]},
        "evidence": {"type": "array", "maxItems": 5, "items": _obj({
            "game_id": {"type": "string"}, "turn": {"type": "integer"},
            "player": {"type": "integer"}, "detail": {"type": "string"},
        })},
        "metric_change": {"anyOf": [{"type": "null"}, _obj({"name": {"type": "string"}, "before": _nullable("number"), "after": _nullable("number")})]},
        "confidence": {"type": "string", "enum": list(CONFIDENCE)},
    })},
    "dominant_strategy": {"anyOf": [{"type": "null"}, _obj({
        "archetype": {"type": "string", "enum": list(ARCHETYPES)},
        "counter_proposal": {"type": "string"},
    })]},
    "proposals": {"type": "array", "maxItems": 8, "items": _obj({
        "kind": {"type": "string", "enum": list(PROPOSAL_KINDS)},
        "description": {"type": "string"},
        "expected_effect": {"type": "string"},
        "confidence": {"type": "string", "enum": list(CONFIDENCE)},
        "evidence": {"type": "array", "maxItems": 6, "items": _obj({
            "game_id": {"type": "string"}, "player": {"type": "integer"},
            "turns": {"type": "array", "maxItems": 20, "items": {"type": "integer"}},
        })},
    })},
    "summary": {"type": "string"},
})


# ---------------------------------------------------------------- sampling and detector

def sample_games(games: list[dict], limit: int = 20) -> list[tuple[dict, str]]:
    """Losses to the Proxy, close games, and the games with the most incidents."""
    proxy = [g for g in games if g["proxy_seat"] is not None]
    losses = [g for g in proxy if g["proxy_seat"] in g["winners"]]
    def margin(g):
        vps = sorted((p["vp"] for p in g["players"]), reverse=True)
        return vps[0] - vps[1] if len(vps) > 1 else 99
    ai_wins = sorted((g for g in proxy if g["winners"] and g["proxy_seat"] not in g["winners"]), key=margin)
    close_losses = sorted(losses, key=margin)
    busiest = sorted((g for g in games if g.get("incidents") is not None), key=lambda g: -g["incidents"])
    picked: list[tuple[dict, str]] = []
    seen = set()
    def take(gs, why, n):
        for g in gs:
            if len([1 for _, w in picked if w == why]) >= n or len(picked) >= limit:
                return
            if g["game_id"] in seen:
                continue
            seen.add(g["game_id"])
            picked.append((g, why))
    take(ai_wins, "AI win against the Proxy (closest margins first)", 4)
    take(close_losses, "close loss to the Proxy", 5)
    take(busiest, "most detector incidents", 5)
    step = max(1, len(losses) // 6)
    take(losses[::step], "loss to the Proxy (spread across seeds)", 6)
    take(games, "other benchmark game", limit)
    return picked


def run_detector(sim: str, archive: str, out_dir: Path) -> dict | None:
    out_dir.mkdir(parents=True, exist_ok=True)
    r = subprocess.run([sim, "--replay", archive, "--analyze", "--out", str(out_dir)],
                       capture_output=True, text=True, timeout=600)
    if r.returncode != 0:
        return None
    try:
        manifest = json.loads(r.stdout.strip().splitlines()[-1])
        return json.loads(Path(manifest["incidents"]).read_text())
    except (ValueError, KeyError, IndexError, OSError):
        return None


def detector_digest(report: dict | None, proxy_seat: int | None, per_behavior: int = 2) -> dict:
    if not report:
        return {"available": False}
    counts: dict[str, int] = {}
    examples: list[dict] = []
    seen: dict[str, int] = {}
    for inc in report.get("incidents", []):
        if inc.get("player") == proxy_seat:
            continue
        b = inc.get("behavior")
        counts[b] = counts.get(b, 0) + 1
        if seen.get(b, 0) < per_behavior:
            seen[b] = seen.get(b, 0) + 1
            examples.append({
                "id": inc.get("id"), "behavior": b, "turn": inc.get("turn"), "player": inc.get("player"),
                "faction": inc.get("faction"), "consecutive_turns": inc.get("consecutive_turns"),
                "evidence": str(inc.get("evidence", ""))[:240],
            })
    return {"available": True, "counts": counts, "examples": examples}


# ---------------------------------------------------------------- facts and prompt

def metric_table(before: dict | None, after: dict) -> list[dict]:
    """Named metrics with before/after values; observations may only cite these names."""
    rows = []
    r4 = lambda x: round(x, 4) if isinstance(x, float) else x
    def add(name, key):
        rows.append({"name": name, "before": r4((before or {}).get(key)), "after": r4(after.get(key))})
    add("proxy_win_rate", "proxy_win_rate")
    add("ai_survival_t150", "ai_survival_t150")
    add("ai_cities_t100", "ai_cities_t100")
    add("league_win_rate", "league_win_rate")
    add("archetype_entropy", "archetype_entropy")
    for a in ARCHETYPES:
        rows.append({"name": f"{a} share", "before": ((before or {}).get("archetype_share") or {}).get(a),
                     "after": (after.get("archetype_share") or {}).get(a)})
    for b in DETECTOR_BEHAVIORS:
        rate = lambda m: ((m or {}).get("incident_rates") or {}).get(b)
        pr = lambda r: round(r["incidents"] / r["opportunities"], 4) if r and r["opportunities"] else None
        rows.append({"name": f"{b} incidents per opportunity", "before": pr(rate(before)), "after": pr(rate(after))})
    return rows


def build_facts(run_id: str, gen: int, prev_gen: int | None, before: dict | None, after: dict,
                sampled: list[tuple[dict, str]], digests: dict[str, dict], genome_change: dict) -> dict:
    share = {a: s for a, s in (after.get("archetype_share") or {}).items() if a != "Generalist"}
    lead = max(sorted(share), key=share.get) if share else None
    return {
        "run_id": run_id,
        "gen": gen,
        "compared_with_gen": prev_gen,
        "vocabulary": {
            "archetypes": list(ARCHETYPES),
            "behaviors": list(BEHAVIORS),
            "observation_kinds": list(KINDS),
        },
        "metrics": metric_table(before, after),
        "dominant_candidate": {"archetype": lead, "share": share.get(lead)} if lead and share.get(lead, 0) >= 0.4 else None,
        "matchup_matrix": after.get("matchup"),
        "genome_change": genome_change,
        "games": [{
            "game_id": g["game_id"], "seed": g["seed"], "turns": g["turns"], "why": why,
            "outcome": g["outcome"], "proxy_seat": g["proxy_seat"], "winners": g["winners"],
            "players": [{k: p[k] for k in ("id", "faction", "brain", "arch", "vp", "alive", "elim")} for p in g["players"]],
            "detector": digests.get(g["game_id"], {"available": False}),
        } for g, why in sampled],
    }


def prompt(facts: dict) -> str:
    return f"""You are the coach of an evolutionary run that tunes Legatum's scripted AI (a Civ-6-like 4X).
The genome is only each faction's five personality knobs (expansion, military, science, economy,
aggression); rules, yields, costs and vision are never tuned (pillar 6: the AI gets harder only by
deciding better). The benchmark is a scripted human-proxy opponent (Proxy v2) against 7 AIs at Hard.
Lower Proxy win rate is better. Pillar 8: factions keep distinct styles (best/worst win rate <= 1.5x).
Pillar 10: no dominant strategy; every archetype needs a counter.

You may read the AI source (crates/ai/src, crates/sim/src/diversity.rs, crates/analysis) and
docs/ai-analysis.md, docs/strategy-diversity.md to ground explanations. Do not edit anything.

Compare generation {facts['gen']} with generation {facts['compared_with_gen']} using ONLY the facts
below. Rules:
- behavior must be one of vocabulary.behaviors: "<Archetype> play" for classifier archetypes, or a
  detector behavior key. archetype (optional) must be a vocabulary archetype.
- Every observation cites evidence from the listed games: exact game_id, a player seat id from that
  game, and a turn between 1 and that game's turns (use detector example turns where you can).
- metric_change.name must be a name from facts.metrics; its numbers will be replaced by the logged
  values, so pick the metric that shows the change. Use null when no metric applies.
- dominant_strategy: if facts.dominant_candidate is set (or one archetype clearly wins the matchup
  matrix), name it and propose a COUNTER: an AI behavior the scripted AI could adopt, or a design
  lever for the lead. Counters that need a rule change are design_lever proposals, never genes.
- proposals: new_behavior (a code change in crates/ai), tunable_parameter (a new decision knob worth
  exposing as a gene, with range and tradeoff), exploit, or design_lever (rules; lead only). Use
  "high" confidence only for findings backed by several cited incidents or games.
- summary: two or three plain sentences a player can read: what the AI does differently than
  {facts['gen'] - (facts['compared_with_gen'] or 0)} generations ago and why it wins or loses more.
  Cite numbers only from facts.metrics.
Return an empty list rather than guessing.

FACTS:
{json.dumps(facts, separators=(',', ':'))}
"""


# ---------------------------------------------------------------- providers

def call_claude(text: str, cwd: Path, timeout: int, model: str) -> tuple[dict, dict]:
    exe = os.environ.get("LEGATUM_CLAUDE") or shutil.which("claude")
    if not exe:
        raise RuntimeError("claude CLI not found on PATH (set LEGATUM_CLAUDE)")
    cmd = [exe, "-p", "--model", model, "--output-format", "json",
           "--allowedTools", "Read,Grep,Glob", "--disallowedTools", "Bash,Edit,Write",
           "--max-turns", "30", "--setting-sources", "user",
           "--settings", json.dumps({"disableAllHooks": True}),
           "--no-session-persistence", "--json-schema", json.dumps(SCHEMA)]
    t0 = time.time()
    r = subprocess.run(cmd, input=text, capture_output=True, text=True, cwd=cwd, timeout=timeout)
    if r.returncode != 0:
        raise RuntimeError(f"claude exited {r.returncode}: {(r.stderr or r.stdout)[-600:]}")
    env = json.loads(r.stdout)
    if env.get("is_error"):
        raise RuntimeError(f"claude error: {str(env.get('result'))[:600]}")
    out = env.get("structured_output")
    if out is None:
        out = _json_in(env.get("result") or "")
    meta = {"provider": "claude", "model": model, "cost_usd": env.get("total_cost_usd"),
            "turns": env.get("num_turns"), "seconds": round(time.time() - t0, 1)}
    return out, meta


def call_codex(text: str, cwd: Path, timeout: int, model: str = "gpt-6-astra") -> tuple[dict, dict]:
    exe = os.environ.get("LEGATUM_CODEX") or shutil.which("codex")
    if not exe:
        raise RuntimeError("codex CLI not found on PATH (set LEGATUM_CODEX)")
    with tempfile.TemporaryDirectory() as tmp:
        schema = Path(tmp, "schema.json")
        schema.write_text(json.dumps(SCHEMA))
        out = Path(tmp, "out.json")
        t0 = time.time()
        r = subprocess.run([exe, "exec", "-m", model, "-s", "read-only", "-C", str(cwd),
                            "--output-schema", str(schema), "-o", str(out), "-"],
                           input=text, capture_output=True, text=True, timeout=timeout)
        if r.returncode != 0 or not out.exists():
            raise RuntimeError(f"codex exited {r.returncode}: {(r.stderr or r.stdout)[-600:]}")
        return _json_in(out.read_text()), {"provider": "codex", "model": model, "seconds": round(time.time() - t0, 1)}


def _json_in(text: str) -> dict:
    text = text.strip()
    try:
        return json.loads(text)
    except json.JSONDecodeError:
        start, end = text.find("{"), text.rfind("}")
        if start < 0 or end <= start:
            raise ValueError("no JSON object in the model's reply")
        return json.loads(text[start:end + 1])


# ---------------------------------------------------------------- validation

def validate(raw: dict, facts: dict) -> tuple[dict, list[str]]:
    """Keep only what the facts support. Returns (coach fields, list of dropped items)."""
    dropped: list[str] = []
    games = {g["game_id"]: g for g in facts["games"]}
    metrics = {m["name"]: m for m in facts["metrics"]}
    def seat_ok(g, player):
        return isinstance(player, int) and any(p["id"] == player for p in g["players"])
    observations = []
    for o in (raw.get("observations") or [])[:8]:
        if not isinstance(o, dict) or o.get("kind") not in KINDS or o.get("behavior") not in BEHAVIORS:
            dropped.append(f"observation with unknown kind/behavior: {str(o)[:120]}")
            continue
        evidence = []
        for e in o.get("evidence") or []:
            g = games.get(e.get("game_id")) if isinstance(e, dict) else None
            turn = e.get("turn") if g else None
            if g and isinstance(turn, int) and 1 <= turn <= g["turns"] and seat_ok(g, e.get("player")):
                evidence.append({"game_id": g["game_id"], "turn": turn, "player": e["player"], "detail": str(e.get("detail", ""))[:400]})
            else:
                dropped.append(f"evidence not in the sampled games: {str(e)[:120]}")
        if not evidence:
            dropped.append(f"observation without valid evidence: {o.get('behavior')}")
            continue
        mc = o.get("metric_change")
        if isinstance(mc, dict) and mc.get("name") in metrics:
            m = metrics[mc["name"]]
            mc = {"name": m["name"], "before": m["before"], "after": m["after"]}
        else:
            mc = None
        arch = o.get("archetype") if o.get("archetype") in ARCHETYPES else None
        observations.append({"kind": o["kind"], "behavior": o["behavior"], **({"archetype": arch} if arch else {}),
                             "evidence": evidence, "metric_change": mc,
                             "confidence": o.get("confidence") if o.get("confidence") in CONFIDENCE else "low"})
    proposals = []
    for p in (raw.get("proposals") or [])[:8]:
        if not isinstance(p, dict) or p.get("kind") not in PROPOSAL_KINDS or not str(p.get("description", "")).strip():
            dropped.append(f"malformed proposal: {str(p)[:120]}")
            continue
        ev = []
        for e in p.get("evidence") or []:
            g = games.get(e.get("game_id")) if isinstance(e, dict) else None
            turns = [t for t in (e.get("turns") or []) if isinstance(t, int) and g and 1 <= t <= g["turns"]] if g else []
            if g and seat_ok(g, e.get("player")) and turns:
                ev.append({"game_id": g["game_id"], "player": e["player"], "turns": sorted(set(turns))})
        conf = p.get("confidence") if p.get("confidence") in CONFIDENCE else "low"
        if not ev and conf == "high":
            conf = "medium"  # a high-confidence proposal must cite games
        proposals.append({"kind": p["kind"], "description": str(p["description"])[:1200],
                          "expected_effect": str(p.get("expected_effect", ""))[:600],
                          "confidence": conf, "evidence": ev})
    dominant = None
    d = raw.get("dominant_strategy")
    fact_dom = facts.get("dominant_candidate")
    if isinstance(d, dict) and d.get("archetype") in ARCHETYPES:
        share = next((m["after"] for m in facts["metrics"] if m["name"] == f"{d['archetype']} share"), None)
        dominant = {"archetype": d["archetype"], "share": share, "counter_proposal": str(d.get("counter_proposal") or "")[:800] or None}
    elif fact_dom:
        dominant = {**fact_dom, "counter_proposal": None}
    summary = " ".join(str(raw.get("summary") or "").split())[:900]
    return {"observations": observations, "dominant_strategy": dominant, "proposals": proposals, "summary": summary}, dropped


# ---------------------------------------------------------------- rule-based fallback

def rule_based(facts: dict, reason: str | None) -> dict:
    obs = []
    metrics = {m["name"]: m for m in facts["metrics"]}
    def ev_for_arch(a):
        out = []
        for g in facts["games"]:
            for p in g["players"]:
                if p["arch"] == a and p["brain"] != "proxy":
                    out.append({"game_id": g["game_id"], "turn": min(g["turns"], 120), "player": p["id"],
                                "detail": f"{p['faction']} classified {a} (classifier window ends turn {min(g['turns'], 120)}), {p['vp']} VP"})
        return out[:3]
    def ev_for_behavior(b):
        out = []
        for g in facts["games"]:
            for x in (g["detector"].get("examples") or []):
                if x["behavior"] == b and x.get("turn"):
                    out.append({"game_id": g["game_id"], "turn": x["turn"], "player": x["player"], "detail": f"{x['faction']}: {x['evidence']} [{x['id']}]"})
        return out[:2]
    for a in ARCHETYPES:
        m = metrics[f"{a} share"]
        if m["before"] is None or m["after"] is None or abs(m["after"] - m["before"]) < 0.08:
            continue
        kind = ("dominant" if m["after"] >= 0.4 and a != "Generalist" else "emerged") if m["after"] > m["before"] else ("disappeared" if m["after"] < 0.05 else None)
        ev = ev_for_arch(a)
        if kind and ev:
            obs.append((abs(m["after"] - m["before"]), {"kind": kind, "behavior": f"{a} play", "archetype": a, "evidence": ev, "metric_change": m, "confidence": "low"}))
    for b in DETECTOR_BEHAVIORS:
        m = metrics[f"{b} incidents per opportunity"]
        if m["before"] is None or m["after"] is None:
            continue
        rel = (m["after"] - m["before"]) / m["before"] if m["before"] else (1.0 if m["after"] else 0.0)
        if abs(rel) < 0.3 or abs(m["after"] - m["before"]) < 0.005:
            continue
        ev = ev_for_behavior(b)
        if ev:
            obs.append((abs(rel) / 3, {"kind": "weakness" if rel > 0 else "disappeared", "behavior": b, "evidence": ev, "metric_change": m, "confidence": "low"}))
    obs.sort(key=lambda t: -t[0])
    observations = [o for _, o in obs[:5]]
    pwr = metrics["proxy_win_rate"]
    pct = lambda x: "n/a" if x is None else f"{round(x * 100)}%"
    lead = observations[0] if observations else None
    what = f"the largest change is {lead['behavior']} ({lead['metric_change']['name']} {lead['metric_change']['before']} -> {lead['metric_change']['after']})" if lead else "no behavior changed enough to report"
    summary = (f"Rule-based coach{' (LLM unavailable: ' + reason + ')' if reason else ''}. "
               f"Compared with generation {facts['compared_with_gen']}, {what}. "
               f"The Proxy won {pct(pwr['after'])} of benchmark games (was {pct(pwr['before'])}).")
    dom = facts.get("dominant_candidate")
    return {"observations": observations, "dominant_strategy": {**dom, "counter_proposal": None} if dom else None,
            "proposals": [], "summary": summary}


def coach(facts: dict, provider: str, cwd: Path, timeout: int, model: str, log_dir: Path) -> dict:
    """One coach cycle. Never raises: failures fall back to the rule-based coach."""
    log_dir.mkdir(parents=True, exist_ok=True)
    (log_dir / f"facts-{facts['gen']}.json").write_text(json.dumps(facts, indent=1))
    meta: dict = {"provider": provider}
    error = None
    fields = None
    if provider in ("claude", "codex"):
        text = prompt(facts)
        try:
            if provider == "claude":
                raw, meta = call_claude(text, cwd, timeout, model)
            else:
                raw, meta = call_codex(text, cwd, timeout)
            (log_dir / f"raw-{facts['gen']}.json").write_text(json.dumps(raw, indent=1))
            fields, dropped = validate(raw, facts)
            meta["dropped"] = dropped
            if not fields["summary"]:
                error = "the model returned no summary"
                fields = None
        except Exception as e:  # noqa: BLE001 - any failure falls back, and is logged
            error = f"{type(e).__name__}: {str(e)[:400]}"
    else:
        error = None if provider == "rules" else f"unknown provider {provider}"
    if fields is None:
        fields = rule_based(facts, error if provider != "rules" else None)
        meta = {**meta, "provider": "rules" if provider == "rules" else f"rules (fallback from {provider})"}
    doc = {"gen": facts["gen"], **fields, "coach": {**meta, "error": error},
           "sampled_games": [{"game_id": g["game_id"], "why": g["why"]} for g in facts["games"]]}
    with (log_dir / "coach.log").open("a") as f:
        f.write(json.dumps({"gen": facts["gen"], "t": time.strftime("%Y-%m-%dT%H:%M:%S"), "provider": doc["coach"]["provider"], "error": error, "observations": len(doc["observations"]), "proposals": len(doc["proposals"])}) + "\n")
    return doc
