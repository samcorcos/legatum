#!/usr/bin/env python3
"""Art gate: run the lookalike check (tools/art/distinctness.py --render --check) on a branch's art, without a checkout.

Extracts the art sets and the art tools at TIP into a temporary folder (git archive), links the main checkout's
app/node_modules for headless Chromium, renders every set at display size and fails when a pair is too similar and not
accepted in tools/art/distinctness.json. About 30 s; a pass is cached per art tree in .progress/merge-gates/.

  tools/gates/art.py [--repo DIR] TIP
Exit 1 when a pair fails or the check cannot run.
"""
import argparse
import hashlib
import os
import re
import shutil
import subprocess
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from common import git  # noqa: E402

PATHS = ["tools/art", "app/src/assets", "app/public/wonders", "app/package.json"]
TRIGGER = re.compile(r"^app/src/assets/.*\.(png|svg|jpe?g|webp)$|^app/public/wonders/"
                     r"|^tools/art/(distinctness\.(py|json)|render-sets\.mjs|launch-chromium\.mjs)$")


def main_checkout(repo):
    common = git(repo, "rev-parse", "--path-format=absolute", "--git-common-dir")
    return os.path.dirname(common.strip()) if common else repo


def check(repo, tip):
    """[finding lines]; empty when the art passes."""
    if git(repo, "cat-file", "-t", f"{tip}:tools/art/distinctness.py") is None:
        return []
    present = [p for p in PATHS if git(repo, "cat-file", "-t", f"{tip}:{p}")]
    key = hashlib.sha1(" ".join((git(repo, "rev-parse", f"{tip}:{p}") or "").strip() for p in present).encode()).hexdigest()
    main = main_checkout(repo)
    cache_dir = os.path.join(main, ".progress", "merge-gates")
    ok = os.path.join(cache_dir, f"art-{key[:16]}.ok")
    if os.path.exists(ok):
        return []
    modules = os.environ.get("LEGATUM_ART_NODE_MODULES") or os.path.join(main, "app", "node_modules")
    if not os.path.isdir(os.path.join(modules, "playwright")):
        return [f"[art] cannot render the art sets: no Playwright in {modules}; run `npm ci` in app/ of the main checkout"]
    tmp = tempfile.mkdtemp(prefix="legatum-art-gate-")
    try:
        archive = subprocess.run(["git", "-C", repo, "archive", tip, "--", *present], capture_output=True)
        if archive.returncode != 0:
            return [f"[art] git archive failed: {archive.stderr.decode()[:200]}"]
        subprocess.run(["tar", "-x", "-C", tmp], input=archive.stdout, check=True)
        os.symlink(modules, os.path.join(tmp, "app", "node_modules"))
        try:
            r = subprocess.run(["python3", os.path.join(tmp, "tools", "art", "distinctness.py"), "--render", "--cache",
                                os.path.join(tmp, "cache"), "--check"], capture_output=True, text=True, timeout=240)
        except subprocess.TimeoutExpired:
            return ["[art] the lookalike check timed out after 240 s"]
        if r.returncode == 0:
            os.makedirs(cache_dir, exist_ok=True)
            with open(ok, "w") as f:
                f.write(f"{tip}\n{r.stdout.strip().splitlines()[-1] if r.stdout.strip() else ''}\n")
            return []
        failing = [ln for ln in r.stdout.splitlines() if ln.startswith("FAIL ")]
        if not failing:
            return [f"[art] tools/art/distinctness.py failed: {(r.stderr or r.stdout).strip()[-400:]}"]
        return [f"[art] too similar at display size: {re.sub(r'  +', ' ', ln[5:])[:200]}" for ln in failing[:8]] + (
            [f"[art] ... and {len(failing) - 8} more"] if len(failing) > 8 else []) + [
            "[art] repaint one of each pair (silhouette first, then colour family and value), or accept the pair in "
            "tools/art/distinctness.json with a reason; reproduce with `python3 tools/art/distinctness.py --render --check`"]
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--repo", default=".")
    ap.add_argument("tip", nargs="?", default="HEAD")
    a = ap.parse_args(argv)
    findings = check(a.repo, a.tip)
    for line in findings:
        print("FAIL   " + line)
    print(f"art: {len(findings)} finding(s)")
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
