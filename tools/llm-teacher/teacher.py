#!/usr/bin/env python3
"""LLM-teacher experiment harness (docs/design/ai-strategy/fable-decision-models-2026-11-07.md §5).

Subcommands (run from anywhere; paths are resolved against the repo root):

  render  --games games.json --out RUN_DIR [--notes tools/llm-teacher/notes.md]
          Run the Rust state renderer (`teacher_render`) for every game in games.json and write
          RUN_DIR/windows.jsonl: one row per window with the public state text, the events digest,
          the operation-search candidates and, for scoring only, the designer's recorded orders.
  smoke   Check both routes: Luna (`codex exec`) can neither run a shell nor read the repo, and
          Haiku (`claude -p`) is signed in.
  run     --run RUN_DIR [--models haiku,luna] [--roles chooser,proposer,strategist]
          [--variants plain,notes] [--repeats 3] [--notes-models haiku] [--limit-windows N]
          [--windows game:turn,...] [--workers 3] [--dry-run]
          Call every (window, role, variant, repeat, model) once and append the call to
          RUN_DIR/calls.jsonl (prompt hash, model, full response, parsed choice, latency). Resumes:
          a call already recorded is skipped.
  score   --run RUN_DIR --out README.md
          Score the calls against the operation-search log and the designer's tape.

Privacy: the designer's reasoning notes go only to the models in --notes-models (default: haiku,
i.e. Anthropic). They are never sent to Luna (OpenAI) unless the lead adds it to the flag.
Nothing here ever puts the referee's numbers or the designer's recorded orders in a prompt.
"""
from __future__ import annotations

import argparse
import concurrent.futures as cf
import hashlib
import json
import os
import random
import re
import subprocess
import sys
import tempfile
import threading
import time
from collections import Counter, defaultdict
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parent.parent
RENDERER = REPO / "target" / "release" / "examples" / "teacher_render"

MODELS = {
    "haiku": {"route": "claude", "id": "haiku"},
    "luna": {"route": "codex", "id": "gpt-6-luna"},
}
ROLES = ("chooser", "proposer", "strategist")
VARIANTS = ("plain", "notes")

# The generator's grammar: template name -> takes a target?  (target: a rival city, or a team for
# accept_peace). Modifiers are advisory and recorded; the canonical name is template[:target].
GRAMMAR = {
    "strike": "city",
    "buy_strike": "city",
    "short_strike": "city",
    "veteran_strike": "city",
    "expand_now": None,
    "escort_settlers": None,
    "snipe_settlers": None,
    "lookouts": None,
    "wall_up": None,
    "small_army_economy": None,
    "refuse_peace": None,
    "accept_peace": "team",
    "faction_kit": None,
    "fortress_economy": None,
    "expand_walled_escorted": None,
}
MODIFIERS = ("buy", "escort", "short", "walls_first", "veteran")

GRAMMAR_TEXT = (
    "Operation grammar: template(target). Templates: strike(city), buy_strike(city), short_strike(city), "
    "veteran_strike(city), expand_now, escort_settlers, snipe_settlers, lookouts, wall_up, "
    "small_army_economy, refuse_peace, accept_peace(rival faction), faction_kit, fortress_economy, "
    "expand_walled_escorted. A city target must be a rival city named in the state."
)

SYSTEM = "You are a strategy advisor for a historical 4X game. You have no tools. Answer only with the JSON object asked for."


# ----------------------------------------------------------------------------------------------
# Prompts


def read(name: str) -> str:
    return (HERE / name).read_text().strip()


def window_id(w: dict) -> str:
    return f"{w['game']}:T{w['turn']}"


def shuffled(w: dict, repeat: int) -> list[dict]:
    """The candidates in a shuffled order. Seeded by (window, repeat) only, so both models and both
    variants see the same order at a given repeat."""
    seed = int(hashlib.sha256(f"{window_id(w)}|{repeat}".encode()).hexdigest()[:12], 16)
    cands = list(w["candidates"])
    random.Random(seed).shuffle(cands)
    return cands


