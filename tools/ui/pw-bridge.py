#!/usr/bin/env python3
"""Host-side, serialized Playwright worker for sandboxed Codex tasks."""

import argparse
import fcntl
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import signal
import subprocess
import time

REPO = Path(__file__).resolve().parents[2]
MAIN = Path(os.environ["LEGATUM_MAIN_REPO"]) if os.environ.get("LEGATUM_MAIN_REPO") else Path(
    subprocess.run(["git", "-C", str(REPO), "rev-parse", "--path-format=absolute", "--git-common-dir"],
                   check=True, capture_output=True, text=True).stdout.strip()).parent
WORKTREES = MAIN / ".claude/worktrees"
QUEUE = Path(os.environ.get("LEGATUM_PW_BRIDGE_DIR", str(MAIN / ".progress/pw-bridge")))
SPEC = re.compile(r"app/e2e/[A-Za-z0-9_.-]+\.spec\.ts\Z")
ID = re.compile(r"[A-Za-z0-9_-]+\Z")
TIMEOUT = 30 * 60


def validate(request, filename):
    if not isinstance(request, dict) or not ID.fullmatch(filename):
        raise ValueError("invalid request filename or JSON object")
    if request.get("id") != filename:
        raise ValueError("request id must match its filename")
    raw = request.get("worktree")
    if not isinstance(raw, str) or not Path(raw).is_absolute():
        raise ValueError("worktree must be an absolute path")
    try:
        worktree = Path(raw).resolve(strict=True)
    except (OSError, RuntimeError) as exc:
        raise ValueError(f"worktree does not exist: {exc}") from exc
    if worktree != MAIN and WORKTREES not in worktree.parents:
        raise ValueError("worktree is outside the allowed repository paths")
    if not (worktree / "app").is_dir():
        raise ValueError("worktree has no app directory")
    specs = request.get("specs")
    e2e = (worktree / "app/e2e").resolve()
    if not isinstance(specs, list) or not specs or any(
        not isinstance(spec, str) or not SPEC.fullmatch(spec)
        or not (worktree / spec).is_file()
        or (worktree / spec).resolve().parent != e2e
        for spec in specs
    ):
        raise ValueError("specs must be existing app/e2e/*.spec.ts files")
    grep = request.get("grep")
    project = request.get("project")
    if grep is not None and (not isinstance(grep, str) or not grep or len(grep) > 4096):
        raise ValueError("grep must be a nonempty string of at most 4096 characters")
    if project is not None and project not in ("game",):
        raise ValueError("project must be game")
    return worktree, specs, grep, project


def run(command, cwd, log, deadline, env=None):
    remaining = deadline - time.monotonic()
    if remaining <= 0:
        raise TimeoutError("request exceeded 30 minutes")
    log.write(f"\n$ {' '.join(command)}\n")
    log.flush()
    process = subprocess.Popen(
        command, cwd=cwd, stdout=log, stderr=subprocess.STDOUT, env=env,
        start_new_session=True, preexec_fn=lambda: os.nice(15),
    )
    try:
        return process.wait(timeout=remaining)
    except subprocess.TimeoutExpired as exc:
        os.killpg(process.pid, signal.SIGTERM)
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGKILL)
            process.wait()
        raise TimeoutError("request exceeded 30 minutes") from exc


def newer_crate(worktree, built):
    if not built.exists():
        return True
    built_at = built.stat().st_mtime_ns
    return any(path.stat().st_mtime_ns > built_at for path in (worktree / "crates").rglob("*") if path.is_file())


def parse_report(path):
    report = json.loads(path.read_text())
    passed = failed = 0
    failures = []

    def visit(suite, parents):
        nonlocal passed, failed
        prefix = parents + ([suite["title"]] if suite.get("title") else [])
        for spec in suite.get("specs", []):
            for test in spec.get("tests", []):
                outcome = test.get("outcome")
                if outcome == "unexpected":
                    failed += 1
                    errors = [e.get("message", "") for result in test.get("results", []) for e in result.get("errors", [])]
                    failures.append({"title": " › ".join(prefix + [spec.get("title", "")]), "error": "\n".join(errors)[:2000]})
                elif outcome in ("expected", "flaky"):
                    passed += 1
        for child in suite.get("suites", []):
            visit(child, prefix)

    for suite in report.get("suites", []):
        visit(suite, [])
    return passed, failed, failures


