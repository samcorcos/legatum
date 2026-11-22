---
name: legatum-coach
description: The post-game coach's playbook for Legatum. Holds the review checklist, the evidence and what-if methods, recurring patterns in how the user beats (or loses to) the AI, and a verdict table across playtests. Load it before reviewing a playtest game or proposing AI changes based on one. The legatum-game-coach agent maintains it.
---

# Legatum coach playbook

The goal is that on Hard the user usually doesn't win, through better AI decisions and never by cheating (pillar 6). Each playtest review checks the patterns below, adds evidence, and records a verdict. Keep this file under about 250 lines: merge evidence into existing rows rather than appending.

## Review checklist
- **Training games are training data, not verdicts (lead, 2026-11-07).** A game whose archive metadata has `mode: training` (an AI at the Training level, its battles searched) stays out of the "Hard in his seat" comparisons and the verdict table. Its log (`<app data>/training/<game id>.jsonl`) holds his battle positions and the AI's searched battles for offline labelling.
- **When was the game decided?** The VP lead's turn and its size. Did any AI threaten after T100 (pillar 11)?
- **Every AI's strategy, across the whole game** (user, 2026-10-31): name each AI's effective strategy per phase from the catalogue (`docs/design/research/4x-winning-strategies-2026-10-31.md`). Grade its coherence and commitment, fit to faction and map, power-spike timing, army-follows-intent (idle-army upkeep), expansion and economy, reactions to threats, opportunities and the leader, and diplomacy. Then grade its tactics in battle: exchange ratio, focus fire, lone attacks, stragglers, sieges. Strategy first, tactics second; both are mandatory (agent sections 4 and 4b).
- **Expansion:** the user's city count at T50, T100 and T150 against the best AI's. Settlers he stole and lost.
- **Conquest:** cities the user captured or lost, when, and from or to whom. Did the victim or its neighbours respond?
- **The runaway problem:** once the user led, when did the AIs first name him their top threat (`ai_opponent_memory[*].threat_log`), did they coalesce, and did their wars ever reach a city (campaign `end` reasons)?
- **Signature use:** did the user exploit the faction's strength better than the AI does with that faction? Compare with the same faction's AI in the same game when there is one.
- **What did the drawback cost him? (outside review, 2026-11-22; every review)** For the faction he played, name its drawback and say what it cost him in this game, with a number or an event (a turn, a Gold amount, a skipped or delayed build). The review found that four drawbacks (India, Germany, Switzerland, Athens) do not bite a human player. If it cost him nothing, say that plainly and add the game to that faction's tally; do this before proposing any rule change to the faction.
- **Signature as a target:** did the AIs attack what the faction's goal hangs on (the Swiss capital and hoard, the Heir, the Courier Hall)?
- **The hold:** once he started the 10-turn hold, what would have broken it (a thin crown, the capital, the goal, a public camp or Great Work tile) and did any AI try? Count rival units within 3 tiles of each removable VP source per hold turn (`py/camps.py` in coach-8925-celts).
- **Diplomacy coherence:** peace and war flips, alliances that did nothing, wars declared without an army, peace signed with the leader.
- **Decision diff, every review (user, 2026-11-03):** run it on his seat (`docs/playtests/diffs/README.md`), add the game to the cross-game pattern table, check whether shipped fixes closed their patterns, and classify each top disagreement as a valuation error, an execution gap, or his exploiting an AI weakness. Across 23 games (`patterns-2026-11.md`) his edge was founding sooner, army routes and targets, garrisons and escorts, and spending the hoard decisively. Not his edge: waiting instead of declaring, or more Settlers alone.
- **Takeover gap, every review:** run `python3 tools/decision-audit/takeover_gap.py SAVE.legatum --out ~/LegatumRuns/coach-tools/<game>/takeover-gap` (48 salted runs per branch turn). Read the winner column too: before his swing it names the AI that would have won without him (8.9.30 Swiss: the Mongols, 15–16/24 from T30–T45, the empire he then eliminated). Leave `--engine` out, or pass the sha: a tag name fails on the box. Read the curve's steepest rise, its event-window diff, and Hard's traced continuation. Report the gates, final unit orders, and Peace desire/score diagnostics that explain the gap; a selected Peace plan is not a signed treaty, and a frequent gate alone is not causal proof. Keep the raw save and tape outside Git.
- **The counter and the playbook, every win (user, 2026-11-05):** after his easy 8.9.25 Celts win every fix taught rivals to counter him and none taught AI Celts to play his plan (AI Celts stayed 5.9%). Every review of a win ends with both: rival counters and his faction's playbook for Hard (agent §4d).
- **Shipping a fix the benchmark can't see** (lead, 2026-11-03: `decisive_buy`, `contest_hold`, `snipe_read`): the benchmark's Hard seat rarely snipes Settlers, buys a decisive Bomber or holds a Celtic goal, so a fix against his tactic reads identical or within noise there. Ship it on when it changes his recorded decisive turn and the benchmark shows no significant harm (rivals-only CI includes zero); say so in the gene's string. Keep it off when the benchmark is significantly worse (the old `settler_safety` pair: +3.12 pp [+0.83, +5.42]).
- **Blind spots:** undefended cities, Settlers without escorts, armies that never arrived, sieges abandoned.
- **His defence, counted** (canary miss, 8.9.25 Celts: he said "my cities were basically undefended almost the entire game"): per turn, his military units in or next to each city and the strongest rival army within 6 hexes. Name the turns he was exposed and unpunished, and which AI could have punished it. Report his garrison plainly; don't let Walls hits make it read as a defence.
- **His timing and his reasoning** (canary miss, 8.9.25 Celts: he delayed Call the Wild so early VP wouldn't draw a coalition, and used camps as a shield): for each faction trigger or goal he controls, say when he pulled it, the earliest he could have, and what holding back bought him. Look for faction mechanics he used defensively.
- **Smarter AI or a weaker leader?** (user, 2026-10-17; every review) Split the competitiveness into rubber bands and AI decisions, with numbers (method below). The bands are deliberate (pillar 11); report the balance, never "remove them".
- **What would have beaten him?** (user, 2026-10-17; every review) Counterfactual branches from the turning points, ranked by how much they cut his seat's win rate (method below).
- **"Is it the faction?"** (2026-11-01, Aztecs game 2): switch the signature off from the deciding turn with a throwaway core flag gated on `state.turn >= T` (his tape seat and Hard seat), let Hard play his faction from T1 on his map, and run AI-only games with and without the signature (`aztecs_smoke`-style, 192 games). Report the faction's AI win rate against the even share before naming any balance option, and name only drawback-side options (pillar 8).
- **Rank fixes across factions** (user, 2026-10-28: "i win whether I play a defensive faction playing defense or ... sparta and just go offense all game"): rank recommendations by how much they'd cut his win rate with any faction, citing the same pattern in earlier games; faction balance gets one line unless he asks.

- **Questions for the designer are only about intent (user, 2026-11-07): "Only ask me questions that you cannot automatically definitively know from the game logs."** Settle what he saw, knew, placed or ordered, and when, from the tape: per-seat visibility, reveal turns, orders. Ask him only about plans and reasons. Drop any question the tape answers, and state the answer in the review instead.
- **Scenario Lab:** each review ends by cutting 1 or 2 scenarios (`tools/scenarios/cut.sh`, see `scenarios/README.md`) and deletes scenarios that no longer teach anything; log the designer's runs in `references/scenario-lab-feedback.md`.

## His early game, T1 to T60 (for a model seat; 45 tapes, `docs/playtests/early-defence.md`)
Resolves "capital: Walls first" against "Granary, Archer, three Settlers": the first is how he plays now, the second is his opening before 2026-11-07 and when the rush faction is far. Counts are his games that show the rule.
- **Builder with Sparta's or the Mongols' capital within about 20 hexes:** capital Walls first or second (at most a Granary or a Soldier before), done T9 to T18, then Settlers (5 of 5 since 8.9.27; 7 of 23 before). **Rush capital 27+ hexes away:** one unit, three Settlers, Walls about T40 (8.9.40 Byzantium; his words, 2026-11-15: "sparta was pretty far away ... that let me focus on building population, science, and gold"). The 20-to-27-hex line is untested.
- **Sparta or the Aztecs:** three to seven Soldiers queued T2 to T4, Settlers from T26 to T51 or growth by capture (13 of 13). **Mongols:** Horseman or Quarry, then Settlers, no Walls (3 of 3).
- **Gold:** nothing bought before T30 (43 of 45), no Settler bought before T60 (44 of 45). Median 219 at T30, about half ruins and natural wonders; 500 by T30 twice. "About 500 by T30" is not his norm.
- **Army:** his first unit leaves the capital on T1 (42 of 45: scouting, camps, escorts, Settler snipes; 21 AI Settlers taken in 17 games). The capital is garrisoned at T30 in 8 of 41: its Walls are the defence. Builders own 1 to 2 military units to T40.
- **New cities:** 4 hexes out in every direction (no avoidance of the rush side in T1 to T60: 41 cities toward, 34 away), Walls first (82 of 99), the escort stays as the garrison; a unit queued first in 2.
- **Under early attack (25 rival attacks before T61):** walled cities held 8 of 8; unwalled cities with 3+ attackers fell 10 of 12, with a purchase 5 of 6. He buys a shooter in the city (10 of 25; once at Peace before the declaration, Basel 8.9.15), keeps the Settler queues (one exception), proposes Peace twice in 25 and never an Alliance. No capital fell. He abandoned 5 of the 9 games where he lost a city before T61, so his record holds no recovery from an early siege to copy.

## Recurring patterns (status only; evidence in `references/patterns.md`)

Check each review against these, add evidence to the reference table, and update the status here.

- The AI under-expands compared with the user: Partly fixed.
- AI attacks are rare and weak against a human: Improving, uneven.
- An economic runaway is invisible until late: Fixed for the read.
- Coalition wars without the army to win them: Improving.
- Targets the nearest small city, never the faction's signature: Open.
- The strongest AI is busy elsewhere: Improving.
- Nobody contests the hold: Improving.
- Settlers left unescorted: Open.
- Peace priced at 0, so peace-then-war flips: Fixed for the hollow leader.
- The strongest AI's choice of war decides the game: Open.
- Rubber bands tax the challenger too: Design note.
- Leaders aren't checked: no coalition against the runaway: Open.
- A war plan the army can't fight: Fixed for the Celts.
- Declares on a stale or fogged picture: Open.
- One bought defender ends the war: Open.
- Points left on the table (VP conversion): Open.
- One VP short with no two-step plan: Open.
- AI Sparta can't convert a rush into cities: Fixed for the chain.
- The AI pays for an army it never uses (the economy gap): Open.
- Money: Hard spends to a floor, he banks for a moment (user, 2026-11-15: "the AI is basically flat all game"): Open.
- Walls swapped for a shooter mid-build: Fixed by `walls_under_way`.
- AI defence is tuned against AI attackers: Open.
- Walls fall in three turns to phalanx Hoplites (a rules gap in the opening): Rule question for the user.
- Defence doesn't scale with the army at the door, and answers melee with Archers: Open.
- No reaction to "Sparta next door" until the army arrives: Open.
- Campaign flip-flops leave home empty; nobody counterattacks: Open.
- One event scores several eras (objective stacking): Fixed 2026-10-21.
- A neighbour that contests the prey stops a conqueror; garrisons don't: Open.
- The AI trades cities; the human keeps them: Open.
- The human grows into the next era's objective; the AI arrives too small: Open.
- The vow can't close a game against several wars: Resolved 2026-10-22.
- The air stage is uncontested: Open.
- Obsolete armies (army quality before operations): Open.
- Hard stops expanding at War; he keeps settling behind Walls: Open.
- Blind to the leader: no known city, no war: Open.
- His closing VP is a rival's public VP object (Great Work, goal unit): Open.
- The faction kit is a multiplier the AI never gets to use: Open.
- His veteran army of the faction's unique unit; Hard never fields it: Open.

## References (open only when needed)

| File | Open it when |
| --- | --- |
| `references/methods.md` | Running a review: replaying a save at its tag, rubber-band accounting, what-if branches, seat models, tracing a gene, finding the right build |
| `references/patterns.md` | Adding evidence to a pattern or reading why its status is what it is |
| `references/verdicts.md` | Recording a game's verdict (append a row) or comparing with past games |
| `references/scenario-lab-feedback.md` | Reading what the designer said about past scenarios and which were cut |
