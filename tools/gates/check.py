#!/usr/bin/env python3
"""Pre-merge gates for one branch: the deterministic checks, and which reviewers the branch needs.

  tools/gates/check.py <branch> [--repo DIR] [--base REF] [--checks-only] [--skip NAME ...] [--pr-comments FILE]

CI (.github/workflows/ci.yml, job `gates`, and tools/ci/local-ci.sh) runs `--base origin/master --checks-only
--skip art` on every pull request: the deterministic checks only, since the art check needs a browser.

Reviewer gate: each required reviewer posts its verdict as a PR comment that names the reviewer and the head commit
SHA; the merger checks those comments are present for the PR's head before merging. Without --pr-comments this
script lists the reviewers the changed paths require. With --pr-comments FILE (the PR's comments as JSON, e.g.
`gh pr view N --json comments > FILE` or `gh api repos/OWNER/REPO/issues/N/comments > FILE`, or plain text with one
verdict per line) it also checks that each required reviewer has a comment naming it and the branch tip's SHA
(7 or more hex digits); a comment that says FAIL or BLOCK counts as a failed review.

By changed path:
  every branch            hygiene.py           secrets, personal data, saves, large binaries, dead code markers
  every branch            public-claims.py     README counts against the code: peoples and their names, eras,
                                               technologies (the docs reviewer checks the other public claims)
  crates/core, crates/ai  engine-invariants.py floats, hash-order iteration, unseeded RNG, clocks, render deps,
                                               saved-state changes without a RULES_VERSION bump, stale fixtures
  art assets              art.py               tools/art/distinctness.py --render --check (cached per art tree)
  code                    legatum-docs-reviewer
  player text             legatum-plain-language-reviewer  (the copy files below, or any app/src source line that
                                               adds a sentence)
  scenario-only branch    scenario.py          only scenarios/lab/*.json plus docs: no docs review, no extended verify;
                                               instead the fast scenario check (rules version, baseline doc, state
                                               loads on master's engine, copy lint; seconds)
  UI                      legatum-ui-reviewer
  art, or art-related code  legatum-art-reviewer  (art files, or the names/kinds/wiring that art depends on:
                                               ART_IMPACT)
  public front door       legatum-docs-reviewer and legatum-art-reviewer  (README.md, docs/media/, the link-preview
                                               card: FRONT; the copy and every image or clip a visitor sees first)
Exit 1 when a check fails, or (with --pr-comments) a required review is missing or failed.
"""
import argparse
import importlib.util
import json
import os
import re
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from common import changed_files, diff_added, git  # noqa: E402

# Which reviewer each kind of change needs (CLAUDE.md, "Before merging a PR").
REVIEWERS = {
    "docs": "legatum-docs-reviewer",
    "plain": "legatum-plain-language-reviewer",
    "ui": "legatum-ui-reviewer",
    "art": "legatum-art-reviewer",
}

TESTS = re.compile(r"^crates/[^/]+/tests/|_tests\.rs$|/tests\.rs$|^app/e2e/|\.test\.tsx?$|^app/src/test/|/__snapshots__/")
# Scenario Lab files: data, not code, when the branch changes nothing else that is code (see tools/gates/scenario.py).
LAB_SCENARIO = re.compile(r"^scenarios/lab/[a-z0-9-]+\.json$")
# Not code: a branch touching only these merges without a docs review.
NON_CODE = re.compile(r"^(docs/|plans/|\.progress/|\.reviews/|\.claude/|\.gitignore$)|\.md$|" + TESTS.pattern)
UI = re.compile(r"^app/src/.*\.(tsx|css)$|^app/src/map/|^app/src/assets/|^app/public/|^app/index\.html$")
COPY = re.compile(r"^app/src/(data|factionSheets|airText|buildLabel|notice|milestones)\.ts$|^app/src/map/labels\.ts$"
                  r"|^app/src/ui/(GuideContent|Tip|TurnReport|PostGameReport|DecisionNotice|EraNotice|ObjectiveSheet)\.tsx$"
                  r"|^app/src/.*(Text|text|Lore|lore|Copy|copy|Tips|tips)\w*\.tsx?$")
