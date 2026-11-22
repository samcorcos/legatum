# Workflow: model routing (full text)

The model routing table with prices and the user's words, and patterns for sub-agents.

Moved verbatim from SKILL.md on 2026-11-22 (original order kept). The current rules are in SKILL.md; where this file disagrees with SKILL.md, SKILL.md wins.

## Model routing: the cheapest model that meets the bar (user, 2026-10-04)
- **Balance the two subscriptions (user, 2026-10-31):** "the more we can offload to chatgpt the better." While Codex has headroom against its reset:
  - **Codex:** AI audit rounds, implementation, landing prep (master merges, conflict fixes, repins), sims and analysis (Sol; Astra for the hardest work). The merge into master stays with Claude, behind the docs gate.
  - **Claude:** design calls, Chromium UI checks, the final review before merge, and talking with the user.

  The ops supervisor checks both budgets each run and says which way to lean.
The lead judges the model for every task, delivered or requested, to maximise token efficiency without losing quality. Both the Claude and Codex budgets are limited. Default routing:

| Work | Model |
|---|---|
| Design, rules and faction decisions; merge decisions; talking with the user; novel architecture (e.g. the goal-driven AI's value function); hard root-cause debugging; the ops supervisor (user asked for Opus) | Claude Opus 5.5 ($4/$20 per MTok) |
| **Default for agent work** (Sonnet 5.5, current since 2026-10; $2/$10, half Opus's price, positioned for everyday coding and agent work): implementing well-specified features and rules changes, landing agents, UI work with Playwright verification, code review, routine fixes, resuming a spec'd task | Claude Sonnet 5.5 (the Agent tool's `sonnet`) |
| **Cheap tier, plenty of credit (user 2026-10-17: "you can use haiku 5.5 and luna on codex for tasks that don't need peak intelligence and we have way more credits for that")**: log or status sweeps, file moves, scripted checks, summarising sim reports, running sims and reporting numbers, simple copy and UI tweaks with a clear spec, docs | Claude Haiku 5.5 (the Agent tool's `haiku`) or Codex Luna, or a plain script with no model |
| Implementation, AI changes, analysis | Codex Sol where the sandbox allows it (user 2026-10-23: "we have a lot of ChatGPT credits right now... sol is a good model... use your judgment"); otherwise Claude Sonnet 5.5 |
| **First choice for any well-specified task** (user 2026-10-07: plenty of ChatGPT credit, "anytime we can hand something off to luna without losing quality"): implementation with a clear spec and a measurable test, AI tuning, playtest analyses, small UI fixes, docs, scripts, running sims | Codex Luna |
| Diagnosis, valuation audits, trace reading, design research | Fable, while the 2026-11-11 lift lasts |

Patterns:
- **Sub-agents hand back every time they wait on a box job** (2026-10-23: the bisect agent sent about a dozen full "PARTIAL" reports in a row). Brief long jobs to send a one-line interim note while waiting and the full report only when done or blocked; that saves the lead's context and the user's attention.
- **A sub-agent must not start its own UI reviewer and then hand back** (2026-10-20): the strategic-supply agent did, the reviewer's result never reached anyone, and the branch sat unlanded. The lead runs the `legatum-ui-reviewer` on a finished branch, or the agent waits for its reviewer before handing back.
- **Commits in the main checkout use explicit paths** (`git commit -m … -- path`), never `-a` or `add -A`: agents may have unfinished edits there (2026-10-18, a learnings commit swept an agent's AI work onto master). Agents edit only in their own worktrees.
- **Pace the session (user 2026-10-17):** at most about 4 Claude agents at once; Opus and Sonnet agents share the session limit, which ran out once with 15 agents running. Queue the rest in `docs/status/next-tasks.md`.
- **Cheap does, better checks:** Luna or Sol implements, Sonnet verifies (tests and screenshots), and Opus only judges design-relevant results.
- **Escalate on evidence:** move up a tier when the cheaper model fails a check twice, or the work turns out to need design judgement. Move down when a tier proves reliable for a task type.
- **Scripts before models:** anything repeated (status snapshots, release checks, re-pinning) becomes a script, so no model is spent on it.
- **Learn:** the supervisor records which tier worked or failed per task type in the legatum-ops playbook, and adjusts these defaults.
