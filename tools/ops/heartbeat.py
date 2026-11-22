#!/usr/bin/env python3
"""Text progress bars for every outstanding Codex task, narrow enough for a phone.

Stages: queued 0% -> running (elapsed vs. the typical run time for its effort, capped at 80%)
-> finished, awaiting review 85% -> merged to master 100% (shown for 12 hours).
No model needed: the heartbeat cron only pastes this output.
"""
import datetime as dt, glob, os, re, statistics, subprocess

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
Q = os.path.join(ROOT, ".progress/astra-queue")
LOG = os.path.join(ROOT, ".progress/astra.log")
NOW = dt.datetime.now()
W = 10  # bar width

def git(*a):
    r = subprocess.run(["git", "-C", ROOT, *a], capture_output=True, text=True)
    if r.returncode and "license" in r.stderr:  # Xcode updated, license not yet accepted
        env = {**os.environ, "DEVELOPER_DIR": "/Library/Developer/CommandLineTools"}
        r = subprocess.run(["git", "-C", ROOT, *a], capture_output=True, text=True, env=env)
    return r.stdout.strip()

def header(path, key):
    try:
        for line in open(path):
            if line.startswith(key + ":"):
                return line.split(":", 1)[1].strip()
            if not line.strip():
                break
    except OSError:
        pass
    return ""

# Start and finish times per task, and typical durations per effort, from the runner log.
started, finished, effort_of, durations = {}, {}, {}, {}
for line in open(LOG) if os.path.exists(LOG) else []:
    m = re.match(r"(\S+) \| (started|finished) task: (\S+)(?: \((?:write|review), \S+ (\w+)\)| \(exit (\d+)\))?", line)
    if not m:
        continue
    t = dt.datetime.fromisoformat(m.group(1)); name = m.group(3)
    if m.group(2) == "started":
        started[name] = t; effort_of[name] = m.group(4) or effort_of.get(name, "medium")
    else:
        finished[name] = (t, m.group(5))
        if name in started and m.group(5) == "0":
            durations.setdefault(effort_of.get(name, "medium"), []).append((t - started[name]).total_seconds() / 60)
typical = {e: statistics.median(v) for e, v in durations.items() if len(v) >= 2}
default = {"high": 180, "medium": 120, "low": 60}

retired = {l.strip() for l in open(os.path.join(ROOT, ".progress/heartbeat-retired.txt"))
           if l.strip() and not l.startswith("#")} if os.path.exists(os.path.join(ROOT, ".progress/heartbeat-retired.txt")) else set()
merged = set(git("branch", "--merged", "master", "--list", "astra/*").replace("*", "").split())

def bar(p):
    n = round(p / 100 * W)
    return "█" * n + "░" * (W - n)

rows = []
for f in sorted(glob.glob(f"{Q}/running/*.md")):
    name = header(f, "name"); eff = header(f, "effort") or "medium"
    t0 = started.get(name, NOW); mins = (NOW - t0).total_seconds() / 60
    exp = typical.get(eff, default.get(eff, 120))
    pct = min(80, int(mins / exp * 80)); late = mins > exp * 1.5
    rows.append((pct, name, f"{int(mins)}m{'⚠' if late else ''}"))
for f in sorted(glob.glob(f"{Q}/pending/*.md"), key=os.path.getmtime):
    rows.append((0, header(f, "name"), "queued"))
for f in sorted(glob.glob(f"{Q}/hold/*.md"), key=os.path.getmtime):
    rows.append((0, header(f, "name"), "⏸ held"))
for f in glob.glob(f"{Q}/done/*.md"):
    name = header(f, "name")
    if not name or name not in finished or name in retired:
        continue
    t, code = finished[name]
    if f"astra/{name}" in merged:
        if (NOW - t).total_seconds() < 12 * 3600:
            rows.append((100, name, "merged"))
    elif (NOW - t).total_seconds() < 2 * 86400 and header(f, "mode") == "write":
        rows.append((85, name, "review") if code in ("0", None) else (0, name, f"✗ failed ({code})"))

rows.sort(key=lambda r: -r[0])
# Deltas against the previous heartbeat (state kept in .progress/heartbeat-last.json).
import json
state_path = os.path.join(ROOT, ".progress/heartbeat-last.json")
try:
    prev = json.load(open(state_path))
except (OSError, ValueError):
    prev = {}
prev_tasks = prev.get("tasks", {})
done = sum(1 for r in rows if r[0] == 100); total = len(rows) or 1
overall = int(sum(r[0] for r in rows) / total)
since = f" since {prev['at'][11:16]}" if prev.get("at") else ""

def delta(cur, before):
    if before is None:
        return " new"
    d = cur - before
    return f"{d:+4d}" if d else "    "

print(f"Heartbeat {NOW:%H:%M} · {len(rows)} tasks · {overall}% ({delta(overall, prev.get('overall')).strip() or '±0'}{since})")
print(f"{bar(overall)} overall")
for pct, name, note in rows:
    short = name if len(name) <= 16 else name[:15] + "…"
    print(f"{bar(pct)}{pct:>4}%{delta(pct, prev_tasks.get(name))} {short:<16} {note}")
gone = [n for n in prev_tasks if n not in {r[1] for r in rows}]
if gone:
    print("dropped off: " + ", ".join(gone))
running = sum(1 for r in rows if 0 < r[0] <= 80)
failed = sum(1 for r in rows if r[2].startswith("✗"))
held = sum(1 for r in rows if r[2] == "⏸ held")
print(f"running {running} · held {held} · failed {failed} · queued {sum(1 for r in rows if r[2] == 'queued')} · review {sum(1 for r in rows if r[0] == 85)} · merged {done}")
if os.environ.get("HEARTBEAT_NO_SAVE") != "1":
    json.dump({"at": NOW.isoformat(), "overall": overall, "tasks": {r[1]: r[0] for r in rows}}, open(state_path, "w"))
