#!/usr/bin/env bash
# Kills Playwright WebKit on the macOS host. Its screenshots go through WindowServer, which aborted on
# 2026-10-05 (twice) and 2026-10-07 (during a one-worker release-check run), logging the user out and
# killing every app. WebKit runs only in a Linux container (tools/release/webkit-docker.sh). Runs as a
# loop every 10 s; started by tools/ops/watchdog.sh and at session start. Logs to .progress/jobs/no-host-webkit.log.
set -u
log="$(cd "$(dirname "$0")/../.." && pwd)/.progress/jobs/no-host-webkit.log"
while :; do
  # pgrep -f only to FIND matching processes; each is killed by its own PID after logging it.
  for pid in $(pgrep -f "ms-playwright/webkit-[0-9]+/(Playwright.app|pw_run|com.apple.WebKit)" 2>/dev/null); do
    cmd="$(ps -p "$pid" -o command= 2>/dev/null)"
    [ -n "$cmd" ] || continue
    echo "$(date -u +%FT%TZ) killed host WebKit pid $pid: ${cmd:0:160}" >> "$log"
    kill "$pid" 2>/dev/null
  done
  sleep 10
done
