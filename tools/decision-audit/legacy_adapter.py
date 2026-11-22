#!/usr/bin/env python3
"""Adapt the current decision audit example to engines before it was introduced.

Old releases lack the optional diagnostic APIs and several later order variants. The
replay, shadow Hard orders, category swaps, and paired continuations stay unchanged.
"""
import sys
import subprocess
import re
from pathlib import Path


def replace_once(source, old, new):
    assert source.count(old) == 1, old[:80]
    return source.replace(old, new)


def remove_between(source, first, last):
    start = source.index(first)
    end = source.index(last, start)
    return source[:start] + source[end:]


def adapt(source, has):
    variants = {
        "SetAutoFire": "        | Order::SetAutoFire { unit, .. }\n",
        "UpgradeSiegeUnit": "        | Order::UpgradeSiege { unit }",
        "AirStrike": "        Order::AirStrike { plane, .. } | Order::Rebase { plane, .. } => Some(*plane),\n",
        "HireCompany": "        | Order::HireCompany { .. }\n",
        "UpgradeSiegeBuy": "        | Order::UpgradeSiege { .. }",
    }
    for name, old in variants.items():
        symbol = "UpgradeSiege" if name.startswith("UpgradeSiege") else name
        if not has(symbol):
            source = replace_once(source, old, "")
    if not has("Action"):
        source = replace_once(source, "use legatum_core::orders::Action;\n", "")
        source = replace_once(source, "        Order::Action {\n            action: Action::ChooseFreeTechs { .. },\n        } => 1,\n", "")
    if not has("is_aircraft", "crates/core/src/data.rs"):
        source = replace_once(source, "    if legatum_core::data::is_aircraft(u.kind) {\n        return 4;\n    }\n", "")
    source = remove_between(source, "/// `--army-detail`: one row", "/// One value run:")
    source = remove_between(source, "/// Candidate orders use only", "#[derive(Clone)]\nstruct Score")
    source = remove_between(source, "    let mut army_out = args", "    let mut snaps:")
    source = remove_between(source, "        let mut alt_trace = None;", "        for id in alive {")
    source = remove_between(source, "        if let Some(f) = peace_out.as_mut() {", "        let in_range =")
    source = remove_between(source, "            if let (Some(f), Some(tr)) =", "            let differs:")
    source = re.sub(r"                if args\.rival \{\n                    let view = player_view\(&state, seat\);.*?\n                \} else \{\n", "", source, count=1, flags=re.S)
    source = replace_once(source, "                }\n            }\n        }\n        state = next;", "            }\n        }\n        state = next;")
    start = source.index("            let planned = if id == seat &&")
    end = source.index("            if id == seat {", start)
    source = source[:start] + "            let planned = brains.get_mut(&id).unwrap().orders(&player_view(&state, id));\n" + source[end:]
    return source


if __name__ == "__main__":
    src, dst = map(Path, sys.argv[1:3])
    repo, engine = sys.argv[3:5]
    def has(symbol, path="crates/core/src/orders.rs"):
        return subprocess.run(["git", "-C", repo, "grep", "-q", symbol, engine, "--", path], check=False).returncode == 0
    dst.write_text(adapt(src.read_text(), has))
    subprocess.run(["rustfmt", "--edition", "2024", str(dst)], check=True)
