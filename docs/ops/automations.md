# Automation schedule directory

Concise directory of where recurring agent heartbeats and repository automations are configured and how to inspect, pause, or stop them.

> **Not a live status board:** GitHub issues, labels, and PRs are the coordination source of truth. This file is a directory of onboarding defaults, configured schedules, and control locations; verify live running state directly in each platform's scheduler (items not directly inspectable from git are marked *unverified from repo*).

## Rules

1. **Check the actual scheduler first** before creating a new recurring task so you do not create duplicates. If a schedule already exists for `samcorcos/legatum`, preserve its configured cadence rather than recreating or resetting it to the generic onboarding default.
2. **Update this directory** when an agent's configured cadence, scheduler location, or stop procedure changes.
3. **Stop means stop:** when the maintainer says stop, cancel any recurring schedule or active timer you control immediately and report what stopped and what must be paused in another platform's UI.
4. **Public-repo hygiene:** never record account IDs, machine names, secrets, or provider quota/budget numbers here.

## Agent heartbeat directory

Protocol: [docs/ops/multi-agent.md](multi-agent.md). Generic onboarding defaults apply only when creating a new schedule from scratch; existing maintainer-configured schedules are preserved rather than reset.

| Agent | Onboarding default | Current configured schedule | Scheduler / runtime location | How to inspect, pause, or stop |
| --- | --- | --- | --- | --- |
| `claude` | Hourly at `:05` | *Unverified from repo* (preserve existing schedule if configured) | Claude cloud scheduled tasks (`claude.ai` -> **Scheduled**, formerly Routines); cloud-hosted *(live state unverified from repo)* | Open `claude.ai` -> **Scheduled** and pause or delete the Legatum task |
| `chatgpt` | Hourly at `:25` | Every 10 minutes (`:05/:15/:25/:35/:45/:55`, maintainer-configured fallback; preserve existing cadence) | ChatGPT / Codex scheduled tasks (configured in the maintainer's ChatGPT/Codex environment; laptop-closed execution depends on host/cloud runtime, *unverified from repo*) | Open ChatGPT/Codex **Scheduled tasks** and pause or delete the Legatum task |
| `gemini` | Hourly at `:45` | *Unverified from repo* (preserve existing schedule if configured) | Antigravity desktop **Automations** (`~/.gemini/config/sidecars/`, local to **This computer** — pauses on sleep), or Antigravity **Remote Agent** `/schedule` where supported *(live state unverified from repo; plain VM shell timers are not durable across timeouts)* | Open Antigravity **Automations** to pause/delete local jobs, or reply `stop` in any active Remote Agent session |

## Repository automations (GitHub)

GitHub-hosted workflows and dependency updates live directly in their config files (do not duplicate their schedules here):

- [`.github/workflows/`](../../.github/workflows/) — CI (`ci.yml` / `ci-ok`), peer-review gate (`peer-review.yml`), web deploy (`deploy-web.yml`), nightly E2E (`e2e-nightly.yml`), temporary moderation (`moderate.yml`), and Cloudflare Workers (`games.yml`, `relay.yml`).
- [`.github/dependabot.yml`](../../.github/dependabot.yml) — scheduled dependency update PRs.
- [GitHub Actions tab](https://github.com/samcorcos/legatum/actions) — inspect live runs or disable workflows.
