**Repetitive input in large empires: a measurement, not a feature — 2026-10-31**

**Why**

The external review (full review of 2026-10-31, §2 rank 1) says a player with ten to eighteen cities repeats the same city orders, and that "Turns stay the same size" (pillar 3) is not yet true there. It asks for a measurement first and batching second. This note is the measurement. It builds nothing.

**Method**

- Data: the order tapes in two of the designer's winning games, read from copies of their autosaves.
  - Mongols win, 2026-10-30: turns 96 to 172, the human seat only (the save resumed mid-game, so the tape is partial). 10 cities at turn 96, 18 at the end.
  - Switzerland win, 2026-10-28: the whole game, turns 1 to 180. 1 city at turn 1, 15 at the end.
- The tape (`archive.tape.turns[*].orders[seat]`) holds the orders a seat committed at Ready. It does **not** hold clicks. An item chosen and then replaced in the same turn never reaches it, and opening a city costs a click that leaves no order. So every count below is a lower bound on real input, and the click model below is an estimate. The review's own first step stands: instrument real inputs before building.
- Script: `tools/analysis/repetitive_input.py` (standard library only; run it on any save). Definitions:
  - **Pick**: an item newly asked for in a city. A `SetProduction` is one pick. A `SetProductionQueue` counts only the items not already in that city's previous queue (the previous queue is seeded from the save's state when the tape starts mid-game).
  - **Repeat** (window W): a pick of the same item, tile ignored, in a *different* city by the same player within the last W turns. W = 0 means the same turn.
  - **Production inputs** (the click model): one open per city edited in a turn, plus one per pick, plus one per Buy.
  - **Repetitive inputs** (W = 5): the repeat picks, plus the city opens whose every pick is a repeat (the open exists only to repeat something).

**What the tapes say**

| | Mongols T96–172 | Switzerland T1–180 | Switzerland, turns with 10+ cities (24 turns) |
| --- | --- | --- | --- |
| Committed human orders | 238 | 405 | n/a |
| Of them, city production (queue, set, buy) | 55 (23%) | 65 (16%) | n/a |
| Active play time (sum of recorded turn times) | 1,418 s (18 s a turn) | 1,661 s (9 s a turn) | n/a |
| City opens (turn, city) | 49 | 59 | 19 |
| Item picks | 106 | 115 | 28 |
| Production inputs (click model) | 161 | 179 | 48 |
| Repeat picks, same turn (W = 0) | 5 | 5 | 3 |
| Repeat picks within 5 turns | 52 (49% of picks) | 29 (25%) | 11 (39%) |
| Repeat picks within 10 turns | 59 | 46 | 18 |
| **Repetitive inputs (W = 5)** | **69 (43% of production inputs)** | 43 (24%) | **18 (38%)** |
| Whole queue repeated in another city within 10 turns | 5 | 4 | n/a |
| Cities edited more than once / cities edited | 12 / 18 | 11 / 16 | n/a |
| Edits of a city within 10 turns of its previous edit | 6 | 9 | n/a |

Findings:

