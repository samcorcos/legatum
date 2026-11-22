#!/usr/bin/env python3
"""Print the left-out files that code in the exported tree still names, so the export can bring them back.

    git ls-tree -r --name-only COMMIT | tools/release/referenced-excluded.py OUT_DIR

Reads every path of the exported commit on stdin, keeps the ones missing from OUT_DIR, and prints those that a string
literal in the exported code names: an `include_str!`/`include_bytes!` fixture, a default `--ai-params` path, a file a
test opens. A literal counts when it resolves to the path from the file's own directory or from the repository root.

    --commit SHA --repo DIR --scan-pattern REGEX   read each candidate from the commit and leave out any file over
                                                   10 MB or whose text (gzip unpacked) matches the personal-data pattern;
                                                   what it leaves out goes to stderr.
"""
import argparse
import gzip
import os
import re
import subprocess
import sys

ap = argparse.ArgumentParser()
ap.add_argument('out')
ap.add_argument('--commit')
ap.add_argument('--repo', default='.')
ap.add_argument('--scan-pattern')
args = ap.parse_args()
out = os.path.abspath(args.out)
missing = {p for p in (line.strip() for line in sys.stdin) if p and not os.path.lexists(os.path.join(out, p))}

CODE_DIRS = ('crates', 'app/src', 'app/e2e', 'scripts', 'tools', '.github')
CODE_EXT = ('.rs', '.ts', '.tsx', '.js', '.mjs', '.py', '.sh', '.yml', '.yaml', '.toml')
LITERAL = re.compile(r'["\'`]([^"\'`\s$]*/[^"\'`\s$]+\.[A-Za-z0-9]+)["\'`]')

found = set()
for top in CODE_DIRS:
    for d, dirs, files in os.walk(os.path.join(out, top)):
        dirs[:] = [x for x in dirs if x not in ('node_modules', 'target', 'wasm-pkg', 'assets')]
        for f in files:
            if not f.endswith(CODE_EXT):
                continue
            path = os.path.join(d, f)
            try:
                text = open(path, encoding='utf-8').read()
            except (UnicodeDecodeError, OSError):
                continue
            here = os.path.relpath(d, out)
            for lit in LITERAL.findall(text):
                for cand in (os.path.normpath(os.path.join(here, lit)), os.path.normpath(lit.lstrip('./'))):
                    if cand in missing:
                        found.add(cand)


def blocked(path):
    """Why a referenced file still stays out, or None."""
    if not (args.commit and args.scan_pattern):
        return None
    data = subprocess.run(['git', '-C', args.repo, 'cat-file', 'blob', f'{args.commit}:{path}'],
                          check=True, capture_output=True).stdout
    if len(data) > 10 * 1024 * 1024:
        return 'over 10 MB'
    if path.endswith('.gz'):
        try:
            data = gzip.decompress(data)
        except OSError:
            pass
    if re.search(args.scan_pattern, data.decode('utf-8', 'replace')):
        return 'matches the personal-data pattern'
    return None


for path in sorted(found):
    why = blocked(path)
    if why:
        print(f'referenced but left out ({why}): {path}', file=sys.stderr)
    else:
        print(path)