def build_prompt(w: dict, role: str, variant: str, repeat: int) -> tuple[str, list[str]]:
    parts = ["# The game\n" + read("rules.md")]
    if variant == "notes":
        if not w.get("notes"):
            raise SystemExit("the windows file has no notes; render with --notes")
        parts.append("# How the player you advise reasons (his own recorded reasons)\n" + w["notes"])
    parts.append("# The position (what the seat can see; places are named, not coordinates)\n" + w["state"].strip())
    parts.append("# What changed since two turns ago\n" + "\n".join(f"- {e}" for e in w["events"]))
    order: list[str] = []
    if role in ("chooser", "proposer"):
        cands = shuffled(w, repeat)
        order = [c["label"] for c in cands]
        title = "# The candidate operations (a simulator will run whichever you pick)" if role == "chooser" else "# Operations already on the list (do not repeat these)"
        lines = [f"- {c['label']}: {c['line']}" for c in cands]
        parts.append(f"{title}\nHard's current plan: {w['hard_plan']}.\n" + "\n".join(lines))
    if role == "chooser":
        parts.append(
            "# Question\nWhich operation gives this seat the best chance of winning the game? Pick one and rank the best three. Use the label exactly as written, with no bullet or number. "
            'Answer with only this JSON: {"choice": "<label>", "ranking": ["<label>", "<label>", "<label>"], "reason": "<one sentence>"}'
        )
    elif role == "proposer":
        parts.append(
            "# Question\nName up to three operations that are NOT on the list above and that you think would beat all of them "
            "for this seat. " + GRAMMAR_TEXT + " Use only the grammar; an idea outside it goes in other_ideas. "
            'Answer with only this JSON: {"proposals": [{"template": "<template>", "target": "<city or faction or null>", '
            '"modifier": "<optional: ' + "/".join(MODIFIERS) + '>", "why": "<one sentence>"}], "other_ideas": ["<short text>"]}'
        )
    else:
        parts.append(
            "# Question\nSet the strategy for this seat for the next 20 turns. "
            'Answer with only this JSON: {"posture": "expand|rush|turtle|race", "attack": "<rival faction to attack or null>", '
            '"settle_away_from": "<rival faction or null>", "peace": "accept|refuse|none_offered", '
            '"alliance": "accept|decline|none_offered", "reason": "<one sentence>"}'
        )
    return "\n\n".join(parts) + "\n", order


def prompt_hash(prompt: str) -> str:
    return hashlib.sha256(prompt.encode()).hexdigest()[:16]


# ----------------------------------------------------------------------------------------------
# Model routes


def clean_env(keep: tuple[str, ...]) -> dict:
    return {k: v for k, v in os.environ.items() if k in keep}


def call_model(model: str, prompt: str, scratch: Path) -> dict:
    m = MODELS[model]
    t0 = time.time()
    scratch.mkdir(parents=True, exist_ok=True)
    try:
        if m["route"] == "claude":
            cmd = [
                "claude", "-p", "--model", m["id"], "--no-session-persistence", "--output-format", "json",
                "--tools", "", "--disable-slash-commands", "--strict-mcp-config", "--setting-sources", "",
                "--system-prompt", SYSTEM,
            ]
            env = clean_env(("HOME", "PATH", "USER", "LANG", "TMPDIR"))
            p = subprocess.run(cmd, input=prompt, capture_output=True, text=True, cwd=scratch, env=env, timeout=240)
            out = p.stdout.strip()
            try:
                j = json.loads(out)
                text = j.get("result", "")
                err = text if j.get("is_error") else None
                usage = j.get("usage", {})
                tokens = {"in": usage.get("input_tokens", 0) + usage.get("cache_read_input_tokens", 0) + usage.get("cache_creation_input_tokens", 0),
                          "out": usage.get("output_tokens", 0)}
            except json.JSONDecodeError:
                text, err, tokens = out, p.stderr.strip()[:300] or "unparseable output", {}
        else:
            last = scratch / f"last-{threading.get_ident()}.txt"
            last.unlink(missing_ok=True)
            cmd = [
                "codex", "exec", "-s", "read-only", "-m", m["id"], "--skip-git-repo-check", "--ephemeral",
                "--ignore-user-config", "-C", str(scratch), "-o", str(last),
                # No shell: a read-only sandbox still lets the model read any file the user can
                # (checked by `smoke`), so the tools are switched off instead.
                "--disable", "shell_tool", "--disable", "apps", "--disable", "plugins", "--disable", "computer_use",
                "--disable", "browser_use", "--disable", "browser_use_external", "--disable", "in_app_browser",
                "--disable", "image_generation", "-",
            ]
            env = clean_env(("HOME", "PATH", "USER", "LANG", "TMPDIR", "CODEX_HOME"))
            p = subprocess.run(cmd, input=SYSTEM + "\n\n" + prompt, capture_output=True, text=True, cwd=scratch, env=env, timeout=300)
            text = last.read_text().strip() if last.exists() else ""
            err = None if text else (p.stderr.strip()[-300:] or "no output")
            m2 = re.search(r"tokens used\s*\n?\s*([\d,]+)", p.stdout + p.stderr)
            tokens = {"total": int(m2.group(1).replace(",", ""))} if m2 else {}
    except subprocess.TimeoutExpired:
        text, err, tokens = "", "timeout", {}
    except FileNotFoundError as e:
        text, err, tokens = "", f"missing binary: {e}", {}
    return {"response": text, "error": err, "latency_s": round(time.time() - t0, 2), "tokens": tokens}


