# Hard Sparta's conquest opening still pays under city upkeep, at a smaller gain

2026-11-27. The follow-up `26-sparta-conquest-opening.md` asked for: its numbers were measured on rules 62, before city upkeep (rules 63). Same runner and arms, one Sparta seat against five built-in Hard AIs, the opening genes off and on, paired seed for seed:

```sh
cargo build --release -p legatum-sim --example offense_ladder
./target/release/examples/offense_ladder \
  --arm off:sparta_rome_opening=0,sparta_opening_ready=0,sparta_opening_search=0 --arm on \
  --games 300 --seed 1001 --players 6 --faction Sparta --out sparta-opening.jsonl
```

Run at `9abb1d31` (rules 62), `0c5d2e8f` (rules 63), `5f98e07d` (rules 64), `025325a4` (rules 64, after Hard Egypt's peace growth), `82a0e544` (rules 66: after India's Revenue Offices, `sparta_chain_opening` 1 and Byzantium's Buy a city) `90e6ae07` (rules 67: after `modern_army` 2, the delegated city economy and research queues that span eras) and `7c9bb07b` (rules 67, the newest measured: after the Hard AI's purchasing and reserve caps were removed), and at the last four again with `--seed 2001` as held-out seeds. Paired differences are on minus off with a 95% interval.

| | Wins off | Wins on | Paired | VP share off | VP share on | Paired | Game ends (on) |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Rules 62, seeds 1001-1300 | 22.3% | 37.0% | +14.7 ± 6.4 | 0.183 | 0.305 | +0.122 ± 0.033 | turn 229 |
| Rules 63, seeds 1001-1300 | 16.3% | 21.7% | +5.3 ± 6.0 | 0.168 | 0.216 | +0.047 ± 0.022 | turn 267 |
| Rules 64 at `5f98e07d`, seeds 1001-1300 | 13.7% | 23.0% | +9.3 ± 5.6 | 0.166 | 0.221 | +0.055 ± 0.021 | turn 269 |
| Rules 64 at `025325a4`, seeds 1001-1300 | 9.3% | 18.3% | +9.0 ± 5.3 | 0.139 | 0.206 | +0.067 ± 0.021 | turn 264 |
| Rules 64 at `025325a4`, seeds 2001-2300 | 8.0% | 16.7% | +8.7 ± 4.8 | 0.120 | 0.187 | +0.067 ± 0.020 | turn 271 |
| Rules 64 at `025325a4`, all 600 | 8.7% | 17.5% | +8.8 ± 3.6 | 0.129 | 0.196 | +0.067 ± 0.014 | turn 267 |
| Rules 66 at `82a0e544`, seeds 1001-1300 | 8.3% | 17.7% | +9.3 ± 5.2 | 0.120 | 0.195 | +0.075 ± 0.021 | turn 263 |
| Rules 66 at `82a0e544`, seeds 2001-2300 | 8.0% | 13.7% | +5.7 ± 4.6 | 0.121 | 0.183 | +0.061 ± 0.020 | turn 267 |
| Rules 66 at `82a0e544`, all 600 | 8.2% | 15.7% | +7.5 ± 3.5 | 0.121 | 0.189 | +0.068 ± 0.015 | turn 265 |
| Rules 67 at `90e6ae07`, seeds 1001-1300 | 8.7% | 13.7% | +5.0 ± 4.9 | 0.118 | 0.197 | +0.078 ± 0.022 | turn 257 |
| Rules 67 at `90e6ae07`, seeds 2001-2300 | 5.0% | 18.3% | +13.3 ± 5.0 | 0.118 | 0.188 | +0.070 ± 0.022 | turn 264 |
| Rules 67 at `90e6ae07`, all 600 | 6.8% | 16.0% | +9.2 ± 3.5 | 0.118 | 0.192 | +0.074 ± 0.015 | turn 261 |
| Rules 67 at `7c9bb07b`, seeds 1001-1300 | 7.7% | 19.3% | +11.7 ± 5.2 | 0.128 | 0.198 | +0.070 ± 0.022 | turn 263 |
| Rules 67 at `7c9bb07b`, seeds 2001-2300 | 12.0% | 13.3% | +1.3 ± 4.8 | 0.134 | 0.184 | +0.050 ± 0.023 | turn 262 |
| Rules 67 at `7c9bb07b`, all 600 | 9.8% | 16.3% | +6.5 ± 3.6 | 0.131 | 0.191 | +0.060 ± 0.016 | turn 263 |

