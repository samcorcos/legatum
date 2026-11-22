# Raze and buys: two genes from the decision audit (2026-10-31)

Branch `worktree-agent-ae3fb8d35315eaaf2`. Follow-up to `decision-audit.md`, which found two narrow Hard mistakes in the
designer's seat: razing captured cities (+0.11 cities per 10-turn window) and production and buys (+0.13 cities, +0.15 VP
@50). Both fixes are AI decisions only, behind genes that default off. Runner: `raze-buys/run.sh`, arms in
`raze-buys/params/`, logs in `raze-buys/out/`. Census tool: `crates/sim/examples/raze_buy_census.rs`. Everything ran on
the cloud box.

**Verdict.** `raze_value` 1 stops the razes and closes the raze gap in the designer's seat; all-AI games keep 72% of
captured cities instead of 60% and end with +0.8 cities a game. `buy_hold` 3 cuts Hard's purchases in his seat from 184 to
68 (Mongols) and 99 to 71 (Swiss) and shrinks his production-and-buys lead from +0.12 to +0.02 cities @20 (mean of the two
games). The Gold it keeps goes to door defense, rescues and upgrades. The war suite doesn't move. Both are ready for the
lead's gate as `raze_value` 1 and `buy_hold` 3, alone and together.

## 1. Why Hard razes

`production.rs::plan_raze` (called from `scripted.rs` right after `plan_cities`) razed whenever the visible pressure on a
captured city exceeded 130% of the troops within 2 hexes (`raze_pressure_percent`) and the loot plus the "denied" value
beat the retained value. Two things made that nearly automatic:

- **The city was valued at almost nothing:** `city_keep_value` is 75 + 20 x population, with no yields, growth or VP. A
  population-3 city was worth 135, the same scale as its 30 Gold of loot plus a denial term.
- **The loss chance was the raw pressure ratio** `(pressure - defense) / pressure`. A captured city has just lost its
  garrison, so any nearby army reads as a near-certain loss, whether or not it can reach the city this turn.

All-AI census (512 games, master): 6.58 ordered razes a game out of 16.44 captures, so Hard burns 40% of the cities it
takes (Sparta 2.42 a seat, Mongols 2.02). No faction rule forces a raze except the Celtic sack, which is a capture rule
(`battle.rs::capture_city`), not this order; it is unchanged (2.3 sacks a game in every arm).

**Gene `raze_value` 1** (`crates/ai/src/raze.rs`). Keep or raze by value, from public information and our own cities:

- **Worth of keeping:** the city's output over 40 turns (less its resistance turns), estimated from our settled cities'
  yield per population (a captured city yields nothing during resistance), plus growth (one population per 15 turns),
  plus 300 Gold for each live "gain cities" objective, minus the extra Decadence its output adds to our lead
  (`rule_experiments.decadence_percent` before and after, on public Power).
- **Keep arm:** the better of defending it (Gold for enough of our cheapest buyable defender to cover the pressure gap,
  only if affordable within 5 turns of income) or risking it: a loss chance of 90% if the visible attack is forecast to
  take it this turn (`defense_risk`), else the unmatched share of the pressure times 75% (an attack can reach it now) or
  40% (not yet); a lost city goes back to its former owner, counted at half its worth.
- **Raze arm:** population x 10 Gold. Raze only when that beats the keep arm.

## 2. What Hard buys

Census, master, per game (8 seats; `buy_trace` tags each purchase with the pass that ordered it):

| Pass / item | Buys | Gold | Turns the city would need | At war |
|---|---:|---:|---:|---:|
| routine improvements (`plan_buy`) | 53.1 | 3,777 | 4.3 | 57% |
| readiness defenders (`buy_ready_defenders`) | 20.9 | 3,652 | 7.2 | 100% |
| routine buildings (`plan_buy`) | 21.2 | 3,235 | 9.2 | 67% |
| frontline defenders (`buy_frontline_defender`) | 14.6 | 2,530 | 7.4 | 89% |
| door defense | 6.1 | 1,086 | 10.0 | 100% |
| emergency rescues | 2.9 | 631 | 14.0 | 94% |
| everything else (surplus army, victory, threat, goals, settlers) | 4.2 | 739 | | |

