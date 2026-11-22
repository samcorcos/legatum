# AI competitiveness: playtest findings and plan (2026-09-27)

## What the user's games show

The user won two games easily; the second was against 7 **Hard** AIs. The playtest build (tag `playtest-1`) predates the game archive, so the only record is the Mac app's autosave of game 2: its final state, turn 197 of a Standard game (the typical length is about 300). By then the human had conquered every AI.

| | Human (Celts) | Each Hard AI |
| --- | --- | --- |
| Cities founded | 7 (plus 19 captured) | 1–5 (19 in total, all captured) |
| Technologies | 17 | 2–7 |
| Gold at elimination | — | up to 181, unspent while cities fell |
| Human army at the end | 6 units | — |

Observed in play (user reports): AI units stand still while a Ranger shoots them; scouts don't flee; a garrison never counterattacks ranged units; the AI never buys defenders.

## Diagnosis

1. **Macro is the biggest gap, not tactics.** The AIs expand to 1–5 cities and research 2–7 technologies in about 200 turns, so they are small, poor and backward long before any war. Tactics alone won't fix that.
2. **Science starved everyone early in this playtest** (citizens only gave population/2), and the AI didn't invest in science buildings to escape it. D1 now adds +2 Science per turn from the capital; the [pacing measurements](sim-results/2026-09-27-capital-science-comparison.md) separate that baseline improvement from the remaining AI investment and balance work.
3. **Resources sit unused.** The AIs die with gold in the bank: no emergency purchases, no walls, no rush-buying defenders.
4. **Tactical passivity:** no response to ranged harassment, no retreat or heal, no focus fire, no counterattack from garrisons.
5. **Our measurement was blind to this.** Every simulation was AI against AI. "Hard beats Normal 1.55×" only means Hard beats a weaker copy of itself. Nothing measured the AI against a competent, human-like opponent.

## Strategy (in order)

1. **Benchmark first: a human-proxy opponent.** Build a strong scripted "Proxy" brain that plays the way the user did: fast expansion, ranged harassment of weak garrisons, capturing low-HP cities, snowballing. The headline metric becomes **Proxy vs 7 AIs**: the Proxy's win rate, the turn it wins, and how many AI cities survive to turn 150. The AI is competitive when a Hard lobby usually stops the Proxy (Proxy win rate under about 25% against 7 Hard AIs) and Normal gives it a real game. This is also how we check every later change.
2. **Fix macro: expansion, economy, science, gold use.** Settler timing and site selection (shared with the new settle-suggestion scoring), a growth-then-expand plan, science buildings on a schedule, and gold always working: buy defenders when threatened, walls in exposed cities, never hoard during a war.
3. **Fix tactics with a deterministic look-ahead.** The engine is deterministic and fast (about 10 ms per turn), so the AI can compare its candidate moves using the engine's own combat forecasts. That covers target selection, focus fire, retreat and heal, garrison sorties and counterattacks on ranged units. It must use only its own fogged view (pillar 6: deciding better, never cheating).
4. **Tune with a self-play league against the Proxy.** Optimize the AI's personality settings (for example with CMA-ES) on thousands of games against the Proxy and previous AI versions. Validate on held-out seeds. Difficulty separates by decision quality (search depth and how well settings are tuned), never by bonuses.
5. **Coalitions against the leader** (the diplomacy alliances phase): an AI that sees the human running away should gang up on them, as in EU4.
6. **Learn from every human game.** Every game is now archived. The incident detector plus an LLM review (astra `ai-game-analysis`) turns each playtest into concrete AI fixes.

Each step is judged against the Proxy benchmark, plus the existing faction balance target (best faction at most 1.5× the worst) and the no-stall checks.

## F13 tactical follow-up (2026-09-27)

Scouts and settlers now choose a safer arrival they can reach this turn before exploring or founding. Safety comes before friendly territory; when no safe hex is reachable, they move farther out of known attack reach if possible. The same retreat logic handles exposed weak military units after favorable attacks have been considered.

A garrison checks favorable sorties against visible ranged bombardiers before wounded recovery can reserve it. Emergency purchases also check whether nearby defenders can actually counterattack or return fire: fortification alone does not answer bombardment. Purchases still require an affordable quoted unit, an upkeep buffer and room to spawn it.

Immediate groups evaluate their combined strength before generic retreat. Only units with a known route they can complete this turn and an acceptable individual melee forecast count toward this early attack. The later attack pass retains longer approaches, and both passes share a single occupation reservation for an empty defeated city.

The fogged real-turn scenarios in `crates/ai/tests/tactical_combat.rs` cover one-turn scout escapes, wounded and healthy garrison sorties, blocked counterattack routes, emergency purchases and a three-Soldier assault. The [500-seed Quick comparison](sim-results/2026-09-27-ai-scout-garrison-comparison.md) records stalls, length, faction win rates and deterministic replay checks. This is tactical regression evidence; it does not establish strength against the Proxy or human players.