1. **The repetition is real and it is across turns, not within one.** Of the Mongols picks, 49% repeat an item chosen for another city in the last five turns. Only 5 of 106 repeat in the same turn. The player does not set one item in six cities at once. He walks his cities as each one finishes, choosing the same Bomber, Rifleman or Workshop again and again. The most repeated picks: Mongols Bomber 18, Rifleman 10, Workshop 9, Laboratory 8, Hydroponics 8, Settler 8, Clearinghouse 8; Switzerland Market 14, Granary 14, Workshop 12, Walls 10, Laboratory 10, Hydroponics 9.
2. **So "apply to the cities selected right now" would save almost nothing on this data.** Same-turn groups across both games total 7 groups, each of 2 cities, which batching removes 7 picks from about 220. The repeat that matters is "this city just finished; give it the same thing its neighbours got". That asks for a queue template or a per-city "same as" (copy the production of another city, or apply a saved sequence to every city that can build it), not a multi-select.
3. **Units dominate the repeated picks in the late Mongols game** (30 of the 52 five-turn repeats are units, mostly Bombers and Riflemen; Switzerland's repeats are 24 buildings of 29). Buildings repeat because every city wants the same economy set. Units repeat because one plan (an air wing) is built everywhere.
4. **City production is only 16 to 23% of committed orders.** Unit orders are the rest (Mongols: 65 path moves, 30 moves, 30 Fortify-or-Wait, 20 skips; Switzerland: 76 path moves, 66 moves, 60 ranged attacks, 46 Fortify-or-Wait, 34 auto-explore). Unit orders in runs of three or more of one kind in a turn are few: 44 of 165 unit orders in the Mongols game, 35 of 330 in Switzerland, and moves have different destinations. A city batching feature addresses a minority of inputs. It is still the repeated minority the review pointed at.
5. **Re-editing is modest.** 12 of 18 Mongols cities were edited more than once, but only 6 edits came within 10 turns of the last edit of the same city, and none were pure reorder-or-remove edits in the Mongols game (one in Switzerland). Queue-editing churn is not the problem on this tape. It may be on the clicks the tape cannot see.
6. **Empire size shows in the order count.** Switzerland: 1.9 committed orders a turn under 10 cities, 4.4 with 10 or more. Mongols, always 10 or more: 3.1.

**What this means for the design**

- The first feature is a per-city template, not a multi-select: "Same as Bukhara" or a saved production sequence applied on a city's idle prompt and to selected cities, with the whole queue, the Gold commitment and the skipped cities (missing prerequisite or resource, conflicting order) shown before it applies. Same-turn multi-select comes second, for the case that is 5 of 106 today.
- It targets the idle-city decision: the cities that just finished something. That is where the repeat occurs, so the batch entry point belongs on that prompt, not in a separate overview.
- The ceiling is lower than the review's phrasing suggests. If every five-turn repeat disappeared at no cost, Mongols production inputs would fall from 161 to 109 (32%) and Switzerland's from 179 to 150 (16%; 48 to 37 in its 10-plus-city turns, 23%). A feature has to absorb nearly every repeat to reach 30% of production inputs, so the test below measures repetitive inputs and counts the batch UI's own clicks against it.

**Proposed acceptance test (one test, for a future batching feature)**

A replay task, run by a scripted driver against the real app (Playwright, WASM engine) and counted by the input log:

- Start from the Mongols save at turn 96 (10 cities, growing to 18). For turns 96 to 172, give the driver the human's recorded production orders per city per turn as the goal.
- The driver reaches each goal twice: once with today's UI (open city, pick, queue, Buy), once allowed to use the batching feature however it likes. An input is a click or key press that opens a city or panel, picks an item, confirms, or buys, including every click inside the batch UI.
- **Pass:** repetitive inputs (as defined above; 69 on this replay with today's UI) fall by **at least 30% (to 48 or fewer)**, while all three hold:
  1. the final committed production orders in every turn equal the recorded ones;
  2. the pending-decision count at each End Turn equals the baseline's (no urgent decision is missed or hidden by the batch);
  3. every city the batch skipped is listed with its reason before Ready, and one undo restores the previous orders.
- Report the Switzerland turns with 10 or more cities (baseline 18 repetitive inputs, 48 production inputs) as a second row, not a gate.
- The first run is the baseline. It must reproduce 161 production inputs and 69 repetitive inputs within 5%, or the model is wrong and the threshold is re-derived before any feature work.

**Limits**

- Two games, one player, both wins. The tape has no clicks, no scrolling, no hover, and no undone choices, so real input is higher than counted. The repeated-pick definition ignores an improvement's tile, which is a separate placement click.
- A previous queue seeded from orders (not from the engine's state turn by turn) can count a re-queued, already completed item as not new. This lowers the repeat count slightly.
- The Mongols game is a late-game segment dominated by one plan (Bombers). A mid-game segment or another faction could repeat differently.
- Before building: add a local, anonymous input counter (city opens, picks, queue buttons, Buy) to the app and replay a few more games so the baseline is real clicks, not committed orders.