- Every purchase pays 200% of the Production cost (175% Swiss): it is a full-price copy, so the price relative to
  Production is fixed and the only question is timing. Improvements are bought when the city would finish them in
  4 turns, buildings in 9: routine buys trade Gold for a few turns of Production.
- Military buys rise through the game (T100-150 2,242 Gold a game, T150-200 2,709); improvements are bought most in the
  first 50 turns (1,006 Gold).
- Per seat: 15.4 purchases and 1,956 Gold, 17.1 upgrades for 1,202 Gold. Median treasury 71 at T100, 79 at T150.
- In the designer's seat (diff of the audit) Hard's 184 Mongols purchases are 108 units (34,420 Gold) and 71 improvements;
  the Japan game's 396 are 286 Soldiers bought at war.

**The passes, audited.** Emergency rescue, door defense and threat defense buy for a city that can be attacked now.
Readiness buys the best unit per Gold in every city whose visible threat exceeds 80% of the troops nearby, every turn,
before upgrades get the Gold. Frontline buys one defender where an enemy is within 5 hexes of a thinly held city (the 10-30
fix holds its price instead of ending all buying). Routine `plan_buy` ranks buildings, improvements and Settlers by value
per Gold above the ordinary reserve. Nothing in any of them prices what the Gold would buy later.

**Gene `buy_hold`** (`crates/ai/src/gold_hold.rs`, used in `plan_buy`, `buy_ready_defenders`, `buy_frontline_defender`).
The routine passes spend only the Gold above its later uses:

- **Level 1:** the upgrade bill for units whose next form is researched or due within 10 research turns, plus, at war or
  under threat, the cheapest defender quote for each threatened city (one at war with nothing in sight, two at most).
- **Level 2:** readiness and frontline defenders also leave it unless an attack can reach the city this turn.
- **Level 3:** the hold also banks the army spike, the dearest unit we can buy now for each of up to four cities.

The hold never exceeds 25 turns of income, so a treasury can't freeze (the Swiss hoard lesson, 10-30). Upgrades
(`plan_modernization`), rescues, door defense, threat defense, victory, faction goals and the hoard rules don't see it, so
the kept Gold is spent where it pays: at the spike and in an emergency.

## 3. Before and after

### Decision audit (his-minus-Hard, 10-turn policy windows, 32 salts)