# A source line that adds player-facing prose: a string literal or JSX text of five or more words starting with a
# capital letter. Comments are skipped. A false hit only costs a cheap plain-language review.
APP_SOURCE = re.compile(r"^app/src/.*\.tsx?$")
PROSE = re.compile(r"(['\"`]|>)\s*[A-Z][a-z']*(?:[ ,;:]+[A-Za-z0-9'$%{}.+-]+){4,}")
COMMENT = re.compile(r"^\s*(//|/?\*)")
SCENARIO_COPY = re.compile(r'^\s*"(title|briefing|tests|summary|question)"\s*:')
ENGINE = re.compile(r"^crates/(core|ai)/(src/|Cargo\.toml$)")
ART = re.compile(r"^app/src/assets/.*\.(png|svg|jpe?g|webp)$|^app/public/(wonders|terrain)/"
                 r"|^tools/art/(distinctness\.(py|json)|render-sets\.mjs)$")
# The public front door: what a visitor sees first (user, 2026-11-22: a generic boxes-and-arrows README diagram merged
# with no reviewer because README.md and docs/ count as docs). Its copy and its visuals both get a review.
FRONT = re.compile(r"^README\.md$|^docs/media/|^app/public/og\.jpg$")
# Code that can make existing art stale or missing: names and kinds of things that have pictures, and the art wiring.
ART_IMPACT = re.compile(r"^crates/core/src/(data|eras|units|buildings|factions?)\.rs$"
                        r"|^app/src/(unitArt|unlockArt|resourceIcons|landmarks|portraits|siegeProps|icons|wonders)\.ts$"
                        r"|^app/src/map/(cityArt|landmarks|improvementArt)\.ts$")


def load(name):
    spec = importlib.util.spec_from_file_location(name.replace("-", "_"), os.path.join(HERE, name + ".py"))
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def classify(repo, base, tip, files=None):
    """{'code','ui','copy','engine','art','artimpact','scenario': [paths]} for BASE..TIP. A branch whose only code is Lab scenario
    files is `scenario`: those files are not `code` or `copy` (the scenario check covers them instead)."""
    files = files if files is not None else [p for s, p in changed_files(repo, base, tip)]
    out = {k: [] for k in ("code", "ui", "copy", "engine", "art", "artimpact", "scenario", "front")}
    scen = [f for f in files if f.startswith("scenarios/") and f.endswith(".json")]
    lab_only = bool(scen) and all(LAB_SCENARIO.match(f) for f in files if not NON_CODE.search(f))
    if lab_only:
        out["scenario"] = scen
        scen = []
    prose_candidates = [f for f in files if APP_SOURCE.match(f) and not TESTS.search(f) and not COPY.search(f)]
    added = diff_added(repo, base, tip) if scen or prose_candidates else {}
    for f in files:
        test = bool(TESTS.search(f))
        if lab_only and f in out["scenario"]:
            continue
        if not NON_CODE.search(f):
            out["code"].append(f)
        if UI.search(f) and not test:
            out["ui"].append(f)
        if (COPY.search(f) and not test) or (f in scen and any(SCENARIO_COPY.match(added[f][0][n])
                                                                for n in added.get(f, ({}, set()))[1])) \
                or (f in prose_candidates and adds_prose(added.get(f, ({}, set())))):
            out["copy"].append(f)
        if ENGINE.search(f) and not test:
            out["engine"].append(f)
        if ART.search(f):
            out["art"].append(f)
        if ART_IMPACT.search(f) and not test:
            out["artimpact"].append(f)
        if FRONT.search(f):
            out["front"].append(f)
    return out


def adds_prose(entry):
    """Whether a file's added lines (a diff_added entry) include a player-facing sentence."""
    lines, added = entry
    return any(PROSE.search(lines[n]) and not COMMENT.match(lines[n]) for n in added)


def run_checks(repo, base, tip, kinds, skip=()):
    """[(check name, seconds, [finding lines])] for the deterministic checks the changed paths call for."""
    plan = [("hygiene", lambda: load("hygiene").check(repo, base, tip)[0]),
            ("public-claims", lambda: load("public-claims").check(repo, tip))]
    if kinds["engine"]:
        plan.append(("engine-invariants", lambda: load("engine-invariants").check(repo, base, tip)[0]))
    if kinds["art"]:
        plan.append(("art", lambda: load("art").check(repo, tip)))
    if kinds["scenario"]:
        plan.append(("scenario", lambda: load("scenario").check(repo, tip, kinds["scenario"])))
    plan = [(name, fn) for name, fn in plan if name not in skip]
    results = []
    for name, fn in plan:
        t = time.time()
        findings = fn()
        results.append((name, time.time() - t, findings))
    return results


