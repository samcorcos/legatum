#!/usr/bin/env python3
"""Self-test for pw-summary.py: error classes, the transport-only rule that gates the single rerun, the failed-file list."""
import json
import os
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
TOOL = os.path.join(HERE, 'pw-summary.py')


def report(*tests):
    suites = {}
    for file, title, status, message in tests:
        result = {'duration': 1000, 'errors': [{'message': message}] if message else []}
        suites.setdefault(file, []).append({'title': title, 'file': file, 'tests': [{'status': status, 'results': [result]}]})
    return {'stats': {'expected': sum(1 for t in tests if t[2] == 'expected'), 'unexpected': sum(1 for t in tests if t[2] == 'unexpected'),
                      'flaky': 0, 'skipped': 0, 'duration': 5000},
            'suites': [{'file': f, 'specs': s, 'suites': []} for f, s in suites.items()]}


def run(rep, *args):
    with tempfile.NamedTemporaryFile('w', suffix='.json', delete=False) as f:
        json.dump(rep, f)
    try:
        p = subprocess.run([sys.executable, TOOL, f.name, *args], capture_output=True, text=True)
    finally:
        os.unlink(f.name)
    return p.returncode, p.stdout


TRANSPORT = 'Error: page.goto: net::ERR_NETWORK_CHANGED at http://localhost:1/'
ASSERT = 'Error: expect(locator).toBeVisible() failed\n\nExpected: visible'
TIMEOUT = 'Test timeout of 90000ms exceeded.'
CONSOLE = 'Error: No browser console errors or uncaught exceptions\n- console: Failed to load resource: net::ERR_NETWORK_CHANGED (http://x)'

only_transport = report(('a.spec.ts', 't1', 'unexpected', TRANSPORT), ('b.spec.ts', 't2', 'expected', ''), ('c.spec.ts', 't3', 'unexpected', CONSOLE))
rc, out = run(only_transport, '--transport-only')
assert rc == 0, 'transport-only failures may be rerun once'
rc, out = run(only_transport)
assert 'transport=2' in out, out
rc, out = run(only_transport, '--failed-files')
assert out.split() == ['a.spec.ts', 'c.spec.ts'], out

mixed = report(('a.spec.ts', 't1', 'unexpected', TRANSPORT), ('b.spec.ts', 't2', 'unexpected', ASSERT))
rc, _ = run(mixed, '--transport-only')
assert rc == 1, 'an assertion failure beside a transport error must never be rerun'
rc, out = run(mixed)
assert 'assertion=1' in out and 'transport=1' in out, out

timeout = report(('a.spec.ts', 't1', 'unexpected', TIMEOUT))
assert run(timeout, '--transport-only')[0] == 1, 'timeouts are not transport errors'
assert 'timeout=1' in run(timeout)[1]

assert run(report(('a.spec.ts', 't1', 'expected', '')), '--transport-only')[0] == 1, 'nothing failed: nothing to rerun'
assert run({'suites': []}, '--transport-only')[0] == 1
print('pw-summary self-test passed')
