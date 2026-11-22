#!/usr/bin/env bash
# PreToolUse(Bash): the pre-merge gates. Before a branch merges into master it must pass the static checks and carry
# the reviewer markers its changed paths call for (designer, 2026-11-07: docs always up to date; static reviewers only,
# no simulations). The checks and markers live in tools/gates/check.py; run `tools/gates/check.py <branch>` to see
# the same verdict before merging.
#
#   every branch            tools/gates/hygiene.py            secrets, personal data, saves, big binaries, dead code
#   crates/core, crates/ai  tools/gates/engine-invariants.py  determinism and integer-maths invariants, save shape
#   art assets              tools/gates/art.py                lookalike check (about 30 s, cached per art tree)
#   scenario-only branch    tools/gates/scenario.py           only scenarios/lab/*.json plus docs: the fast scenario check
#                                                             (seconds) replaces the docs review and the extended verify
#   code                    .progress/docs-review/<branch>.ok from legatum-docs-reviewer
#   player text             .progress/plain-review/<branch>.ok from legatum-plain-language-reviewer
#   UI                      .progress/ui-review/<branch>.ok from legatum-ui-reviewer
#
# Fires when a command runs `git merge <branch>` in a checkout whose current branch is master (any form:
# `git -C DIR merge`, `cd DIR && git merge`, chained or piped commands, redirections such as `2>&1`, `bash -c '...'`).
# Docs- and test-only branches need no review (logged). Emergencies: LEGATUM_GATE_BYPASS="<reason>" in the command
# or the environment skips every gate for that merge, logged to .progress/merge-gates/bypass.log.
# `git merge master` inside a worktree (current branch is not master) never fires, nor do --abort/--continue/--quit.
# Test: .claude/hooks/test-merge-gates.sh
input=$(cat)
GATES_DIR="$(cd "$(dirname "$0")/../../tools/gates" 2>/dev/null && pwd)" HOOK_INPUT="$input" exec python3 - <<'PY'
import datetime, json, os, re, subprocess, sys

try:
    data = json.loads(os.environ.get("HOOK_INPUT") or "{}")
except Exception:
    sys.exit(0)
command = (data.get("tool_input") or {}).get("command") or ""
if "merge" not in command:
    sys.exit(0)
start_cwd = data.get("cwd") or os.getcwd()

OPS = {"&&", "||", ";", "|", "&", "(", ")", "\n", ";;", "|&"}
ASSIGN = re.compile(r"^([A-Za-z_][A-Za-z0-9_]*)=(.*)$", re.S)


def git(cwd, *args):
    r = subprocess.run(["git", "-C", cwd, *args], capture_output=True, text=True)
    return r.stdout.strip() if r.returncode == 0 else None


def tokens(cmd):
    """Shell words and control operators. Quotes are honoured; redirections (`2>&1`, `>/dev/null`, `&>log`,
    `< in`) are dropped with their targets, so they never read as merge arguments or split a command at `&`."""
    out, cur, have = [], [], False
    i, n = 0, len(cmd)

    def flush():
        nonlocal cur, have
        if have:
            out.append("".join(cur))
        cur, have = [], False

    def skip_target(j):
        while j < n and cmd[j] in " \t":
            j += 1
        while j < n and cmd[j] not in " \t\n;&|()<>":
            if cmd[j] in "'\"":
                q = cmd[j]
                j = cmd.find(q, j + 1)
                j = n if j < 0 else j + 1
            else:
                j += 1
        return j

    while i < n:
        c = cmd[i]
        if c == "\\" and i + 1 < n:
            cur.append(cmd[i + 1]); have = True; i += 2
        elif c == "'":
            j = cmd.find("'", i + 1)
            j = n if j < 0 else j
            cur.append(cmd[i + 1:j]); have = True; i = j + 1
        elif c == '"':
            j, buf = i + 1, []
            while j < n and cmd[j] != '"':
                if cmd[j] == "\\" and j + 1 < n and cmd[j + 1] in '"\\$`':
                    buf.append(cmd[j + 1]); j += 2
                else:
                    buf.append(cmd[j]); j += 1
            cur.append("".join(buf)); have = True; i = j + 1
        elif c in "<>" or (c == "&" and cmd[i + 1:i + 2] == ">"):
            if have and "".join(cur).isdigit():
                cur, have = [], False  # the fd number of `2>`
            flush()
            m = re.match(r"&>>?|\d*>>?&?|>\||<&?|<<<?|<>", cmd[i:])
            j = i + (len(m.group(0)) if m else 1)
            i = skip_target(j)
        elif c in " \t\r":
            flush(); i += 1
        elif c == "#" and not have:
            while i < n and cmd[i] != "\n":
                i += 1
        else:
            op = next((o for o in ("&&", "||", ";;", "|&", ";", "|", "&", "(", ")", "\n") if cmd.startswith(o, i)), None)
            if op:
                flush(); out.append(op); i += len(op)
            else:
                cur.append(c); have = True; i += 1
    flush()
    return out


