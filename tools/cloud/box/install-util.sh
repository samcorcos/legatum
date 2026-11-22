#!/bin/bash
# Root, idempotent: install the once-a-minute CPU utilisation sampler (util-sample.sh). Called by box_sync.
set -eu
[ -x /data/bin/util-sample.sh ] || exit 0
cat > /etc/systemd/system/legatum-util.service <<'U'
[Unit]
Description=Legatum build box CPU utilisation sample
[Service]
Type=oneshot
ExecStart=/data/bin/util-sample.sh
U
cat > /etc/systemd/system/legatum-util.timer <<'U'
[Unit]
Description=Legatum build box CPU utilisation sampler
[Timer]
OnBootSec=1min
OnUnitActiveSec=1min
AccuracySec=1s
[Install]
WantedBy=timers.target
U
systemctl daemon-reload
systemctl enable --now legatum-util.timer >/dev/null 2>&1