`tools/decision-audit/run.sh --engine here` on today's master (`18e64fcef`/`b56fe4119`), `--alt params:ARM
--continue params:ARM`. The Mongols tape replays with 0 hash mismatches. The Swiss tape (8.9.15) drifts from about T150
(32 mismatched hashes by T180) because rules changed since. Today's AIs differ from the recorded ones from T81 (Mongols)
and T88 (Swiss), so the AI seats' memory there is today's Hard's own. Values are cities / VP, ± 95%.

| Arm | Hard's buys in his seat (M / S) | Hard's razes (M / S) | Production share @20 | Production share @50 | Raze ("other") share @20 | Raze share @50 | Whole policy @20 | Whole policy @50 |
|---|---|---|---|---|---|---|---|---|
| control | 184 / 99 | 4 / 8 | M +0.17 / +0.22, S +0.06 / +0.24 | M +0.10 / +0.20, S +0.12 / +0.11 | M -0.00, S +0.05 ±0.01 | M +0.02, S +0.06 ±0.02 | M +0.14 / -0.09, S +0.59 / +0.61 | M +0.11 / +0.11, S +0.49 / +0.41 |
| `raze_value` 1 | 188 / 103 | 0 / 0 | M +0.15 / +0.21, S +0.08 / +0.22 | M +0.10 / +0.15, S +0.13 / +0.11 | M -0.05, S 0.00 | M -0.03, S 0.00 | M +0.08 / -0.08, S +0.59 / +0.58 | M +0.09 / +0.06, S +0.49 / +0.41 |
| `buy_hold` 2 | 163 / 88 | 4 / 8 | M +0.12 / +0.09, S -0.00 / +0.23 | M +0.10 / +0.14, S +0.03 / +0.10 | | | M +0.09 / -0.24, S +0.55 / +0.61 | M +0.13 / +0.03, S +0.48 / +0.37 |
| `buy_hold` 3 | 68 / 71 | 4 / 8 | M +0.12 / +0.16, S -0.08 / +0.19 | M +0.13 / +0.14, S -0.02 / +0.15 | | | M +0.11 / -0.13, S +0.51 / +0.50 | M +0.17 / +0.07, S +0.39 / +0.35 |
| both (`buy_hold` 3) | 69 / 72 | 0 / 0 | M +0.15 / +0.15, S -0.06 / +0.19 | M +0.15 / +0.11, S -0.01 / +0.16 | M -0.03, S 0.00 | M -0.00, S 0.00 | M +0.12 / -0.11, S +0.52 / +0.49 | M +0.24 / +0.09, S +0.38 / +0.37 |

Mean of the two games, cities @20: production share +0.12 (control) -> +0.02 (`buy_hold` 3) -> +0.05 (both); raze share
+0.03 -> -0.03 (`raze_value`). VP @20: production share +0.23 -> +0.18 -> +0.17. The production gap at @50 shrinks less
(+0.11 -> +0.06 cities; VP +0.16 -> +0.15). Sampling noise is ±0.04 to ±0.09 per game.

- On today's master the Mongols gap is far smaller than the 8.9.17 Hard's (+0.14 cities @20 for the whole policy against
  +0.49): focus fire, the hoard and dead-war changes already moved Hard toward his play. The raze matters there only at
  T167-172 (4 orders).
- In the Swiss game the razes come at T161-180, in the stretch where the replay has drifted; the raze share goes from
  +0.05/+0.06 cities to exactly zero, since Hard now keeps the same cities he does.
- The Mongols "whole policy @50" reads worse with both genes (+0.11 -> +0.24, ±0.10 each); the production and raze shares
  don't, so it is the categories interacting or noise.

### War suite (64 runs, 33 cases)

| Arm | Seat | All seats |
|---|---:|---:|
| control | 42% | |
| `raze_value` 1 | 42% | 41% |
| `buy_hold` 1 / 2 / 3 | 41 / 42 / 42% | 41 / 42 / 41% |
| both, `buy_hold` 2 / 3 | 42 / 42% | 41 / 40% |

Both genes together on the seat, case by case: every case within one run of the control except
`war-mongols-chur-t137` 48 -> 45% (±16). Fixtures are 5 to 20 turns, too short for either decision to matter much.

### All-AI census (512 eight-empire Hard games, seed 7300001, paired by seed, ± 95%)

| Measure | control | `raze_value` 1 | `buy_hold` 1 | `buy_hold` 2 | `buy_hold` 3 | both (2) | both (3) |
|---|---:|---:|---:|---:|---:|---:|---:|
| Captures a game | 16.44 | 14.69 (-1.75 ±0.71) | 16.85 | 16.59 | 16.70 | 14.82 | 15.16 (-1.28 ±0.83) |
| Ordered razes a game | 6.58 | 4.19 | 6.63 | 6.65 | 6.72 | 4.07 | 4.18 (-2.40 ±0.39) |
| Captured cities kept | 60.0% | 71.5% | 60.6% | 59.9% | 59.7% | 72.6% | 72.4% |
| Cities, all empires, T100 | 30.76 | +0.28 ±0.11 | -0.32 ±0.20 | -0.33 ±0.20 | -0.50 ±0.25 | -0.08 ±0.20 | -0.18 ±0.26 |
| Cities, T150 | 35.22 | +0.48 ±0.17 | -0.02 | -0.08 | -0.06 | +0.37 ±0.27 | +0.26 ±0.27 |
| Cities, game end | 37.25 | +0.80 ±0.32 | +0.08 | -0.19 | -0.15 | +0.71 ±0.35 | +0.79 ±0.36 |
| Buy Gold a seat | 1,956 | 1,943 | 1,607 | 1,467 | 1,418 | 1,456 | 1,382 |
| Upgrade Gold a seat | 1,202 | 1,187 | 1,229 | 1,400 | 1,408 | 1,396 | 1,396 (+194 ±50) |
| Military Gold in the 10 turns after an era entry | 494 | 508 | 583 | 541 | 547 | 552 | 567 (+73 ±18) |
| Treasury median T100 / T150 | 71 / 79 | 71 / 79 | 78 / 86 | 85 / 111 | 84 / 113 | 84 / 107 | 82 / 110 |

Where the Gold goes with `buy_hold` 3, per game: door defense 1,086 -> 3,081 Gold, emergency rescues 631 -> 1,953,
readiness 3,652 -> 1,557, frontline 2,530 -> 915, routine improvements 3,777 -> 1,416, routine buildings 3,235 -> 922. Total
spending (buys plus upgrades) falls about 10% a seat (3,158 -> 2,826), so the treasury banks a little, and what it banks
is spent at the door, on rescues and on upgrades: the 10-27/10-30 failure (held Gold starving upgrades and attackers) did
not repeat. Captures hold with `buy_hold` (+0.26 ±0.87).

Fewer captures with `raze_value` is fewer recaptures: a kept city stands, a razed one can't be taken again, and every
empire ends with more cities. Wins by faction, control -> both (3): Sparta 119 -> 156, Mongols 152 -> 150, Athens 87 ->
80, Byzantium 59 -> 42, Japan 29 -> 22, Germany 23 -> 21, Switzerland 22 -> 22, Celts 20 -> 17.

### Decision cost

`war_trace` seat planning ms (runs 0-2, control and both genes at `buy_hold` 3, twice back to back on the box): Golden
Horde thin 3.9 / 4.5 / 3.6 -> 4.1 / 4.6 / 3.7, Chur T137 7.8 / 7.9 / 6.2 -> 8.5 / 7.9 / 6.4, Swiss tepid T220 3.4 / 3.7 / 3.7
-> 3.3 / 3.7 / 3.7, hollow Swiss T96 2.8 / 3.2 / 4.0 -> 3.5 / 4.5 / 6.0. Only the hollow Swiss case is slower, the same in
both passes, where the Swiss game itself plays out differently (more units). The hold is one pass over the seat's units and
quotes, cached for the turn; the raze value runs only for captured cities. The census's ms per seat-turn swings 3.5 to 8.3
with the box's load and doesn't measure the genes.

## 4. Risks

- **Faction balance:** with every seat keeping its conquests, Sparta's census wins rise from 119 to 156 of 512 and the
  defensive factions lose a little. The designer's seat gate is the measure; the faction spread is worth watching.
- **Early cities:** `buy_hold` costs 0.3 to 0.5 cities across all empires at T100 (fewer routine improvements and
  Settlers bought early); it is gone by T150.
- **The Swiss tape drifts after T150** on today's rules, which is where its razes are; the Mongols tape is exact.
- **`raze_value` still razes** 4.2 cities a game in all-AI play: cities the visible army can take this turn, or the last
  turns of a capped game. That is the intended call; whether the designer would keep those too is not in the audit.

## Reproduce

```
tools/cloud/remote.sh <ref> --env CENSUS_GAMES=512 --artifact docs/sim-results/ai-war/raze-buys/out -- \
  docs/sim-results/ai-war/raze-buys/run.sh census ctl raze buy3 both3
tools/cloud/remote.sh <ref> --artifact docs/sim-results/ai-war/raze-buys/out -- \
  docs/sim-results/ai-war/raze-buys/run.sh suite raze buy3 both3
tools/decision-audit/run.sh --engine here --out DIR --ship raze-buys/params/both3.json SAVE... -- \
  --window 10 --turns 10,40,70,100,130,160 --salts 32 --horizons 20,50 \
  --alt params:audit-in/both3.json --continue params:audit-in/both3.json
```
