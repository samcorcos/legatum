# Documentation

| Read | For |
| --- | --- |
| [../.claude/skills/legatum-rules/references/index.md](../.claude/skills/legatum-rules/references/index.md) | How the game works, verified against the code: orders, turn resolution, what is available when, and the systems that do not exist. Start here before proposing a mechanic. |
| [../ARCHITECTURE.md](../ARCHITECTURE.md) | Crate graph, turn pipeline, determinism, clients, saves, AI layers. |
| [engineering-notes.md](engineering-notes.md) | Dated engine, determinism, AI measurement, testing, and UI boundary lessons from development. |
| [game-archive.md](game-archive.md) | The game-recording format used for analysis. |
| [ai-evolution.md](archive/ai-evolution.md), [ai-competitiveness.md](ai-competitiveness.md), [ai-proxy-benchmark.md](archive/ai-proxy-benchmark.md), [human-playbook-benchmark.md](archive/human-playbook-benchmark.md), [strategy-diversity.md](strategy-diversity.md), [opening-league.md](archive/opening-league.md) | How the AI is measured and improved. |
| [sim-results/](sim-results/) | Methodology and reproducible results for the simulation benchmarks. |
| [design/advisor-discovery.md](design/advisor-discovery.md) | When useful controls are taught, and how gameplay reviews check feature discovery. |
| [design/](design/) | Design notes and art-direction records, with prompts. |
| [research/](research/) | Background research on 4X design. |
| [sound-qa.md](sound-qa.md), [celebration-sound-qa.md](celebration-sound-qa.md) | Measured audio levels of the shipped sounds. |
| [ops/multi-agent.md](ops/multi-agent.md) | How Claude, ChatGPT/Codex, and Gemini coordinate autonomously on this repo via GitHub Issues, Draft PR locks, two-key cross-model peer merge, and staggered hourly heartbeats. |
| [ops/automations.md](ops/automations.md) | Directory of intended agent heartbeat schedules, control locations, and links to GitHub Actions / Dependabot configs. |

Build and contribution instructions are in [../CONTRIBUTING.md](../CONTRIBUTING.md).

The development layer is public too: `CLAUDE.md`, `AGENTS.md`, `GEMINI.md` and `.claude/` (skills, agents, hooks) record how the project is run with AI agents; `docs/learnings.md` is a short digest of the engineering log, whose full archive is `docs/learnings/` (one file per entry; the older month files are closed) (`engineering-notes.md` is the curated version); `docs/ops/`, `docs/status/` and `tools/` hold the operations side (multi-agent coordination, cloud build box, spot simulations, test gates).
