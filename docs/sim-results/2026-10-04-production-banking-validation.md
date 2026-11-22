# Production banking validation (2026-10-04)

This validation combines production banking with the latest master batch (simulation code `0a4a802e`), including the Byzantium limiter, terrain sight/Celts resync, Sparta redesign, and Japan/Swiss tuning. The branch banks Production per item, requires population 2 before starting a Settler, makes the first growth cost 10 Food, hides faction-useless items, and prices the first empire Settler at 65%. These banking rules were approved by Sam; the discount was selected from the measurements below.

## Opening pacing

A 48-game, 60-turn, eight-player Hard probe on seed 5200001 measured first Settler / second city medians on the combined batch: at 75%, T18/T25 Standard and T16/T23 Quick; at 70%, T17/T24 and T16/T22; at 65%, T17/T23 and T15/T22. The 65% price met all four by-turn targets. The full 400-game result follows:

| Speed | Target first / second | Master first / second | Banking first / second | Master win ratio | Banking win ratio |
|---|---:|---:|---:|---:|---:|
| Standard | 18 / 25 | 18 / 27 | 16 / 24 | 15.38× | 7.53× |
| Quick | 15 / 22 | 15 / 24 | 14 / 22 | 9.62× | 12.00× |

Each speed cohort contains 400 eight-player Hard games on seed 5200001. Medians use seats with a recorded completion (completion counts vary by faction); wins use all 400 seats per faction. The ratio is the highest faction win rate divided by the lowest. The four pacing targets are first Settler by T18 and second city by T25 Standard, by T15 and T22 Quick. Founding also depends on travel and site choice.

## Per-faction results

Cells show **master → banking**. First and second are median turn; win columns are percent of games won. Each speed has 400 seats per faction; proxy v6 rotates across 50 games per faction.

| Faction | Standard first | Standard second | Standard win % | Quick first | Quick second | Quick win % | v6 proxy win % |
|---|---:|---:|---:|---:|---:|---:|---:|
| Japan | 18 → 18 | 24 → 25 | 6.5 → 6.8 | 14 → 16 | 21 → 23 | 10.2 → 9.0 | 20.0 → 32.0 |
| Germany | 18 → 14 | 28 → 23 | 14.0 → 12.5 | 15 → 13 | 25 → 22 | 11.0 → 11.0 | 14.0 → 20.0 |
| Mongols | 18 → 14 | 31 → 25 | 30.8 → 28.2 | 15 → 13 | 26 → 23 | 19.2 → 27.0 | 66.0 → 76.0 |
| Sparta | 18 → 19 | 26 → 25 | 2.0 → 3.8 | 15 → 16 | 23 → 24 | 2.0 → 2.2 | 12.0 → 14.0 |
| Byzantium | 18 → 20 | 28 → 26 | 13.8 → 14.2 | 15 → 17 | 23 → 23 | 19.2 → 17.0 | 0.0 → 4.0 |
| Athens | 18 → 14 | 26 → 21 | 12.5 → 10.5 | 15 → 13 | 23 → 20 | 9.5 → 10.5 | 20.0 → 24.0 |
| Switzerland | 18 → 15 | 26 → 22 | 8.8 → 8.2 | 15 → 13 | 24 → 21 | 11.5 → 7.0 | 22.0 → 28.0 |
| Celts | 18 → 14 | 26 → 21 | 11.8 → 15.8 | 15 → 13 | 23 → 20 | 17.2 → 16.2 | 14.0 → 24.0 |

Faction medians later than the aggregate targets: Standard first Settler: Sparta T19, Byzantium T20; Standard second city: Byzantium T26; Quick first Settler: Japan T16, Sparta T16, Byzantium T17; Quick second city: Japan T23, Mongols T23, Sparta T24, Byzantium T23. The Quick faction win ratio rises on banking even though its aggregate pacing meets target.

## Proxy v6 and scenario

Proxy v6 won 84/400 (21.0%) on master and 111/400 (27.8%) on banking: **+6.8 percentage points for the proxy**. On matched seeds the proxy won only on banking in 64 games and only on master in 37 (both 47, neither 252). A higher proxy win rate means Hard opponents were easier for this benchmark. Per-faction proxy rows have only 50 games each, so small differences are noisy.

The re-recorded `scenarios/opening-race.json` Hard baseline (50 runs on its fixed starting state) has median score 340, p10 240, p90 350, and meets the three-city-by-turn-50 objective in 25/50 runs (50%). Its state hash remains `5936678055640939814`.

## Reproduction and limits

Master results were retrieved only through `tools/sim/cached-baseline.sh -- <args>` from matching simulation code cache `code-85f39d9eb541efc9`; the branch cohorts ran through `tools/sim/slot.sh` with six threads. Standard and Quick use `--players 8 --difficulty hard --games 400 --seed 5200001`. Proxy v6 uses `--proxy 0 --proxy-version 6 --proxy-start peace --rotate-proxy --legacy-hard all --difficulty hard --barbarians true --speed standard --games 400 --seed 3610001`. Raw final branch reports are under `~/LegatumRuns/production-banking/v2/batch/final/runs/`; the probe scripts are under `~/LegatumRuns/production-banking/v2/`; master reports remain in the baseline cache.

The branch report headers carry different commit labels because the release binary was rebuilt while merge, fixture, and documentation commits were being finalized. The Standard report labels `d1323627` with a dirty worktree, Quick labels `34a59fdf` with a dirty worktree, and v6 labels clean `1506501c`. These labels are build-time provenance, not claims that those committed trees contain the final combined rules. Replaying each cohort's first seed against the current release binary produced the identical game record aside from elapsed wall time; the replay reports are under `~/LegatumRuns/production-banking/v2/batch/final/provenance/`. No simulation-source file changed after the combined-code build.

The 65% price is the first tested setting on the combined batch that met all four probe pacing targets. The full-cohort result and v6 result are measured above. v6 is an AI-vs-proxy benchmark and does not by itself measure a human game.
