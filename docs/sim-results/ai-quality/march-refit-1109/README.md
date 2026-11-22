# Modernise before marching: gene `march_refit`, 2026-11-09

**The gene code is not on master:** `march_refit` (`crates/ai/src/purchase_refit.rs` and its hooks) lives only on branch `claude/modernize-before-march`, left there because the measurement showed no gain. What was kept on master from this work: this directory, the Lab card `scenarios/lab/celts-parked-at-st-gallen.json` and the example `crates/sim/examples/parked_army_census.rs`. The rules-reference section the branch added (§10 "Modernise before marching") is also not on master.

**Status:** gene `march_refit`, **default 0 (off)**, measured and left off. Branch `claude/modernize-before-march` from master `916e35431`. Code: `crates/ai/src/purchase_refit.rs` (new), hooks in `production.rs::plan_cities` (the pass, before `plan_modernization`), the routine purchase reserve and `item_value`; `scripted.rs` (`refit_reserve`); `war_diagnosis.rs` (the parked read in `war_trace`). Rules reference: `game-rules.md` §10 "Modernise before marching". Source: Fable review 2026-11-09, `docs/status/fable-review-2026-11-09/3-human-gap.md`, reason 6 and change 2.

## The problem

In 8.9.29 a Celtic strike kept 7 of its 8 units within three tiles of unwalled St. Gallen (1 Spearman) from T99 to T114 and attacked 0 times: "capture at 6%, era deficit −80" (coach review `docs/playtests/reviews/2026-11-08-8.9.29-aztecs.md`). Once upgraded, Hard storms walled Therapne from the designer's own position 49 times in 50. He upgrades before he marches and spends Gold on units, not buildings, when a war is planned.

## What was built (one gene, three bits)

- **Bit 1, the parked army.** Two or more free combat units within 3 tiles of a city of an empire we have been at War with for 3+ turns, whose siege-stance storm forecast does not take it, price three options in Production per forecast turn (city 400 + 80 per population, less forecast losses, less Gold at 2 a Production): refit the group (most gain per Gold first, until the forecast with upgraded units takes the city; units upgrade only in own territory, so the turns home and back are charged), merge with every free unit that reaches the city within the approach window plus 2 turns, or retarget to another visible city of a war within 8 tiles that the group alone takes. The best positive option is taken. With none, the group's units inside the city's strike step back to a staging tile 3 or 4 tiles out.
- **Bit 2, Gold ahead of buildings.** While the plan fights a war (Campaign or Contain), funded upgrades are bought before routine purchases without `plan_modernization`'s comparison with the displaced optional purchase; Gold for the next upgrades reachable within 10 turns of income is held from routine building purchases.
- **Bit 4, era resources ahead of buildings.** While the plan fights a war, a building that would take the spare supply an army upgrade waits for scores 0.
- Existing pieces reused, no parallel system: `modernization.rs::refit` (gain and price of an upgrade), `return_for_modernization`, `opportunity.rs::forecast_siege_rounds` and `strike_force_within` (the stance's own forecast), `military.rs::campaign_staging_spot`, `campaign_approach`, `go_to`. No `military.rs`, `plan.rs` or `campaign.rs` code was touched.
- Off is hash-identical: 16 Hard 8-player games, master `916e35431` against the branch with the gene at 0, every turn hash equal.

## The parked-Celts card

`scenarios/lab/celts-parked-at-st-gallen.json`, cut from 8.9.29 T100 (Celts seat, at War with the Aztecs since T98, 30 Gold, +1 a turn; six Spearmen, a Phalangite, two Composite Archers and two Knights; St. Gallen has a Castle and one Spearman). The objective is to burn any Aztec city in 15 turns (Celts burn what they take, so "own St. Gallen" can't be met). Baseline 50 runs: 1/50 (`docs/sim-results/scenarios/celts-parked-at-st-gallen-baseline.md`).

100 paired runs per arm (`scen_pair.py`):

| Arm | Objective met | Mean score | Paired delta |
|---|---:|---:|---|
| off | 3/100 | 737.9 | |
| bit 1 | 0/100 | 732.4 | −5.5 [−20.2, +9.2] |
| bit 2 | 3/100 | 737.9 | 0 (identical runs) |
| bit 4 | 3/100 | 737.9 | 0 (identical runs) |
| 7 (all) | 0/100 | 732.4 | −5.5 [−20.2, +9.2] |

Why nothing helps here (`war_trace` with `WAR_TRACE_DIAG=3`): the refit option never prices positive. Six Spearmen and a Phalangite to Pikemen is about 1,000 Gold against 30 in hand and +1 a turn. The parked read fires from T101 and, with none of the three options positive, holds the group out of the city's strike; the retarget once found an unseen colony (a fogged city's forecast assumes one Spearman) and walked the group past walled Solothurn, which is why retarget now takes only cities in sight. In the card the Aztecs are a Hard AI with 363 Gold that buys Pikemen and Musketmen and kills the parked army (11 combat units to 3 or 4 by T107); the designer left it alone, so the card is harder than his game.

## 200 Hard games per arm

`legatum-sim --games 200 --seed 92000 --difficulty hard` (8 players, medium map), tapes replayed by `crates/sim/examples/parked_army_census.rs`. Per game, summed over the 8 seats. *Idle-army turns*: seat-turns with 3+ combat units within 3 tiles of a city of an empire at War with the seat and no attack by the seat's units that turn. Paired by seed, 95% intervals (`paired.py`, files `census-bit*.json`).

| Arm | Idle-army turns | Upgrades | Upgrade Gold | Captures | City attacks |
|---|---|---|---|---|---|
| off | 73.5 | 83.2 | 9,418 | 20.1 | 72.6 |
| bit 1 | +5.4 [+1.2, +9.6] | +1.8 [−0.8, +4.3] | +330 [−54, +714] | +0.6 [−0.9, +2.0] | −2.1 [−5.5, +1.3] |
| bit 2 | −0.4 [−2.4, +1.6] | **+1.7 [+0.4, +3.0]** | +185 [−22, +392] | +0.4 [−0.4, +1.1] | +1.7 [−0.4, +3.8] |
| bit 4 | +2.5 [−0.8, +5.8] | +1.3 [−0.7, +3.3] | +186 [−130, +502] | +0.7 [−0.5, +1.8] | +1.6 [−0.9, +4.2] |
| 7 (all) | +5.9 [+1.5, +10.2] | +2.4 [−0.2, +5.0] | +306 [−98, +711] | +0.1 [−1.5, +1.7] | **−4.0 [−7.5, −0.5]** |

Bit 1's idle-turn rise is mostly its fallback: holding three tiles out is idle by this metric (it is what the designer did at Therapne, but here no upgrade follows, because the Gold isn't there). Bit 2 is the only clean effect: more upgrades with nothing else moving.

