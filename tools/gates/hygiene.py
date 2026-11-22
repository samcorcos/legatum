#!/usr/bin/env python3
"""Hygiene and public-repo check: what a branch adds must be fit for the public repository.

Checks the lines and files that BASE..TIP adds:
  secret         credentials and keys in any file (AWS access keys, private keys, Anthropic/OpenAI/GitHub/Slack/
                 Google tokens, secret assignments); gitleaks also runs over the commits when it is installed
  personal       the public exporter's personal-data patterns (home paths, relationship words, account IDs, private
                 IPs) and email addresses, in files the exporter publishes. The generic patterns are read from
                 tools/release/scrub-patterns.txt and EXCLUDE from tools/release/export-public.sh, both at master, so
                 the gate and the export never drift; the owner's untracked .private/scrub-patterns.txt is added when
                 it exists (tools/release/scrub.py)
  save           committed game saves (*.legatum, *.legatum.gz) outside the test fixture folders
  large-binary   binary files over 2 MB outside the art asset folders
  backup-file    *.backup, *.bak, *.orig, *~ files
  commented-code commented-out code: a comment line that parses as a statement, outside doc comments
  legacy-marker  `// legacy`, `// old:`, `// legacy code` markers (delete deprecated code; git keeps history); prose about
                 old data ("// Legacy saves have no history") is fine
  migration      an edit to, or deletion of, a committed migration (add a new migration instead)
  split-file     docs/learnings.md over 30 KB or the game-rules.md stub over 5 KB: an older branch's merge brought
                 the one-file version back (restore it and run the split script the message names)

Escape for one line: a `gate-allow: <reason>` comment on the line or the line above (not for secrets).

  tools/gates/hygiene.py [--repo DIR] BASE TIP     what BASE..TIP adds (the merge gate)
  tools/gates/hygiene.py [--repo DIR] --all [REF]  every tracked file at REF (default HEAD)
Exit 1 when anything is found.
"""
import argparse
import ast
import os
import re
import shutil
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(1, os.path.join(os.path.dirname(HERE), "release"))
from common import allowed, cat_files, changed_files, diff_added, git, show  # noqa: E402
import scrub  # noqa: E402

BINARY_CAP = 2_000_000
ART = re.compile(r"^(app/src/assets/|app/public/|docs/media/|docs/design/|tools/design/)")
FIXTURES = re.compile(r"(^|/)(tests?/fixtures|e2e/saves|test/fixtures|testdata)/")
SECRET = re.compile(
    r"AKIA[0-9A-Z]{16}|ASIA[0-9A-Z]{16}|-----BEGIN (?:RSA |EC |DSA |OPENSSH |PGP )?PRIVATE KEY"
    r"|\bsk-ant-[A-Za-z0-9_-]{20,}|\bsk-(?:proj-)?[A-Za-z0-9]{32,}|\bgh[pousr]_[A-Za-z0-9]{36}\b|\bgithub_pat_[A-Za-z0-9_]{40,}"
    r"|\bxox[abprs]-[A-Za-z0-9-]{10,}|\bAIza[0-9A-Za-z_-]{35}\b"
    r"|(?i:aws_secret_access_key|secret_key|api_key|apikey|auth_token|password)\s*[:=]\s*[\"'][A-Za-z0-9/+_=-]{16,}[\"']")
CODE_EXT = re.compile(r"\.(rs|ts|tsx|js|mjs|cjs|py|sh)$")
SLASH = re.compile(r"^\s*//(?![/!])\s?(.*)$")
HASH = re.compile(r"^\s*#(?![!\[])\s?(.*)$")
# A comment body that is a statement rather than prose.
STATEMENT = re.compile(
    r"^(?:(?:let|const|var) (?:mut )?[\w(), \[\]{}]+(?::[^=]+)?=[^=].*|fn \w+\(.*|pub (?:fn|struct|enum) \w+.*|use [\w:{}, ]+;$|import [\w{}*, ]+ from |return\b.*;$|if \(.*\)\s*\{?$"
    r"|if .*\{$|for \(.*\)|for \w+ in .*\{$|while \(.*\)|match .*\{$|\} else \{$|\}\s*\)?;?$"
    r"|[\w.:\[\]]+(?:\(.*\))?\s*(?:[+\-*/]?=|\.\w+\(.*\))\s*.*;$|[\w.:]+\(.*\);$|self\.\w+.*;$|await .*;$|def \w+\(.*\):$)")
