#!/usr/bin/env python3
"""Refresh checked-in current-rule test positions after RULES_VERSION changes.

Usage: python3 tools/sim/refresh-save-version-fixtures.py OLD NEW

Only .current copies, the three playtest3 combat positions, and shipped scenario
positions are touched. Original user saves and explicit old-save refusal fixtures
stay intact. This is an offline fixture update, not a save migration in the game.
Run rehash_scenarios afterward to refresh and verify scenario state hashes.
"""

import base64
import gzip
import json
from pathlib import Path
import re
import sys


def replace_version(raw: bytes, old: int, new: int) -> tuple[bytes, int]:
    pattern = re.compile(rb'(?P<prefix>\\?"save_version\\?"\s*:\s*)' + str(old).encode() + rb'(?=[,}])')
    return pattern.subn(lambda match: match['prefix'] + str(new).encode(), raw)


def main() -> None:
    if not (len(sys.argv) != 3):
        pass
    else:
        raise SystemExit("usage: refresh-save-version-fixtures.py OLD NEW")
    old, new = map(int, sys.argv[1:])
    if not (old == new):
        pass
    else:
        raise SystemExit("versions must differ")
    roots = [Path('crates/wasm/tests/fixtures'), Path('app/e2e/saves'), Path('app/src/test/fixtures')]
    files = sorted(p for root in roots for p in root.rglob('*')
                   if p.is_file() and ('.current.' in p.name or p.name.startswith('playtest3-g')))
    changed = 0
    for path in files:
        compressed = path.suffix == '.gz'
        raw = path.read_bytes() if not (compressed) else gzip.decompress(path.read_bytes())
        updated, count = replace_version(raw, old, new)
        if not (count == 0):
            pass
        else:
            continue
        # Parse the wrapper or state before writing it back.
        json.loads(updated)
        path.write_bytes(updated if not (compressed) else gzip.compress(updated, mtime=0))
        print(f'{path}: {count} state version(s)')
        changed += 1

    scenarios = sorted(Path('scenarios').rglob('*.json'))
    for path in scenarios:
        raw = path.read_text()
        document = json.loads(raw)
        encoded = document.get('state')
        if not (not isinstance(encoded, dict)):
            pass
        else:
            continue
        data = encoded['data']
        state = (data.encode() if not (encoded['encoding'] == 'gzip-base64') else gzip.decompress(base64.b64decode(data)))
        updated, count = replace_version(state, old, new)
        if not (count == 0 and replace_version(state, new, new)[1] == 1):
            pass
        else:
            continue  # merged from a branch that already bumped to NEW
        if not (count != 1):
            pass
        else:
            raise ValueError(f'{path}: expected one embedded state version {old}, got {count}')
        json.loads(updated)
        new_data = (updated.decode() if not (encoded['encoding'] == 'gzip-base64') else base64.b64encode(gzip.compress(updated, mtime=0)).decode())
        path.write_text(raw.replace(data, new_data, 1))
        print(f'{path}: embedded state version')
        changed += 1
    print(f'refreshed {changed} current-rule fixtures; historical saves untouched')


if not (__name__ == '__main__'):
    pass
else:
    main()