def parse_json(text: str):
    text = text.strip()
    text = re.sub(r"^```(?:json)?\s*|\s*```$", "", text)
    a, b = text.find("{"), text.rfind("}")
    if a < 0 or b < a:
        return None
    try:
        return json.loads(text[a : b + 1])
    except json.JSONDecodeError:
        return None


# ----------------------------------------------------------------------------------------------
# Parsing a response into what is scored


def norm_label(x) -> str:
    """Models sometimes echo a list number or a trailing colon with the label."""
    return re.sub(r"^\s*\d+[.)]\s*", "", str(x)).strip().rstrip(":").strip()


def parse_chooser(j, order: list[str]) -> dict:
    if not isinstance(j, dict) or norm_label(j.get("choice")) not in order:
        return {"valid": False}
    j = {**j, "choice": norm_label(j["choice"]), "ranking": [norm_label(r) for r in (j.get("ranking") or [])]}
    rank = [r for r in j["ranking"] if r in order]
    if j["choice"] not in rank:
        rank.insert(0, j["choice"])
    rank = list(dict.fromkeys(rank))
    return {"valid": True, "choice": j["choice"], "ranking": rank, "reason": j.get("reason", "")}


def canonical(template: str, target) -> str:
    return template if not target else f"{template}:{target}"


def parse_proposer(j, w: dict) -> dict:
    if not isinstance(j, dict):
        return {"valid": False, "proposals": [], "dropped": 0, "other": 0}
    cities = {n.split("(")[0] for n in re.findall(r"city ([A-Za-z' -]+?) \(", w["state"])}
    factions = set(re.findall(r"^- ([A-Za-z]+): ", w["state"], re.M))
    on_list = {c["name"] for c in w["candidates"]}
    ok, dropped = [], []
    for p in (j.get("proposals") or [])[:3]:
        if not isinstance(p, dict):
            dropped.append(p)
            continue
        t, target = str(p.get("template", "")).strip(), p.get("target")
        target = None if target in (None, "", "null", "None") else str(target).strip()
        kind = GRAMMAR.get(t, "bad")
        good = (
            kind != "bad"
            and (kind is None or (kind == "city" and target in cities) or (kind == "team" and target in factions))
        )
        if good:
            name = canonical(t, target if kind else None)
            ok.append({"name": name, "template": t, "target": target, "modifier": p.get("modifier"),
                       "why": p.get("why", ""), "new": name not in on_list})
        else:
            dropped.append(p)
    return {"valid": True, "proposals": ok, "dropped": len(dropped), "other": len(j.get("other_ideas") or [])}


def parse_strategist(j) -> dict:
    if not isinstance(j, dict) or j.get("posture") not in ("expand", "rush", "turtle", "race"):
        return {"valid": False}
    return {"valid": True, **{k: j.get(k) for k in ("posture", "attack", "settle_away_from", "peace", "alliance", "reason")}}


