#!/usr/bin/env python3
"""Run a command and append its wall and CPU time to the timing ledger.

    tools/ops/timed.py <kind> <name> [--label L] [--ref R] -- <command...>
    tools/ops/timed.py --record <kind> <name> <label> <ref> <rc> <wall_s> <cpu_s> <load1>

The second form only writes a row, for callers that time shell functions themselves.

kind groups rows (verify, codex, sim, build, release, cloud); name is the step; label is the
branch or task; ref is a commit or tree hash, so a step that both fails and passes on one
ref shows up as flaky. CPU is user+sys of the command and every descendant it waited for,
so it stays comparable under load where wall time does not.

Rows go to ~/LegatumRuns/timings/YYYY-MM-DD.tsv (not in git); files older than 30 days are
deleted. The command's exit code is passed through, and the ledger never fails the command.
"""
import datetime
import os
import resource
import signal
import subprocess
import sys
import time
from pathlib import Path

DIR = Path(os.environ.get("LEGATUM_TIMINGS_DIR", Path.home() / "LegatumRuns" / "timings"))
KEEP_DAYS = 30
COLUMNS = "ts\tkind\tname\tlabel\tref\trc\twall_s\tcpu_s\tload1\tcores\n"


def parse(argv):
    if "--" not in argv:
        sys.exit("usage: timed.py <kind> <name> [--label L] [--ref R] -- <command...>")
    i = argv.index("--")
    head, cmd = argv[:i], argv[i + 1:]
    if len(head) < 2 or not cmd:
        sys.exit("usage: timed.py <kind> <name> [--label L] [--ref R] -- <command...>")
    opts = {"label": "", "ref": ""}
    rest = head[2:]
    while rest:
        flag = rest.pop(0)
        if flag not in ("--label", "--ref") or not rest:
            sys.exit(f"timed.py: bad option {flag}")
        opts[flag[2:]] = rest.pop(0)
    return head[0], head[1], opts, cmd


def clean(s):
    return s.replace("\t", " ").replace("\n", " ")


def record(row):
    # tools/astra/test-verify-cache.sh intentionally stubs verifier steps, including a
    # failing fmt step. Those are harness checks, not measurements of box reliability.
    if os.environ.get("VERIFY_STEP_STUB") == "1":
        return
    try:
        DIR.mkdir(parents=True, exist_ok=True)
        today = datetime.date.today()
        path = DIR / f"{today.isoformat()}.tsv"
        new = not path.exists()
        with open(path, "a") as f:
            f.write((COLUMNS if new else "") + "\t".join(clean(str(c)) for c in row) + "\n")
        if new:
            cutoff = (today - datetime.timedelta(days=KEEP_DAYS)).isoformat()
            for old in DIR.glob("*.tsv"):
                if old.stem < cutoff:
                    old.unlink(missing_ok=True)
    except OSError:
        # Recording is deliberately invisible to the wrapped command's output and status.
        pass


def main():
    if sys.argv[1:2] == ["--record"]:
        if len(sys.argv) != 10:
            sys.exit("usage: timed.py --record <kind> <name> <label> <ref> <rc> <wall_s> <cpu_s> <load1>")
        record([datetime.datetime.now().isoformat(timespec="seconds"), *sys.argv[2:10], os.cpu_count()])
        return 0
    kind, name, opts, cmd = parse(sys.argv[1:])
    load1 = os.getloadavg()[0]
    start_ts = datetime.datetime.now().isoformat(timespec="seconds")
    before = resource.getrusage(resource.RUSAGE_CHILDREN)
    t0 = time.monotonic()
    try:
        child = subprocess.Popen(cmd)
    except OSError as e:
        print(f"timed.py: {e}", file=sys.stderr)
        return 127
    for sig in (signal.SIGTERM, signal.SIGINT, signal.SIGHUP):
        signal.signal(sig, lambda s, _f: child.send_signal(s))
    rc = child.wait()
    wall = time.monotonic() - t0
    after = resource.getrusage(resource.RUSAGE_CHILDREN)
    cpu = (after.ru_utime - before.ru_utime) + (after.ru_stime - before.ru_stime)
    record([start_ts, kind, name, opts["label"], opts["ref"], rc,
            f"{wall:.1f}", f"{cpu:.1f}", f"{load1:.1f}", os.cpu_count()])
    return rc if rc >= 0 else 128 - rc


if __name__ == "__main__":
    sys.exit(main())
