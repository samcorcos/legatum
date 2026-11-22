#!/usr/bin/env python3
"""Run matched Hard conditions with launch/exit provenance.

Build target/release/legatum-sim first, then run from the repository root.
Completed batches can be resumed only with the same binary and source files.
"""
import argparse
from datetime import datetime, timezone
import gzip
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import time


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", type=Path, default=Path('.progress/hard-openings-handoff'))
    parser.add_argument("--tables", nargs='+', choices=['old', 'v2', 'handoff'],
                        default=['old', 'v2', 'handoff'])
    args = parser.parse_args()
    root = Path.cwd()
    out = args.out.resolve()
    out.mkdir(parents=True, exist_ok=True)
    # Cargo workspace tests can relink target/release/legatum-sim with unified
    # features. Keep this run's executable outside Cargo's output directory.
    binary = out / 'bin/legatum-sim'
    if not binary.exists():
        binary.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(root / 'target/release/legatum-sim', binary)
    binary_hash = sha(binary)
    paths = subprocess.check_output(['git', 'ls-files', '-co', '--exclude-standard'], text=True).splitlines()
    hashes = {p: sha(Path(p)) for p in sorted(set(paths))
              if p.endswith('.rs') or Path(p).name in ('Cargo.toml', 'Cargo.lock')}
    source_path = out / 'source.json'
    if source_path.exists():
        source = json.loads(source_path.read_text())
        assert source['binary_sha256'] == binary_hash
        assert source['files_sha256'] == hashes
        assert source.get('tables', ['old', 'v2', 'handoff']) == args.tables
    else:
        with gzip.open('docs/sim-results/2026-10-01-hard-openings-confirmation-data.json.gz', 'rt') as handle:
            previous = json.load(handle)
        (out / 'evolved-members.json').write_text(previous['members_json'])
        source = {
            'base_commit': subprocess.check_output(['git', 'merge-base', 'HEAD', 'master'], text=True).strip(),
            'tables': args.tables,
            'head_at_launch': subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip(),
            'branch': subprocess.check_output(['git', 'branch', '--show-current'], text=True).strip(),
            'rustc': subprocess.check_output(['rustc', '--version'], text=True).strip(),
            'binary_sha256': binary_hash,
            'members_sha256': sha(out / 'evolved-members.json'),
            'files_sha256': hashes,
            'started_at': datetime.now(timezone.utc).isoformat(),
        }
        source_path.write_text(json.dumps(source, indent=2) + '\n')
    receipts_path = out / 'runs.json'
    receipts = json.loads(receipts_path.read_text()) if receipts_path.exists() else []
    completed = {r['label'] for r in receipts if r['exit_code'] == 0}
    assert len(completed) == len(receipts), 'inspect failed batches before resuming'
    for opponent in ['GreedyExpand', 'Snowball', 'Rush', 'Proxy']:
        for table in args.tables:
            label = f'{opponent.lower()}-{table}'
            if label in completed:
                continue
            assert sha(binary) == binary_hash, 'frozen simulator changed during the run'
            dest = out / 'raw' / label
            dest.mkdir(parents=True, exist_ok=True)
            assert not list(dest.glob('*.json')), 'unreceipted results require inspection'
            command = ['nice', '-n', '10', str(binary), '--brain', 'scripted',
                       '--difficulty', 'hard', '--players', '8',
                       '--threads', '8', '--seed', '3500001', '--games',
                       '128' if opponent == 'Proxy' else '400',
                       '--label', label, '--out-dir', str(dest),
                       '--status-file', str(out / 'status.json')]
            command += ['--proxy', '0', '--rotate-proxy'] if opponent == 'Proxy' else ['--challenger', opponent]
            command += {'old': ['--legacy-hard', 'all'], 'v2': ['--hard-v2'],
                        'handoff': ['--hard-v2-handoff']}[table]
            print(f'START {label}', flush=True)
            start = time.time()
            with (out / f'{label}.log').open('w') as log:
                process = subprocess.Popen(command, stdout=log, stderr=subprocess.STDOUT)
                niceness = os.getpriority(os.PRIO_PROCESS, process.pid)
                while niceness < 10 and process.poll() is None:
                    time.sleep(0.01)  # Let nice exec before recording the launch priority.
                    niceness = os.getpriority(os.PRIO_PROCESS, process.pid)
                try:
                    exit_code = process.wait()
                except KeyboardInterrupt:
                    if process.poll() is None:
                        process.terminate()
                    process.wait()
                    raise
            assert sha(binary) == binary_hash, 'frozen simulator changed during the run'
            receipts.append({'label': label, 'opponent': opponent, 'table': table,
                             'command': command, 'exit_code': exit_code,
                             'observed_niceness': niceness, 'wall_seconds': time.time() - start,
                             'binary_hash_basis': 'Frozen executable verified before and after execution.',
                             'binary_sha256': binary_hash})
            receipts_path.write_text(json.dumps(receipts, indent=2) + '\n')
            print(f'END {label}: exit {exit_code}, niceness {niceness}, {time.time()-start:.1f}s', flush=True)
            if exit_code:
                raise SystemExit(exit_code)


if __name__ == '__main__':
    main()