# A marker, not prose about old data: "// legacy", "// old:", "// legacy code", "# old implementation" (not when quoted).
LEGACY = re.compile(r"(?<![`\"'])(?://|#)\s*(?:legacy|old)\b(?:\s*[:(\-]|\s*$|\s+(?:code|version|impl|implementation|path|logic|api|"
                    r"approach|method|way|behaviou?r|function|handler|branch|stuff)\b)", re.I)
BACKUP = re.compile(r"(\.backup|\.bak|\.orig|~)$")
MIGRATION = re.compile(r"(^|/)migrations?/")
SPLIT_FILES = [
    ("docs/learnings.md", 30_000,
     "re-add your entries at the top of docs/learnings/<YYYY-MM>.md, or run `python3 tools/docs/split_learnings.py "
     "<your branch before the merge> --base $(git merge-base origin/master <that commit>) --write`"),
    (".claude/skills/legatum-rules/references/game-rules.md", 5_000,
     "re-add your rule lines to the matching file in .claude/skills/legatum-rules/references/ (index.md lists them)"),
]


def export_rules(repo, rev):
    """(personal-data rules, [excluded path prefixes], [kept paths]) from the public exporter at REV, or (None, [], [])."""
    text = show(repo, rev, "tools/release/export-public.sh")
    if not text:
        return None, [], []
    private = scrub.private_lines(os.path.abspath(repo))
    patterns = show(repo, rev, scrub.PATTERNS)
    if patterns:
        rules = scrub.Rules.from_text(patterns, private)
    else:  # an exporter from before scrub-patterns.txt carries one inline regex
        m = re.search(r"^\s*pat='([^']+)'", text, re.M)
        rules = scrub.Rules.legacy(m.group(1), private) if m else None
    ex = re.search(r"^EXCLUDE=\((.*?)\)", text, re.M | re.S)
    keep = re.search(r"^SIM_KEEP=\((.*?)\)", text, re.M | re.S)
    excluded = ex.group(1).split() if ex else []
    return rules, excluded, (keep.group(1).split() if keep else [])


def exported(path, excluded, kept):
    if path in kept:
        return True
    return not any(path == e or path.startswith(e.rstrip("/") + "/") for e in excluded)


def python_statement(body):
    """A Python comment body that parses as code (an assignment, call, import or block header), not prose."""
    if not re.search(r"[=(\[.:]", body):
        return False
    src = body + ("\n    pass" if body.endswith(":") else "")
    try:
        tree = ast.parse(src)
    except (SyntaxError, ValueError):
        return False
    first = tree.body[0] if tree.body else None
    return first is not None and not (isinstance(first, ast.Expr) and isinstance(first.value, (ast.Name, ast.Constant)))


def line_checks(path, lines, only, personal, publish):
    """[(line_no, rule, msg, src, allow_reason)] for LINES ({line no: text}), checking the line numbers in ONLY."""
    hits = []
    code = bool(CODE_EXT.search(path))
    comment = HASH if path.endswith((".py", ".sh")) else SLASH

    def allow(i):
        return allowed([lines.get(i - 1, ""), lines.get(i, "")], 1)

    for i in sorted(only):
        raw = lines.get(i, "")
        if SECRET.search(raw):
            hits.append((i, "secret", "a credential or key; remove it, rotate it, and load it from the environment",
                         "<redacted>", None))
            continue
        if publish:
            hit = personal.match(path, raw) if personal else None
            if hit:
                hits.append((i, "personal", f"personal data ({hit[0]}) in a file the public export publishes; "
                             "generalise it (`~/`, `<name>`, `<lan-ip>`, a role)", raw.strip(), allow(i)))
            for e in scrub.emails(path, raw):
                hits.append((i, "personal", f"email address `{e}`; use an example.com or noreply address",
                             raw.strip(), allow(i)))
        if code:
            if LEGACY.search(raw):
                hits.append((i, "legacy-marker", "legacy/old marker: delete deprecated code instead of labelling it",
                             raw.strip(), allow(i)))
            m = comment.match(raw)
            body = m.group(1).strip() if m else ""
            if m and "gate-allow" not in raw and (STATEMENT.match(body) if comment is SLASH else
                                                  path.endswith(".py") and python_statement(body)):
                hits.append((i, "commented-code", "commented-out code: delete it (git keeps the history)",
                             raw.strip(), allow(i)))
    return hits


def gitleaks(repo, base, tip):
    if not shutil.which("gitleaks") or base is None:
        return []
    r = subprocess.run(["gitleaks", "git", repo, "--no-banner", "--redact", "--exit-code", "1",
                        f"--log-opts={base}..{tip}"], capture_output=True, text=True, timeout=120)
    if r.returncode != 1:
        return []
    found = [ln.strip() for ln in (r.stdout + r.stderr).splitlines() if ln.strip().startswith(("File:", "Line:", "RuleID:"))]
    return ["[secret] gitleaks: " + " ".join(found[k:k + 3]) for k in range(0, len(found), 3)] or ["[secret] gitleaks found a secret"]


