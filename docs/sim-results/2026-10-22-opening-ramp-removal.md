# Opening ramp removal and the early price retune (2026-10-22)

User decision 2026-10-21: "delete any leftover machinery and fix the tech debt". The opening ramp (`economy::ramp_price`) phased the old speed surcharge in over the first third of the game: every Production price was half the listed price on turn 0 and the full price from turn 100. It is gone. A production price is the listed price; the early prices were retuned so the opening pacing targets still hold.

## Method

8-player Hard, 400 games, seed 5200001, full length (the method of `2026-10-02-pacing-study.md` and `2026-10-04-production-banking-validation.md`): `legatum-sim --players 8 --difficulty hard --games 400 --seed 5200001`, medians over empires that reached the milestone. Targets: first Settler about T18, second city about T25. Candidate prices were first screened with 48- and 400-game 60-turn probes (a probe-only `LEGATUM_TUNE` build, not committed): plain prices with every unit at its listed price and the early buildings at 50% reached T18/T23; units at 50-70% did not help the opening and made later wars cheaper, so units stay at their listed prices.

## Prices

| Item | Before (listed; the ramp halved it on turn 0) | After |
| --- | --- | --- |
| Settler | 150, +40 per trained Settler | 70, +50 per trained Settler |
| Granary, Brood Hall | 100 | 50 |
| Workshop, Market | 120 | 60 |
| Library (`Laboratory`) | 160 | 80 |
| Walls (Swiss Alpine Redoubt) | 120 (160) | 60 (80) |
| Athens Lyceum | 140 | 70 |
| Farm, Pasture, Quarry / Mine | 60 / 80 | 30 / 40 |
| Units, later buildings, Roadworks, Great Works, techs | unchanged | unchanged |

`settler_escalation` (experiment) now adds 100 more per trained Settler (it added 80). The removed experiment `early-military-discount` and the no-op flags `opening-ramp` and `capital-production` are gone with the ramp.

## Pacing, before and after (400 games each)

| Prices | 1st Settler (median) | 2nd city | 1st Soldier | Mean game turns | Mean cities | Best / worst win share |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| before (ramp, Settler 150+40) | 18 | 25 | 30 | 309 | 4.95 | Byzantium 26.0% / Japan 3.8% |
| Settler 80+50 | 19 | 26 | 34 | 292 | 5.04 | Byzantium 29.5% / Japan 2.0% |
| Settler 75+50 | 19 | 25 | 33 | 286 | 5.06 | Byzantium 32.2% / Celts 3.0% |
| Settler 65+50 | 17 | 24 | 32 | 277 | 5.07 | Byzantium 27.0% / Japan 1.8% |
| Settler 70+50 (chosen) | 18 | 25 | 32 | 285 | 5.05 | Byzantium 31.8% / Japan 2.0% |

Settler 70+50 is the first setting at 18/25 with the base price a round number between 65 (17/24) and 75 (19/25). Reading:

- The opening holds: first Settler T18 and second city T25 before and after.
- Games are about 8% shorter (309 to 285 turns on average): the early buildings now stay cheap all game where the ramp made them full price from turn 100. The first Soldier is built a little later (T30 to T32): a Spearman is 80 from turn 1 where it was 40.
- One of the 400 games (seed 5200142) hit the 1,200-turn safety cap under the chosen prices; none did before and none under the 65, 75 and 80 settings (0.25%, not chased).
- Faction win shares move within the sampling noise of 400 games (Byzantium 26% to 32%, Japan 3.8% to 2.0%: eight wins against fifteen). Pillar 8 balance is not measured here.

## Human-seat benchmark (Hard, 8 maps x 128 reseeded salts, every seat gets the change)

Overall score 0.380 [0.352, 0.403] on the control (master after the speed-layer removal, `36ad0b462`) and 0.354 [0.328, 0.381] with the ramp removed and the early prices retuned (`ecc2c952e`): a change of about -2.5 points with overlapping intervals; the milestone ratio is 0.45 on both. Per map win rate, control to change: Switzerland 8.3 14.1% to 7.0%, Sparta 8.4 52% to 31%, Japan 8.4 52% to 50%, Celts 8.4 10% to 18%, Mongols 8.5 1.6% to 9.4%, Switzerland 8.5 82% to 69%, Germany 8.6 37% to 33%, Sparta 8.6 55% to 66%. Rows are in `docs/sim-results/hard-vs-human/ledger.jsonl` (the control row first).

## Re-measured after merging master (era objectives step 1)

The branch merged master's era-objectives work before landing; the same 400 games on the merged tree: first Settler T18, second city T25, first Soldier T33, 299 mean turns (era objectives change game length too, so this is not comparable with the 285 above), 399 VP wins and one stalled game (seed 5200372; the unmerged tree stalled seed 5200142 instead, so the stall is a rare game-dependent event, not one seed).
