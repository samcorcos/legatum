---
name: legatum-ops
description: This skill should be used when supervising or running Legatum's parallel work at scale (many Claude agents, Codex/Astra tasks, simulation cohorts, releases): budgeting tokens and usage limits, choosing concurrency, recovering from failures, and keeping the machine efficient. It's the operations playbook maintained by the legatum-ops-supervisor agent from real incidents.
---

# Legatum operations playbook (self-updating)

Maintained by the `legatum-ops-supervisor` agent; the lead may edit it too. Keep this file to current rules, one line each with date and who; put incidents and detail in `references/`. Delete anything proven wrong.

**Where work happens now (2026-11-22).** All coding happens in Claude cloud sessions; GitHub `samcorcos/legatum` `master` is the source of truth (branch, PR, CI via `tools/ci/local-ci.sh`, merge on GitHub). The game is web-only; playtests are Cloudflare Pages previews. Mac-side operations (Codex/Astra lanes, the battery rule, the watchdog daemon, the build box from the Mac) are maintainer-only: see `docs/ops/maintainer.md` and `references/mac-era-operations.md`.

## Principle

- Question every requirement first: question, delete, simplify, accelerate, automate. Each supervisor run names one requirement questioned and its verdict (user, 2026-10-04).
- A new standing check is added whenever we find a cost nobody was measuring (user, 2026-10-06).
- A policy change does not reach running tasks: their prompts are frozen at start. Edit pending tasks and tell the lead about running ones (2026-10-04).

## Budgets and concurrency

- Claude and Codex budgets are both limited and cannot grow; pace both, and move work to whichever is further ahead of its pace (user, 2026-10-04).
- Default to at most 4-5 concurrent Claude agents; about 12 Opus agents once exhausted the 5-hour window and every agent died mid-task (2026-10-03). Above 60% of the window stop spawning; above 80% have agents commit WIP and pause.
- Extra usage stays off; never suggest turning it on (user, 2026-10-04).
- Model choice: cheapest model that meets the bar (table in `legatum-workflow`). Fable is allowed for diagnosis and audit agents (user, 2026-11-11, CLAUDE.md; this replaces "Never use Fable", user 2026-10-04; he may bring the ban back).
- Near 90% of the main session's context, the lead writes current state to the handoff before compaction.

## Standing rules

- Verify a long job's first unit of progress within minutes of launching it (first game finished, first decision logged); do not trust the launch (2026-10-05).
- Judge a job by its exit code and a fresh status file, never by the tail of its log.
- Each agent uses its own scratch directory and its own worktree; never several agents in one worktree, and never borrow another job's `CARGO_TARGET_DIR`.
- Check ownership before re-running, requeuing or landing someone else's branch: if an open PR, a thread or an agent owns it, report instead (2026-10-10).
- A landed-check must see real commits: a branch that still equals the master it forked from is not "landed" (2026-10-10).
- Never `pkill -f` or `killall` shared patterns (a hook blocks them); kill by PID. Supervisor kills of other workloads may be denied by the classifier: report PIDs and evidence to the lead, do not retry.
- Benchmarks in task prompts must cap threads (`tools/sim/hard-vs-human.sh` or `RAYON_NUM_THREADS=4`) (2026-10-10).
- State truth (2026-11-25): every session's progress checklist must show its true state. RUNNING needs live processes and fresh output; WAITING ON THE MAINTAINER needs a decision card in the thread plus a "Waiting on the maintainer: <what>" ✱ line, posted the moment the block starts, with the exact steps needed; DONE means the result is reported; anything else is STALLED and is a bug. The ops check classifies every queue, lane and session this way and puts anything stalled or mislabelled on the first line of its report.
- After any session restart (or a context reset), re-arm every background watcher and wait loop at once (they don't survive a restart; on 2026-11-24 a dead download watcher left the anim pilot silently idle for hours) and re-post the current state to the checklist.
- Product performance (sim speed, AI thinking time, app size and load, CI and test time, flakiness, dead code) belongs to `legatum-perf-supervisor` and its nightly ledger; the ops check tracks turn size, game length, AI quality, rework and system health (user, 2026-10-06; split 2026-11-21).

## Cloud compute (AWS)

- Spot Graviton batches run sim binaries only (`tools/cloud/run-batch.sh`); always pass `--max-hours` with headroom, and after every batch `tools/cloud/status.sh` must say no servers are running, else `tools/cloud/kill-all.sh` (user, 2026-10-06).
- Never run `terraform apply` or any AWS write yourself; the lead asks the user first. Only the launcher scripts start servers (user, 2026-10-06).
- Cost tracks benchmark volume, so the lever is gate size, not cleanup (2026-11-01 audit).
- Build a narrow Rust harness for a subsystem study; never write a second rules implementation (user, 2026-10-10).

## Decided and rejected

- A local open-weights LLM as a cheap tier: rejected for now (user, 2026-10-04).
- A big EC2 instance to make one job faster: no; offloading to relieve overload is wanted (user, 2026-10-04; revised 2026-10-10).
- "No balance sims until the AI is competent" (user, 2026-10-04): historical; pillar 8 faction balance is measured in sims again (for example Egypt, 2026-11-21). Gate size still matters: run the smallest cohort that answers the question.
- Budget appetite: keep the pipeline full at about 5 agents rather than throttling to pace the window (user, 2026-10-04 evening).

## References (open only when needed)

| File | Open it when |
| --- | --- |
| `references/budgets-and-decisions.md` | You need the incidents and numbers behind the budget, concurrency and 2026-10-04 audit decisions |
| `references/mac-era-operations.md` | You are doing maintainer work on the Mac (Codex lanes, runner, slot pool, sccache, watchdog, battery, disk) or diagnosing an old incident; historical for cloud sessions |
| `references/cloud-batches.md` | You run or cost AWS sim batches or the build box |