def sizes(repo, rev, paths=None):
    out = git(repo, "ls-tree", "-r", "-l", rev, *(["--"] + paths if paths else [])) or ""
    res = {}
    for line in out.splitlines():
        meta, _, path = line.partition("\t")
        parts = meta.split()
        if len(parts) == 4 and parts[3].isdigit():
            res[path] = int(parts[3])
    return res


def check(repo, base, tip, all_mode=False):
    master = "refs/heads/master" if git(repo, "rev-parse", "-q", "--verify", "refs/heads/master") else tip
    personal, excluded, kept = export_rules(repo, master)
    findings, allows = [], []

    def add(where, rule, msg, src, reason):
        if reason and rule != "secret":
            allows.append(f"{where}: [{rule}] allowed ({reason})")
        else:
            findings.append(f"{where}: [{rule}] {msg}" + (f"\n    {src[:160]}" if src else ""))

    size = sizes(repo, tip)
    if all_mode:
        rows = [("A", p) for p in size]
        binary = set()
        texts = {}
        wanted = [p for p in size if size[p] <= 5_000_000]
        for path, data in cat_files(repo, tip, wanted):
            if b"\0" in data[:8000]:
                binary.add(path)
            else:
                t = data.decode("utf-8", "replace").split("\n")
                lines = {i + 1: v for i, v in enumerate(t)}
                texts[path] = (lines, set(lines))
    else:
        rows = changed_files(repo, base, tip)
        texts = diff_added(repo, base, tip)
        numstat = git(repo, "diff", "--numstat", "--no-renames", base, tip) or ""
        binary = {ln.split("\t")[-1] for ln in numstat.splitlines() if ln.startswith("-\t-\t")}
    for status, path in rows:
        name = os.path.basename(path)
        if status in "MD" and MIGRATION.search(path) and not all_mode:
            add(path, "migration", "a committed migration was edited or deleted; add a new migration instead", "", None)
        if status == "D":
            continue
        if BACKUP.search(name):
            add(path, "backup-file", "backup file; delete it (git keeps the history)", "", None)
        if re.search(r"\.legatum(\.gz)?$", name) and not FIXTURES.search(path):
            add(path, "save", "a game save outside the test fixture folders; keep saves in ~/LegatumData, or move a "
                "test position under tests/fixtures", "", None)
        if path in binary:
            if size.get(path, 0) > BINARY_CAP and not ART.search(path):
                add(path, "large-binary", f"{size[path] / 1e6:.1f} MB binary outside the art folders (cap "
                    f"{BINARY_CAP / 1e6:.0f} MB); keep run output out of git or shrink it", "", None)
            continue
        if path not in texts or size.get(path, 0) > 5_000_000:
            continue  # huge generated text (sim output): not worth a line scan
        lines, only = texts[path]
        for line_no, rule, msg, src, reason in line_checks(path, lines, only, personal, exported(path, excluded, kept)):
            add(f"{path}:{line_no}", rule, msg, src, reason)
    # The learnings log and the rules reference were split on 2026-11-22 (docs/learnings/<YYYY-MM>.md, one rules file
    # per system). A merge from an older branch can drag the one-file versions back in; block that loudly.
    for path, cap, fix in SPLIT_FILES:
        if size.get(path, 0) > cap:
            add(path, "split-file", f"{size[path]:,} bytes, over the {cap:,}-byte cap for this file since the split. "
                f"Restore it from origin/master (`git checkout origin/master -- {path}`), then {fix} "
                f"(CONTRIBUTING.md, rule 8)", "", None)
    if not all_mode:
        findings.extend(gitleaks(repo, base, tip))
    return findings, allows


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--repo", default=".")
    ap.add_argument("--all", action="store_true", help="scan every tracked file at one commit")
    ap.add_argument("--verbose", "-v", action="store_true", help="also list the lines a gate-allow covers")
    ap.add_argument("revs", nargs="*")
    a = ap.parse_args(argv)
    if a.all:
        base, tip = None, (a.revs[0] if a.revs else "HEAD")
    elif len(a.revs) == 2:
        base, tip = a.revs
    else:
        ap.error("give BASE TIP, or --all [REF]")
    findings, allows = check(a.repo, base, tip, a.all)
    if a.verbose:
        for line in allows:
            print("allow  " + line)
    for line in findings:
        print("FAIL   " + line)
    print(f"hygiene: {len(findings)} finding(s), {len(allows)} allowed")
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
