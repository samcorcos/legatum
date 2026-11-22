# Decision audit: what Hard would have done in the designer's seat, and what it cost

Re-audit on today's master (round 3, 2026-10-31): `audit-round3.md`.

2026-10-30. Branch `worktree-agent-a761c4267795ddc68`. Tool: `tools/decision-audit/run.sh` (one command per game) over
`crates/sim/examples/decision_audit.rs`; reports from `tools/decision-audit/report.py` and `summary.py`. Per-game reports:
`docs/sim-results/ai-war/decision-audit/<game>-window10.md`. Raw data: `~/LegatumRuns/decision-audit/2026-10-30/`.

## The question

The designer's idea: let a strong player play against Hard, and track what Hard would have done in his place, what he did
instead, and whether that was better or worse. Our strongest player is the designer himself, and we have his recorded wins,
so the audit replays his games.

1. **Diff, every turn.** Replay the tape on the engine that recorded it, with every hash checked. Shadow his seat with the
   app's Hard (`ScriptedBrain::with_difficulty(.., Hard)`), which sees his own PlayerView and keeps its memory across turns,
   and shadow every AI seat too, so each AI has its real memory. Then resolve the turn twice from the same state, once with
   his orders and once with Hard's, and compare the results in seven categories: production and buys, research, diplomacy,
   settling, army (military units outside his cities), garrison (units in or next to a city), and other (Raze, civilians).
2. **Value, sampled turns.** Branch arms from the same state that differ only in his seat's orders: `rec` (his),
   `alt` (Hard's), and for each category `fwd` (his, with that category from Hard) and `rev` (Hard's, with that category
   kept his). Hard then plays his seat, and every AI seat plays Hard from its restored memory. Each arm runs 32 salts with
   the dice reseeded, paired by salt, and is scored at 20 and 50 turns on his VP, his cities and the `winchance` estimate.
   The category's **share** is the mean of `fwd` and `rev`: mixing two planners costs something on its own, and the two
   swaps charge that cost to opposite sides.
3. **Window.** A one-turn swap (the designer's spec) barely registers for choices Hard makes again every turn, such as
   production and research, because Hard takes over the next turn. So the main run swaps a **10-turn policy**
   (`--window 10`): his recorded orders for 10 turns, against Hard live for 10 turns, then Hard in both arms.
   The one-turn run is reported too.

Positive values mean his choice was better than Hard's. ± is a 95% interval.

## Games audited

| Game | Engine (tag) | Turns | Replay | AI orders reproduced | Sampled turns (window 10) |
|---|---|---:|---|---|---:|
| Mongols win 10-30 (two saves: T1-95 and T96-172) | 0cf2a61d3 (8.9.17) | 172 | 0 hash mismatches | 688/688; T96 memory restore from the earlier save | 17 |
| Swiss win 10-28 | 060f78ef4 (8.9.15) | 180 | 0 | 531/531 | 17 |
| Athens win 10-27 (Bombers) | a58e80856 (8.9.13) | 177 | 0 | 696/698 (from T104) | 16 |
| Sparta win 10-27 (stopped at T125) | a58e80856 (8.9.13) | 125 | 0 | 309/309 | 11 |
| Japan win 10-26 (Bombers) | d1233d486 (8.9.11) | 234 | 0 | 919/919 | 22 |

Each game runs on the engine that recorded it, so "Hard" means the Hard he played against at that release. In every
branch at salt 0, the AI seats reproduced the tape's strategic orders on the branch turn (4,784 of 4,784 seat-turns over
the five games), so the branches start from the AI's real memory. The window-10 run is 42,496 branch games.

## Where Hard disagrees with him

Turns in which the category differs, and what Hard picks where a city's production differs (his / Hard's):

| Game | Production | Army | Garrison | Purchases his / Hard | Military units chosen | Settlers chosen | Walls | Markets + Granaries |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mongols | 168/172 | 135 | 106 | 7 / 180 | 105 / 261 | 35 / 2 | 8 / 21 | 61 / 8 |
| Swiss | 159/180 | 116 | 123 | 5 / 80 | 23 / 219 | 59 / 1 | 10 / 0 | 32 / 0 |
| Athens | 161/177 | 101 | 104 | 2 / 181 | 82 / 154 | 8 / 10 | 3 / 0 | 11 / 1 |
| Sparta | 111/125 | 102 | 50 | 0 / 83 | 105 / 190 | 2 / 15 | 3 / 4 | 66 / 2 |
| Japan | 233/234 | 157 | 192 | 5 / 396 | 72 / 176 | 22 / 27 | 7 / 0 | 25 / 2 |

