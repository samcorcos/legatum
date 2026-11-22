#!/usr/bin/env python3
"""Drop a Scenario Lab scenario into the installed app's scenarios folder: no build, no install.

  tools/scenarios/publish.sh SCENARIO.json [--rules-version N] [--dir DIR] [--dry-run]

It checks the file against the INSTALLED app's rules version, runs the quick validation (the state loads on the
engine, the objective parses, the copy lint passes), and copies it to ~/LegatumData/scenarios/lab/<id>.json. The
desktop app reads that folder each time the Scenario Lab opens, so the scenario is there at once. A file with the same
id as a bundled scenario replaces it; a file the app cannot use is listed greyed out with the reason.

The installed rules version comes, in order, from `--rules-version N`; from ~/LegatumData/app-info.json when the app
wrote it after the install (it does on every launch since the scenario folder landed); else from the newest playtest-*
tag older than /Applications/Legatum.app's executable (read from `crates/core/src/lib.rs` at that tag). The script
says which one it used. The engine check runs only when that version is this checkout's; otherwise the file is
checked for shape, version, copy and nothing else, and the script says so.
"""
import argparse
import json
import os
import re
import shutil
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, os.path.join(REPO, "tools", "gates"))
import scenario as gate  # noqa: E402

APP = "/Applications/Legatum.app"


def home_dir():
    return os.environ.get("LEGATUM_DATA_DIR") or os.path.join(os.path.expanduser("~"), "LegatumData")


def app_executable(app):
    macos = os.path.join(app, "Contents", "MacOS")
    try:
        names = sorted(os.listdir(macos))
    except OSError:
        return None
    return os.path.join(macos, names[0]) if names else None


def installed_rules_version(app=APP, data=None, repo=REPO):
    """(version, how it was found) or (None, why not)."""
    data = data or home_dir()
    exe = app_executable(app)
    if not exe:
        return None, f"no installed app at {app}; pass --rules-version N"
    built = os.path.getmtime(exe)
    info = os.path.join(data, "app-info.json")
    try:
        if os.path.getmtime(info) >= built:
            v = json.load(open(info))["rules_version"]
            return int(v), f"{info} (written by the app after it was installed)"
    except (OSError, ValueError, KeyError, TypeError):
        pass
    tags = gate.git(repo, "for-each-ref", "refs/tags/playtest-*", "--format=%(refname:short) %(*committerdate:unix)%(committerdate:unix)") or ""
    best = None
    for line in tags.splitlines():
        name, _, stamp = line.partition(" ")
        stamp = re.sub(r"\D", "", stamp)
        # a tag's `%(*committerdate)` is empty for a lightweight tag; take the commit's date directly
        when = (gate.git(repo, "log", "-1", "--format=%ct", name) or "").strip()
        if when.isdigit() and int(when) <= built and (best is None or int(when) > best[0]):
            best = (int(when), name)
    if not best:
        return None, "no playtest-* tag older than the installed app; pass --rules-version N"
    v = gate.rules_version_of(gate.git(repo, "show", f"{best[1]}:crates/core/src/lib.rs"))
    if v is None:
        return None, f"cannot read RULES_VERSION at {best[1]}; pass --rules-version N"
    return v, f"{best[1]}, the newest playtest tag older than {exe}"


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("file")
    ap.add_argument("--rules-version", type=int)
    ap.add_argument("--dir", default=os.environ.get("LEGATUM_SCENARIO_DIR") or os.path.join(home_dir(), "scenarios", "lab"))
    ap.add_argument("--app", default=APP)
    ap.add_argument("--dry-run", action="store_true", help="check only; copy nothing")
    a = ap.parse_args(argv)

    try:
        text = open(a.file, encoding="utf-8").read()
        sid = json.loads(text)["id"]
    except (OSError, ValueError, KeyError, TypeError) as e:
        print(f"publish: {a.file} is not a scenario file ({e})", file=sys.stderr)
        return 2
    if not isinstance(sid, str) or not re.fullmatch(r"[a-z0-9-]+", sid):
        print(f"publish: the id {sid!r} may use lowercase letters, digits and dashes only", file=sys.stderr)
        return 2

    if a.rules_version is not None:
        version, how = a.rules_version, "--rules-version"
    else:
        version, how = installed_rules_version(a.app)
        if version is None:
            print(f"publish: {how}", file=sys.stderr)
            return 2
    print(f"installed rules version {version} ({how})")

    root = gate.master_checkout(REPO)
    mine = gate.rules_version_of(open(os.path.join(root, "crates", "core", "src", "lib.rs")).read())
    findings = gate.check_files(root, {f"{sid}.json": a.file}, version, engine=(mine == version))
    if mine != version:
        print(f"note: this checkout's engine is rules version {mine}, so the state was not loaded in the engine; "
              f"the shape, rules version and copy were checked")
    if findings:
        print("NOT published:", file=sys.stderr)
        for f in findings:
            print("  " + f, file=sys.stderr)
        return 1
    dest = os.path.join(a.dir, f"{sid}.json")
    if a.dry_run:
        print(f"ok: would copy to {dest}")
        return 0
    os.makedirs(a.dir, exist_ok=True)
    tmp = dest + ".tmp"
    shutil.copyfile(a.file, tmp)
    os.replace(tmp, dest)
    print(f"published {sid} -> {dest}")
    print("Open the Scenario Lab in the app; it reads the folder each time it opens. Commit the file to scenarios/lab/ too, "
          "so later builds bundle it.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
