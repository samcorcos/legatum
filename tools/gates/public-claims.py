#!/usr/bin/env python3
"""Public-claims check: the countable claims in README.md must agree with the code.

The README went public saying "nine peoples" and "seven eras, 35 technologies" when the code had eleven, nine and 45
(2026-11-21). This check reads the numbers from the Rust and TypeScript sources and fails when the README disagrees:

  peoples   the README line marked `<!-- peoples-count -->` states the number of peoples in `Faction::ALL`
            (crates/core/src/types.rs) and names every one of them; when some have no faction goal yet
            (`GoalRule::None` in crates/core/src/goals.rs) it also states how many are finished. `FACTIONS` in
            app/src/data.ts must list the same peoples in the same order.
  eras      the README line marked `<!-- eras-count -->` states `ERA_COUNT` eras and the number of rows in `TECHS`
            technologies (crates/core/src/eras.rs); every other "<number> eras" in the README states `ERA_COUNT` too.

If a people is hidden from the picker without leaving `Faction::ALL`, change this check in the same branch.
The claims a regex cannot check (player counts, difficulty levels, victory rules, platforms, what the AI may see)
are on the docs reviewer's checklist (.claude/agents/legatum-docs-reviewer.md).

  tools/gates/public-claims.py [--repo DIR] [REF]   the files at REF (default HEAD)
Exit 1 when anything is found.
"""
import argparse
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from common import show  # noqa: E402

README = "README.md"
TYPES = "crates/core/src/types.rs"
ERAS = "crates/core/src/eras.rs"
GOALS = "crates/core/src/goals.rs"
DATA_TS = "app/src/data.ts"
SOURCES = (README, TYPES, ERAS, GOALS, DATA_TS)

WORDS = ["zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten", "eleven", "twelve",
         "thirteen", "fourteen", "fifteen", "sixteen", "seventeen", "eighteen", "nineteen", "twenty"]
NUMBER = r"(\d+|" + "|".join(WORDS[1:]) + r")"
PEOPLES_MARK = "<!-- peoples-count -->"
ERAS_MARK = "<!-- eras-count -->"


def number(text):
    text = text.lower()
    return int(text) if text.isdigit() else WORDS.index(text)


def numbers(line):
    return [number(m) for m in re.findall(r"\b" + NUMBER + r"\b", line, re.I)]


def code_facts(types_rs, eras_rs, goals_rs, data_ts):
    """({'factions', 'unfinished', 'app_factions', 'eras', 'techs'}, [findings about sources that no longer parse])."""
    facts, bad = {}, []
    m = re.search(r"pub const ALL: \[Faction; (\d+)\] = \[(.*?)\];", types_rs, re.S)
    facts["factions"] = re.findall(r"Self::(\w+)", m.group(2)) if m else []
    if not m or len(facts["factions"]) != int(m.group(1)):
        bad.append(f"{TYPES}: cannot read Faction::ALL (update tools/gates/public-claims.py)")
    facts["unfinished"] = re.findall(r"Faction::(\w+) => GoalRule::None", goals_rs)
    m = re.search(r"export const FACTIONS: Faction\[\] = \[(.*?)\];", data_ts, re.S)
    facts["app_factions"] = re.findall(r"[\"'](\w+)[\"']", m.group(1)) if m else []
    if not m:
        bad.append(f"{DATA_TS}: cannot read FACTIONS (update tools/gates/public-claims.py)")
    m = re.search(r"pub const ERA_COUNT: usize = (\d+);", eras_rs)
    facts["eras"] = int(m.group(1)) if m else 0
    m = re.search(r"pub static TECHS: .*?= \{(.*?)\n\};", eras_rs, re.S)
    facts["techs"] = len(re.findall(r"^\s*t\(\w+, \"", m.group(1), re.M)) if m else 0
    if not facts["eras"] or not facts["techs"]:
        bad.append(f"{ERAS}: cannot read ERA_COUNT or TECHS (update tools/gates/public-claims.py)")
    return facts, bad


def marked(readme, mark):
    return next(((i + 1, line) for i, line in enumerate(readme.splitlines()) if mark in line), (0, ""))


def findings(readme, types_rs, eras_rs, goals_rs, data_ts):
    """Finding lines for a README that disagrees with the code. Pure: every argument is a file's text."""
    facts, out = code_facts(types_rs, eras_rs, goals_rs, data_ts)
    if out:
        return out
    factions, total = facts["factions"], len(facts["factions"])
    if facts["app_factions"] != factions:
        out.append(f"{DATA_TS}: FACTIONS lists {facts['app_factions']}, but Faction::ALL in {TYPES} is {factions}")

    n, line = marked(readme, PEOPLES_MARK)
    if not n:
        out.append(f"{README}: no line carries {PEOPLES_MARK} (the peoples paragraph must keep its marker)")
    else:
        found = numbers(line)
        if not found or found[0] != total:
            out.append(f"{README}:{n}: says {found[0] if found else 'no number of'} peoples; "
                       f"Faction::ALL in {TYPES} has {total}")
        missing = [f for f in factions if not re.search(r"\b" + f + r"\b", line)]
        if missing:
            out.append(f"{README}:{n}: does not name {', '.join(missing)} (Faction::ALL in {TYPES})")
        finished = total - len(facts["unfinished"])
        if facts["unfinished"] and finished not in found[1:]:
            out.append(f"{README}:{n}: does not say that {finished} peoples are finished; {GOALS} gives "
                       f"{', '.join(facts['unfinished'])} no faction goal")

    n, line = marked(readme, ERAS_MARK)
    if not n:
        out.append(f"{README}: no line carries {ERAS_MARK} (the eras and technologies line must keep its marker)")
    else:
        m = re.search(r"\b" + NUMBER + r" technologies\b", line, re.I)
        if not m or number(m.group(1)) != facts["techs"]:
            out.append(f"{README}:{n}: says {m.group(1) if m else 'no number of'} technologies; "
                       f"TECHS in {ERAS} has {facts['techs']}")
        if not re.search(r"\b" + NUMBER + r" eras\b", line, re.I):
            out.append(f"{README}:{n}: states no number of eras; ERA_COUNT in {ERAS} is {facts['eras']}")
    for i, text in enumerate(readme.splitlines()):
        for m in re.finditer(r"\b" + NUMBER + r" eras\b", text, re.I):
            if number(m.group(1)) != facts["eras"]:
                out.append(f"{README}:{i + 1}: says \"{m.group(0)}\"; ERA_COUNT in {ERAS} is {facts['eras']}")
    return out


def check(repo, tip):
    """Finding lines for the tree at TIP."""
    texts = [show(repo, tip, path) for path in SOURCES]
    absent = [path for path, text in zip(SOURCES, texts) if text is None]
    if "README.md" in absent and "crates/core/src/types.rs" in absent:
        return []  # not a Legatum tree (the hook's test fixtures): nothing to compare
    if absent:
        return [f"{path}: missing at {tip[:9]} (update tools/gates/public-claims.py if it moved)" for path in absent]
    return findings(*texts)


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("ref", nargs="?", default="HEAD")
    ap.add_argument("--repo", default=".")
    a = ap.parse_args(argv)
    found = check(a.repo, a.ref)
    for f in found:
        print(f)
    print(f"public-claims: {'FAIL' if found else 'PASS'}")
    return 1 if found else 0


if __name__ == "__main__":
    sys.exit(main())
