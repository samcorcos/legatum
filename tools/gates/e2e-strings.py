#!/usr/bin/env python3
"""Browser specs that still expect text a branch removes from the app.

  tools/gates/e2e-strings.py <branch> [--repo DIR] [--base REF]

Pull requests run only the browser smoke set (ci.yml, e2e-smoke); the rest of app/e2e runs nightly. A branch that
rewords a label, title, notice or class name in app/src can leave a non-smoke spec asserting the old words, and only
the nightly finds it. This check is the cheap version: every string literal or JSX text the branch removes from
app/src (tests left out) that no app/src file still contains at the tip is looked up in app/e2e at the tip, and each
spec line that still holds it is a finding. A spec line names the string literally or as a regex (`\\.` and friends
are unescaped before the match).

Escape for one line: `// gate-allow: <reason>` on the spec line or the line above it.
"""
import argparse
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from common import allowed, cat_files, git  # noqa: E402

SRC = re.compile(r"^app/src/.*\.(tsx?|css)$")
SRC_TEST = re.compile(r"\.test\.tsx?$|^app/src/test/|/__snapshots__/")
E2E = re.compile(r"^app/e2e/.*\.(ts|mjs)$")
# Quoted literals (no escaped quote handling beyond skipping \x) and JSX text between tags.
LITERAL = re.compile(r"'((?:[^'\\\n]|\\.)*)'|\"((?:[^\"\\\n]|\\.)*)\"|`((?:[^`\\]|\\.)*)`")
JSX_TEXT = re.compile(r">\s*([^<>{}\n]*[A-Za-z][^<>{}\n]*?)\s*<")
# A JSX child on a line of its own: words only, no code punctuation.
JSX_LINE = re.compile(r"^\s*([A-Z][^<>{}=;()'\"`]*[A-Za-z.!?])\s*$")
COMMENT = re.compile(r"^\s*(//|/?\*)")
INLINE_COMMENT = re.compile(r"\s//\s.*$")
# Short or code-shaped strings match too much: import paths, single identifiers like 'none', CSS values.
MIN_LEN = 6
SKIP = re.compile(r"^(\.{1,2}/|@|https?:|[a-z]+$|[\d\s.,%px#()-]+$)")


def strings(line):
    """The static text pieces of the literals and JSX text on one source line."""
    if COMMENT.match(line):
        return []
    out = []
    for m in LITERAL.finditer(line):
        text = next(g for g in m.groups() if g is not None)
        out.extend(re.split(r"\$\{[^}]*\}", text))
    out.extend(m.group(1) for m in JSX_TEXT.finditer(line))
    line_text = JSX_LINE.match(line)
    if line_text and " " in line_text.group(1):
        out.append(line_text.group(1))
    seen = []
    for s in (s.strip() for s in out):
        if len(s) >= MIN_LEN and re.search(r"[A-Za-z]", s) and not SKIP.match(s) and s not in seen:
            seen.append(s)
    return seen


def removed(repo, base, tip):
    """{string: source path} for literals on lines BASE..TIP deletes or changes in app/src (tests left out)."""
    out = git(repo, "diff", "-U0", "--no-renames", "--no-color", "--no-ext-diff", base, tip, "--", "app/src") or ""
    res, path = {}, None
    for line in out.split("\n"):
        if line.startswith("--- "):
            path = line[6:] if line.startswith("--- a/") else None
        elif line.startswith("-") and path and SRC.search(path) and not SRC_TEST.search(path):
            for s in strings(line[1:]):
                res.setdefault(s, path)
    return res


def texts(repo, rev, pattern, exclude=None):
    paths = [p for p in (git(repo, "ls-tree", "-r", "--name-only", rev) or "").splitlines()
             if pattern.search(p) and not (exclude and exclude.search(p))]
    return {p: d.decode("utf-8", "replace") for p, d in cat_files(repo, rev, paths)}


def unescape(line):
    return re.sub(r"\\(.)", r"\1", line)


def check(repo, base, tip):
    gone = removed(repo, base, tip)
    if not gone:
        return []
    # What the app still says: its source without comments (a comment naming the old words keeps nothing on screen).
    src = "\n".join(INLINE_COMMENT.sub("", line) for text in texts(repo, tip, SRC, SRC_TEST).values()
                    for line in text.split("\n") if not COMMENT.match(line))
    gone = {s: p for s, p in gone.items() if s not in src}
    if not gone:
        return []
    findings = []
    for path, text in sorted(texts(repo, tip, E2E).items()):
        lines = text.split("\n")
        for i, line in enumerate(lines):
            if COMMENT.match(line):
                continue
            plain = unescape(line)
            for s, where in gone.items():
                if (s in line or s in plain) and not allowed(lines, i):
                    findings.append(f"{path}:{i + 1}: still expects {s!r}, which this branch removes from {where}")
    return findings


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("branch")
    ap.add_argument("--repo", default=".")
    ap.add_argument("--base", default="refs/heads/master")
    a = ap.parse_args(argv)
    tip = (git(a.repo, "rev-parse", "--verify", "-q", a.branch + "^{commit}") or "").strip()
    base = (git(a.repo, "merge-base", a.base, tip) or "").strip() if tip else ""
    if not tip or not base:
        print(f"cannot resolve {a.branch} or its merge base with {a.base}", file=sys.stderr)
        return 2
    findings = check(a.repo, base, tip)
    for f in findings:
        print(f)
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
