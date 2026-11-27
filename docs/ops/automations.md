# Automations and recurring loops registry

Single source of truth for every recurring agent loop, scheduled job, and repository automation running on `samcorcos/legatum`.

## Rules (for all agents)

1. **Register every loop here.** Whenever you create, change, pause, or stop a recurring schedule or background wakeup loop, update this file in the same PR (or note it in your PR) so Claude, ChatGPT/Codex, Gemini, and the maintainer share one accurate picture.
2. **Track live VM runtime state in `.progress/automations.json` (gitignored).** Never commit per-tick timestamps to git. Instead, any agent running an in-session background wakeup timer writes its current armed/paused state, next due UTC timestamp, and last action to `.progress/automations.json` on every tick and reads it back after context compaction or session resume so a stopped loop is caught immediately.
3. **Never duplicate a running loop.** Check this table (and your platform's scheduler list) before creating a new recurring task.
4. **Stop means stop.** When the maintainer says stop, cancel every running background timer/schedule you control immediately and report what stopped and what (if anything) is still running elsewhere.
5. **Public-repo hygiene.** Never record account IDs, machine names, secrets, or provider quota/budget numbers here.

## Agent heartbeats

Follow the protocol in [docs/ops/multi-agent.md](multi-agent.md).

| Name | Agent | Platform & runtime | Laptop closed? | Cadence | Status | How to pause or stop |
| --- | --- | --- | --- | --- | --- | --- |
| Claude hourly heartbeat | `claude` | Claude cloud scheduled task (`claude.ai` -> Scheduled) | Yes (cloud) | Hourly at `:05` | Planned / maintainer-enabled | Pause or delete in `claude.ai` -> Scheduled |
| ChatGPT / Codex hourly heartbeat | `chatgpt` | ChatGPT/Codex scheduled task (desktop app or always-on `codex exec`) | Only on always-on host; desktop app requires awake machine | Hourly at `:25` | Planned / maintainer-enabled | Pause or delete in ChatGPT/Codex -> Scheduled tasks |
| Gemini hourly heartbeat | `gemini` | Antigravity Remote Agent cloud VM (in-thread `:45` wakeup timer, state in `.progress/automations.json`) | Yes (cloud VM; runs inside the active Remote Agent session) | Hourly at `:45` | Active (since 2026-11-27) | Reply `stop` in the active Antigravity Remote Agent thread |

## Repository and CI automations (GitHub)

| Name | Workflow / config | Trigger / cadence | Status | Purpose / notes |
| --- | --- | --- | --- | --- |
| CI + static gates (`ci-ok`) | `.github/workflows/ci.yml` | Every PR and push to `master` | Active | Required by `Protect master` ruleset (with strict up-to-date branch check) |
| Deploy web preview / prod | `.github/workflows/deploy-web.yml` | Push to `master` & manual dispatch | Active | Builds WASM + web app and deploys static site to Cloudflare Pages |
| Nightly full Playwright E2E | `.github/workflows/e2e-nightly.yml` | Daily at `09:17 UTC` (`17 9 * * *`) | Active | Runs full `app/e2e` suite against built site + relay |
| Issue & PR moderation | `.github/workflows/moderate.yml` | New outside issues, PRs, comments | Active until `2026-12-09` | Auto-closes/hides off-topic wave posts; delete workflow + `tools/moderation/moderate.py` after `MODERATION_UNTIL` |
| Game uploads Worker CI | `.github/workflows/games.yml` | Changes under `infra/games/**` | Active | Tests and deploys the game-upload Cloudflare Worker |
| Online relay Worker CI | `.github/workflows/relay.yml` | Changes under `infra/relay/**` | Active | Tests and deploys the multiplayer relay Cloudflare Worker |
| Dependency updates | `.github/dependabot.yml` | Scheduled by GitHub Dependabot | Active | Opens dependency bump PRs for `app/`, Rust crates, and Actions |
