# Whole-game review, 2026-11-09 (Fable 5.2)

Five parallel read-only reviews of master `a9eb46864`, merged by the lead. The lead spot-checked the code claims marked ✔ below. Nothing here is decided: the proposals wait for the designer.

Full reports: [AI decision code](1-ai-code.md) · [training and measurement](2-training.md) · [his games against Hard](3-human-gap.md) · [game design](4-design.md) · [app and engineering](5-app.md).

## The one-paragraph answer

Hard is not improving, and more of the same work will not change that. The human-seat benchmark has sat near 23% since 11-01 against a 50% target, and he is 8–0 since 10-27. Three causes stack: (1) the AI's structure limits which moves exist, so tuning and per-scenario patches only choose among bad options; (2) there is no training loop, only measurement around hand fixes, and the gate's noise (about ±1.9 pp) is larger than the typical change (0.65 pp); (3) the rules pay conquest four ways and the rubber bands are switched off or advisory, so a snowball is correct play. The latest evidence fits: `astra/coalition-strikes-bite` formed more alliances, captured nothing more, and failed its gate.

## Findings, ranked

| # | Finding | Evidence |
|---|---|---|
| 1 | The army is ordered by about 60 reflex passes in fixed sequence; the plan's campaign pass runs late, after other passes have claimed its units | `crates/ai/src/military.rs:63-283` ✔ |
| 2 | War is priced as one city over a fixed 10-turn horizon, one campaign at a time; follow-on captures are not valued | `plan.rs:741` ✔, `campaign.rs:56`, `:915` ✔ |
| 3 | Vetoes run before valuation (no second front, walls need a shooter, shortlist-only candidates), and faction code overrides the valued plan afterwards | `plan.rs:1195-1238`, `3649-3707` |
| 4 | The win estimate the plan reads is worse than a uniform guess (Brier skill −33%), fitted on Hard-vs-Hard | `docs/learnings.md:3839` ✔ |
| 5 | No training loop: nothing takes a 0/50 card and works it until it moves. CMA-ES last ran 10-23; the Codex queue is empty | `.progress/astra-queue/pending` ✔ |
| 6 | Every gate is Hard against Hard; no instrument contains a player like him. The designer seat measures finishing his position, not beating him | [2-training.md](2-training.md) |
| 7 | Rivals do not attack him: 10 AI city hits in 116 turns in the 8.9.29 Aztecs game. The gap opens with one act at T25–45 and is structural by T70 | [3-human-gap.md](3-human-gap.md) |
| 8 | Conquest pays four ways (permanent capital VP, size crowns, captured wonders, era objectives) for 1 population and 6 quiet turns. Decadence is off; declining a call to arms is free | `scoring.rs:203` ✔, `experiments.rs:318` ✔ |
| 9 | Live Hard uses 31 ms of a 1 s turn budget; the engine runs on the main thread at End Turn, so search cannot use his 18–24 s of thinking time | `docs/sim-results/perf/turn-latency.md:36` ✔ |
| 10 | Co-op blockers: saves are refused on any rules-version change (20 in 9 days); hosting needs a terminal | `crates/wasm/src/json.rs:92` ✔ |
| 11 | AI intent is invisible: `WarDeclared` carries no reason though the AI builds reason strings | `events.rs:184` ✔ |

## Recommended order (each step is usable alone)

1. **Test the structural claim (verdict 11-11).** Behind one gene, in war turns, replace the reflex passes with one value-priced assignment of units to three objectives (take the plan city, kill a visible stack, hold a threatened city). Score on the two 0% cards, the war suite and Lab capture. One day to build, one overnight local sim.
2. **A real loop on one card (11-10), then all failing cards nightly (11-12).** Order-level search seeded from his recorded orders finds a winning line; the first divergence from Hard becomes a general fix; accept only if sibling cards do not drop. Report his win rate over his last 10 games beside every score.
3. **Fix the measurement.** Short independent scenarios as the primary gate in place of the 8-map benchmark; an imitation opponent built from his 24 tapes in the rivals seat (11-18).
4. **Freeze the rules until the checkpoint**, and pin a co-op build so a co-op game with the second player survives.
5. **Decide conquest pricing at the checkpoint** (his call): occupation (captured cities yield half until peace or elimination), capital VP held instead of permanent, and alliance commitment (G37). Build as off-by-default experiments first.
6. **Ship now, no rules change:** AI reasons on war declarations, threat alerts in the turn report, engine in a Worker (the enabler for search).

## Disagreements between reviewers

- The app review found late-game order load already low (median 2–5 orders a turn after T80); the design review says tedium is hidden because he wins with 5–8 units and will return once the AI defends. Open question for him.
- The human-gap review says not to weaken the leader by rules before the AI attacks ("a longer dull game"); the design review wants conquest pricing decided before evaluator games are regenerated. Both hold if the experiments are built now and flipped at the checkpoint.
