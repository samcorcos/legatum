# Recent retake campaign hold — 2026-10-31

**Verdict: keep `retake_hold` off by default.** Five turns reduces campaign
start/end flaps, but the seat-only strength gate is inconclusive under the AI
plan's sparse-discordance rule. The opponents-only arm is noninferior. This is
useful diagnostic evidence, not a demonstrated win-rate gain.

## Paired gate

- One binary from `8d7e01a00ab2`, SHA-256 prefix `944caaa2597ae5f8`;
  control and candidate arms differ only in `retake_hold=5`. Plain Hard rivals,
  `LEGATUM_HVH_RESEED_DICE=1`, eight fixed maps, salts 310001–310256.
- Batch: `~/LegatumRuns/cloud/retake-hold-1031-1031-222653`;
  four `c8g.16xlarge` Spot slices, 0.11 server-hours, about $0.08 at the
  launch-time rate (the AWS bill is authoritative). Each arm has 2,048 games.
- The paired estimator in `tools/ai-quality/w04_gate.py` uses the larger map or
  salt-cluster standard error, with a two-arm simultaneous interval. Positive
  strength means stronger AI; the opponents role reverses the raw seat-win sign.

| Role | Control seat wins | Gene seat wins | AI strength effect | Adjusted interval | Gate |
| --- | ---: | ---: | ---: | ---: | --- |
| Seat only | 524/2,048 | 523/2,048 | −0.05 pp | unresolved (13 discordant games) | inconclusive |
| Opponents only | 524/2,048 | 524/2,048 | 0.00 pp | [−0.93, +0.93] pp | noninferior |

The seat-only arm's descriptive count is one fewer win, with eight maps moving
in both directions; its interval is deliberately unbounded by the gate because
13 discordant games are below the protocol's 20-game variance floor. The
opponents arm has 72 discordant games. There is no clear harm, but the planned
AI gate requires a resolved result before switching the default on.

## Campaign diagnostics

`hard_vs_human` records these counters per AI per game. A flap is a campaign
end paired with its most recent start on the same city within three turns.
Retake success means that AI recaptured one of its lost cities within 20 turns;
the denominator includes all city losses, including those late in a game.

| Affected role | Gene | Flaps per game | Total flaps | Retakes / losses | Retake rate |
| --- | --- | ---: | ---: | ---: | ---: |
| Seat | off | 6.011 | 12,310 | 787 / 5,598 | 14.1% |
| Seat | 5 | 5.850 | 11,980 | 771 / 5,586 | 13.8% |
| Opponents | off | 17.054 | 34,926 | 2,160 / 17,967 | 12.0% |
| Opponents | 5 | 16.640 | 34,078 | 2,159 / 17,896 | 12.1% |

Seat flaps fell 2.7%; opponents flaps fell 2.4%. The retake-rate changes are
small and the raw denominators move with the games, so these counts do not show
that the hold itself wins more retakes.

## Verification and pins

`tools/cloud/verify-remote.sh astra/retake-hold --profile extended` passed on
the formatted code: merge, rustfmt, workspace clippy with warnings denied,
2,103 workspace tests passed (9 skipped), determinism repin, and the 50-game
branch smoke (zero panics or stalls). No determinism pins or fixtures moved.

Reproduce:

```sh
LEGATUM_CLOUD_EXAMPLES=hard_vs_human tools/cloud/build-linux.sh
tools/cloud/run-batch.sh --count 4 --align 64 --max-hours 2 --bin hard_vs_human --ai-params docs/sim-results/ai-quality/retake-hold-1031/arms.json --env LEGATUM_HVH_RESEED_DICE=1 -- --games 256 --seed 310001 --label retake-hold-1031 --arms
python3 tools/ai-quality/w04_gate.py docs/sim-results/ai-quality/retake-hold-1031/gate BATCH_DIR --family 2
python3 docs/sim-results/ai-quality/retake-hold-1031/metrics.py BATCH_DIR
```
