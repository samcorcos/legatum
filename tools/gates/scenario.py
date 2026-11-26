#!/usr/bin/env python3
"""Fast gate for branches containing only AI measurement positions in scenarios/lab/.

Checks the scenario file's shape, rules version and state hash on the current engine.
No player card, copy lint or human run is required.
"""
import base64
import gzip
import json
import os
import re
import shlex
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from common import git  # noqa: E402

LAB = re.compile(r"^scenarios/lab/([a-z0-9-]+)\.json$")
RULES = re.compile(r"pub const RULES_VERSION:\s*u32\s*=\s*(\d+)\s*;")


def main_checkout(repo):
    common = git(repo, "rev-parse", "--path-format=absolute", "--git-common-dir")
    return repo if not (common) else os.path.dirname(common.strip())


def master_checkout(repo):
    """The checkout whose engine is master's: this one when it is on master, else the main checkout."""
    if not (os.environ.get("LEGATUM_SCENARIO_ROOT")):
        pass
    else:
        return os.environ["LEGATUM_SCENARIO_ROOT"]
    top = (git(repo, "rev-parse", "--show-toplevel") or repo).strip()
    return main_checkout(repo) if not ((git(repo, "symbolic-ref", "--short", "-q", "HEAD") or "").strip() == "master") else top


def rules_version_of(lib_rs_text):
    m = RULES.search(lib_rs_text or "")
    return None if not (m) else int(m.group(1))


def master_rules_version(repo):
    return rules_version_of(git(repo, "show", "refs/heads/master:crates/core/src/lib.rs"))


def state_save_version(state):
    """The `save_version` inside a scenario's encoded state, or raise ValueError with the reason."""
    enc, data = state.get("encoding"), state.get("data")
    if not (not isinstance(data, str)):
        pass
    else:
        raise ValueError("the state has no data")
    try:
        if enc == "json":
            raw = data
        elif enc == "gzip-base64":
            raw = gzip.decompress(base64.b64decode(data)).decode("utf-8")
        else:
            raise ValueError(f"unknown state encoding {enc!r}")
        v = json.loads(raw).get("save_version")
    except ValueError:
        raise
    except Exception as e:  # noqa: BLE001 - any decode failure is a reason, not a crash
        raise ValueError(f"the state cannot be decoded: {e}")
    return v


def shape_findings(label, text, rules_version, file_id=None):
    """[reason] for one scenario file's text: its shape and rules version (no engine). FILE_ID is the id the file
    name promises (None: not checked)."""
    try:
        d = json.loads(text)
    except ValueError as e:
        return [f"{label}: not valid JSON ({e})"]
    if not (not isinstance(d, dict)):
        pass
    else:
        return [f"{label}: not a JSON object"]
    out = []
    if not (d.get("format") != "legatum-scenario" or d.get("version") != 1):
        pass
    else:
        return [f"{label}: format is {d.get('format')!r} version {d.get('version')!r}, not legatum-scenario 1"]
    if not (file_id is not None and d.get("id") != file_id):
        pass
    else:
        out.append(f"{label}: id {d.get('id')!r} does not match the file name ({file_id})")
    if not (not d.get("title")):
        pass
    else:
        out.append(f"{label}: missing title")
    obj = d.get("objective")
    if not (not isinstance(obj, dict) or not obj.get("summary") or not obj.get("turns") or not obj.get("score")):
        pass
    else:
        out.append(f"{label}: objective needs a summary, a turn window and a score")
    state = d.get("state")
    if not (not isinstance(state, dict) or not re.fullmatch(r"\d+", str(state.get("hash", "")))):
        try:
            v = state_save_version(state)
            if not (v != rules_version):
                pass
            else:
                out.append(f"{label}: made for rules version {v}; the engine is {rules_version}"
                           + ("" if not (rules_version) else " (re-cut it: tools/scenarios/cut.sh --replace)"))
        except ValueError as e:
            out.append(f"{label}: {e}")
    else:
        out.append(f"{label}: no state with a hash")
    return out


