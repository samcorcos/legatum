# Distilled macro value experiment — 2026-10-08

The live Hard + Search seat cannot afford the 64 or more determinizations that stabilized the 8.5 Sparta T14 plan choice. This experiment trains a small view-only model offline, then uses integer arithmetic to score its plan and opening options. `learned_macro` defaults to **0 (off)**. Modes 1, 2, and 3 test both decisions, plans only, and openings only. The model is offered only in games with at least four players, matching the main training distribution. A plan pool containing Snipe or Hold still uses the original search.

## Labels and fit

`crates/sim/examples/macro_labels.rs` samples the human seat on each of the five benchmark maps and living seats in seeded Standard four- and eight-player all-AI games. The sampled turns are 1, 20, 40, 60, and 90. Each option gets 64 shared view-derived determinizations and a 40-turn rollout, except T40 gets 80 turns. Labels are the mean projected win chance with the existing search's beyond-horizon term. Opponents use scripted Hard. The second collection pass also preserves each seat's campaign memory and forces a review at the sampled turn. Neither fit sees hidden state at inference time.

`tools/sim/train-macro.py` fits pairwise option value differences with ridge regression, then quantizes ten plan/opening heads and faction biases into `i32` weights. `crates/ai/src/learned_macro.rs` uses only `PlayerView` fields: force, city and economy totals, titles and VP, threat, target distance, walls, defenders, and exposure. Its dot product is `i64`; no floats, randomness, file reads, or rollout runs in the live decision. The raw seed-split labels, trained fit reports, and the candidate params files are in this directory. Refit with NumPy from `tools/sim/requirements-macro.txt`, for example:

```sh
python3 tools/sim/train-macro.py \
  --train docs/sim-results/distilled-macro/raw/train.jsonl \
  --held-out docs/sim-results/distilled-macro/raw/held-out.jsonl \
  --source crates/ai/src/learned_macro.rs \
  --report docs/sim-results/distilled-macro/fit.json
```

The first collection used seeds 7301001–7301096 for fit and 7301097–7301128 for held-out scoring: 1,129 and 371 decision groups, respectively. All eight factions appear in both splits. The 371 held-out groups include 168 openings and 203 later plan reviews. The source and held-out seed sets do not overlap; benchmark layouts do recur across both splits and in the subsequent human-seat evaluation.

| Held-out choice, first fit | Model | First two of the oracle's 64 samples |
| --- | ---: | ---: |
| All choices, agreement with 64-sample best | 56.6% | 59.8% |
| Openings, agreement | 51.8% | 43.5% |
| Openings, mean oracle regret (hundredths of a percentage point) | 206.8 | 261.5 |
| Macro-only plan pools, agreement | 60.1% | 73.9% |
| Macro-only plan pools, mean oracle regret (hundredths of a percentage point) | 68.9 | 27.9 |

The first plan fit is a regression and is **not a successful plan replacement**. In 46 of 188 held-out macro plan pools, it chose Contain where the oracle chose Economy. The opening fit did better on held-out opening labels, but the opening win was checked against full games before considering it useful.

The memory-preserving second pass used seeds 7302001–7302096 for training and 7302097–7302128 for held-out scoring: 1,379 and 478 groups from another 128 games. Its 1,857 groups include 1,628 H40 and 229 H80 decisions, all at 64 samples, with every faction represented in both splits. Preserving the simulated seat's campaign memory did not make plan values predictable enough for live replacement:

| On the second pass's 302 held-out macro reviews and 168 openings | First-fit model | Second-fit model | First two samples |
| --- | ---: | ---: | ---: |
| Macro plan agreement | 58.9% | 59.9% | 72.8% |
| Macro plan regret (hundredths of a percentage point) | 62.0 | 71.0 | 34.2 |
| Opening agreement | 51.2% | 42.9% | 39.9% |
| Opening regret (hundredths of a percentage point) | 171.4 | 214.6 | 291.4 |

The first-fit model's second-pass scores are cross-dataset checks; its own training seeds do not overlap this held-out set. The trained Rust weights therefore remain those of the first fit. The second-fit weights were evaluated offline and not installed. Comparing models on this held-out set is exploratory model selection, not a fresh untouched test.

## Full-game checks on the first fit

