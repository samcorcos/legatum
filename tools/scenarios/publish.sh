#!/usr/bin/env bash
# Drop a Scenario Lab scenario into the installed app: no build, no install. See publish.py or scenarios/README.md.
#   tools/scenarios/publish.sh scenarios/lab/ID.json [--rules-version N] [--dry-run]
exec python3 "$(cd "$(dirname "$0")" && pwd)/publish.py" "$@"
