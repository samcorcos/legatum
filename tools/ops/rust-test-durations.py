#!/usr/bin/env python3
"""Per-test-binary durations from cargo-nextest output (a verify's steps/test.log).

    tools/ops/rust-test-durations.py LOG [LOG...] [--top N] [--tsv]

Nextest prints "PASS [   0.011s] (   1/1877) <binary id> <test name>". The binary id is the crate (unit tests) or
"crate::test_file" (integration tests). Output: binary, test count, summed seconds, longest single test, sorted by
summed seconds. Summed time is CPU-ish work, the number that matters for how long a gate takes on a busy machine;
the longest test is the floor on wall time. With several logs the per-binary values are the median across logs.
"""
import re
import statistics
import sys

LINE = re.compile(r'^\s*(?:PASS|FAIL|SLOW|LEAK|SIGSEGV|TIMEOUT)\s+(?:\[\s*([\d.]+)s\])\s+\(\s*\d+/\d+\)\s+(\S+)\s+(\S+)')


def parse(path):
    bins = {}
    for line in open(path, errors='replace'):
        m = LINE.match(line)
        if not m or not line.lstrip().startswith(('PASS', 'FAIL')):
            continue
        d, b, _ = float(m.group(1)), m.group(2), m.group(3)
        n, s, mx = bins.get(b, (0, 0.0, 0.0))
        bins[b] = (n + 1, s + d, max(mx, d))
    return bins


def main(argv):
    paths = [a for a in argv[1:] if not a.startswith('--') and not a.isdigit()]
    top = int(argv[argv.index('--top') + 1]) if '--top' in argv else 40
    runs = [parse(p) for p in paths]
    names = sorted({b for r in runs for b in r})
    rows = []
    for b in names:
        have = [r[b] for r in runs if b in r]
        rows.append((b, int(statistics.median(h[0] for h in have)), statistics.median(h[1] for h in have), statistics.median(h[2] for h in have)))
    rows.sort(key=lambda r: -r[2])
    total = sum(r[2] for r in rows)
    if '--tsv' in argv:
        print('binary\ttests\tsum_s\tmax_s')
        for r in rows:
            print('%s\t%d\t%.1f\t%.1f' % r)
        return 0
    print('%-52s %6s %9s %8s' % ('binary', 'tests', 'sum s', 'max s'))
    acc = 0
    for r in rows[:top]:
        acc += r[2]
        print('%-52s %6d %9.1f %8.1f   cum %3.0f%%' % (r[0], r[1], r[2], r[3], 100 * acc / total))
    print('total: %d binaries, %d tests, %.0f s summed' % (len(rows), sum(r[1] for r in rows), total))
    return 0


sys.exit(main(sys.argv))