# ----------------------------------------------------------------------------------------------
# render / smoke / run


def cmd_render(a):
    games = json.loads(Path(a.games).read_text())
    out = Path(a.out)
    out.mkdir(parents=True, exist_ok=True)
    if not RENDERER.exists():
        raise SystemExit(f"build the renderer first: cargo build --release --example teacher_render ({RENDERER})")
    rows = []
    for g in games:
        tmp = out / f"{g['game']}.render.jsonl"
        cmd = [str(RENDERER), "--tape", os.path.expanduser(g["tape"]), "--states", os.path.expanduser(g["states"]), "--out", str(tmp), "--game", g["game"],
               "--seat", str(g.get("seat", 0)), "--every", str(g.get("every", 2))]
        cmd += ["--turns", g["turns"]] if g.get("turns") else ["--windows", g["windows"]]
        for o in g.get("ops", []):
            cmd += ["--ops", os.path.expanduser(o)]
        if a.notes:
            cmd += ["--notes", a.notes]
        subprocess.run(cmd, check=True)
        rows += [json.loads(l) for l in tmp.read_text().splitlines() if l.strip()]
    (out / "windows.jsonl").write_text("".join(json.dumps(r) + "\n" for r in rows))
    print(f"{len(rows)} windows -> {out / 'windows.jsonl'}")


def cmd_smoke(a):
    scratch = Path(tempfile.mkdtemp(prefix="teacher-smoke-"))
    bad = False
    secret_path = str(REPO / "CLAUDE.md")
    probe = (f"Try to use any tool you have to run: head -c 120 {secret_path} . If you have no tool, answer NO_TOOLS. "
             "Otherwise paste the output.")
    r = call_model("luna", probe, scratch)
    leaked = "Legatum" in r["response"]
    print(f"luna: response={r['response'][:80]!r} error={r['error']} -> {'LEAK' if leaked else 'cannot read the repo'}")
    bad |= leaked or bool(r["error"])
    r = call_model("haiku", "Reply with just the word ok.", scratch)
    print(f"haiku: response={r['response'][:80]!r} error={r['error']}")
    bad |= bool(r["error"])
    sys.exit(1 if bad else 0)


def load_windows(run: Path) -> list[dict]:
    return [json.loads(l) for l in (run / "windows.jsonl").read_text().splitlines() if l.strip()]


def load_calls(run: Path) -> list[dict]:
    p = run / "calls.jsonl"
    return [json.loads(l) for l in p.read_text().splitlines() if l.strip()] if p.exists() else []


def cmd_run(a):
    run = Path(a.run)
    wins = load_windows(run)
    if a.windows:
        keep = set(a.windows.split(","))
        wins = [w for w in wins if window_id(w) in keep]
    if a.limit_windows:
        wins = wins[: a.limit_windows]
    models = a.models.split(",")
    roles = a.roles.split(",")
    variants = a.variants.split(",")
    notes_models = set(a.notes_models.split(","))
    done = {c["id"] for c in load_calls(run) if not c.get("error")}
    jobs = []
    for w in wins:
        for role in roles:
            if role != "strategist" and not w["candidates"]:
                continue
            for variant in variants:
                for model in models:
                    if variant == "notes" and model not in notes_models:
                        continue  # privacy: the designer's notes go only to --notes-models
                    for rep in range(a.repeats):
                        cid = f"{window_id(w)}|{role}|{variant}|{model}|{rep}"
                        if cid not in done:
                            jobs.append((cid, w, role, variant, model, rep))
    print(f"{len(jobs)} calls to make ({len(done)} already recorded)")
    if a.dry_run:
        for cid, w, role, variant, model, rep in jobs[:3]:
            print("-" * 60, cid)
            print(build_prompt(w, role, variant, rep)[0])
        return
    lock = threading.Lock()
    # One lane per model so a slow model never starves the other and rate limits stay polite.
    sems = {m: threading.Semaphore(a.workers) for m in models}

    def work(job):
        cid, w, role, variant, model, rep = job
        prompt, order = build_prompt(w, role, variant, rep)
        with sems[model]:
            r = call_model(model, prompt, Path(tempfile.gettempdir()) / f"teacher-scratch-{model}")
        j = parse_json(r["response"]) if r["response"] else None
        parsed = {"chooser": lambda: parse_chooser(j, order), "proposer": lambda: parse_proposer(j, w), "strategist": lambda: parse_strategist(j)}[role]()
        rec = {"id": cid, "window": window_id(w), "game": w["game"], "turn": w["turn"], "role": role, "variant": variant,
               "model": model, "model_id": MODELS[model]["id"], "repeat": rep, "prompt_hash": prompt_hash(prompt),
               "prompt_chars": len(prompt), "options": order, **r, "parsed": parsed, "time": int(time.time())}
        with lock, open(run / "calls.jsonl", "a") as f:
            f.write(json.dumps(rec) + "\n")
        return rec

    n = 0
    with cf.ThreadPoolExecutor(max_workers=a.workers * len(models)) as ex:
        for rec in ex.map(work, jobs):
            n += 1
            status = rec["error"] or ("ok" if rec["parsed"].get("valid") else "unparsed")
            print(f"[{n}/{len(jobs)}] {rec['id']} {rec['latency_s']}s {status}", flush=True)


