#!/usr/bin/env python3
"""Summarise a Playwright JSON report: counts, slowest specs, and failures grouped by error class.

    tools/ops/pw-summary.py REPORT.json [--workers N] [--classes] [--transport-only]

Output (stdout): one "pw-summary:" line of counts, then one line per failure class. Exit 0 always, except:
  --transport-only  exit 0 when the report has failures and EVERY failure is a browser transport error, 1 otherwise
                    (the verifier uses this to decide whether a single bounded rerun of the failed shard is allowed).
  --failed-files    print the spec files that have failing tests, one per line, and nothing else.

Error classes: transport (the browser lost its connection: net::ERR_NETWORK_CHANGED, ERR_CONNECTION_*, ERR_NAME_*,
"Target page, context or browser has been closed", "Browser has been closed"), timeout ("Test timeout of" / "Timeout
Nms exceeded"), assertion (expect(...) failed), other. Only "transport" is ever eligible for a rerun.
"""
import collections
import json
import re
import sys

TRANSPORT = re.compile(
    r'net::ERR_(NETWORK_CHANGED|CONNECTION_(RESET|REFUSED|CLOSED|ABORTED|TIMED_OUT)|INTERNET_DISCONNECTED|NAME_NOT_RESOLVED|EMPTY_RESPONSE)'
    r'|NS_ERROR_(NET|CONNECTION)\w*|Target (page, context or )?browser has been closed|browser has been closed|Browser closed|Target closed')
ASSERT = re.compile(r'expect\(|toBe|toEqual|toHave|toContain|toMatch|AssertionError|Expected|Received', re.I)
TIMEOUT = re.compile(r'Test timeout of|Timeout \d+ms exceeded|Timed out \d+ms|exceeded while')


def classify(text):
    if TRANSPORT.search(text):
        return 'transport'
    if TIMEOUT.search(text) and not ASSERT.search(text.split('Timeout')[0]):
        return 'timeout'
    if ASSERT.search(text):
        return 'assertion'
    return 'other'


def walk(suite, file, out):
    file = suite.get('file') or file
    for spec in suite.get('specs', []):
        f = spec.get('file') or file
        for t in spec.get('tests', []):
            out.append((f, spec.get('title', ''), t))
    for s in suite.get('suites', []):
        walk(s, file, out)


def load(path):
    data = json.load(open(path))
    tests = []
    for s in data.get('suites', []):
        walk(s, '', tests)
    return data, tests


def failures(tests):
    """(file, title, class, first line of the error) for every test whose final status is unexpected."""
    out = []
    for f, title, t in tests:
        if t.get('status') != 'unexpected':
            continue
        results = t.get('results', [])
        last = results[-1] if results else {}
        err = ''
        for e in last.get('errors', []) or ([last['error']] if last.get('error') else []):
            err += (e.get('message') or '') + '\n'
        out.append((f, title, classify(err), err.strip().splitlines()[0][:160] if err.strip() else '(no message)'))
    return out


def main(argv):
    path = argv[1]
    workers = ''
    if '--workers' in argv:
        workers = argv[argv.index('--workers') + 1]
    try:
        data, tests = load(path)
    except (OSError, ValueError) as e:
        print(f'pw-summary: cannot read {path}: {e}')
        return 1 if '--transport-only' in argv else 0
    fails = failures(tests)
    if '--failed-files' in argv:
        for f in sorted({x[0] for x in fails}):
            print(f)
        return 0
    if '--transport-only' in argv:
        return 0 if fails and all(c == 'transport' for _, _, c, _ in fails) else 1
    stats = data.get('stats', {})
    files = collections.Counter()
    shots = 0
    for f, _, t in tests:
        files[f] += sum(r.get('duration', 0) for r in t.get('results', []))
    print('pw-summary: expected=%s unexpected=%s flaky=%s skipped=%s wall=%.0fs workers=%s tests=%d' % (
        stats.get('expected', 0), stats.get('unexpected', 0), stats.get('flaky', 0), stats.get('skipped', 0),
        stats.get('duration', 0) / 1000, workers or '?', len(tests)))
    by = collections.Counter(c for _, _, c, _ in fails)
    flaky = [(f, title) for f, title, t in tests if t.get('status') == 'flaky']
    if fails:
        print('pw-summary: failure classes: ' + ', '.join(f'{k}={v}' for k, v in sorted(by.items())))
        for f, title, c, first in fails[:40]:
            print(f'  [{c}] {f}: {title[:80]} :: {first}')
    if flaky:
        print(f'pw-summary: flaky (passed on retry): {len(flaky)}')
    for f, d in files.most_common(5):
        print(f'  slowest spec {f}: {d / 1000:.0f}s summed test time')
    return 0


sys.exit(main(sys.argv))
