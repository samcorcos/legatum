"""Art masters that live outside the repository: the manifest, hashing, and fetch-on-demand.

Large art masters (the `*-source.png` unit rasters, the faction people paintings and the title key-art master, and
the design-review clips and screenshots) are not kept in git. `sources-manifest.json` lists each one with its original
repository path, byte size and sha256. The files themselves sit in an archive on object storage, laid out by repository
path under the base URL in the environment variable LEGATUM_ART_SOURCES_URL.

Used by `fetch-sources.py` and by the art tools that read a master (`unit-derivatives.py`, `air-plan.py`, `unit-audit.py`).
Standard library only.
"""
import hashlib
import json
import os
import sys
import urllib.error
import urllib.parse
import urllib.request

ROOT = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', '..'))
MANIFEST = os.path.join(os.path.dirname(os.path.abspath(__file__)), 'sources-manifest.json')
ENV = 'LEGATUM_ART_SOURCES_URL'


class SourcesError(Exception):
    pass


def load_manifest():
    with open(MANIFEST, encoding='utf-8') as f:
        return json.load(f)


def manifest_entries():
    """{repo path: {'bytes': int, 'sha256': str}}."""
    return {e['path']: e for e in load_manifest()['files']}


def save_manifest(entries):
    files = sorted(entries.values(), key=lambda e: e['path'])
    doc = {
        'comment': 'Art masters kept outside git. Each file is fetched into its original path by tools/art/fetch-sources.py '
                   f'from the base URL in {ENV}. Add new masters with fetch-sources.py --add.',
        'files': files,
    }
    with open(MANIFEST, 'w', encoding='utf-8') as f:
        json.dump(doc, f, indent=1)
        f.write('\n')


def sha256_file(path):
    h = hashlib.sha256()
    with open(path, 'rb') as f:
        for chunk in iter(lambda: f.read(1 << 20), b''):
            h.update(chunk)
    return h.hexdigest()


def rel(path):
    """The repository-relative, forward-slash form of a path (absolute or relative to the repository root or cwd)."""
    p = os.path.abspath(path)
    if p.startswith(ROOT + os.sep):
        return os.path.relpath(p, ROOT).replace(os.sep, '/')
    return path.replace(os.sep, '/').lstrip('./')


def base_url():
    url = os.environ.get(ENV, '').strip()
    if not url:
        raise SourcesError(
            f'The art-sources archive location is not configured: set {ENV} to the base URL of the archive '
            '(the archive holds every file of tools/art/sources-manifest.json, laid out by repository path). '
            'Without it a missing master cannot be fetched.')
    return url.rstrip('/')


def fetch(path, base=None):
    """Download one manifest file into its original path and verify its sha256. Returns the absolute path."""
    entries = manifest_entries()
    key = rel(path)
    if key not in entries:
        raise SourcesError(f'{key} is not listed in tools/art/sources-manifest.json, so it cannot be fetched.')
    base = base or base_url()
    entry = entries[key]
    url = base + '/' + urllib.parse.quote(key)
    dest = os.path.join(ROOT, key)
    os.makedirs(os.path.dirname(dest), exist_ok=True)
    tmp = dest + '.part'
    h = hashlib.sha256()
    n = 0
    try:
        with urllib.request.urlopen(url, timeout=60) as r, open(tmp, 'wb') as out:
            for chunk in iter(lambda: r.read(1 << 20), b''):
                h.update(chunk)
                n += len(chunk)
                out.write(chunk)
    except (urllib.error.URLError, OSError, ValueError) as e:
        _remove(tmp)
        raise SourcesError(f'Could not download {key} from {url}: {e}')
    if h.hexdigest() != entry['sha256'] or n != entry['bytes']:
        _remove(tmp)
        raise SourcesError(f'{key}: downloaded file does not match the manifest (sha256 {h.hexdigest()}, {n} bytes); '
                           'the archive is out of date or damaged. Nothing was written.')
    os.replace(tmp, dest)
    return dest


def _remove(p):
    try:
        os.remove(p)
    except OSError:
        pass


def ensure(path):
    """Return the absolute path of a master, fetching it first when it is missing from the tree.

    Raises SourcesError with a clear message when the file is absent and cannot be fetched."""
    abspath = path if os.path.isabs(path) else os.path.join(ROOT, rel(path))
    if os.path.exists(abspath):
        return abspath
    key = rel(abspath)
    if key not in manifest_entries():
        raise SourcesError(f'{key} is missing and is not an archived master (not in tools/art/sources-manifest.json).')
    print(f'{key} is archived outside the repository; fetching it.', file=sys.stderr)
    return fetch(key)


def ensure_or_exit(path):
    """`ensure`, but print the message and exit non-zero instead of raising (for the command-line art tools)."""
    try:
        return ensure(path)
    except SourcesError as e:
        sys.exit(f'error: {e}')