def required_reviews(kinds):
    """[(kind, reviewer agent, why)] for the reviews the changed paths call for."""
    out = []
    front = kinds.get("front", [])
    if kinds["code"] or front:
        out.append(("docs", REVIEWERS["docs"], "code changed" if kinds["code"] else
                    "public front door changed: " + ", ".join(front[:4])))
    if kinds["copy"]:
        # Player text (user, 2026-11-21): a card says the short rule, the Guide holds the detail, in plain words.
        out.append(("plain", REVIEWERS["plain"], "player text changed: " + ", ".join(kinds["copy"][:4])))
    if kinds["ui"]:
        out.append(("ui", REVIEWERS["ui"], "UI changed"))
    if kinds["art"] or kinds.get("artimpact") or front:
        # Art, changes that may make art stale, and the front door's images and clips (user, 2026-11-22): the art
        # reviewer judges all three.
        out.append(("art", REVIEWERS["art"], "art, art-related code or front-door media changed: "
                    + ", ".join((kinds["art"] + kinds.get("artimpact", []) + front)[:4])))
    return out


def load_comments(path):
    """Comment bodies from a PR comments file: JSON (a list of strings or of objects with `body`, or an object with
    a `comments` list, as `gh pr view --json comments` writes) or plain text (each line is one comment)."""
    with open(path, encoding="utf-8") as f:
        text = f.read()
    try:
        data = json.loads(text)
    except ValueError:
        return [ln for ln in text.splitlines() if ln.strip()]
    if isinstance(data, dict):
        data = data.get("comments", [])
    bodies = []
    for c in data if isinstance(data, list) else []:
        body = c.get("body", "") if isinstance(c, dict) else c
        if isinstance(body, str):
            bodies.append(body)
    return bodies


SHA = re.compile(r"\b[0-9a-f]{7,40}\b")
FAILED = re.compile(r"\b(FAIL(ED|S)?|BLOCK(ED|S|ER)?)\b", re.I)


def comment_status(reviews, comments, tip):
    """[(kind, ok, message)]: a review counts when a comment names its reviewer and a prefix of TIP; the last such
    comment decides, and one that says FAIL or BLOCK is a failed review."""
    out = []
    for kind, reviewer, why in reviews:
        last = None
        for body in comments:
            if reviewer in body and any(tip.startswith(m) for m in SHA.findall(body.lower())):
                last = body
        if last is None:
            out.append((kind, False, f"no PR comment from {reviewer} naming {tip[:9]} ({why})"))
        elif FAILED.search(last):
            out.append((kind, False, f"{reviewer} reported a failure at {tip[:9]}"))
        else:
            out.append((kind, True, ""))
    return out


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("branch")
    ap.add_argument("--repo", default=".")
    ap.add_argument("--base", default="refs/heads/master", help="ref the branch lands on (CI: origin/master)")
    ap.add_argument("--checks-only", action="store_true", help="skip the reviewer requirements")
    ap.add_argument("--skip", action="append", default=[], help="a check to leave out (e.g. art)")
    ap.add_argument("--pr-comments", metavar="FILE", help="the PR's comments; check each required reviewer's verdict")
    a = ap.parse_args(argv)
    tip = (git(a.repo, "rev-parse", "--verify", "-q", a.branch + "^{commit}") or "").strip()
    base = (git(a.repo, "merge-base", a.base, tip) or "").strip() if tip else ""
    if not tip or not base:
        print(f"cannot resolve {a.branch} or its merge base with {a.base}", file=sys.stderr)
        return 2
    kinds = classify(a.repo, base, tip)
    checks = run_checks(a.repo, base, tip, kinds, set(a.skip))
    reviews = [] if a.checks_only else required_reviews(kinds)
    bad = 0
    for name, secs, findings in checks:
        print(f"{name}: {'PASS' if not findings else 'FAIL'} ({secs:.1f} s)")
        for f in findings:
            print("  " + f)
        bad += bool(findings)
    if a.checks_only:
        print("reviews: not checked (--checks-only)")
    elif not reviews:
        print("reviews: none needed (" + ("scenario files only, the scenario check stands in" if kinds["scenario"] else
                                          "docs or tests only" if not kinds["code"] else "") + ")")
    else:
        print("reviewers required: " + ", ".join(r for _, r, _ in reviews)
              + f" (verdicts as PR comments naming {tip[:9]})")
        if a.pr_comments:
            for kind, ok, msg in comment_status(reviews, load_comments(a.pr_comments), tip):
                print(f"{kind} review: {'ok' if ok else 'MISSING'}{' (' + msg + ')' if msg else ''}")
                bad += not ok
        else:
            for kind, reviewer, why in reviews:
                print(f"  {reviewer}: {why}")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
