#!/bin/bash
# Install the game-analysis job on the developer's Mac (tools/games/README.md). Idempotent; run from the repo root.
#   tools/games/install-mac.sh            install or refresh the launchd job (every 15 minutes)
#   tools/games/install-mac.sh --token    also make a new read token: stores it in the keychain and as the
#                                         Worker secret READ_TOKEN (needs CLOUDFLARE_API_TOKEN for wrangler)
#   tools/games/install-mac.sh --remove   stop and remove the job (the ledger and reports stay)
set -euo pipefail
label=io.legatum.game-analysis
plist="$HOME/Library/LaunchAgents/$label.plist"
home="$HOME/LegatumData/game-analysis"
root="$(cd "$(dirname "$0")/../.." && pwd)"

if [ "${1:-}" = --remove ]; then
  launchctl bootout "gui/$(id -u)/$label" 2>/dev/null || true
  rm -f "$plist"
  echo "removed $label (kept $home)"
  exit 0
fi

mkdir -p "$home/logs" "$home/reports" "$HOME/Library/LaunchAgents"

if [ "${1:-}" = --token ]; then
  token="$(openssl rand -hex 32)"
  (cd "$root/infra/games" && npm ci --silent && printf '%s' "$token" | npx wrangler secret put READ_TOKEN)
  # Through security's stdin, not its arguments, so the token never shows in the process list.
  printf 'add-generic-password -U -s legatum-games-read -a %s -w %s\n' "$USER" "$token" | security -i
  echo "read token stored in the keychain (legatum-games-read) and on the Worker"
fi

# The job runs a copy of the script, so a branch checkout in the repo can't break it.
mkdir -p "$home/bin"
cp "$root/tools/games/analyse_games.py" "$root/tools/games/review_prompt.md" "$home/bin/"

cat > "$plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key><string>$label</string>
  <key>ProgramArguments</key>
  <array>
    <string>/usr/bin/python3</string>
    <string>$home/bin/analyse_games.py</string>
    <string>poll</string>
  </array>
  <key>StartInterval</key><integer>900</integer>
  <key>RunAtLoad</key><true/>
  <key>LowPriorityIO</key><true/>
  <key>Nice</key><integer>10</integer>
  <key>EnvironmentVariables</key>
  <dict>
    <key>PATH</key><string>$HOME/.local/bin:$HOME/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin</string>
  </dict>
  <key>StandardOutPath</key><string>$home/logs/launchd.out</string>
  <key>StandardErrorPath</key><string>$home/logs/launchd.err</string>
</dict>
</plist>
EOF

launchctl bootout "gui/$(id -u)/$label" 2>/dev/null || true
launchctl bootstrap "gui/$(id -u)" "$plist"
echo "installed $label: every 15 minutes, reports in $home/reports (INDEX.md)"
python3 "$home/bin/analyse_games.py" poll --dry-run || true
