# Legatum

A historical 4X on a hex map (real peoples such as Switzerland, the Mongols and Sparta; grounded in real history, no sci-fi or fantasy, user 2026-10-10), built to feel familiar to Civ 6 players. The design target is two humans on a permanent team plus 6 AI empires; solo against AI empires, hot seat, LAN and online games also exist. Simultaneous turns, 3–6 hour games.

## Sources of truth

| What | Where |
| --- | --- |
| Design: what we're building, why, and the decisions log | Design doc (Claude Doc): (a private document, not published) — read it with the docs connector before any design work |
| Engineering learnings: bug causes, balance findings, dead ends, gotchas | `docs/learnings.md` |
| The active AI plan: goal, increments, gates, dates, open decisions | `plans/ai-war.md`, tracked in `plans/ai-war.prd.json` and `plans/ai-war-progress.txt`. Update both every session that touches it |
| Simulation results | `docs/sim-results/`, written by the simulation runner |
| How the game actually works: orders, turn resolution, what's available when, systems that don't exist | `.claude/skills/legatum-rules/` (code-verified). Load it before proposing any mechanic, faction, opening or balance change; update it in the same change whenever a rule or mechanic changes |

Each kind of knowledge lives in exactly one place. Don't duplicate design content into this repo.

## Pillars (test every change against these)

1. Question every requirement. A mechanic stays only if it serves explore → expand → build → compete.
2. Familiar to Civ 6 players, unless we're deliberately fixing a known Civ problem.
3. Turns stay the same size: delegation grows gradually.
4. The victory race is always visible (live public VP track).
5. No hidden math: every number has a breakdown, every choice a forecast.
6. AI gets harder by deciding better, never by cheating.
7. Simultaneous turns: nobody waits.
8. Asymmetry with compensating drawbacks (TI4, StarCraft; not Eclipse): each faction leans hard into its own play style, and its signature strength is paid for by a weakness elsewhere that pushes it further into that style. Balance by adding or deepening drawbacks and by making weak identities pay off; never by weakening strengths or homogenizing. The measure: the best faction's win rate is at most 1.5x the worst's at Standard speed (the only speed; Quick removed 2026-10-10), validated on about 2,000 games plus held-out seeds.
9. The next action is always obvious: every screen makes clear what decision or action it wants from the player, with one unmistakable primary call to action. Anything clickable looks clickable before it's hovered.
10. Strategic diversity, no dominant strategy: every strategy (rush, wide, turtle, economic, diplomatic) has a real counter, rock-paper-scissors style, so the best plan depends on your faction, map, neighbours and what opponents do. Measured in the sims: winning strategies stay varied, and no archetype beats all others. This starts at turn 1: several openings (e.g. rush, greedy expand, defended expand, snowball) must be viable, each with a counter. If an opening phase has one right answer, cut it (e.g. start with more Settlers) rather than make players sit through it. Expansion, by settling or by conquest, is the core loop: "tall" (staying small) is deliberately not a viable strategy.

11. Competitive to the end: a leader can pull ahead, but nobody should know the winner by turn 100 of 300. Rubber-band mechanics (e.g. tech diffusion, coalitions against the leader) keep games close without erasing a deserved lead.

## Architecture

- Rust simulation core: game state + all orders + seed → next state. Deterministic, integer math only, no rendering code.
- Desktop app: Tauri with a TypeScript web UI and a 2D hex renderer.
- Headless simulation runner: Rust CLI, parallel across cores.
- Python bindings (PyO3) only if we reach learned AI.

## Recording what we learn (mandatory)