These were cloud runs on branch base `739d1429`, with the same salts for each candidate. The hard-vs-human run is 64 salts per benchmark map (320 games). The proxy7 duel is 1,024 paired Standard Small games. Control is Hard + Search with its existing in-app budget. The plan and opening candidate modes here still affected two-player games; a later guard restricts them to four or more players.

| Hard-vs-human, first fit | Overall wins | Switzerland | Sparta | Japan | Celts | Mongols | Milestone ratio |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Control | 43.13% | 0/64 | 1/64 | 60/64 | 35/64 | 42/64 | 0.708 |
| Both | 47.19% | 0/64 | 22/64 | 61/64 | 35/64 | 33/64 | 0.708 |
| Plans only | 40.31% | 0/64 | 0/64 | 61/64 | 35/64 | 33/64 | 0.708 |
| Openings only | 46.88% | 0/64 | 13/64 | 60/64 | 35/64 | 42/64 | 0.708 |

The opening-only candidate changed twelve Sparta losses to wins and no other map's win outcomes, a paired overall +3.75 percentage points (salt bootstrap 95% interval +1.88 to +5.94). It still failed to match the human's Sparta pace: its median winning turn was 253 versus the human's 61. The both-mode gain was not as clean: Sparta gained 21 wins, while Mongols lost 12.

| Proxy7 duel, first fit | Score | Paired change from control | Search-seat p95 think time |
| --- | ---: | ---: | ---: |
| Control | 35.40% | — | 95 ms |
| Both | 34.08% | −1.32 points (95% CI −4.02 to +1.38) | 1 ms |
| Openings only | 34.42% | −0.98 points (95% CI −3.59 to +1.64) | 77 ms |

With the final four-plus-player guard, opening-only again scored **46.88%** on the same 320 human-seat games. Its Switzerland milestones reverted to control and its other per-map results stayed as above. On the 1,024-game proxy7 duel it had **the same outcome in every paired game as control**: 35.40% score, zero changed outcomes, search-seat p95 94 ms versus control's 95 ms. The original two-player regression was therefore removed by the guard. This guard was chosen after seeing the first duel results, so this paired duel is a confirmation of the guard, not an independent selection test.

The timing of one model inference on a native release build at a T60 eight-player view with 11 military units and 8 cities was 261 ns per option (measured with `macro_inference_time`). This is a local microbenchmark of scoring, not the full turn's wall time.

## Reproduction and limits

Build the current, memory-preserving collector with `LEGATUM_CLOUD_EXAMPLES=macro_labels CARGO_BUILD_JOBS=4 tools/sim/build-slot.sh -- tools/cloud/build-linux.sh`, then run `tools/cloud/run-batch.sh --max-hours 2 --bin macro_labels -- --games 128 --seed 7302001 --samples 64 --label macro-data-warmed`. The first collection used the earlier collector without a memory-preserving review copy; its checked-in raw labels reproduce the installed fit, while this command reproduces the second method. `tools/cloud/status.sh` checks for running servers after each batch. For the five-map benchmark use `tools/sim/hard-vs-human.sh --cloud --brain hard+search --salts 64 --ai-params docs/sim-results/distilled-macro/opening-only.json --label distilled-macro-opening-only`.

The historical 8.5 T14 save was re-examined with `deep_search --mode model`, but the current branch replay has 14 hash mismatches from that older tape. Its model favored Campaign (5003 against Economy 4766) in the approximate position; this is a diagnostic, not an exact replay validation. The horizon labels use a projected win chance, not an observed final win. Same-layout benchmark training and evaluation and the weak plan fit limit generalization claims.

After the relative-value merge, new `macro_labels` rows inherit `winchance` version 5 through the search rollout means: projected title transfers now move VP from the old holder to the new one. The checked-in raw labels and installed model weights were fit with version 4 and have not been regenerated or revalidated under version 5. `learned_macro` remains off by default; retrain and benchmark before enabling it.

The 16 task batches, including failed pilots and both data collections, used an estimated **1.408 server-hours, about $1.00** at $0.7093 per hour including IPv4 ([batch ledger](cost.csv)). This is a launch-price estimate with a one-minute floor per batch. The AWS invoice is not available to the launcher. The final status check after the warm batch showed its server stopped; another agent's unrelated batch was running.

## Fresh-map opening-only validation — 2026-10-08