- **Hard spends; he banks.** In his seat Hard makes 80 to 396 purchases in a game (units and tile improvements); he makes
  0 to 7. After the same turn Hard holds 71 to 260 Gold less on average. His median treasury runs 276 to 1,493 Gold.
- **Hard builds an army; he builds an economy.** Where they differ, Hard picks military units 2 to 10 times as often; he
  picks Markets, Granaries, Workshops, Hydroponics and (Mongols, Swiss) Settlers. He wins with a median of 3 to 5 land
  military units.
- **Hard declares wars he never fights.** In his seat Hard orders a declaration it repeats turn after turn: the Swiss
  on team 3 for 94 turns, Japan on team 2 for 35 and team 4 for 20, Athens on team 2 for 16. He accepts few of the peace
  offers he gets; Hard would accept them (Sparta 5 of 5, Athens 3, Japan 3, Swiss 2).
- **Hard razes captured cities.** Hard orders Raze in his seat in four of the five games (Sparta on 20 turns, Swiss 8,
  Mongols 4, Athens 2); he never razes.
- **Attacks:** in the turn itself the counts are close (his 83 to 204 a game, Hard's 77 to 195), but see below: over 10
  turns his units take cities and Hard's don't.

## What the disagreements cost (10-turn policy swaps)

His 10 turns against Hard's 10 turns, then Hard in both arms, mean per sampled turn. Cities and VP are the reliable
measures; win chance is noisier (see the caveats).

| Contrast | Cities @20 | Cities @50 | VP @50 | Win chance pp @20 | Win chance pp @50 |
|---|---:|---:|---:|---:|---:|
| **Whole policy** | **+0.51 ±0.03** | **+0.53 ±0.05** | **+0.25 ±0.05** | +1.1 ±0.5 | +2.1 ±0.7 |
| Army share | +0.21 ±0.02 | +0.20 ±0.04 | +0.08 ±0.03 | +0.1 ±0.3 | +0.2 ±0.5 |
| Production and buys share | +0.13 ±0.02 | +0.18 ±0.04 | +0.15 ±0.03 | +0.8 ±0.3 | +2.0 ±0.5 |
| Garrison share | +0.12 ±0.02 | +0.15 ±0.03 | +0.05 ±0.03 | +0.9 ±0.3 | +0.9 ±0.5 |
| Other (Raze) share | +0.11 ±0.01 | +0.09 ±0.02 | +0.07 ±0.02 | +0.4 ±0.1 | +0.1 ±0.2 |
| Diplomacy share | +0.08 ±0.01 | +0.05 ±0.03 | -0.01 ±0.03 | -0.9 ±0.2 | -0.2 ±0.4 |
| Settling share | +0.03 ±0.01 | +0.01 ±0.02 | -0.02 ±0.02 | +0.2 ±0.2 | -0.0 ±0.3 |
| Research share | +0.01 ±0.01 | +0.00 ±0.02 | +0.01 ±0.02 | -0.1 ±0.1 | +0.2 ±0.3 |

Mean over the five games (each game's interval pooled over its sampled turns). By game, the whole-policy value in cities
@20 is +0.49 (Mongols), +0.65 (Swiss), +0.39 (Athens), +0.62 (Sparta), +0.42 (Japan): every 10-turn window of Hard in
his seat costs about half a city within 20 turns, in every game. Over a 170-turn game that is 17 windows.

The army share is positive in all five games (cities @20: +0.12, +0.15, +0.17, +0.41, +0.18), and both swaps agree
(`fwd` +0.28, `rev` +0.11 cities @50). Production is positive in four of five (Athens 0.00); its `fwd` and `rev` agree
at @50 (+0.19, +0.16). Settling and research are near zero everywhere.

### The largest single decisions (10-turn windows, cities @50)

| Game, turn | Category | Cities @50 | VP @50 | His orders on the first turn | Hard's |
|---|---|---:|---:|---|---|
| Japan T220 | garrison | +3.45 ±0.28 | +1.70 ±0.49 | Sends Autonomous Armour out of its city toward (4,27), at war with all four empires | Skips the Armour and a second unit |
| Athens T160 | whole window | +4.72 ±0.38 | +4.50 ±0.48 | Keeps captured city 7, refuses Peace proposal 69, Infantry attacks (2 attacks) | Razes city 7, accepts the Peace, pulls the Infantry back (0 attacks) |
| Athens T160 | army | +2.50 ±0.35 | +1.23 ±0.40 | Infantry to (20,22), next to the enemy | Infantry back to (17,21) |
| Athens T160 | other (Raze) | +2.47 ±0.41 | +3.80 ±0.41 | Keeps the captured city | `Raze { city: 7 }` |
| Sparta T90 | army | +2.25 ±0.46 | +1.20 ±0.27 | Two units onto (23,7), one attack | Trooper back to (18,9), the other skips |
| Sparta T10 | production | +2.67 ±0.43 | +1.42 ±0.31 | Over the window his capital keeps building Soldiers for the rush | Switches the capital to a Ranger (T12) |
| Swiss T160 | army | +1.42 ±0.34 | +1.28 ±0.62 | Two Riflemen forward (rows 7-8), Field Cannon shoots (5 attacks that turn) | All three moved back (rows 9-13), no shot (4 attacks) |
| Mongols T150 | production | +1.39 ±0.37 | +1.05 ±0.34 | Walls, Market, Granary queued in a new city; Bukhara stays on the Relay Uplink wonder | Buys a Rifleman for 580 Gold; switches Bukhara from the wonder to a Field Cannon |
| Swiss T120 | diplomacy | +0.92 ±0.23 | +0.62 ±0.41 | No new war | Declares on team 3; Geneva's Settler and Fribourg's Workshop become Soldiers |

The pattern: Hard in his seat parks or pulls back units that he sends in, keeps attackers inside cities, razes what he
keeps, and turns production into units and Gold into purchases.

### One-turn swaps (`--window 1`)

On a single turn the swaps are small, because Hard takes over the next turn and makes most of the same choices again.
Mean over the five games: whole turn +0.06 ±0.05 cities @50 and +0.00 ±0.05 VP. By category everything is within ±0.1
cities except the one-order Raze (+0.32 ±0.10 cities @50) and the army (+0.08 ±0.04). Win chance @20 swings widely (Sparta
−11.1 ±0.9 pp, Japan −3.9) with VP and cities unchanged: the estimate reacts to army strength after his attacks, not to a
better position. Clearing the seat's standing orders at the handover (`--handover clear`, Mongols and Sparta) changes
nothing (Sparta identical to the decimal; Mongols cities @50 +0.02 both ways): Hard re-orders every unit each turn, so
his standing paths don't leak into its play.

## Against the coach's and today's findings

| Finding | What the audit says |
|---|---|
| **Idle armies** | **Agrees, and it is the largest consistent cost.** Hard in his seat holds units back from attacks he makes; the army and garrison shares together are worth +0.33 cities per 10-turn window (@20), positive in all five games. |
| **No march on a hollow leader** | Not testable from his seat: he was the leader. In his seat Hard declares wars it then does not fight (the Swiss on team 3 for 94 turns), the same declare-and-idle shape. |
| **Gold banked and not spent** | **Not a flaw in his seat; the reverse.** Hard spends far more than he does (80 to 396 purchases against 0 to 7), and his banked play beats it: production and buys share +0.18 cities, +0.15 VP @50. Spending more is not the fix; spending on the right thing, or not at all, is. |
| **Walls first** | Only in the Mongols game (Hard Walls 21 / his 8 where they differ); elsewhere Hard picks Walls no more than he does. Not a general pattern in his seat. |
| **Settlers** | Mixed. Hard picks Settlers far less than he does as the Mongols (2 / 35) and Swiss (1 / 59), more as Sparta (15 / 2). Settling moves (where to found) are worth nothing either way (+0.03 cities). Settler production sits inside the production share. |

## What it means for fixes

1. **Commit units to attacks the AI already has the army for.** The army and garrison shares are the largest consistent
   loss, in every game. The examples are tactical: a unit next to an enemy city steps back or skips. Look at the tactical
   layer's retreat and skip rules (`crates/ai` tactics, campaign and `hollow_march` work in flight) before strategy.
2. **Production mix: economy before units when not threatened.** Hard turns production into military units and Gold into
   unit and improvement purchases; his Markets, Granaries and Workshops beat that by +0.15 VP per window. This argues
   against any fix that makes Hard spend more (the gold-freeze direction should spend less on units, not more).
3. **Stop razing captured cities** (Raze share +0.11 cities; Athens T160 alone +2.5 cities). Cheap and clear.
4. **Do not declare wars the army won't fight**; low on value here (diplomacy share ±0.1), so after the three above.

## Caveats

- **The `rec` arm hands Hard a position built for his plan, which Hard doesn't follow**, and his recorded
  orders misfire once reseeded dice move the game away from the record. Both bias the result against him: the values
  are lower bounds on his advantage.
- **Reseeding redraws the era objectives** (they derive from `setup.seed`), identically in both arms of a pair. Paired
  differences stay fair; absolute levels are not his game. **Check with the recorded dice** (`--dice fixed`, salts vary
  only the AI seeds, his objectives kept; Mongols and Swiss): the same signs and sizes. Whole policy +0.63 and +0.76 cities
  @20 (reseeded +0.49, +0.65); army share +0.21 and +0.26 (reseeded +0.12, +0.15); production +0.30 and +0.07 (+0.32,
  +0.12). Its intervals are near zero because AI salts alone replay nearly the same game, so they overstate precision;
  quote the reseeded ones.
- **Win chance is the AI's own estimate** and credits army size: in Japan it reads Hard's 10 turns as better (−5.9 pp
  @20) while VP and cities read his as better (+0.07 VP, +0.63 cities @50). Rank by cities and VP.
- Shares don't add up to the whole (+0.69 against +0.51 cities @20): the categories interact.
- Each game measures the Hard of its release (8.9.11 to 8.9.17), not today's master.

## Reusing it for any decision source

```
tools/decision-audit/run.sh --out DIR SAVE [LATER_SAVE] -- --window 10 --turns 10,20,... --salts 32
tools/decision-audit/run.sh --out DIR --engine here --ship gene.json SAVE -- --alt params:audit-in/gene.json ...
tools/decision-audit/run.sh --out DIR --ship llm.jsonl SAVE -- --alt orders:audit-in/llm.jsonl   # one-turn only
tools/decision-audit/run.sh --out DIR --engine here \
  --ship tools/decision-audit/continuations/economic.json \
  --ship tools/decision-audit/continuations/conquest.json SAVE -- \
  --window 10 --salts 32 \
  --continue hard,params:audit-in/economic.json,params:audit-in/conquest.json --memory-reset
tools/decision-audit/report.py DIR [PART_DIR ...] [--rank cities]
tools/decision-audit/summary.py --metric cities --horizon 20 Mongols=DIR,PARTS Swiss=DIR,PARTS ...
```

A self-check passes: feeding his own recorded orders back as the `orders:` source gives zero differences in every
category and exactly zero value. `--alt params:FILE` audits Hard with a candidate gene (a tape replays on `--engine here` only while the rules match
the recording; check `fidelity.json`). `--alt orders:FILE.jsonl` takes any other player's orders per turn
(`{"turn": T, "orders": [...]}`), such as an LLM's. `--continue params:FILE` changes who plays the seat after the branch.

For policy sensitivity, `--continue` takes a comma list. Every continuation plays the same sampled
turns, arms, horizons and salts; `value.jsonl` names the continuation on each row. The two fixed
challengers above use only existing AI decision genes; see their
[`README.md`](../../../tools/decision-audit/continuations/README.md) for the settings and rationale.
`--memory-reset` adds `hard:cold`: a Hard seat brain without the shadow's warmed opponent memory at
the branch turn. It still sees and plans that turn's observation before the recorded branch order
is substituted. Other AI seats retain their restored memory. The ordinary `hard` row remains warm.

With multiple continuations, `report.py --rank cities` ranks category shares by their smallest
mean cities advantage at the final horizon. It shows each continuation's paired 95% interval, the
spread between means, and `robust=yes` only when every interval excludes zero in the same direction.
The original value and largest-decision tables use the first continuation in the list. With just
one continuation, the report format is unchanged. `continuation_invalid_orders` in the raw value
rows and the multi-policy report counts orders rejected by validation while a continuation is
actually playing; replayed window orders are excluded because reseeded games may invalidate them.
The challengers need a legal-order and no-panic smoke check on a tape before interpreting a ranking.

## Cost

60 box jobs (diffs, smoke tests, the final runs and the robustness checks): about 50 core-hours on the shared box,
roughly $0.60 at its $0.73 an hour for 64 cores. A full game at window 10 takes 5 to 15 minutes, split over three box jobs.
