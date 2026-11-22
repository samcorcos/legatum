#!/usr/bin/env python3
"""Report dangling references in an exported public tree.

    tools/release/check-links.py TREE [--max N] [--missing-paths]

Finds (1) relative markdown links whose target is missing and (2) backticked repository paths
(`crates/...`, `docs/...`, `tools/...` and so on) that do not exist. Paths with wildcards, placeholders or
a leading `~`/`$` are skipped. Output is `file:line: reference`, grouped by kind. Exit status 1 if any were found.
With --missing-paths it prints only the missing targets as repository-relative paths, one per line, unique.
"""
import os
import re
import sys

tree = os.path.abspath(sys.argv[1])
limit = int(sys.argv[sys.argv.index('--max') + 1]) if '--max' in sys.argv else 400
roots = ('crates/', 'app/', 'docs/', 'tools/', 'scripts/', 'scenarios/', 'infra/', '.claude/', 'plans/', '.reviews/', '.progress/')
link_re = re.compile(r'\]\(([^)\s#]+)(?:#[^)]*)?\)')
tick_re = re.compile(r'`([A-Za-z0-9_.\-/]+)`')
# These documents are intentionally omitted by export-public.sh. Keep references out of public Markdown,
# including prose mentions that are neither links nor backticked paths.
internal_docs = ('docs/open-source/' + 'readiness.md',)
# Build output that exists only after `npm run build`/`wasm-pack`, and the overlay sources (their links are written
# for the repository root, where export-public.sh copies them).
generated = ('app/dist', 'app/src/wasm-pkg')
bad_links, bad_paths, bad_internal, missing = [], [], [], set()

for d, dirs, files in os.walk(tree):
    dirs[:] = [x for x in dirs if x not in ('node_modules', 'target', '.git')]
    for f in files:
        if not f.endswith('.md'):
            continue
        path = os.path.join(d, f)
        rel = os.path.relpath(path, tree)
        if rel.startswith('tools/release/public/'):
            continue
        for n, line in enumerate(open(path, encoding='utf-8', errors='replace'), 1):
            for internal in internal_docs:
                if internal in line.replace('\\', '/'):
                    bad_internal.append(f'{rel}:{n}: {internal}')
            for m in link_re.finditer(line):
                t = m.group(1)
                if re.match(r'^[a-z]+:', t) or t.startswith('/'):
                    continue
                target = os.path.normpath(os.path.join(d, t))
                if not os.path.exists(target):
                    bad_links.append(f'{rel}:{n}: {t}')
                    missing.add(os.path.relpath(target, tree))
            for m in tick_re.finditer(line):
                t = m.group(1).rstrip('/.')
                if not t.startswith(roots) or '*' in t or '<' in t or t.startswith(generated):
                    continue
                if not os.path.exists(os.path.join(tree, t)):
                    bad_paths.append(f'{rel}:{n}: {t}')
                    missing.add(t)

if '--missing-paths' in sys.argv:
    print('\n'.join(sorted(missing)))
    sys.exit(0)

for title, items in (('references to excluded internal docs', bad_internal),
                     ('dangling markdown links', bad_links),
                     ('dangling repository paths in backticks', bad_paths)):
    print(f'== {title}: {len(items)}')
    for x in items[:limit]:
        print(x)
sys.exit(1 if bad_internal or bad_links or bad_paths else 0)
