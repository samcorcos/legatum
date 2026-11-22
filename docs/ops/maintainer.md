# Maintainer operations

Moved verbatim from `CLAUDE.md` (2026-11-22), so agents that do game work don't load it every session. It applies to the maintainer's lead sessions: running Codex/Astra, pacing budgets and routing models. The operations playbook is the `legatum-ops` skill; routing tables are in `legatum-workflow`.

## Astra (gpt-6-astra via Codex CLI)

Codex and Claude both have limited budgets (user, 2026-10-04): pace both. Offload implementation to Codex when it fits, keep its queue fed but not oversubscribed, and have Claude review the results before anything merges. Codex runs only on the user's Mac; cloud sessions reach it through Remote Control.
- `tools/astra/runner.sh queue <k>` (one per lane k=1..4, plus `runner.sh reaper`) works through `.progress/astra-queue/pending/*.md` (headers `mode: review|write`, `name:`, optional `model: astra|sol|luna` and `effort:`, blank line, prompt). Write tasks run on branch `astra/<name>`, cut from a freshly fetched `origin/master`, in the worker's persistent lane worktree (`.claude/worktrees/lane-<k>`), so builds stay incremental. The runner pushes the finished branch to GitHub; the lead opens the PR and merges it after review.
- Pick the model per task (user, 2026-10-04): only `sol` (gpt-6-sol, default: implementation, AI and design work, reviews) and `luna` (gpt-6-luna, cheapest: mechanical work such as removals, docs and running sims). `astra` (gpt-6-astra) is for max-intelligence reviews and audits (user, 2026-10-24).
- Model routing (user, 2026-10-04): pick the cheapest model that meets the quality bar for every task (table in the legatum-workflow skill). Sonnet 5.5 (half Opus's price) is the default for agent implementation work; Opus 5.5 for design judgement, novel architecture and hard debugging. Fable is allowed for diagnosis and audit agents for now (user, 2026-11-11; he may bring the ban back).
- `tools/astra/runner.sh reviewer` reviews every new `origin/master` commit range.
- Outputs land in `.reviews/astra-*.md`; progress in `.progress/astra.log`. Queue new tasks whenever the queue runs dry.