def handle(request_file):
    request_id = request_file.stem
    if not ID.fullmatch(request_id):
        request_id = "invalid_" + hashlib.sha256(request_file.name.encode()).hexdigest()[:16]
    result = QUEUE / "results" / request_id
    # IDs are unique. Never replace an earlier result with a later request.
    if result.exists():
        request_file.rename(QUEUE / "done" / f"duplicate_{time.time_ns()}_{request_file.name}")
        return
    result.mkdir()
    summary = {"passed": 0, "failed": 0, "exit_code": 1, "failing_tests": []}
    report_file = result / "playwright.json"
    worktree = None
    played = False
    with (result / "output.log").open("w") as log:
        try:
            if request_file.stat().st_size > 65536:
                raise ValueError("request JSON exceeds 64 KiB")
            worktree, specs, grep, project = validate(json.loads(request_file.read_text()), request_file.stem)
            app = worktree / "app"
            deadline = time.monotonic() + TIMEOUT
            modules = app / "node_modules"
            if not modules.exists():
                if modules.is_symlink():
                    raise ValueError("app/node_modules is a broken symlink")
                modules.symlink_to(MAIN / "app/node_modules", target_is_directory=True)
            if not (modules / ".bin/playwright").exists():
                raise ValueError("app/node_modules has no Playwright installation")
            if newer_crate(worktree, app / "src/wasm-pkg/legatum_bg.wasm"):
                code = run([str(worktree / "scripts/build-wasm.sh")], worktree, log, deadline)
                if code:
                    raise RuntimeError(f"WASM build exited {code}")
            if "app/e2e/staging.spec.ts" in specs:
                web_env = {**os.environ, "LEGATUM_RELAY_URL": "http://127.0.0.1:8787"}
                code = run([str(worktree / "scripts/build-web.sh"), "--skip-wasm"], worktree, log, deadline, env=web_env)
                if code:
                    raise RuntimeError(f"web build exited {code}")
            command = ["npx", "playwright", "test", *specs]
            if grep is not None:
                command += ["-g", grep]
            if project is not None:
                command += ["--project", project]
            command += ["--reporter=line,json"]
            env = os.environ.copy()
            env["PLAYWRIGHT_JSON_OUTPUT_FILE"] = str(report_file)
            played = True
            summary["exit_code"] = run(command, app, log, deadline, env)
            if report_file.exists():
                summary["passed"], summary["failed"], summary["failing_tests"] = parse_report(report_file)
            elif summary["exit_code"] == 0:
                raise RuntimeError("Playwright produced no JSON report")
            if summary["exit_code"] and not summary["failing_tests"]:
                summary["error"] = "Playwright failed before reporting a test; see output.log"
        except (Exception, KeyboardInterrupt) as exc:
            summary["exit_code"] = 124 if isinstance(exc, TimeoutError) else 1
            summary["error"] = str(exc)
            log.write(f"\nBridge error: {exc}\n")
            log.flush()
        finally:
            if played:
                source = worktree / "app/test-results"
                if source.is_dir():
                    for shot in source.rglob("*.png"):
                        dest = result / "shots" / shot.relative_to(source)
                        dest.parent.mkdir(parents=True, exist_ok=True)
                        try:
                            shutil.copy2(shot, dest)
                        except OSError as exc:
                            log.write(f"Could not copy screenshot {shot}: {exc}\n")
            summary["failing_test_titles"] = [failure["title"] for failure in summary["failing_tests"]]
            (result / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
            request_file.rename(QUEUE / "done" / request_file.name)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--once", action="store_true", help="process one queued request, then exit")
    args = parser.parse_args()
    for name in ("requests", "results", "done"):
        (QUEUE / name).mkdir(parents=True, exist_ok=True)
    with (QUEUE / "daemon.lock").open("w") as lock:
        try:
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            raise SystemExit("another Playwright bridge daemon is running")
        while True:
            pending = sorted((QUEUE / "requests").glob("*.json"), key=lambda p: (p.stat().st_mtime_ns, p.name))
            if pending:
                handle(pending[0])
            if args.once:
                break
            if not pending:
                time.sleep(2)


if __name__ == "__main__":
    main()
