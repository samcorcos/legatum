# Air and Gold AI gene gate, 2026-10-26

The merged source starts from `06c260e6e` (master) and incorporates
`astra/analyze-japan-air-war-1026` and `astra/gold-usage-diagnosis`. The
`air_defense` gene already makes Fighter procurement respond to public enemy
Flight-era bases and exposed own assets without requiring any owned Bombers;
the strategy-lab finding does not require another gene. Rival hangars are
hidden, so it does not read actual rival Bomber inventory.

## Protocol

`tools/ai-quality/gate-air-gold-1026/arms.json` defines one default-off Hard
control and each individual gene plus the four-gene bundle, once for all seats
and once for the measured seat only. The same eight fixed human-seat maps use
128 AI salts each, with `LEGATUM_HVH_RESEED_DICE=1` for every arm. The all-seat
and seat-only arms share the same binary, control games, map/salt identities,
and game-dice schedule. Each arm has 1,024 games. Paired 95% intervals use a
stratified bootstrap of the 128 matched salts within each map, including for
air-stage differences. These are fixed-map intervals, not estimates over new
maps. Air-stage figures are descriptive per AI that reached Flight; arm changes
can change that denominator and game length.

The build used `tools/cloud/build-linux.sh` with the `hard_vs_human` example;
the batch used `tools/cloud/run-batch.sh` on four spot servers, 32 salts each.
The smoke batch (two salts per map) completed all 11 arms and populated the
Fighter and interception counters before the full gate was launched.

## Results

Control: 399/1,024 measured-seat wins (39.0%). Each delta is candidate minus
that one control. Air measures use all AI seats for all-seat arms, and only the
changed AI seat for seat-only arms. A Fighter is counted when construction or
a purchase completes; an interception is a Bomber strike stopped by a Fighter
in the air battle. The numbers are per AI that reached Flight. Interceptions
on the measured seat are zero in both control and every seat-only arm; those
rows cannot resolve an interception effect on that seat.

| Arm | Win delta, pp [95%] | Flight AIs | Fighters / Flight AI, before → after [delta 95%] | Interceptions / Flight AI, before → after [delta 95%] |
| --- | ---: | ---: | ---: | ---: |
| `air_defense_all` | -13.1 [-15.4, -10.6] | 3435 → 3447 | 1.25 → 3.09 [+1.74, +1.93] | 0.22 → 0.94 [+0.63, +0.83] |
| `air_commitment_all` | +1.2 [-1.1, +3.5] | 3435 → 3441 | 1.25 → 1.32 [-0.00, +0.14] | 0.22 → 0.31 [+0.02, +0.15] |
| `air_rebase_all` | +4.0 [+2.0, +6.1] | 3435 → 3442 | 1.25 → 1.23 [-0.09, +0.04] | 0.22 → 0.31 [+0.02, +0.15] |
| `purchase_savings_turns_all` | -2.8 [-6.0, +0.5] | 3435 → 3452 | 1.25 → 1.23 [-0.12, +0.09] | 0.22 → 0.23 [-0.06, +0.07] |
| `bundle_all` | -13.0 [-16.1, -9.8] | 3435 → 3469 | 1.25 → 3.53 [+2.15, +2.40] | 0.22 → 1.64 [+1.28, +1.58] |
| `air_defense_seat` | -4.8 [-6.9, -2.5] | 839 → 839 | 1.59 → 3.27 [+1.54, +1.81] | 0.00 → 0.00 [+0.00, +0.00] |
| `air_commitment_seat` | +1.5 [-0.5, +3.5] | 839 → 839 | 1.59 → 1.66 [-0.04, +0.17] | 0.00 → 0.00 [+0.00, +0.00] |
| `air_rebase_seat` | +3.7 [+1.7, +5.8] | 839 → 839 | 1.59 → 1.84 [+0.13, +0.34] | 0.00 → 0.00 [+0.00, +0.00] |
| `purchase_savings_turns_seat` | -0.6 [-3.2, +2.1] | 839 → 842 | 1.59 → 1.60 [-0.16, +0.15] | 0.00 → 0.00 [+0.00, +0.00] |
| `bundle_seat` | +2.1 [-0.6, +4.8] | 839 → 842 | 1.59 → 3.56 [+1.78, +2.15] | 0.00 → 0.00 [+0.00, +0.00] |

**Decision:** enable `air_rebase=1` by default. Its paired win intervals are
positive for both seat-only and all-seats arms. Keep `air_defense=0`: it builds
and intercepts more, but loses substantially more games. Keep
`air_commitment=0`: its positive point estimates do not resolve from zero at
95%. Keep `purchase_savings_turns=0`: the measured-seat point estimate is
negative and the all-seats estimate is also negative, with both intervals
crossing zero. The combined four-gene arm loses 13.0 pp with all seats; it
does not justify enabling genes that failed individually. These are pointwise
intervals across a screened family, so the exact intervals are descriptive
rather than a familywise guarantee.

Full batch: `~/LegatumRuns/cloud/air-gold-gate-1026-045724` (four successful
slices, 2.74 server-hours, about $1.80 at the launch-time spot price). The
benchmark source was `7909a646e370fbfdde599fac144d1a6a89154e59`, the
Linux ARM binary SHA-256 was
`ce5c3832d54fc143197489316923eb6f0726c6f69c94acabc645da17cc4f1988`,
and the arms file SHA-256 was
`57fca4f6762c9428db29adf77a4161986005dae390e911044c3adb9c7d768829`.
The report script validated continuous salt slices and matching maps before
calculating the table.

## Checks

After setting `air_rebase=1`, `tools/sim/repin.sh` passed and returned no
fixture or pin changes: its ignored repin test passed, both pin assertions
passed, and all 12 determinism tests passed. `cargo fmt --all` and
`cargo clippy --workspace --all-targets -- -D warnings` passed. The workspace
`release-checked` test pass had **1,969/1,969 passed**, 10 skipped, and all
doctests passed. Cargo checks ran through `tools/sim/build-slot.sh --` on the
cloud box. The focused air tests passed 14/14 and the Gold savings fixture
passed 1/1 before the default change.

`tools/sim/war-scenarios.sh --check --runs 128` passed the thin-treasury
Golden Horde attack regression: 79/128 goals met (the suite reports 61%),
above the 30% floor, with zero reported bugs. The cloud artifact is
`~/LegatumRuns/cloud/war-hard-1026-055102`.
