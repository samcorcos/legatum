#!/bin/bash
# Power the build box off when nothing has used it for a while. Runs as root every minute (systemd timer
# legatum-idle-watch). Shutting down STOPS the instance (instance-initiated shutdown behaviour = stop):
# the disks and the warm caches stay, only EBS is billed.
#
# Busy = a registered job is alive (/data/jobs/running/*.pid), an ssh or Session Manager session is open,
# or the 1-minute load is above 1.5. The Lambda in infra/cloud-box/cap.tf is the independent net under this.
set -u
IDLE_MINUTES="$(cat /etc/legatum-idle-minutes 2>/dev/null || echo 20)"
STATE=/run/legatum-idle
mkdir -p "$STATE"

busy=""
for f in /data/jobs/running/*.pid; do
  [ -f "$f" ] || continue
  if kill -0 "$(cat "$f")" 2>/dev/null; then busy="job $(basename "$f" .pid)"; break; fi
done
if [ -z "$busy" ] && pgrep -f 'sshd: [a-z_-]+@' >/dev/null 2>&1; then busy="ssh session"; fi
if [ -z "$busy" ] && pgrep -f 'ssm-session-worker' >/dev/null 2>&1; then busy="ssm session"; fi
if [ -z "$busy" ] && awk '{exit !($1 > 1.5)}' /proc/loadavg; then busy="load $(cut -d' ' -f1 /proc/loadavg)"; fi

now="$(date +%s)"
if [ -n "$busy" ]; then
  rm -f "$STATE/since"
  echo "$busy" > "$STATE/busy"
  exit 0
fi
rm -f "$STATE/busy"
[ -f "$STATE/since" ] || echo "$now" > "$STATE/since"
since="$(cat "$STATE/since")"
if [ $(( now - since )) -ge $(( IDLE_MINUTES * 60 )) ]; then
  logger -t legatum-idle-watch "idle for $IDLE_MINUTES minutes: powering off (stops the instance)"
  systemctl poweroff
fi
