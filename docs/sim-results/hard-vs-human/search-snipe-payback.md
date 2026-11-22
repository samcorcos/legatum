# Search Snipe payback: default off

## Change

Added `decision.search_snipe_payback` with range 0–1 and built-in default 0. At 1, a Snipe
candidate receives an off-horizon Search leaf credit equal to 90% of the speed-scaled base
Settler Production cost. The 90% factor matches the existing discount on the Snipe plan's
projected city effect. The target must be a visible enemy Settler in the searching seat's
`PlayerView`; missing or hidden targets receive no credit. No game rule, game data, default
behavior or pin changed. The supervisor decides whether to enable the opt-in gene.

## Divergence check

The 8.8 diagnosis identified salts 2, 25 and 27 as exposed-Settler Snipe cases on
`playtest-8.4-japan-win`. Focused post-change traces used the cherry-picked T180 trace runner and
the gene-on seat-only configuration. The leaf credit was 135 Standard Production points in all
three traces. Search changed its pick on salt 2, but all three games remained losses:

| Salt | Before: Economy / Snipe total | After: Economy / Snipe total | Search pick after | Search result before → after |
| ---: | ---: | ---: | --- | --- |
| 2 | 1037 / 1012 | 1037 / 1147 | Snipe | Loss → Loss |
| 25 | 1269 / 848 | 1269 / 983 | Economy | Loss → Loss |
| 27 | 1236 / 988 | 1236 / 1123 | Economy | Loss → Loss |

Salt 25 still trails by 286 and salt 27 by 113 after payback. The complete trace comparison is
in [`runs/search-snipe-payback-traces.json`](runs/search-snipe-payback-traces.json).

All 12 qualifying salts from the original report also remained losses for the Search seat:

| Salt | 1 | 2 | 6 | 13 | 19 | 22 | 24 | 25 | 27 | 30 | 34 | 41 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Search, gene off → on | L → L | L → L | L → L | L → L | L → L | L → L | L → L | L → L | L → L | L → L | L → L | L → L |

Per-salt outcomes are read from the archived 8.8 control and the candidate seat-only games on
the same map and salts. The original Hard seat won all 12, as recorded in
[`search-divergence-88.md`](search-divergence-88.md).

## Eight-map benchmark

The benchmark used Hard + Search, all eight human maps, salts 1–64, and the same candidate Linux
ARM binary for seat-only and opponents-only. The archived 8.8 Hard + Search control was reused:
its game sources match the pre-change branch at `5de2c379`, and the new gene's default-off path
is covered by the focused ranking test. Each candidate arm ran on AWS with
`AWS_PROFILE=<launcher-profile> tools/cloud/run-batch.sh --max-hours 1 --bin hard_vs_human`;
`tools/cloud/status.sh` ran after each batch and reported no sim servers running.

| Arm | Control wins | Candidate wins | Candidate minus control, 95% paired CI | Improved / worsened salt outcomes |
| --- | ---: | ---: | ---: | ---: |
| Seat-only | 111/512 (21.68%) | 110/512 (21.48%) | −0.20 pp [−1.56, +1.37] pp | 7 / 8 |
| Opponents-only | 111/512 (21.68%) | 113/512 (22.07%) | +0.39 pp [−1.56, +2.34] pp | 14 / 12 |

**Seat-only gate: fail.** The lower bound, −1.56 pp, is above the −2 pp floor, but the point
estimate is negative; the required positive point estimate was not met. The code default remains
0.

Per-map wins (control / seat-only / opponents-only):

| Map | Control | Seat-only | Opponents-only |
| --- | ---: | ---: | ---: |
| 8.3 Switzerland | 1/64 | 1/64 | 2/64 |
| 8.4 Sparta | 35/64 | 35/64 | 35/64 |
| 8.4 Japan | 18/64 | 18/64 | 17/64 |
| 8.4 Celts | 10/64 | 11/64 | 9/64 |
| 8.5 Mongols | 0/64 | 0/64 | 0/64 |
| 8.5 Switzerland | 19/64 | 18/64 | 21/64 |
| 8.6 Germany | 14/64 | 13/64 | 13/64 |
| 8.6 Sparta | 14/64 | 14/64 | 16/64 |

The paired intervals use the project comparator's 10,000 salt-bootstrap draws, resampling each
map's salt outcomes and giving the eight maps equal weight. They describe variation on these
fixed maps, not uncertainty across maps.

## Provenance and artifacts

The candidate binary SHA-256 is
`940b178e543e7c8bcda62ad407a013f9ea0a2659147e500a1c1066b6a28ff2d9` (Linux ARM, sim profile;
source `6fb87711` plus the worktree changes). The control is the archived `5de2c379` report.

- Seat-only report and games: [`runs/search-snipe-payback-seat-only/`](runs/search-snipe-payback-seat-only/)
- Opponents-only report and games: [`runs/search-snipe-payback-opponents-only/`](runs/search-snipe-payback-opponents-only/)
- Candidate parameter file: [`params/search-snipe-payback-on.json`](params/search-snipe-payback-on.json)
- Original divergence evidence: [`search-divergence-88.md`](search-divergence-88.md)

## Verification

`cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings` passed. The
focused `search_snipe_payback_changes_only_the_leaf_ranking_when_enabled` test passed, and
`cargo test --workspace --release` passed using
`CARGO_TARGET_DIR=/tmp/legatum-search-snipe-release-target` because Cargo could not write an
existing dependency file under the worktree's `target/release`. `tools/sim/repin.sh` passed before the master merge and was rerun after the merge. The post-merge
run passed the repin runner, both pinned-tape tests, and all 13 determinism tests. All fixture and
tape hashes were unchanged.