## War suite

`war_suite --brain hard --runs 64`, gene on the seat only (`war-suite-compare.md`): suite mean −0.2 points; 16 of 19 cards unchanged, India–Sparta T142 −3.1 [−18.8, +12.5], Mongol bombers T150 +3.1 [−3.1, +9.4], Mongols Chur T137 −3.1 [−9.4, +1.6].

## Designer seat

48 salts per start turn, local, control = branch commit `db71bc110` (gene 0, same AI as master) against measurement-only commit `40a9859b1` (the same tree with the default at 7, so every Hard seat, his and the rivals', plays it). Files `designer-seat-control.json`, `designer-seat-7.json`, `designer-seat-compare.txt` (`ds_compare.py`; intervals combine the two runs' intervals).

| Set | Control | Gene 7 | Delta |
|---|---:|---:|---|
| Held out (Mongols 8.9.25, Celts 8.9.26, Swiss 8.9.24) | 58.1% | 55.8% | −2.2 pp [−5.4, +1.0] |
| Training (Aztecs 8.9.29, Germany 8.9.27, Sparta 8.9.26, Swiss 8.9.26, India 8.9.25) | 37.4% | 35.6% | −1.8 pp [−4.4, +0.7] |

No tape gains outside its interval; the largest moves are Swiss 8.9.26 −5.0 [−12.7, +2.7], Mongols 8.9.25 −3.8 [−8.6, +1.1] and Swiss 8.9.24 −3.3 [−9.4, +2.7]. Aztecs 8.9.29, the tape the Celtic case comes from, is flat (23.3% to 22.9%).

## Verdict

**No gain; leave the gene off.** Every measure is flat or leans negative: the card 3/100 to 0/100, captures flat in 200 AI games, the war suite −0.2, the designer seat −2.2 pp held out and −1.8 training (both intervals include zero, neither includes a gain above 1 pp). Only bit 2 does what it says, measurably (+1.7 upgrades a game), and it changes no outcome. The parked-army options rarely have anything to price: an obsolete army's refit costs several times what a Hard treasury holds, and a visible city it can take nearby is uncommon. What would bite is upstream: saving Gold and building the era's units *before* a war is declared (the plan pricing the refit as part of the Campaign's cost), which is `plan.rs`/`campaign.rs` territory. Bit 1's hold fallback also needs a reason to end (the upgrade arriving), or it only parks the army more safely.

## Reproduce

- Card: `legatum-sim scenario scenarios/lab/celts-parked-at-st-gallen.json --runs 100 --ai-params P.json --out DIR`, with `P.json` = `{"schema": 1, "decisions": {"march_refit": 7}}`; `python3 scen_pair.py DIR_OFF DIR_ON`.
- Census: `legatum-sim --games 200 --seed 92000 --difficulty hard --tape-out TAPES [--ai-params P.json]`, then `cargo run --release -p legatum-sim --example parked_army_census -- TAPES 14 > census.jsonl`; `python3 paired.py census-off.jsonl census-on.jsonl`.
- Designer seat: `tools/decision-audit/designer_seat.py --commit <branch tip>` against a measurement-only commit with the default at 7 (not on any branch).
