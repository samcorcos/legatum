#!/usr/bin/env python3
"""Personal-data scan for the public tree: generic patterns from the repository plus the owner's private ones.

    tools/release/scrub.py scan DIR          every text file under DIR (the exported tree)
    tools/release/scrub.py scan --tracked    every tracked file of this checkout
    tools/release/scrub.py pattern           one combined regex (for referenced-excluded.py --scan-pattern)
    tools/release/scrub.py sample CLASS      a line that matches CLASS (for tests)
    tools/release/scrub.py classes           the class names

Generic patterns live in tools/release/scrub-patterns.txt (format described there). Owner-specific literals (an
account id, an address, a name) are never tracked: they are read, when the file exists, from the first of
    $LEGATUM_SCRUB_PATTERNS, <checkout>/.private/scrub-patterns.txt, <main checkout>/.private/scrub-patterns.txt
one regex per line (see .private.example/scrub-patterns.txt). Hits print as `path:line: [class] text`; exit 1 on any.
Standard library only, and it runs under `python3 -I`.
"""
import os
import re
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
PATTERNS = "tools/release/scrub-patterns.txt"
PRIVATE = ".private/scrub-patterns.txt"
# The pattern files describe the patterns, so they are not scanned with them.
SELF = re.compile(r"^(tools/release/scrub-patterns\.txt|\.private\.example/.*|\.private/.*)$")
SKIP_NAME = re.compile(r"(\.lock|^package-lock\.json)$")
SKIP_DIR = {".git", "node_modules", "target"}
EMAIL = re.compile(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9-]+(?:\.[A-Za-z0-9-]+)*\.[A-Za-z]{2,}")
EMAIL_OK = re.compile(r"(@(?:[\w.-]+\.)?(?:example\.(?:com|org|net)|example|invalid|test|localhost)$|^noreply@|^no-reply@"
                      r"|@users\.noreply\.github\.com$|\.(?:png|jpe?g|webp|svg|gif)$|@\d+x\b)", re.I)


class Rules:
    """Compiled patterns by class. `match(path, line)` gives (class, matched text) for the first hit, or None."""

    def __init__(self):
        self.classes = {}  # name -> {"patterns": [compiled], "allow": [compiled], "samples": [str]}
        self.sources = []  # every regex as written, for `pattern`

    def add(self, cls, regex, where):
        try:
            compiled = re.compile(regex)
        except re.error as e:
            raise SystemExit(f"scrub: bad regex in {where}: {e}")
        self.classes.setdefault(cls, {"patterns": [], "allow": [], "samples": []})["patterns"].append(compiled)
        self.sources.append(regex)

    @classmethod
    def from_text(cls, text, private_lines=(), where=PATTERNS):
        rules, current = cls(), None
        for n, raw in enumerate(text.splitlines(), 1):
            line = raw.strip()
            if not line or line.startswith("#"):
                continue
            head = re.fullmatch(r"\[([a-z0-9-]+)\]", line)
            if head:
                current = head.group(1)
                rules.classes.setdefault(current, {"patterns": [], "allow": [], "samples": []})
            elif current is None:
                raise SystemExit(f"scrub: {where}:{n}: a pattern before the first [class]")
            elif line.startswith("allow-path:"):
                rules.classes[current]["allow"].append(re.compile(line.split(":", 1)[1].strip()))
            elif line.startswith("sample:"):
                rules.classes[current]["samples"].append(line.split(":", 1)[1].strip().replace("{}", ""))
            else:
                rules.add(current, line, f"{where}:{n}")
        for n, raw in enumerate(private_lines, 1):
            line = raw.strip()
            if line and not line.startswith("#"):
                rules.add("private", line, f"the private pattern file, line {n}")
        return rules

    @classmethod
    def legacy(cls, regex, private_lines=()):
        """Rules from one combined regex (an exporter that still carries its patterns inline)."""
        rules = cls.from_text("", private_lines)
        rules.add("personal", regex, "the exporter's inline pattern")
        return rules

    def match(self, path, line):
        if SELF.search(path):
            return None
        for name, c in self.classes.items():
            if any(a.search(path) for a in c["allow"]):
                continue
            for p in c["patterns"]:
                m = p.search(line)
                if m:
                    return name, m.group(0)
        return None

    def combined(self):
        return "|".join(f"(?:{s})" for s in self.sources)