This holdout tests the opening-only setting (`learned_macro=3`) against the same Hard + Search policy with only that gene set to zero. All seats use `SearchConfig::in_app`, Hard difficulty, Standard speed, and the Standard map setup. For every map seed, both mirrored seat assignments were played. Thus each map contributes four gene-on and four gene-off seat results in 4-player games, and eight of each in 8-player games. The faction schedule is balanced exactly: in 4-player games each faction appears in 256 seats per arm; in 8-player games, 512 per arm. The treatment rotates by seat and map, with the reverse assignment on the mirror.

The 512 4-player map seeds are `9000000000000000000..9000000000000000511`; the 512 8-player map seeds are `9000000000000001024..9000000000000001535`. They are separate from each other and from the training/benchmark seeds recorded in `docs/sim-results`. The proxy7 duel uses the separate paired-seed range `9000000000000002048..9000000000000002559`. A scan of recorded JSON/JSONL seed fields found no overlap. Results are stored in `raw/fresh-4-player.json.gz` and `raw/fresh-8-player.json.gz`; the nine-seed terminal-state rerun is `raw/fresh-8-terminal-diagnostic.json.gz`.

The cohort binaries were built from `02591f27`, on the task-start master base `e4e7d9d0`. Master advanced to `d56536b9` while the cohort was running; that commit was merged into this work branch afterward. Following the task's merge instruction, the repin/determinism checks were rerun on the merge. The statistical results below therefore measure the task-start baseline and are not a post-`d56536b9` comparison. The fmt, clippy, and workspace test results in this branch were run on the merged tree.

Intervals below are 95% percentile intervals from 20,000 bootstrap resamples of paired map-seed clusters. For the 8-player win rate, nine games reached the harness's 1,000-turn cap without ending. A targeted rerun of those nine seeds in both mirrored assignments confirmed that exactly those nine games were unfinished; their win outcomes are excluded. Their completed mirrors remain in the paired sample. The other 1,015 8-player games finished. City and VP observations remain usable regardless of terminal status. T100 was not reached by 112 of 4,096 seats in 4-player games and 128 of 8,192 seats in 8-player games.

| Players | Measure | Map seeds | Gene off (95% CI) | Gene on (95% CI) | On − off (paired 95% CI) |
| ---: | --- | ---: | ---: | ---: | ---: |
| 4 | Win rate | 512 | 25.34% (24.22–26.51) | 24.66% (23.49–25.78) | −0.68 pp (−3.03 to +1.56) |
| 4 | Cities at T50 | 512 | 3.04 (3.00–3.08) | 2.86 (2.83–2.89) | −0.18 (−0.23 to −0.13) |
| 4 | VP at T100 | 507 | 4.02 (3.96–4.09) | 3.94 (3.88–4.00) | −0.08 (−0.19 to +0.03) |
| 8 | Win rate | 512 | 11.13% (10.52–11.74) | 13.87% (13.26–14.48) | +2.73 pp (+1.51 to +3.96) |
| 8 | Cities at T50 | 512 | 2.28 (2.25–2.31) | 2.39 (2.37–2.42) | +0.11 (+0.08 to +0.15) |
| 8 | VP at T100 | 512 | 2.59 (2.55–2.63) | 2.78 (2.75–2.82) | +0.19 (+0.12 to +0.26) |

The gain is specific to the 8-player holdout. Four-player wins are statistically flat, while cities at T50 are lower with the gene. This does not support enabling one global default, so `learned_macro` remains **off (0)**. The 4-player and 8-player results should be treated as separate contexts; they are not pooled into an overall rate.

The requested 1,024-game Standard Small proxy7 duel used the opening-only parameter file and `SearchConfig k=3,h=20,d=4`. The search seat scored **30.96%** (317 wins; no-winner/stalled games count as non-wins; 512 paired seeds; 95% seed-cluster bootstrap CI 27.73–34.18). However, opening-only mode is deliberately guarded off in lobbies with fewer than four players. The duel therefore checks that the two-player path remains unchanged; it is not evidence for the gene's active performance against proxy7. Raw results: `raw/fresh-proxy7-duel.json.gz`.

The three main cloud batches used an estimated **0.63 server-hours and $0.44** at launch spot prices; the nine-seed terminal diagnostic used **0.10 server-hours and about $0.07**. Combined estimated cost is about **$0.51**, under the $3 limit; the AWS invoice is authoritative. Each batch used `<launcher-profile>` with a one-hour cap, followed by `tools/cloud/status.sh`. Other users' active goal-first batches were left running.