- A design decision is made → add a row to the design doc's decisions log in the same turn.
- Something non-obvious is learned → add a dated entry to `docs/learnings.md` in the same turn. Correct or delete entries that turn out to be wrong.
- The user states a preference, rejects an idea, or corrects a recommendation → update the matching skill in the same turn: `legatum-design` (design philosophy, decided and rejected ideas), `legatum-factions` (faction method, current designs, lessons), `legatum-ui` (UI preferences), `legatum-art` (art direction and consistency), `legatum-workflow` (how we work, operational gotchas), `legatum-rules` (how the game works). Load the relevant skills before design, UI or planning work so settled questions aren't reopened.
- Playtest builds ship first and are reviewed after (user, 2026-11-11): cut the build as soon as the code passes verify, then run the `legatum-ui-reviewer` agent (`.claude/agents/`) only if the build changes enough UI to warrant it, and ship its fixes as a follow-up build. Before merging any branch that changes UI to master, the reviewer still runs. It tours every screen at 1000x600 (the user's laptop) and 1440x900, and judges the screenshots against `legatum-ui`. Fix blockers first, and fold its proposed rules into the skills.
- GitHub `samcorcos/legatum` is the source of truth, with one line on `master` (user, 2026-11-21). All work goes branch → pull request into `master` → CI → merge on GitHub; local checkouts only `git pull`. Never push to `master` directly.
- While GitHub Actions is unavailable (user, 2026-11-21: "use something other than github actions for now", until the repo is public), "CI is green" means a passing `tools/ci/local-ci.sh` run on the PR's head, in a Claude cloud container or on the user's Mac, with its summary table posted as a PR comment. It runs the same steps as `.github/workflows/ci.yml`.
- Who merges (user, 2026-11-21): Claude merges a PR it made at the user's request once CI is green and the reviewers the change needs have passed. Never merge anyone else's PR (outside contributors) without the user.
- Before merging a PR, the gates must pass: CI's `gates` job runs the static checks (`tools/gates/check.py --checks-only`), and the lead runs the reviewers the changed paths need (`legatum-docs-reviewer` for code; `legatum-plain-language-reviewer` for any player-facing text; `legatum-ui-reviewer` for UI; for art, `legatum-art` plus `tools/gates/art.py`). Run `tools/gates/check.py <branch> --base origin/master` locally to see the full verdict with markers.
- Player text is board-game style (user, 2026-11-21): a card, tooltip or tip says the short rule ("Barbarians don't attack the Celts."); the nuance lives in the Guide, the in-game rulebook. Rules in `legatum-ui` ("Game text: plain language").
- A Stop hook (`.claude/hooks/learnings-check.sh`) blocks finishing when code changed without a learnings update. If nothing is worth recording, say so in one line.
- After context compaction, a SessionStart hook reminds you to re-read this file and `docs/learnings.md`.

## Astra (gpt-6-astra via Codex CLI)

Codex and Claude both have limited budgets (user, 2026-10-04): pace both. Offload implementation to Codex when it fits, keep its queue fed but not oversubscribed, and have Claude review the results before anything merges. Codex runs only on the user's Mac; cloud sessions reach it through Remote Control.
- `tools/astra/runner.sh queue <k>` (one per lane k=1..4, plus `runner.sh reaper`) works through `.progress/astra-queue/pending/*.md` (headers `mode: review|write`, `name:`, optional `model: astra|sol|luna` and `effort:`, blank line, prompt). Write tasks run on branch `astra/<name>`, cut from a freshly fetched `origin/master`, in the worker's persistent lane worktree (`.claude/worktrees/lane-<k>`), so builds stay incremental. The runner pushes the finished branch to GitHub; the lead opens the PR and merges it after review.
- Pick the model per task (user, 2026-10-04): only `sol` (gpt-6-sol, default: implementation, AI and design work, reviews) and `luna` (gpt-6-luna, cheapest: mechanical work such as removals, docs and running sims). `astra` (gpt-6-astra) is for max-intelligence reviews and audits (user, 2026-10-24).
- Model routing (user, 2026-10-04): pick the cheapest model that meets the quality bar for every task (table in the legatum-workflow skill). Sonnet 5.5 (half Opus's price) is the default for agent implementation work; Opus 5.5 for design judgement, novel architecture and hard debugging. Fable is allowed for diagnosis and audit agents for now (user, 2026-11-11; he may bring the ban back).
- `tools/astra/runner.sh reviewer` reviews every new `origin/master` commit range.
- Outputs land in `.reviews/astra-*.md`; progress in `.progress/astra.log`. Queue new tasks whenever the queue runs dry.