def segments(cmd):
    seg = []
    for t in tokens(cmd):
        if t in OPS:
            if seg:
                yield seg
            seg = []
        else:
            seg.append(t)
    if seg:
        yield seg


def expand(s, env):
    def rep(m):
        name = m.group(1) or m.group(2)
        return env.get(name, os.environ.get(name, m.group(0)))
    return re.sub(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}|\$([A-Za-z_][A-Za-z0-9_]*)", rep, s)


merges = []  # (dir, [commits], bypass_reason)


def scan(cmd, cwd, env):
    for seg in segments(cmd):
        local = {}
        i = 0
        while i < len(seg) and ASSIGN.match(seg[i]):
            k, v = ASSIGN.match(seg[i]).groups()
            local[k] = expand(v, env)
            i += 1
        words = [expand(w, {**env, **local}) for w in seg[i:]]
        if not words:
            env.update(local)  # `B=foo; git merge $B`
            continue
        while words and words[0] in ("env", "command", "nice", "time", "exec", "sudo"):
            words = words[1:]
            while words and (ASSIGN.match(words[0]) or words[0].startswith("-")):
                m = ASSIGN.match(words[0])
                if m:
                    local[m.group(1)] = m.group(2)
                words = words[1:]
        if not words:
            continue
        w0 = os.path.basename(words[0])
        if w0 == "cd" and len(words) > 1:
            cwd = os.path.normpath(os.path.join(cwd, os.path.expanduser(words[1])))
        elif w0 == "export":
            for w in words[1:]:
                m = ASSIGN.match(w)
                if m:
                    env[m.group(1)] = m.group(2)
        elif w0 in ("bash", "sh", "zsh") and "-c" in words:
            j = words.index("-c")
            if j + 1 < len(words):
                scan(words[j + 1], cwd, dict(env, **local))
        elif w0 == "git":
            d = cwd
            j = 1
            while j < len(words) and words[j].startswith("-"):
                if words[j] == "-C" and j + 1 < len(words):
                    d = os.path.normpath(os.path.join(d, os.path.expanduser(words[j + 1])))
                    j += 2
                elif words[j] in ("-c", "--git-dir", "--work-tree", "--namespace") and j + 1 < len(words):
                    j += 2
                else:
                    j += 1
            if j < len(words) and words[j] == "merge":
                args = words[j + 1:]
                if any(a in ("--abort", "--continue", "--quit") for a in args):
                    continue
                commits = []
                k = 0
                while k < len(args):
                    a = args[k]
                    if a in ("-m", "-F", "-s", "-X", "--file", "--strategy", "--strategy-option", "--into-name", "--cleanup"):
                        k += 2
                        continue
                    if a == "--":
                        commits += args[k + 1:]
                        break
                    if not a.startswith("-"):
                        commits.append(a)
                    k += 1
                reason = local.get("LEGATUM_GATE_BYPASS") or env.get("LEGATUM_GATE_BYPASS") or os.environ.get("LEGATUM_GATE_BYPASS", "")
                merges.append((d, commits, reason.strip()))


scan(command, start_cwd, {})
if not merges:
    sys.exit(0)


def deny(msg):
    print(json.dumps({"hookSpecificOutput": {"hookEventName": "PreToolUse", "permissionDecision": "deny",
                                             "permissionDecisionReason": msg}}))
    sys.exit(0)


