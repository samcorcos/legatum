#!/usr/bin/env python3
"""List the files under app/src/assets that the app bundles at runtime or that its tests read.

The source tree keeps art masters, prompts, review sheets and derivatives side by side. The app only loads the
derivatives, through Vite `import.meta.glob(...)` patterns and a few direct imports. This script expands those patterns
against the tree, so tools/release/export-public.sh can leave the unreferenced masters out of the public repository.

    tools/release/runtime-assets.py [REPO_ROOT]      prints repo-relative paths, one per line

The set is deliberately conservative: a literal `assets/...` path that appears in non-test source counts as used, and a
`${...}` placeholder becomes a `*` wildcard. A test that names an asset directory (`new URL('./assets/units/', ...)`)
keeps every file directly in it, because such tests check masters and sources the app itself never loads.
"""
import fnmatch
import os
import re
import sys

root = os.path.abspath(sys.argv[1] if len(sys.argv) > 1 else os.path.join(os.path.dirname(__file__), '..', '..'))
src = os.path.join(root, 'app', 'src')
assets = os.path.join(src, 'assets')


def brace_expand(pattern):
    m = re.search(r'\{([^{}]*)\}', pattern)
    if not m:
        return [pattern]
    return [p for alt in m.group(1).split(',') for p in brace_expand(pattern[:m.start()] + alt + pattern[m.end():])]


def glob_to_regex(pattern):
    out, i = '', 0
    while i < len(pattern):
        if pattern.startswith('**/', i):
            out += '(?:.*/)?'
            i += 3
        elif pattern[i] == '*':
            out += '[^/]*'
            i += 1
        elif pattern[i] == '?':
            out += '[^/]'
            i += 1
        else:
            out += re.escape(pattern[i])
            i += 1
    return re.compile('^' + out + '$')


source_files = []
test_files = []
for d, dirs, files in os.walk(src):
    dirs[:] = [x for x in dirs if x not in ('assets', 'mock', 'wasm-pkg', 'node_modules')]
    for f in files:
        if f.endswith(('.ts', '.tsx', '.css')):
            (test_files if '.test.' in f else source_files).append(os.path.join(d, f))

patterns = []  # absolute glob patterns
for path in source_files:
    text = open(path, encoding='utf-8').read()
    here = os.path.dirname(path)
    for m in re.finditer(r'import\.meta\.glob(?:<[^>]*>)?\(\s*(\[[^\]]*\]|[\'"][^\'"]+[\'"])', text):
        for lit in re.findall(r'[\'"]([^\'"]+)[\'"]', m.group(1)):
            patterns.append(os.path.normpath(os.path.join(here, lit)))
    for m in re.finditer(r'[\'"`]([^\'"`\n]*?assets/[^\'"`\n]+?)[\'"`]', text):
        lit = re.sub(r'\$\{[^}]*\}', '*', m.group(1))
        tail = lit[lit.index('assets/'):]
        if re.search(r'\.\w+$', tail):
            patterns.append(os.path.join(src, tail))

for path in test_files:
    text = open(path, encoding='utf-8').read()
    here = os.path.dirname(path)
    for m in re.finditer(r'[\'"`]((?:\.{1,2}/)*assets/[\w./-]*/)[\'"`]', text):
        patterns.append(os.path.join(os.path.normpath(os.path.join(here, m.group(1))), '*'))

regexes = [glob_to_regex(p) for pat in patterns for p in brace_expand(pat)]

kept = []
for d, _, files in os.walk(assets):
    for f in files:
        full = os.path.join(d, f)
        if any(r.match(full) for r in regexes):
            kept.append(os.path.relpath(full, root))
print('\n'.join(sorted(kept)))
