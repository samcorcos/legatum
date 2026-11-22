# Why does the AI keep missing strategies that are obvious to the designer? (brief, 2026-11-05)

## The question (designer, 2026-11-05)
"can you do a thorough strategy review and think through how we keep missing strategies that seem obvious to me but are not as obvious to the AI?"

## The pattern so far
Again and again the designer wins easily with a plan the Hard AI does not play, and porting his plan into the AI finds big gains that had been sitting there:
- **Swiss (11-05):** Walls stand in for garrison troops; the AI kept a troop quota per city and paid upkeep it could not afford. +4.4 pp Swiss win share when fixed.
- **Celts (11-05):** the AI could not hold its own goal (the Wild): an execution bug, +2.8 pp.
- **Aztecs (11-05):** his lean army is what helped (army size targets), not his rush.
- **India (11-05):** five units for twenty cities, Walls first, frontier towns at 100% corruption used as walled buffers that "poke and delay enemy troops", a wide empire where losing one to three border towns "wouldn't affect me that much... by the time they got to cities that mattered, i could shift production to get more units and fight back. I had buffer." AI India garrisons every city like the core, loses 3.6 cities a game. Walls-as-garrison plus shared guard values: +6.0 pp.
- **Earlier:** conquest as the fastest economy, zero-sum valuation, pick one strategy and commit, Settler sniping, decisive late tech wars (his Bomber war on the leader at T149), Masonry first, etc. (`.claude/skills/legatum-design/SKILL.md`, `.claude/skills/legatum-coach/SKILL.md`).
- **Learning (docs/learnings.md, 2026-11-05 "Copying his faction playbooks paid through bugs and shared quotas, not through his moves"):** the gains came from execution bugs and shared quotas the faction's kit makes wrong; copying his literal moves did nothing.

The AI is search plus hand-written valuation and plans (`crates/ai`; read `.claude/skills/legatum-rules/references/game-rules.md` AI sections and the `crates/ai/src` module map), tuned by hill-climbing genes on an AI-only census and a human-seat benchmark (`docs/sim-results/ai-quality/`, `tools/sim/quality_gate.py`, `crates/sim/examples/hard_vs_human.rs`, `seat_run.rs`).

## Sources to mine
- Every coach review: `docs/playtests/reviews/*.md` (about 25 games), the decision diffs `docs/playtests/diffs/*`, the patterns ledger `docs/playtests/diffs/patterns-ledger.md` and `patterns.jsonl`.
- `docs/playtests/human-reasoning.md` (his own words on why he did things).
- `.claude/skills/legatum-coach/SKILL.md` (recurring patterns, verdict table), `.claude/skills/legatum-design/SKILL.md`, `.claude/skills/legatum-factions/SKILL.md`.
- `docs/learnings.md` (search for playbook, valuation, quota, garrison, decisive, coach).
- `docs/sim-results/ai-quality/ideas.md` and the AI quality ledger.
- The AI code itself: where are quotas, fixed minimums, per-city rules, faction overrides, and value terms that ignore position or faction.

## What we want
1. **A catalogue** of every strategy he found that the AI missed: what it is, when found, how it was found (playtest, coach, sim), how much it was worth when ported, and **why the AI missed it**, classified by root cause. Candidate classes (refine them): a fixed quota or minimum instead of a valued choice; uniform per-city treatment where cities differ in value (core vs frontier); the valuation omits a whole effect (time bought, denial, defensive depth, tempo); horizon too short; a shared default the faction's kit makes wrong; an execution bug in a plan; no concept of a whole-game plan; information the AI could use but doesn't model (e.g. what rivals will do with freed armies).
2. **The meta-answer:** why our process keeps finding these one at a time, after he plays, instead of before. What in the AI's architecture or our tuning loop makes them invisible (e.g. hill-climbing local genes can't discover structural plans; AI-only censuses reward AI-vs-AI quirks; quotas encode the designer-of-the-AI's assumptions).
3. **Still-hidden strategies:** using the catalogue's root causes, predict the next five to ten strategies a strong human would exploit that the AI still misses, per faction where relevant, with evidence from the code (the quota or rule that encodes the blind spot).
4. **A discovery method** that would find these before he does: concrete, runnable here (cloud sims are cheap; the seat runner lets Hard play any recorded turn; an LLM can propose plans; deep offline search exists). Rank options by expected yield per cost.
