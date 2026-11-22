"""Reviewer markers for the merge gate.

A reviewer agent writes `.progress/<kind>-review/<branch with / as ->.ok` in the main checkout when the branch passes:
the first line is the reviewed tip SHA, then free lines (`reviewed <date> by <agent>`, `copy: clean`, ...).
A marker stays valid while the branch tip equals that SHA, or the SHA is an ancestor of the tip and no later branch
commit (outside master) touches the paths that reviewer judges. Any marker naming the exact tip also counts (merging
by SHA, a renamed branch).
"""
import os

from common import git

REVIEWERS = {
    "docs": ("docs-review", "legatum-docs-reviewer"),
    "ui": ("ui-review", "legatum-ui-reviewer"),
    "plain": ("plain-review", "legatum-plain-language-reviewer"),
    "art": ("art-review", "legatum-art-reviewer"),
}


def main_checkout(repo):
    common = git(repo, "rev-parse", "--path-format=absolute", "--git-common-dir")
    return os.path.dirname(common.strip()) if common else repo


def marker_file(main, kind, name):
    return os.path.join(main, ".progress", REVIEWERS[kind][0], name.replace("/", "-") + ".ok")


def read_marker(path):
    try:
        with open(path) as f:
            lines = [ln.strip() for ln in f.read().splitlines()]
    except OSError:
        return None, []
    head = lines[0].split()[0] if lines and lines[0].split() else None
    return head, lines[1:]


def check_marker(repo, kind, name, tip, relevant):
    """(ok, message, extra marker lines). RELEVANT(path) says whether a later commit's file needs a fresh review."""
    main = main_checkout(repo)
    path = marker_file(main, kind, name)
    reviewed, extra = read_marker(path)
    if reviewed and (git(repo, "rev-parse", "--verify", "-q", reviewed + "^{commit}") or "").strip() == tip:
        return True, "", extra
    mdir = os.path.dirname(path)
    try:
        for fn in sorted(os.listdir(mdir)):
            if fn.endswith(".ok"):
                head, more = read_marker(os.path.join(mdir, fn))
                if head == tip:
                    return True, "", more
    except OSError:
        pass
    if not reviewed:
        return False, f"no {REVIEWERS[kind][1]} marker ({os.path.relpath(path, main)})", []
    if git(repo, "merge-base", "--is-ancestor", reviewed, tip) is None:
        return False, f"its marker names {reviewed[:9]}, which is not in the branch", []
    later = (git(repo, "rev-list", "--no-merges", f"{reviewed}..{tip}", "^refs/heads/master") or "").split()
    files = set()
    for sha in later:
        files.update((git(repo, "diff-tree", "--no-commit-id", "--name-only", "-r", sha) or "").splitlines())
    stale = sorted(f for f in files if f and relevant(f))
    if stale:
        return False, (f"files it judges changed after its review at {reviewed[:9]} "
                       f"({', '.join(stale[:5])}{' ...' if len(stale) > 5 else ''})"), []
    return True, "", extra


def marker_status(repo, name, tip, kinds, classes):
    """[(kind, ok, message)] for the reviews this branch needs. CLASSES maps a kind to its path predicate."""
    out = []
    if kinds["code"]:
        ok, msg, _ = check_marker(repo, "docs", name, tip, classes["code"])
        out.append(("docs", ok, msg))
    if kinds["copy"]:
        # Player text (user, 2026-11-21): a card says the short rule, the Guide holds the detail, in plain words.
        ok, msg, _ = check_marker(repo, "plain", name, tip, classes["copy"])
        if not ok:
            msg += " (player text changed: " + ", ".join(kinds["copy"][:4]) + ")"
        out.append(("plain", ok, msg))
    if kinds["ui"]:
        ok, msg, _ = check_marker(repo, "ui", name, tip, classes["ui"])
        out.append(("ui", ok, msg))
    if kinds["art"] or kinds.get("artimpact"):
        # Art, and changes that may make art stale (user, 2026-11-22): the art reviewer judges both.
        ok, msg, _ = check_marker(repo, "art", name, tip, classes["art"])
        if not ok:
            msg += " (art or art-related code changed: " + ", ".join((kinds["art"] + kinds.get("artimpact", []))[:4]) + ")"
        out.append(("art", ok, msg))
    return out
