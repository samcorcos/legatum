#!/usr/bin/env bash
# Build the web version of this checkout and publish it as a Cloudflare Pages preview, for a playtest:
#   scripts/deploy-preview.sh <label> [--skip-wasm]      e.g. scripts/deploy-preview.sh playtest-9
# The preview lives at https://<label>.<project>.pages.dev (project: CLOUDFLARE_PAGES_PROJECT, default
# legatum-game) and never replaces the production site, which only master's deploy (deploy-web.yml) updates.
# The label is the Pages branch name: lower-case letters, digits and hyphens. It also shows on the title screen.
# Needs CLOUDFLARE_API_TOKEN and CLOUDFLARE_ACCOUNT_ID in the environment, or a logged-in `wrangler login`.
set -euo pipefail
cd "$(dirname "$0")/.."
label="${1:?usage: scripts/deploy-preview.sh <label> [--skip-wasm]}"; shift
[[ "$label" =~ ^[a-z0-9][a-z0-9-]*$ ]] || { echo "label must be lower-case letters, digits and hyphens: $label" >&2; exit 2; }
[ "$label" != master ] || { echo "master is the production site; deploy it from GitHub" >&2; exit 2; }
project="${CLOUDFLARE_PAGES_PROJECT:-legatum-game}"

# Keep the preview's earlier files, so a game open on it keeps its art after this deploy.
LEGATUM_CARRY_ASSETS_FROM="${LEGATUM_CARRY_ASSETS_FROM:-https://$label.$project.pages.dev}" \
  LEGATUM_BUILD_LABEL="${LEGATUM_BUILD_LABEL:-$label}" scripts/build-web.sh "$@"
# The same wrangler version as infra/relay.
wrangler="$(node -p "require('./infra/relay/package.json').devDependencies.wrangler")"
npx --yes "wrangler@$wrangler" pages deploy app/dist --project-name="$project" --branch="$label" \
  --commit-hash="$(git rev-parse HEAD)" --commit-dirty=true
echo "Preview: https://$label.$project.pages.dev"
