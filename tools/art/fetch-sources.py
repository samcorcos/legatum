#!/usr/bin/env python3
"""Fetch, pack, verify or register the art masters that are kept outside git.

    python3 tools/art/fetch-sources.py PATH...        fetch these repository paths into place (skips files already there)
    python3 tools/art/fetch-sources.py --all          fetch every manifest file that is missing
    python3 tools/art/fetch-sources.py --pack DIR     copy every manifest file from the tree into DIR (laid out by
                                                      repository path) and verify each hash; upload DIR to object storage
    python3 tools/art/fetch-sources.py --verify       check the hash of every manifest file that is present in the tree
    python3 tools/art/fetch-sources.py --add PATH...  add (or refresh) files in the manifest from their current content

The archive base URL comes from the environment variable LEGATUM_ART_SOURCES_URL; each file is requested at
<base>/<repository path>. No default host is configured. A downloaded file is written only when its sha256 and size match
the manifest. Standard library only. Masters are listed in tools/art/sources-manifest.json.
"""
import argparse
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import artsources as A  # noqa: E402


def cmd_fetch(paths, everything, force):
    entries = A.manifest_entries()
    keys = sorted(entries) if everything else [A.rel(p) for p in paths]
    todo = [k for k in keys if force or not os.path.exists(os.path.join(A.ROOT, k))]
    unknown = [k for k in keys if k not in entries]
    if unknown:
        print('not in the manifest: ' + ', '.join(unknown), file=sys.stderr)
        return 2
    if not todo:
        print(f'{len(keys)} file(s) already present.')
        return 0
    try:
        base = A.base_url()
    except A.SourcesError as e:
        print(f'error: {e}', file=sys.stderr)
        return 1
    failed = 0
    for i, k in enumerate(todo, 1):
        try:
            A.fetch(k, base)
            print(f'[{i}/{len(todo)}] {k}')
        except A.SourcesError as e:
            failed += 1
            print(f'[{i}/{len(todo)}] FAILED {k}: {e}', file=sys.stderr)
    print(f'fetched {len(todo) - failed} of {len(todo)}.')
    return 1 if failed else 0


def _copy(src, out):
    # Plain read/write rather than shutil.copyfile: its in-kernel copy corrupted files on a network-backed mount.
    with open(src, 'rb') as i, open(out, 'wb') as o:
        for chunk in iter(lambda: i.read(1 << 20), b''):
            o.write(chunk)
        o.flush()
        os.fsync(o.fileno())


def cmd_pack(dest):
    entries = A.manifest_entries()
    bad = 0
    for k, e in sorted(entries.items()):
        src = os.path.join(A.ROOT, k)
        if not os.path.isfile(src):
            print(f'MISSING in tree: {k}', file=sys.stderr)
            bad += 1
            continue
        if A.sha256_file(src) != e['sha256'] or os.path.getsize(src) != e['bytes']:
            print(f'HASH MISMATCH in tree: {k}', file=sys.stderr)
            bad += 1
            continue
        out = os.path.join(dest, k)
        os.makedirs(os.path.dirname(out), exist_ok=True)
        _copy(src, out)
    if bad:
        print(f'{bad} file(s) failed; do not delete the masters from the tree.', file=sys.stderr)
        return 1
    # Verify the copies, not the originals.
    for k, e in sorted(entries.items()):
        out = os.path.join(dest, k)
        if not os.path.isfile(out) or A.sha256_file(out) != e['sha256'] or os.path.getsize(out) != e['bytes']:
            print(f'PACKED COPY DOES NOT VERIFY: {k}', file=sys.stderr)
            return 1
    total = sum(e['bytes'] for e in entries.values())
    print(f'packed and verified {len(entries)} files ({total} bytes) in {dest}')
    return 0


def cmd_verify():
    entries = A.manifest_entries()
    present = bad = 0
    for k, e in sorted(entries.items()):
        p = os.path.join(A.ROOT, k)
        if not os.path.isfile(p):
            continue
        present += 1
        if A.sha256_file(p) != e['sha256'] or os.path.getsize(p) != e['bytes']:
            print(f'HASH MISMATCH: {k}', file=sys.stderr)
            bad += 1
    print(f'{present} of {len(entries)} manifest files present; {bad} mismatched.')
    return 1 if bad else 0


def cmd_add(paths):
    entries = A.manifest_entries() if os.path.exists(A.MANIFEST) else {}
    for p in paths:
        k = A.rel(p)
        full = os.path.join(A.ROOT, k)
        if not os.path.isfile(full):
            print(f'not a file: {k}', file=sys.stderr)
            return 2
        entries[k] = {'path': k, 'bytes': os.path.getsize(full), 'sha256': A.sha256_file(full)}
    A.save_manifest(entries)
    print(f'manifest now lists {len(entries)} files. Upload them to the archive and do not commit them.')
    return 0


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('paths', nargs='*')
    ap.add_argument('--all', action='store_true')
    ap.add_argument('--force', action='store_true', help='fetch even when the file is already present')
    ap.add_argument('--pack', metavar='DIR')
    ap.add_argument('--verify', action='store_true')
    ap.add_argument('--add', action='store_true')
    a = ap.parse_args()
    if a.pack:
        return cmd_pack(a.pack)
    if a.verify:
        return cmd_verify()
    if a.add:
        return cmd_add(a.paths)
    if not a.all and not a.paths:
        ap.print_help()
        return 2
    return cmd_fetch(a.paths, a.all, a.force)


if __name__ == '__main__':
    sys.exit(main())