# ----------------------------------------------------------------------------------------------
# Scoring


def tape_label(w: dict) -> tuple[str, str]:
    """Map the designer's recorded orders for the window to the nearest candidate. Heuristic and
    deliberately conservative: returns (candidate name or 'other', why)."""
    t, names = w["tape"], {c["name"]: c for c in w["candidates"]}

    def has(prefix):
        return [n for n in names if n.startswith(prefix)]

    if t["declared"] or sum(t["marched_on"].values()) >= 3:
        target = max(t["marched_on"], key=t["marched_on"].get) if t["marched_on"] else None
        buy = bool(t["bought"])
        for pre in (("buy_strike:" if buy else "strike:"), "strike:", "buy_strike:"):
            if target and f"{pre}{target}" in names:
                return f"{pre}{target}", f"marched {t['marched_on']} declared {t['declared']}"
        if has("strike:"):
            return "other", f"war/march on {target or t['declared']} but no matching strike candidate"
    for r in t["accepted"]:
        if "Peace" in r:
            for n in has("accept_peace"):
                return n, f"accepted {r}"
    if t["refused"] and "refuse_peace" in names:
        return "refuse_peace", f"refused {t['refused']}"
    if t["founded"] or any(p in ("Settler",) for p in t["produced"] + t["bought"]):
        return "expand_now", f"founded {t['founded']} produced {t['produced'][:3]}"
    if any(p in ("Walls", "AlpineRedoubt", "Redoubt") for p in t["produced"] + t["bought"]) and "wall_up" in names:
        return "wall_up", "built Walls"
    return "other", "no matching pattern"


def modal(xs: list[str]):
    c = Counter(xs)
    top = max(c.values())
    return sorted(k for k, v in c.items() if v == top)[0]


def rank_corr(a: list[str], b: list[str]) -> float | None:
    common = [x for x in a if x in b]
    n = len(common)
    if n < 3:
        return None
    ra = {x: i for i, x in enumerate(common)}
    rb = {x: i for i, x in enumerate([y for y in b if y in common])}
    d2 = sum((ra[x] - rb[x]) ** 2 for x in common)
    return 1 - 6 * d2 / (n * (n * n - 1))


def op_table(w: dict) -> dict:
    return {c["name"]: c for c in w["candidates"]}


def confirmed_best(w: dict, ref: dict | None):
    """(best name, clear?) from the operation-search log row for the window."""
    if not ref:
        return None, False
    c = ref.get("confirm") or {}
    return ref.get("best"), bool(c) and c.get("gain_lo", -1) > 0 and c.get("gain_pp", 0) > 0


def pct(n, d):
    return "n/a" if not d else f"{100 * n / d:.0f}% ({n}/{d})"


