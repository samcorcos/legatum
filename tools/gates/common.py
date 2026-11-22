"""Shared helpers for the pre-merge gates in tools/gates/ (standard library only).

A gate check looks at one landing: the commits between master and a branch tip (BASE..TIP), read from git objects,
so it needs no checkout of the branch. Line checks report findings on added lines only; `--all` mode scans every
tracked line at one commit (used to tune the checks on master).

Escape for one line: a `gate-allow: <reason>` comment on the line itself or on the line directly above it
(`// gate-allow: ...` in Rust and TypeScript, `# gate-allow: ...` in Python and shell). The reason is required.
"""
import re
import subprocess

ALLOW = re.compile(r"(?://|#|/\*)\s*gate-allow:\s*(\S.*)$")


def git(repo, *args, binary=False):
    r = subprocess.run(["git", "-C", repo, *args], capture_output=True)
    if r.returncode != 0:
        return None
    return r.stdout if binary else r.stdout.decode("utf-8", "replace")


def changed_files(repo, base, tip):
    """[(status, path)] for BASE..TIP, renames reported as the new path."""
    out = git(repo, "diff", "--name-status", "--no-renames", base, tip) or ""
    rows = []
    for line in out.splitlines():
        parts = line.split("\t")
        if len(parts) >= 2:
            rows.append((parts[0][0], parts[-1]))
    return rows


def show(repo, rev, path, binary=False):
    return git(repo, "show", f"{rev}:{path}", binary=binary)


def added_lines(repo, base, tip, path):
    """Line numbers (1-based, in TIP's version) that BASE..TIP adds or changes in PATH."""
    out = git(repo, "diff", "-U0", "--no-renames", base, tip, "--", path) or ""
    lines = set()
    for m in re.finditer(r"^@@ -\d+(?:,\d+)? \+(\d+)(?:,(\d+))? @@", out, re.M):
        start, count = int(m.group(1)), int(m.group(2) or "1")
        lines.update(range(start, start + count))
    return lines


def allowed(lines, i):
    """The gate-allow reason covering line index i (0-based), or None."""
    for j in (i, i - 1):
        if 0 <= j < len(lines):
            m = ALLOW.search(lines[j])
            if m and m.group(1).strip(" */"):
                return m.group(1).strip(" */")
    return None


def strip_rust(text):
    """Per line: the code with comments removed and string and char literals blanked to "" (Rust lexing, enough
    for pattern checks: nested block comments, raw strings, byte strings, lifetimes)."""
    out, cur = [], []
    i, n = 0, len(text)
    depth = 0  # block comment nesting
    while i < n:
        c = text[i]
        if c == "\n":
            out.append("".join(cur))
            cur = []
            i += 1
            continue
        if depth:
            if text.startswith("*/", i):
                depth -= 1
                i += 2
            elif text.startswith("/*", i):
                depth += 1
                i += 2
            else:
                i += 1
            continue
        if text.startswith("//", i):
            while i < n and text[i] != "\n":
                i += 1
            continue
        if text.startswith("/*", i):
            depth = 1
            i += 2
            continue
        m = re.match(r'b?r(#*)"', text[i:i + 40])
        if m and (i == 0 or not (text[i - 1].isalnum() or text[i - 1] == "_")):
            close = '"' + m.group(1)
            j = text.find(close, i + m.end())
            j = n if j < 0 else j + len(close)
            cur.append('""')
            for _ in range(text.count("\n", i, j)):  # keep line numbers for multi-line raw strings
                out.append("".join(cur))
                cur = []
            i = j
            continue
        if c == '"':
            j = i + 1
            while j < n and text[j] != '"':
                j += 2 if text[j] == "\\" else 1
            seg = text[i:j + 1]
            cur.append('""')
            for _ in range(seg.count("\n")):
                out.append("".join(cur))
                cur = []
            i = j + 1
            continue
        if c == "'":
            m = re.match(r"'(\\.[^']*|[^\\'])'", text[i:i + 12])
            if m:
                cur.append("''")
                i += m.end()
                continue
        cur.append(c)
        i += 1
    out.append("".join(cur))
    return out


def test_regions(code_lines):
    """Flags per line: True inside a `#[cfg(test)]` module or item (by brace matching on stripped code)."""
    flags = [False] * len(code_lines)
    i = 0
    while i < len(code_lines):
        if re.search(r"#\[cfg\((?:all\()?test\b", code_lines[i]):
            # the attribute covers the next item: find its opening brace (or `;` for `mod x;` / `use`)
            depth, started, j = 0, False, i
            while j < len(code_lines):
                for ch in code_lines[j]:
                    if ch == "{":
                        depth += 1
                        started = True
                    elif ch == "}":
                        depth -= 1
                flags[j] = True
                if (started and depth <= 0) or (not started and code_lines[j].rstrip().endswith(";")):
                    break
                j += 1
            i = j + 1
            continue
        i += 1
    return flags


def diff_added(repo, base, tip):
    """{path: ({tip line no: text} for added and context lines, {added line nos})} for BASE..TIP, from one diff
    with one line of context (so a gate-allow comment on the unchanged line above an added line is seen)."""
    out = git(repo, "diff", "-U1", "--no-renames", "--no-color", "--no-ext-diff", base, tip) or ""
    files, path, lines, added, n = {}, None, None, None, 0
    for line in out.split("\n"):
        if line.startswith("diff --git "):
            path = None
        elif line.startswith("+++ "):
            path = line[6:] if line.startswith("+++ b/") else None
            if path is not None:
                lines, added = {}, set()
                files[path] = (lines, added)
        elif line.startswith("@@"):
            m = re.match(r"@@ -\d+(?:,\d+)? \+(\d+)", line)
            n = int(m.group(1)) if m else 0
        elif path is None or line.startswith(("--- ", "\\")):
            continue
        elif line.startswith("+"):
            lines[n] = line[1:]
            added.add(n)
            n += 1
        elif line.startswith(" "):
            lines[n] = line[1:]
            n += 1
    return files


def cat_files(repo, rev, paths):
    """Yield (path, bytes) for PATHS at REV through one `git cat-file --batch` process."""
    p = subprocess.Popen(["git", "-C", repo, "cat-file", "--batch"], stdin=subprocess.PIPE, stdout=subprocess.PIPE)
    try:
        for path in paths:
            p.stdin.write(f"{rev}:{path}\n".encode())
            p.stdin.flush()
            header = p.stdout.readline().split()
            if len(header) < 3 or header[1] == b"missing":
                continue
            size = int(header[2])
            data = p.stdout.read(size)
            p.stdout.read(1)
            yield path, data
    finally:
        p.stdin.close()
        p.wait()