def log(main, line):
    d = os.path.join(main, ".progress", "merge-gates")
    os.makedirs(d, exist_ok=True)
    stamp = datetime.datetime.now().astimezone().isoformat(timespec="seconds")
    with open(os.path.join(d, "bypass.log"), "a") as f:
        f.write(f"{stamp}\t{line}\n")


gates_dir = os.environ.get("GATES_DIR") or ""
if not os.path.isfile(os.path.join(gates_dir, "check.py")):
    deny(f"Merge gates: tools/gates/check.py is missing (looked in `{gates_dir}`); restore it from master, "
         "or prefix the merge with LEGATUM_GATE_BYPASS=\"<reason>\".")
sys.path.insert(0, gates_dir)
import check  # noqa: E402

problems, todo = [], set()
for d, commits, reason in merges:
    if not os.path.isdir(d):
        continue
    if git(d, "symbolic-ref", "--short", "-q", "HEAD") != "master":
        continue  # merging into a branch (e.g. `git merge master` in a worktree): not a landing
    common = git(d, "rev-parse", "--path-format=absolute", "--git-common-dir")
    main = os.path.dirname(common) if common else d
    master = git(d, "rev-parse", "refs/heads/master")
    for c in commits:
        if c in ("master", "origin/master", "refs/heads/master", "HEAD", "FETCH_HEAD", "ORIG_HEAD"):
            continue
        tip = git(d, "rev-parse", "--verify", "-q", c + "^{commit}")
        if not tip:
            problems.append(f"cannot resolve `{c}` to a commit; name the branch literally so the gates can check it")
            continue
        if tip == master or git(d, "merge-base", "--is-ancestor", tip, "refs/heads/master") is not None:
            continue  # already on master: nothing lands
        name = c[len("refs/heads/"):] if c.startswith("refs/heads/") else c
        if reason:
            log(main, f"override\t{name}\t{tip}\t{reason}")
            continue
        base = git(d, "merge-base", "refs/heads/master", tip)
        if not base:
            problems.append(f"`{name}` shares no history with master")
            continue
        checks, markers, kinds = check.gate(d, name, base, tip)
        for check_name, _, findings in checks:
            if findings:
                shown = [f.split("\n")[0] for f in findings[:6]]
                problems.append(f"`{name}` fails {check_name}: " + "; ".join(shown)
                                + (f"; and {len(findings) - 6} more" if len(findings) > 6 else ""))
                todo.add("checks")
        for kind, ok, msg in markers:
            if not ok:
                problems.append(f"`{name}` (tip {tip[:9]}) needs a {kind} review: {msg}")
                todo.add(kind)
        if not markers and not any(f for _, _, f in checks):
            log(main, f"no-code\t{name}\t{tip}\tdocs/tests only")

if problems:
    steps = []
    if "checks" in todo:
        steps.append("fix the static findings on the branch (`tools/gates/check.py <branch>` reruns every check; a "
                     "line that is right as written takes a `gate-allow: <reason>` comment on it or the line above)")
    if "ui" in todo:
        steps.append("run the `legatum-ui-reviewer` agent on the branch (it writes .progress/ui-review/"
                     "<branch-with-slashes-as-dashes>.ok when its verdict is ship or ship with known issues)")
    if "plain" in todo:
        steps.append("run the `legatum-plain-language-reviewer` agent on the branch and apply its rewrites (it writes "
                     ".progress/plain-review/<branch-with-slashes-as-dashes>.ok when the player text is plain)")
    if "docs" in todo:
        steps.append("run the `legatum-docs-reviewer` agent on the branch, last"
                     " (it fixes stale docs on the branch and writes .progress/docs-review/<branch-with-slashes-as-dashes>.ok)")
    deny("Merge gates: " + "; ".join(problems) + ". To land: " + "; then ".join(steps) + "; then merge again. "
         "Emergency only: prefix the merge with LEGATUM_GATE_BYPASS=\"<reason>\" (logged to "
         ".progress/merge-gates/bypass.log) and tell the lead.")
sys.exit(0)
PY