The opening itself is untouched by upkeep, which starts at the seventh city: through turn 50 the rules 62, 63 and `5f98e07d` runs are the same games (declared by turn 30 in 78% of games against 33%, a capital taken by turn 50 in 71% against 15%, a rival eliminated by turn 50 in 54% against 6%, 1.41 captures by turn 50 against 0.49). At `025325a4`, 600 games: declared by turn 30 in 82% against 41%, a capital by turn 50 in 73% against 20%, a rival eliminated by turn 50 in 56% against 8%, 1.47 captures against 0.59. At `82a0e544`, 600 games: 83% against 42%, 73% against 20%, 57% against 8%, 1.61 captures against 0.59. At `90e6ae07`, 600 games: 83% against 42%, 73% against 20%, 57% against 8%, 1.61 captures against 0.59. At `7c9bb07b`, 600 games: 83% against 41%, 72% against 20%, 57% against 8%, 1.67 captures against 0.60.

What upkeep changed is the conversion of that lead into a win. With the opening on, Sparta on rules 62 ended with 10.3 cities and lost 3.3 along the way; at `025325a4` it ends with 7.0 and loses 5.2, and at `82a0e544` with 6.9 and 5.2, at `90e6ae07` with 6.9 and 5.1, and at `7c9bb07b` with 6.8 and 5.3; games run about 35 turns longer. The win gain fell from +14.7 to +8.8 points (rules 64), +7.5 (rules 66), +9.2 (rules 67 at `90e6ae07`) and +6.5 (rules 67 at `7c9bb07b`), and the VP share gain from +0.12 to +0.06; both are still clear of zero on 600 games at each commit, so the opening genes stay at their defaults. At `7c9bb07b` the win gain is the smallest measured and rests on the first seeds: the held-out seeds alone show +1.3 ± 4.8 win points, with the VP share gain still clear (+0.050 ± 0.023). Between the two rules 67 commits Sparta without the opening went from 6.8% to 9.8% wins and Sparta with it stayed where it was (16.0% to 16.3%). The rules 63 run alone (+5.3 ± 6.0) would not have shown it; rules 63 and 64 differ only in the Upgrade path, and that is enough to reshuffle the late game seed by seed. Hard Egypt's peace growth (`egypt_peace_army`, `a0f44d9f`) took about four more points of wins from Sparta with the opening off and on alike (seeds 1001-1300: 13.7% to 9.3% off, 23.0% to 18.3% on) and left the gain where it was.

The two follow-up genes still do not earn a default. At `025325a4`, seeds 1001-1300, paired against the opening on: `sparta_second_war_from=30` takes more cities by turn 50 (1.67 against 1.51) and wins -1.3 ± 3.5 points; `sparta_war_army=6` wins -0.7 ± 4.6; both together -1.7 ± 4.8. At `82a0e544`, same seeds: `sparta_second_war_from=30` wins +1.7 ± 3.3 points with VP share +0.015 ± 0.011, which would pass the gate on those seeds alone, but on the held-out seeds it is +0.0 ± 3.1 wins and +0.003 ± 0.011 VP share (600 games: +0.8 ± 2.2 and +0.009 ± 0.008); `sparta_war_army=6` wins -1.7 ± 5.1; both together -2.7 ± 5.1. At `90e6ae07`: `sparta_second_war_from=30` wins +0.7 ± 2.9 points on seeds 1001-1300 and -1.7 ± 3.0 on the held-out seeds (600 games: -0.5 ± 2.1, VP share +0.002 ± 0.009); `sparta_war_army=6` wins +1.3 ± 4.6; both together +1.7 ± 4.9. At `7c9bb07b`: `sparta_second_war_from=30` wins -2.0 ± 3.2 points on seeds 1001-1300 and -0.3 ± 2.9 on the held-out seeds (600 games: -1.2 ± 2.1, VP share +0.002 ± 0.009); `sparta_war_army=6` wins -5.7 ± 4.3; both together -3.7 ± 4.7. More early conquest is not what Sparta lacks under upkeep; holding and paying for what it took after turn 100 is.

`sparta_chain_opening` (`27-sparta-offense-r2.md`, measured on rules 63) was the one default not yet measured on rules 64 or later. At `82a0e544`, 600 paired games, `--arm nochain:sparta_chain_opening=0` against the built-in default: the chain still takes more cities by turn 50 (1.61 against 1.45), and wins are 15.7% against 16.2% (-0.5 ± 2.2 points), VP share 0.189 against 0.186 (+0.003 ± 0.008). On rules 66 it is neutral rather than the +3.3 and +3.7 win points it measured on rules 63; nothing here says to turn it off, so it stays 1. At `90e6ae07`, 600 paired games, the same: 1.61 captures by turn 50 against 1.45, wins 16.0% against 15.7% (+0.3 ± 2.2 points), VP share 0.192 against 0.185 (+0.007 ± 0.008). At `7c9bb07b`, 600 paired games: 1.67 captures by turn 50 against 1.49, wins 16.3% against 14.7% (+1.7 ± 1.8 points), VP share 0.191 against 0.185 (+0.007 ± 0.008); still inside the interval, so it stays 1.

`offense_ladder` prints `capital50` as the mean number of capitals taken by turn 50 (0.74 on rules 62), not the share of games with one (71%); the 74% in the 26 November entry is that mean.
