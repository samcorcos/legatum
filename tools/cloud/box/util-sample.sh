#!/bin/bash
# Root, once a minute (systemd timer legatum-util, installed by install-util.sh): append "epoch busy% filler%" to
# /data/util/cpu.tsv. busy% is all CPU use (/proc/stat), filler% the part used by legatum-filler.slice (the idle filler,
# tools/cloud/box/box-filler). `box-job util` and `box.sh status` average it per hour. It also leaves the filler's
# current core count in /run/legatum-filler-cores, which tools/astra/verify-branch.sh subtracts from the load average
# it sizes Playwright workers by (low-priority filler work must not make a real verify think the box is busy).
set -u
S=/data/util; mkdir -p "$S"
read -r _ u n s idle iow irq sirq steal _ < /proc/stat
total=$((u + n + s + idle + iow + irq + sirq + steal)); busy=$((total - idle - iow))
fu="$(awk '/^usage_usec/{print $2}' /sys/fs/cgroup/legatum.slice/legatum-filler.slice/cpu.stat 2>/dev/null)"; fu="${fu:-0}"
now="$(date +%s)"
if [ -f "$S/last" ]; then
  read -r t0 total0 busy0 fu0 < "$S/last"
  dt=$((now - t0)); dtot=$((total - total0))
  if [ "$dt" -gt 0 ] && [ "$dtot" -gt 0 ]; then
    ncpu="$(nproc)"
    fdelta=$((fu - fu0)); [ "$fdelta" -ge 0 ] || fdelta="$fu"   # the slice vanished and came back: count what it has now
    pct=$((100 * (busy - busy0) / dtot))
    fpct=$((100 * fdelta / (dt * 1000000 * ncpu)))
    printf '%s\t%s\t%s\n' "$now" "$pct" "$fpct" >> "$S/cpu.tsv"
    echo $((fdelta / (dt * 1000000))) > /run/legatum-filler-cores
  fi
fi
echo "$now $total $busy $fu" > "$S/last"
[ "$(wc -l < "$S/cpu.tsv" 2>/dev/null || echo 0)" -le 30000 ] || { tail -n 20000 "$S/cpu.tsv" > "$S/cpu.tsv.new" && mv "$S/cpu.tsv.new" "$S/cpu.tsv"; }
exit 0