def emails(path, line):
    """Email addresses on LINE that are not on the allow-list (example domains, noreply addresses, image names)."""
    if SELF.search(path):
        return []
    return [e for e in EMAIL.findall(line) if not EMAIL_OK.search(e)]


def private_file(repo=REPO):
    """The owner's private pattern file, or None. Never tracked; see .private.example/."""
    cands = [os.environ.get("LEGATUM_SCRUB_PATTERNS"), os.path.join(repo, PRIVATE)]
    try:
        common = subprocess.run(["git", "-C", repo, "rev-parse", "--path-format=absolute", "--git-common-dir"],
                                capture_output=True, text=True).stdout.strip()
        if common:
            cands.append(os.path.join(os.path.dirname(common), PRIVATE))
    except OSError:
        pass
    for c in cands:
        if c and os.path.isfile(c):
            return c
    return None


def private_lines(repo=REPO):
    path = private_file(repo)
    if not path:
        return []
    with open(path, encoding="utf-8") as f:
        return f.read().splitlines()


def load(repo=REPO):
    with open(os.path.join(REPO, PATTERNS), encoding="utf-8") as f:
        return Rules.from_text(f.read(), private_lines(repo))


def files_under(root, tracked):
    if tracked:
        out = subprocess.run(["git", "-C", root, "ls-files", "-z"], capture_output=True, check=True).stdout
        return [p for p in out.decode("utf-8", "replace").split("\0") if p]
    found = []
    for d, dirs, names in os.walk(root):
        dirs[:] = [x for x in dirs if x not in SKIP_DIR]
        found.extend(os.path.relpath(os.path.join(d, n), root) for n in names)
    return sorted(found)


def scan(root, tracked=False, rules=None):
    """[(path, line number, class, text)] for every hit under ROOT."""
    rules = rules or load()
    hits = []
    for rel in files_under(root, tracked):
        if SKIP_NAME.search(os.path.basename(rel)) or SELF.search(rel):
            continue
        try:
            with open(os.path.join(root, rel), "rb") as f:
                data = f.read()
        except OSError:
            continue
        if b"\0" in data[:8000] or len(data) > 5_000_000:
            continue
        for n, line in enumerate(data.decode("utf-8", "replace").split("\n"), 1):
            hit = rules.match(rel, line)
            if hit:
                hits.append((rel, n, hit[0], line.strip()))
            for _ in emails(rel, line):
                hits.append((rel, n, "email", line.strip()))
    return hits


def main(argv):
    if len(argv) >= 1 and argv[0] == "scan" and len(argv) in (2, 3):
        tracked = "--tracked" in argv[1:]
        rest = [a for a in argv[1:] if a != "--tracked"]
        root = os.path.abspath(rest[0]) if rest else REPO
        if not tracked and not rest:
            print(__doc__, file=sys.stderr)
            return 2
        hits = scan(root, tracked)
        for rel, n, cls, text in hits:
            print(f"{rel}:{n}: [{cls}] {text[:200]}")
        if not private_file():
            print("scrub: no private pattern file found; only the generic patterns were applied "
                  "(.private.example/scrub-patterns.txt explains how to add yours)", file=sys.stderr)
        return 1 if hits else 0
    if argv == ["pattern"]:
        print(load().combined())
        return 0
    if argv == ["classes"]:
        print("\n".join(c for c in load().classes if c != "private"))  # the generic classes; private has no sample
        return 0
    if len(argv) == 2 and argv[0] == "sample":
        samples = load().classes.get(argv[1], {}).get("samples")
        if not samples:
            print(f"scrub: no sample for class `{argv[1]}`", file=sys.stderr)
            return 2
        print(samples[0])
        return 0
    print(__doc__, file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