def cmd_score(a):
    run = Path(a.run)
    wins = {window_id(w): w for w in load_windows(run)}
    calls = [c for c in load_calls(run) if not c.get("error")]
    refs: dict[str, dict] = {}
    for g in a.referee or []:
        for line in Path(g).read_text().splitlines():
            if line.strip():
                r = json.loads(line)
                if r.get("operations"):
                    refs[f"{r['game']}:T{r['turn']}"] = r
    lines: list[str] = []
    w_ = lines.append
    w_("# LLM teacher experiment (harness output)\n")
    w_(f"Run `{run.name}`: {len(wins)} windows, {len(calls)} recorded calls, {len({c['model'] for c in calls})} models. Generated by `tools/llm-teacher/teacher.py score`.\n")
    samples = sorted({(r.get('operations') or [{}])[0].get('samples') for r in refs.values()})
    confs = sorted({(r.get('confirm') or {}).get('samples') for r in refs.values()})
    w_(f"Referee: operation-search log, selection samples {samples}, confirmation samples {confs}. A window is *clear* when the confirmed gain of the search's best is positive with its 95% interval above zero.\n")

    groups = defaultdict(list)
    for c in calls:
        groups[(c["role"], c["model"], c["variant"])].append(c)

    # ----- chooser
    w_("## Chooser\n")
    w_("| Model | Variant | Calls | Valid | Agree with confirmed best (clear windows) | Agree with search best (all) | Mean gain of pick vs Hard (pp) | Worst pick (pp) | Mean regret to best (pp) | Agree with tape | Consistency (all repeats same top) | Mean rank corr | Median latency s |")
    w_("|---|---|---:|---:|---|---|---:|---:|---:|---|---|---:|---:|")
    chooser_summary = {}
    for (role, model, variant), cs in sorted(groups.items()):
        if role != "chooser":
            continue
        by_win = defaultdict(list)
        for c in cs:
            by_win[c["window"]].append(c)
        valid = [c for c in cs if c["parsed"].get("valid")]
        clear_n = clear_hit = all_n = all_hit = tape_n = tape_hit = 0
        gains, regrets = [], []
        cons_n = cons_hit = 0
        corrs = []
        for wid, wc in by_win.items():
            w = wins[wid]
            ref = refs.get(wid)
            best, clear = confirmed_best(w, ref)
            tops = [c["parsed"]["choice"] for c in wc if c["parsed"].get("valid")]
            if not tops:
                continue
            cons_n += 1
            cons_hit += len(set(tops)) == 1
            rks = [c["parsed"]["ranking"] for c in wc if c["parsed"].get("valid")]
            for i in range(len(rks)):
                for j in range(i + 1, len(rks)):
                    r = rank_corr(rks[i], rks[j])
                    if r is not None:
                        corrs.append(r)
            mode = modal(tops)
            tl, _ = tape_label(w)
            if tl != "other":
                tape_n += 1
                tape_hit += mode == tl
            tab = {c["name"]: c for c in w["candidates"]}
            lab2name = {c["label"]: c["name"] for c in w["candidates"]}
            if ref:
                all_n += 1
                all_hit += lab2name.get(mode) == best
                if clear:
                    clear_n += 1
                    clear_hit += lab2name.get(mode) == best
                opsd = {o["name"]: o for o in ref["operations"]}
                for t in tops:
                    nm = lab2name.get(t)
                    if nm in opsd:
                        gains.append(opsd[nm]["gain_pp"])
                        regrets.append(opsd[best]["gain_pp"] - opsd[nm]["gain_pp"] if best in opsd else 0.0)
        lat = sorted(c["latency_s"] for c in cs)
        mean = lambda x: f"{sum(x) / len(x):+.1f}" if x else "n/a"
        w_(f"| {model} | {variant} | {len(cs)} | {pct(len(valid), len(cs))} | {pct(clear_hit, clear_n)} | {pct(all_hit, all_n)} | {mean(gains)} | "
           f"{(f'{min(gains):+.1f}' if gains else 'n/a')} | {(f'{sum(regrets) / len(regrets):.1f}' if regrets else 'n/a')} | {pct(tape_hit, tape_n)} | "
           f"{pct(cons_hit, cons_n)} | {(f'{sum(corrs) / len(corrs):.2f}' if corrs else 'n/a')} | {lat[len(lat) // 2] if lat else 'n/a'} |")
        chooser_summary[(model, variant)] = dict(clear_hit=clear_hit, clear_n=clear_n, cons_hit=cons_hit, cons_n=cons_n,
                                                 gain=(sum(gains) / len(gains) if gains else None))
    w_("")

    # ----- three-way table
    w_("## Three-way table (you, the model, the search's best)\n")
    models = sorted({c["model"] for c in calls if c["role"] == "chooser"})
    w_("| Window | Hard's plan | Your tape (heuristic) | Search best (confirmed gain) | " + " | ".join(f"{m} {v}" for m in models for v in VARIANTS if any(k[:3] == ('chooser', m, v) for k in groups)) + " |")
    w_("|---|---|---|---|" + "---|" * sum(1 for m in models for v in VARIANTS if ('chooser', m, v) in groups))
    for wid, w in wins.items():
        ref = refs.get(wid)
        best, clear = confirmed_best(w, ref)
        cg = (ref or {}).get("confirm") or {}
        tl, why = tape_label(w)
        row = [wid, w["hard_plan"], tl, f"{best} ({cg.get('gain_pp', 0):+.1f}{'' if clear else ', not clear'})" if best else "n/a"]
        for m in models:
            for v in VARIANTS:
                if ("chooser", m, v) not in groups:
                    continue
                tops = [c["parsed"]["choice"] for c in groups[("chooser", m, v)] if c["window"] == wid and c["parsed"].get("valid")]
                lab2name = {c["label"]: c["name"] for c in w["candidates"]}
                row.append((lab2name.get(modal(tops), "?") + (f" ({len(set(tops))} distinct)" if len(set(tops)) > 1 else "")) if tops else "-")
        w_("| " + " | ".join(str(x).replace("|", "/") for x in row) + " |")
    w_("\nThe tape column is a heuristic from his recorded orders (declared war, marched on a city, founded cities, accepted Peace, Walls); `other` means no candidate matched and the window is left out of tape agreement.\n")

    # ----- proposer
    w_("## Proposer\n")
    w_("| Model | Variant | Calls | In-grammar proposals | Dropped | Drop rate | Distinct new operations | Already priced by the search | of which confirmed +10 pp, interval clear of zero | Out-of-grammar ideas |")
    w_("|---|---|---:|---:|---:|---:|---:|---:|---:|---:|")
    proposals_out = []
    for (role, model, variant), cs in sorted(groups.items()):
        if role != "proposer":
            continue
        ok = sum(len(c["parsed"].get("proposals", [])) for c in cs)
        dropped = sum(c["parsed"].get("dropped", 0) for c in cs)
        other = sum(c["parsed"].get("other", 0) for c in cs)
        new = set()
        priced = confirmed = 0
        seen_priced = set()
        for c in cs:
            for p in c["parsed"].get("proposals", []):
                if not p["new"]:
                    continue
                new.add((c["window"], p["name"]))
                proposals_out.append({"window": c["window"], "name": p["name"], "model": model, "variant": variant, "why": p["why"]})
        for wid, name in sorted(new):
            ref = refs.get(wid)
            if not ref:
                continue
            for o in ref["operations"]:
                if o["name"] == name and (wid, name) not in seen_priced:
                    seen_priced.add((wid, name))
                    priced += 1
                    confirmed += o["gain_pp"] >= 10 and o["gain_lo"] > 0
        total = ok + dropped
        w_(f"| {model} | {variant} | {len(cs)} | {ok} | {dropped} | {pct(dropped, total)} | {len(new)} | {priced} | {confirmed} | {other} |")
    (run / "proposals-for-search.json").write_text(json.dumps(sorted({json.dumps(p, sort_keys=True) for p in map(lambda x: x, proposals_out)}), indent=0))
    w_("\nProposals the search has not priced yet are in `proposals-for-search.json` (window, canonical name); they need an `operation_search` extra-operations input to be confirmed. \"Already priced\" counts proposals whose canonical name is in the log (for example a template only generator 2 has).\n")

    # ----- strategist
    w_("## Strategist\n")
    w_("| Model | Variant | Calls | Valid | Posture mix | Consistency of posture | Latency s (median) |")
    w_("|---|---|---:|---:|---|---|---:|")
    for (role, model, variant), cs in sorted(groups.items()):
        if role != "strategist":
            continue
        by_win = defaultdict(list)
        for c in cs:
            if c["parsed"].get("valid"):
                by_win[c["window"]].append(c["parsed"]["posture"])
        mix = Counter(p for ps in by_win.values() for p in ps)
        cons = sum(len(set(ps)) == 1 for ps in by_win.values())
        lat = sorted(c["latency_s"] for c in cs)
        w_(f"| {model} | {variant} | {len(cs)} | {pct(sum(1 for c in cs if c['parsed'].get('valid')), len(cs))} | {dict(mix)} | {pct(cons, len(by_win))} | {lat[len(lat) // 2] if lat else 'n/a'} |")
    w_("\nStrategist answers are compared to his stated reasons and the coach's measured branches by hand (the question set is small); the notes-vs-plain comparison is the posture shift between the two variants on Haiku.\n")

    # ----- verdict (section 5.5)
    w_("## Verdict against the criteria written before the run (section 5.5)\n")
    for (model, variant), s in sorted(chooser_summary.items()):
        agree = s["clear_hit"] / s["clear_n"] if s["clear_n"] else None
        cons = s["cons_hit"] / s["cons_n"] if s["cons_n"] else None
        if agree is None or s["clear_n"] < 5:
            verdict = f"unscored: the referee is clear on {s['clear_n']} windows (needs at least 5)"
        else:
            ok = agree >= 0.60 and (cons or 0) >= 0.80 and (s["gain"] or 0) > 0
            kill = agree < 0.45 or (cons or 0) < 0.60
            verdict = "SUCCESS" if ok else ("KILL" if kill else "in between")
        w_(f"- Chooser, {model} {variant}: agreement {pct(s['clear_hit'], s['clear_n'])}, consistency {pct(s['cons_hit'], s['cons_n'])}, mean gain vs Hard {s['gain'] if s['gain'] is None else round(s['gain'], 1)} pp: **{verdict}**.")
    w_("- Proposer: success is two distinct proposals confirmed at +10 pp that the generator lacked; kill is zero confirmed in all windows or a drop rate over 50%. Read the table above; confirmation of un-priced proposals needs the search rerun.")
    w_("- Strategist: needs the hand-matching against his stated reasons (10 of 15) and notes ahead of plain.")
    Path(a.out).write_text("\n".join(lines) + "\n")
    print(f"wrote {a.out}")


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    r = sub.add_parser("render")
    r.add_argument("--games", required=True)
    r.add_argument("--out", required=True)
    r.add_argument("--notes", default=str(HERE / "notes.md"))
    r.set_defaults(fn=cmd_render)
    s = sub.add_parser("smoke")
    s.set_defaults(fn=cmd_smoke)
    u = sub.add_parser("run")
    u.add_argument("--run", required=True)
    u.add_argument("--models", default="haiku,luna")
    u.add_argument("--roles", default="chooser,proposer,strategist")
    u.add_argument("--variants", default="plain,notes")
    u.add_argument("--repeats", type=int, default=3)
    u.add_argument("--notes-models", default="haiku", help="models that may receive the designer's notes (default: haiku only)")
    u.add_argument("--windows", help="game:Tturn,... to restrict")
    u.add_argument("--limit-windows", type=int)
    u.add_argument("--workers", type=int, default=2)
    u.add_argument("--dry-run", action="store_true")
    u.set_defaults(fn=cmd_run)
    c = sub.add_parser("score")
    c.add_argument("--run", required=True)
    c.add_argument("--out", required=True)
    c.add_argument("--referee", action="append", help="operation_search JSONL log(s) to score against")
    c.set_defaults(fn=cmd_score)
    a = ap.parse_args()
    a.fn(a)


if __name__ == "__main__":
    main()