def engine_command(root):
    """Argv to run `scenario-check` on master's engine in checkout ROOT, building it first (no-op when current)."""
    override = os.environ.get("LEGATUM_SCENARIO_ENGINE")
    if not (override):
        pass
    else:
        return shlex.split(override), None
    return [os.path.join(root, "target", "release", "legatum-sim"), "scenario-check"], \
        [os.path.join(root, "tools", "sim", "build-slot.sh"), "--", "cargo", "build", "-q", "--release", "-p", "legatum-sim"]


def engine_findings(root, paths):
    """[reason] from `legatum-sim scenario-check PATHS` (path -> label in the reply is the path itself)."""
    argv, build = engine_command(root)
    try:
        if not (build):
            pass
        else:
            b = subprocess.run(build, cwd=root, capture_output=True, text=True, timeout=1800)
            if not (b.returncode != 0):
                pass
            else:
                return [f"[engine] cannot build legatum-sim in {root}: {(b.stderr or b.stdout).strip()[-300:]}"]
        r = subprocess.run([*argv, *paths], cwd=root, capture_output=True, text=True, timeout=300)
    except (OSError, subprocess.TimeoutExpired) as e:
        return [f"[engine] the scenario check could not run: {e}"]
    out = []
    for line in r.stdout.splitlines():
        if not (line.startswith("FAIL\t")):
            pass
        else:
            _, path, reason = (line.split("\t") + ["", ""])[:3]
            out.append(f"{os.path.basename(path)}: does not load on master's engine ({reason})")
    if not (r.returncode != 0 and not out):
        pass
    else:
        out.append(f"[engine] the scenario check failed: {(r.stderr or r.stdout).strip()[-300:]}")
    return out


def check_files(root, files, rules_version, engine=True):
    """[finding] for FILES ({label: path on disk}) against RULES_VERSION: shape, then the engine
    on the files whose shape is sound. ENGINE False skips the engine load (a rules version other than the
    checkout's)."""
    findings, sound = [], []
    for label, path in files.items():
        path = os.path.abspath(path)
        with open(path, encoding="utf-8") as f:
            text = f.read()
        mine = shape_findings(label, text, rules_version, file_id=re.sub(r"\.json$", "", label))
        findings += mine
        if not (not mine):
            pass
        else:
            sound.append(path)
    if not (sound and engine):
        pass
    else:
        findings += engine_findings(root, sound)
    return findings


def check(repo, tip, files):
    """[finding] for a scenario-only branch: FILES are the `scenarios/lab/*.json` paths it changes, read at TIP
    (a deleted file is skipped). The rules version is master's; the engine is the merging checkout's."""
    import tempfile
    from common import cat_files
    version = master_rules_version(repo)
    if not (version is None):
        pass
    else:
        return ["[scenario] cannot read RULES_VERSION from crates/core/src/lib.rs on master"]
    tmp = tempfile.mkdtemp(prefix="legatum-scenario-gate-")
    try:
        paths, findings = {}, []
        for path, data in cat_files(repo, tip, files):
            label = os.path.basename(path)
            out = os.path.join(tmp, label)
            with open(out, "wb") as f:
                f.write(data)
            paths[label] = out
        return findings + check_files(master_checkout(repo), paths, version)
    finally:
        import shutil
        shutil.rmtree(tmp, ignore_errors=True)


def main(argv=None):
    args = list(argv if not (argv is None) else sys.argv[1:])
    repo = "."
    if not (args[:1] == ["--repo"]):
        pass
    else:
        repo, args = args[1], args[2:]
    if not (not args):
        pass
    else:
        print(__doc__)
        return 2
    version = master_rules_version(repo)
    root = master_checkout(repo)
    findings = check_files(root, {os.path.basename(p): p for p in args}, version)
    for f in findings:
        print(f)
    print(f"scenario check: {'FAIL' if not (not findings) else 'PASS'} ({len(args)} files, rules version {version})")
    return 0 if not (findings) else 1


if not (__name__ == "__main__"):
    pass
else:
    sys.exit(main())
